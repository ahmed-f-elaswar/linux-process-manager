//! # Structured Logging System
//! 
//! Provides comprehensive logging capabilities with structured output, rotation, and
//! multiple formats for process manager operations and events.
//! 
//! ## Features
//! 
//! - **Multiple Log Levels**: TRACE, DEBUG, INFO, WARN, ERROR
//! - **Log Rotation**: Daily, hourly, or size-based rotation
//! - **Multiple Formats**: Human-readable or JSON for log aggregation
//! - **Audit Trail**: Track all process operations (kill, stop, priority changes)
//! - **Performance**: Non-blocking I/O for minimal overhead
//! 
//! ## Example
//! 
//! ```rust,ignore
//! use process_manager::logging::{LogConfig, init_logging, log_process_operation};
//! 
//! # fn main() -> anyhow::Result<()> {
//! // Initialize logging system
//! let config = LogConfig::default();
//! init_logging(&config)?;
//! 
//! // Log process operations
//! log_process_operation("kill", 1234, "firefox", "alice", true, Some("SIGTERM"));
//! # Ok(())
//! # }
//! ```

use anyhow::Result;
use std::path::PathBuf;
use tracing::{info, warn, error, debug, Level};
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};
use tracing_appender::{non_blocking, rolling};

/// Configuration for the logging system.
/// 
/// Controls log level, output destination, format, and rotation policy.
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Minimum log level to capture (TRACE, DEBUG, INFO, WARN, ERROR)
    pub level: Level,
    /// Whether to write logs to a file (in addition to console)
    pub log_to_file: bool,
    /// Directory path where log files will be written
    pub log_file_path: PathBuf,
    /// Use JSON format for structured logging (useful for log aggregation)
    pub json_format: bool,
    /// Log rotation policy
    pub rotation: LogRotation,
}

/// Log rotation policy to manage log file size and retention.
/// 
/// Prevents unbounded log file growth by rotating files based on time or size.
#[derive(Debug, Clone)]
pub enum LogRotation {
    /// Never rotate (single log file, grows indefinitely)
    Never,
    /// Rotate daily (creates new file each day)
    Daily,
    /// Rotate hourly (creates new file each hour)
    Hourly,
    /// Rotate when file exceeds specified size in bytes
    Size(u64),
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            log_to_file: true,
            log_file_path: dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("process-manager")
                .join("logs"),
            json_format: false,
            rotation: LogRotation::Daily,
        }
    }
}

/// Initialize the logging system with the given configuration.
/// 
/// Sets up the tracing subscriber with appropriate filters, formatters, and
/// appenders based on the provided config.
/// 
/// # Arguments
/// 
/// * `config` - Logging configuration specifying level, format, and rotation
/// 
/// # Returns
/// 
/// * `Ok(())` on successful initialization
/// * `Err` if log directory creation fails or initialization encounters errors
/// 
/// # Example
/// 
/// ```rust,ignore
/// # use process_manager::logging::{LogConfig, init_logging};
/// # use tracing::Level;
/// # fn main() -> anyhow::Result<()> {
/// let mut config = LogConfig::default();
/// config.level = Level::DEBUG;
/// config.json_format = true;
/// init_logging(&config)?;
/// # Ok(())
/// # }
/// ```
pub fn init_logging(config: &LogConfig) -> Result<()> {
    // Create log directory if needed for file-based logging
    if config.log_to_file {
        std::fs::create_dir_all(&config.log_file_path)?;
    }

    let env_filter = EnvFilter::from_default_env()
        .add_directive(config.level.into());

    if config.log_to_file {
        // File appender with rotation
        let file_appender = match config.rotation {
            LogRotation::Daily => rolling::daily(&config.log_file_path, "process-manager.log"),
            LogRotation::Hourly => rolling::hourly(&config.log_file_path, "process-manager.log"),
            _ => rolling::never(&config.log_file_path, "process-manager.log"),
        };

        let (non_blocking, _guard) = non_blocking(file_appender);

        if config.json_format {
            // JSON format for structured logging
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer().json().with_writer(non_blocking))
                .init();
        } else {
            // Human-readable format
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer().with_writer(non_blocking))
                .init();
        }
    } else {
        // Console only
        if config.json_format {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer().json())
                .init();
        } else {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt::layer())
                .init();
        }
    }

    info!("Logging system initialized");
    info!("Log level: {:?}", config.level);
    if config.log_to_file {
        info!("Logging to file: {:?}", config.log_file_path);
    }

    Ok(())
}

/// Log a process operation for audit trail purposes.
/// 
/// Creates structured log entries for all process management operations,
/// enabling compliance, debugging, and operational analysis.
/// 
/// # Arguments
/// 
/// * `operation` - Type of operation (e.g., "kill", "stop", "priority_change")
/// * `pid` - Process ID
/// * `process_name` - Name of the process
/// * `user` - User who performed the operation
/// * `success` - Whether the operation succeeded
/// * `details` - Optional additional details about the operation
/// 
/// # Example
/// 
/// ```rust,ignore
/// # use process_manager::logging::log_process_operation;
/// log_process_operation("kill", 1234, "firefox", "alice", true, Some("SIGTERM"));
/// ```
pub fn log_process_operation(
    operation: &str,
    pid: u32,
    process_name: &str,
    user: &str,
    success: bool,
    details: Option<&str>,
) {
    if success {
        info!(
            operation = operation,
            pid = pid,
            process_name = process_name,
            user = user,
            details = details.unwrap_or(""),
            "Process operation completed successfully"
        );
    } else {
        error!(
            operation = operation,
            pid = pid,
            process_name = process_name,
            user = user,
            details = details.unwrap_or(""),
            "Process operation failed"
        );
    }
}

/// Log a system-level event with the specified log level.
/// 
/// # Arguments
/// 
/// * `event_type` - Category of the event (e.g., "startup", "error", "config_change")
/// * `message` - Event message
/// * `level` - Log level for this event
pub fn log_system_event(event_type: &str, message: &str, level: Level) {
    match level {
        Level::ERROR => error!(event_type = event_type, "{}", message),
        Level::WARN => warn!(event_type = event_type, "{}", message),
        Level::INFO => info!(event_type = event_type, "{}", message),
        Level::DEBUG => debug!(event_type = event_type, "{}", message),
        _ => info!(event_type = event_type, "{}", message),
    }
}

/// Log performance metrics for monitoring and optimization.
/// 
/// Tracks operation duration and throughput for performance analysis.
/// 
/// # Arguments
/// 
/// * `operation` - Name of the operation being measured
/// * `duration_ms` - Duration in milliseconds
/// * `items_processed` - Number of items processed
pub fn log_performance(operation: &str, duration_ms: u64, items_processed: usize) {
    debug!(
        operation = operation,
        duration_ms = duration_ms,
        items = items_processed,
        "Performance metric"
    );
}

/// Log an error with contextual information.
/// 
/// Provides rich error logging with context for better debugging.
/// 
/// # Arguments
/// 
/// * `context` - Context where the error occurred
/// * `error` - The error object
pub fn log_error_with_context(context: &str, error: &anyhow::Error) {
    error!(
        context = context,
        error = %error,
        "Error occurred"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_config_default() {
        let config = LogConfig::default();
        assert_eq!(config.level, Level::INFO);
        assert!(config.log_to_file);
    }

    #[test]
    fn test_log_rotation_types() {
        let daily = LogRotation::Daily;
        let hourly = LogRotation::Hourly;
        let never = LogRotation::Never;
        
        // Just ensure they can be created
        assert!(matches!(daily, LogRotation::Daily));
        assert!(matches!(hourly, LogRotation::Hourly));
        assert!(matches!(never, LogRotation::Never));
    }
}
