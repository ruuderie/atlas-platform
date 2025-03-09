# Build stage
FROM node:20-slim AS build

RUN apt-get update && apt-get install -y --no-install-recommends dumb-init && \
    npm install -g pnpm

WORKDIR /app
COPY package.json pnpm-lock.yaml ./
RUN pnpm install
COPY . .
RUN pnpm exec svelte-kit sync && \
    pnpm run build -- --empty-outdir || exit 1
RUN ls -la /app && ls -la /app/build || echo "Build directory is empty or missing"

# Runtime stage
FROM node:20-slim

RUN apt-get update && apt-get install -y --no-install-recommends dumb-init && \
    groupadd -r svelteuser && useradd -r -g svelteuser svelteuser && \
    npm install -g pnpm

WORKDIR /app
COPY --from=build /app/build ./build
COPY --from=build /app/package.json ./
COPY --from=build /app/pnpm-lock.yaml ./
RUN pnpm install --prod
RUN ls -la /app && ls -la /app/build || echo "Build directory not copied correctly"

USER svelteuser
EXPOSE 5150
ENV NODE_ENV=production HOST=0.0.0.0 PORT=5150 VITE_SSR=false
CMD ["dumb-init", "node", "./build/index.js"]