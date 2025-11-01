//! # Linux Process Manager Library
//! 
//! A comprehensive process management library for Linux systems providing
//! real-time monitoring, control, and analysis capabilities.
//! 
//! This library can be used as a standalone crate or as part of the
//! interactive process manager application.
//! 
//! ## Modules
//! 
//! ### Core Modules
//! - [`process`] - Process discovery, monitoring, and signal control
//! - [`ui`] - Terminal user interface
//! - [`tree`] - Process tree hierarchy
//! - [`network`] - Network connections per process
//! - [`config`] - Configuration management
//! 
//! ### Advanced Modules
//! - [`gpu`] - GPU monitoring (NVIDIA, AMD, Intel)
//! - [`history`] - Historical data storage (SQLite)
//! - [`api`] - REST API server
//! - [`metrics`] - Prometheus/InfluxDB export
//! - [`anomaly`] - Anomaly detection
//! 
//! ### Phase IV Modules
//! - [`logging`] - Structured logging with rotation
//! - [`affinity`] - CPU affinity and priority management
//! - [`alerts`] - Smart alerting system
//! - [`snapshots`] - Process state capture and replay
//! - [`groups`] - Process group management
//! - [`memmap`] - Memory map visualization
//! - [`profiles`] - Saved view profiles
//! - [`diffing`] - Process state comparison
//! - [`containers`] - Container deep dive
//! 
//! ## Example
//! 
//! ```rust,ignore
//! use process_manager::process::ProcessManager;
//! 
//! fn main() -> anyhow::Result<()> {
//!     let mut manager = ProcessManager::new();
//!     manager.refresh()?;
//!     
//!     for process in manager.get_processes() {
//!         if process.cpu_usage > 50.0 {
//!             println!("High CPU: {} ({}%)", process.name, process.cpu_usage);
//!         }
//!     }
//!     
//!     Ok(())
//! }
//! ```

// Core modules
pub mod process;
pub mod ui;
pub mod tree;
pub mod network;
pub mod config;

// Advanced modules
pub mod gpu;
pub mod history;
pub mod api;
pub mod metrics;
pub mod anomaly;

// Phase IV modules
pub mod affinity;
pub mod alerts;
pub mod containers;
pub mod diffing;
pub mod groups;
pub mod logging;
pub mod memmap;
pub mod profiles;
pub mod snapshots;
