use std::fs;
use std::path::Path;
use std::process::Command;
use anyhow::{Context, Result};
use tracing::{debug, info, warn};

/// GPU information for a process
#[derive(Debug, Clone, Default)]
pub struct GpuStats {
    pub pid: u32,
    pub gpu_memory_used: u64,    // GPU memory in MB
    pub gpu_utilization: f32,     // GPU utilization percentage
    pub gpu_device: String,       // GPU device name
    pub gpu_index: usize,         // GPU index (for multi-GPU systems)
}

/// System GPU information
#[derive(Debug, Clone)]
pub struct SystemGpuInfo {
    pub gpu_count: usize,
    pub gpus: Vec<GpuDevice>,
}

#[derive(Debug, Clone)]
pub struct GpuDevice {
    pub index: usize,
    pub name: String,
    pub memory_total: u64,       // Total GPU memory in MB
    pub memory_used: u64,        // Used GPU memory in MB
    pub temperature: Option<f32>, // GPU temperature in Celsius
    pub utilization: f32,        // Overall GPU utilization
    pub driver_version: String,
}

/// Detect GPU vendor and get stats
pub fn get_system_gpu_info() -> Result<SystemGpuInfo> {
    debug!("Detecting GPU information");
    // Try NVIDIA first
    if let Ok(nvidia_info) = get_nvidia_gpu_info() {
        info!("Detected {} NVIDIA GPU(s)", nvidia_info.gpu_count);
        return Ok(nvidia_info);
    }

    // Try AMD
    if let Ok(amd_info) = get_amd_gpu_info() {
        info!("Detected {} AMD GPU(s)", amd_info.gpu_count);
        return Ok(amd_info);
    }

    // Try Intel
    if let Ok(intel_info) = get_intel_gpu_info() {
        info!("Detected {} Intel GPU(s)", intel_info.gpu_count);
        return Ok(intel_info);
    }

    // No GPU found
    warn!("No GPU detected on system");
    Ok(SystemGpuInfo {
        gpu_count: 0,
        gpus: Vec::new(),
    })
}

/// Get NVIDIA GPU information using nvidia-smi
fn get_nvidia_gpu_info() -> Result<SystemGpuInfo> {
    let output = Command::new("nvidia-smi")
        .args(&[
            "--query-gpu=index,name,memory.total,memory.used,temperature.gpu,utilization.gpu,driver_version",
            "--format=csv,noheader,nounits"
        ])
        .output()
        .context("Failed to execute nvidia-smi")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("nvidia-smi failed"));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut gpus = Vec::new();

    for line in output_str.lines() {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() >= 7 {
            let gpu = GpuDevice {
                index: parts[0].parse().unwrap_or(0),
                name: parts[1].to_string(),
                memory_total: parts[2].parse().unwrap_or(0),
                memory_used: parts[3].parse().unwrap_or(0),
                temperature: parts[4].parse().ok(),
                utilization: parts[5].parse().unwrap_or(0.0),
                driver_version: parts[6].to_string(),
            };
            gpus.push(gpu);
        }
    }

    Ok(SystemGpuInfo {
        gpu_count: gpus.len(),
        gpus,
    })
}

/// Get per-process GPU stats for NVIDIA
pub fn get_nvidia_process_stats(pid: u32) -> Result<GpuStats> {
    let output = Command::new("nvidia-smi")
        .args(&[
            "--query-compute-apps=pid,used_memory,gpu_name,gpu_bus_id",
            "--format=csv,noheader,nounits"
        ])
        .output()
        .context("Failed to execute nvidia-smi")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("nvidia-smi failed"));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    
    for line in output_str.lines() {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() >= 3 {
            if let Ok(process_pid) = parts[0].parse::<u32>() {
                if process_pid == pid {
                    return Ok(GpuStats {
                        pid,
                        gpu_memory_used: parts[1].parse().unwrap_or(0),
                        gpu_utilization: 0.0, // Not available per-process
                        gpu_device: parts[2].to_string(),
                        gpu_index: 0,
                    });
                }
            }
        }
    }

    Err(anyhow::anyhow!("Process not using GPU"))
}

/// Get AMD GPU information
fn get_amd_gpu_info() -> Result<SystemGpuInfo> {
    // Check if rocm-smi is available
    let rocm_path = "/opt/rocm/bin/rocm-smi";
    if !Path::new(rocm_path).exists() {
        return Err(anyhow::anyhow!("rocm-smi not found"));
    }

    let output = Command::new(rocm_path)
        .args(&["--showmeminfo", "vram", "--csv"])
        .output()
        .context("Failed to execute rocm-smi")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("rocm-smi failed"));
    }

    // Parse AMD GPU info (simplified)
    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = output_str.lines().collect();
    
    if lines.len() > 1 {
        let mut gpus = Vec::new();
        for (index, _line) in lines.iter().enumerate().skip(1) {
            gpus.push(GpuDevice {
                index,
                name: "AMD GPU".to_string(),
                memory_total: 0,
                memory_used: 0,
                temperature: None,
                utilization: 0.0,
                driver_version: "Unknown".to_string(),
            });
        }
        
        return Ok(SystemGpuInfo {
            gpu_count: gpus.len(),
            gpus,
        });
    }

    Err(anyhow::anyhow!("No AMD GPUs found"))
}

/// Get Intel GPU information
fn get_intel_gpu_info() -> Result<SystemGpuInfo> {
    // Check for Intel GPU via sysfs
    let intel_gpu_path = "/sys/class/drm/card0";
    if !Path::new(intel_gpu_path).exists() {
        return Err(anyhow::anyhow!("Intel GPU not found"));
    }

    // Basic Intel GPU detection
    let vendor_path = format!("{}/device/vendor", intel_gpu_path);
    if Path::new(&vendor_path).exists() {
        if let Ok(vendor) = fs::read_to_string(&vendor_path) {
            if vendor.trim() == "0x8086" {
                // Intel vendor ID
                return Ok(SystemGpuInfo {
                    gpu_count: 1,
                    gpus: vec![GpuDevice {
                        index: 0,
                        name: "Intel GPU".to_string(),
                        memory_total: 0,
                        memory_used: 0,
                        temperature: None,
                        utilization: 0.0,
                        driver_version: "Unknown".to_string(),
                    }],
                });
            }
        }
    }

    Err(anyhow::anyhow!("Intel GPU not found"))
}

/// Check if GPU monitoring is available
pub fn is_gpu_available() -> bool {
    // Check for nvidia-smi
    if Command::new("nvidia-smi").arg("--version").output().is_ok() {
        return true;
    }

    // Check for rocm-smi
    if Path::new("/opt/rocm/bin/rocm-smi").exists() {
        return true;
    }

    // Check for Intel GPU
    if Path::new("/sys/class/drm/card0").exists() {
        return true;
    }

    false
}

/// Format GPU memory for display
pub fn format_gpu_memory(memory_mb: u64) -> String {
    if memory_mb >= 1024 {
        format!("{:.1} GB", memory_mb as f64 / 1024.0)
    } else {
        format!("{} MB", memory_mb)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_gpu_memory() {
        assert_eq!(format_gpu_memory(512), "512 MB");
        assert_eq!(format_gpu_memory(2048), "2.0 GB");
        assert_eq!(format_gpu_memory(8192), "8.0 GB");
    }

    #[test]
    fn test_gpu_detection() {
        // This test will vary by system
        let available = is_gpu_available();
        println!("GPU available: {}", available);
    }
}