use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    
    #[serde(default)]
    pub ui: UiConfig,
    
    #[serde(default)]
    pub api: ApiConfig,
    
    #[serde(default)]
    pub history: HistoryConfig,
    
    #[serde(default)]
    pub alerts: AlertConfig,
    
    #[serde(default)]
    pub features: FeatureConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Default refresh interval in seconds
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval: u64,
    
    /// Show only user processes by default
    #[serde(default)]
    pub show_only_user_processes: bool,
    
    /// Default sort column (pid, name, cpu, memory)
    #[serde(default = "default_sort_column")]
    pub default_sort_column: String,
    
    /// Sort in ascending order
    #[serde(default)]
    pub sort_ascending: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Start in tree view mode
    #[serde(default)]
    pub start_in_tree_view: bool,
    
    /// Show system resource graphs by default
    #[serde(default = "default_true")]
    pub show_graphs: bool,
    
    /// Color scheme (dark, light, custom)
    #[serde(default = "default_color_scheme")]
    pub color_scheme: String,
    
    /// Number of processes to display per page
    #[serde(default = "default_page_size")]
    pub page_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Enable API server on startup
    #[serde(default)]
    pub enabled: bool,
    
    /// API server port
    #[serde(default = "default_api_port")]
    pub port: u16,
    
    /// API server bind address
    #[serde(default = "default_bind_address")]
    pub bind_address: String,
    
    /// Enable CORS
    #[serde(default = "default_true")]
    pub enable_cors: bool,
    
    /// Auto-record history when API is enabled
    #[serde(default = "default_true")]
    pub auto_record_history: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryConfig {
    /// Enable historical data collection
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Database file path
    #[serde(default = "default_db_path")]
    pub database_path: String,
    
    /// Data retention period in days
    #[serde(default = "default_retention_days")]
    pub retention_days: i64,
    
    /// Recording interval in seconds
    #[serde(default = "default_recording_interval")]
    pub recording_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Enable alert system
    #[serde(default)]
    pub enabled: bool,
    
    /// CPU usage threshold (percentage)
    #[serde(default = "default_cpu_threshold")]
    pub cpu_threshold: f32,
    
    /// Memory usage threshold (percentage)
    #[serde(default = "default_memory_threshold")]
    pub memory_threshold: f32,
    
    /// Alert sound enabled
    #[serde(default)]
    pub sound_enabled: bool,
    
    /// Bookmarked processes to watch
    #[serde(default)]
    pub bookmarked_processes: Vec<BookmarkedProcess>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkedProcess {
    pub name: String,
    pub alert_on_exit: bool,
    pub alert_on_high_cpu: bool,
    pub alert_on_high_memory: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    /// Enable GPU monitoring
    #[serde(default = "default_true")]
    pub gpu_monitoring: bool,
    
    /// Enable network monitoring
    #[serde(default = "default_true")]
    pub network_monitoring: bool,
    
    /// Enable container detection
    #[serde(default = "default_true")]
    pub container_detection: bool,
    
    /// Enable anomaly detection
    #[serde(default = "default_true")]
    pub anomaly_detection: bool,
}

// Default value functions
fn default_refresh_interval() -> u64 { 2 }
fn default_sort_column() -> String { "cpu".to_string() }
fn default_color_scheme() -> String { "dark".to_string() }
fn default_page_size() -> usize { 50 }
fn default_api_port() -> u16 { 8080 }
fn default_bind_address() -> String { "127.0.0.1".to_string() }
fn default_db_path() -> String { 
    if let Some(home) = dirs::data_local_dir() {
        home.join("process-manager").join("history.db").to_string_lossy().to_string()
    } else {
        "process_history.db".to_string()
    }
}
fn default_retention_days() -> i64 { 30 }
fn default_recording_interval() -> u64 { 60 }
fn default_cpu_threshold() -> f32 { 80.0 }
fn default_memory_threshold() -> f32 { 85.0 }
fn default_true() -> bool { true }

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            ui: UiConfig::default(),
            api: ApiConfig::default(),
            history: HistoryConfig::default(),
            alerts: AlertConfig::default(),
            features: FeatureConfig::default(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            refresh_interval: default_refresh_interval(),
            show_only_user_processes: false,
            default_sort_column: default_sort_column(),
            sort_ascending: false,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            start_in_tree_view: false,
            show_graphs: true,
            color_scheme: default_color_scheme(),
            page_size: default_page_size(),
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: default_api_port(),
            bind_address: default_bind_address(),
            enable_cors: true,
            auto_record_history: true,
        }
    }
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            database_path: default_db_path(),
            retention_days: default_retention_days(),
            recording_interval: default_recording_interval(),
        }
    }
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cpu_threshold: default_cpu_threshold(),
            memory_threshold: default_memory_threshold(),
            sound_enabled: false,
            bookmarked_processes: Vec::new(),
        }
    }
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            gpu_monitoring: true,
            network_monitoring: true,
            container_detection: true,
            anomaly_detection: true,
        }
    }
}

impl Config {
    /// Load configuration from file, or create default if not found
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        debug!("Loading config from {:?}", config_path);
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .context("Failed to read config file")?;
            let config: Config = toml::from_str(&content)
                .context("Failed to parse config file")?;
            info!("Loaded configuration from {:?}", config_path);
            Ok(config)
        } else {
            // Create default config
            warn!("Config file not found, creating default at {:?}", config_path);
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }
    
    /// Load from specific file
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path)
            .context("Failed to read config file")?;
        let config: Config = toml::from_str(&content)
            .context("Failed to parse config file")?;
        Ok(config)
    }
    
    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        debug!("Saving config to {:?}", config_path);
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }
        
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        fs::write(&config_path, content)
            .context("Failed to write config file")?;
        
        Ok(())
    }
    
    /// Get the default config file path
    pub fn get_config_path() -> Result<PathBuf> {
        if let Some(config_dir) = dirs::config_dir() {
            Ok(config_dir.join("process-manager").join("config.toml"))
        } else {
            Ok(PathBuf::from("process-manager.toml"))
        }
    }
    
    /// Create example configuration file
    pub fn create_example_config(path: &PathBuf) -> Result<()> {
        let config = Config::default();
        let content = toml::to_string_pretty(&config)
            .context("Failed to serialize config")?;
        fs::write(path, content)
            .context("Failed to write example config")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.general.refresh_interval, 2);
        assert_eq!(config.api.port, 8080);
        assert_eq!(config.history.retention_days, 30);
        assert!(config.features.gpu_monitoring);
    }
    
    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("[general]"));
        assert!(toml_str.contains("[api]"));
        assert!(toml_str.contains("[history]"));
    }
    
    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
            [general]
            refresh_interval = 5
            show_only_user_processes = true
            
            [api]
            enabled = true
            port = 9090
        "#;
        
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.general.refresh_interval, 5);
        assert_eq!(config.general.show_only_user_processes, true);
        assert_eq!(config.api.enabled, true);
        assert_eq!(config.api.port, 9090);
    }
    
    #[test]
    fn test_bookmarked_process() {
        let bookmark = BookmarkedProcess {
            name: "nginx".to_string(),
            alert_on_exit: true,
            alert_on_high_cpu: false,
            alert_on_high_memory: false,
        };
        
        assert_eq!(bookmark.name, "nginx");
        assert!(bookmark.alert_on_exit);
    }
}
