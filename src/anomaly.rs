// Anomaly detection module for identifying unusual process behavior
// Uses statistical analysis to detect CPU, memory, and other resource anomalies

use crate::process::ProcessInfo;
use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use tracing::{debug, info, warn};

/// Anomaly type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnomalyType {
    CpuSpike,
    MemorySpike,
    SuddenTermination,
    RapidRespawn,
    ExcessiveNetworkConnections,
    UnusualGpuUsage,
}

/// Detected anomaly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub anomaly_type: AnomalyType,
    pub pid: u32,
    pub process_name: String,
    pub severity: f32, // 0.0 to 1.0
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub current_value: f64,
    pub expected_value: f64,
    pub threshold: f64,
}

/// Historical data point for a process
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ProcessDataPoint {
    cpu_usage: f32,
    memory_usage: u64,
    network_connections: Option<usize>,
    gpu_memory: Option<u64>,
    timestamp: DateTime<Utc>,
}

/// Process statistics tracker
#[derive(Debug, Clone)]
struct ProcessStats {
    data_points: VecDeque<ProcessDataPoint>,
    max_history: usize,
}

impl ProcessStats {
    fn new(max_history: usize) -> Self {
        Self {
            data_points: VecDeque::with_capacity(max_history),
            max_history,
        }
    }
    
    fn add_data_point(&mut self, data: ProcessDataPoint) {
        self.data_points.push_back(data);
        if self.data_points.len() > self.max_history {
            self.data_points.pop_front();
        }
    }
    
    fn calculate_cpu_stats(&self) -> (f32, f32) {
        if self.data_points.is_empty() {
            return (0.0, 0.0);
        }
        
        let values: Vec<f32> = self.data_points.iter().map(|p| p.cpu_usage).collect();
        let mean = values.iter().sum::<f32>() / values.len() as f32;
        
        let variance = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f32>() / values.len() as f32;
        let std_dev = variance.sqrt();
        
        (mean, std_dev)
    }
    
    fn calculate_memory_stats(&self) -> (f64, f64) {
        if self.data_points.is_empty() {
            return (0.0, 0.0);
        }
        
        let values: Vec<f64> = self.data_points.iter()
            .map(|p| p.memory_usage as f64)
            .collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        
        let variance = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();
        
        (mean, std_dev)
    }
}

/// Anomaly detector configuration
#[derive(Debug, Clone)]
pub struct AnomalyDetectorConfig {
    pub cpu_threshold_sigma: f32,
    pub memory_threshold_sigma: f32,
    pub network_connection_threshold: usize,
    pub min_data_points: usize,
    pub history_size: usize,
}

impl Default for AnomalyDetectorConfig {
    fn default() -> Self {
        Self {
            cpu_threshold_sigma: 3.0,      // 3 standard deviations
            memory_threshold_sigma: 3.0,    // 3 standard deviations
            network_connection_threshold: 100,
            min_data_points: 10,
            history_size: 60,               // Keep 60 data points (e.g., 1 hour at 1 min intervals)
        }
    }
}

/// Main anomaly detector
pub struct AnomalyDetector {
    config: AnomalyDetectorConfig,
    process_history: HashMap<u32, ProcessStats>,
    process_names: HashMap<u32, String>,
    last_seen_processes: HashMap<u32, DateTime<Utc>>,
    detected_anomalies: VecDeque<Anomaly>,
    max_anomaly_history: usize,
}

impl AnomalyDetector {
    pub fn new(config: AnomalyDetectorConfig) -> Self {
        Self {
            config,
            process_history: HashMap::new(),
            process_names: HashMap::new(),
            last_seen_processes: HashMap::new(),
            detected_anomalies: VecDeque::new(),
            max_anomaly_history: 1000,
        }
    }
    
    /// Update with current process list and detect anomalies
    pub fn update(&mut self, processes: &[ProcessInfo]) -> Vec<Anomaly> {
        debug!("Anomaly detector analyzing {} processes", processes.len());
        let mut new_anomalies = Vec::new();
        let now = Utc::now();
        
        // Track currently active PIDs
        let mut active_pids = std::collections::HashSet::new();
        
        for process in processes {
            active_pids.insert(process.pid);
            
            // Store process name
            self.process_names.insert(process.pid, process.name.clone());
            
            // Create data point
            let data_point = ProcessDataPoint {
                cpu_usage: process.cpu_usage,
                memory_usage: process.memory_usage,
                network_connections: process.network_connections,
                gpu_memory: process.gpu_memory,
                timestamp: now,
            };
            
            // Get or create stats tracker
            let stats = self.process_history
                .entry(process.pid)
                .or_insert_with(|| ProcessStats::new(self.config.history_size));
            
            stats.add_data_point(data_point);
            
            // Update last seen timestamp
            self.last_seen_processes.insert(process.pid, now);
        }
        
        // Now check for anomalies without holding mutable borrows
        for process in processes {
            if let Some(stats) = self.process_history.get(&process.pid) {
                // Only check for anomalies if we have enough data
                if stats.data_points.len() >= self.config.min_data_points {
                    // Check CPU anomalies
                    if let Some(anomaly) = self.check_cpu_anomaly(process, stats, now) {
                        new_anomalies.push(anomaly);
                    }
                    
                    // Check memory anomalies
                    if let Some(anomaly) = self.check_memory_anomaly(process, stats, now) {
                        new_anomalies.push(anomaly);
                    }
                    
                    // Check network connection anomalies
                    if let Some(anomaly) = self.check_network_anomaly(process, now) {
                        new_anomalies.push(anomaly);
                    }
                }
            }
        }
        
        // Check for sudden terminations
        let terminated_pids: Vec<u32> = self.last_seen_processes.keys()
            .filter(|pid| !active_pids.contains(pid))
            .copied()
            .collect();
        
        for pid in terminated_pids {
            if let Some(anomaly) = self.check_sudden_termination(pid, now) {
                new_anomalies.push(anomaly);
            }
            // Clean up old data
            self.last_seen_processes.remove(&pid);
            // Keep stats for potential respawn detection
        }
        
        // Store detected anomalies
        for anomaly in &new_anomalies {
            self.detected_anomalies.push_back(anomaly.clone());
            if self.detected_anomalies.len() > self.max_anomaly_history {
                self.detected_anomalies.pop_front();
            }
        }
        
        if !new_anomalies.is_empty() {
            info!("Detected {} anomalies in current update", new_anomalies.len());
            for anomaly in &new_anomalies {
                match anomaly.anomaly_type {
                    AnomalyType::CpuSpike | AnomalyType::MemorySpike => {
                        warn!("{:?} detected for {} (pid {}): current={:.2}, expected={:.2}, severity={:.2}",
                              anomaly.anomaly_type, anomaly.process_name, anomaly.pid,
                              anomaly.current_value, anomaly.expected_value, anomaly.severity);
                    },
                    AnomalyType::SuddenTermination => {
                        warn!("Sudden termination detected for {} (pid {})", 
                              anomaly.process_name, anomaly.pid);
                    },
                    _ => {
                        info!("{:?} detected for {} (pid {})", 
                              anomaly.anomaly_type, anomaly.process_name, anomaly.pid);
                    }
                }
            }
        }
        
        new_anomalies
    }
    
    fn check_cpu_anomaly(
        &self,
        process: &ProcessInfo,
        stats: &ProcessStats,
        timestamp: DateTime<Utc>,
    ) -> Option<Anomaly> {
        let (mean, std_dev) = stats.calculate_cpu_stats();
        
        // Skip if not enough variance
        if std_dev < 1.0 {
            return None;
        }
        
        let z_score = (process.cpu_usage - mean) / std_dev;
        
        if z_score.abs() > self.config.cpu_threshold_sigma {
            let severity = (z_score.abs() / self.config.cpu_threshold_sigma).min(1.0);
            
            debug!("CPU anomaly detected for {} (pid {}): current={:.2}%, mean={:.2}%, std_dev={:.2}, z_score={:.2}",
                   process.name, process.pid, process.cpu_usage, mean, std_dev, z_score);
            
            return Some(Anomaly {
                anomaly_type: AnomalyType::CpuSpike,
                pid: process.pid,
                process_name: process.name.clone(),
                severity,
                description: format!(
                    "CPU usage {:.1}% is {:.1} standard deviations above mean {:.1}%",
                    process.cpu_usage, z_score, mean
                ),
                timestamp,
                current_value: process.cpu_usage as f64,
                expected_value: mean as f64,
                threshold: mean as f64 + (self.config.cpu_threshold_sigma * std_dev) as f64,
            });
        }
        
        None
    }
    
    fn check_memory_anomaly(
        &self,
        process: &ProcessInfo,
        stats: &ProcessStats,
        timestamp: DateTime<Utc>,
    ) -> Option<Anomaly> {
        let (mean, std_dev) = stats.calculate_memory_stats();
        
        // Skip if not enough variance
        if std_dev < 1024.0 { // 1 MB
            return None;
        }
        
        let z_score = (process.memory_usage as f64 - mean) / std_dev;
        
        if z_score.abs() > self.config.memory_threshold_sigma as f64 {
            let severity = (z_score.abs() / self.config.memory_threshold_sigma as f64).min(1.0) as f32;
            
            debug!("Memory anomaly detected for {} (pid {}): current={} KB, mean={:.0} KB, std_dev={:.0}, z_score={:.2}",
                   process.name, process.pid, process.memory_usage, mean, std_dev, z_score);
            
            return Some(Anomaly {
                anomaly_type: AnomalyType::MemorySpike,
                pid: process.pid,
                process_name: process.name.clone(),
                severity,
                description: format!(
                    "Memory usage {} KB is {:.1} standard deviations above mean {} KB",
                    process.memory_usage, z_score, mean as u64
                ),
                timestamp,
                current_value: process.memory_usage as f64,
                expected_value: mean,
                threshold: mean + (self.config.memory_threshold_sigma as f64 * std_dev),
            });
        }
        
        None
    }
    
    fn check_network_anomaly(
        &self,
        process: &ProcessInfo,
        timestamp: DateTime<Utc>,
    ) -> Option<Anomaly> {
        if let Some(connections) = process.network_connections {
            if connections > self.config.network_connection_threshold {
                let severity = (connections as f32 / (self.config.network_connection_threshold as f32 * 2.0)).min(1.0);
                
                debug!("Network anomaly detected for {} (pid {}): {} connections (threshold: {})",
                       process.name, process.pid, connections, self.config.network_connection_threshold);
                
                return Some(Anomaly {
                    anomaly_type: AnomalyType::ExcessiveNetworkConnections,
                    pid: process.pid,
                    process_name: process.name.clone(),
                    severity,
                    description: format!(
                        "Process has {} network connections (threshold: {})",
                        connections, self.config.network_connection_threshold
                    ),
                    timestamp,
                    current_value: connections as f64,
                    expected_value: self.config.network_connection_threshold as f64 / 2.0,
                    threshold: self.config.network_connection_threshold as f64,
                });
            }
        }
        
        None
    }
    
    fn check_sudden_termination(
        &mut self,
        pid: u32,
        timestamp: DateTime<Utc>,
    ) -> Option<Anomaly> {
        // Check if process had high resource usage before termination
        if let Some(stats) = self.process_history.get(&pid) {
            if stats.data_points.len() < 5 {
                return None;
            }
            
            let (cpu_mean, _) = stats.calculate_cpu_stats();
            let (mem_mean, _) = stats.calculate_memory_stats();
            
            // If process was using significant resources, flag termination
            if cpu_mean > 50.0 || mem_mean > 100_000.0 { // 100 MB
                let process_name = self.process_names.get(&pid)
                    .cloned()
                    .unwrap_or_else(|| format!("PID {}", pid));
                
                debug!("Sudden termination detected for {} (pid {}): was using {:.1}% CPU, {:.0} KB memory",
                       process_name, pid, cpu_mean, mem_mean);
                
                return Some(Anomaly {
                    anomaly_type: AnomalyType::SuddenTermination,
                    pid,
                    process_name,
                    severity: 0.5,
                    description: format!(
                        "Process terminated suddenly (was using {:.1}% CPU, {} KB memory)",
                        cpu_mean, mem_mean as u64
                    ),
                    timestamp,
                    current_value: 0.0,
                    expected_value: 1.0,
                    threshold: 1.0,
                });
            }
        }
        
        None
    }
    
    /// Get recent anomalies
    pub fn get_recent_anomalies(&self, count: usize) -> Vec<Anomaly> {
        self.detected_anomalies.iter()
            .rev()
            .take(count)
            .cloned()
            .collect()
    }
    
    /// Get anomalies for a specific process
    pub fn get_process_anomalies(&self, pid: u32) -> Vec<Anomaly> {
        self.detected_anomalies.iter()
            .filter(|a| a.pid == pid)
            .cloned()
            .collect()
    }
    
    /// Clear old anomaly history
    pub fn clear_old_anomalies(&mut self, older_than: DateTime<Utc>) {
        self.detected_anomalies.retain(|a| a.timestamp > older_than);
    }
    
    /// Get statistics summary
    pub fn get_stats(&self) -> AnomalyStats {
        let mut cpu_spikes = 0;
        let mut memory_spikes = 0;
        let mut network_anomalies = 0;
        let mut terminations = 0;
        
        for anomaly in &self.detected_anomalies {
            match anomaly.anomaly_type {
                AnomalyType::CpuSpike => cpu_spikes += 1,
                AnomalyType::MemorySpike => memory_spikes += 1,
                AnomalyType::ExcessiveNetworkConnections => network_anomalies += 1,
                AnomalyType::SuddenTermination => terminations += 1,
                _ => {}
            }
        }
        
        AnomalyStats {
            total_anomalies: self.detected_anomalies.len(),
            cpu_spikes,
            memory_spikes,
            network_anomalies,
            sudden_terminations: terminations,
            tracked_processes: self.process_history.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AnomalyStats {
    pub total_anomalies: usize,
    pub cpu_spikes: usize,
    pub memory_spikes: usize,
    pub network_anomalies: usize,
    pub sudden_terminations: usize,
    pub tracked_processes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_process_stats_cpu_calculation() {
        let mut stats = ProcessStats::new(10);
        
        for i in 0..10 {
            stats.add_data_point(ProcessDataPoint {
                cpu_usage: 10.0 + i as f32,
                memory_usage: 1000,
                network_connections: None,
                gpu_memory: None,
                timestamp: Utc::now(),
            });
        }
        
        let (mean, _std_dev) = stats.calculate_cpu_stats();
        assert!((mean - 14.5).abs() < 0.1);
    }
    
    #[test]
    fn test_anomaly_detector_creation() {
        let config = AnomalyDetectorConfig::default();
        let detector = AnomalyDetector::new(config);
        assert_eq!(detector.detected_anomalies.len(), 0);
    }
}
