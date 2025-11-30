# Functional Requirements Specification

## Overview

This document details all 27 functional requirements implemented in the Linux Process Manager. Requirements are organized by priority level and include acceptance criteria.

---

## Priority 1: Core Features (6/6 Complete)

### FR-01: Display Running Processes

**Description**: Display all running processes with essential information including PID, name, user, CPU%, and memory usage.

**Implementation**: `src/process.rs`

**Acceptance Criteria**:
- [x] List all processes visible to the current user
- [x] Display PID (Process ID)
- [x] Display process name
- [x] Display owner username
- [x] Display CPU usage percentage
- [x] Display memory usage (in MB)
- [x] Display process state (Running, Sleeping, Zombie, etc.)
- [x] Display command line arguments
- [x] Display start time

**API Endpoint**: `GET /api/processes`

---

### FR-02: Kill Processes with Signal Selection

**Description**: Allow users to send various signals to processes, not just SIGKILL.

**Implementation**: `src/process.rs`, `src/api.rs`

**Supported Signals**:
| Signal | Number | Description |
|--------|--------|-------------|
| SIGTERM | 15 | Graceful termination |
| SIGKILL | 9 | Forceful termination |
| SIGHUP | 1 | Hangup |
| SIGINT | 2 | Interrupt |
| SIGSTOP | 19 | Stop process |
| SIGCONT | 18 | Continue stopped process |
| SIGUSR1 | 10 | User-defined signal 1 |
| SIGUSR2 | 12 | User-defined signal 2 |
| SIGQUIT | 3 | Quit |

**Acceptance Criteria**:
- [x] Support at least 5 different signals
- [x] Require confirmation before sending signals
- [x] Handle permission errors gracefully
- [x] Report success/failure to user

**API Endpoint**: `POST /api/processes/kill`

---

### FR-03: Sort by Any Column

**Description**: Allow sorting the process list by any displayed column.

**Implementation**: `src/process.rs`

**Sortable Columns**:
- PID (ascending/descending)
- Name (alphabetical)
- User (alphabetical)
- CPU % (ascending/descending)
- Memory (ascending/descending)
- Start time (oldest/newest)

**Acceptance Criteria**:
- [x] Click column header to sort
- [x] Toggle ascending/descending
- [x] Visual indicator of sort direction
- [x] Persist sort preference during session

**Keyboard Shortcuts**:
- `c` - Sort by CPU
- `m` - Sort by Memory
- `p` - Sort by PID
- `n` - Sort by Name

---

### FR-04: Filter Processes

**Description**: Filter processes by user, name pattern, or resource thresholds.

**Implementation**: `src/process.rs`

**Filter Types**:
1. **User filter**: Show only processes owned by specific user
2. **Name filter**: Regex pattern matching on process name
3. **CPU threshold**: Show processes above X% CPU
4. **Memory threshold**: Show processes above X MB memory

**Acceptance Criteria**:
- [x] Filter by username (exact match)
- [x] Filter by name pattern (regex support)
- [x] Filter by CPU threshold
- [x] Filter by memory threshold
- [x] Combine multiple filters (AND logic)
- [x] Clear filters easily

**Keyboard Shortcut**: `/` to enter search mode

---

### FR-05: Tree View

**Description**: Display process hierarchy showing parent-child relationships.

**Implementation**: `src/tree.rs`

**Features**:
- Visual tree structure with indent levels
- Show parent PID (PPID) for each process
- Collapse/expand subtrees
- Highlight orphan processes

**Acceptance Criteria**:
- [x] Show hierarchical process tree
- [x] Display parent-child relationships
- [x] Visual indentation for hierarchy levels
- [x] Toggle between flat and tree view

**Keyboard Shortcut**: `t` to toggle tree view

---

### FR-06: Real-time Updates

**Description**: Automatically refresh process information at configurable intervals.

**Implementation**: `src/ui.rs`, `src/config.rs`

**Configuration**:
- Default refresh interval: 2 seconds
- Minimum: 1 second
- Maximum: 60 seconds
- Configurable via CLI (`--refresh`) or config file

**Acceptance Criteria**:
- [x] Auto-refresh at configurable interval
- [x] Manual refresh option (F5 key)
- [x] Pause/resume updates
- [x] Display time since last update
- [x] Low CPU overhead during refresh

**Keyboard Shortcuts**:
- `F5` - Manual refresh
- `+/-` - Adjust refresh rate

---

## Priority 2: Advanced Features (6/6 Complete)

### FR-07: Network Connection Monitoring

**Description**: Display per-process network connections.

**Implementation**: `src/network.rs`

**Information Displayed**:
- Local address and port
- Remote address and port
- Connection state (ESTABLISHED, LISTEN, etc.)
- Protocol (TCP/UDP)

**Acceptance Criteria**:
- [x] Parse `/proc/net/tcp` and `/proc/net/udp`
- [x] Associate connections with PIDs
- [x] Display connection count per process
- [x] Show detailed connection info on demand

---

### FR-08: Container/Cgroup Awareness

**Description**: Detect and display container information for processes.

**Implementation**: `src/containers.rs`, `src/network.rs`

**Container Support**:
- Docker containers
- Kubernetes pods
- LXC containers
- systemd services

**Information Displayed**:
- Container ID
- Container name
- Cgroup path
- Resource limits (CPU, memory)
- Namespace information

**Acceptance Criteria**:
- [x] Detect containerized processes
- [x] Display container name/ID
- [x] Show cgroup resource limits
- [x] Parse namespace information

---

### FR-09: Historical Data Storage

**Description**: Store process metrics over time for historical analysis.

**Implementation**: `src/history.rs`

**Storage**:
- SQLite database
- Configurable retention period (default: 30 days)
- Recording interval: 60 seconds

**Stored Metrics**:
- CPU usage over time
- Memory usage over time
- Process start/stop events
- Anomaly events

**Acceptance Criteria**:
- [x] Store metrics in SQLite database
- [x] Query historical data by PID and time range
- [x] Automatic data pruning based on retention policy
- [x] Export historical data

**API Endpoint**: `GET /api/history/processes/{pid}`

---

### FR-10: System-wide Resource Graphs

**Description**: Display real-time graphs of system resource usage.

**Implementation**: `src/ui.rs`

**Graphs**:
- CPU usage sparkline (per-core and total)
- Memory usage sparkline
- Unicode block characters for visualization

**Acceptance Criteria**:
- [x] Real-time CPU graph
- [x] Real-time memory graph
- [x] Historical data in graphs (last N samples)
- [x] Per-core CPU breakdown

**Keyboard Shortcut**: `g` to toggle graphs

---

### FR-11: Process Search with Regex

**Description**: Advanced search with regular expression support.

**Implementation**: `src/process.rs`

**Features**:
- Full regex pattern support
- Case-insensitive option
- Search in process name and command line
- Highlight matches

**Acceptance Criteria**:
- [x] Support standard regex syntax
- [x] Search in name and command
- [x] Real-time search results
- [x] Clear search easily

---

### FR-12: Batch Operations

**Description**: Perform operations on multiple processes at once.

**Implementation**: `src/process.rs`

**Batch Operations**:
- Kill multiple processes
- Send signals to multiple processes
- Select by pattern or criteria

**Acceptance Criteria**:
- [x] Select multiple processes
- [x] Apply signal to all selected
- [x] Confirmation before batch kill
- [x] Report success/failure for each

---

## Priority 3: Innovative Features (6/6 Complete)

### FR-13: GPU Monitoring

**Description**: Monitor GPU usage and per-process GPU memory.

**Implementation**: `src/gpu.rs`

**Supported GPUs**:
- NVIDIA (via nvidia-smi)
- AMD (via rocm-smi)
- Intel (via sysfs)

**Metrics**:
- GPU utilization %
- GPU memory usage
- Per-process GPU memory
- GPU temperature (where available)

**Acceptance Criteria**:
- [x] Detect GPU presence
- [x] Display GPU utilization
- [x] Show per-process GPU memory
- [x] Graceful degradation if no GPU

**API Endpoint**: `GET /api/gpu`

---

### FR-14: Web UI for Remote Access

**Description**: Browser-based interface for remote monitoring.

**Implementation**: `web/` (React + TypeScript)

**Features**:
- Modern dashboard design
- Real-time updates
- Process management
- System metrics visualization
- Dark/light theme

**Acceptance Criteria**:
- [x] Responsive web interface
- [x] All core features accessible
- [x] Real-time data refresh
- [x] Mobile-friendly design

---

### FR-15: REST API

**Description**: Programmatic API for integration with other tools.

**Implementation**: `src/api.rs`

**Endpoints**:
| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/health` | GET | Health check |
| `/api/system` | GET | System information |
| `/api/processes` | GET | List processes |
| `/api/processes/{pid}` | GET | Single process |
| `/api/processes/kill` | POST | Send signal |
| `/api/history/processes/{pid}` | GET | Process history |
| `/api/gpu` | GET | GPU stats |
| `/api/containers` | GET | Containers |
| `/api/alerts` | GET | Active alerts |

**Acceptance Criteria**:
- [x] RESTful design
- [x] JSON responses
- [x] CORS support
- [x] Error handling with proper status codes

---

### FR-16: Prometheus/InfluxDB Export

**Description**: Export metrics in formats compatible with monitoring systems.

**Implementation**: `src/metrics.rs`

**Export Formats**:
- Prometheus text format
- InfluxDB line protocol

**Acceptance Criteria**:
- [x] Prometheus-compatible metrics endpoint
- [x] InfluxDB line protocol export
- [x] Configurable export interval
- [x] File or HTTP export options

---

### FR-17: Anomaly Detection

**Description**: Automatically detect unusual process behavior.

**Implementation**: `src/anomaly.rs`

**Detected Anomalies**:
| Type | Description | Default Threshold |
|------|-------------|-------------------|
| CPU Spike | Sudden CPU increase | >80% |
| Memory Spike | Sudden memory increase | >85% |
| Rapid Respawn | Process restarting frequently | >3 in 60s |
| Excessive Connections | Too many network connections | >100 |
| Sudden Termination | Unexpected process death | N/A |
| Unusual GPU | Unexpected GPU usage | >50% |

**Acceptance Criteria**:
- [x] Statistical analysis for anomaly detection
- [x] Configurable thresholds
- [x] Alert generation for detected anomalies
- [x] Low false-positive rate

---

### FR-18: Kubernetes Pod Aggregation

**Description**: Aggregate metrics at the pod level for Kubernetes environments.

**Implementation**: `src/containers.rs`

**Features**:
- Pod-level CPU aggregation
- Pod-level memory aggregation
- Container count per pod
- Pod label detection

**Acceptance Criteria**:
- [x] Detect Kubernetes pods
- [x] Aggregate container metrics
- [x] Display pod-level view
- [x] Support pod filtering

---

## Phase IV: Advanced Enhancements (9/9 Complete)

### FR-19: Structured Logging

**Description**: Comprehensive logging with rotation and structured output.

**Implementation**: `src/logging.rs`

**Features**:
- JSON and text log formats
- Log rotation (size/time-based)
- Multiple log levels
- Async logging for performance

---

### FR-20: CPU Affinity & Priority Management

**Description**: View and modify CPU affinity and process priority.

**Implementation**: `src/affinity.rs`

**Features**:
- Display CPU affinity mask
- Display nice value
- Modify nice value (requires privileges)
- View scheduling policy

---

### FR-21: Process Snapshots

**Description**: Capture and compare process state over time.

**Implementation**: `src/snapshots.rs`

**Features**:
- Save process list snapshot
- Load and compare snapshots
- Identify new/removed/changed processes

---

### FR-22: Smart Alerts

**Description**: Configurable alerts via multiple notification channels.

**Implementation**: `src/alerts.rs`

**Notification Channels**:
- Desktop notifications (notify-rust)
- Email (lettre)
- Webhooks (HTTP POST)

---

### FR-23: Process Group Operations

**Description**: Operate on process groups and sessions.

**Implementation**: `src/groups.rs`

**Features**:
- Display PGID and SID
- Kill entire process group
- Session-based filtering

---

### FR-24: Memory Map Visualization

**Description**: Display detailed process memory maps.

**Implementation**: `src/memmap.rs`

**Information**:
- Memory regions with addresses
- Permissions (read/write/execute)
- Mapped files
- Anonymous mappings

---

### FR-25: Saved View Profiles

**Description**: Save and load custom view configurations.

**Implementation**: `src/profiles.rs`

**Features**:
- Save column selection
- Save sort preferences
- Save filter configurations
- Quick profile switching

---

### FR-26: Process State Diffing

**Description**: Compare process states between two points in time.

**Implementation**: `src/diffing.rs`

**Features**:
- Identify new processes
- Identify terminated processes
- Highlight resource changes

---

### FR-27: Container Deep Dive

**Description**: Detailed container runtime analysis.

**Implementation**: `src/containers.rs`

**Features**:
- Full cgroup hierarchy
- Namespace isolation details
- Resource limit parsing
- Container image information

---

## Requirements Traceability Matrix

| Requirement | Module | API Endpoint | Test Coverage |
|-------------|--------|--------------|---------------|
| FR-01 | process.rs | /api/processes | Yes |
| FR-02 | process.rs, api.rs | /api/processes/kill | Yes |
| FR-03 | process.rs | /api/processes?sort_by= | Yes |
| FR-04 | process.rs | /api/processes?name= | Yes |
| FR-05 | tree.rs | N/A (TUI only) | Yes |
| FR-06 | ui.rs | N/A | Yes |
| FR-07 | network.rs | N/A | Yes |
| FR-08 | containers.rs | /api/containers | Yes |
| FR-09 | history.rs | /api/history/* | Yes |
| FR-10 | ui.rs | N/A (TUI only) | Yes |
| FR-11 | process.rs | /api/processes?name= | Yes |
| FR-12 | process.rs | /api/processes/kill | Yes |
| FR-13 | gpu.rs | /api/gpu | Yes |
| FR-14 | web/ | N/A | Yes |
| FR-15 | api.rs | All /api/* | Yes |
| FR-16 | metrics.rs | N/A | Yes |
| FR-17 | anomaly.rs | /api/alerts | Yes |
| FR-18 | containers.rs | /api/containers | Yes |
| FR-19 | logging.rs | N/A | Yes |
| FR-20 | affinity.rs | N/A | Yes |
| FR-21 | snapshots.rs | N/A | Yes |
| FR-22 | alerts.rs | /api/alerts | Yes |
| FR-23 | groups.rs | N/A | Yes |
| FR-24 | memmap.rs | N/A | Yes |
| FR-25 | profiles.rs | N/A | Yes |
| FR-26 | diffing.rs | N/A | Yes |
| FR-27 | containers.rs | /api/containers | Yes |
