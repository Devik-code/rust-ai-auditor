# Stage 1: Build
FROM rust:1.92-bookworm as builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy configuration files
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

# Build the application in release mode
ENV CARGO_NET_RETRY=10
ENV CARGO_IO_MAX_RETRIES=10
RUN cargo build --release

# Stage 2: Lightweight execution with rustc
FROM debian:bookworm-slim

# Install necessary dependencies including rustc for auditing
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install rustc (required for code auditing)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && . ~/.cargo/env \
    && rustc --version

# Create a non-root user
RUN useradd -m -u 1000 appuser

WORKDIR /app

# Copy the compiled binary
COPY --from=builder /app/target/release/rust-ai-auditor /usr/local/bin/rust-ai-auditor

# Change ownership
RUN chown -R appuser:appuser /app

# Expose port
EXPOSE 3000

# Use the non-root user
USER appuser

# Set PATH for rustc
ENV PATH="/home/appuser/.cargo/bin:${PATH}"

# Healthcheck
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/stats || exit 1

# Start the application
CMD ["rust-ai-auditor"]
