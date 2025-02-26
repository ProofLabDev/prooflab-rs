use aligned_sdk::core::types::ProvingSystemId;
use clap::{Parser, Subcommand};
use env_logger::Env;
use log::error;
use log::info;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;
use tokio::io;
use prooflab_rs::{
    risc0, sp1, submit_proof_to_aligned, telemetry::TelemetryCollector, utils, ProofArgs,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[clap(about = "Generate a proof of execution of a program using SP1")]
    ProveSp1(ProofArgs),
    #[clap(about = "Generate a proof of execution of a program using RISC0")]
    ProveRisc0(ProofArgs),
}

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let cli = Cli::parse();

    match &cli.command {
        Commands::ProveSp1(args) => {
            info!("Proving with SP1, program in: {}", args.guest_path);

            let telemetry = TelemetryCollector::new(
                "SP1",
                args.precompiles,
                args.gpu,
                args.enable_telemetry,
                &args.guest_path,
            );
            let workspace_start = Instant::now();

            // Perform sanitation checks on directory
            let proof_data_dir = PathBuf::from(&args.proof_data_directory_path);
            if !proof_data_dir.exists() {
                info!("Saving Proofs to: {:?}", &args.proof_data_directory_path);
                std::fs::create_dir_all(proof_data_dir)?;
            }
            if utils::validate_directory_structure(&args.guest_path) {
                let Some(home_dir) = dirs::home_dir() else {
                    error!("Failed to locate home directory");
                    return Ok(());
                };
                let Ok(current_dir) = std::env::current_dir() else {
                    error!("Failed to locate current directory");
                    return Ok(());
                };
                let home_dir = home_dir.join(".prooflab");
                utils::prepare_workspace(
                    &PathBuf::from(&args.guest_path),
                    &home_dir.join(sp1::SP1_SRC_DIR),
                    &home_dir.join(sp1::SP1_GUEST_CARGO_TOML),
                    &home_dir.join("workspaces/sp1/script"),
                    &home_dir.join("workspaces/sp1/script/Cargo.toml"),
                    &home_dir.join(sp1::SP1_BASE_HOST_CARGO_TOML),
                    &home_dir.join(sp1::SP1_BASE_GUEST_CARGO_TOML),
                )?;

                telemetry.record_workspace_setup(workspace_start.elapsed());

                let compilation_start = Instant::now();
                let Ok(imports) = utils::get_imports(&home_dir.join(sp1::SP1_GUEST_MAIN)) else {
                    error!("Failed to extract imports");
                    return Ok(());
                };

                let main_path = home_dir.join(sp1::SP1_GUEST_MAIN);
                let Ok(function_bodies) = utils::extract_function_bodies(
                    &main_path,
                    vec![
                        "fn main()".to_string(),
                        "fn input()".to_string(),
                        "fn output()".to_string(),
                    ],
                ) else {
                    error!("Failed to extract function bodies");
                    return Ok(());
                };

                utils::prepare_guest(
                    &imports,
                    &function_bodies[0],
                    sp1::SP1_GUEST_PROGRAM_HEADER,
                    sp1::SP1_IO_READ,
                    sp1::SP1_IO_COMMIT,
                    &home_dir.join(sp1::SP1_GUEST_MAIN),
                )?;
                sp1::prepare_host(
                    &function_bodies[1],
                    &function_bodies[2],
                    &imports,
                    &home_dir.join(sp1::SP1_BASE_HOST),
                    &home_dir.join(sp1::SP1_HOST_MAIN),
                )?;

                if args.precompiles {
                    let mut toml_file = OpenOptions::new()
                        .append(true)
                        .open(home_dir.join(sp1::SP1_GUEST_CARGO_TOML))?;

                    writeln!(toml_file, "{}", sp1::SP1_ACCELERATION_IMPORT)?;
                }

                let script_dir = home_dir.join(sp1::SP1_SCRIPT_DIR);

                // Build the program first
                let build_result = sp1::build_sp1_program(&script_dir)?;
                if !build_result.success() {
                    error!("SP1 program build failed");
                    return Ok(());
                }
                info!("SP1 program built successfully");
                telemetry.record_compilation(compilation_start.elapsed());

                // Record compiled program size
                if let Ok(metadata) = fs::metadata(home_dir.join(
                    "workspaces/sp1/program/target/elf-compilation/riscv32im-succinct-zkvm-elf/release/method",
                )) {
                    telemetry.record_program_size(metadata.len());
                    info!("Recorded SP1 program size: {} bytes", metadata.len());
                } else {
                    error!("Failed to read SP1 program size");
                }

                let proof_gen_start = Instant::now();

                // Start resource sampling in a separate thread
                let tx = telemetry.start_resource_monitoring();

                let result = sp1::generate_sp1_proof(&script_dir, &current_dir, args.gpu)?;

                // Stop resource sampling
                let _ = tx.send(());

                telemetry.record_proof_generation(proof_gen_start.elapsed());

                if result.success() {
                    info!("SP1 proof and ELF generated");

                    // Read and record SP1 metrics
                    if let Ok(sp1_metrics) = sp1::read_metrics() {
                        telemetry.record_zk_metrics(
                            Some(sp1_metrics.cycles),
                            Some(sp1_metrics.num_segments),
                            Some(sp1_metrics.core_proof_size),
                            Some(sp1_metrics.recursive_proof_size),
                        );
                        telemetry.record_proof_timings(
                            sp1_metrics.core_prove_duration,
                            sp1_metrics.core_verify_duration,
                            Some(sp1_metrics.compress_prove_duration),
                            Some(sp1_metrics.compress_verify_duration),
                        );
                    }

                    utils::replace(
                        &home_dir.join(sp1::SP1_GUEST_CARGO_TOML),
                        sp1::SP1_ACCELERATION_IMPORT,
                        "",
                    )?;

                    // Submit to aligned
                    if args.submit_to_aligned {
                        submit_proof_to_aligned(
                            sp1::SP1_PROOF_PATH,
                            sp1::SP1_ELF_PATH,
                            Some(sp1::SP1_PUB_INPUT_PATH),
                            args,
                            ProvingSystemId::SP1,
                        )
                        .await
                        .map_err(|e| {
                            error!("Proof not submitted to Aligned");
                            io::Error::other(e.to_string())
                        })?;
                        info!("SP1 proof submitted and verified on Aligned");
                    }

                    // Save telemetry data if enabled
                    if let Some(telemetry_data) = telemetry.finalize() {
                        if args.enable_telemetry {
                            fs::create_dir_all(&args.telemetry_output_path)?;
                            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                            let package_name = telemetry_data
                                .program
                                .guest_metadata
                                .package_name
                                .as_deref()
                                .unwrap_or("unknown");
                            let instance_type = telemetry_data
                                .system_info
                                .ec2_instance_type
                                .as_deref()
                                .unwrap_or("local");
                            let telemetry_file = format!(
                                "{}/sp1_telemetry_{}_{}_{}_{}.json",
                                args.telemetry_output_path,
                                package_name,
                                instance_type,
                                timestamp,
                                if result.success() {
                                    "success"
                                } else {
                                    "failed"
                                }
                            );
                            fs::write(
                                &telemetry_file,
                                serde_json::to_string_pretty(&telemetry_data)?,
                            )?;
                            info!("Telemetry data saved to: {}", telemetry_file);
                        }
                    }

                    std::fs::copy(
                        home_dir.join(sp1::SP1_BASE_HOST_FILE),
                        home_dir.join(sp1::SP1_HOST_MAIN),
                    )
                    .inspect_err(|_e| {
                        error!("Failed to clear SP1 host file");
                    })?;
                    return Ok(());
                }
                error!(
                    "SP1 proof generation failed with exit code: {}",
                    result.code().unwrap_or(-1)
                );
                if let Some(code) = result.code() {
                    match code {
                        101 => error!(
                            "Proof verification failed - the generated proof could not be verified"
                        ),
                        102 => error!("ELF file generation failed"),
                        _ => error!(
                            "Unknown error occurred during proof generation, code: {}",
                            code
                        ),
                    }
                }

                // Save telemetry data even on failure
                if let Some(telemetry_data) = telemetry.finalize() {
                    if args.enable_telemetry {
                        fs::create_dir_all(&args.telemetry_output_path)?;
                        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                        let package_name = telemetry_data
                            .program
                            .guest_metadata
                            .package_name
                            .as_deref()
                            .unwrap_or("unknown");
                        let instance_type = telemetry_data
                            .system_info
                            .ec2_instance_type
                            .as_deref()
                            .unwrap_or("local");
                        let telemetry_file = format!(
                            "{}/sp1_telemetry_{}_{}_{}_{}.json",
                            args.telemetry_output_path,
                            package_name,
                            instance_type,
                            timestamp,
                            if result.success() {
                                "success"
                            } else {
                                "failed"
                            }
                        );
                        fs::write(
                            &telemetry_file,
                            serde_json::to_string_pretty(&telemetry_data)?,
                        )?;
                        info!("Telemetry data saved to: {}", telemetry_file);
                    }
                }

                // Clear host
                std::fs::copy(
                    home_dir.join(sp1::SP1_BASE_HOST_FILE),
                    home_dir.join(sp1::SP1_HOST_MAIN),
                )?;
                return Ok(());
            } else {
                error!("prooflab-rs directory structure invalid please consult the README",);
                return Ok(());
            }
        }

        Commands::ProveRisc0(args) => {
            info!("Proving with Risc0, program in: {}", args.guest_path);

            let telemetry = TelemetryCollector::new(
                "RISC0",
                args.precompiles,
                args.gpu,
                args.enable_telemetry,
                &args.guest_path,
            );
            let workspace_start = Instant::now();

            // Perform sanitation checks on directory
            if utils::validate_directory_structure(&args.guest_path) {
                let proof_data_dir = PathBuf::from(&args.proof_data_directory_path);
                if !proof_data_dir.exists() {
                    info!(
                        "Saving generated proofs to: {:?}",
                        &args.proof_data_directory_path
                    );
                    std::fs::create_dir_all(proof_data_dir)?;
                }
                let Some(home_dir) = dirs::home_dir() else {
                    error!("Failed to locate home directory");
                    return Ok(());
                };
                let Ok(current_dir) = std::env::current_dir() else {
                    error!("Failed to locate current directory");
                    return Ok(());
                };
                let home_dir = home_dir.join(".prooflab");
                utils::prepare_workspace(
                    &PathBuf::from(&args.guest_path),
                    &home_dir.join(risc0::RISC0_SRC_DIR),
                    &home_dir.join(risc0::RISC0_GUEST_CARGO_TOML),
                    &home_dir.join("workspaces/risc0/host"),
                    &home_dir.join("workspaces/risc0/host/Cargo.toml"),
                    &home_dir.join(risc0::RISC0_BASE_HOST_CARGO_TOML),
                    &home_dir.join(risc0::RISC0_BASE_GUEST_CARGO_TOML),
                )?;

                telemetry.record_workspace_setup(workspace_start.elapsed());

                let compilation_start = Instant::now();
                let Ok(imports) = utils::get_imports(&home_dir.join(risc0::RISC0_GUEST_MAIN))
                else {
                    error!("Failed to extract imports");
                    return Ok(());
                };
                let main_path = home_dir.join(risc0::RISC0_GUEST_MAIN);
                let Ok(function_bodies) = utils::extract_function_bodies(
                    &main_path,
                    vec![
                        "fn main()".to_string(),
                        "fn input()".to_string(),
                        "fn output()".to_string(),
                    ],
                ) else {
                    error!("Failed to extract function bodies");
                    return Ok(());
                };

                utils::prepare_guest(
                    &imports,
                    &function_bodies[0],
                    risc0::RISC0_GUEST_PROGRAM_HEADER,
                    risc0::RISC0_IO_READ,
                    risc0::RISC0_IO_COMMIT,
                    &home_dir.join(risc0::RISC0_GUEST_MAIN),
                )?;
                risc0::prepare_host(
                    &function_bodies[1],
                    &function_bodies[2],
                    &imports,
                    &home_dir.join(risc0::RISC0_BASE_HOST),
                    &home_dir.join(risc0::RISC0_HOST_MAIN),
                )?;

                if args.precompiles {
                    let mut toml_file = OpenOptions::new()
                        .append(true)
                        .open(home_dir.join(risc0::RISC0_GUEST_CARGO_TOML))?;

                    writeln!(toml_file, "{}", risc0::RISC0_ACCELERATION_IMPORT)?;
                }

                let workspace_dir = home_dir.join(risc0::RISC0_WORKSPACE_DIR);

                // Build the program first
                let build_result = risc0::build_risc0_program(&workspace_dir)?;
                if !build_result.success() {
                    error!("RISC0 program build failed");
                    return Ok(());
                }
                info!("RISC0 program built successfully");
                telemetry.record_compilation(compilation_start.elapsed());

                // Record compiled program size
                if let Ok(metadata) = fs::metadata(home_dir.join(
                    "workspaces/risc0/target/riscv-guest/riscv32im-risc0-zkvm-elf/release/method",
                )) {
                    telemetry.record_program_size(metadata.len());
                    info!("Recorded RISC0 program size: {} bytes", metadata.len());
                } else {
                    error!("Failed to read RISC0 program size");
                }

                let proof_gen_start = Instant::now();

                // Start resource sampling in a separate thread
                let tx = telemetry.start_resource_monitoring();

                let result = risc0::generate_risc0_proof(&workspace_dir, &current_dir, args.gpu)?;

                // Stop resource sampling
                let _ = tx.send(());

                telemetry.record_proof_generation(proof_gen_start.elapsed());

                if result.success() {
                    info!("Risc0 proof and Image ID generated");

                    // Read and record RISC0 metrics
                    if let Ok(risc0_metrics) = risc0::read_metrics() {
                        telemetry.record_zk_metrics(
                            Some(risc0_metrics.cycles),
                            Some(risc0_metrics.num_segments),
                            Some(risc0_metrics.core_proof_size),
                            Some(risc0_metrics.recursive_proof_size),
                        );
                        telemetry.record_proof_timings(
                            risc0_metrics.core_prove_duration,
                            risc0_metrics.core_verify_duration,
                            Some(risc0_metrics.compress_prove_duration),
                            Some(risc0_metrics.compress_verify_duration),
                        );
                    }

                    utils::replace(
                        &home_dir.join(risc0::RISC0_GUEST_CARGO_TOML),
                        risc0::RISC0_ACCELERATION_IMPORT,
                        "",
                    )?;

                    // Submit to aligned
                    if args.submit_to_aligned {
                        submit_proof_to_aligned(
                            risc0::PROOF_FILE_PATH,
                            risc0::IMAGE_ID_FILE_PATH,
                            Some(risc0::PUBLIC_INPUT_FILE_PATH),
                            args,
                            ProvingSystemId::Risc0,
                        )
                        .await
                        .map_err(|e| {
                            error!("Error submitting proofs to Aligned: {:?}", e);
                            io::Error::other(e.to_string())
                        })?;

                        info!("Risc0 proof submitted and verified on Aligned");
                    }

                    // Save telemetry data if enabled
                    if let Some(telemetry_data) = telemetry.finalize() {
                        if args.enable_telemetry {
                            fs::create_dir_all(&args.telemetry_output_path)?;
                            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                            let package_name = telemetry_data
                                .program
                                .guest_metadata
                                .package_name
                                .as_deref()
                                .unwrap_or("unknown");
                            let instance_type = telemetry_data
                                .system_info
                                .ec2_instance_type
                                .as_deref()
                                .unwrap_or("local");
                            let telemetry_file = format!(
                                "{}/risc0_telemetry_{}_{}_{}_{}.json",
                                args.telemetry_output_path,
                                package_name,
                                instance_type,
                                timestamp,
                                if result.success() {
                                    "success"
                                } else {
                                    "failed"
                                }
                            );
                            fs::write(
                                &telemetry_file,
                                serde_json::to_string_pretty(&telemetry_data)?,
                            )?;
                            info!("Telemetry data saved to: {}", telemetry_file);
                        }
                    }

                    // Clear Host file
                    std::fs::copy(
                        home_dir.join(risc0::RISC0_BASE_HOST_FILE),
                        home_dir.join(risc0::RISC0_HOST_MAIN),
                    )
                    .inspect_err(|_e| {
                        error!("Failed to clear Risc0 host file");
                    })?;
                    return Ok(());
                }
                info!("Risc0 proof generation failed");

                // Save telemetry data even on failure
                if let Some(telemetry_data) = telemetry.finalize() {
                    if args.enable_telemetry {
                        fs::create_dir_all(&args.telemetry_output_path)?;
                        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                        let package_name = telemetry_data
                            .program
                            .guest_metadata
                            .package_name
                            .as_deref()
                            .unwrap_or("unknown");
                        let instance_type = telemetry_data
                            .system_info
                            .ec2_instance_type
                            .as_deref()
                            .unwrap_or("local");
                        let telemetry_file = format!(
                            "{}/risc0_telemetry_{}_{}_{}_{}.json",
                            args.telemetry_output_path,
                            package_name,
                            instance_type,
                            timestamp,
                            if result.success() {
                                "success"
                            } else {
                                "failed"
                            }
                        );
                        fs::write(
                            &telemetry_file,
                            serde_json::to_string_pretty(&telemetry_data)?,
                        )?;
                        info!("Telemetry data saved to: {}", telemetry_file);
                    }
                }

                // Clear Host file
                std::fs::copy(
                    home_dir.join(risc0::RISC0_BASE_HOST_FILE),
                    home_dir.join(risc0::RISC0_HOST_MAIN),
                )?;
                return Ok(());
            } else {
                error!("prooflab-rs directory structure incorrect please consult the README",);
                return Ok(());
            }
        }
    }
}
