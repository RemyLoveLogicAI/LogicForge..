# Genesis Engine - Production Dockerfile
# Multi-stage build for optimal image size

# Build stage
FROM rust:1.75-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy source to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub fn lib() {}" > src/lib.rs

# Build dependencies (this layer is cached)
RUN cargo build --release && \
    rm -rf src target/release/deps/genesis*

# Copy actual source code
COPY src ./src
COPY config ./config
COPY templates ./templates

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -s /bin/bash genesis

# Copy binary from builder
COPY --from=builder /app/target/release/genesis /usr/local/bin/genesis

# Copy configuration and templates
COPY --from=builder /app/config /etc/genesis/config
COPY --from=builder /app/templates /etc/genesis/templates

# Create data directory
RUN mkdir -p /var/lib/genesis && \
    chown -R genesis:genesis /var/lib/genesis /etc/genesis

# Set environment variables
ENV GENESIS_CONFIG_DIR=/etc/genesis/config
ENV GENESIS_DATA_DIR=/var/lib/genesis
ENV GENESIS_TEMPLATE_DIR=/etc/genesis/templates
ENV RUST_LOG=info

# Switch to non-root user
USER genesis

# Create volume for persistent data
VOLUME ["/var/lib/genesis"]

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD genesis health || exit 1

# Default command - run in monitor mode
ENTRYPOINT ["genesis"]
CMD ["monitor", "--watch"]
