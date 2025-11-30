# Linux Process Manager - Multi-stage Docker Build
# Builds both React frontend and Rust backend in a single container

# =============================================================================
# Stage 1: Build React Frontend
# =============================================================================
FROM node:20-alpine AS frontend-builder

WORKDIR /app/web

# Copy package files first for better caching
COPY web/package*.json ./

# Install dependencies
RUN npm ci --silent

# Copy source files
COPY web/ ./

# Build production bundle
RUN npm run build

# =============================================================================
# Stage 2: Build Rust Backend
# =============================================================================
FROM rust:1.75-bookworm AS backend-builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Cargo files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy src to build dependencies first
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub fn dummy() {}" > src/lib.rs

# Build dependencies only
RUN cargo build --release && rm -rf src

# Copy real source code
COPY src/ ./src/

# Touch main.rs to force rebuild
RUN touch src/main.rs

# Build the actual application
RUN cargo build --release

# =============================================================================
# Stage 3: Final Runtime Image
# =============================================================================
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    procps \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN useradd -m -s /bin/bash procmgr

WORKDIR /app

# Copy the built binary
COPY --from=backend-builder /app/target/release/process-manager ./

# Copy the built frontend
COPY --from=frontend-builder /app/web/dist ./web/dist

# Copy configuration template
COPY config.example.toml ./config.toml

# Set ownership
RUN chown -R procmgr:procmgr /app

# Note: We run as root to access /proc for all processes
# In production, consider using capabilities instead
# USER procmgr

# Expose API port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/health || exit 1

# Default command: start API server
CMD ["./process-manager", "--api", "--api-port", "8080"]

# =============================================================================
# Usage:
# =============================================================================
# Build:
#   docker build -t linux-process-manager .
#
# Run (interactive TUI - requires terminal):
#   docker run -it --rm --pid=host linux-process-manager ./process-manager
#
# Run (API server):
#   docker run -d -p 8080:8080 --pid=host --name procmgr linux-process-manager
#
# Run with host process visibility:
#   docker run -d -p 8080:8080 --pid=host --privileged linux-process-manager
#
# View logs:
#   docker logs procmgr
#
# Stop:
#   docker stop procmgr && docker rm procmgr
# =============================================================================
