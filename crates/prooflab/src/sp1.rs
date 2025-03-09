use regex;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    process::{Command, ExitStatus},
    time::Duration,
};
use tracing::info;

use crate::utils;

#[derive(Default, Serialize, Deserialize)]
pub struct SP1Metrics {
    pub cycles: u64,
    pub num_segments: usize,
    pub core_proof_size: usize,
    pub recursive_proof_size: usize,
    pub core_prove_duration: Duration,
    pub core_verify_duration: Duration,
    pub compress_prove_duration: Duration,
    pub compress_verify_duration: Duration,
    pub input_size: usize,
    pub output_size: usize,
}

/// SP1 workspace directories
pub const SP1_SCRIPT_DIR: &str = "workspaces/sp1/script";
pub const SP1_SRC_DIR: &str = "workspaces/sp1/program";
pub const SP1_GUEST_MAIN: &str = "workspaces/sp1/program/src/main.rs";
pub const SP1_HOST_MAIN: &str = "workspaces/sp1/script/src/main.rs";
pub const SP1_BASE_GUEST_CARGO_TOML: &str = "workspaces/base_files/sp1/cargo_guest";
pub const SP1_BASE_HOST_CARGO_TOML: &str = "workspaces/base_files/sp1/cargo_host";
pub const SP1_BASE_HOST: &str = "workspaces/base_files/sp1/host";
pub const SP1_BASE_HOST_FILE: &str = "workspaces/base_files/sp1/host";
pub const SP1_GUEST_CARGO_TOML: &str = "workspaces/sp1/program/Cargo.toml";

// Proof data generation paths
pub const SP1_ELF_PATH: &str = "./proof_data/sp1/sp1.elf";
pub const SP1_PROOF_PATH: &str = "./proof_data/sp1/sp1.proof";
pub const SP1_PUB_INPUT_PATH: &str = "./proof_data/sp1/sp1.pub";
pub const SP1_METRICS_PATH: &str = "./proof_data/sp1/sp1_metrics.json";

/// SP1 header added to programs for generating proofs of their execution
pub const SP1_GUEST_PROGRAM_HEADER: &str = "#![no_main]\nsp1_zkvm::entrypoint!(main);\n";

/// SP1 Cargo patch for accelerated SHA-256, K256, and bigint-multiplication circuits
pub const SP1_ACCELERATION_IMPORT: &str = "\n[patch.crates-io]\nsha2 = { git = \"https://github.com/sp1-patches/RustCrypto-hashes\", package = \"sha2\", branch = \"patch-sha2-v0.10.8\" }\nsha3 = { git = \"https://github.com/sp1-patches/RustCrypto-hashes\", package = \"sha3\", branch = \"patch-sha3-v0.10.8\" }\ncrypto-bigint = { git = \"https://github.com/sp1-patches/RustCrypto-bigint\", branch = \"patch-v0.5.5\" }\ntiny-keccak = { git = \"https://github.com/sp1-patches/tiny-keccak\", branch = \"patch-v2.0.2\" }\ned25519-consensus = { git = \"https://github.com/sp1-patches/ed25519-consensus\", branch = \"patch-v2.1.0\" }\necdsa-core = { git = \"https://github.com/sp1-patches/signatures\", package = \"ecdsa\", branch = \"patch-ecdsa-v0.16.9\" }\n";

/// SP1 User I/O
// Host
pub const SP1_HOST_WRITE: &str = "stdin.write";
pub const SP1_HOST_READ: &str = "proof.public_values.read();";

// Guest
pub const SP1_IO_READ: &str = "sp1_zkvm::io::read();";
pub const SP1_IO_COMMIT: &str = "sp1_zkvm::io::commit";

pub fn prepare_host(
    input: &str,
    output: &str,
    imports: &str,
    host_dir: &PathBuf,
    host_main: &PathBuf,
) -> io::Result<()> {
    let mut host_program = imports.to_string();
    let contents = fs::read_to_string(host_dir)?;

    host_program.push_str(&contents);

    // Insert input body
    let host_program = host_program.replace(utils::HOST_INPUT, input);
    // Insert output body
    let host_program = host_program.replace(utils::HOST_OUTPUT, output);

    // Initialize input size tracking
    let mut new_host_program = host_program.replace(
        "let mut metrics = SP1Metrics::default();",
        "let mut metrics = SP1Metrics::default();\nlet mut total_input_size = 0;",
    );

    // Read the entire file content to find all prooflab_io::write calls
    let file_content = fs::read_to_string(host_main)?;

    // Process each prooflab_io::write call
    let write_regex =
        regex::Regex::new(&format!("{}\\((.*?)\\);", regex::escape(utils::IO_WRITE))).unwrap();
    for capture in write_regex.captures_iter(&file_content) {
        if let Some(value_match) = capture.get(1) {
            let value = value_match.as_str();
            let old_write = format!("{}({});", utils::IO_WRITE, value);

            // Convert each write call to SP1 format appropriately
            let new_write = if value.starts_with('&') {
                // Handle reference values like &something or &something.method()
                format!(
                    "{{\n    let serialized = bincode::serialize({}).unwrap();\n    total_input_size += serialized.len();\n    {}({});\n}}",
                    value, SP1_HOST_WRITE, value
                )
            } else {
                // Handle non-reference values
                format!(
                    "{{\n    let serialized = bincode::serialize(&{}).unwrap();\n    total_input_size += serialized.len();\n    {}(&{});\n}}",
                    value, SP1_HOST_WRITE, value
                )
            };

            new_host_program = new_host_program.replace(&old_write, &new_write);
        }
    }

    // Add code to record the input size before running the prover
    new_host_program = new_host_program.replace(
        "// Setup the prover",
        "metrics.input_size = total_input_size;\n\n// Setup the prover",
    );

    // Add code to record the output size after getting the proof
    new_host_program = new_host_program.replace(
        "// Save proof artifacts",
        "metrics.output_size = compressed.public_values.to_vec().len();\n\n// Save proof artifacts",
    );

    // replace prooflab_io::out()
    new_host_program = new_host_program.replace(utils::IO_OUT, SP1_HOST_READ);

    // Write to host
    let mut file = fs::File::create(host_main)?;
    file.write_all(new_host_program.as_bytes())?;
    Ok(())
}

/// Build the SP1 program
pub fn build_sp1_program(script_dir: &PathBuf) -> io::Result<ExitStatus> {
    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(script_dir)
        .status()
}

/// Generates SP1 proof and ELF using pre-built artifacts
pub fn generate_sp1_proof(
    script_dir: &PathBuf,
    current_dir: &PathBuf,
    use_gpu: bool,
) -> io::Result<ExitStatus> {
    let mut cmd = Command::new("cargo");
    cmd.arg("run").arg("--release");

    if use_gpu {
        cmd.arg("--features").arg("cuda");
        cmd.env("SP1_PROVER", "cuda");
    }

    cmd.arg("--")
        .arg(current_dir)
        .current_dir(script_dir)
        .status()
}

pub fn read_metrics() -> io::Result<SP1Metrics> {
    let metrics_str = fs::read_to_string(SP1_METRICS_PATH)?;
    let metrics = serde_json::from_str(&metrics_str)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    // Log the raw metrics for debugging
    info!("Read SP1 metrics: {:?}", metrics_str);

    Ok(metrics)
}
