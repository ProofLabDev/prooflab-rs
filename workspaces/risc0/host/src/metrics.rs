use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tracing::{error, info};

#[derive(Default, Serialize, Deserialize)]
pub struct Risc0Metrics {
    pub cycles: u64,
    pub num_segments: usize,
    pub core_proof_size: usize,
    pub recursive_proof_size: usize,
    pub core_prove_duration: Duration,
    pub core_verify_duration: Duration,
    pub compress_prove_duration: Duration,
    pub compress_verify_duration: Duration,
    pub input_size: usize,  // Size of inputs in bytes
    pub output_size: usize, // Size of outputs/journal in bytes
}

pub struct MetricsCollector {
    start_time: Option<Instant>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self { start_time: None }
    }

    pub fn start_timing(&mut self) {
        self.start_time = Some(Instant::now());
    }

    pub fn elapsed(&self) -> Option<Duration> {
        self.start_time.map(|t| t.elapsed())
    }
}

pub fn write_metrics(metrics: &Risc0Metrics, output_path: &std::path::Path) -> std::io::Result<()> {
    info!("About to write metrics");
    let metrics_path = output_path.join("risc0_metrics.json");
    info!("Full metrics path: {}", metrics_path.display());
    let json = serde_json::to_string_pretty(metrics)?;
    info!("Generated JSON: {}", json);
    match std::fs::write(&metrics_path, &json) {
        Ok(_) => {
            info!("Successfully wrote metrics to {}", metrics_path.display());
            Ok(())
        }
        Err(e) => {
            error!("Failed to write metrics: {}", e);
            Err(e)
        }
    }
}
