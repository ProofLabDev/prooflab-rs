use prooflab::telemetry::{CargoMetadata, TelemetryCollector, ZkMetrics};
use std::time::Duration;

#[test]
fn test_telemetry_collector_creation() {
    // Create a telemetry collector
    let collector = TelemetryCollector::new(
        "TEST",
        false, // precompiles
        false, // gpu
        false, // enable_telemetry
        "test/path",
    );

    // Basic sanity check that it was created
    assert!(
        collector.finalize().is_none(),
        "Telemetry data should be None when enable_telemetry is false"
    );
}

#[test]
fn test_telemetry_collector_with_telemetry_enabled() {
    // Create a telemetry collector with telemetry enabled
    let mut collector = TelemetryCollector::new(
        "TEST",
        true, // precompiles
        true, // gpu
        true, // enable_telemetry
        "test/path",
    );

    // Record some metrics
    collector.record_workspace_setup(Duration::from_secs(1));
    collector.record_compilation(Duration::from_secs(2));
    collector.record_program_size(1024);

    // Record ZK metrics
    collector.record_zk_metrics(
        Some(1000), // cycles
        Some(2),    // num_segments
        Some(2048), // core_proof_size
        Some(1536), // recursive_proof_size
        Some(1024), // input_size
        Some(2048), // output_size
    );

    // Record proof timings
    collector.record_proof_timings(
        Duration::from_secs(4),       // core_prove_duration
        Duration::from_secs(1),       // core_verify_duration
        Some(Duration::from_secs(2)), // compress_prove_duration
        Some(Duration::from_secs(1)), // compress_verify_duration
    );

    // Finalize and get telemetry data
    let telemetry_data = collector.finalize();

    // Telemetry data should be Some when enable_telemetry is true
    assert!(
        telemetry_data.is_some(),
        "Telemetry data should be Some when enable_telemetry is true"
    );

    // Check telemetry data
    let data = telemetry_data.unwrap();

    // Check timing fields
    assert_eq!(
        data.timing.workspace_setup_duration,
        Some(Duration::from_secs(1))
    );
    assert_eq!(
        data.timing.compilation_duration,
        Some(Duration::from_secs(2))
    );

    // Check program fields
    assert_eq!(data.zk_metrics.compiled_program_size, Some(1024));
    assert_eq!(data.precompiles_enabled, true);

    // Check zk_metrics fields
    assert_eq!(data.zk_metrics.cycles, Some(1000));
    assert_eq!(data.zk_metrics.num_segments, Some(2));
    assert_eq!(data.zk_metrics.core_proof_size, Some(2048));
    assert_eq!(data.zk_metrics.recursive_proof_size, Some(1536));

    // Check timing fields for proof operations
    assert_eq!(
        data.timing.core_prove_duration,
        Some(Duration::from_secs(4))
    );
    assert_eq!(
        data.timing.core_verify_duration,
        Some(Duration::from_secs(1))
    );
    assert_eq!(
        data.timing.compress_prove_duration,
        Some(Duration::from_secs(2))
    );
    assert_eq!(
        data.timing.compress_verify_duration,
        Some(Duration::from_secs(1))
    );

    // Check system info
    assert_eq!(data.proving_system, "TEST");
    assert_eq!(data.gpu_enabled, true);
}

#[test]
fn test_cargo_metadata_defaults() {
    let metadata = CargoMetadata::default();

    assert!(metadata.package_name.is_none());
    assert!(metadata.version.is_none());
    assert!(metadata.authors.is_none());
    assert!(metadata.edition.is_none());
    assert!(metadata.dependencies.is_none());
}

#[test]
fn test_zk_metrics_defaults() {
    let metrics = ZkMetrics::default();

    assert!(metrics.cycles.is_none());
    assert!(metrics.num_segments.is_none());
    assert!(metrics.core_proof_size.is_none());
    assert!(metrics.recursive_proof_size.is_none());
}
