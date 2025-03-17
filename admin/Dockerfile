# Base stage with common dependencies
FROM node:20-slim AS base
RUN apt-get update && apt-get install -y --no-install-recommends dumb-init curl && \
    npm install -g pnpm && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Development stage
FROM base AS development
COPY package.json pnpm-lock.yaml ./
# Make sure to install all dependencies including dev dependencies
RUN pnpm install --frozen-lockfile
# Explicitly install the missing package
RUN pnpm add -D @sveltejs/vite-plugin-svelte
COPY . .
# Development command will be specified in docker-compose.yml
EXPOSE 5150
ENV NODE_ENV=development HOST=0.0.0.0 PORT=5150
CMD ["dumb-init", "pnpm", "dev", "--host"]

# Build stage
FROM base AS build
COPY package.json pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile
# Explicitly install the missing package
RUN pnpm add -D @sveltejs/vite-plugin-svelte
COPY . .
RUN pnpm exec svelte-kit sync && \
    pnpm run build -- --empty-outdir || exit 1

# Production stage
FROM base AS production
RUN groupadd -r svelteuser && useradd -r -g svelteuser svelteuser
COPY --from=build /app/build ./build
COPY --from=build /app/package.json ./
COPY --from=build /app/pnpm-lock.yaml ./
RUN pnpm install --prod && \
    chown -R svelteuser:svelteuser /app
USER svelteuser
EXPOSE 5150
ENV NODE_ENV=production HOST=0.0.0.0 PORT=5150 VITE_SSR=false
CMD ["dumb-init", "node", "./build/index.js"]