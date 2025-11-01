use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use anyhow::Result;
use tracing::debug;

/// Network statistics for a process
#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    pub pid: u32,
    pub rx_bytes: u64,      // Received bytes
    pub tx_bytes: u64,      // Transmitted bytes
    pub rx_packets: u64,    // Received packets
    pub tx_packets: u64,    // Transmitted packets
    pub connections: usize, // Number of network connections
}

/// Container/cgroup information
#[derive(Debug, Clone, Default)]
pub struct CgroupInfo {
    pub pid: u32,
    pub container_id: Option<String>,
    pub container_name: Option<String>,
    pub cgroup_path: String,
    pub memory_limit: Option<u64>,    // Memory limit in bytes
    pub cpu_quota: Option<i64>,       // CPU quota
    pub cpu_period: Option<u64>,      // CPU period
    pub is_container: bool,
    pub pod_name: Option<String>,      // Kubernetes pod name
    pub namespace: Option<String>,     // Kubernetes namespace
}

/// Get network statistics for a process
pub fn get_network_stats(pid: u32) -> Result<NetworkStats> {
    debug!("Getting network stats for pid {}", pid);
    let mut stats = NetworkStats {
        pid,
        ..Default::default()
    };

    // Read network connections from /proc/net
    let connections = count_network_connections(pid)?;
    stats.connections = connections;
    debug!("Process {} has {} network connections", pid, connections);

    // Note: Per-process network bandwidth requires eBPF or netfilter
    // For now, we'll track connection count which is available from /proc
    
    Ok(stats)
}

/// Count network connections for a process
fn count_network_connections(pid: u32) -> Result<usize> {
    let fd_path = format!("/proc/{}/fd", pid);
    if !Path::new(&fd_path).exists() {
        return Ok(0);
    }

    let mut count = 0;
    
    // Read file descriptors
    if let Ok(entries) = fs::read_dir(&fd_path) {
        for entry in entries.flatten() {
            if let Ok(link) = fs::read_link(entry.path()) {
                let link_str = link.to_string_lossy();
                // Check if it's a socket
                if link_str.starts_with("socket:") {
                    count += 1;
                }
            }
        }
    }

    Ok(count)
}

/// Get cgroup information for a process
pub fn get_cgroup_info(pid: u32) -> Result<CgroupInfo> {
    debug!("Getting cgroup info for pid {}", pid);
    let mut info = CgroupInfo {
        pid,
        ..Default::default()
    };

    // Read cgroup information
    let cgroup_path = format!("/proc/{}/cgroup", pid);
    if !Path::new(&cgroup_path).exists() {
        return Ok(info);
    }

    let file = fs::File::open(&cgroup_path)?;
    let reader = BufReader::new(file);

    for line in reader.lines().flatten() {
        // Parse cgroup line format: hierarchy-ID:controller-list:cgroup-path
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 3 {
            let cgroup_path = parts[2];
            info.cgroup_path = cgroup_path.to_string();

            // Check if running in a container (Docker/Kubernetes)
            if cgroup_path.contains("/docker/") {
                info.is_container = true;
                // Extract container ID from path like /docker/abc123...
                if let Some(container_id) = extract_container_id(cgroup_path, "docker") {
                    info.container_id = Some(container_id);
                }
            } else if cgroup_path.contains("/kubepods/") {
                info.is_container = true;
                if let Some(container_id) = extract_container_id(cgroup_path, "kubepods") {
                    info.container_id = Some(container_id);
                }
            } else if cgroup_path.contains("/lxc/") {
                info.is_container = true;
                if let Some(container_id) = extract_container_id(cgroup_path, "lxc") {
                    info.container_id = Some(container_id);
                }
            }
        }
    }

    // Read memory limits from cgroup v2
    if let Ok(limit) = read_cgroup_memory_limit(pid) {
        info.memory_limit = Some(limit);
    }

    // Read CPU quota
    if let Ok((quota, period)) = read_cgroup_cpu_quota(pid) {
        info.cpu_quota = Some(quota);
        info.cpu_period = Some(period);
    }

    Ok(info)
}

fn extract_container_id(cgroup_path: &str, container_type: &str) -> Option<String> {
    // Extract container ID from cgroup path
    let parts: Vec<&str> = cgroup_path.split('/').collect();
    for (i, part) in parts.iter().enumerate() {
        if part.contains(container_type) && i + 1 < parts.len() {
            let id = parts[i + 1].to_string();
            // Take first 12 characters for Docker-style short ID
            return Some(if id.len() > 12 {
                id[..12].to_string()
            } else {
                id
            });
        }
    }
    None
}

fn read_cgroup_memory_limit(pid: u32) -> Result<u64> {
    // Try cgroup v2 first
    let v2_path = format!("/sys/fs/cgroup/system.slice/proc-{}.scope/memory.max", pid);
    if Path::new(&v2_path).exists() {
        if let Ok(content) = fs::read_to_string(&v2_path) {
            if let Ok(limit) = content.trim().parse::<u64>() {
                return Ok(limit);
            }
        }
    }

    // Try cgroup v1
    let v1_path = format!("/sys/fs/cgroup/memory/system.slice/proc-{}.scope/memory.limit_in_bytes", pid);
    if Path::new(&v1_path).exists() {
        if let Ok(content) = fs::read_to_string(&v1_path) {
            if let Ok(limit) = content.trim().parse::<u64>() {
                return Ok(limit);
            }
        }
    }

    Err(anyhow::anyhow!("Memory limit not found"))
}

fn read_cgroup_cpu_quota(pid: u32) -> Result<(i64, u64)> {
    // Try cgroup v2
    let v2_path = format!("/sys/fs/cgroup/system.slice/proc-{}.scope/cpu.max", pid);
    if Path::new(&v2_path).exists() {
        if let Ok(content) = fs::read_to_string(&v2_path) {
            let parts: Vec<&str> = content.trim().split_whitespace().collect();
            if parts.len() == 2 {
                let quota = parts[0].parse::<i64>().unwrap_or(-1);
                let period = parts[1].parse::<u64>().unwrap_or(100000);
                return Ok((quota, period));
            }
        }
    }

    // Try cgroup v1
    let quota_path = format!("/sys/fs/cgroup/cpu/system.slice/proc-{}.scope/cpu.cfs_quota_us", pid);
    let period_path = format!("/sys/fs/cgroup/cpu/system.slice/proc-{}.scope/cpu.cfs_period_us", pid);
    
    if Path::new(&quota_path).exists() && Path::new(&period_path).exists() {
        let quota = fs::read_to_string(&quota_path)?.trim().parse::<i64>()?;
        let period = fs::read_to_string(&period_path)?.trim().parse::<u64>()?;
        return Ok((quota, period));
    }

    Err(anyhow::anyhow!("CPU quota not found"))
}

/// Get Docker container name from container ID
pub fn get_docker_container_name(container_id: &str) -> Option<String> {
    // This requires Docker CLI or API access
    // For now, return the short ID
    Some(format!("container-{}", &container_id[..8]))
}

/// Format cgroup CPU percentage
pub fn format_cpu_limit(quota: i64, period: u64) -> String {
    if quota < 0 {
        "unlimited".to_string()
    } else {
        let limit_percent = (quota as f64 / period as f64) * 100.0;
        format!("{:.1}%", limit_percent)
    }
}

/// Format memory limit
pub fn format_memory_limit(limit: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if limit >= GB {
        format!("{:.1} GB", limit as f64 / GB as f64)
    } else if limit >= MB {
        format!("{:.1} MB", limit as f64 / MB as f64)
    } else if limit >= KB {
        format!("{:.1} KB", limit as f64 / KB as f64)
    } else {
        format!("{} B", limit)
    }
}

/// Kubernetes pod aggregation data
#[derive(Debug, Clone, Default)]
pub struct PodAggregation {
    pub pod_name: String,
    pub namespace: String,
    pub process_count: usize,
    pub total_cpu_usage: f32,
    pub total_memory_usage: u64,
    pub pids: Vec<u32>,
}

/// Aggregate processes by Kubernetes pod
pub fn aggregate_by_pod(cgroup_infos: &[(u32, CgroupInfo)]) -> Vec<PodAggregation> {
    use std::collections::HashMap;
    
    let mut pod_map: HashMap<(String, String), PodAggregation> = HashMap::new();
    
    for (pid, cgroup) in cgroup_infos {
        if let (Some(pod_name), Some(namespace)) = (&cgroup.pod_name, &cgroup.namespace) {
            let key = (pod_name.clone(), namespace.clone());
            
            let aggregation = pod_map.entry(key.clone()).or_insert_with(|| PodAggregation {
                pod_name: pod_name.clone(),
                namespace: namespace.clone(),
                process_count: 0,
                total_cpu_usage: 0.0,
                total_memory_usage: 0,
                pids: Vec::new(),
            });
            
            aggregation.process_count += 1;
            aggregation.pids.push(*pid);
        }
    }
    
    pod_map.into_values().collect()
}

/// Add CPU and memory usage to pod aggregation
pub fn aggregate_pod_resources(
    mut pod_agg: PodAggregation,
    process_list: &[(u32, f32, u64)], // (pid, cpu, memory)
) -> PodAggregation {
    for &(pid, cpu, memory) in process_list {
        if pod_agg.pids.contains(&pid) {
            pod_agg.total_cpu_usage += cpu;
            pod_agg.total_memory_usage += memory;
        }
    }
    pod_agg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_memory_limit() {
        assert_eq!(format_memory_limit(1024), "1.0 KB");
        assert_eq!(format_memory_limit(1024 * 1024), "1.0 MB");
        assert_eq!(format_memory_limit(2 * 1024 * 1024 * 1024), "2.0 GB");
    }

    #[test]
    fn test_format_cpu_limit() {
        assert_eq!(format_cpu_limit(-1, 100000), "unlimited");
        assert_eq!(format_cpu_limit(50000, 100000), "50.0%");
        assert_eq!(format_cpu_limit(200000, 100000), "200.0%");
    }

    #[test]
    fn test_extract_container_id() {
        let docker_path = "/docker/abc123def456/system.slice";
        assert_eq!(
            extract_container_id(docker_path, "docker"),
            Some("abc123def456".to_string())
        );

        let k8s_path = "/kubepods/pod123/xyz789";
        assert!(extract_container_id(k8s_path, "kubepods").is_some());
    }
}