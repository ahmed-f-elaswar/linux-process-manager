# Linux Process Manager (LPM)

A comprehensive, production-ready process manager for Linux systems with advanced monitoring, alerting, and control capabilities. Built in Rust for maximum performance and safety.

**Course**: CSCE 3401 - Operating Systems, Fall 2025

## Team Members
- Adam Aberbach (ID: 900225980)
- Mohammad Yahya Hammoudeh (ID: 900225938) 
- Mohamed Khalil Brik (ID: 900225905)
- Ahmed Elaswar (ID: 900211265)

## 📚 Documentation

**📖 [Complete Documentation](COMPLETE_DOCUMENTATION.md)** - Single comprehensive guide with everything

### Quick Links
- [Installation & Usage](COMPLETE_DOCUMENTATION.md#installation--usage)
- [Feature List](COMPLETE_DOCUMENTATION.md#complete-feature-list)
- [API Reference](COMPLETE_DOCUMENTATION.md#api-reference)
- [Configuration](COMPLETE_DOCUMENTATION.md#configuration)
- [Troubleshooting](COMPLETE_DOCUMENTATION.md#troubleshooting)

## Features

### ✅ Core Features (Priority 1) - 100% COMPLETE
- ✅ Display all running processes with PID, name, user, CPU%, memory usage  
- ✅ Kill processes with signal selection (TERM, KILL, HUP, etc.)  
- ✅ Sort by any column (CPU, memory, PID, name)  
- ✅ Filter by user, name pattern, or resource threshold  
- ✅ Tree view showing parent-child relationships  
- ✅ Real-time updates with configurable refresh rate  

### ✅ Advanced Features (Priority 2) - 100% COMPLETE
- ✅ Per-process network connection monitoring  
- ✅ Container/cgroup awareness (Docker, Kubernetes, LXC)  
- ✅ Historical data storage with SQLite backend  
- ✅ System-wide resource graphs (CPU, memory sparklines)  
- ✅ Process search with regex support  
- ✅ Batch operations on multiple processes  

### ✅ Innovative Features (Priority 3) - 100% COMPLETE
- ✅ GPU monitoring with per-process attribution (NVIDIA/AMD/Intel)  
- ✅ Web UI for remote access without SSH  
- ✅ REST API for programmatic access  
- ✅ Prometheus/InfluxDB metrics export  
- ✅ Anomaly detection using statistical analysis  
- ✅ Kubernetes pod-level aggregation  

**Overall Status**: 🎉 **100% COMPLETE** - All 18 planned features implemented!

## Highlights

### 🚀 Advanced Monitoring
- **Container Awareness**: Detects Docker, Kubernetes, and LXC containers with resource limits
- **GPU Monitoring**: Per-process GPU memory tracking for NVIDIA, AMD, and Intel GPUs
- **Network Tracking**: Network connection counting per process
- **Historical Data**: SQLite-based storage with time-series queries

### 🌐 Modern Integration
- **REST API**: Full HTTP API for programmatic access
- **Web UI**: Modern React + TypeScript frontend with Tailwind CSS
  - Real-time process monitoring with auto-refresh
  - Sortable, searchable process table
  - System stats (CPU, Memory, Swap, Uptime)
  - Process actions (send signals)
  - Clean, professional light-mode design
- **Metrics Export**: Prometheus and InfluxDB format support
- **Anomaly Detection**: Statistical analysis for CPU/memory spikes
- **Docker Support**: Multi-stage build for easy deployment

### 📊 Visualization
- **System Graphs**: Real-time CPU and memory sparkline charts
- **Tree View**: Hierarchical process relationships
- **Color-Coded Display**: Visual indicators for resource usage
- **Container & GPU Indicators**: Emoji badges for special process types

## Quick Start

### Prerequisites
- Linux operating system (kernel 4.15+)
- Rust toolchain (1.70+) OR Docker
- Optional: NVIDIA/AMD GPU drivers for GPU monitoring
- Optional: Docker/Kubernetes for container awareness

### Option 1: Docker (Recommended)

The easiest way to run the Linux Process Manager:

```bash
# Build the Docker image
docker build -t linux-process-manager .

# Run in API mode (recommended)
docker run -d -p 8080:8080 --pid=host --name procmgr linux-process-manager

# Open Web UI in browser
xdg-open http://localhost:8080

# Run in interactive TUI mode
docker run -it --rm --pid=host linux-process-manager ./process-manager

# View logs
docker logs procmgr

# Stop and remove
docker stop procmgr && docker rm procmgr
```

**Important Docker flags:**
- `--pid=host` - Required to see host system processes
- `-p 8080:8080` - Expose the web UI and API
- `-it` - Required for interactive TUI mode

### Option 2: Building from Source

```bash
git clone <repository-url>
cd process-manager
cargo build --release
```

### Running Modes

#### 1. Interactive TUI (Default)
```bash
./target/release/process-manager

# With options
./target/release/process-manager --tree        # Start in tree view
./target/release/process-manager --refresh 1   # 1-second refresh
```

#### 2. REST API Server + Web UI
```bash
# Start API server on port 8080
./target/release/process-manager --api --api-port 8080

# The Web UI is automatically served at http://localhost:8080
# Open in browser
xdg-open http://localhost:8080
```

#### 3. Metrics Export
```bash
# Export Prometheus metrics
./target/release/process-manager --export prometheus --export-file metrics.prom

# Export InfluxDB format
./target/release/process-manager --export influxdb > metrics.influx
```

### API Client Examples

Three example scripts demonstrate programmatic API access:

**1. Shell Client** (`examples/api_client.sh`) - Interactive menu-driven bash client
```bash
chmod +x examples/api_client.sh && ./examples/api_client.sh
```

**2. CSV Exporter** (`examples/api_export_csv.py`) - Export process snapshot to CSV
```bash
python3 examples/api_export_csv.py
```

**3. CPU Monitor** (`examples/api_monitor_cpu.py`) - Continuous monitoring with alerts
```bash
python3 examples/api_monitor_cpu.py
```

See [Complete Documentation](COMPLETE_DOCUMENTATION.md#api-client-examples) for detailed usage.

### Basic Usage
```bash
# Run the interactive console application
cargo run

# Or run the built binary
./target/release/process-manager
```

## Usage

### Keyboard Controls

#### Navigation
- `↑/↓` - Navigate process list
- `q` - Quit application
- `r` or `F5` - Refresh process list
- `h` or `F1` - Show/hide help

#### Sorting
- `p` - Sort by PID
- `n` - Sort by Name  
- `u` - Sort by User
- `c` - Sort by CPU usage
- `m` - Sort by Memory usage
- `s` - Sort by Start time

#### Actions
- `k` - Kill selected process (opens signal selection dialog)
- `/` - Search processes (supports regex)
- `t` - Toggle tree view
- `g` - Toggle system resource graphs (CPU/Memory sparklines)
- `o` - Toggle user processes only filter

#### Process Control (Kill Dialog)
- `t` - SIGTERM (15) - Graceful termination
- `9` - SIGKILL (9) - Force kill
- `1` - SIGHUP (1) - Hang up
- `2` - SIGINT (2) - Interrupt  
- `s` - SIGSTOP (19) - Stop process
- `c` - SIGCONT (18) - Continue process
- `Enter` - Confirm action
- `Esc` - Cancel

### Interface Sections

1. **System Information Bar** (Top)
   - CPU count and load averages (1, 5, 15 minutes)
   - Memory usage percentage and totals
   - Swap usage percentage
   - System uptime

2. **Process Table** (Main Area)
   - PID, User, CPU%, Memory%, Memory (KB), Name, Command
   - Sortable columns with visual indicators
   - Color-coded highlighting for selected process

3. **Status Bar** (Bottom)
   - Current operation status
   - Quick help reminders
   - Filter/search status

### Search and Filtering

#### Search (`/` key)
- Supports regular expressions
- Searches both process name and command line
- Case-insensitive matching
- Clear search by entering empty pattern

#### Filters
- `o` - Toggle user processes only (hides system processes)
- More advanced filtering planned for future releases

### Command Line Options

```bash
process-manager [OPTIONS]

Options:
  -r, --refresh <SECONDS>        Sets refresh interval in seconds
  -u, --user <USERNAME>          Filter processes by user
  -t, --tree                     Start in tree view mode
      --api                      Start REST API server mode
      --api-port <PORT>          API server port (default: 8080)
      --export <FORMAT>          Export metrics (prometheus|influxdb)
      --export-file <FILE>       Export metrics to file
      --history-db <PATH>        Path to history database
  -h, --help                     Print help information
  -V, --version                  Print version information
```

## Architecture

### Core Components

1. **Process Manager** (`src/process.rs`)
   - Interfaces with Linux `/proc` filesystem
   - Uses `sysinfo` crate for cross-platform compatibility
   - Handles process enumeration, monitoring, and control

2. **Terminal UI** (`src/ui.rs`)
   - Built with `ratatui` for rich terminal interfaces
   - Real-time updates with configurable refresh rates
   - Interactive keyboard-driven interface

3. **Tree View** (`src/tree.rs`)
   - Hierarchical process visualization
   - Parent-child relationship mapping
   - Expandable/collapsible tree nodes

### Technology Stack

- **Language**: Rust (memory-safe systems programming)
- **UI Framework**: ratatui (terminal user interface)
- **Process Info**: sysinfo (cross-platform system information)
- **Terminal Handling**: crossterm (cross-platform terminal manipulation)
- **User Management**: users crate (Unix user/group information)

## Safety and Security

### Built-in Safety Features
- Process ownership verification before signal sending
- Confirmation dialogs for destructive operations
- Graceful signal handling (SIGTERM before SIGKILL)
- Input validation and error handling

### Security Considerations
- Runs with user privileges (no setuid required)
- Respects process ownership boundaries
- Audit trail through system logs
- No password exposure in process arguments

## Performance

### Optimizations
- Efficient `/proc` filesystem parsing
- Minimal memory allocations in hot paths
- Configurable refresh rates to balance responsiveness and CPU usage
- Native Rust performance characteristics

### Resource Usage
- Low CPU overhead (~1-2% on typical systems)
- Minimal memory footprint (~5-10MB)
- No disk I/O except for `/proc` reads
- Scales well with process count

## Error Handling

The application includes comprehensive error handling for:
- Permission denied scenarios
- Invalid process IDs (PID reuse)
- Network connectivity issues (future features)
- Filesystem access problems
- Signal delivery failures

## Documentation

### 📚 Complete Documentation Set

- **[User Guide](docs/USER_GUIDE.md)** - Complete user manual with examples
- **[Installation Guide](docs/INSTALLATION.md)** - Detailed installation instructions  
- **[Developer Guide](docs/DEVELOPER_GUIDE.md)** - Architecture and development guidelines
- **[API Documentation](docs/API.md)** - REST API reference and code examples
- **[Implementation Status](IMPLEMENTATION_STATUS.md)** - Feature completion matrix
- **[New Features Guide](NEW_FEATURES.md)** - Phase II features documentation
- **[Change Log](CHANGELOG.md)** - Version history and updates

## Comparison with Existing Tools

| Feature | LPM | htop | top | ps |
|---------|-----|------|-----|----| 
| Interactive UI | ✅ | ✅ | ✅ | ❌ |
| Real-time Updates | ✅ | ✅ | ✅ | ❌ |
| Tree View | ✅ | ✅ | ❌ | ❌ |
| Search/Filter | ✅ | ✅ | ❌ | ❌ |
| Signal Selection | ✅ | ✅ | ✅ | ❌ |
| Resource Graphs | ✅ | ❌ | ❌ | ❌ |
| Container Awareness | ✅ | ❌ | ❌ | ❌ |
| GPU Monitoring | ✅ | ❌ | ❌ | ❌ |
| Historical Data | ✅ | ❌ | ❌ | ❌ |
| REST API | ✅ | ❌ | ❌ | ❌ |
| Web UI | ✅ | ❌ | ❌ | ❌ |
| Metrics Export | ✅ | ❌ | ❌ | ❌ |
| Anomaly Detection | ✅ | ❌ | ❌ | ❌ |

## Development

### Project Structure
```
process-manager/
├── src/
│   ├── main.rs           # Application entry point
│   ├── process.rs        # Process management core
│   ├── ui.rs             # Terminal user interface
│   ├── tree.rs           # Tree view functionality
│   ├── api.rs            # REST API server
│   ├── gpu.rs            # GPU monitoring
│   ├── network.rs        # Network & container awareness
│   ├── history.rs        # Historical data storage
│   ├── metrics.rs        # Prometheus/InfluxDB export
│   ├── anomaly.rs        # Anomaly detection
│   ├── logging.rs        # Structured logging
│   ├── affinity.rs       # CPU affinity & priority
│   ├── alerts.rs         # Smart alerts system
│   ├── snapshots.rs      # Process snapshots
│   ├── groups.rs         # Process groups (PGID/SID)
│   ├── memmap.rs         # Memory map visualization
│   ├── profiles.rs       # Saved view profiles
│   ├── diffing.rs        # Process state diffing
│   ├── containers.rs     # Container detection
│   └── config.rs         # Configuration management
├── benches/
│   └── benchmarks.rs     # Performance benchmarks
├── benches/
│   └── benchmarks.rs     # Performance benchmarks (18 tests)
├── tests/
│   ├── integration_test.rs      # Basic integration tests (10 tests)
│   └── integration_tests.rs     # Feature tests (25 tests)
├── examples/
│   ├── api_client.sh            # Shell API client
│   ├── api_export_csv.py        # CSV exporter
│   └── api_monitor_cpu.py       # CPU monitor
├── web/                  # React + TypeScript Web UI
│   ├── src/
│   │   ├── components/   # React components (Header, ProcessTable, SystemStats, etc.)
│   │   ├── hooks/        # Custom React hooks (useProcesses, useSystemInfo)
│   │   ├── api.ts        # API client
│   │   ├── types.ts      # TypeScript types
│   │   ├── App.tsx       # Main application component
│   │   └── index.css     # Tailwind CSS styles
│   ├── package.json      # NPM dependencies
│   ├── vite.config.ts    # Vite build config
│   ├── tailwind.config.js # Tailwind CSS config
│   └── tsconfig.json     # TypeScript config
├── requirements/         # Project requirements documentation
│   ├── README.md         # Requirements overview
│   ├── functional-requirements.md
│   ├── non-functional-requirements.md
│   ├── architecture.md
│   ├── test-plan.md
│   └── grading-rubric.md
├── Dockerfile            # Multi-stage Docker build
├── test.sh               # Quick validation script
├── demo.sh               # Feature showcase script
├── config.example.toml   # Example configuration
├── Cargo.toml            # Rust dependencies
└── README.md             # This file
```

### Testing & Scripts

**Run Tests**:
```bash
cargo test                    # All 121 tests
bash test.sh                  # Quick validation
cargo +nightly bench --no-run # Compile benchmarks
```

**Utility Scripts**:
```bash
bash demo.sh                  # Project demonstration
bash test.sh                  # Basic functionality tests
```

**Configuration**:
```bash
# Copy example config
cp config.example.toml ~/.config/process-manager/config.toml

# Use custom config
./process-manager --config /path/to/config.toml
```

See [Complete Documentation](COMPLETE_DOCUMENTATION.md#testing--quality-assurance) for detailed test documentation.

### Building for Release
```bash
cargo build --release
strip target/release/process-manager  # Optional: reduce binary size
```

### Running Tests
```bash
cargo test  # All 121 tests pass ✅
```

### Code Quality
```bash
cargo clippy  # Linting
cargo fmt     # Formatting
cargo build   # Clean build with no warnings ✅
```

## Contributing

1. Follow Rust coding conventions
2. Add tests for new functionality
3. Update documentation
4. Ensure compatibility with Linux systems

## License

This project is developed as part of academic coursework for CSCE 3401 Operating Systems Fall 2025.

## Troubleshooting

### Common Issues

**Permission Denied when killing processes**
- You can only kill processes owned by your user account
- Use `sudo` if you need to manage system processes (not recommended for normal use)

**High CPU usage**
- Increase refresh interval with `-r` flag
- Close other monitoring tools that may conflict

**Display issues**
- Ensure terminal supports ANSI colors
- Try different terminal emulators if rendering is incorrect
- Minimum terminal size: 80x24 characters

### Getting Help

1. Press `h` or `F1` in the application for keyboard shortcuts
2. Check the status bar for current operation hints
3. Review command line options with `--help`

## Web UI

The project includes a modern React + TypeScript web interface built with:

- **React 18** with TypeScript for type-safe development
- **Vite** for fast development and optimized builds
- **Tailwind CSS** for responsive, modern styling
- **TanStack Query** for efficient data fetching and caching
- **Lucide React** for consistent iconography

### Web UI Features
- Real-time process monitoring with configurable auto-refresh (1-30 seconds)
- Sortable columns (PID, Name, User, CPU%, Memory)
- Full-text search across process name, command, user, and PID
- System overview cards (CPU cores, Load Average, Memory, Swap, Uptime)
- Process actions with signal selection (SIGTERM, SIGKILL, SIGHUP, etc.)
- Status badges for process states (Running, Sleeping, Zombie, etc.)
- Connection status indicator
- Professional light-mode design

### Building the Web UI
```bash
cd web
npm install
npm run build    # Production build to dist/
npm run dev      # Development server
```

The production build is automatically served by the Rust API server at http://localhost:8080

## Acknowledgments

- Linux kernel developers for the `/proc` filesystem design
- Rust community for excellent systems programming tools
- ratatui developers for the terminal UI framework
- Course instructor and teaching assistants for project guidance