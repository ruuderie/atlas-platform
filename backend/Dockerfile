# Base stage with common dependencies
FROM rust:1.84.1 AS base
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Development stage
FROM base AS development
EXPOSE 8000
ENV RUST_LOG=debug
CMD ["cargo", "run"]

# Build stage
FROM base AS builder
COPY Cargo.toml Cargo.lock ./
# Create a dummy main.rs to cache dependencies
RUN mkdir -p src && \
    echo "fn main() {println!(\"Dummy main\");}" > src/main.rs && \
    cargo build --release && \
    rm -rf src
# Copy the actual source code
COPY . .
# Build the application with nightly features
RUN cargo build --release

# Production stage
FROM debian:bookworm-slim AS production
RUN apt-get update && apt-get install -y \
    libssl3 \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* && \
    useradd -m appuser
WORKDIR /app
COPY --from=builder /app/target/release/business_directory_backend .
USER appuser
EXPOSE 8000
ENV RUST_LOG=info
HEALTHCHECK --interval=30s --timeout=3s --start-period=30s --retries=3 \
  CMD curl -f http://localhost:8000/health || exit 1
CMD ["./business_directory_backend"]