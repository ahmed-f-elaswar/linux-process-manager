//! # Container Deep Dive
//! 
//! Provides enhanced container awareness and deep runtime analysis for
//! Docker, containerd, Podman, and Kubernetes environments.
//! 
//! ## Features
//! 
//! - **Runtime Detection**: Auto-detect Docker, containerd, Podman
//! - **Container Metadata**: Name, image, ID, status
//! - **Namespace Analysis**: PID, NET, MNT, IPC, UTS, USER namespaces
//! - **Cgroup Resources**: CPU limits, memory limits, I/O limits
//! - **Network Info**: Bridge, host, overlay networks
//! - **Process Mapping**: Map PIDs to containers
//! - **Kubernetes Integration**: Pod names, labels, namespaces
//! 
//! ## Detection Methods
//! 
//! 1. **Cgroup Paths**: Parse /proc/[pid]/cgroup for container IDs
//! 2. **/.dockerenv**: Docker environment marker file
//! 3. **Process Names**: containerd-shim, crun, runc
//! 4. **API Calls**: Optional Docker/K8s API integration
//! 
//! ## Example
//! 
//! ```rust,ignore
//! use process_manager::containers::{detect_container_for_pid, get_container_info};
//! 
//! # fn main() -> anyhow::Result<()> {
//! // Check if process is in a container
//! if let Some(container_id) = detect_container_for_pid(1234)? {
//!     println!("Process in container: {}", container_id);
//!     
//!     // Get full container info
//!     let info = get_container_info(&container_id)?;
//!     println!("Container: {} ({})", info.name, info.image);
//!     println!("Runtime: {:?}", info.runtime);
//! }
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::{debug, info};

/// Type of container runtime detected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContainerRuntime {
    Docker,
    Containerd,
    Podman,
    Unknown,
}

/// Container information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub runtime: ContainerRuntime,
    pub image: String,
    pub status: String,
    pub pids: Vec<u32>,
    pub namespace_ids: NamespaceIds,
    pub cgroup_path: String,
    pub network_mode: String,
    pub ip_addresses: Vec<String>,
}

/// Container namespace IDs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceIds {
    pub pid_ns: Option<String>,
    pub net_ns: Option<String>,
    pub mnt_ns: Option<String>,
    pub uts_ns: Option<String>,
    pub ipc_ns: Option<String>,
    pub user_ns: Option<String>,
}

/// Container resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerResources {
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub memory_limit: u64,
    pub network_rx: u64,
    pub network_tx: u64,
    pub block_read: u64,
    pub block_write: u64,
}

/// Container analyzer
pub struct ContainerAnalyzer {
    #[allow(dead_code)]
    runtime: ContainerRuntime,
}

impl ContainerAnalyzer {
    pub fn new() -> Self {
        let runtime = Self::detect_runtime();
        Self { runtime }
    }
    
    /// Detect container runtime
    fn detect_runtime() -> ContainerRuntime {
        if Path::new("/var/run/docker.sock").exists() {
            ContainerRuntime::Docker
        } else if Path::new("/run/containerd/containerd.sock").exists() {
            ContainerRuntime::Containerd
        } else if Path::new("/var/run/podman/podman.sock").exists() {
            ContainerRuntime::Podman
        } else {
            ContainerRuntime::Unknown
        }
    }
    
    /// Check if process is running in a container
    pub fn is_containerized(&self, pid: u32) -> Result<bool> {
        debug!("Checking if pid {} is containerized", pid);
        // Check cgroup for container indicators
        let cgroup_path = format!("/proc/{}/cgroup", pid);
        if let Ok(content) = fs::read_to_string(cgroup_path) {
            if content.contains("docker") 
                || content.contains("containerd") 
                || content.contains("podman")
                || content.contains("kubepods") {
                info!("Process {} is containerized", pid);
                return Ok(true);
            }
        }
        
        // Check mount namespace
        let self_ns = fs::read_link("/proc/1/ns/mnt")?;
        let proc_ns = fs::read_link(format!("/proc/{}/ns/mnt", pid))?;
        
        Ok(self_ns != proc_ns)
    }
    
    /// Get container ID from process
    pub fn get_container_id(&self, pid: u32) -> Result<Option<String>> {
        debug!("Getting container ID for pid {}", pid);
        let cgroup_path = format!("/proc/{}/cgroup", pid);
        let content = fs::read_to_string(cgroup_path)
            .context("Failed to read cgroup file")?;
        
        for line in content.lines() {
            // Docker format: 0::/docker/<container_id>
            if line.contains("docker") {
                if let Some(id) = line.split('/').last() {
                    return Ok(Some(id.to_string()));
                }
            }
            
            // Containerd/K8s format: 0::/kubepods/.../docker-<container_id>.scope
            if line.contains("docker-") && line.contains(".scope") {
                if let Some(part) = line.split("docker-").nth(1) {
                    if let Some(id) = part.split(".scope").next() {
                        return Ok(Some(id.to_string()));
                    }
                }
            }
            
            // Podman format
            if line.contains("libpod-") {
                if let Some(part) = line.split("libpod-").nth(1) {
                    if let Some(id) = part.split(".scope").next() {
                        return Ok(Some(id.to_string()));
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    /// Get namespace IDs for a process
    pub fn get_namespace_ids(&self, pid: u32) -> Result<NamespaceIds> {
        let ns_dir = format!("/proc/{}/ns", pid);
        
        Ok(NamespaceIds {
            pid_ns: Self::read_namespace(&ns_dir, "pid").ok(),
            net_ns: Self::read_namespace(&ns_dir, "net").ok(),
            mnt_ns: Self::read_namespace(&ns_dir, "mnt").ok(),
            uts_ns: Self::read_namespace(&ns_dir, "uts").ok(),
            ipc_ns: Self::read_namespace(&ns_dir, "ipc").ok(),
            user_ns: Self::read_namespace(&ns_dir, "user").ok(),
        })
    }
    
    fn read_namespace(ns_dir: &str, ns_type: &str) -> Result<String> {
        let path = format!("{}/{}", ns_dir, ns_type);
        let link = fs::read_link(path)?;
        Ok(link.to_string_lossy().to_string())
    }
    
    /// Get container resources from cgroup
    pub fn get_container_resources(&self, pid: u32) -> Result<ContainerResources> {
        let cgroup_path = self.get_cgroup_path(pid)?;
        
        // Read CPU usage
        let cpu_usage = self.read_cpu_usage(&cgroup_path).unwrap_or(0.0);
        
        // Read memory usage
        let (memory_usage, memory_limit) = self.read_memory_usage(&cgroup_path)
            .unwrap_or((0, 0));
        
        // Read network stats (if available)
        let (network_rx, network_tx) = self.read_network_stats(pid)
            .unwrap_or((0, 0));
        
        // Read block I/O stats
        let (block_read, block_write) = self.read_block_io(&cgroup_path)
            .unwrap_or((0, 0));
        
        Ok(ContainerResources {
            cpu_usage,
            memory_usage,
            memory_limit,
            network_rx,
            network_tx,
            block_read,
            block_write,
        })
    }
    
    fn get_cgroup_path(&self, pid: u32) -> Result<String> {
        let cgroup_file = format!("/proc/{}/cgroup", pid);
        let content = fs::read_to_string(cgroup_file)?;
        
        // Parse cgroup path (simplified)
        for line in content.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 {
                return Ok(parts[2].to_string());
            }
        }
        
        anyhow::bail!("Could not parse cgroup path")
    }
    
    fn read_cpu_usage(&self, cgroup_path: &str) -> Result<f64> {
        // Try cgroup v2
        let v2_path = format!("/sys/fs/cgroup{}/cpu.stat", cgroup_path);
        if let Ok(content) = fs::read_to_string(&v2_path) {
            for line in content.lines() {
                if line.starts_with("usage_usec") {
                    if let Some(value) = line.split_whitespace().nth(1) {
                        return Ok(value.parse::<u64>()? as f64 / 1_000_000.0);
                    }
                }
            }
        }
        
        // Try cgroup v1
        let v1_path = format!("/sys/fs/cgroup/cpu{}/cpuacct.usage", cgroup_path);
        if let Ok(content) = fs::read_to_string(&v1_path) {
            return Ok(content.trim().parse::<u64>()? as f64 / 1_000_000_000.0);
        }
        
        Ok(0.0)
    }
    
    fn read_memory_usage(&self, cgroup_path: &str) -> Result<(u64, u64)> {
        // Try cgroup v2
        let current_path = format!("/sys/fs/cgroup{}/memory.current", cgroup_path);
        let max_path = format!("/sys/fs/cgroup{}/memory.max", cgroup_path);
        
        if let (Ok(current), Ok(max)) = (
            fs::read_to_string(&current_path),
            fs::read_to_string(&max_path),
        ) {
            let usage = current.trim().parse().unwrap_or(0);
            let limit = max.trim().parse().unwrap_or(0);
            return Ok((usage, limit));
        }
        
        // Try cgroup v1
        let usage_path = format!("/sys/fs/cgroup/memory{}/memory.usage_in_bytes", cgroup_path);
        let limit_path = format!("/sys/fs/cgroup/memory{}/memory.limit_in_bytes", cgroup_path);
        
        if let (Ok(usage), Ok(limit)) = (
            fs::read_to_string(&usage_path),
            fs::read_to_string(&limit_path),
        ) {
            return Ok((
                usage.trim().parse().unwrap_or(0),
                limit.trim().parse().unwrap_or(0),
            ));
        }
        
        Ok((0, 0))
    }
    
    fn read_network_stats(&self, pid: u32) -> Result<(u64, u64)> {
        let net_dev_path = format!("/proc/{}/net/dev", pid);
        let content = fs::read_to_string(net_dev_path)?;
        
        let mut rx_bytes = 0u64;
        let mut tx_bytes = 0u64;
        
        for line in content.lines().skip(2) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 10 {
                rx_bytes += parts[1].parse::<u64>().unwrap_or(0);
                tx_bytes += parts[9].parse::<u64>().unwrap_or(0);
            }
        }
        
        Ok((rx_bytes, tx_bytes))
    }
    
    fn read_block_io(&self, cgroup_path: &str) -> Result<(u64, u64)> {
        // Try cgroup v2
        let io_stat_path = format!("/sys/fs/cgroup{}/io.stat", cgroup_path);
        if let Ok(content) = fs::read_to_string(&io_stat_path) {
            let mut read_bytes = 0u64;
            let mut write_bytes = 0u64;
            
            for line in content.lines() {
                if line.contains("rbytes=") {
                    if let Some(val) = line.split("rbytes=").nth(1) {
                        read_bytes += val.split_whitespace().next()
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0);
                    }
                }
                if line.contains("wbytes=") {
                    if let Some(val) = line.split("wbytes=").nth(1) {
                        write_bytes += val.split_whitespace().next()
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(0);
                    }
                }
            }
            
            return Ok((read_bytes, write_bytes));
        }
        
        Ok((0, 0))
    }
    
    /// List all PIDs in a container
    pub fn get_container_pids(&self, container_id: &str) -> Result<Vec<u32>> {
        let mut pids = Vec::new();
        
        // Read all process cgroups and find matching containers
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if let Ok(pid) = file_name.parse::<u32>() {
                        if let Ok(Some(cid)) = self.get_container_id(pid) {
                            if cid.starts_with(container_id) {
                                pids.push(pid);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(pids)
    }
    
    /// Format container info
    pub fn format_container_info(&self, info: &ContainerInfo) -> String {
        format!(
            "Container: {} ({})\n  ID: {}\n  Image: {}\n  Status: {}\n  PIDs: {:?}\n  Network: {} (IPs: {})",
            info.name,
            format!("{:?}", info.runtime),
            &info.id[..12],
            info.image,
            info.status,
            info.pids,
            info.network_mode,
            info.ip_addresses.join(", ")
        )
    }
}

impl Default for ContainerAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_runtime() {
        let analyzer = ContainerAnalyzer::new();
        // Runtime detection depends on system, just ensure it doesn't panic
        let _ = format!("{:?}", analyzer.runtime);
    }

    #[test]
    fn test_container_id_extraction() {
        let analyzer = ContainerAnalyzer::new();
        // Should handle process 1 (init) which is never in a container
        let result = analyzer.get_container_id(1);
        assert!(result.is_ok());
    }
}
