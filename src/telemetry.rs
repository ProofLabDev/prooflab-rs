use log::{debug, info};
use nvml_wrapper::Nvml;
use serde::Serialize;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration as StdDuration;
use std::time::{Duration, Instant};
use sysinfo::System;
use toml::Value;

const BYTES_TO_KB: u64 = 1024;
const EC2_METADATA_TOKEN_URL: &str = "http://169.254.169.254/latest/api/token";
const EC2_METADATA_INSTANCE_TYPE_URL: &str =
    "http://169.254.169.254/latest/meta-data/instance-type";
const DOCKER_CHECK_FILE: &str = "/.dockerenv";

#[derive(Default, Serialize, Clone)]
pub struct CargoMetadata {
    pub package_name: Option<String>,
    pub version: Option<String>,
    pub authors: Option<Vec<String>>,
    pub edition: Option<String>,
    pub dependencies: Option<Vec<(String, String)>>, // (name, version)
}

#[derive(Default, Serialize, Clone)]
pub struct ZkMetrics {
    pub cycles: Option<u64>,                 // Number of VM cycles executed
    pub num_segments: Option<usize>,         // Number of segments/shards
    pub core_proof_size: Option<usize>,      // Size of the core proof in bytes
    pub recursive_proof_size: Option<usize>, // Size of the recursive/compressed proof in bytes
    pub execution_speed: Option<f64>,        // Cycles per second during proof generation
    pub compiled_program_size: Option<u64>,  // Size of the compiled program in bytes
}

#[derive(Default, Serialize, Clone)]
pub struct TimingMetrics {
    pub workspace_setup_duration: Option<Duration>,
    pub compilation_duration: Option<Duration>,
    pub proof_generation_duration: Option<Duration>,
    pub core_prove_duration: Option<Duration>, // Time to generate initial proof
    pub core_verify_duration: Option<Duration>, // Time to verify initial proof
    pub compress_prove_duration: Option<Duration>, // Time to generate compressed/recursive proof
    pub compress_verify_duration: Option<Duration>, // Time to verify compressed/recursive proof
    pub total_duration: Option<Duration>,
}

#[derive(Default, Serialize, Clone)]
pub struct ProgramInfo {
    pub file_path: String,
    pub file_name: String,
    pub absolute_path: Option<String>,
    pub guest_metadata: CargoMetadata,
    pub host_metadata: CargoMetadata,
}

#[derive(Default, Serialize, Clone)]
pub struct ResourceMetrics {
    max_memory_kb: u64,
    min_memory_kb: u64,
    avg_memory_kb: u64,
    max_cpu_percent: f32,
    min_cpu_percent: f32,
    avg_cpu_percent: f32,
    samples: usize,
}

#[derive(Default, Serialize, Clone)]
pub struct GpuInfo {
    pub name: String,
    pub memory_total_kb: Option<u64>,
    pub vendor: String,
}

#[derive(Default, Serialize, Clone)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub total_memory_kb: u64,
    pub cpu_brand: String,
    pub cpu_count: usize,
    pub cpu_frequency_mhz: u64,
    pub gpu_enabled: bool,
    pub gpus: Vec<GpuInfo>,
    pub is_ec2: bool,
    pub ec2_instance_type: Option<String>,
    pub llvm_version: Option<String>,
}

#[derive(Default, Serialize, Clone)]
pub struct TelemetryData {
    pub timing: TimingMetrics,
    pub resources: ResourceMetrics,
    pub proving_system: String,
    pub precompiles_enabled: bool,
    pub gpu_enabled: bool,
    pub program: ProgramInfo,
    pub zk_metrics: ZkMetrics,
    pub system_info: SystemInfo,
}

pub struct TelemetryCollector {
    start_time: Instant,
    system: System,
    metrics: Arc<Mutex<TelemetryData>>,
    enabled: bool,
    resource_samples: Arc<Mutex<Vec<(u64, f32)>>>, // (memory_kb, cpu_percent)
}

impl TelemetryCollector {
    pub fn new(
        proving_system: &str,
        precompiles_enabled: bool,
        gpu_enabled: bool,
        enabled: bool,
        guest_path: &str,
    ) -> Self {
        let mut system = System::new();
        system.refresh_all();
        system.refresh_cpu_frequency();

        let path = PathBuf::from(guest_path);
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Extract guest metadata
        let guest_metadata = Self::extract_cargo_metadata(guest_path);

        // Extract host metadata from the appropriate host Cargo.toml
        let host_metadata = if let Some(home_dir) = dirs::home_dir() {
            let home_dir = home_dir.join(".prooflab");
            let host_toml_path = if proving_system == "SP1" {
                home_dir.join("workspaces/sp1/script/Cargo.toml")
            } else {
                home_dir.join("workspaces/risc0/host/Cargo.toml")
            };
            if host_toml_path.exists() {
                Self::extract_cargo_metadata(host_toml_path.parent().unwrap().to_str().unwrap())
            } else {
                CargoMetadata::default()
            }
        } else {
            CargoMetadata::default()
        };

        // Calculate average CPU frequency across all CPUs
        let cpu_frequency = if !system.cpus().is_empty() {
            let total_freq: u64 = system.cpus().iter().map(|cpu| cpu.frequency()).sum();
            total_freq / system.cpus().len() as u64
        } else {
            0
        };

        // Fetch EC2 metadata
        let (is_ec2, ec2_instance_type) = Self::fetch_ec2_metadata();

        // Discover GPUs
        let gpus = if gpu_enabled {
            Self::discover_gpus()
        } else {
            Vec::new()
        };

        // Get LLVM version
        let llvm_version = Self::get_llvm_version();
        if let Some(version) = &llvm_version {
            debug!("Detected LLVM version: {}", version);
        } else {
            debug!("Could not detect LLVM version");
        }

        // Collect system information
        let system_info = SystemInfo {
            os_name: System::name().unwrap_or_else(|| "unknown".to_string()),
            os_version: System::os_version().unwrap_or_else(|| "unknown".to_string()),
            kernel_version: System::kernel_version().unwrap_or_else(|| "unknown".to_string()),
            total_memory_kb: system.total_memory() / BYTES_TO_KB,
            cpu_brand: system
                .cpus()
                .first()
                .map(|cpu| cpu.brand().to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            cpu_count: system.cpus().len(),
            cpu_frequency_mhz: cpu_frequency,
            gpu_enabled,
            gpus,
            is_ec2,
            ec2_instance_type,
            llvm_version,
        };

        let metrics = TelemetryData {
            proving_system: proving_system.to_string(),
            precompiles_enabled,
            gpu_enabled,
            program: ProgramInfo {
                file_path: guest_path.to_string(),
                file_name,
                absolute_path: std::fs::canonicalize(guest_path)
                    .ok()
                    .map(|p| p.to_string_lossy().to_string()),
                guest_metadata,
                host_metadata,
            },
            system_info,
            ..Default::default()
        };

        Self {
            start_time: Instant::now(),
            system,
            metrics: Arc::new(Mutex::new(metrics)),
            enabled,
            resource_samples: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn extract_cargo_metadata(guest_path: &str) -> CargoMetadata {
        let cargo_path = PathBuf::from(guest_path).join("Cargo.toml");

        if let Ok(contents) = fs::read_to_string(cargo_path) {
            if let Ok(cargo_toml) = contents.parse::<Value>() {
                let mut metadata = CargoMetadata::default();

                if let Some(package) = cargo_toml.get("package") {
                    if let Some(name) = package.get("name").and_then(|v| v.as_str()) {
                        metadata.package_name = Some(name.to_string());
                    }
                    if let Some(version) = package.get("version").and_then(|v| v.as_str()) {
                        metadata.version = Some(version.to_string());
                    }
                    if let Some(authors) = package.get("authors").and_then(|v| v.as_array()) {
                        metadata.authors = Some(
                            authors
                                .iter()
                                .filter_map(|a| a.as_str())
                                .map(String::from)
                                .collect(),
                        );
                    }
                    if let Some(edition) = package.get("edition").and_then(|v| v.as_str()) {
                        metadata.edition = Some(edition.to_string());
                    }
                }

                if let Some(deps) = cargo_toml.get("dependencies") {
                    if let Some(table) = deps.as_table() {
                        metadata.dependencies = Some(
                            table
                                .iter()
                                .map(|(name, value)| {
                                    let version = match value {
                                        Value::String(v) => v.clone(),
                                        Value::Table(t) => {
                                            if let Some(git) = t.get("git").and_then(|v| v.as_str())
                                            {
                                                if let Some(tag) =
                                                    t.get("tag").and_then(|v| v.as_str())
                                                {
                                                    format!("git:{} tag:{}", git, tag)
                                                } else if let Some(branch) =
                                                    t.get("branch").and_then(|v| v.as_str())
                                                {
                                                    format!("git:{} branch:{}", git, branch)
                                                } else {
                                                    format!("git:{}", git)
                                                }
                                            } else {
                                                t.get("version")
                                                    .and_then(|v| v.as_str())
                                                    .unwrap_or("*")
                                                    .to_string()
                                            }
                                        }
                                        Value::Array(_) => "*".to_string(),
                                        _ => "*".to_string(),
                                    };
                                    (name.clone(), version)
                                })
                                .collect(),
                        );
                    }
                }

                return metadata;
            }
        }
        CargoMetadata::default()
    }

    fn is_running_in_docker() -> bool {
        std::path::Path::new(DOCKER_CHECK_FILE).exists()
    }

    fn fetch_ec2_metadata() -> (bool, Option<String>) {
        let in_docker = Self::is_running_in_docker();
        debug!("Running in Docker container: {}", in_docker);
        if in_docker {
            debug!("Note: EC2 metadata service might not be accessible without host networking");
        }

        // Try to get IMDSv2 token first
        let client = reqwest::blocking::Client::new();
        debug!("Attempting to fetch EC2 metadata token...");
        let token_result = client
            .put(EC2_METADATA_TOKEN_URL)
            .header("X-aws-ec2-metadata-token-ttl-seconds", "21600")
            .timeout(StdDuration::from_secs(1))
            .send();

        match token_result {
            Ok(token_response) => {
                if !token_response.status().is_success() {
                    debug!(
                        "Failed to get EC2 metadata token: HTTP {}",
                        token_response.status()
                    );
                    return (false, None);
                }

                let token = token_response.text().unwrap_or_default();
                debug!("Successfully obtained EC2 metadata token");

                // Use token to get instance type
                debug!("Attempting to fetch instance type...");
                match client
                    .get(EC2_METADATA_INSTANCE_TYPE_URL)
                    .header("X-aws-ec2-metadata-token", token)
                    .timeout(StdDuration::from_secs(1))
                    .send()
                {
                    Ok(response) => {
                        if response.status().is_success() {
                            let instance_type = response.text().unwrap_or_default();
                            debug!("Successfully retrieved instance type: {}", instance_type);
                            (true, Some(instance_type))
                        } else {
                            debug!("Failed to get instance type: HTTP {}", response.status());
                            (true, None)
                        }
                    }
                    Err(e) => {
                        debug!("Error fetching instance type: {}", e);
                        (true, None)
                    }
                }
            }
            Err(e) => {
                debug!("Error fetching EC2 metadata token: {}", e);
                if in_docker {
                    debug!("This might be due to running in a Docker container without host networking");
                }
                (false, None)
            }
        }
    }

    fn discover_gpus() -> Vec<GpuInfo> {
        let mut gpus = Vec::new();

        // Try to initialize NVIDIA Management Library
        match Nvml::init() {
            Ok(nvml) => {
                // Get all NVIDIA devices
                match nvml.device_count() {
                    Ok(device_count) => {
                        debug!("Found {} NVIDIA GPU(s)", device_count);
                        for i in 0..device_count {
                            if let Ok(device) = nvml.device_by_index(i) {
                                let mut gpu_info = GpuInfo {
                                    vendor: "NVIDIA".to_string(),
                                    name: device
                                        .name()
                                        .unwrap_or_else(|_| "Unknown NVIDIA GPU".to_string()),
                                    memory_total_kb: None,
                                };

                                // Get memory information
                                if let Ok(memory) = device.memory_info() {
                                    gpu_info.memory_total_kb = Some(memory.total / 1024);
                                }

                                gpus.push(gpu_info);
                            }
                        }
                    }
                    Err(e) => {
                        debug!("Failed to get NVIDIA GPU count: {}", e);
                    }
                }
            }
            Err(e) => {
                debug!("Failed to initialize NVIDIA GPU detection: {}", e);
            }
        }

        if gpus.is_empty() {
            debug!("No GPUs detected");
        }

        gpus
    }

    fn get_llvm_version() -> Option<String> {
        // Try to get LLVM version using llvm-config
        if let Ok(output) = std::process::Command::new("llvm-config")
            .arg("--version")
            .output()
        {
            if output.status.success() {
                if let Ok(version) = String::from_utf8(output.stdout) {
                    return Some(version.trim().to_string());
                }
            }
        }

        // Try to get LLVM version using clang
        if let Ok(output) = std::process::Command::new("clang")
            .arg("--version")
            .output()
        {
            if output.status.success() {
                if let Ok(version) = String::from_utf8(output.stdout) {
                    if let Some(v) = version.lines().next() {
                        if let Some(idx) = v.find("version") {
                            return Some(v[idx..].trim().to_string());
                        }
                    }
                }
            }
        }

        None
    }

    pub fn record_workspace_setup(&self, duration: Duration) {
        if !self.enabled {
            return;
        }
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.timing.workspace_setup_duration = Some(duration);
        }
    }

    pub fn record_compilation(&self, duration: Duration) {
        if !self.enabled {
            return;
        }
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.timing.compilation_duration = Some(duration);
        }
    }

    pub fn record_proof_generation(&self, duration: Duration) {
        if !self.enabled {
            return;
        }
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.timing.proof_generation_duration = Some(duration);
        }
    }

    pub fn sample_resources(&mut self) {
        if !self.enabled {
            return;
        }

        self.system.refresh_all();

        let memory_used = self.system.used_memory() / BYTES_TO_KB;
        let cpu_usage: f32 = self
            .system
            .cpus()
            .iter()
            .map(|cpu| cpu.cpu_usage())
            .sum::<f32>()
            / self.system.cpus().len() as f32;

        if let Ok(mut samples) = self.resource_samples.lock() {
            samples.push((memory_used, cpu_usage));
        }
    }

    pub fn start_resource_monitoring(&self) -> std::sync::mpsc::Sender<()> {
        let (tx, rx) = std::sync::mpsc::channel();
        let samples = self.resource_samples.clone();
        let enabled = self.enabled;

        std::thread::spawn(move || {
            if !enabled {
                return;
            }
            let mut system = System::new();

            while rx.try_recv().is_err() {
                system.refresh_all();
                let memory_used = system.used_memory() / BYTES_TO_KB;
                let cpu_usage: f32 = system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>()
                    / system.cpus().len() as f32;

                if let Ok(mut samples) = samples.lock() {
                    samples.push((memory_used, cpu_usage));
                }
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        });

        tx
    }

    pub fn record_zk_metrics(
        &self,
        cycles: Option<u64>,
        num_segments: Option<usize>,
        core_proof_size: Option<usize>,
        recursive_proof_size: Option<usize>,
    ) {
        if !self.enabled {
            return;
        }
        if let Ok(mut metrics) = self.metrics.lock() {
            let proof_duration = metrics
                .timing
                .proof_generation_duration
                .unwrap_or(Duration::from_secs(0));
            let execution_speed = cycles.map(|c| c as f64 / proof_duration.as_secs_f64());
            let compiled_program_size = metrics.zk_metrics.compiled_program_size;

            metrics.zk_metrics = ZkMetrics {
                cycles,
                num_segments,
                core_proof_size,
                recursive_proof_size,
                execution_speed,
                compiled_program_size,
            };
        }
    }

    pub fn record_proof_timings(
        &self,
        core_prove: Duration,
        core_verify: Duration,
        compress_prove: Option<Duration>,
        compress_verify: Option<Duration>,
    ) {
        if !self.enabled {
            return;
        }
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.timing.core_prove_duration = Some(core_prove);
            metrics.timing.core_verify_duration = Some(core_verify);
            metrics.timing.compress_prove_duration = compress_prove;
            metrics.timing.compress_verify_duration = compress_verify;
        }
    }

    pub fn record_program_size(&self, size: u64) {
        if !self.enabled {
            return;
        }
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.zk_metrics.compiled_program_size = Some(size);
        }
    }

    pub fn finalize(self) -> Option<TelemetryData> {
        if !self.enabled {
            return None;
        }

        let mut final_metrics = self.metrics.lock().ok()?.clone();
        final_metrics.timing.total_duration = Some(self.start_time.elapsed());

        // Calculate resource statistics
        if let Ok(samples) = self.resource_samples.lock() {
            if !samples.is_empty() {
                let memory_stats = samples
                    .iter()
                    .map(|(mem, _)| *mem)
                    .fold((u64::MAX, 0u64, 0u64), |(min, max, sum), val| {
                        (min.min(val), max.max(val), sum + val)
                    });

                let cpu_stats = samples
                    .iter()
                    .map(|(_, cpu)| *cpu)
                    .fold((f32::MAX, 0f32, 0f32), |(min, max, sum), val| {
                        (min.min(val), max.max(val), sum + val)
                    });

                final_metrics.resources = ResourceMetrics {
                    max_memory_kb: memory_stats.1,
                    min_memory_kb: memory_stats.0,
                    avg_memory_kb: memory_stats.2 / samples.len() as u64,
                    max_cpu_percent: cpu_stats.1,
                    min_cpu_percent: cpu_stats.0,
                    avg_cpu_percent: cpu_stats.2 / samples.len() as f32,
                    samples: samples.len(),
                };
            }
        }

        // Log summary
        info!("Telemetry Summary:");
        info!(
            "Program: {} ({})",
            final_metrics.program.file_name, final_metrics.program.file_path
        );
        if let Some(abs_path) = &final_metrics.program.absolute_path {
            info!("Absolute Path: {}", abs_path);
        }

        // Log system information
        info!("System Information:");
        info!(
            "OS: {} {}",
            final_metrics.system_info.os_name, final_metrics.system_info.os_version
        );
        info!(
            "Kernel Version: {}",
            final_metrics.system_info.kernel_version
        );
        info!(
            "CPU: {} ({} cores @ {} MHz)",
            final_metrics.system_info.cpu_brand,
            final_metrics.system_info.cpu_count,
            final_metrics.system_info.cpu_frequency_mhz
        );
        info!(
            "Total Memory: {} KB",
            final_metrics.system_info.total_memory_kb
        );
        if let Some(llvm_version) = &final_metrics.system_info.llvm_version {
            info!("LLVM Version: {}", llvm_version);
        } else {
            info!("LLVM Version: Not detected");
        }
        info!(
            "GPU Acceleration: {}",
            if final_metrics.system_info.gpu_enabled {
                "Enabled"
            } else {
                "Disabled"
            }
        );

        if !final_metrics.system_info.gpus.is_empty() {
            for (i, gpu) in final_metrics.system_info.gpus.iter().enumerate() {
                info!("GPU {}: {} ({})", i + 1, gpu.name, gpu.vendor);
                if let Some(total) = gpu.memory_total_kb {
                    info!("  Memory Total: {} KB", total);
                }
            }
        }

        // Log Guest Cargo metadata
        let metadata = &final_metrics.program.guest_metadata;
        info!("Guest Program Metadata:");
        if let Some(name) = &metadata.package_name {
            info!("  Package Name: {}", name);
            if let Some(version) = &metadata.version {
                info!("  Version: {}", version);
            }
        }
        if let Some(authors) = &metadata.authors {
            info!("  Authors: {}", authors.join(", "));
        }
        if let Some(edition) = &metadata.edition {
            info!("  Rust Edition: {}", edition);
        }
        if let Some(deps) = &metadata.dependencies {
            info!("  Dependencies:");
            for (name, version) in deps {
                info!("    {} = {}", name, version);
            }
        }

        // Log Host Cargo metadata
        let metadata = &final_metrics.program.host_metadata;
        info!("\nHost Program Metadata:");
        if let Some(name) = &metadata.package_name {
            info!("  Package Name: {}", name);
            if let Some(version) = &metadata.version {
                info!("  Version: {}", version);
            }
        }
        if let Some(authors) = &metadata.authors {
            info!("  Authors: {}", authors.join(", "));
        }
        if let Some(edition) = &metadata.edition {
            info!("  Rust Edition: {}", edition);
        }
        if let Some(deps) = &metadata.dependencies {
            info!("  Dependencies:");
            for (name, version) in deps {
                info!("    {} = {}", name, version);
            }
        }

        // Log ZK metrics
        let zk = &final_metrics.zk_metrics;
        if let Some(cycles) = zk.cycles {
            info!("VM Cycles: {}", cycles);
            if let Some(speed) = zk.execution_speed {
                info!("Execution Speed: {:.2} cycles/second", speed);
            }
        }
        if let Some(segments) = zk.num_segments {
            info!("Number of Segments/Shards: {}", segments);
        }
        if let Some(size) = zk.core_proof_size {
            info!("Core Proof Size: {} bytes", size);
        }
        if let Some(size) = zk.recursive_proof_size {
            info!("Recursive Proof Size: {} bytes", size);
        }
        if let Some(size) = zk.compiled_program_size {
            info!("Compiled Program Size: {} bytes", size);
        }

        // Log timings
        let timing = &final_metrics.timing;
        info!("Total Duration: {:?}", timing.total_duration.unwrap());
        if let Some(d) = timing.workspace_setup_duration {
            info!("Workspace Setup: {:?}", d);
        }
        if let Some(d) = timing.compilation_duration {
            info!("Compilation: {:?}", d);
        }
        if let Some(d) = timing.proof_generation_duration {
            info!("Total Proof Generation: {:?}", d);
        }
        if let Some(d) = timing.core_prove_duration {
            info!("Core Proof Generation: {:?}", d);
        }
        if let Some(d) = timing.core_verify_duration {
            info!("Core Proof Verification: {:?}", d);
        }
        if let Some(d) = timing.compress_prove_duration {
            info!("Recursive Proof Generation: {:?}", d);
        }
        if let Some(d) = timing.compress_verify_duration {
            info!("Recursive Proof Verification: {:?}", d);
        }

        // Log resource usage
        info!(
            "Memory Usage - Max: {} KB, Min: {} KB, Avg: {} KB",
            final_metrics.resources.max_memory_kb,
            final_metrics.resources.min_memory_kb,
            final_metrics.resources.avg_memory_kb
        );
        info!(
            "CPU Usage - Max: {:.1}%, Min: {:.1}%, Avg: {:.1}%",
            final_metrics.resources.max_cpu_percent,
            final_metrics.resources.min_cpu_percent,
            final_metrics.resources.avg_cpu_percent
        );

        Some(final_metrics)
    }
}
