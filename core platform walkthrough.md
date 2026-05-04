# CorePlatformApp — PR Walkthrough

**Branch:** `feat/core-platform-app` → `uat`
**Commits:** 4 (`f0e7c167` → `125b04e2`)
**Tests:** 44 integration + 5 unit, 0 failures

---

## What This PR Does

Formalizes the Atlas Platform's implicit "core layer" into an explicit, fully encapsulated
`CorePlatformApp` that implements the `AtlasApp` trait. After this work:

- `api.rs` contains **only Tier 3 platform infrastructure** (auth, sessions, passkeys, admin, rate-limiting, CORS).
- All CMS and platform service routes are owned by `CorePlatformApp` at the Tier 1 boundary.
- Platform admins can manage pages, navigation menus, and view block content through a visual CMS editor.
- Every API response carries an `X-Atlas-Version` header for cross-node drift detection.

---

## Phases Completed

### Phase 1 — Handler Refactoring
Refactored all Tier 1 handlers to expose **state-free** `*_raw()` constructors returning
`Router<DatabaseConnection>`. State is now bound exactly once at the `AtlasApp` boundary via
`.with_state(db)`.

**Handlers refactored:** `tenant`, `app_pages`, `app_menus`, `app_instance`, `onboarding`, `feeds`, `app_seeds`

**Why this matters:** Axum silently drops routes when a `Router` that already has `with_state()` bound
is merged into another router. The `_raw()` pattern prevents this class of regression entirely.
(Caused two production 404 regressions before — Apr 8 and commit `1b84c375`.)

---

### Phase 2+3 — CorePlatformApp + api.rs Cleanup

**New file:** `backend/src/atlas_apps/core_platform.rs`

`CorePlatformApp` is registered **first** in `get_active_apps()` and owns:
- All public CMS routes (`/api/public/pages/*`, `/api/public/menus/*`, `/api/public/onboarding/*`, etc.)
- All authenticated CRUD routes (`/api/pages/*`, `/api/menus/*`, `/api/app-instances/*`, etc.)

`api.rs` was stripped of all Tier 1 route registrations. It now only manages Tier 3:
auth, sessions, passkeys, admin, A/B testing, setup.

---

### Phase 4 — Provision Lifecycle

**New file:** `backend/src/admin/provision.rs`

`POST /api/admin/platform/provision/{tenant_id}` iterates all registered `AtlasApp`s and calls
`provision()` on each. `CorePlatformApp::provision()` seeds:

1. A published `home` page in `app_pages` — `ON CONFLICT DO NOTHING`
2. A `header` navigation entry in `app_menus` — `ON CONFLICT DO NOTHING`

This is called automatically by the `create_network()` flow in platform-admin, so every new
tenant immediately has working CMS infrastructure without manual intervention.

**Frontend:** `api/provision.rs` + `api/networks.rs` updated to bundle all three steps
(create tenant → create app instance → provision) into one `create_network()` call.

---

### Phase 5 — CMS Pages & Menus CRUD

**Backend:**
- `app_pages.rs`: added `POST`, `PUT`, `DELETE` authenticated routes at `/api/pages/{tenant_id}`
- `app_menus.rs`: added `POST`, `PUT`, `DELETE` authenticated routes at `/api/menus/{tenant_id}`
- Both registered in `CorePlatformApp::authenticated_router()`

**Frontend (platform-admin):**
- `api/pages.rs` — typed wrappers: `list_pages`, `create_page`, `update_page`, `delete_page`
- `api/menus.rs` — typed wrappers: `list_menus`, `create_menu`, `update_menu`, `delete_menu`
- `pages/cms_editor.rs` — repointed from legacy `api/listings` to `api/pages`; added slug, page type, is_published, and tenant warning banner
- `pages/menu_editor.rs` — **new** full CRUD page at `/menus`:
  - Menus grouped by type (header / footer / sidebar)
  - Inline row editing (no modal)
  - Sticky add-item panel
  - Hover-revealed delete + edit actions
  - Empty state pointing to provisioning
  - Quick Reference card with public API endpoints
- Sidebar: "Content" section split into **Pages** (`/cms`) + **Navigation** (`/menus`)

---

### Phase 6 — Visual Block Editor

**New file:** `apps/platform-admin/src/pages/block_editor.rs`

Provides:

| Export | Purpose |
|--------|---------|
| `Block` enum | `#[serde(tag = "type")]` discriminated union matching Anchor's seed JSON format |
| `BlockPreview` | Admin card component — renders a lightweight preview of each block type |
| `block_templates()` | One-click palette: Hero, Grid, Callout, RichText, Stats, RawHtml |
| `parse_blocks(json)` | Parses `blocks_json` signal → `(Vec<Block>, Option<String error>)` |

**`cms_editor.rs` changes:**
- Replaced raw JSON textarea with a 3-column block palette
- Click adds block JSON to `blocks_json` signal (append before `]`)
- Visual ↔ Raw JSON toggle
- Live parse error banner in both panes
- Right pane: page meta summary card + live scrollable block stack

---

### Phase 7 — Version Endpoint + X-Atlas-Version Header

**New file:** `backend/src/handlers/version.rs`

```
GET /api/version → { "version": "0.9.1", "build_sha": "f0e7c167", "build_date": "2026-05-02" }
```

- No auth required (monitoring, ops, health probes)
- `ATLAS_VERSION` from `CARGO_PKG_VERSION` at compile time
- `ATLAS_BUILD_SHA` / `ATLAS_BUILD_DATE` from CI env vars via `option_env!` (const-safe `match` pattern)
- `version_header_middleware` injects `X-Atlas-Version: <semver>+<sha>` on **every** API response

**Frontend:**
- `api/version.rs` — `get_version()` client wrapper
- Sidebar footer: version chip shows `v0.9.1 f0e7c16` — full SHA on hover

---

### Phase 8 — Architecture Documentation

| File | Contents |
|------|---------|
| `docs/architecture/platform_layer_map.md` | Full 3-tier route ownership table, State Binding Contract, X-Atlas-Version spec, provisioning lifecycle, platform-admin API module map, test coverage summary |
| `docs/atlas_app_integration.md` | Updated integration protocol: new `public_router`/`authenticated_router` signature, State Binding Contract examples, `provision()` hook, route prefix ownership table |

---

## Files Changed (key files)

```
backend/src/atlas_apps/core_platform.rs     [NEW]  CorePlatformApp — Tier 1 route owner
backend/src/admin/provision.rs              [NEW]  POST /api/admin/platform/provision/{tenant_id}
backend/src/handlers/version.rs            [NEW]  GET /api/version + X-Atlas-Version middleware
backend/src/api.rs                         [MOD]  Stripped to Tier 3 only
backend/src/atlas_apps/mod.rs              [MOD]  CorePlatformApp registered first
backend/src/handlers/app_pages.rs          [MOD]  CRUD + state-free _raw()
backend/src/handlers/app_menus.rs          [MOD]  CRUD + state-free _raw()
backend/src/traits/atlas_app.rs            [MOD]  provision() default no-op + State Binding Contract doc

apps/platform-admin/src/pages/cms_editor.rs    [MOD]  Visual block editor, api/pages
apps/platform-admin/src/pages/menu_editor.rs   [NEW]  Full menus CRUD UI
apps/platform-admin/src/pages/block_editor.rs  [NEW]  Block enum + BlockPreview + palette
apps/platform-admin/src/app.rs             [MOD]  /menus route, nav links, version footer chip
apps/platform-admin/src/api/pages.rs       [NEW]  list/create/update/delete page wrappers
apps/platform-admin/src/api/menus.rs       [NEW]  list/create/update/delete menu wrappers
apps/platform-admin/src/api/version.rs     [NEW]  get_version() client
apps/platform-admin/src/api/networks.rs    [MOD]  3-step create_network() with provision

docs/architecture/platform_layer_map.md    [NEW]  Full 3-tier architecture map
docs/atlas_app_integration.md              [MOD]  Updated integration protocol
```

---

## Verification

### Automated Tests
```
cargo test --workspace -j 1 -- --test-threads=1

test result: ok. 44 passed; 0 failed   ← integration tests (postgres)
test result: ok.  5 passed; 0 failed   ← unit tests
```

### Manual UAT Checklist

| Step | Command / Action | Expected |
|------|-----------------|---------|
| Check version header | `curl -I https://uat.atlas.ruuderie.com/api/version` | `X-Atlas-Version: 0.x.x+<sha>` in headers |
| Version endpoint | `GET /api/version` | `{ version, build_sha, build_date }` |
| Version chip | Open platform-admin sidebar | Version chip visible in footer |
| Pages CRUD | `/cms?tenant_id=<uuid>` → "All Pages" tab | Lists existing pages |
| Create page | Fill form, click "Publish Page" | Page appears in table |
| Menu CRUD | `/menus?tenant_id=<uuid>` | Header menu items visible |
| Add menu item | Fill "Add Menu Item" panel | Row appears in Header group |
| Inline edit | Hover row → pencil → change label → Save | Row updates without reload |
| Block palette | `/cms` → Editor → click "Hero" block | Block card appears in left + right pane |
| Raw JSON toggle | Click `{ } Raw JSON` | Textarea with JSON visible |
| Back to visual | Click `← Visual` | Block stack restored from JSON |
| Provisioning | `POST /api/admin/platform/provision/{tenant_id}` | `200 OK`, home page + header menu seeded |
| Multi-call idempotency | Repeat provision call | `200 OK`, no duplicates |

---

## Deployment Notes

1. **CI must set** `ATLAS_BUILD_SHA` and `ATLAS_BUILD_DATE` as build args — otherwise the version
   endpoint returns `{ "build_sha": "dev", "build_date": "unknown" }`.

2. **Existing tenants** already have home pages and menus — provisioning is idempotent and safe
   to run against existing data.

3. **`CorePlatformApp` must remain first** in `get_active_apps()`. Reordering causes any app
   that depends on the provisioned `app_pages`/`app_menus` records to fail on first render.
