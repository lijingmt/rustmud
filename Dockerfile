# Dockerfile for RustMUD - 1:1 Rust port of txpike9
# Multi-stage build for optimized image

# Stage 1: Builder
FROM rust:1.83-slim as builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Create dummy source to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copy actual source
COPY src ./src

# Build the application
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 rustmud

# Create application directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /usr/src/app/target/release/pikenv /app/pikenv

# Create necessary directories
RUN mkdir -p log gamenv/u gamenv/clone gamenv/single gamenv/inherit \
    gamenv/single/daemons gamenv/data && \
    chown -R rustmud:rustmud /app

# Switch to non-root user
USER rustmud

# Environment variables
ENV RUST_LOG=info
ENV GAME_AREA=tx01
ENV PORT=9999
ENV IP=0.0.0.0

# Expose MUD port
EXPOSE 9999

# Expose HTTP API port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD nc -z localhost 9999 || exit 1

# Set entrypoint
ENTRYPOINT ["/app/pikenv"]

# Default arguments
CMD ["--port", "9999"]
