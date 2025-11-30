# System Architecture

## Overview

The Linux Process Manager is designed with a modular architecture that separates concerns into distinct components. This document describes the high-level architecture, data flow, and design decisions.

---

## High-Level Architecture Diagram

```
+------------------------------------------------------------------+
|                     Linux Process Manager                          |
+------------------------------------------------------------------+
|                                                                    |
|  +------------------+     +------------------+     +-------------+ |
|  |   Terminal UI    |     |    REST API      |     |   Web UI    | |
|  |   (ratatui)      |     |   (actix-web)    |     |  (React)    | |
|  +--------+---------+     +--------+---------+     +------+------+ |
|           |                        |                      |        |
|           v                        v                      v        |
|  +------------------------------------------------------------------+
|  |                        Core Process Manager                      |
|  |  +------------+  +------------+  +------------+  +------------+  |
|  |  | Process    |  | System     |  | Filter     |  | Sort       |  |
|  |  | Collector  |  | Metrics    |  | Engine     |  | Engine     |  |
|  |  +------------+  +------------+  +------------+  +------------+  |
|  +------------------------------------------------------------------+
|           |                        |                      |        |
|           v                        v                      v        |
|  +------------------------------------------------------------------+
|  |                        Feature Modules                           |
|  |  +--------+ +----------+ +---------+ +--------+ +-------------+  |
|  |  | GPU    | | Network  | | History | | Alerts | | Containers  |  |
|  |  +--------+ +----------+ +---------+ +--------+ +-------------+  |
|  |  +--------+ +----------+ +---------+ +--------+ +-------------+  |
|  |  |Logging | | Affinity | |Snapshots| | Groups | | Memory Map  |  |
|  |  +--------+ +----------+ +---------+ +--------+ +-------------+  |
|  +------------------------------------------------------------------+
|           |                        |                      |        |
|           v                        v                      v        |
|  +------------------------------------------------------------------+
|  |                      Linux Kernel Interface                      |
|  |  /proc filesystem    |    syscalls    |    /sys filesystem       |
|  +------------------------------------------------------------------+
|                                                                    |
+------------------------------------------------------------------+
```

---

## Component Architecture

### 1. Presentation Layer

```
+-------------------+     +-------------------+     +-------------------+
|   Terminal UI     |     |    REST API       |     |     Web UI        |
|   (src/ui.rs)     |     |   (src/api.rs)    |     |     (web/)        |
+-------------------+     +-------------------+     +-------------------+
|                   |     |                   |     |                   |
| - ratatui widgets |     | - actix-web 4.0   |     | - React 18        |
| - crossterm input |     | - JSON responses  |     | - TypeScript      |
| - Sparkline graphs|     | - CORS support    |     | - Tailwind CSS    |
| - Tree view       |     | - 9 endpoints     |     | - Recharts        |
|                   |     |                   |     |                   |
+-------------------+     +-------------------+     +-------------------+
         |                         |                        |
         +-----------+-------------+------------------------+
                     |
                     v
            +-------------------+
            |  ProcessManager   |
            |  (src/process.rs) |
            +-------------------+
```

### 2. Core Layer

```
+-----------------------------------------------------------------------+
|                         ProcessManager                                  |
+-----------------------------------------------------------------------+
|  pub struct ProcessManager {                                           |
|      processes: Vec<ProcessInfo>,      // Current process list         |
|      system: System,                   // System info (sysinfo crate)  |
|      filters: FilterConfig,            // Active filters               |
|      sort_config: SortConfig,          // Current sort settings        |
|  }                                                                      |
+-----------------------------------------------------------------------+
|  Key Methods:                                                          |
|  - refresh()           -> Update process list from /proc               |
|  - get_processes()     -> Return filtered/sorted process list          |
|  - kill_process()      -> Send signal to process                       |
|  - get_system_info()   -> Return CPU/memory/load statistics            |
+-----------------------------------------------------------------------+
```

### 3. Data Structures

```
ProcessInfo                          SystemInfo
+------------------------+           +------------------------+
| pid: u32               |           | cpu_count: usize       |
| name: String           |           | total_memory: u64      |
| user: String           |           | used_memory: u64       |
| cpu_usage: f32         |           | load_average: LoadAvg  |
| memory_usage: u64      |           | uptime: u64            |
| state: ProcessState    |           +------------------------+
| command: String        |
| ppid: Option<u32>      |           FilterConfig
| start_time: u64        |           +------------------------+
| threads: u32           |           | user: Option<String>   |
| is_container: bool     |           | name_pattern: Option   |
| container_id: Option   |           | min_cpu: Option<f32>   |
| gpu_memory: Option<u64>|           | min_memory: Option<u64>|
| network_conns: Vec     |           +------------------------+
+------------------------+
```

---

## Data Flow Diagrams

### Process Refresh Flow

```
+----------+     +------------+     +------------+     +----------+
|  Timer   | --> | Refresh    | --> | Parse      | --> | Filter   |
| (2 sec)  |     | Trigger    |     | /proc      |     | & Sort   |
+----------+     +------------+     +------------+     +----------+
                                           |                |
                                           v                v
                              +------------+     +----------+
                              | ProcessInfo| --> | Display  |
                              | Vec        |     | Update   |
                              +------------+     +----------+
```

### API Request Flow

```
+----------+     +------------+     +------------+     +----------+
| HTTP     | --> | actix-web  | --> | Handler    | --> | Process  |
| Request  |     | Router     |     | Function   |     | Manager  |
+----------+     +------------+     +------------+     +----------+
                                                            |
                                                            v
+----------+     +------------+     +------------+     +----------+
| HTTP     | <-- | Serialize  | <-- | Build      | <-- | Query    |
| Response |     | to JSON    |     | Response   |     | Result   |
+----------+     +------------+     +------------+     +----------+
```

### Signal Handling Flow

```
+----------+     +------------+     +------------+     +----------+
| User     | --> | Validate   | --> | Permission | --> | Send     |
| Request  |     | PID/Signal |     | Check      |     | Signal   |
+----------+     +------------+     +------------+     +----------+
                       |                  |                |
                       v                  v                v
                 +------------+     +------------+     +----------+
                 | Error:     |     | Error:     |     | Success  |
                 | Invalid    |     | Permission |     | Response |
                 +------------+     +------------+     +----------+
```

---

## Module Dependencies

```
                              main.rs
                                 |
                 +---------------+---------------+
                 |               |               |
                 v               v               v
              ui.rs          api.rs         config.rs
                 |               |               |
                 +-------+-------+               |
                         |                       |
                         v                       |
                    process.rs <-----------------+
                         |
         +-------+-------+-------+-------+-------+
         |       |       |       |       |       |
         v       v       v       v       v       v
      tree.rs gpu.rs network.rs history.rs alerts.rs containers.rs
                         |
                         v
                    anomaly.rs
                         |
         +-------+-------+-------+-------+
         |       |       |       |       |
         v       v       v       v       v
    logging.rs affinity.rs snapshots.rs groups.rs memmap.rs
```

---

## Concurrency Model

### Threading Architecture

```
+-------------------+
|    Main Thread    |
|  (UI Event Loop)  |
+-------------------+
         |
         | spawn
         v
+-------------------+     +-------------------+
|  Refresh Thread   |     |   API Thread      |
| (sysinfo polling) |     | (actix runtime)   |
+-------------------+     +-------------------+
         |                         |
         | channel                 | async
         v                         v
+-------------------+     +-------------------+
|  Process Data     |     |   Request         |
|  (Arc<RwLock>)    |     |   Handlers        |
+-------------------+     +-------------------+
```

### Synchronization

| Component | Mechanism | Purpose |
|-----------|-----------|---------|
| Process list | `Arc<RwLock>` | Thread-safe read/write access |
| UI state | Single-threaded | No synchronization needed |
| API handlers | Async | Non-blocking I/O |
| Database | SQLite mutex | Single-writer, multi-reader |
| Logging | `tracing` | Lock-free async logging |

---

## Error Handling Strategy

```
+-------------------+
|   Error Types     |
+-------------------+
| - ProcessError    |  -> Process operations (kill, query)
| - ApiError        |  -> HTTP request/response errors
| - ConfigError     |  -> Configuration parsing errors
| - DatabaseError   |  -> SQLite operations
| - IoError         |  -> File system operations
+-------------------+
         |
         | wrapped by
         v
+-------------------+
|   anyhow::Error   |
|   (context chain) |
+-------------------+
         |
         | displayed as
         v
+-------------------+
|   User Message    |
|   + Log Entry     |
+-------------------+
```

---

## Database Schema

```sql
-- Historical process data
CREATE TABLE process_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pid INTEGER NOT NULL,
    name TEXT NOT NULL,
    cpu_usage REAL NOT NULL,
    memory_usage INTEGER NOT NULL,
    timestamp INTEGER NOT NULL,
    UNIQUE(pid, timestamp)
);

-- Index for efficient queries
CREATE INDEX idx_process_history_pid_time
ON process_history(pid, timestamp);

-- Anomaly events
CREATE TABLE anomaly_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pid INTEGER NOT NULL,
    anomaly_type TEXT NOT NULL,
    description TEXT NOT NULL,
    timestamp INTEGER NOT NULL
);

-- Saved snapshots
CREATE TABLE snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    data BLOB NOT NULL,
    created_at INTEGER NOT NULL
);
```

---

## API Architecture

### REST Endpoints

```
/api
├── /health          GET   -> Health check
├── /system          GET   -> System information
├── /processes       GET   -> List all processes
│   ├── ?sort_by=cpu|memory|pid|name
│   ├── ?ascending=true|false
│   ├── ?name=<pattern>
│   └── ?user=<username>
├── /processes/{pid} GET   -> Single process details
├── /processes/kill  POST  -> Send signal to process
│   └── Body: { "pid": 1234, "signal": 15 }
├── /history
│   └── /processes/{pid} GET -> Historical data
├── /gpu             GET   -> GPU statistics
├── /containers      GET   -> Container list
└── /alerts          GET   -> Active alerts
```

### Response Format

```json
{
  "success": true,
  "data": { ... },
  "timestamp": 1234567890,
  "error": null
}
```

---

## Security Architecture

```
+-------------------+     +-------------------+     +-------------------+
|   Input Layer     | --> | Validation Layer  | --> | Execution Layer   |
+-------------------+     +-------------------+     +-------------------+
|                   |     |                   |     |                   |
| - HTTP requests   |     | - PID validation  |     | - Signal dispatch |
| - CLI arguments   |     | - Signal check    |     | - Process queries |
| - Config files    |     | - Path sanitize   |     | - File operations |
|                   |     | - Regex safety    |     |                   |
+-------------------+     +-------------------+     +-------------------+
                                    |
                                    v
                          +-------------------+
                          |   Permission      |
                          |   Checks          |
                          +-------------------+
                          | - User ownership  |
                          | - Capability check|
                          | - Protected PIDs  |
                          +-------------------+
```

---

## Deployment Architecture

### Single Container Deployment

```
+--------------------------------------------------+
|                 Docker Container                  |
|  +--------------------------------------------+  |
|  |              Alpine Linux 3.19              |  |
|  |  +----------------+  +------------------+   |  |
|  |  | process-manager|  | Static Web Files |   |  |
|  |  | (Rust binary)  |  | (React build)    |   |  |
|  |  +-------+--------+  +--------+---------+   |  |
|  |          |                    |             |  |
|  |          +--------+-----------+             |  |
|  |                   |                         |  |
|  |          +--------v---------+               |  |
|  |          | actix-web server |               |  |
|  |          | Port 8080        |               |  |
|  |          +------------------+               |  |
|  +--------------------------------------------+  |
+--------------------------------------------------+
                       |
                       | :8080
                       v
+--------------------------------------------------+
|                   Host System                     |
|  - /proc filesystem (read-only)                  |
|  - /sys filesystem (read-only)                   |
|  - Volume mount for data persistence             |
+--------------------------------------------------+
```

---

## Design Decisions

### 1. Why Rust?

| Factor | Justification |
|--------|---------------|
| Memory safety | No buffer overflows, use-after-free |
| Performance | Zero-cost abstractions, no GC |
| Concurrency | Fearless concurrency with ownership |
| Ecosystem | Strong crates for TUI, HTTP, SQLite |
| Reliability | Compiler enforces correctness |

### 2. Why SQLite for History?

| Factor | Justification |
|--------|---------------|
| Simplicity | No separate database server |
| Performance | Fast for time-series queries |
| Portability | Single file, easy backup |
| Reliability | ACID compliant, WAL mode |

### 3. Why actix-web?

| Factor | Justification |
|--------|---------------|
| Performance | One of the fastest Rust web frameworks |
| Async | Non-blocking I/O for API handlers |
| Ecosystem | Mature, well-documented |
| Features | Built-in CORS, middleware support |

### 4. Why React for Web UI?

| Factor | Justification |
|--------|---------------|
| Ecosystem | Largest component ecosystem |
| TypeScript | Type safety for API contracts |
| Performance | Virtual DOM, efficient updates |
| Tooling | Excellent dev experience with Vite |
