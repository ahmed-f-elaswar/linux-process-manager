//! # Linux Process Manager (LPM)
//! 
//! A comprehensive, feature-rich process management system for Linux.
//! 
//! ## Features
//! 
//! ### Core Features
//! - Real-time process monitoring with sub-second refresh rates
//! - Multi-signal process control (SIGTERM, SIGKILL, SIGSTOP, etc.)
//! - Advanced sorting and filtering capabilities
//! - Hierarchical process tree visualization
//! - Network connection tracking per process
//! - Container awareness (Docker, Kubernetes, Podman)
//! 
//! ### Advanced Features
//! - GPU monitoring (NVIDIA, AMD, Intel)
//! - Historical data storage with SQLite
//! - REST API with JSON responses
//! - Modern web UI for remote monitoring
//! - Prometheus/InfluxDB metrics export
//! - Anomaly detection for unusual process behavior
//! 
//! ### Phase IV Features
//! - Structured logging with rotation
//! - CPU affinity and priority management
//! - Process snapshots and replay
//! - Multi-channel alerting (email, webhook, desktop)
//! - Process group operations
//! - Memory map visualization
//! - Custom view profiles
//! - Process state diffing
//! - Deep container runtime analysis
//! 
//! ## Usage
//! 
//! ```bash
//! # Interactive TUI mode (default)
//! cargo run
//! 
//! # API server mode
//! cargo run -- --api --api-port 8080
//! 
//! # Export metrics
//! cargo run -- --export prometheus --export-file metrics.txt
//! 
//! # Generate config
//! cargo run -- --generate-config config.toml
//! ```
//! 
//! ## Authors
//! 
//! CSCE 3401 - Operating Systems Fall 2025
//! - Adam Aberbach
//! - Mohammad Yahya Hammoudeh
//! - Mohamed Khalil Brik
//! - Ahmed Elaswar

// Core modules
pub mod process;   // Process management and monitoring
pub mod ui;        // Terminal user interface
pub mod tree;      // Process tree hierarchy
pub mod network;   // Network connections and containers
pub mod gpu;       // GPU monitoring
pub mod history;   // Historical data storage
pub mod api;       // REST API server
pub mod metrics;   // Metrics export
pub mod anomaly;   // Anomaly detection
pub mod config;    // Configuration management

// Phase IV modules
pub mod logging;    // Structured logging system
pub mod affinity;   // CPU affinity and priority
pub mod alerts;     // Smart alerting system
pub mod snapshots;  // Process snapshots
pub mod groups;     // Process group management
pub mod memmap;     // Memory map visualization
pub mod profiles;   // Saved view profiles
pub mod diffing;    // Process state comparison
pub mod containers; // Container deep dive

use clap::{Arg, Command};
use ui::run_app;
use tracing::{Level, debug, info, error};
use logging::log_system_event;

/// Main entry point for the Linux Process Manager.
/// 
/// Handles command-line argument parsing and routes execution to one of three modes:
/// 1. Interactive TUI mode (default)
/// 2. REST API server mode (--api)
/// 3. Metrics export mode (--export)
/// 
/// # Command-Line Arguments
/// 
/// - `-r, --refresh <SECONDS>`: Set refresh interval in seconds
/// - `-u, --user <USERNAME>`: Filter processes by user
/// - `-t, --tree`: Start in tree view mode
/// - `--api`: Start REST API server
/// - `--api-port <PORT>`: API server port (default: 8080)
/// - `--export <FORMAT>`: Export metrics (prometheus|influxdb)
/// - `--export-file <FILE>`: Export metrics to file
/// - `--history-db <PATH>`: Path to history database
/// - `-c, --config <FILE>`: Path to configuration file
/// - `--generate-config <FILE>`: Generate example configuration file
#[tokio::main]
async fn main() {
    let matches = Command::new("Linux Process Manager")
        .version("1.0")
        .author("CSCE 3401 Team")
        .about("A comprehensive process manager for Linux systems")
        .arg(
            Arg::new("refresh")
                .short('r')
                .long("refresh")
                .value_name("SECONDS")
                .help("Sets refresh interval in seconds")
                .value_parser(clap::value_parser!(u64)),
        )
        .arg(
            Arg::new("user")
                .short('u')
                .long("user")
                .value_name("USERNAME")
                .help("Filter processes by user"),
        )
        .arg(
            Arg::new("tree")
                .short('t')
                .long("tree")
                .help("Start in tree view mode")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("api")
                .long("api")
                .help("Start REST API server")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("api-port")
                .long("api-port")
                .value_name("PORT")
                .help("API server port (default: 8080)")
                .default_value("8080")
                .value_parser(clap::value_parser!(u16)),
        )
        .arg(
            Arg::new("export")
                .long("export")
                .value_name("FORMAT")
                .help("Export metrics (prometheus|influxdb)")
                .value_parser(["prometheus", "influxdb"]),
        )
        .arg(
            Arg::new("export-file")
                .long("export-file")
                .value_name("FILE")
                .help("Export metrics to file"),
        )
        .arg(
            Arg::new("history-db")
                .long("history-db")
                .value_name("PATH")
                .help("Path to history database")
                .default_value("process_history.db"),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Path to configuration file"),
        )
        .arg(
            Arg::new("generate-config")
                .long("generate-config")
                .value_name("FILE")
                .help("Generate example configuration file"),
        )
        .get_matches();

    // Initialize logging system early for audit trail and debugging
    // Uses default configuration: INFO level, daily rotation, logs to ~/.local/share/process-manager/logs
    use process_manager::logging::{LogConfig, init_logging, log_system_event};
    use tracing::Level;
    
    let log_config = LogConfig::default();
    if let Err(e) = init_logging(&log_config) {
        eprintln!("⚠️  Warning: Failed to initialize logging: {}", e);
        eprintln!("   Continuing without file logging...");
    }
    
    log_system_event("startup", "Linux Process Manager starting", Level::INFO);

    // Print application banner with version and team information
    println!("Linux Process Manager (LPM) v1.0");
    println!("CSCE 3401 - Operating Systems Fall 2025");
    println!("Team: Adam Aberbach, Mohammad Yahya Hammoudeh, Mohamed Khalil Brik, Ahmed Elaswar");
    println!();

    // Handle configuration file generation if requested
    // This creates an example config file with all available options
    if let Some(config_file) = matches.get_one::<String>("generate-config") {
        let path = std::path::PathBuf::from(config_file);
        match config::Config::create_example_config(&path) {
            Ok(_) => {
                println!("✓ Example configuration file created at: {}", config_file);
                println!("  Edit this file and use --config to load it.");
                return;
            }
            Err(e) => {
                eprintln!("✗ Failed to create config file: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Load configuration from file or use defaults
    // Priority: --config flag > default location (~/.config/lpm/config.toml) > built-in defaults
    let _config = if let Some(config_file) = matches.get_one::<String>("config") {
        let path = std::path::PathBuf::from(config_file);
        match config::Config::load_from_file(&path) {
            Ok(cfg) => {
                println!("✓ Loaded configuration from: {}", config_file);
                cfg
            }
            Err(e) => {
                eprintln!("✗ Failed to load config file: {}", e);
                eprintln!("  Using default configuration instead.");
                config::Config::default()
            }
        }
    } else {
        // Try to load from default location
        match config::Config::load() {
            Ok(cfg) => {
                println!("✓ Loaded configuration from default location");
                cfg
            }
            Err(_) => {
                // Use defaults silently
                config::Config::default()
            }
        }
    };

    // Handle metrics export mode (non-interactive)
    // Exports current system metrics in Prometheus or InfluxDB format
    if let Some(format) = matches.get_one::<String>("export") {
        handle_export_mode(format, matches.get_one::<String>("export-file"));
        return;
    }

    // Handle REST API server mode (long-running service)
    // Provides JSON API endpoints and serves the web UI
    if matches.get_flag("api") {
        let port = *matches.get_one::<u16>("api-port").unwrap();
        let bind_address = format!("0.0.0.0:{}", port);
        let history_db = matches.get_one::<String>("history-db").map(|s| s.to_string());
        
        println!("Starting REST API server mode...");
        println!("API will be available at http://localhost:{}/api", port);
        println!("Web UI available at: file://web/index.html");
        println!();
        
        let process_manager = process::ProcessManager::new();
        if let Err(e) = api::start_api_server(&bind_address, process_manager, history_db).await {
            eprintln!("API server error: {}", e);
            std::process::exit(1);
        }
        return;
    }

    // Default mode: Start interactive Terminal User Interface (TUI)
    // This is the primary mode for interactive process management
    println!("Starting interactive process manager...");
    println!("Press 'h' for help once the application starts.");
    println!();
    
    if let Err(err) = run_app() {
        eprintln!("Application error: {}", err);
        std::process::exit(1);
    }
}

/// Handles metrics export to Prometheus or InfluxDB format.
/// 
/// # Arguments
/// 
/// * `format` - Export format ("prometheus" or "influxdb")
/// * `output_file` - Optional file path to write metrics to (stdout if None)
/// 
/// # Behavior
/// 
/// 1. Refreshes process list to get current metrics
/// 2. Exports metrics in requested format
/// 3. Writes to file or stdout
/// 4. Exits the application
fn handle_export_mode(format: &str, output_file: Option<&String>) {
    use metrics::{ExportFormat, MetricsExporter};
    
    log_system_event("export", &format!("Exporting metrics in {} format", format), Level::INFO);
    println!("Exporting metrics in {} format...", format);
    
    let export_format = match format {
        "prometheus" => ExportFormat::Prometheus,
        "influxdb" => ExportFormat::InfluxDB,
        _ => {
            error!("Unknown export format: {}", format);
            eprintln!("Unknown export format: {}", format);
            std::process::exit(1);
        }
    };
    
    let mut process_manager = process::ProcessManager::new();
    if let Err(e) = process_manager.refresh() {
        error!("Failed to refresh processes during export: {}", e);
        eprintln!("Failed to refresh processes: {}", e);
        std::process::exit(1);
    }
    
    let exporter = MetricsExporter::new("lpm");
    
    if let Some(file_path) = output_file {
        debug!("Exporting metrics to file: {}", file_path);
        if let Err(e) = exporter.export_to_file(&process_manager, export_format, file_path) {
            error!("Failed to export to file: {}", e);
            eprintln!("Failed to export to file: {}", e);
            std::process::exit(1);
        }
        info!("Metrics exported successfully to: {}", file_path);
        println!("Metrics exported to: {}", file_path);
    } else {
        debug!("Exporting metrics to stdout");
        let metrics = exporter.export(&process_manager, export_format);
        println!("{}", metrics);
        info!("Metrics exported successfully to stdout");
    }
}
