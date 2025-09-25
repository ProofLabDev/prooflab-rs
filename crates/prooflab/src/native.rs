use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::{Command, ExitStatus};

// Directory and file constants for native mode
pub const NATIVE_WORKSPACE_DIR: &str = "workspaces/native";
pub const NATIVE_SRC_DIR: &str = "workspaces/native/src";
pub const NATIVE_MAIN: &str = "workspaces/native/src/main.rs";
pub const NATIVE_CARGO_TOML: &str = "workspaces/native/Cargo.toml";
pub const NATIVE_BASE_CARGO_TOML: &str = "workspaces/base_files/native/cargo";
pub const NATIVE_METRICS_FILE: &str = "native_metrics.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct NativeMetrics {
    pub iterations: u64,
    pub input_size: u64,
    pub output_size: u64,
    pub execution_time_ms: u64,
}

pub fn prepare_native_runner(
    imports: &str,
    main_body: &str,
    input_body: &str,
    output_body: &str,
    main_path: &Path,
) -> std::io::Result<()> {
    // Filter out prooflab_io import since we provide our own mock
    let filtered_imports = imports
        .lines()
        .filter(|line| !line.trim().starts_with("use prooflab_io"))
        .collect::<Vec<_>>()
        .join("\n");

    let native_program = format!(
        r#"use std::time::Instant;
use std::fs;
use serde_json::json;

// User imports (filtered)
{}

// Mock prooflab_io module for native execution
mod prooflab_io {{
    use std::sync::Mutex;
    
    static INPUT_DATA: Mutex<Vec<u8>> = Mutex::new(Vec::new());
    static OUTPUT_DATA: Mutex<Vec<Vec<u8>>> = Mutex::new(Vec::new());
    
    pub fn write<T: serde::Serialize>(data: &T) {{
        let serialized = bincode::serialize(data).expect("Failed to serialize input");
        *INPUT_DATA.lock().unwrap() = serialized;
    }}
    
    pub fn read<T: serde::de::DeserializeOwned>() -> T {{
        let data = INPUT_DATA.lock().unwrap();
        bincode::deserialize(&data).expect("Failed to deserialize input")
    }}
    
    pub fn commit<T: serde::Serialize>(data: &T) {{
        let serialized = bincode::serialize(data).expect("Failed to serialize output");
        OUTPUT_DATA.lock().unwrap().push(serialized);
    }}
    
    pub fn out<T: serde::de::DeserializeOwned>() -> T {{
        let outputs = OUTPUT_DATA.lock().unwrap();
        let mut combined = Vec::new();
        for output in outputs.iter() {{
            combined.extend_from_slice(output);
        }}
        bincode::deserialize(&combined).expect("Failed to deserialize output")
    }}
    
    pub fn get_input_size() -> usize {{
        INPUT_DATA.lock().unwrap().len()
    }}
    
    pub fn get_output_size() -> usize {{
        OUTPUT_DATA.lock().unwrap().iter().map(|v| v.len()).sum()
    }}
}}

// User's main function (renamed)
fn user_main() {{
{}
}}

// User's input function
fn input() {{
{}
}}

// User's output function
fn output() {{
{}
}}

fn main() {{
    // Setup input
    input();
    
    // Measure execution time
    let start = Instant::now();
    
    // Run the user's main function
    user_main();
    
    let elapsed = start.elapsed();
    
    // Run output function to display results
    output();
    
    // Collect metrics
    let metrics = json!({{
        "iterations": 1,
        "input_size": prooflab_io::get_input_size(),
        "output_size": prooflab_io::get_output_size(),
        "execution_time_ms": elapsed.as_millis()
    }});
    
    // Write metrics to file
    fs::write("native_metrics.json", serde_json::to_string_pretty(&metrics).unwrap())
        .expect("Failed to write metrics");
    
    println!("\n=== Native Execution Metrics ===");
    println!("Execution Time: {{:.3}} ms", elapsed.as_secs_f64() * 1000.0);
    println!("Input Size: {{}} bytes", prooflab_io::get_input_size());
    println!("Output Size: {{}} bytes", prooflab_io::get_output_size());
    println!("================================");
}}
"#,
        filtered_imports,
        main_body.trim_start_matches('{').trim_end_matches('}'),
        input_body.trim_start_matches('{').trim_end_matches('}'),
        output_body.trim_start_matches('{').trim_end_matches('}')
    );

    fs::write(main_path, native_program)?;
    Ok(())
}

pub fn build_native_program(workspace_dir: &Path) -> std::io::Result<ExitStatus> {
    Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(workspace_dir)
        .status()
}

pub fn run_native_program(workspace_dir: &Path, output_dir: &Path) -> std::io::Result<ExitStatus> {
    let status = Command::new("./target/release/native_runner")
        .current_dir(workspace_dir)
        .status()?;

    // Copy metrics file to output directory if it exists
    let metrics_src = workspace_dir.join(NATIVE_METRICS_FILE);
    let metrics_dst = output_dir.join(NATIVE_METRICS_FILE);
    if metrics_src.exists() {
        fs::copy(metrics_src, metrics_dst)?;
    }

    Ok(status)
}

pub fn read_metrics() -> std::io::Result<NativeMetrics> {
    let metrics_path = Path::new(NATIVE_METRICS_FILE);
    if !metrics_path.exists() {
        return Ok(NativeMetrics {
            iterations: 1,
            input_size: 0,
            output_size: 0,
            execution_time_ms: 0,
        });
    }

    let contents = fs::read_to_string(metrics_path)?;
    let metrics: NativeMetrics = serde_json::from_str(&contents)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(metrics)
}
