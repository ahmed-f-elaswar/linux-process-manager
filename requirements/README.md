# Linux Process Manager - Requirements Documentation

## Overview

This folder contains the formal requirements documentation for the Linux Process Manager project, a comprehensive system monitoring tool developed for CSCE 3401 Operating Systems (Fall 2025).

## Document Index

| Document | Description |
|----------|-------------|
| [functional-requirements.md](./functional-requirements.md) | All 27 implemented features with detailed specifications |
| [non-functional-requirements.md](./non-functional-requirements.md) | Performance, security, usability, and reliability specs |
| [architecture.md](./architecture.md) | System architecture with diagrams and design decisions |
| [test-plan.md](./test-plan.md) | Testing strategy, test cases, and results |
| [grading-rubric.md](./grading-rubric.md) | Phase II grading criteria and self-assessment |

## Project Summary

The Linux Process Manager is a **production-ready** system monitoring application that provides:

- **Terminal UI (TUI)**: Full-featured interactive interface with real-time updates
- **REST API**: Programmatic access to all monitoring capabilities
- **Web UI**: Modern React-based dashboard for remote monitoring

### Key Statistics

| Metric | Value |
|--------|-------|
| Lines of Code | 8,007+ (Rust) |
| Modules | 20 specialized components |
| Features | 27 implemented |
| Tests | 121 passing |
| Benchmarks | 18 performance tests |
| Code Quality | Zero compiler warnings |

### Technology Stack

- **Backend**: Rust 2021 edition
- **TUI Framework**: ratatui with crossterm
- **Web Framework**: actix-web 4.0
- **Database**: SQLite (rusqlite)
- **Frontend**: React + TypeScript + Tailwind CSS
- **Containerization**: Docker (single-container deployment)

## Contributors

| Name | ID | Role |
|------|-----|------|
| Adam Aberbach | 900225980 | Team Member |
| Mohammad Yahya Hammoudeh | 900225938 | Team Member |
| Mohamed Khalil Brik | 900225905 | Team Member |
| Ahmed Elaswar | 900211265 | Team Member |

## Course Information

- **Course**: CSCE 3401 - Operating Systems
- **Term**: Fall 2025
- **Institution**: The American University in Cairo
