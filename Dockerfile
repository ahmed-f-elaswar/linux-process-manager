# Linux Process Manager - Multi-stage Docker Build
# Builds React frontend and embeds it into the Rust binary

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
# Stage 2: Build Rust Backend (with embedded frontend)
# =============================================================================
FROM rust:1.83-bookworm AS backend-builder

WORKDIR /app

# Copy Cargo files and source
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

# Copy the built frontend from stage 1 (for rust-embed to include)
COPY --from=frontend-builder /app/web/dist ./web/dist

# Build the application (frontend gets embedded)
RUN cargo build --release

# =============================================================================
# Stage 3: Final Runtime Image
# =============================================================================
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    procps \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy only the binary (frontend is embedded inside)
COPY --from=backend-builder /app/target/release/process-manager ./

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
# Run (API server with web UI):
#   docker run -d -p 8080:8080 --pid=host --name procmgr linux-process-manager
#
# Run (interactive TUI):
#   docker run -it --rm --pid=host linux-process-manager ./process-manager
#
# View logs:
#   docker logs procmgr
#
# Stop:
#   docker stop procmgr && docker rm procmgr
# =============================================================================
