FROM rustlang/rust:nightly-bookworm AS builder
ARG TARGETARCH
RUN apt-get update -y && \
  apt-get install -y pkg-config make g++ libssl-dev libc++-dev build-essential curl && \
  curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && \
  apt-get install -y nodejs && \
  rm -rf /var/lib/apt/lists/*

# Install cargo-leptos (same version as anchor)
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

RUN rustup target add wasm32-unknown-unknown

RUN mkdir -p /app
WORKDIR /app
ARG CACHEBUST=1
COPY . .

# Pre-compile Tailwind
RUN npm install -D tailwindcss@3.4
WORKDIR /app/apps/folio
RUN npx tailwindcss -i style/main.css -o target/tmp/tailwind.css

# Swap tailwind-input-file → style-file so cargo-leptos uses pre-built CSS
RUN sed -i 's/tailwind-input-file = "style\/main.css"/style-file = "target\/tmp\/tailwind.css"/' Cargo.toml
RUN sed -i '/tailwind-config-file/d' Cargo.toml

ARG BUILD_PROFILE=release
RUN if [ "$BUILD_PROFILE" = "release" ]; then \
      cargo leptos build --release -vv; \
    else \
      cargo leptos build -vv; \
    fi

# ── Runtime ─────────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runner
RUN apt-get update -y && \
    apt-get install -y --no-install-recommends openssl ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
ARG BUILD_PROFILE=release
COPY --from=builder /app/apps/folio/target/site ./site
COPY --from=builder /app/target/${BUILD_PROFILE}/folio ./folio

ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3100"
EXPOSE 3100
CMD ["./folio"]
