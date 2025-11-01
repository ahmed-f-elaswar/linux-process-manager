//! Integration tests for process manager new features

#[cfg(test)]
mod logging_tests {
    use process_manager::logging::*;
    use tempfile::tempdir;
    use tracing::Level;

    #[test]
    fn test_logging_initialization() {
        let temp_dir = tempdir().unwrap();
        let log_file = temp_dir.path().join("test.log");
        
        let config = LogConfig {
            level: Level::INFO,
            log_to_file: true,
            log_file_path: log_file,
            json_format: false,
            rotation: LogRotation::Daily,
        };
        
        let result = init_logging(&config);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_process_operation_logging() {
        log_process_operation("kill", 1234, "test_process", "testuser", true, Some("Test operation"));
        // Should not panic
    }
}

#[cfg(test)]
mod affinity_tests {
    use process_manager::affinity::*;

    #[test]
    fn test_get_cpu_affinity() {
        // Test with current process
        let result = get_cpu_affinity(std::process::id());
        assert!(result.is_ok());
        
        let affinity = result.unwrap();
        assert!(!affinity.is_empty());
    }
    
    #[test]
    fn test_priority_info() {
        let result = get_priority_info(std::process::id());
        assert!(result.is_ok());
        
        let info = result.unwrap();
        // Nice values range from -20 to 19
        assert!(info.nice_value >= -20 && info.nice_value <= 19);
    }
    
    #[test]
    fn test_format_affinity_list() {
        let formatted = format_affinity_list(&vec![0, 1, 2, 3, 7, 8, 9]);
        assert_eq!(formatted, "0-3,7-9");
    }
    
    #[test]
    fn test_parse_affinity_string() {
        let result = parse_affinity_string("0-3,5,7-9");
        assert!(result.is_ok());
        
        let cpus = result.unwrap();
        assert_eq!(cpus, vec![0, 1, 2, 3, 5, 7, 8, 9]);
    }
}

#[cfg(test)]
mod alerts_tests {
    use process_manager::alerts::*;
    use tokio;

    #[tokio::test]
    async fn test_alert_manager_creation() {
        let config = NotificationConfig {
            desktop: false,
            email: None,
            webhook: None,
        };
        let (_manager, _rx) = AlertManager::new(vec![], config);
        // Alert manager created successfully
    }
    
    #[tokio::test]
    async fn test_alert_rule_creation() {
        let rule = AlertRule {
            enabled: true,
            alert_type: AlertType::HighCpu,
            threshold: 80.0,
            duration_secs: 5,
            cooldown_secs: 60,
            process_filter: None,
        };
        
        let config = NotificationConfig {
            desktop: false,
            email: None,
            webhook: None,
        };
        let rules = vec![rule];
        let (_manager, _rx) = AlertManager::new(rules, config);
        // AlertManager created with rules successfully
    }
}

#[cfg(test)]
mod snapshots_tests {
    use process_manager::snapshots::*;
    use tempfile::tempdir;

    #[test]
    fn test_snapshot_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let manager = SnapshotManager::new(Some(temp_dir.path().to_path_buf()));
        assert!(manager.is_ok());
    }
    
    // Note: Snapshot capture tests require system process access
    // and are better tested in the module's own unit tests
}

#[cfg(test)]
mod groups_tests {
    use process_manager::groups::*;

    #[test]
    fn test_get_process_group_info() {
        // Test with init process (PID 1)
        let result = get_process_group_info(1);
        assert!(result.is_ok());
        
        let info = result.unwrap();
        assert_eq!(info.pid, 1);
        assert!(info.is_session_leader);
        assert!(info.is_group_leader);
    }
    
    #[test]
    fn test_tty_name_conversion() {
        assert_eq!(get_tty_name(0), "?");
        assert_eq!(get_tty_name(34816), "pts/0");
    }
    
    #[test]
    fn test_format_group_info() {
        let info = ProcessGroupInfo {
            pid: 1234,
            ppid: 1,
            pgid: 1234,
            sid: 1234,
            tty_nr: 0,
            tpgid: 0,
            is_session_leader: true,
            is_group_leader: true,
        };
        
        let formatted = format_group_info(&info);
        assert!(formatted.contains("1234"));
        assert!(formatted.contains("session leader"));
        assert!(formatted.contains("group leader"));
    }
}

#[cfg(test)]
mod memmap_tests {
    use process_manager::memmap::*;

    #[test]
    fn test_memory_map_visualizer() {
        // Test with current process
        let result = MemoryMapVisualizer::new(std::process::id());
        assert!(result.is_ok());
        
        let visualizer = result.unwrap();
        assert!(!visualizer.regions.is_empty());
        assert!(visualizer.total_size > 0);
    }
    
    #[test]
    fn test_get_regions_by_type() {
        let visualizer = MemoryMapVisualizer::new(std::process::id()).unwrap();
        
        let code_regions = visualizer.get_regions_by_type("code");
        let data_regions = visualizer.get_regions_by_type("data");
        
        // Should have some code and data regions
        assert!(!code_regions.is_empty());
        assert!(!data_regions.is_empty());
    }
    
    #[test]
    fn test_ascii_visualization() {
        let visualizer = MemoryMapVisualizer::new(std::process::id()).unwrap();
        let ascii = visualizer.visualize_ascii(80);
        
        assert!(ascii.contains("Memory Map"));
        assert!(ascii.contains("Code"));
        assert!(ascii.contains("Data"));
    }
    
    #[test]
    fn test_csv_export() {
        let visualizer = MemoryMapVisualizer::new(std::process::id()).unwrap();
        let csv = visualizer.export_csv();
        
        assert!(csv.contains("Start,End,Size"));
    }
}

#[cfg(test)]
mod profiles_tests {
    use process_manager::profiles::*;
    use tempfile::tempdir;
    use std::collections::HashMap;

    #[test]
    fn test_view_profile_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let manager = ViewProfileManager::new(temp_dir.path().to_path_buf());
        assert!(manager.is_ok());
        
        let manager = manager.unwrap();
        assert!(!manager.get_profile_keys().is_empty());
    }
    
    #[test]
    fn test_default_profiles_loaded() {
        let temp_dir = tempdir().unwrap();
        let manager = ViewProfileManager::new(temp_dir.path().to_path_buf()).unwrap();
        
        assert!(manager.get_profile("system_overview").is_some());
        assert!(manager.get_profile("memory_intensive").is_some());
        assert!(manager.get_profile("my_processes").is_some());
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
        
        assert!(manager.matches_filters(&vec![filter], &process_data));
    }
    
    #[test]
    fn test_add_custom_profile() {
        let temp_dir = tempdir().unwrap();
        let mut manager = ViewProfileManager::new(temp_dir.path().to_path_buf()).unwrap();
        
        let profile = ViewProfile {
            name: "Custom".to_string(),
            description: "Test".to_string(),
            columns: vec!["pid".to_string()],
            sort_by: "pid".to_string(),
            sort_order: SortOrder::Ascending,
            filters: vec![],
            refresh_interval: 1000,
            tree_mode: false,
            show_threads: false,
            highlight_rules: vec![],
        };
        
        let result = manager.add_profile("custom".to_string(), profile);
        assert!(result.is_ok());
        assert!(manager.get_profile("custom").is_some());
    }
}

#[cfg(test)]
mod diffing_tests {
    use process_manager::diffing::*;

    #[test]
    fn test_process_differ_creation() {
        let _differ = ProcessDiffer::new();
        // ProcessDiffer created successfully with default thresholds
    }
    
    // Note: Full diff tests with ProcessState objects are in the module's unit tests
}

#[cfg(test)]
mod containers_tests {
    use process_manager::containers::*;

    #[test]
    fn test_container_analyzer_creation() {
        let _analyzer = ContainerAnalyzer::new();
        // ContainerAnalyzer created successfully
    }
    
    #[test]
    fn test_is_containerized() {
        let analyzer = ContainerAnalyzer::new();
        
        // Test with init (should not be containerized)
        // Note: This test may fail if we don't have permission to read /proc/1/ns/mnt
        let result = analyzer.is_containerized(1);
        // Just check it doesn't panic, result may vary based on permissions
        let _ = result;
    }
    
    #[test]
    fn test_get_container_id() {
        let analyzer = ContainerAnalyzer::new();
        
        // Test with current process
        let result = analyzer.get_container_id(std::process::id());
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_get_namespace_ids() {
        let analyzer = ContainerAnalyzer::new();
        
        // Test with current process
        let result = analyzer.get_namespace_ids(std::process::id());
        assert!(result.is_ok());
        
        let ns_ids = result.unwrap();
        // Should have at least some namespace IDs
        assert!(ns_ids.pid_ns.is_some() || ns_ids.net_ns.is_some());
    }
}
