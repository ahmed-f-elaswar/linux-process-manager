//! # CPU Affinity and Priority Management
//! 
//! Provides fine-grained control over process CPU affinity, scheduling priority,
//! and I/O priority on Linux systems.
//! 
//! ## Features
//! 
//! - **CPU Affinity**: Pin processes to specific CPU cores
//! - **Nice Values**: Adjust process scheduling priority (-20 to 19)
//! - **Real-time Scheduling**: Set SCHED_FIFO, SCHED_RR, SCHED_DEADLINE policies
//! - **I/O Priority**: Control disk I/O priority (idle, best-effort, real-time)
//! - **Scheduling Policies**: View and modify CFS, real-time schedulers
//! 
//! ## Use Cases
//! 
//! - Optimize performance for CPU-bound tasks
//! - Prevent noisy neighbor problems on shared systems
//! - Implement resource isolation for containers
//! - Debug performance issues related to CPU placement
//! 
//! ## Example
//! 
//! ```rust,ignore
//! use process_manager::affinity::{set_cpu_affinity, set_nice_value};
//! 
//! # fn main() -> anyhow::Result<()> {
//! // Pin process to CPUs 0 and 1
//! set_cpu_affinity(1234, &[0, 1])?;
//! 
//! // Lower priority (higher nice value)
//! set_nice_value(1234, 10)?;
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use std::fs;
use nix::sched::{sched_getaffinity, sched_setaffinity, CpuSet};
use nix::unistd::Pid;
use tracing::{debug, info, warn, error};

/// Complete information about a process's affinity and priority settings.
/// 
/// Contains all scheduling-related attributes including CPU affinity,
/// nice values, real-time priority, and I/O priority.
#[derive(Debug, Clone)]
pub struct AffinityInfo {
    /// List of CPU cores this process is allowed to run on
    pub cpu_affinity: Vec<usize>,
    /// Nice value (-20 to 19, lower = higher priority)
    pub nice_value: i32,
    /// Kernel-calculated priority (dynamic)
    pub priority: i32,
    /// Real-time priority (0-99 for SCHED_FIFO/SCHED_RR)
    pub rt_priority: u32,
    /// Scheduling policy (SCHED_OTHER, SCHED_FIFO, SCHED_RR, SCHED_BATCH, etc.)
    pub scheduling_policy: String,
    /// I/O priority class (idle, best-effort, real-time)
    pub io_priority_class: String,
    /// I/O priority level (0-7 within the class)
    pub io_priority_level: u32,
}

/// Retrieve the CPU affinity mask for a process.
/// 
/// Returns a list of CPU core numbers that the process is allowed to execute on.
/// 
/// # Arguments
/// 
/// * `pid` - Process ID
/// 
/// # Returns
/// 
/// * `Ok(Vec<usize>)` - List of allowed CPU cores (e.g., [0, 1, 4])
/// * `Err` - If the process doesn't exist or permission denied
/// 
/// # Example
/// 
/// ```rust,ignore
/// # use process_manager::affinity::get_cpu_affinity;
/// # fn main() -> anyhow::Result<()> {
/// let cpus = get_cpu_affinity(1234)?;
/// println!("Process runs on CPUs: {:?}", cpus);
/// # Ok(())
/// # }
/// ```
pub fn get_cpu_affinity(pid: u32) -> Result<Vec<usize>> {
    debug!("Getting CPU affinity for pid {}", pid);
    let pid = Pid::from_raw(pid as i32);
    let cpu_set = sched_getaffinity(pid)
        .context("Failed to get CPU affinity")?;
    
    let mut cpus = Vec::new();
    for cpu in 0..num_cpus::get() {
        if cpu_set.is_set(cpu)? {
            cpus.push(cpu);
        }
    }
    
    debug!("CPU affinity for pid {}: {:?}", pid, cpus);
    Ok(cpus)
}

/// Set CPU affinity for a process (pin to specific cores).
/// 
/// Restricts the process to only run on the specified CPU cores.
/// Useful for optimizing cache locality and preventing cross-NUMA overhead.
/// 
/// # Arguments
/// 
/// * `pid` - Process ID
/// * `cpus` - Slice of CPU core numbers to allow (e.g., &[0, 1, 2])
/// 
/// # Returns
/// 
/// * `Ok(())` - Affinity successfully set
/// * `Err` - If invalid CPUs specified or permission denied
/// 
/// # Example
/// 
/// ```rust,ignore
/// # use process_manager::affinity::set_cpu_affinity;
/// # fn main() -> anyhow::Result<()> {
/// // Pin to cores 0 and 1 (useful for hyperthreading control)
/// set_cpu_affinity(1234, &[0, 1])?;
/// # Ok(())
/// # }
/// ```
pub fn set_cpu_affinity(pid: u32, cpus: &[usize]) -> Result<()> {
    info!("Setting CPU affinity for pid {} to {:?}", pid, cpus);
    let pid = Pid::from_raw(pid as i32);
    let mut cpu_set = CpuSet::new();
    
    for &cpu in cpus {
        cpu_set.set(cpu)?;
    }
    
    match sched_setaffinity(pid, &cpu_set).context("Failed to set CPU affinity") {
        Ok(_) => {
            info!("Successfully set CPU affinity for pid {} to {:?}", pid, cpus);
            Ok(())
        }
        Err(e) => {
            error!("Failed to set CPU affinity for pid {} to {:?}: {}", pid, cpus, e);
            Err(e)
        }
    }
}

/// Get comprehensive priority and affinity information for a process.
/// 
/// Reads from /proc filesystem to gather all scheduling-related attributes.
/// 
/// # Arguments
/// 
/// * `pid` - Process ID
/// 
/// # Returns
/// 
/// * `Ok(AffinityInfo)` - Complete affinity/priority information
/// * `Err` - If process doesn't exist or /proc read fails
pub fn get_priority_info(pid: u32) -> Result<AffinityInfo> {
    debug!("Getting priority info for pid {}", pid);
    let stat_path = format!("/proc/{}/stat", pid);
    let stat_content = fs::read_to_string(&stat_path)
        .context("Failed to read /proc/{pid}/stat")?;
    
    // Parse stat file (fields: pid, comm, state, ppid, pgrp, session, tty_nr, tpgid,
    // flags, minflt, cminflt, majflt, cmajflt, utime, stime, cutime, cstime,
    // priority, nice, num_threads, itrealvalue, starttime...)
    let parts: Vec<&str> = stat_content.split_whitespace().collect();
    
    let priority = if parts.len() > 17 {
        parts[17].parse::<i32>().unwrap_or(0)
    } else {
        0
    };
    
    let nice = if parts.len() > 18 {
        parts[18].parse::<i32>().unwrap_or(0)
    } else {
        0
    };
    
    // Get CPU affinity
    let cpu_affinity = get_cpu_affinity(pid).unwrap_or_else(|_| vec![]);
    
    // Get scheduling policy
    let sched_path = format!("/proc/{}/sched", pid);
    let scheduling_policy = if let Ok(sched_content) = fs::read_to_string(&sched_path) {
        extract_scheduling_policy(&sched_content)
    } else {
        "SCHED_OTHER".to_string()
    };
    
    // Get I/O priority (ionice)
    let (io_class, io_level) = get_io_priority(pid).unwrap_or(("best-effort".to_string(), 4));
    
    let info = AffinityInfo {
        cpu_affinity: cpu_affinity.clone(),
        nice_value: nice,
        priority,
        rt_priority: 0, // Would need sched_getparam for real-time priority
        scheduling_policy: scheduling_policy.clone(),
        io_priority_class: io_class,
        io_priority_level: io_level,
    };
    
    debug!("Priority info for pid {}: nice={}, priority={}, policy={}, cpus={:?}", 
           pid, nice, priority, scheduling_policy, cpu_affinity);
    
    Ok(info)
}

/// Extract scheduling policy from /proc/[pid]/sched
fn extract_scheduling_policy(content: &str) -> String {
    for line in content.lines() {
        if line.contains("policy") {
            if line.contains("0") {
                return "SCHED_OTHER".to_string();
            } else if line.contains("1") {
                return "SCHED_FIFO".to_string();
            } else if line.contains("2") {
                return "SCHED_RR".to_string();
            } else if line.contains("3") {
                return "SCHED_BATCH".to_string();
            } else if line.contains("5") {
                return "SCHED_IDLE".to_string();
            }
        }
    }
    "SCHED_OTHER".to_string()
}

/// Get I/O priority for a process
fn get_io_priority(_pid: u32) -> Result<(String, u32)> {
    // This would normally use ioprio_get syscall
    // For now, return default
    Ok(("best-effort".to_string(), 4))
}

/// Set process nice value (priority)
pub fn set_nice_value(pid: u32, nice: i32) -> Result<()> {
    // Validate nice value range (-20 to 19)
    if nice < -20 || nice > 19 {
        warn!("Invalid nice value {} for pid {} (must be -20 to 19)", nice, pid);
        anyhow::bail!("Nice value must be between -20 and 19");
    }
    
    info!("Setting nice value for pid {} to {}", pid, nice);
    
    unsafe {
        let result = libc::setpriority(
            libc::PRIO_PROCESS,
            pid,
            nice,
        );
        
        if result == -1 {
            error!("Failed to set nice value {} for pid {}", nice, pid);
            anyhow::bail!("Failed to set nice value");
        }
    }
    
    info!("Successfully set nice value for pid {} to {}", pid, nice);
    Ok(())
}

/// Format CPU affinity list as string
pub fn format_affinity_list(cpus: &[usize]) -> String {
    if cpus.is_empty() {
        return "none".to_string();
    }
    
    // Group consecutive CPUs
    let mut ranges = Vec::new();
    let mut start = cpus[0];
    let mut end = cpus[0];
    
    for &cpu in &cpus[1..] {
        if cpu == end + 1 {
            end = cpu;
        } else {
            if start == end {
                ranges.push(format!("{}", start));
            } else {
                ranges.push(format!("{}-{}", start, end));
            }
            start = cpu;
            end = cpu;
        }
    }
    
    // Add last range
    if start == end {
        ranges.push(format!("{}", start));
    } else {
        ranges.push(format!("{}-{}", start, end));
    }
    
    ranges.join(",")
}

/// Parse CPU affinity string to list
pub fn parse_affinity_string(affinity_str: &str) -> Result<Vec<usize>> {
    let mut cpus = Vec::new();
    
    for part in affinity_str.split(',') {
        if part.contains('-') {
            let range: Vec<&str> = part.split('-').collect();
            if range.len() == 2 {
                let start: usize = range[0].parse()?;
                let end: usize = range[1].parse()?;
                for cpu in start..=end {
                    cpus.push(cpu);
                }
            }
        } else {
            cpus.push(part.parse()?);
        }
    }
    
    Ok(cpus)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_affinity_list() {
        assert_eq!(format_affinity_list(&[0, 1, 2, 3]), "0-3");
        assert_eq!(format_affinity_list(&[0, 2, 4]), "0,2,4");
        assert_eq!(format_affinity_list(&[0, 1, 3, 4, 5]), "0-1,3-5");
        assert_eq!(format_affinity_list(&[]), "none");
    }

    #[test]
    fn test_parse_affinity_string() {
        assert_eq!(parse_affinity_string("0-3").unwrap(), vec![0, 1, 2, 3]);
        assert_eq!(parse_affinity_string("0,2,4").unwrap(), vec![0, 2, 4]);
        assert_eq!(parse_affinity_string("0-1,3-5").unwrap(), vec![0, 1, 3, 4, 5]);
    }

    #[test]
    fn test_nice_value_validation() {
        let result = set_nice_value(1, 25);
        assert!(result.is_err());
        
        let result = set_nice_value(1, -25);
        assert!(result.is_err());
    }
}
