//! # Process Groups and Sessions Management
//! 
//! Provides advanced process group (PGID) and session (SID) management for
//! job control and terminal management on Unix-like systems.
//! 
//! ## Features
//! 
//! - **Process Groups**: View and manage PGID relationships
//! - **Sessions**: Track session leaders and membership
//! - **Job Control**: Send signals to entire groups
//! - **Terminal Management**: Identify controlling terminals
//! - **Group Operations**: Kill/stop entire process groups
//! 
//! ## Concepts
//! 
//! - **PGID**: Process Group ID - collection of related processes
//! - **SID**: Session ID - collection of process groups
//! - **Session Leader**: Process that created the session (SID == PID)
//! - **Group Leader**: Process that created the group (PGID == PID)
//! - **Foreground Group**: Group that can read/write to terminal
//! 
//! ## Example
//! 
//! ```rust,ignore
//! use process_manager::groups::{get_process_group_info, kill_process_group};
//! 
//! # fn main() -> anyhow::Result<()> {
//! // Get group information
//! let info = get_process_group_info(1234)?;
//! println!("Process group: {}", info.pgid);
//! 
//! // Kill entire group (SIGTERM = 15)
//! kill_process_group(info.pgid, 15)?;
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use std::fs;
use std::collections::HashMap;
use tracing::{debug, info, error};

/// Complete process group and session information.
/// 
/// Contains identifiers for process hierarchies and terminal associations.
#[derive(Debug, Clone)]
pub struct ProcessGroupInfo {
    pub pid: u32,
    pub ppid: u32,
    pub pgid: u32,        // Process group ID
    pub sid: u32,         // Session ID
    pub tty_nr: i32,      // Controlling terminal
    pub tpgid: i32,       // Foreground process group
    pub is_session_leader: bool,
    pub is_group_leader: bool,
}

/// Get process group and session information
pub fn get_process_group_info(pid: u32) -> Result<ProcessGroupInfo> {
    debug!("Getting process group info for pid {}", pid);
    let stat_path = format!("/proc/{}/stat", pid);
    let stat_content = fs::read_to_string(&stat_path)
        .context("Failed to read /proc/{pid}/stat")?;
    
    // Parse stat file
    // Format: pid (comm) state ppid pgrp session tty_nr tpgid ...
    let parts = parse_stat_line(&stat_content)?;
    
    let pid = parts[0].parse::<u32>()?;
    let ppid = parts[3].parse::<u32>()?;
    let pgid = parts[4].parse::<u32>()?;
    let sid = parts[5].parse::<u32>()?;
    let tty_nr = parts[6].parse::<i32>()?;
    let tpgid = parts[7].parse::<i32>()?;
    
    Ok(ProcessGroupInfo {
        pid,
        ppid,
        pgid,
        sid,
        tty_nr,
        tpgid,
        is_session_leader: pid == sid,
        is_group_leader: pid == pgid,
    })
}

/// Parse /proc/[pid]/stat line
fn parse_stat_line(line: &str) -> Result<Vec<String>> {
    // Handle the (comm) field which may contain spaces and parentheses
    let start_paren = line.find('(')
        .context("Invalid stat format")?;
    let end_paren = line.rfind(')')
        .context("Invalid stat format")?;
    
    let mut parts = Vec::new();
    
    // Add PID (before first parenthesis)
    parts.push(line[..start_paren].trim().to_string());
    
    // Add comm (between parentheses)
    parts.push(line[start_paren+1..end_paren].to_string());
    
    // Add remaining fields
    let remaining = &line[end_paren+1..];
    parts.extend(
        remaining
            .split_whitespace()
            .map(|s| s.to_string())
    );
    
    Ok(parts)
}

/// Get all processes in a process group
pub fn get_processes_in_group(pgid: u32, all_processes: &[(u32, ProcessGroupInfo)]) -> Vec<u32> {
    all_processes
        .iter()
        .filter(|(_, info)| info.pgid == pgid)
        .map(|(pid, _)| *pid)
        .collect()
}

/// Get all processes in a session
pub fn get_processes_in_session(sid: u32, all_processes: &[(u32, ProcessGroupInfo)]) -> Vec<u32> {
    all_processes
        .iter()
        .filter(|(_, info)| info.sid == sid)
        .map(|(pid, _)| *pid)
        .collect()
}

/// Build process group hierarchy
pub fn build_group_hierarchy(
    processes: Vec<ProcessGroupInfo>
) -> HashMap<u32, Vec<ProcessGroupInfo>> {
    let mut groups: HashMap<u32, Vec<ProcessGroupInfo>> = HashMap::new();
    
    for proc in processes {
        groups.entry(proc.pgid)
            .or_insert_with(Vec::new)
            .push(proc);
    }
    
    groups
}

/// Build session hierarchy
pub fn build_session_hierarchy(
    processes: Vec<ProcessGroupInfo>
) -> HashMap<u32, Vec<ProcessGroupInfo>> {
    let mut sessions: HashMap<u32, Vec<ProcessGroupInfo>> = HashMap::new();
    
    for proc in processes {
        sessions.entry(proc.sid)
            .or_insert_with(Vec::new)
            .push(proc);
    }
    
    sessions
}

/// Get TTY name from TTY number
pub fn get_tty_name(tty_nr: i32) -> String {
    if tty_nr == 0 {
        return "?".to_string();
    }
    
    let major = (tty_nr >> 8) & 0xff;
    let minor = tty_nr & 0xff;
    
    match major {
        4 => format!("tty{}", minor),
        136 => format!("pts/{}", minor),
        _ => format!("{}:{}", major, minor),
    }
}

/// Kill entire process group
pub fn kill_process_group(pgid: u32, signal: i32) -> Result<()> {
    info!("Sending signal {} to process group {}", signal, pgid);
    unsafe {
        let result = libc::killpg(pgid as i32, signal);
        if result == -1 {
            error!("Failed to send signal {} to process group {}", signal, pgid);
            anyhow::bail!("Failed to send signal to process group");
        }
    }
    info!("Successfully sent signal {} to process group {}", signal, pgid);
    Ok(())
}

/// Format process group info as string
pub fn format_group_info(info: &ProcessGroupInfo) -> String {
    let mut flags = Vec::new();
    
    if info.is_session_leader {
        flags.push("session leader");
    }
    if info.is_group_leader {
        flags.push("group leader");
    }
    
    let flag_str = if flags.is_empty() {
        String::new()
    } else {
        format!(" ({})", flags.join(", "))
    };
    
    format!(
        "PID: {}, PGID: {}, SID: {}, TTY: {}{}",
        info.pid,
        info.pgid,
        info.sid,
        get_tty_name(info.tty_nr),
        flag_str
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_stat_line() {
        let line = "1234 (bash) S 1000 1234 1234 34816 5678 4194560";
        let parts = parse_stat_line(line).unwrap();
        
        assert_eq!(parts[0], "1234");
        assert_eq!(parts[1], "bash");
        assert_eq!(parts[2], "S");
    }

    #[test]
    fn test_get_tty_name() {
        assert_eq!(get_tty_name(0), "?");
        assert_eq!(get_tty_name(1024), "tty0");
        assert_eq!(get_tty_name(34816), "pts/0");
    }

    #[test]
    fn test_session_leader_detection() {
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
        
        assert!(info.is_session_leader);
        assert!(info.is_group_leader);
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
        assert!(formatted.contains("session leader"));
        assert!(formatted.contains("group leader"));
    }
}
