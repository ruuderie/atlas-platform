# Base stage with common dependencies
FROM rust:1.90.0-bookworm AS base
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Development stage
FROM base AS development
RUN cargo install cargo-leptos && rustup target add wasm32-unknown-unknown
EXPOSE 8080
CMD ["cargo", "leptos", "watch"]

# Build stage
FROM base AS builder
RUN cargo install cargo-leptos && rustup target add wasm32-unknown-unknown
COPY . .
WORKDIR /app/apps/network-instance
RUN cargo leptos build --release

# Production stage
FROM debian:bookworm-slim AS production
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* && \
    useradd -m appuser
WORKDIR /app
# Copy the compiled binary
COPY --from=builder /app/apps/target/release/atlas_network_instance /app/network-instance
# Copy the built site
COPY --from=builder /app/apps/target/site /app/site
# Set env var so leptos knows where to find the site root
ENV LEPTOS_SITE_ROOT="/app/site"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
USER appuser
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --start-period=30s --retries=3 \
  CMD curl -f http://localhost:8080/ || exit 1
CMD ["./network-instance"]
