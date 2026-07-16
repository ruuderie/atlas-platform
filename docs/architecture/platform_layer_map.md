# Atlas Platform — Layer Map

> **Last updated:** 2026-07-11 | **Status:** Production (generics G01–G37+; see CURRENT_STATE Rev 11)

This document formalises the three-tier route and responsibility model introduced
by the `CorePlatformApp` migration. Every handler, route prefix, and state injection
point is traceable back to exactly one tier.

**Ground truth:** [`../CURRENT_STATE.md`](../CURRENT_STATE.md) — generic counts, frontend apps, workers.

---

## Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Tier 3 — Infrastructure                   │
│  api.rs — Auth, Sessions, Passkeys, Admin, A/B, Setup, RBAC  │
│  GET /api/version ← X-Atlas-Version header on ALL responses  │
└──────────────────────────┬──────────────────────────────────┘
                           │ get_active_apps() loop
            ┌──────────────┼──────────────────────────────┐
            ▼              ▼                              ▼
┌───────────────────┐ ┌──────────────────┐ ┌────────────────────────┐
│ Tier 1 — Core CMS │ │ Tier 2 — Anchor  │ │ Tier 2 — Folio (PM)    │
│ CorePlatformApp   │ │ AnchorApp        │ │ FolioApp / handlers/   │
│ core_platform.rs  │ │ Listings / CRM   │ │ folio/ (G09–G37 PM)    │
└───────────────────┘ └──────────────────┘ └────────────────────────┘
```

**Frontend apps (Leptos):** `anchor` · `network-instance` · `platform-admin` · `folio` · `shared-ui`  
**Platform generics:** G01–G37+ (see CURRENT_STATE registry — not “18 generics”).

---

## Tier 3 — Platform Infrastructure (`api.rs`)

**Owned by:** `backend/src/api.rs`  
**Responsible for:** Authentication plumbing, session management, platform security.

| Route prefix          | Owner handler              | Auth? |
|-----------------------|----------------------------|-------|
| `/login`, `/register` | `users`                    | No    |
| `/validate-session`   | `sessions`                 | No    |
| `/refresh-token`      | `sessions`                 | No    |
| `/api/auth/*`         | `auth_frontend`            | Mixed |
| `/api/passkeys/*`     | `passkeys`                 | Mixed |
| `/api/magic-links/*`  | `magic_links`              | No    |
| `/api/setup/*`        | `setup`                    | No    |
| `/api/ab/*`           | `ab_testing`               | Mixed |
| `/api/admin/*`        | `admin::routes`            | Yes   |
| `/api/version`        | `handlers::version`        | No    |
| `/health`             | `handlers::health`         | No    |
| `/logout`             | `users`                    | Yes   |
| `/api/accounts/*`     | `accounts`                 | Yes   |
| `/api/user-accounts/*`| `user_accounts`            | Yes   |
| `/api/users/*`        | `users`                    | Yes   |
| `/api/comms/*`        | `communications`           | Yes   |
| `/api/telemetry/*`    | `telemetry`                | Yes   |

### X-Atlas-Version Header

Every HTTP response from the platform carries:

```
X-Atlas-Version: <semver>+<git-sha>
# example: X-Atlas-Version: 0.9.1+f0e7c167
```

Set via `version_header_middleware` in `handlers/version.rs`.  
The SHA is stamped at compile time from the `ATLAS_BUILD_SHA` environment variable.  
CI must set `ATLAS_BUILD_SHA` and `ATLAS_BUILD_DATE`; they fall back to `"dev"` / `"unknown"` in local builds.

---

## Tier 1 — Core CMS Platform (`CorePlatformApp`)

**Owned by:** `backend/src/atlas_apps/core_platform.rs`  
**Trait:** `AtlasApp`  
**Registration position:** FIRST in `get_active_apps()` — ensures base routes exist before domain apps merge.

### Public Routes (no auth)

| Route                                        | Handler            |
|----------------------------------------------|--------------------|
| `GET /api/public/tenants`                    | `tenant`           |
| `GET /api/public/tenants/{id}`               | `tenant`           |
| `GET /api/tenants/lookup`                    | `tenant`           |
| `GET /api/public/app-instances/{id}/{type}`  | `app_instance`     |
| `GET /api/public/pages/{tenant_id}`          | `app_pages`        |
| `GET /api/public/pages/{tenant_id}/{*slug}`  | `app_pages`        |
| `GET /api/public/menus/{tenant_id}`          | `app_menus`        |
| `GET /api/public/menus/{tenant_id}/tree/{t}` | `app_menus`        |
| `GET /api/public/onboarding/*`               | `onboarding`       |
| `POST /api/forms/*`                          | `forms`            |
| `GET /api/public/feeds/*`                    | `feeds`            |

### Authenticated Routes (JWT required)

| Route                                       | Handler            |
|---------------------------------------------|--------------------|
| `PUT /api/tenants/{id}`                     | `tenant`           |
| `DELETE /api/tenants/{id}`                  | `tenant`           |
| `POST /api/tenants`                         | `tenant`           |
| `GET /api/tenants/{id}/settings`            | `tenant`           |
| `PUT /api/tenants/{id}/settings`            | `tenant`           |
| `GET /api/app-instances/*`                  | `app_instance`     |
| `POST /api/app-instances`                   | `app_instance`     |
| `GET /api/pages/{tenant_id}`                | `app_pages` (CRUD) |
| `GET /api/pages/{tenant_id}/{*slug}`        | `app_pages` (CRUD) |
| `POST /api/pages/{tenant_id}`               | `app_pages` (CRUD) |
| `PUT /api/pages/{tenant_id}/{*slug}`        | `app_pages` (CRUD) |
| `DELETE /api/pages/{tenant_id}/{*slug}`     | `app_pages` (CRUD) |
| `GET /api/menus/{tenant_id}`                | `app_menus` (CRUD) |
| `POST /api/menus/{tenant_id}`               | `app_menus` (CRUD) |
| `PUT /api/menus/{tenant_id}/{id}`           | `app_menus` (CRUD) |
| `DELETE /api/menus/{tenant_id}/{id}`        | `app_menus` (CRUD) |
| `GET/PUT /api/onboarding/*`                 | `onboarding`       |
| `GET /api/feeds/authenticated/*`            | `feeds`            |
| `GET /api/search/*`                         | `search`           |
| `GET /api/audit-logs/*`                     | `audit_logs`       |
| `GET /api/app-seeds/*`                      | `app_seeds`        |

### Provisioning Lifecycle (`AtlasApp::provision`)

```
POST /api/admin/platform/provision/{tenant_id}
```

Calls `provision()` on every registered `AtlasApp`. The `CorePlatformApp`
implementation seeds:
1. A published `home` page in `app_pages` (`ON CONFLICT DO NOTHING`)
2. A `header` navigation entry in `app_menus` (`ON CONFLICT DO NOTHING`)

Idempotent — safe to call multiple times.

---

## Tier 2 — Domain Sub-Apps

### AnchorApp

**Owned by:** `backend/src/atlas_apps/anchor.rs`  
**Responsible for:** Listing pages, CRM, profile rendering, Anchor-specific public routes.

> Routes documented separately in `docs/anchor_route_map.md` (TBD).

### FolioApp (Property Management)

**Owned by:** `backend/src/handlers/folio/` (44+ handler modules) + `backend/src/services/pm/`  
**Frontend:** `apps/folio` — 9 role portals (landlord, tenant, vendor, owner, PMC, agent, broker, STR host, property owner).  
**Surfaces:** G09–G26 PM generics, G27 scorecards, G31 leads, G32 RBAC shells, G35–G37 growth, G05 syndication.

### NetworkInstanceApp

**Frontend:** `apps/network-instance` (SSR + WASM) — contacts, leads, listings, G-27 scorecard mount.  
**Backend:** network/marketplace routes via platform admin + NI app handlers. Further NetworkInstanceApp AtlasApp registration still expanding (Phase 9+).

---

## Frontend Apps Layer

| App | Stack | Role |
|-----|-------|------|
| `apps/anchor` | Leptos SSR+WASM | CMS, listings, CRM (G-31 / Party) |
| `apps/network-instance` | Leptos SSR+WASM | Multi-tenant marketplace / directory |
| `apps/platform-admin` | Leptos CSR | Operator console (instances, flags, G-06, G-08, G-36/G-37, billing) |
| `apps/folio` | Leptos SSR+WASM | Property management portals |
| `apps/shared-ui` | Leptos lib | 85 UI primitives + G-27 Configurator / scorecard widgets |

---

## State Binding Contract

> **Critical invariant.** Violation causes silent route-dropping or panics.

```rust
// ✅ CORRECT — state-free constructor, bound once at AtlasApp boundary
pub fn public_routes_raw() -> Router<DatabaseConnection> { ... }

// in CorePlatformApp::public_router():
Router::new()
    .merge(handler::public_routes_raw())
    .with_state(db)   // ← bound EXACTLY ONCE here

// ❌ WRONG — pre-bound; Axum silently drops routes from pre-finalized sub-routers
pub fn public_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    public_routes_raw().with_state(db)  // ← never call inside an AtlasApp loop
}
```

**Root cause of historical regressions:**
- Apr 8 → Apr 15 404 regression: pre-bound `with_state()` inside handler
- May 2 2026 "Overlapping method route" panic (commit `1b84c375`): same pattern

---

## Platform-Admin API Modules

The Leptos CSR frontend (`apps/platform-admin`) exposes typed wrappers
for every backend API surface:

| Module               | Backend endpoint            | Phase |
|----------------------|-----------------------------|-------|
| `api/auth.rs`        | `/api/auth/*`               | 0     |
| `api/networks.rs`    | `/api/admin/platform/apps`  | 1     |
| `api/listings.rs`    | `/api/listings/*`           | 0     |
| `api/crm.rs`         | `/api/crm/*`                | 0     |
| `api/provision.rs`   | `/api/admin/platform/provision/{id}` | 4 |
| `api/pages.rs`       | `/api/pages/{tenant_id}`    | 5     |
| `api/menus.rs`       | `/api/menus/{tenant_id}`    | 5     |

### CMS Editor

`apps/platform-admin/src/pages/cms_editor.rs` uses `api/pages.rs` (Phase 5+).  
The block editor (`pages/block_editor.rs`, Phase 6) defines:
- `Block` — serde-tagged enum matching Anchor's seed format
- `BlockPreview` — lightweight admin preview card (not the full Anchor renderer)
- `block_templates()` — one-click block palette for the editor
- `parse_blocks()` — live JSON parse driving the right-pane preview

---

## Test Coverage

```
backend/ cargo test --workspace
  43 test files (27 integration + 16 unit) — see CURRENT_STATE.md
```

Key patterns:
- `tests/unit/` — pure no-DB tests
- `tests/*.rs` — DB-backed integration via `setup_test_app`
- `tests::atlas_apps_tests` — CorePlatformApp first in registry
- `traits::atlas_app::tests` — encapsulation compliance
