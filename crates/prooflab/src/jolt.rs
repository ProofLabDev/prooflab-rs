use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    process::{Command, ExitStatus},
    time::Duration,
};

use crate::utils;

#[derive(Default, Serialize, Deserialize)]
pub struct JoltMetrics {
    pub cycles: u64,
    pub num_segments: usize,
    pub core_proof_size: usize,
    pub recursive_proof_size: usize,
    pub core_prove_duration: Duration,
    pub core_verify_duration: Duration,
    pub compress_prove_duration: Duration,
    pub compress_verify_duration: Duration,
}

/// Jolt workspace directories
pub const JOLT_SCRIPT_DIR: &str = "workspaces/jolt/script";
pub const JOLT_SRC_DIR: &str = "workspaces/jolt/program";
pub const JOLT_GUEST_MAIN: &str = "workspaces/jolt/program/src/lib.rs";
pub const JOLT_HOST_MAIN: &str = "workspaces/jolt/script/src/main.rs";
pub const JOLT_BASE_GUEST_CARGO_TOML: &str = "workspaces/base_files/jolt/cargo_guest";
pub const JOLT_BASE_HOST_CARGO_TOML: &str = "workspaces/base_files/jolt/cargo_host";
pub const JOLT_BASE_HOST: &str = "workspaces/base_files/jolt/host";
pub const JOLT_BASE_HOST_FILE: &str = "workspaces/base_files/jolt/host";
pub const JOLT_GUEST_CARGO_TOML: &str = "workspaces/jolt/program/Cargo.toml";

// Proof data generation paths
pub const JOLT_ELF_PATH: &str = "./proof_data/jolt/jolt.elf";
pub const JOLT_PROOF_PATH: &str = "./proof_data/jolt/jolt.proof";
pub const JOLT_PUB_INPUT_PATH: &str = "./proof_data/jolt/jolt.pub";
pub const JOLT_METRICS_PATH: &str = "./proof_data/jolt/jolt_metrics.json";

/// Jolt header added to programs for generating proofs of their execution
pub const JOLT_GUEST_PROGRAM_HEADER: &str = "#![cfg_attr(feature = \"guest\", no_std)]\n#![no_main]\n";

/// Jolt Cargo patch for optimization (if needed in the future)
pub const JOLT_ACCELERATION_IMPORT: &str = "\n[patch.crates-io]\n# Jolt-specific optimized crates would go here\n";

/// Jolt User I/O
// Host
pub const JOLT_HOST_WRITE: &str = "input";
pub const JOLT_HOST_READ: &str = "output";

// Guest
pub const JOLT_IO_READ: &str = "jolt::io::read();";
pub const JOLT_IO_COMMIT: &str = "jolt::io::commit";

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

    // replace prooflab_io::write
    let host_program = host_program.replace(utils::IO_WRITE, JOLT_HOST_WRITE);
    // replace prooflab_io::out()
    let host_program = host_program.replace(utils::IO_OUT, JOLT_HOST_READ);

    // Write to host
    let mut file = fs::File::create(host_main)?;
    file.write_all(host_program.as_bytes())?;
    Ok(())
}

/// Build the Jolt program
pub fn build_jolt_program(script_dir: &PathBuf) -> io::Result<ExitStatus> {
    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(script_dir)
        .status()
}

/// Generates Jolt proof
pub fn generate_jolt_proof(
    script_dir: &PathBuf,
    current_dir: &PathBuf,
    use_gpu: bool,
) -> io::Result<ExitStatus> {
    let mut cmd = Command::new("cargo");
    cmd.arg("run").arg("--release");

    if use_gpu {
        cmd.arg("--features").arg("gpu");
        cmd.env("JOLT_GPU", "1");
    }

    cmd.arg("--")
        .arg(current_dir)
        .current_dir(script_dir)
        .status()
}

pub fn read_metrics() -> io::Result<JoltMetrics> {
    let metrics_str = fs::read_to_string(JOLT_METRICS_PATH)?;
    serde_json::from_str(&metrics_str).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}