# prooflab-rs Development Guide

## Build Commands
- Build: `cargo build --release`
- Run with SP1: `cargo run --release -- prove-sp1 <PROGRAM_DIRECTORY_PATH>`
- Run with Risc0: `cargo run --release -- prove-risc0 <PROGRAM_DIRECTORY_PATH>`
- Examples: `make prove_sp1_fibonacci`, `make prove_risc0_sha`, etc.

## Code Standards
- **Formatting**: 4-space indentation, 100-char line limit, `cargo fmt` compliant
- **Imports**: Standard library first, then external crates, then internal modules
- **Naming**: snake_case for functions/variables, PascalCase for types/traits
- **Error Handling**: Use Result with ? operator, descriptive error messages
- **Types**: Explicit type annotations, proper use of references and ownership
- **Documentation**: Document public APIs with rustdoc-style comments

## Project Structure
- `/src`: Core codebase (VM backends, CLI, utils)
- `/examples`: Example programs for each VM
- `/prooflab_io`: I/O marshalling between host and guest

Maintain consistent style across the codebase. Follow Rust's best practices for memory safety and error handling.