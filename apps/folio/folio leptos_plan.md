# Implementation Plan: Folio Dynamic Data Integration

Connect every Folio portal page to its live backend API, replacing placeholder `"— "` values and `page-placeholder` stubs with real data fetched via Leptos `Resource` / server functions.

## Background

The Folio app (`apps/folio`) is a Leptos SSR+WASM application. All pages have been scaffolded but only the **Landlord Dashboard** has any reactive wiring — and even that only shows `"—"` for every stat. Every other page (`leases.rs`, `portfolio.rs`, `leads.rs`, etc.) is a `page-placeholder` stub that says *"connect to /api/folio/..."*.

The backend handlers (`backend/src/handlers/folio/`) are **fully implemented** and ready — 30 handler files covering assets, leases, leads, maintenance, billing, campaigns, reservations, vendors, and more.

The fix is to:
1. Define shared Rust **model types** in `apps/folio/src/models/` that mirror backend JSON responses.
2. Add Leptos **`#[server]` functions** in `apps/folio/src/api/` (one module per domain) that call `atlas_client::authenticated_get/post`.
3. Wire each page component to call these server fns via `Resource`/`LocalResource`, and render real data.

---

## Scope

> [!IMPORTANT]
> The ROUTES.md shows 57 total routes. **20 are stubs, 37 are missing entirely.**
> This plan focuses exclusively on converting the **20 existing stubs** to dynamic data.
> The 37 missing routes (STR, Owner, PMC, wizards, etc.) are **out of scope** — they require new routes added to `app.rs` first.

### Stubs to convert (20 pages across 3 portals)

| Portal | Page | Backend endpoint |
|--------|------|-----------------|
| Landlord | Dashboard (`/l`) | `GET /api/folio/portfolios`, `GET /api/folio/leases`, `GET /api/folio/leads`, `GET /api/folio/maintenance`, `GET /api/folio/reservations` |
| Landlord | Portfolio (`/l/portfolio`) | `GET /api/folio/portfolios` |
| Landlord | Assets (`/l/assets`) | `GET /api/folio/assets` |
| Landlord | Leases (`/l/leases`) | `GET /api/folio/leases` |
| Landlord | Leads (`/l/leads`) | `GET /api/folio/leads` |
| Landlord | Campaigns (`/l/campaigns`) | `GET /api/folio/campaigns` |
| Landlord | Billing (`/l/billing`) | `GET /api/folio/billing/invoice/btc/audit` |
| Landlord | STR Compliance (`/l/str`) | `GET /api/folio/str` |
| Landlord | Catalog (`/l/catalog`) | `GET /api/folio/catalog` |
| Landlord | Vendors (`/l/vendors`) | `GET /api/folio/vendors` |
| Landlord | Reservations (`/l/reservations`) | `GET /api/folio/reservations` |
| Tenant | Dashboard (`/t`) | `GET /api/folio/leases`, `GET /api/folio/maintenance` |
| Tenant | My Lease (`/t/my-lease`) | `GET /api/folio/leases` |
| Tenant | Payments (`/t/payments`) | `GET /api/folio/billing/invoice/btc/audit` |
| Tenant | Maintenance (`/t/maintenance`) | `GET /api/folio/maintenance` |
| Tenant | Reservations (`/t/reservations`) | `GET /api/folio/reservations` |
| Vendor | Dashboard (`/v`) | `GET /api/folio/maintenance` (work orders) |
| Vendor | Work Orders (`/v/work-orders`) | `GET /api/folio/maintenance` |
| Vendor | Invoices (`/v/invoices`) | `GET /api/folio/billing/invoice/btc/audit` |

---

## Proposed Changes

### 1. Shared Models Layer

#### [MODIFY] [models/mod.rs](file:///Users/oply/src/git/orbit_/atlas-platform/apps/folio/src/models/mod.rs)

Add module declarations for each new domain model file.

#### [NEW] `apps/folio/src/models/portfolio.rs`

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortfolioSummary {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub asset_count: i64,
    pub created_at: String,
}
```

#### [NEW] `apps/folio/src/models/asset.rs`

Mirrors `AssetSummary` from backend: `id`, `name`, `asset_type`, `status`, `serial_or_folio_number`.

#### [NEW] `apps/folio/src/models/lease.rs`

Mirrors `LeaseSummary`: `id`, `asset_id`, `currency`, `status`, `start_date`, `end_date`.

#### [NEW] `apps/folio/src/models/lead.rs`

Mirrors `Lead` from `LeadService::list`: `id`, `first_name`, `last_name`, `email`, `status`, `source`.

#### [NEW] `apps/folio/src/models/maintenance.rs`

Mirrors `MaintenanceSummary`: `id`, `asset_id`, `subject`, `status`, `priority`, `case_type`, `created_at`.

#### [NEW] `apps/folio/src/models/reservation.rs`

Mirrors `ReservationSummary` from `reservations.rs` handler.

#### [NEW] `apps/folio/src/models/campaign.rs`, `vendor.rs`, `catalog.rs`

One per domain — minimal summary structs that match the backend JSON.

---

### 2. API / Server Functions Layer

Create `apps/folio/src/api/` as a new module directory.

#### [NEW] `apps/folio/src/api/mod.rs`

Exports all sub-modules.

#### [NEW] `apps/folio/src/api/portfolio.rs`

```rust
#[server(GetPortfolios, "/api")]
pub async fn get_portfolios() -> Result<Vec<PortfolioSummary>, ServerFnError> {
    let headers = extract::<HeaderMap>().await?;
    let token = extract_token(&headers)?;
    atlas_client::authenticated_get("/api/folio/portfolios", &token, None)
        .await.map_err(ServerFnError::new)
}
```

#### [NEW] `apps/folio/src/api/asset.rs`
#### [NEW] `apps/folio/src/api/lease.rs`
#### [NEW] `apps/folio/src/api/lead.rs`
#### [NEW] `apps/folio/src/api/maintenance.rs`
#### [NEW] `apps/folio/src/api/reservation.rs`
#### [NEW] `apps/folio/src/api/campaign.rs`
#### [NEW] `apps/folio/src/api/billing.rs`
#### [NEW] `apps/folio/src/api/vendor.rs`
#### [NEW] `apps/folio/src/api/catalog.rs`

Each follows the same pattern: one `#[server]` fn that calls `authenticated_get` with the matching backend path.

> [!NOTE]
> Token extraction in server fns uses `leptos_axum::extract::<axum::http::HeaderMap>()` and reads `Authorization: Bearer <token>` or the `atlas_session` cookie — identical to the pattern already in `auth.rs`. We'll extract this into a shared `crate::api::extract_token()` helper to avoid repetition.

---

### 3. Landlord Pages

#### [MODIFY] [dashboard.rs](file:///Users/oply/src/git/orbit_/atlas-platform/apps/folio/src/pages/landlord/dashboard.rs)

The `StatCard` value prop is currently `&'static str`. Change to accept `Signal<String>` so values can be reactive.

Wire each stat card to a `Resource` fetch:
- **Properties** → `get_portfolios()` → count assets across all portfolios
- **Active Leases** → `get_leases()` → filter `status == "active"` count
- **Open Leads** → `get_leads()` → count with status != "converted"
- **Revenue MTD** → `get_btc_audit()` → sum confirmed entries for current month
- **Open Work Orders** → `get_maintenance()` → count open tickets
- **STR Reservations** → `get_reservations()` → count upcoming

Show a spinner (`"…"`) while loading, error message on failure.

#### [MODIFY] [portfolio.rs](file:///Users/oply/src/git/orbit_/atlas-platform/apps/folio/src/pages/landlord/portfolio.rs)

Replace placeholder with a table listing portfolios: name, asset count, description, created date. Add "+ New Portfolio" button (create form in Phase 2).

#### [MODIFY] [assets.rs](file:///Users/oply/src/git/orbit_/atlas-platform/apps/folio/src/pages/landlord/assets.rs)

Replace placeholder with a list/grid of assets: name, type badge, status badge, folio number. Group by portfolio.

#### [MODIFY] [leases.rs](file:///Users/oply/src/git/orbit_/atlas-platform/apps/folio/src/pages/landlord/leases.rs)

Replace placeholder with a table: asset ID, tenant, currency, status badge, start/end dates.

#### [MODIFY] [leads.rs](file:///Users/oply/src/git/orbit_/atlas-platform/apps/folio/src/pages/landlord/leads.rs)

Replace placeholder with a list: name, email, status badge, source, created date. Status filter dropdown (new/qualified/converted/disqualified).

#### [MODIFY] [campaigns.rs](file:///Users/oply/src/git/orbit_/atlas-platform/apps/folio/src/pages/landlord/campaigns.rs)
#### [MODIFY] [billing.rs](file:///Users/oply/src/git/orbit_/atlas-platform/apps/folio/src/pages/landlord/billing.rs)
#### [MODIFY] [str_compliance.rs](file:///Users/oply/src/git/orbit_/atlas-platform/apps/folio/src/pages/landlord/str_compliance.rs)
#### [MODIFY] [catalog.rs](file:///Users/oply/src/git/orbit_/atlas-platform/apps/folio/src/pages/landlord/catalog.rs)
#### [MODIFY] [vendors.rs](file:///Users/oply/src/git/orbit_/atlas-platform/apps/folio/src/pages/landlord/vendors.rs)
#### [MODIFY] [reservations.rs](file:///Users/oply/src/git/orbit_/atlas-platform/apps/folio/src/pages/landlord/reservations.rs)

All follow the same pattern: `Resource::new(|| (), |_| get_<domain>())` → `<Suspense>` → render list or empty state.

---

### 4. Tenant Pages

#### [MODIFY] [dashboard.rs](file:///Users/oply/src/git/orbit_/atlas-platform/apps/folio/src/pages/tenant/dashboard.rs)

Show active lease summary (property name, status, next payment due) + open maintenance ticket count.

#### [MODIFY] [my_lease.rs](file:///Users/oply/src/git/orbit_/atlas-platform/apps/folio/src/pages/tenant/my_lease.rs)

Fetch leases for the logged-in tenant (endpoint resolves from session). Show lease detail card: dates, currency, guarantee type, status.

#### [MODIFY] [payments.rs](file:///Users/oply/src/git/orbit_/atlas-platform/apps/folio/src/pages/tenant/payments.rs)
#### [MODIFY] [maintenance.rs](file:///Users/oply/src/git/orbit_/atlas-platform/apps/folio/src/pages/tenant/maintenance.rs)
#### [MODIFY] [reservations.rs](file:///Users/oply/src/git/orbit_/atlas-platform/apps/folio/src/pages/tenant/reservations.rs)

---

### 5. Vendor Pages

#### [MODIFY] [dashboard.rs](file:///Users/oply/src/git/orbit_/atlas-platform/apps/folio/src/pages/vendor/dashboard.rs)

Show open work order count + total unpaid invoice amount.

#### [MODIFY] [work_orders.rs](file:///Users/oply/src/git/orbit_/atlas-platform/apps/folio/src/pages/vendor/work_orders.rs)

List maintenance tickets (filtered to those assigned to this vendor).

#### [MODIFY] [invoices.rs](file:///Users/oply/src/git/orbit_/atlas-platform/apps/folio/src/pages/vendor/invoices.rs)

List billing ledger entries where `payee = current_user`.

---

### 6. Shared API Helper Extraction

#### [MODIFY] [auth.rs](file:///Users/oply/src/git/orbit_/atlas-platform/apps/folio/src/auth.rs)

Make `extract_bearer_token` pub(crate) so it can be used by all API server fns without duplication.

---

## Fetch Pattern (Standard Template)

Every page follows this Leptos pattern:

```rust
use crate::api::lease::get_leases;
use crate::models::lease::LeaseSummary;

#[component]
pub fn Leases() -> impl IntoView {
    let leases = Resource::new(|| (), |_| get_leases());

    view! {
        <div class="page-header">
            <h1 class="page-title">"Leases"</h1>
        </div>
        <Suspense fallback=|| view! { <div class="loading">"Loading…"</div> }>
            {move || leases.get().map(|result| match result {
                Err(e) => view! { <p class="error">{e.to_string()}</p> }.into_any(),
                Ok(items) if items.is_empty() => view! {
                    <div class="empty-state">"No leases found."</div>
                }.into_any(),
                Ok(items) => view! {
                    <div class="data-table">
                        {items.into_iter().map(|l| view! { <LeaseRow lease=l/> }).collect_view()}
                    </div>
                }.into_any(),
            })}
        </Suspense>
    }
}
```

---

## Verification Plan

### Automated

```bash
cargo check --manifest-path atlas-platform/apps/folio/Cargo.toml
```

### Manual

1. Log in as a Landlord, confirm dashboard stat cards show real counts (or 0 with empty DB, not "—").
2. Navigate to `/l/leases` — confirm table renders rows from DB or shows empty state.
3. Navigate to `/l/leads` — confirm lead list with filter dropdown works.
4. Navigate to `/l/portfolio` — confirm portfolio list renders.
5. Log in as a Tenant, confirm `/t` shows active lease summary and maintenance count.
6. Log in as a Vendor, confirm `/v` shows work order count.
7. Verify error states: disconnect DB → all pages show an error message, not a crash.

---

## Open Questions

> [!IMPORTANT]
> **Q1 — Vendor role isolation**: The maintenance endpoint (`GET /api/folio/maintenance`) currently returns all tickets for the tenant, not filtered to a specific vendor. Should vendor-visible work orders be filtered client-side (by assigned_user_id matching session user_id) or should we add a `?assigned_to_me=true` query param to the backend?

> [!NOTE]
> **Q2 — Missing endpoints**: `GET /api/folio/campaigns`, `GET /api/folio/catalog`, `GET /api/folio/vendors`, and `GET /api/folio/str` endpoints exist in the backend handler files but we should confirm their list routes are registered in `mod.rs` before wiring the frontend.
