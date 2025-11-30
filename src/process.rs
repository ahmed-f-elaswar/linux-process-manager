//! # Process Management Core
//! 
//! Core process monitoring and control functionality providing real-time
//! process information, signal control, and system-wide process management.
//! 
//! ## Features
//! 
//! - **Process Discovery**: Enumerate all processes with full metadata
//! - **Signal Control**: Send SIGTERM, SIGKILL, SIGSTOP, SIGCONT, etc.
//! - **Resource Metrics**: CPU, memory, threads, file descriptors
//! - **Container Detection**: Identify containerized processes
//! - **Network Tracking**: Connection counts per process
//! - **User Information**: UID, GID, username resolution
//! - **Real-time Updates**: Sub-second refresh capabilities
//! 
//! ## Example
//! 
//! ```rust,ignore
//! use process_manager::process::ProcessManager;
//! 
//! # fn main() -> anyhow::Result<()> {
//! let mut manager = ProcessManager::new();
//! manager.refresh()?;
//! 
//! // Get all processes
//! for process in manager.get_processes() {
//!     println!("{}: {} ({}%)", process.pid, process.name, process.cpu_usage);
//! }
//! 
//! // Control a process (SIGTERM = 15)
//! manager.kill_process(1234, 15)?;
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use anyhow::{Context, Result};
use sysinfo::{PidExt, ProcessExt, System, SystemExt};
use users;
use tracing::{debug, info, error};

/// Complete information about a single process.
/// 
/// Contains all available metadata including resource usage, ownership,
/// container status, and system resource limits.
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
    pub command: String,
    pub user: String,
    pub cpu_usage: f32,
    pub memory_usage: u64, // in KB
    pub memory_percent: f32,
    pub status: String,
    pub start_time: u64,
    pub running_time: Duration,
    pub uid: u32,
    pub gid: u32,
    pub threads: u32,
    pub priority: i32,
    pub nice: i32,
    // New fields for enhanced features
    pub network_connections: Option<usize>,
    pub is_container: bool,
    pub container_id: Option<String>,
    pub cgroup_memory_limit: Option<u64>,
    pub gpu_memory: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortColumn {
    Pid,
    Name,
    User,
    CpuUsage,
    MemoryUsage,
    MemoryPercent,
    StartTime,
}

/// Main process manager that maintains process state and provides control operations.
/// 
/// Uses sysinfo for cross-platform process discovery and /proc for Linux-specific details.
pub struct ProcessManager {
    system: System,
    processes: HashMap<u32, ProcessInfo>,
    last_update: SystemTime,
}

impl ProcessManager {
    /// Create a new process manager instance.
    /// 
    /// Initializes the system information gatherer and performs an initial refresh.
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        
        Self {
            system,
            processes: HashMap::new(),
            last_update: SystemTime::now(),
        }
    }

    /// Refresh process information from the system.
    /// 
    /// Scans all running processes and updates internal state with current metrics.
    /// Should be called periodically to keep data fresh.
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` on successful refresh
    /// * `Err` if system information cannot be accessed
    pub fn refresh(&mut self) -> Result<()> {
        debug!("Starting process refresh");
        let start = std::time::Instant::now();
        
        self.system.refresh_processes();
        self.system.refresh_cpu();
        self.system.refresh_memory();
        
        self.processes.clear();
        let mut errors = 0;
        
        for (pid, process) in self.system.processes() {
            match self.extract_process_info(pid.as_u32(), process) {
                Ok(process_info) => {
                    self.processes.insert(pid.as_u32(), process_info);
                }
                Err(e) => {
                    debug!("Failed to extract info for PID {}: {}", pid.as_u32(), e);
                    errors += 1;
                }
            }
        }
        
        self.last_update = SystemTime::now();
        let duration = start.elapsed();
        
        info!(
            process_count = self.processes.len(),
            errors = errors,
            duration_ms = duration.as_millis(),
            "Process refresh completed"
        );
        
        Ok(())
    }

    /// Extract detailed process information from sysinfo Process.
    /// 
    /// Combines sysinfo data with /proc filesystem reads for complete process details.
    fn extract_process_info(&self, pid: u32, process: &sysinfo::Process) -> Result<ProcessInfo> {
        // Get user info - sysinfo v0.29 doesn't have uid/gid methods on Process
        let user_id = self.get_process_uid(pid).unwrap_or(0);
        let group_id = self.get_process_gid(pid).unwrap_or(0);
        
        let user = users::get_user_by_uid(user_id)
            .map(|u| u.name().to_string_lossy().to_string())
            .unwrap_or_else(|| user_id.to_string());

        let start_time = process.start_time();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let running_time = Duration::from_secs(now.saturating_sub(start_time));

        // Try to get additional process info from /proc
        let (priority, nice, threads) = self.get_proc_info(pid).unwrap_or((0, 0, 1));

        // Get network and container info (optional)
        let network_connections = crate::network::get_network_stats(pid).ok()
            .map(|stats| stats.connections);
        
        let cgroup_info = crate::network::get_cgroup_info(pid).ok();
        let is_container = cgroup_info.as_ref().map(|c| c.is_container).unwrap_or(false);
        let container_id = cgroup_info.as_ref().and_then(|c| c.container_id.clone());
        let cgroup_memory_limit = cgroup_info.and_then(|c| c.memory_limit);
        
        // Get GPU info if available (optional)
        let gpu_memory = crate::gpu::get_nvidia_process_stats(pid).ok()
            .map(|stats| stats.gpu_memory_used);

        Ok(ProcessInfo {
            pid,
            ppid: process.parent().map(|p| p.as_u32()).unwrap_or(0),
            name: process.name().to_string(),
            command: process.cmd().join(" "),
            user,
            cpu_usage: process.cpu_usage(),
            memory_usage: process.memory(),
            memory_percent: (process.memory() as f32 / self.system.total_memory() as f32) * 100.0,
            status: format!("{:?}", process.status()),
            start_time,
            running_time,
            uid: user_id,
            gid: group_id,
            threads,
            priority,
            nice,
            network_connections,
            is_container,
            container_id,
            cgroup_memory_limit,
            gpu_memory,
        })
    }

    fn get_process_uid(&self, pid: u32) -> Result<u32> {
        let status_path = format!("/proc/{}/status", pid);
        if !Path::new(&status_path).exists() {
            return Ok(0);
        }

        let content = fs::read_to_string(&status_path)
            .context("Failed to read /proc/{pid}/status")?;
        
        for line in content.lines() {
            if line.starts_with("Uid:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 1 {
                    return Ok(parts[1].parse::<u32>().unwrap_or(0));
                }
            }
        }
        
        Ok(0)
    }

    fn get_process_gid(&self, pid: u32) -> Result<u32> {
        let status_path = format!("/proc/{}/status", pid);
        if !Path::new(&status_path).exists() {
            return Ok(0);
        }

        let content = fs::read_to_string(&status_path)
            .context("Failed to read /proc/{pid}/status")?;
        
        for line in content.lines() {
            if line.starts_with("Gid:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 1 {
                    return Ok(parts[1].parse::<u32>().unwrap_or(0));
                }
            }
        }
        
        Ok(0)
    }

    fn get_proc_info(&self, pid: u32) -> Result<(i32, i32, u32)> {
        let stat_path = format!("/proc/{}/stat", pid);
        if !Path::new(&stat_path).exists() {
            return Ok((0, 0, 1));
        }

        let content = fs::read_to_string(&stat_path)
            .context("Failed to read /proc/{pid}/stat")?;
        
        let fields: Vec<&str> = content.split_whitespace().collect();
        
        if fields.len() < 20 {
            return Ok((0, 0, 1));
        }

        let priority = fields[17].parse::<i32>().unwrap_or(0);
        let nice = fields[18].parse::<i32>().unwrap_or(0);
        let threads = fields[19].parse::<u32>().unwrap_or(1);

        Ok((priority, nice, threads))
    }

    pub fn get_processes(&self) -> Vec<&ProcessInfo> {
        self.processes.values().collect()
    }

    /// Get information about a specific process by PID.
    /// 
    /// # Arguments
    /// 
    /// * `pid` - Process ID to retrieve
    /// 
    /// # Returns
    /// 
    /// * `Some(&ProcessInfo)` if process exists
    /// * `None` if process not found or has terminated
    pub fn get_process(&self, pid: u32) -> Option<&ProcessInfo> {
        self.processes.get(&pid)
    }

    /// Send a signal to a process.
    /// 
    /// Common signals:
    /// - SIGTERM (15): Graceful termination request
    /// - SIGKILL (9): Forceful termination (cannot be caught)
    /// - SIGSTOP (19): Pause process execution
    /// - SIGCONT (18): Resume paused process
    /// 
    /// # Arguments
    /// 
    /// * `pid` - Process ID to signal
    /// * `signal` - Signal number (e.g., libc::SIGTERM)
    /// 
    /// # Returns
    /// 
    /// * `Ok(())` if signal sent successfully
    /// * `Err` if process doesn't exist or permission denied
    /// 
    /// # Safety
    /// 
    /// This function uses unsafe libc::kill() to send signals.
    pub fn kill_process(&self, pid: u32, signal: i32) -> Result<()> {
        debug!("Attempting to send signal {} to PID {}", signal, pid);
        
        unsafe {
            let result = libc::kill(pid as libc::pid_t, signal);
            if result == -1 {
                let errno = *libc::__errno_location();
                error!(
                    pid = pid,
                    signal = signal,
                    errno = errno,
                    "Failed to send signal to process"
                );
                return Err(anyhow::anyhow!(
                    "Failed to send signal {} to process {} (errno: {})", 
                    signal, pid, errno
                ));
            }
        }
        
        debug!("Successfully sent signal {} to PID {}", signal, pid);
        Ok(())
    }

    /// Filter processes based on criteria.
    /// 
    /// # Arguments
    /// 
    /// * `filter` - Filter criteria (user, name, CPU%, etc.)
    /// 
    /// # Returns
    /// 
    /// Vector of processes matching the filter
    pub fn filter_processes(&self, filter: &ProcessFilter) -> Vec<ProcessInfo> {
        self.processes
            .values()
            .filter(|p| filter.matches(p))
            .cloned()
            .collect()
    }
    
    /// Sort processes by a specific column.
    /// 
    /// # Arguments
    /// 
    /// * `column` - Column to sort by (PID, CPU, Memory, etc.)
    /// * `ascending` - true for ascending, false for descending
    /// 
    /// # Returns
    /// 
    /// Sorted vector of all processes
    pub fn sort_processes(&self, column: SortColumn, ascending: bool) -> Vec<ProcessInfo> {
        let mut processes: Vec<ProcessInfo> = self.processes.values().cloned().collect();
        
        processes.sort_by(|a, b| {
            let cmp = match column {
                SortColumn::Pid => a.pid.cmp(&b.pid),
                SortColumn::Name => a.name.cmp(&b.name),
                SortColumn::User => a.user.cmp(&b.user),
                SortColumn::CpuUsage => a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::MemoryUsage => a.memory_usage.cmp(&b.memory_usage),
                SortColumn::MemoryPercent => a.memory_percent.partial_cmp(&b.memory_percent).unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::StartTime => a.start_time.cmp(&b.start_time),
            };
            
            if ascending {
                cmp
            } else {
                cmp.reverse()
            }
        });
        
        processes
    }

    pub fn get_system_info(&self) -> SystemInfo {
        // Calculate used memory as total - available (excludes reclaimable cache/buffers)
        // This matches what GNOME System Monitor and htop show
        let total_memory = self.system.total_memory();
        let available_memory = self.system.available_memory();
        let used_memory = total_memory.saturating_sub(available_memory);

        SystemInfo {
            total_memory,
            used_memory,
            total_swap: self.system.total_swap(),
            used_swap: self.system.used_swap(),
            cpu_count: self.system.cpus().len(),
            load_average: self.system.load_average(),
            uptime: self.system.uptime(),
            hostname: self.system.host_name().unwrap_or_else(|| "unknown".to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub cpu_count: usize,
    pub load_average: sysinfo::LoadAvg,
    pub uptime: u64,
    pub hostname: String,
}

#[derive(Debug, Clone)]
pub struct ProcessFilter {
    pub username: Option<String>,
    pub name_pattern: Option<regex::Regex>,
    pub min_cpu_usage: Option<f32>,
    pub min_memory_usage: Option<u64>,
    pub show_only_user_processes: bool,
}

impl Default for ProcessFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessFilter {
    pub fn new() -> Self {
        Self {
            username: None,
            name_pattern: None,
            min_cpu_usage: None,
            min_memory_usage: None,
            show_only_user_processes: false,
        }
    }

    pub fn matches(&self, process: &ProcessInfo) -> bool {
        if let Some(ref user) = self.username {
            if process.user != *user {
                return false;
            }
        }

        if let Some(ref pattern) = self.name_pattern {
            if !pattern.is_match(&process.name) && !pattern.is_match(&process.command) {
                return false;
            }
        }

        if let Some(min_cpu) = self.min_cpu_usage {
            if process.cpu_usage < min_cpu {
                return false;
            }
        }

        if let Some(min_memory) = self.min_memory_usage {
            if process.memory_usage < min_memory {
                return false;
            }
        }

        if self.show_only_user_processes && process.uid == 0 {
            return false;
        }

        true
    }
}

// Signal constants
pub mod signals {
    pub const SIGTERM: i32 = 15;
    pub const SIGKILL: i32 = 9;
    pub const SIGHUP: i32 = 1;
    pub const SIGINT: i32 = 2;
    pub const SIGQUIT: i32 = 3;
    pub const SIGUSR1: i32 = 10;
    pub const SIGUSR2: i32 = 12;
    pub const SIGSTOP: i32 = 19;
    pub const SIGCONT: i32 = 18;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_manager_creation() {
        let manager = ProcessManager::new();
        // System should be initialized
        assert!(manager.system.name().is_some());
    }

    #[test]
    fn test_process_manager_refresh() {
        let mut manager = ProcessManager::new();
        let result = manager.refresh();
        assert!(result.is_ok(), "Refresh should succeed");
        assert!(manager.processes.len() > 0, "Should have at least one process");
    }

    #[test]
    fn test_signal_constants() {
        assert_eq!(signals::SIGTERM, 15);
        assert_eq!(signals::SIGKILL, 9);
        assert_eq!(signals::SIGHUP, 1);
        assert_eq!(signals::SIGINT, 2);
    }

    #[test]
    fn test_process_filter_default() {
        let filter = ProcessFilter::default();
        assert!(filter.username.is_none());
        assert!(filter.name_pattern.is_none());
        assert_eq!(filter.show_only_user_processes, false);
    }

    #[test]
    fn test_sort_column_enum() {
        let col1 = SortColumn::Pid;
        let col2 = SortColumn::Pid;
        assert_eq!(col1, col2);
    }

    #[test]
    fn test_get_system_info() {
        let manager = ProcessManager::new();
        let info = manager.get_system_info();
        
        assert!(info.cpu_count > 0);
        assert!(info.total_memory > 0);
        assert!(!info.hostname.is_empty());
    }
}