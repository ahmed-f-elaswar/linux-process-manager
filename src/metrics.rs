// Metrics export module for Prometheus and InfluxDB integration
// Provides formatted output compatible with popular monitoring systems

use crate::process::ProcessManager;
use std::fmt::Write;
use chrono::Utc;
use tracing::{debug, info};

/// Export format type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    Prometheus,
    InfluxDB,
}

/// Prometheus metrics exporter
pub struct PrometheusExporter {
    namespace: String,
}

impl PrometheusExporter {
    pub fn new(namespace: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
        }
    }
    
    /// Export all process metrics in Prometheus format
    pub fn export_process_metrics(&self, process_manager: &ProcessManager) -> String {
        let mut output = String::new();
        let processes = process_manager.get_processes();
        
        // Process CPU usage
        writeln!(output, "# HELP {}_process_cpu_usage Process CPU usage percentage", self.namespace).ok();
        writeln!(output, "# TYPE {}_process_cpu_usage gauge", self.namespace).ok();
        for process in &processes {
            writeln!(
                output,
                "{}_process_cpu_usage{{pid=\"{}\",name=\"{}\",user=\"{}\"}} {}",
                self.namespace, process.pid, Self::escape_label(&process.name),
                Self::escape_label(&process.user), process.cpu_usage
            ).ok();
        }
        
        // Process memory usage
        writeln!(output, "# HELP {}_process_memory_bytes Process memory usage in bytes", self.namespace).ok();
        writeln!(output, "# TYPE {}_process_memory_bytes gauge", self.namespace).ok();
        for process in &processes {
            writeln!(
                output,
                "{}_process_memory_bytes{{pid=\"{}\",name=\"{}\",user=\"{}\"}} {}",
                self.namespace, process.pid, Self::escape_label(&process.name),
                Self::escape_label(&process.user), process.memory_usage * 1024 // Convert KB to bytes
            ).ok();
        }
        
        // Process count by user
        writeln!(output, "# HELP {}_process_count_by_user Number of processes per user", self.namespace).ok();
        writeln!(output, "# TYPE {}_process_count_by_user gauge", self.namespace).ok();
        let mut user_counts = std::collections::HashMap::new();
        for process in &processes {
            *user_counts.entry(&process.user).or_insert(0) += 1;
        }
        for (user, count) in user_counts {
            writeln!(
                output,
                "{}_process_count_by_user{{user=\"{}\"}} {}",
                self.namespace, Self::escape_label(user), count
            ).ok();
        }
        
        // Network connections (if available)
        writeln!(output, "# HELP {}_process_network_connections Number of network connections per process", self.namespace).ok();
        writeln!(output, "# TYPE {}_process_network_connections gauge", self.namespace).ok();
        for process in &processes {
            if let Some(connections) = process.network_connections {
                writeln!(
                    output,
                    "{}_process_network_connections{{pid=\"{}\",name=\"{}\"}} {}",
                    self.namespace, process.pid, Self::escape_label(&process.name), connections
                ).ok();
            }
        }
        
        // Container processes
        writeln!(output, "# HELP {}_process_in_container Whether process is running in a container (1=yes, 0=no)", self.namespace).ok();
        writeln!(output, "# TYPE {}_process_in_container gauge", self.namespace).ok();
        for process in &processes {
            if process.is_container {
                let container_id = process.container_id.as_deref().unwrap_or("unknown");
                writeln!(
                    output,
                    "{}_process_in_container{{pid=\"{}\",name=\"{}\",container_id=\"{}\"}} 1",
                    self.namespace, process.pid, Self::escape_label(&process.name),
                    Self::escape_label(container_id)
                ).ok();
            }
        }
        
        // GPU memory (if available)
        writeln!(output, "# HELP {}_process_gpu_memory_bytes GPU memory usage per process in bytes", self.namespace).ok();
        writeln!(output, "# TYPE {}_process_gpu_memory_bytes gauge", self.namespace).ok();
        for process in &processes {
            if let Some(gpu_memory) = process.gpu_memory {
                writeln!(
                    output,
                    "{}_process_gpu_memory_bytes{{pid=\"{}\",name=\"{}\"}} {}",
                    self.namespace, process.pid, Self::escape_label(&process.name),
                    gpu_memory * 1024 * 1024 // Convert MB to bytes
                ).ok();
            }
        }
        
        output
    }
    
    /// Export system metrics in Prometheus format
    pub fn export_system_metrics(&self, process_manager: &ProcessManager) -> String {
        let mut output = String::new();
        let sys_info = process_manager.get_system_info();
        
        // CPU count
        writeln!(output, "# HELP {}_system_cpu_count Number of CPU cores", self.namespace).ok();
        writeln!(output, "# TYPE {}_system_cpu_count gauge", self.namespace).ok();
        writeln!(output, "{}_system_cpu_count {}", self.namespace, sys_info.cpu_count).ok();
        
        // Load average
        writeln!(output, "# HELP {}_system_load_average System load average", self.namespace).ok();
        writeln!(output, "# TYPE {}_system_load_average gauge", self.namespace).ok();
        writeln!(output, "{}_system_load_average{{period=\"1m\"}} {}", self.namespace, sys_info.load_average.one).ok();
        writeln!(output, "{}_system_load_average{{period=\"5m\"}} {}", self.namespace, sys_info.load_average.five).ok();
        writeln!(output, "{}_system_load_average{{period=\"15m\"}} {}", self.namespace, sys_info.load_average.fifteen).ok();
        
        // Memory
        writeln!(output, "# HELP {}_system_memory_bytes System memory in bytes", self.namespace).ok();
        writeln!(output, "# TYPE {}_system_memory_bytes gauge", self.namespace).ok();
        writeln!(output, "{}_system_memory_bytes{{type=\"total\"}} {}", self.namespace, sys_info.total_memory * 1024).ok();
        writeln!(output, "{}_system_memory_bytes{{type=\"used\"}} {}", self.namespace, sys_info.used_memory * 1024).ok();
        writeln!(output, "{}_system_memory_bytes{{type=\"free\"}} {}", self.namespace, 
            (sys_info.total_memory - sys_info.used_memory) * 1024).ok();
        
        // Swap
        writeln!(output, "# HELP {}_system_swap_bytes System swap in bytes", self.namespace).ok();
        writeln!(output, "# TYPE {}_system_swap_bytes gauge", self.namespace).ok();
        writeln!(output, "{}_system_swap_bytes{{type=\"total\"}} {}", self.namespace, sys_info.total_swap * 1024).ok();
        writeln!(output, "{}_system_swap_bytes{{type=\"used\"}} {}", self.namespace, sys_info.used_swap * 1024).ok();
        
        // Uptime
        writeln!(output, "# HELP {}_system_uptime_seconds System uptime in seconds", self.namespace).ok();
        writeln!(output, "# TYPE {}_system_uptime_seconds counter", self.namespace).ok();
        writeln!(output, "{}_system_uptime_seconds {}", self.namespace, sys_info.uptime).ok();
        
        output
    }
    
    /// Export all metrics
    pub fn export_all(&self, process_manager: &ProcessManager) -> String {
        let mut output = String::new();
        output.push_str(&self.export_system_metrics(process_manager));
        output.push('\n');
        output.push_str(&self.export_process_metrics(process_manager));
        output
    }
    
    fn escape_label(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
    }
}

/// InfluxDB line protocol exporter
pub struct InfluxDBExporter {
    measurement_prefix: String,
}

impl InfluxDBExporter {
    pub fn new(measurement_prefix: &str) -> Self {
        Self {
            measurement_prefix: measurement_prefix.to_string(),
        }
    }
    
    /// Export process metrics in InfluxDB line protocol
    pub fn export_process_metrics(&self, process_manager: &ProcessManager) -> String {
        let mut output = String::new();
        let processes = process_manager.get_processes();
        let timestamp = Utc::now().timestamp_nanos_opt().unwrap_or(0);
        
        for process in &processes {
            // Basic process metrics
            writeln!(
                output,
                "{}_process,pid={},name={},user={} cpu_usage={},memory_bytes={},memory_percent={} {}",
                self.measurement_prefix,
                process.pid,
                Self::escape_tag(&process.name),
                Self::escape_tag(&process.user),
                process.cpu_usage,
                process.memory_usage * 1024,
                process.memory_percent,
                timestamp
            ).ok();
            
            // Network connections
            if let Some(connections) = process.network_connections {
                writeln!(
                    output,
                    "{}_process_network,pid={},name={} connections={} {}",
                    self.measurement_prefix,
                    process.pid,
                    Self::escape_tag(&process.name),
                    connections,
                    timestamp
                ).ok();
            }
            
            // Container info
            if process.is_container {
                let container_id = process.container_id.as_deref().unwrap_or("unknown");
                writeln!(
                    output,
                    "{}_process_container,pid={},name={},container_id={} in_container=1 {}",
                    self.measurement_prefix,
                    process.pid,
                    Self::escape_tag(&process.name),
                    Self::escape_tag(container_id),
                    timestamp
                ).ok();
            }
            
            // GPU memory
            if let Some(gpu_memory) = process.gpu_memory {
                writeln!(
                    output,
                    "{}_process_gpu,pid={},name={} memory_bytes={} {}",
                    self.measurement_prefix,
                    process.pid,
                    Self::escape_tag(&process.name),
                    gpu_memory * 1024 * 1024,
                    timestamp
                ).ok();
            }
        }
        
        output
    }
    
    /// Export system metrics in InfluxDB line protocol
    pub fn export_system_metrics(&self, process_manager: &ProcessManager) -> String {
        let mut output = String::new();
        let sys_info = process_manager.get_system_info();
        let timestamp = Utc::now().timestamp_nanos_opt().unwrap_or(0);
        
        // System-wide metrics
        writeln!(
            output,
            "{}_system cpu_count={},load_1m={},load_5m={},load_15m={},memory_total={},memory_used={},swap_total={},swap_used={},uptime={} {}",
            self.measurement_prefix,
            sys_info.cpu_count,
            sys_info.load_average.one,
            sys_info.load_average.five,
            sys_info.load_average.fifteen,
            sys_info.total_memory * 1024,
            sys_info.used_memory * 1024,
            sys_info.total_swap * 1024,
            sys_info.used_swap * 1024,
            sys_info.uptime,
            timestamp
        ).ok();
        
        output
    }
    
    /// Export all metrics
    pub fn export_all(&self, process_manager: &ProcessManager) -> String {
        let mut output = String::new();
        output.push_str(&self.export_system_metrics(process_manager));
        output.push_str(&self.export_process_metrics(process_manager));
        output
    }
    
    fn escape_tag(s: &str) -> String {
        s.replace(',', "\\,")
            .replace('=', "\\=")
            .replace(' ', "\\ ")
    }
}

/// Metrics exporter that supports multiple formats
pub struct MetricsExporter {
    prometheus: PrometheusExporter,
    influxdb: InfluxDBExporter,
}

impl MetricsExporter {
    pub fn new(namespace: &str) -> Self {
        Self {
            prometheus: PrometheusExporter::new(namespace),
            influxdb: InfluxDBExporter::new(namespace),
        }
    }
    
    /// Export metrics in the specified format
    pub fn export(&self, process_manager: &ProcessManager, format: ExportFormat) -> String {
        debug!("Exporting metrics in {:?} format", format);
        let result = match format {
            ExportFormat::Prometheus => self.prometheus.export_all(process_manager),
            ExportFormat::InfluxDB => self.influxdb.export_all(process_manager),
        };
        info!("Exported {} bytes of metrics in {:?} format", result.len(), format);
        result
    }
    
    /// Export to file
    pub fn export_to_file(
        &self,
        process_manager: &ProcessManager,
        format: ExportFormat,
        path: &str,
    ) -> std::io::Result<()> {
        let data = self.export(process_manager, format);
        std::fs::write(path, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_prometheus_escape_label() {
        assert_eq!(PrometheusExporter::escape_label("test"), "test");
        assert_eq!(PrometheusExporter::escape_label("test\\path"), "test\\\\path");
        assert_eq!(PrometheusExporter::escape_label("test\"quote"), "test\\\"quote");
    }
    
    #[test]
    fn test_influxdb_escape_tag() {
        assert_eq!(InfluxDBExporter::escape_tag("test"), "test");
        assert_eq!(InfluxDBExporter::escape_tag("test,tag"), "test\\,tag");
        assert_eq!(InfluxDBExporter::escape_tag("test=value"), "test\\=value");
        assert_eq!(InfluxDBExporter::escape_tag("test value"), "test\\ value");
    }
}
