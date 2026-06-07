# Base stage with common dependencies
FROM rust:1.93.0-bookworm AS base
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
ARG BUILD_PROFILE=release
# Copy workspace manifests for dependency caching
COPY Cargo.toml Cargo.lock ./
COPY crates/atlas-compute-sdk/Cargo.toml ./crates/atlas-compute-sdk/Cargo.toml
# Stub sources for every workspace member so `cargo build` can resolve the
# workspace and pre-compile all external dependencies before copying real source.
#   - atlas_backend  → src/main.rs  (binary)
#   - atlas-compute-sdk → crates/atlas-compute-sdk/src/lib.rs  (library)
RUN mkdir -p src crates/atlas-compute-sdk/src && \
    echo "fn main() {println!(\"Dummy main\");}" > src/main.rs && \
    echo "// stub" > crates/atlas-compute-sdk/src/lib.rs && \
    if [ "$BUILD_PROFILE" = "release" ]; then cargo build --release; else cargo build; fi && \
    rm -rf src crates/atlas-compute-sdk/src
# Copy the actual source code
COPY . .
# Declare build args for versioning
ARG ATLAS_BUILD_SHA
ARG ATLAS_BUILD_DATE
ENV ATLAS_BUILD_SHA=$ATLAS_BUILD_SHA
ENV ATLAS_BUILD_DATE=$ATLAS_BUILD_DATE
# Build the application
RUN if [ "$BUILD_PROFILE" = "release" ]; then \
      cargo build --release; \
    else \
      cargo build; \
    fi

# Production stage
FROM debian:bookworm-slim AS production
RUN apt-get update && apt-get install -y \
    libssl3 \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* && \
    useradd -m appuser
WORKDIR /app

ARG BUILD_PROFILE=release

# Main application
COPY --from=builder /app/target/${BUILD_PROFILE}/atlas_backend .

# Additional admin / migration / tooling binaries (available in dev/uat pods)
# These are useful for one-off data migrations and operational tasks without
# needing a separate image or breaking the Docker/CI process.
COPY --from=builder /app/target/${BUILD_PROFILE}/migrate_legacy_crm ./bin/
COPY --from=builder /app/target/${BUILD_PROFILE}/migrate ./bin/
COPY --from=builder /app/target/${BUILD_PROFILE}/seed_db ./bin/
COPY --from=builder /app/target/${BUILD_PROFILE}/migrate_anchor_users ./bin/

USER appuser
EXPOSE 8000
ENV RUST_LOG=info
HEALTHCHECK --interval=30s --timeout=3s --start-period=30s --retries=3 \
  CMD curl -f http://localhost:8000/health || exit 1
CMD ["./atlas_backend"]