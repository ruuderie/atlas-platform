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
RUN cargo install trunk && rustup target add wasm32-unknown-unknown
EXPOSE 8081
CMD ["trunk", "serve", "--port", "8081", "--address", "0.0.0.0"]

# Build stage
FROM base AS builder
# Install Trunk
RUN cargo install trunk && rustup target add wasm32-unknown-unknown
COPY . .
WORKDIR /app/apps/platform-admin
# Build the application with Trunk
RUN trunk build --release

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
RUN echo '#!/bin/sh\n\
sed -i "s|__API_BASE_URL__|${API_BASE_URL:-http://api.localhost}|g" /usr/share/nginx/html/index.html\n\
exec nginx -g "daemon off;"' > /docker-entrypoint.sh && chmod +x /docker-entrypoint.sh

EXPOSE 80
HEALTHCHECK --interval=30s --timeout=3s --retries=3 \
  CMD wget --spider http://localhost/ || exit 1
CMD ["/docker-entrypoint.sh"]
