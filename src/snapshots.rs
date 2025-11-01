//! # Process Snapshots and Replay
//! 
//! Capture complete system process state at specific points in time for
//! comparison, replay, and forensic analysis.
//! 
//! ## Features
//! 
//! - **Full State Capture**: All processes with complete metrics
//! - **Multiple Formats**: JSON, CSV, HTML export
//! - **Snapshot Comparison**: Diff between two snapshots
//! - **Time-Series Analysis**: Track system evolution
//! - **Metadata**: Hostname, timestamp, system stats
//! 
//! ## Use Cases
//! 
//! - Debug intermittent issues (capture before/after)
//! - Performance analysis (compare different workloads)
//! - Capacity planning (track growth over time)
//! - Incident response (forensic analysis)
//! - Compliance/audit (state documentation)
//! 
//! ## Example
//! 
//! ```rust,ignore
//! use process_manager::snapshots::{SnapshotManager, ExportFormat};
//! use process_manager::process::ProcessManager;
//! 
//! # fn main() -> anyhow::Result<()> {
//! let mut manager = SnapshotManager::new("./snapshots");
//! let mut process_manager = ProcessManager::new();
//! process_manager.refresh()?;
//! 
//! // Capture current state
//! let snapshot = manager.capture(&process_manager, "baseline")?;
//! 
//! // Export to HTML
//! manager.export_snapshot(&snapshot, ExportFormat::Html, "snapshot.html")?;
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info, error};

/// Process information captured in a snapshot.
/// 
/// Contains all relevant process metrics and metadata at capture time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
    pub command: String,
    pub user: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub memory_percent: f32,
    pub status: String,
    pub threads: u32,
}

/// Process snapshot captured at a specific time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub hostname: String,
    pub processes: Vec<ProcessInfo>,
    pub system_stats: SystemStats,
    pub metadata: SnapshotMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub cpu_count: usize,
    pub total_memory: u64,
    pub used_memory: u64,
    pub load_average: (f64, f64, f64),
    pub uptime: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
}

/// Snapshot manager for capturing and replaying process states
pub struct SnapshotManager {
    snapshot_dir: PathBuf,
}

impl SnapshotManager {
    pub fn new(snapshot_dir: Option<PathBuf>) -> Result<Self> {
        let dir = snapshot_dir.unwrap_or_else(|| {
            dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("process-manager")
                .join("snapshots")
        });
        
        fs::create_dir_all(&dir)
            .context("Failed to create snapshot directory")?;
        
        Ok(Self {
            snapshot_dir: dir,
        })
    }
    
    /// Capture current process state
    pub fn capture_snapshot(
        &self,
        processes: Vec<ProcessInfo>,
        system_stats: SystemStats,
        name: String,
        description: String,
        tags: Vec<String>,
    ) -> Result<ProcessSnapshot> {
        let hostname = hostname::get()?
            .to_string_lossy()
            .to_string();
        
        let snapshot = ProcessSnapshot {
            timestamp: chrono::Utc::now(),
            hostname,
            processes,
            system_stats,
            metadata: SnapshotMetadata {
                name: name.clone(),
                description,
                tags,
            },
        };
        
        // Save snapshot to file
        let filename = format!(
            "snapshot_{}_{}.json",
            name.replace(' ', "_"),
            snapshot.timestamp.format("%Y%m%d_%H%M%S")
        );
        
        let path = self.snapshot_dir.join(&filename);
        info!("Capturing snapshot '{}' with {} processes to {:?}", 
              name, snapshot.processes.len(), path);
        
        let json = serde_json::to_string_pretty(&snapshot)?;
        fs::write(&path, json)?;
        
        info!("Snapshot '{}' saved successfully ({} processes)", name, snapshot.processes.len());
        Ok(snapshot)
    }
    
    /// Load snapshot from file
    pub fn load_snapshot(&self, filename: &str) -> Result<ProcessSnapshot> {
        debug!("Loading snapshot from file: {}", filename);
        let path = self.snapshot_dir.join(filename);
        let content = fs::read_to_string(&path)
            .context("Failed to read snapshot file")?;
        let snapshot: ProcessSnapshot = serde_json::from_str(&content)
            .context("Failed to parse snapshot JSON")?;
        info!("Loaded snapshot '{}' with {} processes", 
              snapshot.metadata.name, snapshot.processes.len());
        Ok(snapshot)
    }
    
    /// List all available snapshots
    pub fn list_snapshots(&self) -> Result<Vec<String>> {
        let mut snapshots = Vec::new();
        
        for entry in fs::read_dir(&self.snapshot_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Some(filename) = path.file_name() {
                    snapshots.push(filename.to_string_lossy().to_string());
                }
            }
        }
        
        snapshots.sort();
        Ok(snapshots)
    }
    
    /// Delete a snapshot
    pub fn delete_snapshot(&self, filename: &str) -> Result<()> {
        info!("Deleting snapshot: {}", filename);
        let path = self.snapshot_dir.join(filename);
        fs::remove_file(&path)
            .context("Failed to delete snapshot")?;
        Ok(())
    }
    
    /// Compare two snapshots
    pub fn compare_snapshots(
        &self,
        snapshot1: &ProcessSnapshot,
        snapshot2: &ProcessSnapshot,
    ) -> SnapshotDiff {
        info!("Comparing snapshots: '{}' ({} processes) vs '{}' ({} processes)",
              snapshot1.metadata.name, snapshot1.processes.len(),
              snapshot2.metadata.name, snapshot2.processes.len());
        
        let mut processes1: HashMap<u32, &ProcessInfo> = HashMap::new();
        let mut processes2: HashMap<u32, &ProcessInfo> = HashMap::new();
        
        for proc in &snapshot1.processes {
            processes1.insert(proc.pid, proc);
        }
        
        for proc in &snapshot2.processes {
            processes2.insert(proc.pid, proc);
        }
        
        let mut new_processes = Vec::new();
        let mut terminated_processes = Vec::new();
        let mut changed_processes = Vec::new();
        
        // Find new processes
        for (&pid, &proc) in &processes2 {
            if !processes1.contains_key(&pid) {
                new_processes.push(proc.clone());
            }
        }
        
        // Find terminated processes
        for (&pid, &proc) in &processes1 {
            if !processes2.contains_key(&pid) {
                terminated_processes.push(proc.clone());
            }
        }
        
        // Find changed processes
        for (&pid, &proc2) in &processes2 {
            if let Some(&proc1) = processes1.get(&pid) {
                if has_significant_change(proc1, proc2) {
                    changed_processes.push(ProcessChange {
                        pid,
                        name: proc2.name.clone(),
                        old_cpu: proc1.cpu_usage,
                        new_cpu: proc2.cpu_usage,
                        old_memory: proc1.memory_usage,
                        new_memory: proc2.memory_usage,
                    });
                }
            }
        }
        
        let diff = SnapshotDiff {
            snapshot1_time: snapshot1.timestamp,
            snapshot2_time: snapshot2.timestamp,
            new_processes: new_processes.clone(),
            terminated_processes: terminated_processes.clone(),
            changed_processes: changed_processes.clone(),
            total_processes_before: snapshot1.processes.len(),
            total_processes_after: snapshot2.processes.len(),
        };
        
        info!("Snapshot comparison complete: {} new, {} terminated, {} changed processes",
              new_processes.len(), terminated_processes.len(), changed_processes.len());
        
        diff
    }
}

/// Difference between two snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotDiff {
    pub snapshot1_time: chrono::DateTime<chrono::Utc>,
    pub snapshot2_time: chrono::DateTime<chrono::Utc>,
    pub new_processes: Vec<ProcessInfo>,
    pub terminated_processes: Vec<ProcessInfo>,
    pub changed_processes: Vec<ProcessChange>,
    pub total_processes_before: usize,
    pub total_processes_after: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessChange {
    pub pid: u32,
    pub name: String,
    pub old_cpu: f32,
    pub new_cpu: f32,
    pub old_memory: u64,
    pub new_memory: u64,
}

/// Check if process has significant change
fn has_significant_change(proc1: &ProcessInfo, proc2: &ProcessInfo) -> bool {
    let cpu_change = (proc2.cpu_usage - proc1.cpu_usage).abs();
    let memory_change = (proc2.memory_usage as i64 - proc1.memory_usage as i64).abs();
    
    // Significant if CPU changed by >10% or memory changed by >10MB
    cpu_change > 10.0 || memory_change > 10240
}

/// Export snapshot to different formats
pub fn export_snapshot(snapshot: &ProcessSnapshot, format: ExportFormat) -> Result<String> {
    debug!("Exporting snapshot '{}' to {:?} format", snapshot.metadata.name, format);
    
    let result = match format {
        ExportFormat::Json => {
            serde_json::to_string_pretty(snapshot)
                .context("Failed to export to JSON")
        }
        ExportFormat::Csv => {
            export_to_csv(snapshot)
        }
        ExportFormat::Html => {
            export_to_html(snapshot)
        }
    };
    
    match &result {
        Ok(data) => info!("Successfully exported snapshot '{}' to {:?} format ({} bytes)", 
                         snapshot.metadata.name, format, data.len()),
        Err(e) => error!("Failed to export snapshot '{}' to {:?} format: {}", 
                        snapshot.metadata.name, format, e),
    }
    
    result
}

#[derive(Debug)]
pub enum ExportFormat {
    Json,
    Csv,
    Html,
}

fn export_to_csv(snapshot: &ProcessSnapshot) -> Result<String> {
    let mut csv = String::from("PID,Name,User,CPU%,Memory(KB),Status,Command\n");
    
    for proc in &snapshot.processes {
        csv.push_str(&format!(
            "{},{},{},{:.2},{},{},{}\n",
            proc.pid,
            proc.name,
            proc.user,
            proc.cpu_usage,
            proc.memory_usage,
            proc.status,
            proc.command.replace(',', ";"),
        ));
    }
    
    Ok(csv)
}

fn export_to_html(snapshot: &ProcessSnapshot) -> Result<String> {
    let mut html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Process Snapshot - {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #4CAF50; color: white; }}
        tr:nth-child(even) {{ background-color: #f2f2f2; }}
        .header {{ margin-bottom: 20px; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Process Snapshot</h1>
        <p><strong>Time:</strong> {}</p>
        <p><strong>Host:</strong> {}</p>
        <p><strong>Total Processes:</strong> {}</p>
    </div>
    <table>
        <tr>
            <th>PID</th>
            <th>Name</th>
            <th>User</th>
            <th>CPU%</th>
            <th>Memory (KB)</th>
            <th>Status</th>
        </tr>
"#,
        snapshot.metadata.name,
        snapshot.timestamp,
        snapshot.hostname,
        snapshot.processes.len(),
    );
    
    for proc in &snapshot.processes {
        html.push_str(&format!(
            "        <tr>\
                <td>{}</td>\
                <td>{}</td>\
                <td>{}</td>\
                <td>{:.2}</td>\
                <td>{}</td>\
                <td>{}</td>\
            </tr>\n",
            proc.pid,
            proc.name,
            proc.user,
            proc.cpu_usage,
            proc.memory_usage,
            proc.status,
        ));
    }
    
    html.push_str("    </table>\n</body>\n</html>");
    
    Ok(html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_metadata() {
        let metadata = SnapshotMetadata {
            name: "test".to_string(),
            description: "Test snapshot".to_string(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };
        
        assert_eq!(metadata.name, "test");
        assert_eq!(metadata.tags.len(), 2);
    }

    #[test]
    fn test_has_significant_change() {
        let proc1 = ProcessInfo {
            pid: 1,
            ppid: 0,
            name: "test".to_string(),
            command: "test".to_string(),
            user: "root".to_string(),
            cpu_usage: 10.0,
            memory_usage: 1024,
            memory_percent: 1.0,
            status: "R".to_string(),
            threads: 1,
        };
        
        let mut proc2 = proc1.clone();
        proc2.cpu_usage = 25.0; // 15% increase
        
        assert!(has_significant_change(&proc1, &proc2));
    }
}
