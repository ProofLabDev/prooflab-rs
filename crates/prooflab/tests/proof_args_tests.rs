use clap::Parser;
use prooflab::{NetworkArg, ProofArgs};
use std::path::PathBuf;

#[derive(Parser)]
struct TestCommand {
    #[command(flatten)]
    proof_args: ProofArgs,
}

#[test]
fn test_proof_args_basic_values() {
    // Create a test command with minimal arguments
    let args = TestCommand::parse_from(&["test", "path/to/guest"]);
    
    // Check basic values and defaults
    assert_eq!(args.proof_args.guest_path, "path/to/guest");
    assert_eq!(args.proof_args.submit_to_aligned, false);
    assert_eq!(args.proof_args.keystore_path, None);
    
    // Check default feature flags
    assert_eq!(args.proof_args.precompiles, false);
    assert_eq!(args.proof_args.gpu, false);
    assert_eq!(args.proof_args.enable_telemetry, false);
    
    // Check default paths
    assert_eq!(args.proof_args.proof_data_directory_path, "./proof_data");
    assert_eq!(args.proof_args.telemetry_output_path, "./telemetry");
}

#[test]
fn test_feature_flags() {
    // Create a test command with feature flags enabled
    let args = TestCommand::parse_from(&[
        "test",
        "path/to/guest",
        "--precompiles",
        "--gpu",
        "--enable-telemetry"
    ]);
    
    // Check that feature flags were set
    assert_eq!(args.proof_args.precompiles, true);
    assert_eq!(args.proof_args.gpu, true);
    assert_eq!(args.proof_args.enable_telemetry, true);
}

#[test]
fn test_custom_paths() {
    // Create a test command with custom paths
    let args = TestCommand::parse_from(&[
        "test",
        "custom/guest/path",
        "--proof-data-path", "custom/proof/path",
        "--telemetry-output", "custom/telemetry/path"
    ]);
    
    // Check custom path values
    assert_eq!(args.proof_args.guest_path, "custom/guest/path");
    assert_eq!(args.proof_args.proof_data_directory_path, "custom/proof/path");
    assert_eq!(args.proof_args.telemetry_output_path, "custom/telemetry/path");
}