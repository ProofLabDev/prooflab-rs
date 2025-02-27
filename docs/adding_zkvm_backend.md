# How to Add a New zkVM Backend to prooflab-rs

This guide explains the process of adding a new Zero-Knowledge Virtual Machine (zkVM) backend to the prooflab-rs framework, allowing it to work alongside existing RISC0 and SP1 implementations.

## 1. Create the Backend Module

First, create a new module file for your zkVM in the `src` directory:

```rust
// src/your_zkvm.rs
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
pub struct YourZkVMMetrics {
    pub cycles: u64,
    pub num_segments: usize,
    pub core_proof_size: usize,
    pub recursive_proof_size: usize,
    pub core_prove_duration: Duration,
    pub core_verify_duration: Duration,
    pub compress_prove_duration: Duration,
    pub compress_verify_duration: Duration,
}

// Define all zkVM-specific constants
pub const WORKSPACE_DIR: &str = "workspaces/your_zkvm/";
pub const SRC_DIR: &str = "workspaces/your_zkvm/program";
pub const GUEST_MAIN: &str = "workspaces/your_zkvm/program/src/main.rs";
pub const HOST_MAIN: &str = "workspaces/your_zkvm/script/src/main.rs";
pub const BASE_HOST_CARGO_TOML: &str = "workspaces/base_files/your_zkvm/cargo_host";
pub const BASE_GUEST_CARGO_TOML: &str = "workspaces/base_files/your_zkvm/cargo_guest";
pub const BASE_HOST: &str = "workspaces/base_files/your_zkvm/host";
pub const BASE_HOST_FILE: &str = "workspaces/base_files/your_zkvm/host";
pub const GUEST_CARGO_TOML: &str = "workspaces/your_zkvm/program/Cargo.toml";

// Proof data generation paths
pub const ELF_PATH: &str = "./proof_data/your_zkvm/your_zkvm.elf";
pub const PROOF_PATH: &str = "./proof_data/your_zkvm/your_zkvm.proof";
pub const PUB_INPUT_PATH: &str = "./proof_data/your_zkvm/your_zkvm.pub";
pub const METRICS_PATH: &str = "./proof_data/your_zkvm/your_zkvm_metrics.json";

// Guest program header for your zkVM
pub const GUEST_PROGRAM_HEADER: &str = "#![no_main]\nyour_zkvm::entrypoint!(main);\n";

// Optional acceleration patch for improved performance
pub const ACCELERATION_IMPORT: &str = "\n[patch.crates-io]\n# Your zkVM-specific optimized crates";

// I/O interface constants
// Guest
pub const IO_READ: &str = "your_zkvm::io::read();";
pub const IO_COMMIT: &str = "your_zkvm::io::commit";

// Host
pub const HOST_WRITE: &str = "stdin.write";
pub const HOST_READ: &str = "proof.public_values.read();";

// Implement host preparation function
pub fn prepare_host(
    input: &str,
    output: &str,
    imports: &str,
    host_dir: &PathBuf,
    host_main: &PathBuf,
) -> io::Result<()> {
    // Implement zkVM-specific host preparation
    let mut host_program = imports.to_string();
    let contents = fs::read_to_string(host_dir)?;

    host_program.push_str(&contents);

    // Insert input body
    let host_program = host_program.replace(utils::HOST_INPUT, input);
    // Insert output body
    let host_program = host_program.replace(utils::HOST_OUTPUT, output);

    // Replace I/O operations with zkVM-specific ones
    let host_program = host_program.replace(utils::IO_WRITE, HOST_WRITE);
    let host_program = host_program.replace(utils::IO_OUT, HOST_READ);

    // Write to host
    let mut file = fs::File::create(host_main)?;
    file.write_all(host_program.as_bytes())?;
    Ok(())
}

// Implement zkVM build function
pub fn build_program(script_dir: &PathBuf) -> io::Result<ExitStatus> {
    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(script_dir)
        .status()
}

// Implement proof generation function
pub fn generate_proof(
    script_dir: &PathBuf,
    current_dir: &PathBuf,
    use_gpu: bool,
) -> io::Result<ExitStatus> {
    let mut cmd = Command::new("cargo");
    cmd.arg("run").arg("--release");

    if use_gpu {
        cmd.arg("--features").arg("your_gpu_feature");
        cmd.env("YOUR_ZKVM_PROVER", "gpu");
    }

    cmd.arg("--")
        .arg(current_dir)
        .current_dir(script_dir)
        .status()
}

// Implement metrics reading function
pub fn read_metrics() -> io::Result<YourZkVMMetrics> {
    let metrics_str = fs::read_to_string(METRICS_PATH)?;
    serde_json::from_str(&metrics_str).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
```

## 2. Update the Library Interface

Add your module to `src/lib.rs`:

```rust
pub mod risc0;
pub mod sp1;
pub mod your_zkvm;  // Add your new module
pub mod telemetry;
pub mod utils;
```

## 3. Update the CLI

Modify `src/main.rs` to add a new subcommand for your zkVM:

```rust
#[derive(Subcommand)]
enum Commands {
    #[clap(about = "Generate a proof of execution of a program using SP1")]
    ProveSp1(ProofArgs),
    #[clap(about = "Generate a proof of execution of a program using RISC0")]
    ProveRisc0(ProofArgs),
    #[clap(about = "Generate a proof of execution of a program using YourZkVM")]
    ProveYourZkVM(ProofArgs),  // Add new command
}
```

## 4. Add Command Handler Logic

Add a match arm in `main()` function to handle your new command:

```rust
match &cli.command {
    Commands::ProveSp1(args) => {
        // SP1 logic...
    }
    Commands::ProveRisc0(args) => {
        // RISC0 logic...
    }
    Commands::ProveYourZkVM(args) => {
        info!("Proving with YourZkVM, program in: {}", args.guest_path);

        let telemetry = TelemetryCollector::new(
            "YOUR_ZKVM",
            args.precompiles,
            args.gpu,
            args.enable_telemetry,
            &args.guest_path,
        );
        let workspace_start = Instant::now();

        // Perform sanitation checks on directory
        let proof_data_dir = PathBuf::from(&args.proof_data_directory_path);
        if !proof_data_dir.exists() {
            info!("Saving Proofs to: {:?}", &args.proof_data_directory_path);
            std::fs::create_dir_all(proof_data_dir)?;
        }
        
        // Add the rest of your zkVM-specific logic
        // Similar to SP1 and RISC0 implementations...
        
        // Example structure:
        // 1. Validate directory structure
        // 2. Prepare workspace
        // 3. Extract main, input, output functions
        // 4. Prepare guest and host code
        // 5. Build the program
        // 6. Generate proof
        // 7. Submit to Aligned if requested
        // 8. Save telemetry data
    }
}
```

## 5. Create Base Files

Create base files needed by your zkVM:

```
workspaces/
└── base_files/
    └── your_zkvm/
        ├── cargo_host       # Template for host Cargo.toml
        ├── cargo_guest      # Template for guest Cargo.toml
        └── host             # Template for host code
```

## 6. Update Aligned Integration

If submitting proofs to Aligned, update the `ProvingSystemId` enum to include your zkVM:

```rust
// In aligned_sdk or local wrapper
enum ProvingSystemId {
    Risc0,
    SP1,
    YourZkVM,  // Add your zkVM
}
```

## 7. Documentation

Update the README.md to include instructions for your new zkVM:

```markdown
## Running with YourZkVM
```cargo run --release -- prove-your-zkvm <PROGRAM_DIRECTORY_PATH>```
```

## 8. Testing

Create a simple example program to test your zkVM implementation:

```
examples/
└── your_zkvm_example/
    ├── Cargo.toml
    └── src/
        └── main.rs
```

## 9. Makefile

Add targets for your zkVM in the Makefile:

```makefile
prove_your_zkvm_example:
	cargo run --release -- prove-your-zkvm examples/your_zkvm_example
```

## 10. Integration Details

For complete integration, you'll need to understand several key aspects:

1. **Workspace Structure**: prooflab-rs uses a multi-workspace approach to separate guest (zkVM program) and host (verifier/prover) code.

2. **Function Extraction**: The framework extracts user's `main()`, `input()`, and `output()` functions and wraps them with zkVM-specific code.

3. **I/O Abstraction**: The `prooflab_io` module provides a consistent I/O interface that is mapped to zkVM-specific I/O operations.

4. **Telemetry Collection**: The framework collects performance metrics during proof generation.

5. **Aligned Integration**: The framework can submit proofs to the Aligned Layer system.

Study the existing SP1 and RISC0 implementations in detail to understand how they implement these aspects, and follow the same patterns for your zkVM.