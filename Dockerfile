# Builder stage
FROM rust:1.82-slim-bookworm as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml .
# Create dummy src/main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm src/main.rs

# Copy source code
COPY src src
# Touch main.rs to force rebuild
RUN touch src/main.rs
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies (semgrep requires python/pip usually, but let's check. 
# The Rust wrapper calls `semgrep` binary. So we need semgrep installed in the container.)
RUN apt-get update && apt-get install -y python3 python3-pip python3-venv git curl && \
    python3 -m venv /opt/venv && \
    /opt/venv/bin/pip install semgrep && \
    ln -s /opt/venv/bin/semgrep /usr/local/bin/semgrep && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/sidero /usr/local/bin/sidero

# Set environment variables
ENV SEMGREP_APP_TOKEN=""

# Run the server
CMD ["sidero"]
