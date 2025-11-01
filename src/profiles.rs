//! # Saved View Profiles
//! 
//! Save and load custom view configurations combining filters, sorting, columns,
//! and display preferences for quick access to frequently-used views.
//! 
//! ## Features
//! 
//! - **Predefined Profiles**: CPU Hogs, Memory Hogs, My Processes, Active, System
//! - **Custom Profiles**: Create and save your own views
//! - **Filter Combinations**: User, CPU%, memory, name patterns
//! - **Column Selection**: Choose which metrics to display
//! - **Highlight Rules**: Color-code processes based on conditions
//! - **Quick Switching**: Hotkeys to switch between profiles
//! 
//! ## Built-in Profiles
//! 
//! 1. **CPU Hogs**: Processes using >10% CPU, sorted by CPU usage
//! 2. **Memory Hogs**: Processes using >100MB, sorted by memory
//! 3. **My Processes**: Only current user's processes
//! 4. **Active**: Running/sleeping processes (not zombies)
//! 5. **System**: System processes (root-owned)
//! 
//! ## Example
//! 
//! ```rust,ignore
//! use process_manager::profiles::{ViewProfile, ProfileManager};
//! 
//! # fn main() -> anyhow::Result<()> {
//! let mut manager = ProfileManager::new("~/.config/lpm/profiles");
//! 
//! // Load a built-in profile
//! let cpu_hogs = manager.get_profile("cpu_hogs")?;
//! println!("Using profile: {}", cpu_hogs.name);
//! 
//! // Create custom profile
//! let my_profile = ViewProfile::new("web_servers", "Web server processes");
//! manager.save_profile(&my_profile)?;
//! # Ok(())
//! # }
//! ```

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info};

/// Complete view configuration profile.
/// 
/// Encapsulates all display settings, filters, and sorting preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewProfile {
    pub name: String,
    pub description: String,
    pub columns: Vec<String>,
    pub sort_by: String,
    pub sort_order: SortOrder,
    pub filters: Vec<ProcessFilter>,
    pub refresh_interval: u64,  // milliseconds
    pub tree_mode: bool,
    pub show_threads: bool,
    pub highlight_rules: Vec<HighlightRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// Process filter criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessFilter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    GreaterThan,
    LessThan,
    GreaterOrEqual,
    LessOrEqual,
}

/// Highlight rule for processes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightRule {
    pub name: String,
    pub condition: ProcessFilter,
    pub color: String,
    pub bold: bool,
}

/// Manager for view profiles
pub struct ViewProfileManager {
    profiles: HashMap<String, ViewProfile>,
    config_dir: PathBuf,
    active_profile: Option<String>,
}

impl ViewProfileManager {
    /// Create new profile manager
    pub fn new(config_dir: PathBuf) -> Result<Self> {
        debug!("Creating view profile manager with config dir: {:?}", config_dir);
        let mut manager = Self {
            profiles: HashMap::new(),
            config_dir,
            active_profile: None,
        };
        
        // Create config directory if it doesn't exist
        fs::create_dir_all(&manager.config_dir)?;
        
        // Load default profiles
        manager.load_default_profiles();
        
        // Load custom profiles from disk
        manager.load_profiles()?;
        
        info!("Loaded {} view profiles", manager.profiles.len());
        Ok(manager)
    }
    
    /// Load default built-in profiles
    fn load_default_profiles(&mut self) {
        // System Overview
        self.profiles.insert(
            "system_overview".to_string(),
            ViewProfile {
                name: "System Overview".to_string(),
                description: "General system processes view".to_string(),
                columns: vec![
                    "pid".to_string(),
                    "name".to_string(),
                    "cpu".to_string(),
                    "memory".to_string(),
                    "user".to_string(),
                ],
                sort_by: "cpu".to_string(),
                sort_order: SortOrder::Descending,
                filters: vec![],
                refresh_interval: 1000,
                tree_mode: false,
                show_threads: false,
                highlight_rules: vec![
                    HighlightRule {
                        name: "High CPU".to_string(),
                        condition: ProcessFilter {
                            field: "cpu".to_string(),
                            operator: FilterOperator::GreaterThan,
                            value: "50".to_string(),
                        },
                        color: "red".to_string(),
                        bold: true,
                    },
                ],
            },
        );
        
        // Memory Intensive
        self.profiles.insert(
            "memory_intensive".to_string(),
            ViewProfile {
                name: "Memory Intensive".to_string(),
                description: "Processes sorted by memory usage".to_string(),
                columns: vec![
                    "pid".to_string(),
                    "name".to_string(),
                    "memory".to_string(),
                    "memory_percent".to_string(),
                    "rss".to_string(),
                    "vms".to_string(),
                ],
                sort_by: "memory".to_string(),
                sort_order: SortOrder::Descending,
                filters: vec![
                    ProcessFilter {
                        field: "memory".to_string(),
                        operator: FilterOperator::GreaterThan,
                        value: "10485760".to_string(), // 10 MB
                    },
                ],
                refresh_interval: 2000,
                tree_mode: false,
                show_threads: false,
                highlight_rules: vec![],
            },
        );
        
        // My Processes
        self.profiles.insert(
            "my_processes".to_string(),
            ViewProfile {
                name: "My Processes".to_string(),
                description: "Processes owned by current user".to_string(),
                columns: vec![
                    "pid".to_string(),
                    "name".to_string(),
                    "cpu".to_string(),
                    "memory".to_string(),
                    "command".to_string(),
                ],
                sort_by: "cpu".to_string(),
                sort_order: SortOrder::Descending,
                filters: vec![
                    ProcessFilter {
                        field: "user".to_string(),
                        operator: FilterOperator::Equals,
                        value: "$USER".to_string(), // Will be replaced at runtime
                    },
                ],
                refresh_interval: 1000,
                tree_mode: false,
                show_threads: false,
                highlight_rules: vec![],
            },
        );
        
        // Network Processes
        self.profiles.insert(
            "network_processes".to_string(),
            ViewProfile {
                name: "Network Processes".to_string(),
                description: "Processes with active network connections".to_string(),
                columns: vec![
                    "pid".to_string(),
                    "name".to_string(),
                    "network_rx".to_string(),
                    "network_tx".to_string(),
                    "connections".to_string(),
                ],
                sort_by: "network_tx".to_string(),
                sort_order: SortOrder::Descending,
                filters: vec![
                    ProcessFilter {
                        field: "connections".to_string(),
                        operator: FilterOperator::GreaterThan,
                        value: "0".to_string(),
                    },
                ],
                refresh_interval: 1000,
                tree_mode: false,
                show_threads: false,
                highlight_rules: vec![],
            },
        );
        
        // Process Tree
        self.profiles.insert(
            "process_tree".to_string(),
            ViewProfile {
                name: "Process Tree".to_string(),
                description: "Hierarchical process tree view".to_string(),
                columns: vec![
                    "pid".to_string(),
                    "name".to_string(),
                    "cpu".to_string(),
                    "memory".to_string(),
                ],
                sort_by: "pid".to_string(),
                sort_order: SortOrder::Ascending,
                filters: vec![],
                refresh_interval: 2000,
                tree_mode: true,
                show_threads: false,
                highlight_rules: vec![],
            },
        );
    }
    
    /// Load custom profiles from disk
    fn load_profiles(&mut self) -> Result<()> {
        let profiles_path = self.config_dir.join("view_profiles");
        
        if !profiles_path.exists() {
            return Ok(());
        }
        
        for entry in fs::read_dir(profiles_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = fs::read_to_string(&path)?;
                let profile: ViewProfile = serde_json::from_str(&content)?;
                let key = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                self.profiles.insert(key, profile);
            }
        }
        
        Ok(())
    }
    
    /// Save a profile to disk
    pub fn save_profile(&self, key: &str, profile: &ViewProfile) -> Result<()> {
        let profiles_dir = self.config_dir.join("view_profiles");
        fs::create_dir_all(&profiles_dir)?;
        
        let path = profiles_dir.join(format!("{}.json", key));
        let content = serde_json::to_string_pretty(profile)?;
        fs::write(path, content)?;
        
        Ok(())
    }
    
    /// Get a profile by key
    pub fn get_profile(&self, key: &str) -> Option<&ViewProfile> {
        self.profiles.get(key)
    }
    
    /// Get all profile keys
    pub fn get_profile_keys(&self) -> Vec<String> {
        self.profiles.keys().cloned().collect()
    }
    
    /// Get all profiles
    pub fn get_all_profiles(&self) -> &HashMap<String, ViewProfile> {
        &self.profiles
    }
    
    /// Add a new profile
    pub fn add_profile(&mut self, key: String, profile: ViewProfile) -> Result<()> {
        self.save_profile(&key, &profile)?;
        self.profiles.insert(key, profile);
        Ok(())
    }
    
    /// Delete a profile
    pub fn delete_profile(&mut self, key: &str) -> Result<()> {
        let path = self.config_dir.join("view_profiles").join(format!("{}.json", key));
        if path.exists() {
            fs::remove_file(path)?;
        }
        self.profiles.remove(key);
        Ok(())
    }
    
    /// Set active profile
    pub fn set_active_profile(&mut self, key: Option<String>) {
        self.active_profile = key;
    }
    
    /// Get active profile
    pub fn get_active_profile(&self) -> Option<&ViewProfile> {
        self.active_profile.as_ref()
            .and_then(|key| self.profiles.get(key))
    }
    
    /// Apply filters to check if process matches
    pub fn matches_filters(&self, filters: &[ProcessFilter], process_data: &HashMap<String, String>) -> bool {
        if filters.is_empty() {
            return true;
        }
        
        for filter in filters {
            if !self.matches_filter(filter, process_data) {
                return false;
            }
        }
        
        true
    }
    
    /// Check if process matches a single filter
    fn matches_filter(&self, filter: &ProcessFilter, process_data: &HashMap<String, String>) -> bool {
        let value = match process_data.get(&filter.field) {
            Some(v) => v,
            None => return false,
        };
        
        match filter.operator {
            FilterOperator::Equals => value == &filter.value,
            FilterOperator::NotEquals => value != &filter.value,
            FilterOperator::Contains => value.contains(&filter.value),
            FilterOperator::NotContains => !value.contains(&filter.value),
            FilterOperator::GreaterThan => {
                if let (Ok(v1), Ok(v2)) = (value.parse::<f64>(), filter.value.parse::<f64>()) {
                    v1 > v2
                } else {
                    false
                }
            }
            FilterOperator::LessThan => {
                if let (Ok(v1), Ok(v2)) = (value.parse::<f64>(), filter.value.parse::<f64>()) {
                    v1 < v2
                } else {
                    false
                }
            }
            FilterOperator::GreaterOrEqual => {
                if let (Ok(v1), Ok(v2)) = (value.parse::<f64>(), filter.value.parse::<f64>()) {
                    v1 >= v2
                } else {
                    false
                }
            }
            FilterOperator::LessOrEqual => {
                if let (Ok(v1), Ok(v2)) = (value.parse::<f64>(), filter.value.parse::<f64>()) {
                    v1 <= v2
                } else {
                    false
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_view_profile_creation() {
        let temp_dir = tempdir().unwrap();
        let manager = ViewProfileManager::new(temp_dir.path().to_path_buf()).unwrap();
        
        assert!(manager.get_profile("system_overview").is_some());
        assert!(manager.get_profile("memory_intensive").is_some());
    }

    #[test]
    fn test_filter_matching() {
        let temp_dir = tempdir().unwrap();
        let manager = ViewProfileManager::new(temp_dir.path().to_path_buf()).unwrap();
        
        let mut process_data = HashMap::new();
        process_data.insert("cpu".to_string(), "75".to_string());
        
        let filter = ProcessFilter {
            field: "cpu".to_string(),
            operator: FilterOperator::GreaterThan,
            value: "50".to_string(),
        };
        
        assert!(manager.matches_filter(&filter, &process_data));
    }
}
