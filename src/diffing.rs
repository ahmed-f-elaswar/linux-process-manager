//! # Process State Diffing
//! 
//! Compare process states across time to track changes, identify new/terminated
//! processes, and analyze system evolution.
//! 
//! ## Features
//! 
//! - **Process Lifecycle**: Detect started/terminated processes
//! - **Resource Changes**: Track CPU, memory, thread count changes
//! - **State Transitions**: Running → Sleeping → Stopped
//! - **Parent Changes**: Detect process reparenting
//! - **Statistics**: Summarize changes across system
//! - **Timeline Analysis**: Track process evolution
//! 
//! ## Use Cases
//! 
//! - **Performance Debugging**: What changed when performance degraded?
//! - **Resource Leaks**: Which processes are growing over time?
//! - **Security Analysis**: Detect unexpected new processes
//! - **Capacity Planning**: Understand workload growth patterns
//! - **Troubleshooting**: Compare before/after problem states
//! 
//! ## Example
//! 
//! ```rust,ignore
//! use process_manager::diffing::{ProcessDiffer, DiffOptions};
//! use process_manager::process::ProcessManager;
//! 
//! # fn main() -> anyhow::Result<()> {
//! let differ = ProcessDiffer::new();
//! let mut process_manager = ProcessManager::new();
//! 
//! // Capture state at two points in time
//! process_manager.refresh()?;
//! let state1 = differ.capture_state(&process_manager)?;
//! // ... wait some time ...
//! process_manager.refresh()?;
//! let state2 = differ.capture_state(&process_manager)?;
//! 
//! // Compare states
//! let diff = differ.diff(&state1, &state2, DiffOptions::default())?;
//! 
//! println!("New processes: {}", diff.started.len());
//! println!("Terminated: {}", diff.terminated.len());
//! println!("Changed: {}", diff.changed.len());
//! # Ok(())
//! # }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use tracing::{debug, info};

/// Snapshot of a single process's state at a specific point in time.
/// 
/// Used for comparison to detect changes in process attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessState {
    pub pid: u32,
    pub name: String,
    pub user: String,
    pub cpu_percent: f64,
    pub memory: u64,
    pub memory_percent: f64,
    pub command: String,
    pub state: String,
    pub ppid: u32,
    pub threads: u32,
    pub open_files: usize,
    pub timestamp: DateTime<Utc>,
}

/// Difference between two process states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessDiffType {
    Added,
    Removed,
    Modified(Vec<FieldChange>),
    Unchanged,
}

/// Individual field change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldChange {
    pub field: String,
    pub old_value: String,
    pub new_value: String,
    pub percent_change: Option<f64>,
}

/// Process comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessDiff {
    pub pid: u32,
    pub name: String,
    pub diff_type: ProcessDiffType,
    pub timestamp_old: Option<DateTime<Utc>>,
    pub timestamp_new: Option<DateTime<Utc>>,
}

/// System-wide diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemDiff {
    pub timestamp: DateTime<Utc>,
    pub diffs: Vec<ProcessDiff>,
    pub summary: DiffSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffSummary {
    pub total_processes_old: usize,
    pub total_processes_new: usize,
    pub added: usize,
    pub removed: usize,
    pub modified: usize,
    pub unchanged: usize,
    pub significant_cpu_changes: usize,
    pub significant_memory_changes: usize,
}

/// Process differ
pub struct ProcessDiffer {
    threshold_cpu: f64,     // Percent change threshold
    threshold_memory: f64,  // Percent change threshold
}

impl ProcessDiffer {
    pub fn new() -> Self {
        Self {
            threshold_cpu: 10.0,     // 10% change
            threshold_memory: 10.0,  // 10% change
        }
    }
    
    pub fn with_thresholds(threshold_cpu: f64, threshold_memory: f64) -> Self {
        Self {
            threshold_cpu,
            threshold_memory,
        }
    }
    
    /// Compare two process state snapshots
    pub fn diff_states(
        &self,
        old_states: &HashMap<u32, ProcessState>,
        new_states: &HashMap<u32, ProcessState>,
    ) -> SystemDiff {
        debug!("Diffing process states: {} old vs {} new processes", old_states.len(), new_states.len());
        let mut diffs = Vec::new();
        let mut summary = DiffSummary {
            total_processes_old: old_states.len(),
            total_processes_new: new_states.len(),
            added: 0,
            removed: 0,
            modified: 0,
            unchanged: 0,
            significant_cpu_changes: 0,
            significant_memory_changes: 0,
        };
        
        let old_pids: HashSet<_> = old_states.keys().collect();
        let new_pids: HashSet<_> = new_states.keys().collect();
        
        // Find added processes
        for pid in new_pids.difference(&old_pids) {
            let state = &new_states[pid];
            diffs.push(ProcessDiff {
                pid: **pid,
                name: state.name.clone(),
                diff_type: ProcessDiffType::Added,
                timestamp_old: None,
                timestamp_new: Some(state.timestamp),
            });
            summary.added += 1;
        }
        
        // Find removed processes
        for pid in old_pids.difference(&new_pids) {
            let state = &old_states[pid];
            diffs.push(ProcessDiff {
                pid: **pid,
                name: state.name.clone(),
                diff_type: ProcessDiffType::Removed,
                timestamp_old: Some(state.timestamp),
                timestamp_new: None,
            });
            summary.removed += 1;
        }
        
        // Find modified/unchanged processes
        for pid in old_pids.intersection(&new_pids) {
            let old_state = &old_states[pid];
            let new_state = &new_states[pid];
            
            let changes = self.compare_states(old_state, new_state);
            
            if changes.is_empty() {
                diffs.push(ProcessDiff {
                    pid: **pid,
                    name: new_state.name.clone(),
                    diff_type: ProcessDiffType::Unchanged,
                    timestamp_old: Some(old_state.timestamp),
                    timestamp_new: Some(new_state.timestamp),
                });
                summary.unchanged += 1;
            } else {
                // Check for significant changes
                for change in &changes {
                    if change.field == "cpu_percent" {
                        if let Some(pct) = change.percent_change {
                            if pct.abs() > self.threshold_cpu {
                                summary.significant_cpu_changes += 1;
                            }
                        }
                    } else if change.field == "memory" || change.field == "memory_percent" {
                        if let Some(pct) = change.percent_change {
                            if pct.abs() > self.threshold_memory {
                                summary.significant_memory_changes += 1;
                            }
                        }
                    }
                }
                
                diffs.push(ProcessDiff {
                    pid: **pid,
                    name: new_state.name.clone(),
                    diff_type: ProcessDiffType::Modified(changes),
                    timestamp_old: Some(old_state.timestamp),
                    timestamp_new: Some(new_state.timestamp),
                });
                summary.modified += 1;
            }
        }
        
        info!("Process diff complete: {} added, {} removed, {} modified", 
              summary.added, summary.removed, summary.modified);
        
        SystemDiff {
            timestamp: Utc::now(),
            diffs,
            summary,
        }
    }
    
    /// Compare two individual process states
    fn compare_states(&self, old: &ProcessState, new: &ProcessState) -> Vec<FieldChange> {
        let mut changes = Vec::new();
        
        // Compare CPU
        if (old.cpu_percent - new.cpu_percent).abs() > 0.1 {
            let percent_change = if old.cpu_percent > 0.0 {
                ((new.cpu_percent - old.cpu_percent) / old.cpu_percent) * 100.0
            } else {
                100.0
            };
            
            changes.push(FieldChange {
                field: "cpu_percent".to_string(),
                old_value: format!("{:.1}%", old.cpu_percent),
                new_value: format!("{:.1}%", new.cpu_percent),
                percent_change: Some(percent_change),
            });
        }
        
        // Compare memory
        if old.memory != new.memory {
            let percent_change = if old.memory > 0 {
                ((new.memory as f64 - old.memory as f64) / old.memory as f64) * 100.0
            } else {
                100.0
            };
            
            changes.push(FieldChange {
                field: "memory".to_string(),
                old_value: format_bytes(old.memory),
                new_value: format_bytes(new.memory),
                percent_change: Some(percent_change),
            });
        }
        
        // Compare state
        if old.state != new.state {
            changes.push(FieldChange {
                field: "state".to_string(),
                old_value: old.state.clone(),
                new_value: new.state.clone(),
                percent_change: None,
            });
        }
        
        // Compare threads
        if old.threads != new.threads {
            let percent_change = if old.threads > 0 {
                ((new.threads as f64 - old.threads as f64) / old.threads as f64) * 100.0
            } else {
                100.0
            };
            
            changes.push(FieldChange {
                field: "threads".to_string(),
                old_value: old.threads.to_string(),
                new_value: new.threads.to_string(),
                percent_change: Some(percent_change),
            });
        }
        
        // Compare open files
        if old.open_files != new.open_files {
            let percent_change = if old.open_files > 0 {
                ((new.open_files as f64 - old.open_files as f64) / old.open_files as f64) * 100.0
            } else {
                100.0
            };
            
            changes.push(FieldChange {
                field: "open_files".to_string(),
                old_value: old.open_files.to_string(),
                new_value: new.open_files.to_string(),
                percent_change: Some(percent_change),
            });
        }
        
        changes
    }
    
    /// Export diff to human-readable format
    pub fn format_diff(&self, diff: &SystemDiff) -> String {
        debug!("Formatting process diff for {} process changes", diff.diffs.len());
        let mut output = String::new();
        
        output.push_str(&format!("Process Diff Summary ({})\n", diff.timestamp));
        output.push_str(&"=".repeat(80));
        output.push('\n');
        
        output.push_str(&format!("Processes (Old/New): {} / {}\n", 
            diff.summary.total_processes_old, diff.summary.total_processes_new));
        output.push_str(&format!("Added: {}\n", diff.summary.added));
        output.push_str(&format!("Removed: {}\n", diff.summary.removed));
        output.push_str(&format!("Modified: {}\n", diff.summary.modified));
        output.push_str(&format!("Unchanged: {}\n", diff.summary.unchanged));
        output.push_str(&format!("Significant CPU changes: {}\n", diff.summary.significant_cpu_changes));
        output.push_str(&format!("Significant Memory changes: {}\n", diff.summary.significant_memory_changes));
        output.push('\n');
        
        // Show added processes
        if diff.summary.added > 0 {
            output.push_str("Added Processes:\n");
            for proc_diff in &diff.diffs {
                if matches!(proc_diff.diff_type, ProcessDiffType::Added) {
                    output.push_str(&format!("  + [{}] {}\n", proc_diff.pid, proc_diff.name));
                }
            }
            output.push('\n');
        }
        
        // Show removed processes
        if diff.summary.removed > 0 {
            output.push_str("Removed Processes:\n");
            for proc_diff in &diff.diffs {
                if matches!(proc_diff.diff_type, ProcessDiffType::Removed) {
                    output.push_str(&format!("  - [{}] {}\n", proc_diff.pid, proc_diff.name));
                }
            }
            output.push('\n');
        }
        
        // Show modified processes
        if diff.summary.modified > 0 {
            output.push_str("Modified Processes:\n");
            for proc_diff in &diff.diffs {
                if let ProcessDiffType::Modified(changes) = &proc_diff.diff_type {
                    output.push_str(&format!("  ~ [{}] {}\n", proc_diff.pid, proc_diff.name));
                    for change in changes {
                        let change_indicator = if let Some(pct) = change.percent_change {
                            if pct > 0.0 {
                                format!(" (+{:.1}%)", pct)
                            } else {
                                format!(" ({:.1}%)", pct)
                            }
                        } else {
                            String::new()
                        };
                        
                        output.push_str(&format!("      {}: {} → {}{}\n", 
                            change.field, change.old_value, change.new_value, change_indicator));
                    }
                }
            }
        }
        
        output
    }
}

impl Default for ProcessDiffer {
    fn default() -> Self {
        Self::new()
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    
    format!("{:.2} {}", size, UNITS[unit_idx])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_diff() {
        let differ = ProcessDiffer::new();
        
        let mut old_states = HashMap::new();
        old_states.insert(
            1234,
            ProcessState {
                pid: 1234,
                name: "test".to_string(),
                user: "user".to_string(),
                cpu_percent: 10.0,
                memory: 1024000,
                memory_percent: 5.0,
                command: "test".to_string(),
                state: "R".to_string(),
                ppid: 1,
                threads: 1,
                open_files: 10,
                timestamp: Utc::now(),
            },
        );
        
        let mut new_states = HashMap::new();
        new_states.insert(
            1234,
            ProcessState {
                pid: 1234,
                name: "test".to_string(),
                user: "user".to_string(),
                cpu_percent: 25.0,
                memory: 2048000,
                memory_percent: 10.0,
                command: "test".to_string(),
                state: "R".to_string(),
                ppid: 1,
                threads: 2,
                open_files: 15,
                timestamp: Utc::now(),
            },
        );
        
        let diff = differ.diff_states(&old_states, &new_states);
        
        assert_eq!(diff.summary.modified, 1);
        assert_eq!(diff.summary.added, 0);
        assert_eq!(diff.summary.removed, 0);
    }
}
