# Base stage with common dependencies
FROM rust:1.95.0-bookworm AS base
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install cargo-leptos precompiled binary
RUN ARCH=$(uname -m) && \
    if [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then \
      LEPTOS_URL="https://github.com/leptos-rs/cargo-leptos/releases/download/v0.3.6/cargo-leptos-aarch64-unknown-linux-gnu.tar.gz"; \
    else \
      LEPTOS_URL="https://github.com/leptos-rs/cargo-leptos/releases/download/v0.3.6/cargo-leptos-x86_64-unknown-linux-gnu.tar.gz"; \
    fi && \
    curl -L -o cargo-leptos.tar.gz "$LEPTOS_URL" && \
    tar -xzf cargo-leptos.tar.gz --strip-components=1 && \
    cp cargo-leptos /usr/local/cargo/bin/ && \
    rm -f cargo-leptos.tar.gz cargo-leptos README.md LICENSE

WORKDIR /app

# Development stage
FROM base AS development
RUN rustup target add wasm32-unknown-unknown
EXPOSE 8080
CMD ["cargo", "leptos", "watch"]

# Build stage
FROM base AS builder
RUN rustup target add wasm32-unknown-unknown
COPY . .
WORKDIR /app/apps/network-instance
ARG BUILD_PROFILE=release
RUN if [ "$BUILD_PROFILE" = "release" ]; then \
      cargo leptos build --release; \
    else \
      cargo leptos build; \
    fi

# Production stage
FROM debian:bookworm-slim AS production
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* && \
    useradd -m appuser
WORKDIR /app
# Copy the compiled binary
ARG BUILD_PROFILE=release
COPY --from=builder /app/apps/target/${BUILD_PROFILE}/atlas_network_instance /app/network-instance
# Copy the built site
COPY --from=builder /app/apps/target/site /app/site
# Set env var so leptos knows where to find the site root
ENV LEPTOS_SITE_ROOT="/app/site"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"

# ARCHITECTURE REQUIREMENT:
# We use `hash-files = true` in Cargo.toml to generate cryptographically hashed
# WASM/JS binaries during the build stage to permanently bust Cloudflare edge caching.
# HOWEVER, cargo-leptos has a massive footgun: the SSR server will ONLY inject the 
# <script> tag referencing those hashed files if this environment variable is explicitly
# set at runtime. If you remove this, the server will boot normally but will serve
# 100% dead, static HTML with no interactivity (hydration panic).
ENV LEPTOS_HASH_FILES="true"
USER appuser
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --start-period=30s --retries=3 \
  CMD curl -f http://localhost:8080/ || exit 1
CMD ["./network-instance"]
