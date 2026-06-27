# Atlas Platform — Agent Rules

> **Scope:** All code generation, refactoring, and documentation in this workspace.  
> These rules are non-negotiable. If you believe a rule prevents correct implementation, document why explicitly and propose an exception — do not silently violate it.  
> **Start every session by reading [`docs/CURRENT_STATE.md`](../docs/CURRENT_STATE.md).** It is the single most important document for understanding where the platform is today.

---

## 0. Orientation — Read These First

Before writing any code, read in this order:

1. `docs/CURRENT_STATE.md` — ground-truth platform status (generics, services, entities)
2. `docs/folio/folio_vs_network_instance.md` — which app owns which user type
3. `apps/folio/src/components/nav.rs` — `FolioRoute` and `NavIcon` enums; all routes and icons live here
4. `docs/leptos_ssr_shell_pattern.md` — critical SSR gotchas that cause 502s and hydration mismatches
5. `.agents/AGENTS.md` (this file)

---

## 1. Rust Type System — Use It Fully

**Enums over strings for any finite set of values.** This is non-negotiable.

Every finite, bounded set of values MUST be an enum — not `&str`, not `String`, not `i32`.

```rust
// ❌ BANNED
let icon: &'static str = "apartment";
let route: &'static str = "/l/assets";
let status: &str = "active";
let mode: String = "brokerage".to_string();

// ✅ REQUIRED
let icon: NavIcon = NavIcon::Apartment;
let route: FolioRoute = FolioRoute::LandlordAssets;
let status: AssetStatus = AssetStatus::Active;
let mode: FolioMode = FolioMode::Brokerage;
```

**No `_ => {}` wildcard arms on enums you own.** When a new variant is added, the compiler should point to every match arm that needs updating. Wildcards defeat this.

```rust
// ❌ BANNED — new variants added silently pass through
match status {
    AssetStatus::Active => show_green(),
    _ => {}
}

// ✅ REQUIRED — compiler enforces completeness
match status {
    AssetStatus::Active   => show_green(),
    AssetStatus::Inactive => show_gray(),
    AssetStatus::Pending  => show_amber(),
    // Adding AssetStatus::Archived → compile error here → forces a decision
}
```

---

## 2. Zero-Cost Abstractions — `const fn` and `&'static str`

Use `const fn` for methods on enums that return `&'static str`. No runtime allocation.

```rust
// ✅ Correct — zero allocation, resolved at compile time
impl NavIcon {
    pub const fn as_str(self) -> &'static str {
        match self { Self::Apartment => "apartment", ... }
    }
}

// ❌ Wrong — allocates a String unnecessarily
impl NavIcon {
    pub fn to_icon_string(self) -> String {
        "apartment".to_string()
    }
}
```

Implement `std::fmt::Display` by delegating to `as_str()`. Derive `Copy` on enums that contain no heap data — not just `Clone`.

```rust
// ✅ Small descriptive enum — derives Copy
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FolioMode { Standard, Pmc, Brokerage }

// ✅ Data-bearing struct — Clone only, not Copy
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Asset { pub name: String }
```

---

## 3. No Panics on Server Paths

`unwrap()`, `expect()`, and `panic!()` are **banned in all `#[server]` fn bodies and backend handlers**.

```rust
// ❌ BANNED
let val = something.unwrap();
let val = something.expect("should never fail");

// ✅ REQUIRED — propagate with ?
let val = something.map_err(|e| ServerFnError::new(e.to_string()))?;
```

`unwrap()` in `#[cfg(test)]` blocks is acceptable. `unwrap()` on Option in view macros is acceptable only when the value is provably non-None by construction — document why inline.

---

## 4. No Mock Data in Production Code Paths

No hardcoded business data (names, amounts, addresses, UUIDs, counts) anywhere in `apps/folio/src/`, `apps/network-instance/src/`, or `apps/shared-ui/src/`.

```rust
// ❌ BANNED
let items = vec![
    Asset { name: "Maple Blvd".to_string(), revenue: 14400 },
];

// ✅ REQUIRED — Err if endpoint not ready
#[server]
async fn get_assets() -> Result<Vec<Asset>, ServerFnError> {
    Err(ServerFnError::new("not implemented — needs GET /api/folio/assets"))
    // NOT: Ok(vec![Asset { name: "Mock".to_string() ... }])
}
```

Loading states use `<Suspense>`. Error states render explicitly. Mock data belongs only in `#[cfg(test)]` blocks.

---

## 5. Typed Routes — FolioRoute Enum

All navigation, redirects, and `href` attributes MUST use `FolioRoute::Variant.path()`. String literals for internal Folio routes are banned.

```rust
// ❌ BANNED
<a href="/l/assets">
window.location.set_href("/login");

// ✅ REQUIRED
<a href=FolioRoute::LandlordAssets.path()>
window.location.set_href(FolioRoute::Login.path());
```

When adding a route to `app.rs`, simultaneously add it to `FolioRoute` in `nav.rs`. The path string appears in exactly one place.

---

## 6. Typed Icons — NavIcon Enum

All Material Symbols icon names MUST be `NavIcon::Variant.as_str()`. String literals for icon names are banned.

```rust
// ❌ BANNED
<span class="material-symbols-outlined">{"apartment"}</span>

// ✅ REQUIRED
<span class="material-symbols-outlined">{NavIcon::Apartment.as_str()}</span>
```

When a new icon is needed, add it to `NavIcon` first, then use it.

---

## 7. SSR Safety — cfg-Gated Code in Reactive Closures

**Never call a `#[cfg]`-gated function that returns environment-dependent values inside a reactive closure** in an SSR+hydrate app. This causes hydration mismatches that manifest as full page reloads on every button/tab click.

```rust
// ❌ WRONG — SSR returns cluster URL, WASM returns public URL → mismatch → page reload
{move || {
    let url = format!("{}/api/passkeys", get_atlas_api_url()); // cfg-gated!
    view! { <PasskeyLoginButton api_base_url=url /> }
}}

// ✅ CORRECT — compute once at setup, stable across both compile passes
#[cfg(feature = "ssr")]
let passkey_api_base = String::new();         // empty string — never used server-side
#[cfg(not(feature = "ssr"))]
let passkey_api_base = get_atlas_api_url();   // real URL — only used in browser

{move || view! { <PasskeyLoginButton api_base_url=passkey_api_base.clone() /> }}
```

See `docs/leptos_ssr_shell_pattern.md` § "cfg-gated values in reactive closures" for the full explanation.

**Other SSR safety rules:**
- `web_sys::window()`, `document()`, `localStorage` — only in event handlers or `#[cfg(feature = "hydrate")]`
- `Resource::new` reads must be inside `<Suspense>` — or switch to `LocalResource`
- `#[server]` fn parameters must be `Clone + Serialize + Deserialize`

---

## 8. SSR Shell — Required for Every New SSR App

Every new app using `leptos_axum` MUST have a `shell()` function wired into `leptos_routes`. Passing `<App/>` directly to the router is a **silent migration bug** from Leptos 0.7 — the code compiles, but `leptos_meta` panics at runtime with:

```
thread 'tokio-rt-worker' panicked: you are using leptos_meta without a </head> tag
```

Canonical pattern (copy verbatim):
```rust
#[cfg(feature = "ssr")]
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body><App/></body>
        </html>
    }
}
```

See `docs/leptos_ssr_shell_pattern.md` for the full wiring pattern and k8s hash-files configuration.

---

## 9. Deployment — File Hashing (Three Layers Required)

Every Leptos SSR app deployed behind Cloudflare MUST have file hashing enabled in all three layers. Missing any one causes a different failure mode:

| Layer | Setting | Failure if missing |
|---|---|---|
| `Cargo.toml` | `hash-files = true` | Stale bundles served from CDN |
| `Dockerfile` | `ENV LEPTOS_HASH_FILES="true"` | Fails in local Docker only |
| `k8s/base/<app>.yaml` | `env: LEPTOS_HASH_FILES: "true"` | 502 — pod starts but serves dead static HTML |

The k8s `env:` block is mandatory because `envFrom: configMapRef` does NOT preserve `ENV` values baked into the Docker image. See `docs/leptos_ssr_shell_pattern.md` § "CDN Cache Busting".

---

## 10. App Boundary — Folio vs Network Instance vs Platform Admin

**Folio** = authenticated operational workspace. Every logged-in user (landlord, tenant, vendor, PMC, owner, agent, broker) uses Folio.

**Network Instance** = public-facing marketplace. No authenticated workflows. No role-based navigation.

**Platform Admin** = Oplyst super-admin only. CSR, not SSR.

```
If the user needs to be logged in → Folio.
If a search engine needs to index it → Network Instance.
If it's a platform operation (provisioning, billing oversight) → Platform Admin.
```

`folio_mode` is **per-instance, not per-user**: `standard` (default), `pmc`, or `brokerage`. A user's `FolioRole` (landlord, tenant, etc.) is independent of the instance's `folio_mode`. Both affect routing. See `docs/folio/folio_vs_network_instance.md`.

---

## 11. Shared-UI Component Extraction — Three Criteria

A UI pattern belongs in `apps/shared-ui` if and only if it meets **all three**:

1. **Zero domain logic** — no references to `FolioRole`, `LeaseId`, `TenantId`, or any platform entity
2. **Presentational or headless** — no `#[server]` calls, no direct `fetch()`, purely visual or layout
3. **Cross-app usage** — used in at least two separate apps (`folio` + `network-instance`, or `folio` + `platform-admin`, etc.)

When a shared component needs different behaviour per-context, use **prop-controlled visibility**:

```rust
// ✅ One component, two rendering contexts
#[component]
pub fn ListingCard(
    listing: ListingModel,
    #[prop(optional)] show_agent_controls: bool, // Folio passes true; NI passes nothing
) -> impl IntoView { ... }
```

Components that are Folio-specific (auth shells, nav layouts, role-scoped panels) stay in `apps/folio/src/`. Never extract a layout shell to shared-ui.

---

## 12. Multi-Tenant Data — tenant_id Is Always Required

Every database entity is scoped to a `tenant_id`. Queries that span tenant boundaries are bugs.

Backend handlers must:
- Extract `tenant_id` from the authenticated session (not from query params or request body)
- Include `tenant_id` in every WHERE clause that touches tenant-scoped tables
- Never accept a `tenant_id` override from the client

```rust
// ❌ BANNED — client-supplied tenant_id
async fn get_assets(tenant_id: Uuid) -> Result<Vec<Asset>, ServerFnError> { ... }

// ✅ REQUIRED — tenant_id from authenticated session only
async fn get_assets(session: &SessionInfo) -> Result<Vec<Asset>, ServerFnError> {
    let tenant_id = session.tenant_id.ok_or_else(|| ServerFnError::new("no tenant"))?;
    // ...
}
```

---

## 13. Design Tokens — No Hardcoded Colors

No hardcoded color values in Rust view macros or CSS. All colors come from the token system in `apps/folio/tailwind.config.js`.

Banned patterns (stitch artifacts — must not enter production):
```rust
// ❌ ALL BANNED
<div style="background: #131b2e; color: white;">
class="bg-slate-100"          // use surface-container-low
class="text-slate-700"        // use text-on-surface-variant
class="bg-white"              // use bg-surface-container-lowest
class="bg-black"              // use bg-primary
```

Token reference: `docs/folio/design_preview.html` (open in browser, no build step).

---

## 14. Role → Config via FolioRole, Not Imports

Layouts derive nav config from the session's `FolioRole` enum, not by importing specific statics.

```rust
// ❌ BANNED — import couples layout to a specific config static
use crate::components::nav::LANDLORD_NAV;
<SidebarNav config=&LANDLORD_NAV/>

// ✅ REQUIRED — role IS the index; adding a FolioRole variant forces nav config creation
let config = session.folio_role.nav_config();
<SidebarNav config=config/>
```

---

## 15. Platform Generics — G01–G34+ Are the Source of Truth

The platform's backend is organized around **Platform Generics** (G01–G34+). When implementing a frontend page, always map its data requirements to the correct generic first.

| Domain | Generics | Key service |
|---|---|---|
| Portfolios/assets | G09, G10 | `PortfolioService`, `AssetService` |
| Contracts/leases | G11 | `ContractService` |
| Vendors | G12 | `ServiceProviderService` |
| Cases/maintenance | G13 | `CaseService` |
| Opportunities/leads | G15 | `OpportunityService` |
| Campaigns | G19 | `pm/campaign.rs` |
| Billing/ledger | G03 | `LedgerService` |
| Syndication | G05 | `SyndicationEventBus` |
| Regulatory/STR | G16 | `RegulatoryRegistrationService` |
| Analytics (Meridian) | G27 | `MeridianService` |
| Reservations | G26 | `ReservationService` |

Use `grep -r "router.route" backend/src/handlers/folio/` to find the exact endpoint string for any generic's Folio-facing API. **Do not invent new endpoints if one already exists for the generic.**

---

## 16. Explicit Error Types in Internal Boundaries

Public server functions return `Result<T, ServerFnError>` (the wire type). Internal module-to-module boundaries should use typed error enums, not strings.

```rust
// ✅ Preferred for internal APIs
#[derive(Debug, thiserror::Error)]
pub enum AssetError {
    #[error("asset {0} not found")]
    NotFound(Uuid),
    #[error("permission denied")]
    Forbidden,
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),
}
```

---

## 17. The One-Place Rule

Every piece of information that could drift has exactly one source of truth. Do not duplicate it.

| Information | Single source |
|---|---|
| Route path strings | `FolioRoute::path()` in `apps/folio/src/components/nav.rs` |
| Icon name strings | `NavIcon::as_str()` in `apps/folio/src/components/nav.rs` |
| Nav items per role | `*_NAV` statics in `apps/folio/src/components/nav.rs` |
| Role → nav config | `FolioRole::nav_config()` |
| Role → home path | `FolioRole::home_path()` in `apps/folio/src/auth.rs` |
| Design color tokens | `apps/folio/tailwind.config.js` |
| Platform state | `docs/CURRENT_STATE.md` |
| Page implementation queue | `docs/folio/page_queue.md` |
| App boundary decisions | `docs/folio/folio_vs_network_instance.md` |
| Backend routes | `backend/src/handlers/folio/*.rs` router definitions |

If you find information duplicated across two files, consolidate before adding more.

---

## 18. Operator-Facing UI — No Internal Generic Codes

The operator-facing UI (all Folio portals) must never expose internal platform identifiers:

```
// ❌ BANNED in any label, tab title, or description visible to operators
"G-05 Syndication", "G-27 Meridian", "G-13 Cases"

// ✅ REQUIRED — use the human-readable product name
"Syndication", "Analytics", "Maintenance & Cases"
```

Generic codes (G-01 through G-34+) are internal engineering identifiers. They belong in code comments, database migration names, and internal docs — never in rendered UI.

---

## 19. Leptos 0.8+ — Mandatory API Version

All frontend apps on this platform use **Leptos 0.8** (pinned in `Cargo.toml`). Do not write Leptos 0.6 or 0.7 idioms — many are API-breaking changes that compile with outdated training knowledge but fail at runtime or produce wrong behavior.

### Breaking differences from 0.7 that agents commonly get wrong

**Signals — use `ReadSignal` / `RwSignal` from `leptos::prelude::*`**
```rust
// ❌ 0.7 API — does not exist in 0.8
let (count, set_count) = create_signal(0);

// ✅ 0.8 API
let count = RwSignal::new(0);
let (read, write) = signal(0);   // tuple form still works
```

**Resources — `Resource::new` vs `LocalResource::new`**
```rust
// ✅ Resource::new — SSR-compatible, runs on server AND client
// MUST be inside <Suspense> when read
let data = Resource::new(|| (), |_| async move { fetch_data().await });

// ✅ LocalResource::new — WASM-only, skips SSR pass
// Use when data is not needed for initial HTML (e.g. after user interaction)
let local = LocalResource::new(|| async move { client_only_fetch().await });

// Rule: if the data should be in the SSR-rendered HTML → Resource
//       if the data only matters after hydration → LocalResource
```

**Router — `path!()` macro + typed params**
```rust
// ✅ 0.8 — use path!() macro in Leptos routes
<Route path=path!("/l/assets/:id") view=AssetDetail/>

// ✅ Extracting params in the page component
let params = use_params_map();
let id = params.get().get("id").unwrap_or_default();

// ❌ Axum route syntax in Leptos path — does NOT work
<Route path="/l/assets/{id}" view=AssetDetail/>  // {id} is Axum syntax
```

**`IntoView` — no more `view!` inside `if` without `.into_any()`**
```rust
// ✅ 0.8 — different branches must have the same type
{move || match result {
    Ok(data) => view! { <DataView data=data/> }.into_any(),
    Err(_)   => view! { <ErrorState/> }.into_any(),
}}

// ❌ Will not compile — branches have different concrete types
{move || if loading { view! { <Spinner/> } else { view! { <Content/> } }}
```

**`use_navigate` — explicit navigation**
```rust
// ✅ 0.8
let navigate = use_navigate();
navigate(FolioRoute::LandlordDashboard.path(), Default::default());

// ❌ Do not use window.location for internal SPA navigation (bypasses router)
web_sys::window().unwrap().location().set_href("/l");
```

**`#[server]` — endpoint path is `/api` not `/server_fns`**
```rust
// ✅ Correct endpoint prefix for this platform
#[server(GetAssets, "/api")]
pub async fn get_assets() -> Result<Vec<Asset>, ServerFnError> { ... }
```

**`collect_view()` — required for iterators in `view!`**
```rust
// ✅ 0.8
items.iter().map(|item| view! { <Row item=item.clone()/> }).collect_view()

// ❌ Calling .collect::<Vec<_>>() inside view! does not work correctly
```

**Reactive closures — `move ||` not `|| move`**
```rust
// ✅
on:click=move |_| { /* captures outer vars by move */ }

// ❌ Wrong order
on:click=|| move |_| { }
```

### Pinned versions (as of June 2026)

| Crate | Version | Location |
|---|---|---|
| `leptos` | `0.8` (resolves to 0.8.x latest) | `apps/folio/Cargo.toml` |
| `leptos` | `0.8.17` | `apps/shared-ui/Cargo.toml` |
| `leptos_axum` | `0.8` | `apps/folio/Cargo.toml` |
| `leptos_router` | `0.8` | `apps/folio/Cargo.toml` |
| `leptos_meta` | `0.8` | `apps/folio/Cargo.toml` |

When in doubt about the 0.8 API for a specific pattern, consult `docs/leptos_ssr_shell_pattern.md` or look at an existing working page in `apps/folio/src/pages/` before writing from training knowledge.

---

## Exceptions Policy


A deviation from these rules is permitted only if:

1. The Leptos or Rust compiler makes the typed approach impossible in this specific context (document the compiler constraint), AND
2. The deviation is marked with `// EXCEPTION: <reason>` inline, AND
3. A `// TODO: fix when <condition>` comment documents the path back to compliance

"It's faster to write as a string" is **not** a valid exception.  
"The stitch prototype does it this way" is **not** a valid exception.  
"It's just a prototype" is **not** a valid exception — there are no prototypes that don't eventually ship.
