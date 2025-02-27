# RFC: Unified Macro API for ProofLab-rs

## Summary

This RFC proposes a comprehensive macro-based API for ProofLab-rs that simplifies zkVM development through a unified codebase approach, while respecting the underlying architectural constraints of zkVMs. The proposed system uses declarative struct annotations for inputs and outputs, and function annotations for guest and host code, combined with build-time code generation to handle the separate compilation targets required by zkVMs.

## Motivation

Current zkVM development faces several significant challenges:

1. **Separate Compilation Targets**: Guest code targets specialized architectures (RISC-V for RISC0, custom VM instruction sets for SP1), while host code targets native architectures (x86_64, aarch64). This requires maintaining separate crates with different compilation targets.

2. **Complex Build Management**: Managing build pipelines for different targets, including precompiling guest code before host compilation.

3. **Duplicate Dependencies**: Maintaining separate dependency sets for host and guest, including "precompile" accelerated versions of libraries.

4. **Limited Type Safety**: Poor type safety across the host/guest boundary when using string-based extraction methods.

5. **Verbose I/O Code**: Repetitive, error-prone I/O code that distracts from core logic.

A unified macro-based approach would address these issues by:

1. Providing a single-crate development experience while handling the separate compilation targets behind the scenes
2. Creating clear, type-safe data structures for I/O
3. Automating dependency management across guest and host environments
4. Reducing boilerplate through a declarative API
5. Enabling full IDE support with compile-time checks

## Key Architectural Insight

The fundamental constraint in zkVM development is that **host and guest code must be compiled for different target architectures**. Any solution must respect this constraint while providing a simplified developer experience.

Our approach acknowledges this constraint by using build-time code generation to create and manage separate crates, while presenting a unified API to the developer.

## API Design

The new API centers around four primary annotation types:

```rust
// Define input data structure
#[zkvm_input]
struct Input {
    value: u32,
}

// Define output data structure
#[zkvm_output]
struct Output {
    result: u64,
}

// Define guest code (runs inside zkVM)
#[zkvm_guest]
fn guest() {
    let input: Input = prooflab_io::read();
    let result = input.value as u64 * 2;
    prooflab_io::commit(&Output { result });
}

// Define host code (runs prover)
#[zkvm_host]
fn host() {
    // Set up inputs
    let input = Input { value: 42 };
    
    // Run prover and get output
    let output: Output = prooflab::prove(input);
    
    // Use output
    println!("Result: {}", output.result);
}
```

## Implementation Architecture

The implementation consists of these core components:

### 1. Procedural Macros

- `#[zkvm_input]`: Marks a struct as input data
- `#[zkvm_output]`: Marks a struct as output data
- `#[zkvm_guest]`: Marks a function containing guest code
- `#[zkvm_host]`: Marks a function containing host code

### 2. Build-Time Code Generator

- Analyzes annotated source code
- Generates separate guest and host crates
- Manages dependencies for each target
- Sets up build pipeline for different architectures

### 3. Runtime Libraries

- `prooflab_io`: Common I/O abstraction for guest code
- `prooflab_host`: Host-side utilities for prover interaction

## Build and Execution Flow

The build process works as follows:

```
┌──────────────────┐
│ Source Code with │
│   Annotations    │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  prooflab-build  │
│  (build script)  │
└────────┬─────────┘
         │
         ├───────────────┐
         │               │
         ▼               ▼
┌──────────────────┐ ┌───────────────────┐
│  Guest Crate     │ │  Host Crate       │
│  Generation      │ │  Generation       │
└────────┬─────────┘ └───────────┬───────┘
         │                       │
         ▼                       │
┌──────────────────┐             │
│ Guest Compilation│             │
│ (RISC-V target)  │             │
└────────┬─────────┘             │
         │                       │
         │                       ▼
         │           ┌───────────────────┐
         └──────────►│ Host Compilation  │
                     │ (Native target)   │
                     └───────────┬───────┘
                                 │
                                 ▼
                     ┌───────────────────┐
                     │  Final Binary     │
                     │  with embedded    │
                     │  Guest ELF        │
                     └───────────────────┘
```

## Generated Code

### Generated Guest Crate

```rust
// In target/prooflab/guest/src/lib.rs
use serde::{Serialize, Deserialize};

#[derive(Deserialize)]
struct Input {
    value: u32,
}

#[derive(Serialize)]
struct Output {
    result: u64,
}

// For SP1
#[sp1_zkvm::entrypoint]
fn main() {
    // The user's guest code with I/O replacements
    let input: Input = sp1_zkvm::io::read();
    let result = input.value as u64 * 2;
    sp1_zkvm::io::commit(&Output { result });
}
```

### Generated Host Crate

```rust
// In target/prooflab/host/src/main.rs
use serde::{Serialize, Deserialize};

#[derive(Serialize)]
struct Input {
    value: u32,
}

#[derive(Deserialize)]
struct Output {
    result: u64,
}

// Embed the guest ELF 
const GUEST_ELF: &[u8] = include_bytes!("../../guest/target/riscv32im-succinct-zkvm-elf/release/guest");

fn main() {
    // The user's host code
    let input = Input { value: 42 };
    
    // Create and run prover
    #[cfg(feature = "sp1")]
    let mut prover = sp1_core::SP1Prover::new_from_elf(GUEST_ELF).unwrap();
    #[cfg(feature = "risc0")]
    let mut prover = risc0_zkvm::Prover::new_from_elf(GUEST_ELF).unwrap();
    
    // Add input
    prover.add_input(&input).unwrap();
    
    // Generate proof
    let proof = prover.prove().unwrap();
    
    // Extract output
    #[cfg(feature = "sp1")]
    let output: Output = proof.public_values.read().unwrap();
    #[cfg(feature = "risc0")]
    let output: Output = proof.receipt.journal.decode().unwrap();
    
    // User's output handling
    println!("Result: {}", output.result);
}
```

## Project Setup

A ProofLab-rs project using the new approach would be set up as follows:

```
my_zkvm_project/
├── Cargo.toml
└── src/
    └── main.rs       # Contains all annotated code
```

**my_zkvm_project/Cargo.toml:**
```toml
[package]
name = "my_zkvm_project"
version = "0.1.0"
edition = "2021"

[dependencies]
prooflab = "0.1.0"
regex = "1.5"         # Regular dependencies are automatically managed

[build-dependencies]
prooflab-build = "0.1.0"

[features]
default = ["sp1"]
sp1 = ["prooflab/sp1"]
risc0 = ["prooflab/risc0"]
precompiles = ["prooflab/precompiles"]  # For accelerated crypto implementations
```

**my_zkvm_project/src/main.rs:**
```rust
use prooflab::prelude::*;
use regex::Regex;

#[zkvm_input]
struct RegexInput {
    pattern: String,
    target: String,
}

#[zkvm_output]
struct RegexOutput {
    matches: bool,
}

#[zkvm_guest]
fn regex_matcher() {
    // Read inputs
    let input: RegexInput = prooflab_io::read();
    
    // Try to compile the regex pattern
    let regex = match Regex::new(&input.pattern) {
        Ok(regex) => regex,
        Err(_) => {
            panic!("Invalid regex pattern");
        }
    };

    // Perform the regex search
    let matches = regex.is_match(&input.target);
    
    // Write result
    prooflab_io::commit(&RegexOutput { matches });
}

#[zkvm_host]
fn main() {
    // Prepare input
    let input = RegexInput {
        pattern: "a+".to_string(),
        target: "an era of trust".to_string(),
    };
    
    // Run prover
    let output: RegexOutput = prooflab::prove(input);
    
    // Display result
    println!("Pattern matched: {}", output.matches);
}
```

## Dependency Management

One of the key challenges is managing dependencies across guest and host environments, including precompile support. The build system handles this by:

1. Analyzing source code to identify required dependencies
2. Generating appropriate `Cargo.toml` files for each target
3. Adding precompile replacements when the `precompiles` feature is enabled

For precompiles, the system would replace standard dependencies with optimized versions:

```toml
# Generated guest Cargo.toml with precompiles
[dependencies]
regex = "1.5"

[patch.crates-io]
sha2 = { git = "https://github.com/sp1-patches/RustCrypto-hashes", package = "sha2", branch = "patch-sha2-v0.10.8" }
crypto-bigint = { git = "https://github.com/sp1-patches/RustCrypto-bigint", branch = "patch-v0.5.5" }
```

## Advanced Features

### Multiple Guest Functions

The system supports multiple guest functions for complex applications:

```rust
#[zkvm_guest]
fn process_data() {
    // First guest function
}

#[zkvm_guest]
fn verify_signature() {
    // Second guest function
}

#[zkvm_host]
fn main() {
    // Choose which guest function to run
    let result1 = prooflab::prove_with_guest(process_data, input1);
    let result2 = prooflab::prove_with_guest(verify_signature, input2);
}
```

### Custom Prover Configuration

The system allows customizing prover behavior:

```rust
#[zkvm_host]
fn main() {
    let input = Input { value: 42 };
    
    // Configure custom prover settings
    let prover_config = ProverConfig {
        num_threads: 8,
        memory_limit: Some(16 * 1024 * 1024 * 1024), // 16GB
        gpu_enabled: true,
    };
    
    // Run with custom configuration
    let output = prooflab::prove_with_config(input, prover_config);
}
```

### Library Support

Libraries are naturally supported with the struct-based approach:

```rust
// In my_library/src/lib.rs
pub mod crypto {
    use prooflab::prelude::*;
    
    #[zkvm_exportable]
    pub fn verify_signature(message: &[u8], signature: &[u8], public_key: &[u8]) -> bool {
        // Library function that can be called from zkVM guest code
        // ...
    }
}
```

The main program can use this library:

```rust
use my_library::crypto::verify_signature;

#[zkvm_guest]
fn guest() {
    let input: SignatureInput = prooflab_io::read();
    let is_valid = verify_signature(&input.message, &input.signature, &input.public_key);
    prooflab_io::commit(&SignatureOutput { is_valid });
}
```

## CLI Tool Integration

The ProofLab-rs CLI tool would be updated to support this approach:

```bash
# Build and run with SP1
$ prooflab run --backend sp1 ./my_project

# Enable precompiles for accelerated crypto
$ prooflab run --backend sp1 --precompiles ./my_project

# Use GPU acceleration
$ prooflab run --backend sp1 --gpu ./my_project
```

## Benefits

1. **Unified Development Experience**: Write code in a single crate while the system handles the separate compilation targets
2. **Type Safety**: Well-defined, type-safe data structures for I/O
3. **Automated Dependency Management**: System handles dependencies and precompiles
4. **Reduced Boilerplate**: Cleaner, more declarative code
5. **Improved IDE Support**: Better code completion and error checking
6. **Backend Flexibility**: Switch between zkVM backends with simple configuration changes

## Implementation Plan

1. **Phase 1**: Core Macro System
   - Implement basic annotation macros
   - Create build-time code generator
   - Support basic I/O patterns

2. **Phase 2**: Enhanced Functionality
   - Add support for multiple guest functions
   - Implement dependency analysis and management
   - Add precompile support

3. **Phase 3**: Integration and Tooling
   - Update CLI tool to support the new approach
   - Create migration tools for existing projects
   - Add comprehensive documentation and examples

## Timeline

- **Month 1-2**: Develop core macro system and prototype build generator
- **Month 3-4**: Implement enhanced functionality and test with complex examples
- **Month 5-6**: Complete tooling integration, documentation, and initial release

## Conclusion

The proposed unified macro-based approach addresses the fundamental challenges of zkVM development while providing a significantly improved developer experience. By handling the architectural constraints of zkVMs behind a clean, declarative API, we can make zero-knowledge proofs more accessible to a broader range of developers. This approach builds on the lessons learned from the current ProofLab-rs implementation while incorporating a more modern, type-safe design that leverages Rust's powerful macro and type systems.