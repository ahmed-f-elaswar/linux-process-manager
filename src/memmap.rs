//! # Memory Map Visualization
//! 
//! Parse and visualize process memory layouts from /proc/[pid]/maps for
//! debugging memory issues, analyzing library usage, and understanding address space.
//! 
//! ## Features
//! 
//! - **Memory Region Parsing**: Read /proc/[pid]/maps
//! - **Permission Analysis**: rwxp flags per region
//! - **Size Calculation**: Total size by category (heap, stack, libraries)
//! - **Library Detection**: Identify all loaded shared libraries
//! - **Visualization**: ASCII art memory layout
//! - **Statistics**: Aggregated memory usage by type
//! 
//! ## Memory Region Types
//! 
//! - **Heap**: Dynamic allocations
//! - **Stack**: Function call stack
//! - **Anonymous**: Private anonymous mappings (mmap)
//! - **File-backed**: Libraries (.so files), executables, data files
//! - **Special**: [vdso], [vsyscall], [vvar]
//! 
//! ## Example
//! 
//! ```rust,ignore
//! use process_manager::memmap::{parse_memory_maps, MemoryStats, visualize_memory_map};
//! 
//! # fn main() -> anyhow::Result<()> {
//! // Parse memory map for process
//! let regions = parse_memory_maps(1234)?;
//! 
//! // Get statistics
//! let stats = MemoryStats::from_regions(&regions);
//! println!("Total heap: {} MB", stats.heap_size / 1024 / 1024);
//! 
//! // Visualize layout
//! let viz = visualize_memory_map(&regions, 40);
//! println!("{}", viz);
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use std::fs;
use std::collections::HashMap;
use tracing::{debug, info};

/// Represents a single memory-mapped region in a process's address space.
/// 
/// Parsed from /proc/[pid]/maps, contains address range, permissions, and backing file.
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub start_addr: u64,
    pub end_addr: u64,
    pub size: u64,
    pub perms: String,
    pub offset: u64,
    pub device: String,
    pub inode: u64,
    pub pathname: String,
    pub is_readable: bool,
    pub is_writable: bool,
    pub is_executable: bool,
    pub is_shared: bool,
    pub is_private: bool,
}

/// Memory map visualization
#[derive(Debug)]
pub struct MemoryMapVisualizer {
    pub pid: u32,
    pub regions: Vec<MemoryRegion>,
    pub total_size: u64,
    pub code_size: u64,
    pub data_size: u64,
    pub heap_size: u64,
    pub stack_size: u64,
    pub shared_lib_size: u64,
}

impl MemoryMapVisualizer {
    /// Create a new memory map visualizer
    pub fn new(pid: u32) -> Result<Self> {
        debug!("Creating memory map visualizer for pid {}", pid);
        let regions = parse_memory_maps(pid)?;
        info!("Parsed {} memory regions for pid {}", regions.len(), pid);
        
        let total_size = regions.iter().map(|r| r.size).sum();
        let code_size = regions.iter()
            .filter(|r| r.is_executable && !r.is_writable)
            .map(|r| r.size)
            .sum();
        let data_size = regions.iter()
            .filter(|r| !r.is_executable && r.is_writable)
            .map(|r| r.size)
            .sum();
        let heap_size = regions.iter()
            .filter(|r| r.pathname == "[heap]")
            .map(|r| r.size)
            .sum();
        let stack_size = regions.iter()
            .filter(|r| r.pathname.starts_with("[stack"))
            .map(|r| r.size)
            .sum();
        let shared_lib_size = regions.iter()
            .filter(|r| r.pathname.ends_with(".so") || r.pathname.contains(".so."))
            .map(|r| r.size)
            .sum();
        
        Ok(Self {
            pid,
            regions,
            total_size,
            code_size,
            data_size,
            heap_size,
            stack_size,
            shared_lib_size,
        })
    }
    
    /// Get regions by type
    pub fn get_regions_by_type(&self, region_type: &str) -> Vec<&MemoryRegion> {
        match region_type {
            "code" => self.regions.iter()
                .filter(|r| r.is_executable && !r.is_writable)
                .collect(),
            "data" => self.regions.iter()
                .filter(|r| !r.is_executable && r.is_writable)
                .collect(),
            "heap" => self.regions.iter()
                .filter(|r| r.pathname == "[heap]")
                .collect(),
            "stack" => self.regions.iter()
                .filter(|r| r.pathname.starts_with("[stack"))
                .collect(),
            "library" => self.regions.iter()
                .filter(|r| r.pathname.ends_with(".so") || r.pathname.contains(".so."))
                .collect(),
            _ => Vec::new(),
        }
    }
    
    /// Get library usage summary
    pub fn get_library_summary(&self) -> HashMap<String, u64> {
        let mut libs: HashMap<String, u64> = HashMap::new();
        
        for region in &self.regions {
            if region.pathname.ends_with(".so") || region.pathname.contains(".so.") {
                *libs.entry(region.pathname.clone()).or_insert(0) += region.size;
            }
        }
        
        libs
    }
    
    /// Generate ASCII visualization of memory layout
    pub fn visualize_ascii(&self, width: usize) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("Memory Map for PID {} (Total: {})\n", 
            self.pid, format_size(self.total_size)));
        output.push_str(&"=".repeat(width));
        output.push('\n');
        
        // Calculate bar lengths
        let bars = [
            ("Code", self.code_size),
            ("Data", self.data_size),
            ("Heap", self.heap_size),
            ("Stack", self.stack_size),
            ("Libraries", self.shared_lib_size),
        ];
        
        for (name, size) in bars {
            let percent = if self.total_size > 0 {
                (size as f64 / self.total_size as f64) * 100.0
            } else {
                0.0
            };
            
            let bar_len = ((percent / 100.0) * (width as f64 - 30.0)) as usize;
            let bar = "#".repeat(bar_len);
            
            output.push_str(&format!(
                "{:<12} [{:<40}] {:>6.2}% ({:>10})\n",
                name, bar, percent, format_size(size)
            ));
        }
        
        output
    }
    
    /// Export to CSV
    pub fn export_csv(&self) -> String {
        let mut csv = String::new();
        csv.push_str("Start,End,Size,Permissions,Offset,Device,Inode,Path\n");
        
        for region in &self.regions {
            csv.push_str(&format!(
                "0x{:x},0x{:x},{},{},0x{:x},{},{},{}\n",
                region.start_addr,
                region.end_addr,
                region.size,
                region.perms,
                region.offset,
                region.device,
                region.inode,
                region.pathname
            ));
        }
        
        csv
    }
    
    /// Export to HTML
    pub fn export_html(&self) -> String {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html><head><style>\n");
        html.push_str("table { border-collapse: collapse; width: 100%; }\n");
        html.push_str("th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
        html.push_str("th { background-color: #4CAF50; color: white; }\n");
        html.push_str("tr:nth-child(even) { background-color: #f2f2f2; }\n");
        html.push_str(".code { background-color: #e3f2fd; }\n");
        html.push_str(".data { background-color: #fff3e0; }\n");
        html.push_str(".heap { background-color: #f3e5f5; }\n");
        html.push_str(".stack { background-color: #e8f5e9; }\n");
        html.push_str("</style></head><body>\n");
        html.push_str(&format!("<h1>Memory Map for PID {}</h1>\n", self.pid));
        html.push_str(&format!("<p>Total Size: {}</p>\n", format_size(self.total_size)));
        html.push_str("<table>\n<tr><th>Start</th><th>End</th><th>Size</th><th>Perms</th><th>Path</th></tr>\n");
        
        for region in &self.regions {
            let class = if region.is_executable && !region.is_writable {
                "code"
            } else if region.pathname == "[heap]" {
                "heap"
            } else if region.pathname.starts_with("[stack") {
                "stack"
            } else if region.is_writable {
                "data"
            } else {
                ""
            };
            
            html.push_str(&format!(
                "<tr class=\"{}\"><td>0x{:x}</td><td>0x{:x}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                class,
                region.start_addr,
                region.end_addr,
                format_size(region.size),
                region.perms,
                region.pathname
            ));
        }
        
        html.push_str("</table>\n</body></html>");
        html
    }
}

/// Parse /proc/[pid]/maps file
fn parse_memory_maps(pid: u32) -> Result<Vec<MemoryRegion>> {
    let maps_path = format!("/proc/{}/maps", pid);
    let content = fs::read_to_string(&maps_path)
        .context("Failed to read /proc/{pid}/maps")?;
    
    let mut regions = Vec::new();
    
    for line in content.lines() {
        if let Some(region) = parse_map_line(line) {
            regions.push(region);
        }
    }
    
    Ok(regions)
}

/// Parse single line from /proc/[pid]/maps
fn parse_map_line(line: &str) -> Option<MemoryRegion> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 5 {
        return None;
    }
    
    // Parse address range
    let addr_parts: Vec<&str> = parts[0].split('-').collect();
    if addr_parts.len() != 2 {
        return None;
    }
    
    let start_addr = u64::from_str_radix(addr_parts[0], 16).ok()?;
    let end_addr = u64::from_str_radix(addr_parts[1], 16).ok()?;
    let size = end_addr - start_addr;
    
    // Parse permissions
    let perms = parts[1].to_string();
    let is_readable = perms.chars().nth(0) == Some('r');
    let is_writable = perms.chars().nth(1) == Some('w');
    let is_executable = perms.chars().nth(2) == Some('x');
    let is_shared = perms.chars().nth(3) == Some('s');
    let is_private = perms.chars().nth(3) == Some('p');
    
    // Parse offset, device, inode
    let offset = u64::from_str_radix(parts[2], 16).ok()?;
    let device = parts[3].to_string();
    let inode = parts[4].parse().ok()?;
    
    // Parse pathname (optional)
    let pathname = if parts.len() > 5 {
        parts[5..].join(" ")
    } else {
        String::new()
    };
    
    Some(MemoryRegion {
        start_addr,
        end_addr,
        size,
        perms,
        offset,
        device,
        inode,
        pathname,
        is_readable,
        is_writable,
        is_executable,
        is_shared,
        is_private,
    })
}

/// Format size in human-readable format
fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
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
    fn test_parse_map_line() {
        let line = "7f8a2c000000-7f8a2c021000 rw-p 00000000 00:00 0";
        let region = parse_map_line(line).unwrap();
        
        assert_eq!(region.start_addr, 0x7f8a2c000000);
        assert_eq!(region.end_addr, 0x7f8a2c021000);
        assert!(region.is_readable);
        assert!(region.is_writable);
        assert!(!region.is_executable);
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1048576), "1.00 MB");
        assert_eq!(format_size(1073741824), "1.00 GB");
    }
}
