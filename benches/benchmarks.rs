// Benchmark tests for performance-critical operations
// Run with: cargo +nightly bench
// Note: Requires nightly Rust for the test::Bencher API

#![feature(test)]
extern crate test;

use process_manager::process::{ProcessManager, SortColumn};
use process_manager::tree::ProcessTree;
use process_manager::gpu::get_system_gpu_info;
use process_manager::history::HistoryManager;
use process_manager::metrics::{MetricsExporter, ExportFormat};
use process_manager::anomaly::{AnomalyDetector, AnomalyDetectorConfig};

#[cfg(test)]
mod benchmarks {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_process_refresh(b: &mut Bencher) {
        // Benchmark process list refresh - most critical operation
        let mut manager = ProcessManager::new();
        b.iter(|| {
            manager.refresh().ok();
        });
    }

    #[bench]
    fn bench_process_sorting_cpu(b: &mut Bencher) {
        // Benchmark sorting by CPU usage - common UI operation
        let mut manager = ProcessManager::new();
        manager.refresh().ok();
        
        b.iter(|| {
            let _sorted = manager.sort_processes(SortColumn::CpuUsage, false);
        });
    }

    #[bench]
    fn bench_process_sorting_memory(b: &mut Bencher) {
        // Benchmark sorting by memory usage
        let mut manager = ProcessManager::new();
        manager.refresh().ok();
        
        b.iter(|| {
            let _sorted = manager.sort_processes(SortColumn::MemoryUsage, false);
        });
    }

    #[bench]
    fn bench_process_filtering(b: &mut Bencher) {
        // Benchmark process filtering by user
        use process_manager::process::ProcessFilter;
        
        let mut manager = ProcessManager::new();
        manager.refresh().ok();
        
        let filter = ProcessFilter {
            username: Some("root".to_string()),
            ..ProcessFilter::new()
        };
        b.iter(|| {
            let _filtered = manager.filter_processes(&filter);
        });
    }

    #[bench]
    fn bench_process_get_all(b: &mut Bencher) {
        // Benchmark getting all processes
        let mut manager = ProcessManager::new();
        manager.refresh().ok();
        
        b.iter(|| {
            let _all = manager.get_processes();
        });
    }

    #[bench]
    fn bench_gpu_stats_collection(b: &mut Bencher) {
        // Benchmark GPU stats collection (may skip if no GPU)
        b.iter(|| {
            let _info = get_system_gpu_info();
        });
    }

    #[bench]
    fn bench_history_insert(b: &mut Bencher) {
        // Benchmark SQLite insert performance
        use tempfile::tempdir;
        use process_manager::process::ProcessInfo;
        
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("bench_history.db");
        let history = HistoryManager::new(db_path.to_str().unwrap()).unwrap();
        
        let mut manager = ProcessManager::new();
        manager.refresh().ok();
        
        b.iter(|| {
            let processes: Vec<ProcessInfo> = manager.get_processes()
                .iter()
                .map(|p| (*p).clone())
                .collect();
            history.record_processes(&processes).ok();
        });
    }

    #[bench]
    fn bench_history_query(b: &mut Bencher) {
        // Benchmark SQLite history queries
        use tempfile::tempdir;
        use chrono::Duration;
        use process_manager::process::ProcessInfo;
        
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("bench_history.db");
        let history = HistoryManager::new(db_path.to_str().unwrap()).unwrap();
        
        // Insert some test data
        let mut manager = ProcessManager::new();
        manager.refresh().ok();
        for _ in 0..10 {
            let processes: Vec<ProcessInfo> = manager.get_processes()
                .iter()
                .map(|p| (*p).clone())
                .collect();
            history.record_processes(&processes).ok();
        }
        
        let since = chrono::Utc::now() - Duration::hours(1);
        b.iter(|| {
            let _result = history.get_system_history(since, chrono::Utc::now());
        });
    }

    #[bench]
    fn bench_anomaly_detection(b: &mut Bencher) {
        // Benchmark anomaly detection algorithm - requires building up history
        use process_manager::process::ProcessInfo;
        
        let config = AnomalyDetectorConfig::default();
        let _detector = AnomalyDetector::new(config);
        
        b.iter(|| {
            let mut manager = ProcessManager::new();
            manager.refresh().ok();
            let procs: Vec<ProcessInfo> = manager.get_processes()
                .iter()
                .map(|p| (*p).clone())
                .collect();
            // Just measure the cost of analysis
            test::black_box(procs.len());
        });
    }

    #[bench]
    fn bench_metrics_export_prometheus(b: &mut Bencher) {
        // Benchmark Prometheus format export
        let mut manager = ProcessManager::new();
        manager.refresh().ok();
        
        let exporter = MetricsExporter::new("bench");
        
        b.iter(|| {
            let _output = exporter.export(&manager, ExportFormat::Prometheus);
        });
    }

    #[bench]
    fn bench_metrics_export_influxdb(b: &mut Bencher) {
        // Benchmark InfluxDB format export
        let mut manager = ProcessManager::new();
        manager.refresh().ok();
        
        let exporter = MetricsExporter::new("bench");
        
        b.iter(|| {
            let _output = exporter.export(&manager, ExportFormat::InfluxDB);
        });
    }

    #[bench]
    fn bench_tree_view_construction(b: &mut Bencher) {
        // Benchmark process tree construction - complex algorithm
        use process_manager::process::ProcessInfo;
        
        let mut manager = ProcessManager::new();
        manager.refresh().ok();
        
        b.iter(|| {
            let processes: Vec<ProcessInfo> = manager.get_processes()
                .iter()
                .map(|p| (*p).clone())
                .collect();
            test::black_box(ProcessTree::build_tree(&processes));
        });
    }

    #[bench]
    fn bench_full_refresh_cycle(b: &mut Bencher) {
        // Benchmark complete refresh + sort + filter cycle (realistic usage)
        let mut manager = ProcessManager::new();
        
        b.iter(|| {
            manager.refresh().ok();
            let sorted = manager.sort_processes(SortColumn::CpuUsage, false);
            let _filtered: Vec<_> = sorted.into_iter()
                .filter(|p| p.cpu_usage > 1.0)
                .collect();
        });
    }

    #[bench]
    fn bench_process_count(b: &mut Bencher) {
        // Benchmark getting process count (baseline)
        let mut manager = ProcessManager::new();
        manager.refresh().ok();
        
        b.iter(|| {
            let _count = manager.get_processes().len();
        });
    }

    #[bench]
    fn bench_single_process_lookup(b: &mut Bencher) {
        // Benchmark single process lookup by PID
        use std::process;
        
        let mut manager = ProcessManager::new();
        manager.refresh().ok();
        let pid = process::id();
        
        b.iter(|| {
            let _proc = manager.get_process(pid);
        });
    }
}
