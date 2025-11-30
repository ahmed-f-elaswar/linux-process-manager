# Non-Functional Requirements Specification

## Overview

This document specifies the non-functional requirements (NFRs) for the Linux Process Manager, covering performance, security, usability, reliability, and other quality attributes.

---

## NFR-01: Performance

### NFR-01.1: CPU Overhead

**Requirement**: The application must have minimal CPU overhead during normal operation.

| Metric | Target | Actual |
|--------|--------|--------|
| Idle CPU usage | < 2% | 1-2% |
| Active refresh CPU | < 5% | 2-3% |
| Peak CPU (sorting 1000+ processes) | < 10% | 5-8% |

**Verification**: Measured using `top` and benchmarks in `benches/benchmarks.rs`

### NFR-01.2: Memory Usage

**Requirement**: The application must have a small memory footprint.

| Metric | Target | Actual |
|--------|--------|--------|
| Base memory usage | < 20 MB | 5-10 MB |
| With 1000 processes | < 50 MB | 15-25 MB |
| With history enabled | < 100 MB | 30-50 MB |

**Verification**: Measured using `ps` and `/proc/self/status`

### NFR-01.3: Refresh Latency

**Requirement**: Process list refresh must be fast.

| Metric | Target | Actual |
|--------|--------|--------|
| Full refresh (500 processes) | < 100ms | 50-80ms |
| Full refresh (1000 processes) | < 200ms | 100-150ms |
| Incremental refresh | < 50ms | 20-40ms |

**Verification**: Benchmarks in `benches/benchmarks.rs`

### NFR-01.4: Startup Time

**Requirement**: Application must start quickly.

| Metric | Target | Actual |
|--------|--------|--------|
| Cold start (TUI) | < 1s | 0.3-0.5s |
| Cold start (API server) | < 2s | 0.5-1s |

---

## NFR-02: Scalability

### NFR-02.1: Process Count

**Requirement**: The application must handle large numbers of processes.

| Metric | Target | Actual |
|--------|--------|--------|
| Minimum supported | 1000 processes | Tested |
| Maximum tested | 5000 processes | Tested |
| Performance degradation | Linear | Confirmed |

### NFR-02.2: Historical Data

**Requirement**: Historical data storage must scale with retention period.

| Metric | Target | Actual |
|--------|--------|--------|
| 7-day retention | < 100 MB | ~50 MB |
| 30-day retention | < 500 MB | ~200 MB |
| Query time (1 process, 30 days) | < 1s | 100-300ms |

---

## NFR-03: Security

### NFR-03.1: Privilege Separation

**Requirement**: The application must work with minimal privileges.

**Implementation**:
- Runs as normal user by default
- Only requests elevated privileges for specific operations (kill, nice)
- Never stores credentials
- No SUID/SGID requirements

### NFR-03.2: Input Validation

**Requirement**: All user input must be validated.

**Implementation**:
- PID validation (positive integers only)
- Signal validation (known signals only)
- Path validation (no path traversal)
- Regex validation (no ReDoS vulnerabilities)

### NFR-03.3: API Security

**Requirement**: REST API must be secure.

**Implementation**:
- CORS configuration (configurable allowed origins)
- No sensitive data exposure in API responses
- Rate limiting consideration (future enhancement)
- No authentication by default (local use only)

### NFR-03.4: Safe Signal Handling

**Requirement**: Signal operations must be safe.

**Implementation**:
- Confirmation required before kill operations
- Cannot kill init (PID 1) or kernel threads
- Permission errors handled gracefully
- Logging of all signal operations

---

## NFR-04: Reliability

### NFR-04.1: Error Handling

**Requirement**: All errors must be handled gracefully.

**Implementation**:
- No panics in production code
- All `Result` types properly handled
- User-friendly error messages
- Errors logged with context

### NFR-04.2: Crash Recovery

**Requirement**: Application must handle unexpected failures.

**Implementation**:
- SQLite database with WAL mode (write-ahead logging)
- Graceful shutdown on SIGTERM/SIGINT
- No data corruption on crash
- Auto-recovery of database on restart

### NFR-04.3: Resource Cleanup

**Requirement**: Resources must be properly released.

**Implementation**:
- RAII patterns for all resources
- File descriptors properly closed
- Memory freed when no longer needed
- No resource leaks

---

## NFR-05: Usability

### NFR-05.1: Keyboard Navigation

**Requirement**: TUI must be fully keyboard-navigable.

**Implementation**:
- All features accessible via keyboard
- Consistent key bindings
- Help screen with all shortcuts
- Vim-style navigation (j/k)

### NFR-05.2: Visual Feedback

**Requirement**: User actions must have immediate feedback.

**Implementation**:
- Loading indicators during operations
- Success/error messages
- Visual confirmation for destructive actions
- Status bar with current state

### NFR-05.3: Accessibility

**Requirement**: Interface must be accessible.

**Implementation**:
- High contrast default theme
- No color-only indicators
- Screen reader friendly (where possible)
- Configurable colors

### NFR-05.4: Documentation

**Requirement**: Comprehensive documentation must be provided.

**Implementation**:
- In-app help (h key)
- README with quick start
- Complete API documentation
- Example scripts provided

---

## NFR-06: Portability

### NFR-06.1: Linux Distribution Support

**Requirement**: Application must work on major Linux distributions.

| Distribution | Status | Tested Version |
|--------------|--------|----------------|
| Ubuntu | Supported | 22.04, 24.04 |
| Fedora | Supported | 39, 40 |
| Debian | Supported | 12 |
| Arch Linux | Supported | Rolling |
| RHEL/CentOS | Supported | 8, 9 |
| Alpine | Supported | 3.19 |

### NFR-06.2: Kernel Version

**Requirement**: Support modern Linux kernels.

| Metric | Requirement |
|--------|-------------|
| Minimum kernel | 4.15+ |
| Recommended kernel | 5.0+ |
| Tested kernels | 5.15, 6.1, 6.5, 6.8 |

### NFR-06.3: Terminal Compatibility

**Requirement**: TUI must work in various terminal emulators.

| Terminal | Status |
|----------|--------|
| GNOME Terminal | Full support |
| Konsole | Full support |
| xterm | Full support |
| Alacritty | Full support |
| kitty | Full support |
| tmux | Full support |
| SSH sessions | Full support |

---

## NFR-07: Maintainability

### NFR-07.1: Code Quality

**Requirement**: Code must be maintainable and readable.

**Metrics**:
| Metric | Target | Actual |
|--------|--------|--------|
| Compiler warnings | 0 | 0 |
| Clippy warnings | 0 | 0 |
| Code formatting | Consistent | rustfmt applied |
| Test coverage | > 70% | ~80% |

### NFR-07.2: Modularity

**Requirement**: Code must be well-organized.

**Implementation**:
- 20 specialized modules
- Clear separation of concerns
- Single responsibility principle
- Minimal coupling between modules

### NFR-07.3: Documentation

**Requirement**: Code must be documented.

**Implementation**:
- Doc comments on public APIs
- Inline comments for complex logic
- Architecture documentation
- API reference

---

## NFR-08: Testability

### NFR-08.1: Unit Tests

**Requirement**: Core functionality must have unit tests.

**Implementation**:
- 43 unit tests in library
- 43 unit tests in binary
- Test utilities and helpers

### NFR-08.2: Integration Tests

**Requirement**: Component interactions must be tested.

**Implementation**:
- 10 basic integration tests
- 25 feature integration tests
- API endpoint tests

### NFR-08.3: Benchmarks

**Requirement**: Performance must be measurable.

**Implementation**:
- 18 performance benchmarks
- Covers critical paths
- Reproducible results

---

## NFR-09: Configuration

### NFR-09.1: Configuration File

**Requirement**: Application must be configurable via file.

**Implementation**:
- TOML configuration format
- Example config provided
- All settings optional (sensible defaults)
- Hot-reload not required

### NFR-09.2: Command-Line Options

**Requirement**: Common options available via CLI.

**Implementation**:
- `--refresh` - Set refresh interval
- `--user` - Filter by user
- `--tree` - Start in tree view
- `--api` - Start API server
- `--api-port` - API server port
- `--export` - Export metrics format
- `--history-db` - Database path

### NFR-09.3: Environment Variables

**Requirement**: Support configuration via environment.

**Implementation**:
- `PROCESS_MANAGER_CONFIG` - Config file path
- `PROCESS_MANAGER_LOG_LEVEL` - Log level
- Standard Rust environment variables

---

## NFR-10: Deployment

### NFR-10.1: Installation

**Requirement**: Easy installation process.

**Methods**:
1. Build from source (`cargo build --release`)
2. Docker container
3. Pre-built binary (future)

### NFR-10.2: Dependencies

**Requirement**: Minimal runtime dependencies.

**Runtime Dependencies**:
- glibc (standard on all Linux)
- libgcc (for Rust runtime)
- Optional: nvidia-smi (for NVIDIA GPU support)
- Optional: rocm-smi (for AMD GPU support)

### NFR-10.3: Docker

**Requirement**: Docker container support.

**Implementation**:
- Multi-stage Dockerfile
- Alpine-based runtime image
- Single container for all components
- Configurable via environment

---

## Compliance Matrix

| Requirement | Priority | Status | Verification |
|-------------|----------|--------|--------------|
| NFR-01.1 CPU Overhead | High | Met | Benchmark |
| NFR-01.2 Memory Usage | High | Met | Measurement |
| NFR-01.3 Refresh Latency | High | Met | Benchmark |
| NFR-01.4 Startup Time | Medium | Met | Measurement |
| NFR-02.1 Process Count | High | Met | Load test |
| NFR-02.2 Historical Data | Medium | Met | Measurement |
| NFR-03.1 Privilege Separation | High | Met | Code review |
| NFR-03.2 Input Validation | High | Met | Code review |
| NFR-03.3 API Security | Medium | Met | Testing |
| NFR-03.4 Safe Signal Handling | High | Met | Testing |
| NFR-04.1 Error Handling | High | Met | Testing |
| NFR-04.2 Crash Recovery | Medium | Met | Testing |
| NFR-04.3 Resource Cleanup | High | Met | Code review |
| NFR-05.1 Keyboard Navigation | High | Met | Manual test |
| NFR-05.2 Visual Feedback | Medium | Met | Manual test |
| NFR-05.3 Accessibility | Low | Partial | Manual test |
| NFR-05.4 Documentation | Medium | Met | Review |
| NFR-06.1 Distribution Support | High | Met | Testing |
| NFR-06.2 Kernel Version | High | Met | Testing |
| NFR-06.3 Terminal Compatibility | High | Met | Testing |
| NFR-07.1 Code Quality | High | Met | CI/Linting |
| NFR-07.2 Modularity | Medium | Met | Code review |
| NFR-07.3 Documentation | Medium | Met | Review |
| NFR-08.1 Unit Tests | High | Met | CI |
| NFR-08.2 Integration Tests | Medium | Met | CI |
| NFR-08.3 Benchmarks | Medium | Met | CI |
| NFR-09.1 Configuration File | Medium | Met | Testing |
| NFR-09.2 Command-Line Options | High | Met | Testing |
| NFR-09.3 Environment Variables | Low | Met | Testing |
| NFR-10.1 Installation | High | Met | Testing |
| NFR-10.2 Dependencies | High | Met | Testing |
| NFR-10.3 Docker | High | Met | Testing |
