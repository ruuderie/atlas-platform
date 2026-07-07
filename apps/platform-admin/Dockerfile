# Base stage with common dependencies
FROM rust:1.95.0-bookworm AS base
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install Trunk precompiled binary and add the WASM target
RUN ARCH=$(uname -m) && \
    if [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then \
      TRUNK_URL="https://github.com/trunk-rs/trunk/releases/download/v0.21.14/trunk-aarch64-unknown-linux-gnu.tar.gz"; \
    else \
      TRUNK_URL="https://github.com/trunk-rs/trunk/releases/download/v0.21.14/trunk-x86_64-unknown-linux-gnu.tar.gz"; \
    fi && \
    curl -fsSL "$TRUNK_URL" | tar -xzf - -C /usr/local/bin && \
    chmod +x /usr/local/bin/trunk && \
    rustup target add wasm32-unknown-unknown

WORKDIR /app

# Development stage
FROM base AS development
EXPOSE 8081
CMD ["trunk", "serve", "--port", "8081", "--address", "0.0.0.0"]

# Build stage
FROM base AS builder
COPY . .
WORKDIR /app/apps/platform-admin
# Build the application with Trunk
ARG BUILD_PROFILE=release
RUN if [ "$BUILD_PROFILE" = "release" ]; then \
      trunk build --release; \
    else \
      trunk build; \
    fi

# Production stage
FROM nginx:alpine AS production
COPY --from=builder /app/apps/platform-admin/dist /usr/share/nginx/html
# Custom Nginx configuration to support client-side routing
RUN echo 'server { \
    listen 80; \
    location / { \
        root /usr/share/nginx/html; \
        index index.html index.htm; \
        try_files $uri $uri/ /index.html; \
    } \
}' > /etc/nginx/conf.d/default.conf

# Entrypoint script to inject runtime environment variables into the static index.html
COPY apps/platform-admin/docker-entrypoint.sh /docker-entrypoint.sh
RUN chmod +x /docker-entrypoint.sh

EXPOSE 80
HEALTHCHECK --interval=30s --timeout=3s --retries=3 \
  CMD wget --spider http://localhost/ || exit 1
CMD ["/docker-entrypoint.sh"]
