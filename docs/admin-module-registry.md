# Admin Module Registry

> **Platform primitive.** Every Atlas app participates. No hardcoded tabs anywhere.

## Overview

The Admin Module Registry replaces the previous hardcoded sidebar navigation in both `anchor` and `network-instance` admin dashboards with a database-driven, per-tenant configurable module system.

```
┌─────────────────────────────────────────────────────┐
│  Platform Operator (PlatformSuperAdmin)             │
│  POST /api/platform/tenants/{id}/modules            │
│  → upserts module config, enforces is_fixed         │
└────────────────────┬────────────────────────────────┘
                     │
         ┌───────────▼───────────┐
         │  app_instance_module  │  ← source of truth (DB)
         │  (one row per module  │
         │   per app instance)   │
         └───────────┬───────────┘
                     │
         ┌───────────▼──────────────────────────────┐
         │  GET /api/admin/modules                   │
         │  → sorted, enabled modules for caller     │
         └───────────┬──────────────────────────────┘
                     │
         ┌───────────▼─────────────────────────────────┐
         │  AdminModuleSidebar (shared-ui)              │
         │  SidebarTheme::Anchor  / SidebarTheme::Network│
         └─────────────────────────────────────────────┘
```

---

## Schema

```sql
CREATE TABLE app_instance_module (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    app_instance_id UUID        NOT NULL REFERENCES app_instances(id) ON DELETE CASCADE,
    module_type     TEXT        NOT NULL,                 -- SCREAMING_SNAKE_CASE AdminModuleType
    display_name    TEXT        NOT NULL,
    icon            TEXT,                                 -- material-symbols name (optional)
    sort_order      INTEGER     NOT NULL DEFAULT 0,       -- ascending → left-to-right tab order
    is_enabled      BOOLEAN     NOT NULL DEFAULT true,
    is_fixed        BOOLEAN     NOT NULL DEFAULT false,   -- true = cannot be disabled
    config          JSONB,                               -- reserved for per-module tenant config
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (app_instance_id, module_type)
);
```

---

## Module Types

Defined in `backend/src/models/admin_module.rs` as `AdminModuleType` (strum `SCREAMING_SNAKE_CASE`).

| Type | Category | Fixed | Notes |
|------|----------|-------|-------|
| `DASHBOARD` | Platform | ✅ | Always first |
| `SETTINGS` | Platform | ✅ | |
| `SECURITY` | Platform | ✅ | Passkeys, sessions |
| `BLOG` | Content | | |
| `RESUME_PROFILES` | Content | | |
| `RESUME_ENTRIES` | Content | | |
| `LANDING_PAGES` | Content | | |
| `WEBFORMS` | Content | | Lead capture schemas |
| `NAVIGATION` | Appearance | | |
| `FOOTER` | Appearance | | |
| `PAGE_HEADERS` | Appearance | | |
| `LEADS` | CRM & Comms | | Unvetted / inbound inquiries |
| `CONTACTS` | CRM & Comms | | Opted-in, vetted members |
| `LEAD_OPTIONS` | CRM & Comms | | |
| `SERVICES` | B2B | | |
| `CASE_STUDIES` | B2B | | |
| `HIGHLIGHTS` | B2B | | |
| `LISTINGS` | Advanced | | network-instance marketplace |
| `PROPERTIES` | Advanced | | |
| `CUSTOM` | Advanced | | Escape hatch |

### CRM Semantics

> [!IMPORTANT]
> **Lead** = an inbound, unvetted inquiry. The person has NOT opted in to communications.
> **Contact** = a vetted, onboarded person who HAS opted in to receive updates/communications.
>
> Do not conflate these. Never use "Mailing List" as a tab label — it has been renamed to `CONTACTS`.

---

## Adding a New Module Type

1. **Backend model** — add variant to `AdminModuleType` enum in `backend/src/models/admin_module.rs`  
   Set `category()`, `is_fixed()`, `default_sort_order()`, `to_display_name()`.

2. **App default set** — add a tuple to `AnchorApp::default_modules()` or `NetworkInstanceApp::default_modules()` in `backend/src/atlas_apps/`.

3. **Migration** — write a migration to seed existing tenants with the new module if it should be enabled by default.

4. **shared-ui** — add the new variant to `AdminModuleType` in `apps/shared-ui/src/components/admin_module_sidebar.rs` and add an icon mapping in `default_icon()`.

5. **Frontend dispatch** — add a `match` arm in the admin page(s) (`anchor/admin.rs`, `network-instance/admin.rs`) to render the new content panel.

---

## AtlasApp Contract

Every app that wants admin tabs must implement `default_modules()` on its `AtlasApp` impl:

```rust
fn default_modules(&self) -> Vec<(AdminModuleType, &'static str, i32, bool)> {
    use AdminModuleType as M;
    vec![
        // (type, display_name, sort_order, is_fixed)
        (M::Dashboard, "Dashboard", 0,  true),
        (M::Blog,      "Blog",      10, false),
        (M::Settings,  "Settings",  20, true),
        (M::Security,  "Security",  30, true),
    ]
}

async fn provision(&self, db: &DatabaseConnection, tenant_id: Uuid) -> Result<(), String> {
    use crate::services::module_provisioning::{resolve_app_instance_id, seed_default_modules};
    let app_instance_id = resolve_app_instance_id(db, tenant_id, self.app_id()).await?;
    seed_default_modules(db, app_instance_id, self.default_modules()).await
}
```

`seed_default_modules()` uses `ON CONFLICT DO NOTHING` — idempotent, safe to call multiple times.

---

## Theming

`AdminModuleSidebar` in `shared-ui` uses `SidebarTheme` to style itself per-app:

| Theme | App | Appearance |
|-------|-----|------------|
| `SidebarTheme::Anchor` | anchor | Dark, monospace, `#0a0a0a` bg |
| `SidebarTheme::Network` | network-instance | Light, sans-serif, white bg |
| `SidebarTheme::Custom(tokens)` | any future app | Fully custom token set |

To add a new app theme: add a new `SidebarTheme` variant and fill in a `SidebarThemeTokens` struct. No component internals change.

---

## API Reference

### `GET /api/admin/modules`

**Auth:** Session cookie (tenant-scoped — returns modules for the caller's tenant only)  
**Response:** `Vec<AdminModuleConfig>` sorted by `sort_order` ASC, `is_enabled = true` only

```json
[
  { "module_type": "DASHBOARD", "display_name": "Dashboard", "icon": null, "sort_order": 0, "is_fixed": true, "category": "PLATFORM" },
  { "module_type": "BLOG",      "display_name": "Blog",      "icon": null, "sort_order": 10, "is_fixed": false, "category": "CONTENT" }
]
```

### `POST /api/platform/tenants/{tenant_id}/modules`

**Auth:** `PlatformSuperAdmin` only  
**Body:** `UpsertModuleInput`

```json
{
  "module_type": "LEADS",
  "display_name": "Leads",
  "is_enabled": true,
  "sort_order": 160,
  "icon": null
}
```

**Invariants enforced server-side:**
- Fixed modules (`DASHBOARD`, `SETTINGS`, `SECURITY`) always have `is_enabled = true` regardless of input
- Unknown `module_type` strings are rejected with `400 Bad Request`

---

## Provisioning Flow

```
New Tenant created
    ↓
AtlasApp::provision(db, tenant_id)
    ↓
resolve_app_instance_id(db, tenant_id, app_id)
    ↓
seed_default_modules(db, app_instance_id, app.default_modules())
    ↓
INSERT INTO app_instance_module ... ON CONFLICT DO NOTHING
    ↓
Tenant has all default tabs ready immediately
```

---

## Future Work

- **Phase 6**: Email/SMS opt-in on the `Contact` model (communication preferences, GDPR-safe unsubscribe)
- **Sort order drag-to-reorder**: Platform Admin UI to drag-reorder tabs without deployment
- **Per-module config JSONB**: Allow tenants to configure module-level options (e.g. Blog category filter)
- **Module enable/disable toggle**: Platform Admin UI to toggle modules on/off per tenant
