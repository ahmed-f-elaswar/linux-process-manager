use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use tracing::{debug, info};

use crate::process::ProcessInfo;

/// Historical data manager for storing process statistics
pub struct HistoryManager {
    conn: Connection,
}

impl HistoryManager {
    /// Create a new history manager with SQLite backend
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)
            .context("Failed to open history database")?;
        
        let manager = Self { conn };
        manager.initialize_db()?;
        
        Ok(manager)
    }

    /// Initialize database schema
    fn initialize_db(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS process_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                pid INTEGER NOT NULL,
                name TEXT NOT NULL,
                user_name TEXT NOT NULL,
                cpu_usage REAL NOT NULL,
                memory_usage INTEGER NOT NULL,
                memory_percent REAL NOT NULL,
                command TEXT
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON process_history(timestamp)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_pid ON process_history(pid)",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS system_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                cpu_count INTEGER NOT NULL,
                load_avg_1 REAL NOT NULL,
                load_avg_5 REAL NOT NULL,
                load_avg_15 REAL NOT NULL,
                total_memory INTEGER NOT NULL,
                used_memory INTEGER NOT NULL,
                total_swap INTEGER NOT NULL,
                used_swap INTEGER NOT NULL,
                uptime INTEGER NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sys_timestamp ON system_history(timestamp)",
            [],
        )?;

        Ok(())
    }

    /// Record process snapshot
    pub fn record_processes(&self, processes: &[ProcessInfo]) -> Result<()> {
        debug!("Recording {} processes to history database", processes.len());
        let start = std::time::Instant::now();
        let timestamp = Utc::now().timestamp();
        
        let mut stmt = self.conn.prepare(
            "INSERT INTO process_history 
             (timestamp, pid, name, user_name, cpu_usage, memory_usage, memory_percent, command)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )?;

        let mut inserted = 0;
        let mut errors = 0;
        
        for process in processes {
            match stmt.execute(params![
                timestamp,
                process.pid,
                process.name,
                process.user,
                process.cpu_usage,
                process.memory_usage,
                process.memory_percent,
                process.command,
            ]) {
                Ok(_) => inserted += 1,
                Err(e) => {
                    debug!("Failed to insert process {}: {}", process.pid, e);
                    errors += 1;
                }
            }
        }

        let duration = start.elapsed();
        info!(
            inserted = inserted,
            errors = errors,
            duration_ms = duration.as_millis(),
            "Process history recorded"
        );

        Ok(())
    }

    /// Record system statistics
    pub fn record_system_stats(
        &self,
        cpu_count: usize,
        load_avg: (f64, f64, f64),
        total_memory: u64,
        used_memory: u64,
        total_swap: u64,
        used_swap: u64,
        uptime: u64,
    ) -> Result<()> {
        let timestamp = Utc::now().timestamp();

        self.conn.execute(
            "INSERT INTO system_history 
             (timestamp, cpu_count, load_avg_1, load_avg_5, load_avg_15,
              total_memory, used_memory, total_swap, used_swap, uptime)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                timestamp,
                cpu_count,
                load_avg.0,
                load_avg.1,
                load_avg.2,
                total_memory,
                used_memory,
                total_swap,
                used_swap,
                uptime,
            ],
        )?;

        Ok(())
    }

    /// Get process history for a specific PID
    pub fn get_process_history(
        &self,
        pid: u32,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<HistoricalProcessData>> {
        let mut stmt = self.conn.prepare(
            "SELECT timestamp, pid, name, user_name, cpu_usage, memory_usage, memory_percent, command
             FROM process_history
             WHERE pid = ? AND timestamp BETWEEN ? AND ?
             ORDER BY timestamp ASC"
        )?;

        let rows = stmt.query_map(
            params![pid, start_time.timestamp(), end_time.timestamp()],
            |row| {
                Ok(HistoricalProcessData {
                    timestamp: row.get(0)?,
                    pid: row.get(1)?,
                    name: row.get(2)?,
                    user_name: row.get(3)?,
                    cpu_usage: row.get(4)?,
                    memory_usage: row.get(5)?,
                    memory_percent: row.get(6)?,
                    command: row.get(7)?,
                })
            },
        )?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }

    /// Get system history
    pub fn get_system_history(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<HistoricalSystemData>> {
        let mut stmt = self.conn.prepare(
            "SELECT timestamp, cpu_count, load_avg_1, load_avg_5, load_avg_15,
                    total_memory, used_memory, total_swap, used_swap, uptime
             FROM system_history
             WHERE timestamp BETWEEN ? AND ?
             ORDER BY timestamp ASC"
        )?;

        let rows = stmt.query_map(
            params![start_time.timestamp(), end_time.timestamp()],
            |row| {
                Ok(HistoricalSystemData {
                    timestamp: row.get(0)?,
                    cpu_count: row.get(1)?,
                    load_avg_1: row.get(2)?,
                    load_avg_5: row.get(3)?,
                    load_avg_15: row.get(4)?,
                    total_memory: row.get(5)?,
                    used_memory: row.get(6)?,
                    total_swap: row.get(7)?,
                    used_swap: row.get(8)?,
                    uptime: row.get(9)?,
                })
            },
        )?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }

    /// Get top CPU consumers over time period
    pub fn get_top_cpu_consumers(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        limit: usize,
    ) -> Result<Vec<(String, f32)>> {
        let mut stmt = self.conn.prepare(
            "SELECT name, AVG(cpu_usage) as avg_cpu
             FROM process_history
             WHERE timestamp BETWEEN ? AND ?
             GROUP BY name
             ORDER BY avg_cpu DESC
             LIMIT ?"
        )?;

        let rows = stmt.query_map(
            params![start_time.timestamp(), end_time.timestamp(), limit],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, f32>(1)?)),
        )?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }

    /// Clean old data (older than specified days)
    pub fn clean_old_data(&self, days: i64) -> Result<usize> {
        let cutoff = Utc::now().timestamp() - (days * 86400);
        
        let deleted = self.conn.execute(
            "DELETE FROM process_history WHERE timestamp < ?",
            params![cutoff],
        )?;

        self.conn.execute(
            "DELETE FROM system_history WHERE timestamp < ?",
            params![cutoff],
        )?;

        // Vacuum to reclaim space
        self.conn.execute("VACUUM", [])?;

        Ok(deleted)
    }

    /// Get database size
    pub fn get_db_size(&self) -> Result<u64> {
        let page_count: i64 = self.conn.query_row(
            "PRAGMA page_count",
            [],
            |row| row.get(0),
        )?;

        let page_size: i64 = self.conn.query_row(
            "PRAGMA page_size",
            [],
            |row| row.get(0),
        )?;

        Ok((page_count * page_size) as u64)
    }
}

/// Historical process data point
#[derive(Debug, Clone, serde::Serialize)]
pub struct HistoricalProcessData {
    pub timestamp: i64,
    pub pid: u32,
    pub name: String,
    pub user_name: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub memory_percent: f32,
    pub command: String,
}

/// Historical system data point
#[derive(Debug, Clone, serde::Serialize)]
pub struct HistoricalSystemData {
    pub timestamp: i64,
    pub cpu_count: usize,
    pub load_avg_1: f64,
    pub load_avg_5: f64,
    pub load_avg_15: f64,
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub uptime: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_history_manager() -> Result<()> {
        let db_path = "/tmp/test_process_history.db";
        
        // Clean up if exists
        let _ = fs::remove_file(db_path);

        let manager = HistoryManager::new(db_path)?;
        
        // Test recording system stats
        manager.record_system_stats(
            4,
            (1.0, 0.8, 0.5),
            8192,
            4096,
            2048,
            512,
            10000,
        )?;

        // Clean up
        fs::remove_file(db_path)?;

        Ok(())
    }
}