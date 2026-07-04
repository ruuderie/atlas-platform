# Adding a New Application to the Atlas Platform

> **Prerequisite Reading**: [`docs/CURRENT_STATE.md`](CURRENT_STATE.md) — this explains the current state after the Platform Generics v2 + Unification effort, including the mandatory "Generic Fitness Test" before creating new tables.

This document is the definitive checklist for integrating a new frontend application (Leptos/WASM or otherwise) into the Atlas Platform's CI/CD pipeline and Kubernetes infrastructure. Follow every step in order. Skipping any step will result in broken deployments or `ImagePullBackOff` errors.

---

## Overview of the Moving Parts

Every application in the platform is registered in **six places**:

1. `apps/<your-app>/Dockerfile` — how to build the image
2. `k8s/base/<your-app>.yaml` — the Kubernetes Deployment + Service manifest
3. `k8s/base/kustomization.yaml` — registers the manifest with Kustomize
4. `k8s/overlays/uat/ingress.yaml` + `k8s/overlays/prod/ingress.yaml` — routes a domain to the service
5. `.woodpecker.yml` — builds, pushes, and deploys the image in CI
6. `platform/registry.json` — registers the application for the Platform Product Dashboard

If any of these are missing, the deployment will silently fail or the app will be unreachable/unlisted.

---

## Step 1: Create the Dockerfile

Create `apps/<your-app>/Dockerfile`. Use an existing app as a reference:

- **Leptos SSR app** → copy `apps/anchor/Dockerfile`
- **Leptos CSR/WASM app** → copy `apps/platform-admin/Dockerfile`
- **Network-facing app** → copy `apps/network-instance/Dockerfile`

The Dockerfile must accept these build args that the pipeline injects automatically:

```dockerfile
ARG ATLAS_BUILD_SHA=dev
ARG ATLAS_BUILD_DATE=unknown
# Controls build optimization. Pipeline passes 'debug' on dev branch, 'release' on uat.
# Defaults to release so local docker builds are always optimized.
ARG BUILD_PROFILE=release
```

Use `BUILD_PROFILE` to gate the release flag in your build command:

```dockerfile
RUN if [ "$BUILD_PROFILE" = "release" ]; then \
      cargo leptos build --release; \
    else \
      cargo leptos build; \
    fi
```

And use it in the `COPY` stage to pick the correct binary path:

```dockerfile
ARG BUILD_PROFILE=release
COPY --from=builder /app/apps/target/${BUILD_PROFILE}/my-app /app/
```

---

## Step 2: Create the Kubernetes Base Manifest

Create `k8s/base/<your-app>.yaml`. This file defines the Deployment and Service. Use this template:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: <your-app>            # kebab-case, e.g. "my-app"
spec:
  replicas: 2
  selector:
    matchLabels:
      app: <your-app>
  template:
    metadata:
      labels:
        app: <your-app>
    spec:
      imagePullSecrets:
      - name: ghcr-login-secret
      containers:
      - name: <your-app>      # must match the deployment name exactly
        image: ghcr.io/ruuderie/<your-app>:<stable-sha>
        imagePullPolicy: Always
        envFrom:
        - configMapRef:
            name: app-config
        - secretRef:
            name: app-secrets
        ports:
        - containerPort: <port>   # the port your app listens on
        # ── Leptos Runtime Configuration ─────────────────────────────────────
        # CRITICAL — DO NOT REMOVE. For any Leptos SSR app (leptos_axum), these
        # env vars MUST be declared here, NOT only in the Dockerfile.
        # When Kubernetes applies envFrom: configMapRef, variables baked into the
        # Docker image via ENV are not guaranteed to survive. Any Leptos runtime
        # variable not explicitly listed in env: below will be silently dropped.
        #
        # LEPTOS_HASH_FILES must mirror `hash-files = true` in Cargo.toml.
        # If missing: server boots normally but injects no <script> tag → 502.
        # See docs/leptos_ssr_shell_pattern.md for the full incident breakdown.
        env:
        - name: DATABASE_URL
          value: "postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@$(DATABASE_HOST):$(DATABASE_PORT)/$(DATABASE_NAME)"
        - name: LEPTOS_SITE_ADDR
          value: "0.0.0.0:<port>"
        - name: LEPTOS_SITE_ROOT
          value: "site"             # must match the Dockerfile COPY destination
        - name: LEPTOS_HASH_FILES
          value: "true"
---
apiVersion: v1
kind: Service
metadata:
  name: <your-app>
spec:
  selector:
    app: <your-app>
  ports:
  - port: 80
    targetPort: <port>
```

> **Critical:** The `image:` field must contain a **real, pullable SHA tag** — never the placeholder string `ATLAS_IMAGE_TAG`. Set it to the SHA of the first image you push manually (see Step 5). The pipeline will update it automatically after that using `kubectl set image`.

> **Note (CSR apps only):** `platform-admin` is a pure CSR app built with Trunk. It does not use `leptos_axum` or SSR. The Leptos runtime env vars above do not apply to it — omit them from its manifest.

---

## Step 3: Register with Kustomize

Add your manifest to `k8s/base/kustomization.yaml`:

```yaml
resources:
  - namespace.yaml
  - backend.yaml
  - platform-admin.yaml
  - network-instance.yaml
  - anchor-app.yaml
  - <your-app>.yaml     # ← add this line
```

---

## Step 4: Add Ingress Rules

Add a routing rule for your app's domain in both overlays:

**`k8s/overlays/uat/ingress.yaml`** — add under `spec.rules`:
```yaml
- host: uat.<your-tenant-domain>.com
  http:
    paths:
    - path: /
      pathType: Prefix
      backend:
        service:
          name: <your-app>
          port:
            number: 80
```

**`k8s/overlays/prod/ingress.yaml`** — same block without the `uat.` prefix.

Also add the domain to `spec.tls[0].hosts` in both ingress files for automatic SSL via cert-manager.

---

## Step 5: Push a First Image Manually

Before the pipeline can deploy your app, a real image must exist in GHCR. Build and push it once manually so the base manifest has a valid tag to start from:

```bash
docker buildx build \
  --platform linux/amd64 \
  -t ghcr.io/ruuderie/<your-app>:<short-sha> \
  -t ghcr.io/ruuderie/<your-app>:latest \
  --push \
  -f apps/<your-app>/Dockerfile .
```

Then update the `image:` field in `k8s/base/<your-app>.yaml` to `ghcr.io/ruuderie/<your-app>:<short-sha>` and commit it.

> This is a one-time bootstrap step. After this, all future image updates are handled automatically by the Woodpecker pipeline.

---

## Step 6: Register in the Woodpecker Pipeline

Add two blocks to `.woodpecker.yml`:

### 6a — Publish steps (builds and pushes the Docker image)

Publish steps are **split by branch** so dev gets a fast debug build and uat gets an optimized release build. Add both variants after the last `publish_*_uat` step and **before** `deploy_platform_k8s`:

```yaml
  # dev branch — debug build (fast iteration)
  publish_<your_app>_dev:
    image: woodpeckerci/plugin-docker-buildx
    privileged: true
    settings:
      mirror: https://mirror.gcr.io
      repo: ghcr.io/ruuderie/<your-app>
      tags:
        - latest
        - ${CI_COMMIT_SHA:0:8}
      registry: ghcr.io
      context: .
      dockerfile: apps/<your-app>/Dockerfile
      build_args:
        - ATLAS_BUILD_SHA=${CI_COMMIT_SHA}
        - ATLAS_BUILD_DATE=${CI_PIPELINE_CREATED}
        - BUILD_PROFILE=debug
      username:
        from_secret: docker_username
      password:
        from_secret: docker_password
    when:
      branch: dev
      path:
        - 'apps/<your-app>/**'
        - 'apps/shared-ui/**'   # shared-ui changes rebuild all dependent apps
        - '.woodpecker.yml'

  # uat branch — release build (optimized)
  publish_<your_app>_uat:
    image: woodpeckerci/plugin-docker-buildx
    privileged: true
    settings:
      mirror: https://mirror.gcr.io
      repo: ghcr.io/ruuderie/<your-app>
      tags:
        - latest
        - ${CI_COMMIT_SHA:0:8}
      registry: ghcr.io
      context: .
      dockerfile: apps/<your-app>/Dockerfile
      build_args:
        - ATLAS_BUILD_SHA=${CI_COMMIT_SHA}
        - ATLAS_BUILD_DATE=${CI_PIPELINE_CREATED}
        - BUILD_PROFILE=release
      username:
        from_secret: docker_username
      password:
        from_secret: docker_password
    when:
      branch: uat
      path:
        - 'apps/<your-app>/**'
        - 'apps/shared-ui/**'
        - '.woodpecker.yml'
```

> **Note on `context`:** Most apps use `.` (repo root) as the Docker build context because they import from `apps/shared-ui`. Only `backend` uses `backend` as context. Check your Dockerfile's `COPY` statements to confirm which context you need.

### 6b — Deploy wiring (updates the running pod image)

Because Woodpecker's `CI_PIPELINE_FILES` variable is notoriously fragile on large commits, we use a robust file-flag approach to trigger deployments.

First, add a marker step **before** `deploy_platform_k8s`:

```yaml
  mark_<your-app>_built:
    image: alpine
    commands: [ "touch .<your-app>_built" ]
    when:
      branch: [dev, uat]
      path: [ "apps/<your-app>/**", "apps/shared-ui/**", ".woodpecker.yml" ]
```

Next, inside the `deploy_platform_k8s` step's script block, add an `if` block for your app:

```yaml
          if [ -f .<your-app>_built ]; then
            update_service <your-app> "$REGISTRY/<your-app>:$SHORT_SHA"
          fi
```

And finally, add `&& [ ! -f .<your-app>_built ]` to the skip condition at the top of that block.

---

## Step 7: Phase 2 Platform Product Registration

Update `platform/registry.json` to allow your application to be discovered by the Platform Product Dashboard and side-nav integrations. 

Add your entry to the `apps` array:

```json
{
  "name": "<your-app>",
  "display_name": "My New App",
  "route": "/<path>",
  "enabled": true,
  "roles": ["admin", "user"]
}
```

This triggers the platform UI to include your app in global navigation and health monitoring automatically.

---

## How the Pipeline Deploys Images (Architecture Note)

The pipeline uses a **`kubectl set image` strategy** — not manifest substitution. This is intentional and important:

- `kubectl apply -k` is called first to apply config changes (ConfigMaps, Secrets, Ingress rules).
- `kubectl set image` is then called **only for services whose source code changed** in this commit, using the exact `$SHORT_SHA` that was built and pushed.
- Services whose source did not change are **not touched** — their existing running pod image is left as-is.

This means a `.woodpecker.yml`-only change (e.g., updating pipeline logic) will apply config/secrets but will **not** roll out any pods, because no new images were built.

> **Why this matters:** The old approach used `sed` to substitute `ATLAS_IMAGE_TAG` in base manifests before `kubectl apply`. This caused `ImagePullBackOff` whenever the tag substitution ran for a service that wasn't rebuilt in that pipeline run — K8s would try to pull a non-existent image. The current approach eliminates this entirely.

---

## Checklist Summary

| Step | What | Where |
|------|------|-------|
| 1 | Create `Dockerfile` with `BUILD_PROFILE` arg and `ENV LEPTOS_HASH_FILES="true"` | `apps/<your-app>/Dockerfile` |
| 2 | Create K8s manifest with real image tag **and Leptos env block** | `k8s/base/<your-app>.yaml` |
| 3 | Register in Kustomize | `k8s/base/kustomization.yaml` |
| 4 | Add ingress rules (UAT + prod) | `k8s/overlays/uat/ingress.yaml`, `k8s/overlays/prod/ingress.yaml` |
| 5 | Push first image manually to GHCR | (one-time bootstrap) |
| 6a | Add `publish_<app>_dev` + `publish_<app>_uat` steps | `.woodpecker.yml` |
| 6b | Add `update_service` call in deploy step | `.woodpecker.yml` |
| 7 | Register in product registry | `platform/registry.json` |

---

## Related Documentation

- [`deployment_environments.md`](./deployment_environments.md) — UAT vs. prod environment config, secrets management, Kustomize overlays
- [`architecture.md`](./architecture.md) — overall platform architecture
- [`apps_walkthrough.md`](./apps_walkthrough.md) — existing app descriptions and conventions
- [`platform_registry_schema.md`](./platform_registry_schema.md) — schema validation for the product registry

---

# Phase 2 — Platform Product Registration

> [!IMPORTANT]
> This phase is **required for every product app that has a marketing homepage**. Phase 1 gets the pod running; Phase 2 makes the homepage visible. Missing Phase 2 was the root cause of the folio 404 in July 2026.

For a detailed explanation of the content resolution algorithm and all DB tables involved, see [`product_page_system.md`](./product_page_system.md).

## Step 8: Seed `platform_products`

Create an idempotent migration. The `launch_mode` must **never** be `"draft"` for a live product.

```rust
manager.get_connection().execute_unprepared(
    "INSERT INTO platform_products (
         id, name, slug, app_slug, status, launch_mode,
         pre_order_enabled, pre_order_currency, pre_order_sold, waitlist_count,
         apex_domain_verified, created_at, updated_at
     )
     VALUES (
         gen_random_uuid(), 'MyApp', 'myapp', 'property_management',
         'active', 'waitlist',
         false, 'usd', 0, 0, false, NOW(), NOW()
     )
     ON CONFLICT (slug) DO NOTHING;"
).await?;
```

**`launch_mode` → homepage behavior:**

| Value | Homepage shows |
|---|---|
| `"draft"` | ❌ `<NotFound/>` — dev only |
| `"waitlist"` | ✅ waitlist form |
| `"active"` | ✅ get-started CTA |
| `"beta"` / `"invite_only"` | ✅ invite flow |

## Step 9: Seed `product_page_templates`

This row is the content fallback until the GTM Landing Page Builder publishes a page for this product.

```rust
manager.get_connection().execute_unprepared(
    "INSERT INTO product_page_templates (
         id, product_id, hero_payload, blocks_payload,
         meta_title, meta_description, cta_label, cta_action,
         created_at, updated_at
     )
     SELECT gen_random_uuid(), p.id,
            '{}'::jsonb, '{}'::jsonb,
            'MyApp — Tagline', 'Meta description.',
            'Join the Waitlist', 'waitlist',
            NOW(), NOW()
     FROM platform_products p
     WHERE p.slug = 'myapp'
       AND NOT EXISTS (
           SELECT 1 FROM product_page_templates t WHERE t.product_id = p.id
       );"
).await?;
```

> `hero_payload` and `blocks_payload` can be `{}` when the Leptos frontend has hardcoded UI. The fields exist for CMS-driven content when the GTM builder is used.

**Reference implementation:** `backend/src/migration/m20260926_folio_product_seed.rs`

## Step 10: Provision Domain via Ingress Sidecar

The ingress sidecar creates a k8s Ingress object mapping domain → service. Call it programmatically (not YAML):

```bash
curl -X POST http://ingress-sidecar:9100/api/ingress/provision \
  -H 'Content-Type: application/json' \
  -d '{
    "tenant_slug": "myapp1",
    "domain":      "myapp1.atlas.oply.co",
    "app_slug":    "property_management"
  }'
```

**App slug → k8s service:**

| `app_slug` | k8s Service |
|---|---|
| `"property_management"` / `"folio"` | `folio` |
| `"anchor"` | `anchor-app` |
| `"network_instance"` | `network-instance` |

TLS is automatic: `*.atlas.oply.co` uses the shared wildcard cert. Custom domains get cert-manager HTTP-01.

## Step 11: Verify

```bash
# API returns 200 with correct launch_mode
curl -s https://api.atlas.oply.co/api/pub/products/myapp | jq '.launch_mode'

# Homepage returns 200
curl -sk -o /dev/null -w "%{http_code}" https://myapp1.atlas.oply.co/

# No <NotFound/> rendered in HTML
curl -sk https://myapp1.atlas.oply.co/ | grep -c "not-found"
# should output: 0
```

---

## Full Checklist

### Phase 1 — Infrastructure

| Step | What | Where |
|---|---|---|
| 1 | `Dockerfile` with `BUILD_PROFILE` + `LEPTOS_HASH_FILES` | `apps/<app>/Dockerfile` |
| 2 | K8s manifest (real image SHA, Leptos env block, `ATLAS_API_URL`) | `k8s/base/<app>.yaml` |
| 3 | Register in Kustomize | `k8s/base/kustomization.yaml` |
| 4 | Ingress rules (UAT + prod) | `k8s/overlays/*/ingress.yaml` |
| 5 | Push first image manually to GHCR | One-time bootstrap |
| 6a | `publish_<app>_dev` + `publish_<app>_uat` steps | `.woodpecker.yml` |
| 6b | `update_service` call in deploy step | `.woodpecker.yml` |

### Phase 2 — Platform Product Registration

| Step | What | Where |
|---|---|---|
| 8 | Seed `platform_products` (`launch_mode ≠ "draft"`) | Migration `m2026xxxx_<slug>_product_seed.rs` |
| 9 | Seed `product_page_templates` (idempotent) | Same migration |
| 10 | Provision domain via ingress sidecar | Platform admin or `curl` |
| 11 | Verify API 200 + homepage 200 + no `not-found` | `curl` checks |
