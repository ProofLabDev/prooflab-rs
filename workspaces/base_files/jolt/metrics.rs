use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{Duration, Instant};

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
        self.start_time.map(|start| start.elapsed())
    }
}

pub fn write_metrics(metrics: &JoltMetrics, path: &Path) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(metrics)?;
    std::fs::write(path.join("jolt_metrics.json"), json)
}