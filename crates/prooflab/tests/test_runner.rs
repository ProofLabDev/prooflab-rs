// Import all the test modules
mod utils_tests;
mod network_tests;
mod proof_args_tests;
mod telemetry_tests;
mod vm_module_tests;

// Re-export test modules so they're included in the test runner
pub use utils_tests::*;
pub use network_tests::*;
pub use proof_args_tests::*;
pub use telemetry_tests::*;
pub use vm_module_tests::*;

// This file is a wrapper for running all the tests
// You can run the tests with: cargo test --package prooflab

#[test]
fn test_runner_loads() {
    // Simply verifies the test runner loads properly
    assert!(true);
}

#[test]
fn test_vm_modules_utils() {
    // This test just makes sure the test runner successfully imports all module tests
    // The actual tests are in their respective module files
    assert!(true);
}