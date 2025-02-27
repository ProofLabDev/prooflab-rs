# ProofLab Tests

This directory contains tests for the `prooflab` crate. The tests are organized into several modules that cover different aspects of the codebase.

## Running Tests

To run all tests:

```bash
cargo test --package prooflab
```

To run a specific test:

```bash
cargo test --package prooflab test_name
```

## Test Structure

- **utils_tests.rs**: Tests for utility functions in the utils module
- **network_tests.rs**: Tests for the NetworkArg enum and conversions
- **proof_args_tests.rs**: Tests for the ProofArgs struct and CLI argument parsing
- **telemetry_tests.rs**: Tests for the telemetry module
- **vm_module_tests.rs**: Tests for the SP1 and RISC0 modules
- **submit_proof_tests.rs**: Tests for the submit_proof_to_aligned function

## Adding New Tests

When adding new tests:

1. Organize tests by module or functionality
2. Add your new test module to the imports in test_runner.rs
3. Use tempfile and mock objects where appropriate to avoid external dependencies
4. Avoid tests that require actual blockchain or network interactions

## Mocking External Dependencies

Some prooflab functionality relies on external services like blockchain interactions. In these cases:

- Use mockall to create mock objects for external dependencies
- Create test fixtures that can be reused in multiple tests
- Separate core logic from external I/O where possible to make testing easier