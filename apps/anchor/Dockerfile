# Get started with a build env with Rust nightly
FROM rustlang/rust:nightly-bookworm AS builder
ARG TARGETARCH
RUN apt-get update -y && \
  apt-get install -y pkg-config make g++ libssl-dev libc++-dev build-essential curl && \
  curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && \
  apt-get install -y nodejs && \
  rm -rf /var/lib/apt/lists/*

# Install cargo-leptos precompiled binary
RUN ARCH=$(uname -m) && \
    if [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then \
      LEPTOS_URL="https://github.com/leptos-rs/cargo-leptos/releases/download/v0.3.6/cargo-leptos-aarch64-unknown-linux-gnu.tar.gz"; \
    else \
      LEPTOS_URL="https://github.com/leptos-rs/cargo-leptos/releases/download/v0.3.6/cargo-leptos-x86_64-unknown-linux-gnu.tar.gz"; \
    fi && \
    wget -O cargo-leptos.tar.gz "$LEPTOS_URL" && \
    tar -xzf cargo-leptos.tar.gz --strip-components=1 && \
    cp cargo-leptos /usr/local/cargo/bin/ && \
    rm -f cargo-leptos.tar.gz cargo-leptos README.md LICENSE

# Add the WASM target
RUN rustup target add wasm32-unknown-unknown

# Copy source code
RUN mkdir -p /app
WORKDIR /app
ARG CACHEBUST=1
ARG CACHEBUST=1
COPY . .

# Pre-compile Tailwind using NodeJS
RUN npm install -D tailwindcss@3.4
WORKDIR /app/apps/anchor
RUN npx tailwindcss -i style/main.css -o target/tmp/tailwind.css

# Rewrite Cargo.toml to use the pre-compiled stylesheet instead of triggering cargo-leptos' faulty downloader
RUN sed -i 's/tailwind-input-file = "style\/main.css"/style-file = "target\/tmp\/tailwind.css"/' Cargo.toml
RUN sed -i '/tailwind-config-file/d' Cargo.toml

# Build the app — release for prod/uat, debug for dev (much faster iteration)
ARG BUILD_PROFILE=release
RUN if [ "$BUILD_PROFILE" = "release" ]; then \
      cargo leptos build --release -vv; \
    else \
      cargo leptos build -vv; \
    fi

# Runtime Environment
FROM debian:bookworm-slim AS runner
ARG TARGETARCH# Install OpenSSL for reqwest and sqlx
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates libc-bin wget tar \
  && if [ "$TARGETARCH" = "arm64" ]; then \
       TECTONIC_URL="https://github.com/tectonic-typesetting/tectonic/releases/download/tectonic%400.15.0/tectonic-0.15.0-aarch64-unknown-linux-musl.tar.gz"; \
     else \
       TECTONIC_URL="https://github.com/tectonic-typesetting/tectonic/releases/download/tectonic%400.15.0/tectonic-0.15.0-x86_64-unknown-linux-musl.tar.gz"; \
     fi \
  && wget -O tectonic.tar.gz "$TECTONIC_URL" \
  && tar -xzf tectonic.tar.gz \
  && mv tectonic /usr/local/bin/ \
  && rm tectonic.tar.gz \
  && apt-get clean \
  && rm -f /var/lib/apt/lists/*_*

ARG BUILD_PROFILE=release
COPY --from=builder /app/apps/target/${BUILD_PROFILE}/anchor /app/
COPY --from=builder /app/apps/target/site /app/site
COPY --from=builder /app/apps/anchor/Cargo.toml /app/
COPY --from=builder /app/apps/anchor/migrations /app/migrations

WORKDIR /app

ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV LEPTOS_SITE_ROOT="site"

EXPOSE 3000

CMD ["/app/anchor"]
