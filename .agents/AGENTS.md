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

`unwrap()`, `expect()`, `panic!()`, and `todo!()` are **banned in all `#[server]` fn bodies and backend handlers**. No exceptions.

```rust
// ❌ ALL BANNED in server/handler paths
let val = something.unwrap();
let val = something.expect("should never fail");
todo!()         // crashes the server process on that code path
unreachable!()  // same — if it's truly unreachable, prove it with the type system instead

// ✅ REQUIRED — propagate with ?
let val = something.map_err(|e| ServerFnError::new(e.to_string()))?;
```

`unwrap()` and `todo!()` in `#[cfg(test)]` blocks are acceptable. `unwrap()` on `Option` in view macros is acceptable only when the value is provably non-None by construction — comment why inline.

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

## 20. Security — Resource-Level Authorization (IDOR Prevention)

**Rule 12 (tenant_id from session) is necessary but not sufficient.** An attacker who has a valid session for tenant A can still probe resource IDs that belong to tenant B. Every individual resource fetch must verify that the requested resource belongs to the authenticated tenant.

```rust
// ❌ CRITICAL BUG — tenant_id is in session, but the query doesn't filter by it.
// Any valid session can read any asset UUID by guessing or enumerating IDs.
async fn get_asset(session: SessionInfo, asset_id: Uuid) -> Result<Asset, ServerFnError> {
    db.find_asset_by_id(asset_id).await  // returns ANY asset — IDOR vulnerability
}

// ✅ REQUIRED — filter by BOTH resource ID AND tenant_id in the same query
async fn get_asset(session: SessionInfo, asset_id: Uuid) -> Result<Asset, ServerFnError> {
    let tenant_id = session.tenant_id.ok_or_else(|| ServerFnError::new("no tenant"))?;
    db.find_asset_for_tenant(asset_id, tenant_id)
        .await?
        .ok_or_else(|| ServerFnError::new("not found"))
        // If asset_id belongs to another tenant, query returns None → 404.
        // The caller cannot distinguish "doesn't exist" from "not yours".
}
```

The SQL pattern must be: `WHERE id = $1 AND tenant_id = $2`. Never `WHERE id = $1` followed by an in-memory ownership check.

---

## 21. Security — Server Function Input Validation

Every `#[server]` fn parameter that accepts user-supplied data must be validated before use. The client controls all inputs to server functions — assume adversarial input.

```rust
// ❌ DANGEROUS — no validation; attacker can send negative amounts, empty strings,
// SQL-injection via ORM edge cases, or IDs pointing to other tenants' data
#[server]
async fn create_payment(amount: f64, note: String) -> Result<(), ServerFnError> {
    db.insert_payment(amount, note).await
}

// ✅ REQUIRED — validate at the server fn boundary before any DB or service call
#[server]
async fn create_payment(
    amount_cents: i64,
    note: String,
) -> Result<(), ServerFnError> {
    // Validate: amount must be positive
    if amount_cents <= 0 {
        return Err(ServerFnError::new("amount must be positive"));
    }
    // Validate: note length (prevent DB column overflow and log spam)
    if note.len() > 500 {
        return Err(ServerFnError::new("note too long"));
    }
    // Validate: strip any control characters from text fields
    let note = note.trim().to_string();
    db.insert_payment(amount_cents, note).await
        .map_err(|e| ServerFnError::new(e.to_string()))
}
```

Minimum validations for every server fn:
- Numeric ranges (positive amounts, valid page sizes, max pagination limits)
- String lengths (prevents column overflow and memory exhaustion)
- UUID format (sqlx handles this, but validate before expensive joins)
- Enum membership (already enforced if you use typed enums per Rule 1)

---

## 22. Security — No Secrets in the Frontend

Session tokens, API keys, and any credentials must **never** appear in:
- Leptos signals or `RwSignal` values
- `provide_context` values that propagate to WASM
- `LocalStorage` or `SessionStorage`
- URL query parameters
- Any value rendered into the DOM

```rust
// ❌ BANNED — session token in a signal is visible in DevTools WASM heap inspection
let token = RwSignal::new(get_session_token());

// ❌ BANNED — token in localStorage is accessible to any JS (XSS exposure)
web_sys::window().unwrap()
    .local_storage().unwrap().unwrap()
    .set_item("token", &token).unwrap();

// ✅ REQUIRED — tokens live in httpOnly, SameSite=Strict cookies set by the server
// The cookie is invisible to JavaScript and inaccessible to WASM.
// The frontend never touches the token directly — it's sent automatically with every request.
```

The Atlas backend already uses `atlas_session` as an httpOnly cookie. Do not add any client-side token storage. If a component needs to know "is the user logged in?", it calls `check_session()` — it does not read a stored token.

**Environment variables with secrets** must only appear in server-gated code:
```rust
// ✅ Only accessible on the server — never compiled into the WASM bundle
#[cfg(feature = "ssr")]
fn get_api_key() -> String {
    std::env::var("ATLAS_API_KEY").expect("ATLAS_API_KEY not set")
}
```

---

## 23. Security — No Raw HTML Injection (`inner_html`)

Leptos `view!` macro prevents XSS by design — all string values are escaped. **Never bypass this** using raw DOM manipulation.

```rust
// ❌ BANNED — bypasses Leptos escaping; attacker-controlled content becomes executable JS
let user_content = get_user_bio(); // e.g. "<script>steal_cookies()</script>"
element.set_inner_html(&user_content); // XSS

// ❌ BANNED — same problem via web_sys
div_ref.get().unwrap().set_inner_html(&markdown_output);

// ✅ REQUIRED — use Leptos typed view for user content (auto-escaped)
view! { <p>{user_content}</p> }

// ✅ For Markdown: sanitize FIRST with a whitelist-based sanitizer, then use inner_html
// only with the sanitized output. Document the sanitizer used inline.
// EXCEPTION requires explicit security review.
```

If you need to render Markdown or rich text (e.g., property descriptions), sanitize with a whitelist-based HTML sanitizer **on the server** before the data ever reaches the frontend. Raw unsanitized HTML from any user-controlled source into the DOM is an unconditional ban.

---

## 24. Financial Arithmetic — No `f64` for Money

Floating-point arithmetic is **banned for all monetary values**. IEEE 754 `f64` cannot represent most decimal fractions exactly, causing rounding errors that compound over transactions.

```rust
// ❌ BANNED — f64 loses precision; $0.10 + $0.20 ≠ $0.30 in IEEE 754
let rent: f64 = 1450.00;
let fee: f64 = 0.10 * rent;  // may be 144.99999999999997

// ✅ REQUIRED — integer cents (no fractions lost)
let rent_cents: i64 = 145000;   // $1,450.00
let fee_cents: i64 = rent_cents / 10;

// ✅ ALSO ACCEPTABLE — rust_decimal crate for display/formatting
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
let rent = Decimal::new(145000, 2);  // $1,450.00

// ✅ REQUIRED for arithmetic on integers — use checked math to prevent overflow
let total = rent_cents.checked_add(fee_cents)
    .ok_or_else(|| ServerFnError::new("arithmetic overflow"))?;
```

The database stores monetary values as `BIGINT` (cents) or `NUMERIC(19,4)`. The frontend receives cents and formats for display. Never accept `f64` from a server fn for an amount field — use `i64` (cents) or `Decimal`.

---

## 25. Code Quality — Clippy Is Law

`cargo clippy -- -D warnings` must pass with zero warnings. Clippy warnings are bugs waiting to happen — treat them as errors.

```bash
# Run before every commit
cargo clippy --workspace --all-features -- -D warnings
```

`#[allow(clippy::some_lint)]` suppressions require an inline justification comment:

```rust
// ❌ BANNED — silent suppression
#[allow(clippy::too_many_arguments)]
fn process(...) { }

// ✅ REQUIRED — document why
// CLIPPY-ALLOW: This function coordinates 8 independent subsystem configs.
// Refactoring into a builder would add 200 lines for no behavioral improvement.
// Filed: TODO track in issue #NNN
#[allow(clippy::too_many_arguments)]
fn process(...) { }
```

The most important Clippy lints for this codebase:

| Lint | Why it matters here |
|---|---|
| `clippy::unwrap_used` | Enforces Rule 3 — no panics on server paths |
| `clippy::expect_used` | Same |
| `clippy::panic` | Same |
| `clippy::float_arithmetic` | Enforces Rule 24 — no f64 money |
| `clippy::arithmetic_side_effects` | Catches unchecked integer overflow in financial code |
| `clippy::clone_on_ref_ptr` | Prevents accidental Arc clone explosions |
| `clippy::large_futures` | Catches futures that will overflow the async stack |
| `clippy::unsafe_code` | Enforces Rule 31 — unsafe is presumed banned |

---

## 26. Database Migrations — Naming and Scope

Migration files follow the naming convention established in the codebase:

```
m{YYYYMMDD}_{short_snake_case_description}
```

Examples from the codebase:
```
m20260909_folio_instance_mode
m20260910_brokerage_portals
m20260911_asset_listing_mode
```

Rules:
- **One logical change per migration** — do not bundle unrelated schema changes
- **Never edit a migration that has run in UAT or PROD** — always add a new migration
- **No data mutations in schema migrations** — data backfills are separate migrations named `m{YYYYMMDD}_backfill_{description}`
- **All migrations are reversible** — implement the `Down` side even if it's `// not reversible — document why`
- **Foreign key additions must be preceded by index creation** in the same or a prior migration to avoid full-table scans on `JOIN`

```rust
// ❌ BANNED — editing an existing migration that has already run
// (will corrupt UAT/PROD schema state)

// ✅ REQUIRED — new migration for any change
// m20261001_add_folio_str_host_mode.rs — adds the new column/constraint
```

## 30. Unit Tests — Real Types, Realistic Scenarios, Regression Guards

Unit tests are the safety net that confirms the app works as operators will actually use it. Tests written against fake types or invented strings give false confidence — they pass when the enum variant is renamed or the status slug changes, which is exactly when you need them to fail.

### Three categories — all required when adding a new type or feature

**1. Enum roundtrip tests** — every `Display`/`TryFrom` pair must be exhaustively covered

Every enum variant must be in the test. No wildcard shortcuts. If a new variant is added and the test doesn't include it, the test should fail or the author should consciously add it.

```rust
// ✅ From backend/src/tests/unit/type_system_unit_tests.rs — the established pattern
#[test]
fn asset_status_display_roundtrip() {
    // ALL variants listed — not "a few representative ones"
    let variants = [
        (AssetStatus::Active,   "active"),
        (AssetStatus::Inactive, "inactive"),
        (AssetStatus::Pending,  "pending"),
        (AssetStatus::Archived, "archived"),
    ];
    for (variant, slug) in &variants {
        assert_eq!(variant.to_string(), *slug, "Display mismatch for {variant:?}");
        let parsed = AssetStatus::try_from(slug.to_string())
            .expect("TryFrom should succeed for valid slug");
        assert_eq!(&parsed, variant, "TryFrom roundtrip failed for '{slug}'");
    }
}

#[test]
fn asset_status_invalid_slug_returns_err() {
    assert!(AssetStatus::try_from("ACTIVE".to_string()).is_err(), "must be lowercase");
    assert!(AssetStatus::try_from("".to_string()).is_err());
    assert!(AssetStatus::try_from("unknown".to_string()).is_err());
}
```

**2. State machine / business rule tests** — terminal states, allowed transitions, computed properties

```rust
// ✅ Tests mirror what operators actually care about
#[test]
fn lease_status_terminal_states() {
    // Operators cannot take action on terminated leases — regression guard
    assert!(LeaseStatus::Expired.is_terminal());
    assert!(LeaseStatus::Terminated.is_terminal());
    assert!(!LeaseStatus::Active.is_terminal());
    assert!(!LeaseStatus::PendingRenewal.is_terminal());
}

#[test]
fn case_priority_ordering() {
    // Vendors sort by priority — ordering must be stable
    assert!(CasePriority::Emergency > CasePriority::High);
    assert!(CasePriority::High > CasePriority::Medium);
    assert!(CasePriority::Medium > CasePriority::Low);
}
```

**3. Regression guard tests** — label and comment the specific bug or security property being protected

Regression guards use the `T<N> REGRESSION GUARD:` comment pattern established in `session_unit_tests.rs`. They protect a specific, named invariant and explain *why* it must hold.

```rust
// T5 REGRESSION GUARD: asset_id must be scoped to tenant_id in the same query.
// GET /api/folio/assets/:id must return 404 — not 403 — when the asset belongs
// to a different tenant, preventing enumeration of valid UUIDs.
// See AGENTS.md Rule 20 (IDOR prevention).
#[test]
fn asset_query_requires_tenant_scope() {
    let asset_tenant_id = Uuid::new_v4();
    let requesting_tenant_id = Uuid::new_v4(); // different tenant
    let result = filter_asset_for_tenant(some_asset(asset_tenant_id), requesting_tenant_id);
    assert!(result.is_none(), "cross-tenant asset access must return None, not the asset");
}
```

### Rules that apply to every test in this codebase

**Use real app types — never invent test-only stringly-typed data:**
```rust
// ❌ BANNED — if AssetStatus variants are renamed, this test still passes
let status: &str = "active";
assert_eq!(status, "active"); // proves nothing

// ✅ REQUIRED — compiler catches renames
let status = AssetStatus::Active;
assert_eq!(status.to_string(), "active");
```

**Use realistic field values — names, amounts, and IDs that reflect actual operator data:**
```rust
// ❌ BANNED — foo/bar/baz test data
let asset = Asset { name: "foo".to_string(), rent_cents: 0, tenant_id: Uuid::nil() };

// ✅ REQUIRED — values that reflect real operator usage
let asset = Asset {
    name: "Maple Blvd Unit 4B".to_string(), // plausible property name
    rent_cents: 195000,                      // $1,950/mo — realistic rent
    tenant_id: Uuid::new_v4(),               // random but valid UUID
    status: AssetStatus::Active,             // real enum variant, not "active"
};
```

**No `unwrap()` in tests that test error paths — use `assert!(result.is_err())`:**
```rust
// ❌ BANNED — panics instead of failing the test cleanly
let result = AssetStatus::try_from("bad".to_string()).unwrap_err();

// ✅ REQUIRED
assert!(AssetStatus::try_from("bad".to_string()).is_err());
```

**Test file location:**
- Backend types and services → `backend/src/tests/unit/[domain]_unit_tests.rs`
- Folio server functions → inline `#[cfg(test)] mod tests { ... }` at the bottom of the page file
- Shared-ui components → inline in the component file (logic only — no DOM rendering in unit tests)

**Test naming:** `snake_case` describing what it asserts, not what it calls. `fn lease_expired_status_is_terminal()` not `fn test_lease_status()`.

---

## 31. No `unsafe` Code — Strong Justification Required

`unsafe` is **presumed banned**. In 99% of cases there is a safe alternative; find it first.

The bar for an `unsafe` block is **high** — all four of the following must be true before writing one:

1. **The safe API cannot express this** — a specific Rust or library limitation prevents the safe equivalent (document the exact limitation, not just "it's faster").
2. **The soundness invariant is provable** — the SAFETY comment must state the concrete invariant that makes the code sound, not just "this should be fine".
3. **The scope is minimal** — the `unsafe` block is as small as possible; surrounding code is safe.
4. **An alternative was evaluated and rejected** — name the safe alternative and explain why it was ruled out.

```rust
// ❌ BANNED — no justification, no invariant, no alternative considered
unsafe { std::env::set_var("KEY", "val"); }

// ❌ BANNED — vague SAFETY comment
// SAFETY: this is fine in practice
unsafe { ptr.write(value); }

// ✅ REQUIRED — all four criteria met
// SAFETY: We hold the process-global env mutex (via `temp_env`) for the
// duration of this call, guaranteeing no concurrent set_var from another
// thread. Alternative: `temp_env::with_var` (safe wrapper) — used
// everywhere else; this path requires the raw call because [specific reason].
unsafe { std::env::set_var("KEY", "val"); }
```

### Common safe alternatives (reach for these first)

| Instead of | Use |
|---|---|
| `unsafe { std::env::set_var(...) }` in tests | `temp_env::with_var(...)` (dev-dep in `backend/Cargo.toml`) |
| `unsafe { ptr.as_ref().unwrap_unchecked() }` | `.expect("invariant: ...")` or restructure to prove non-null via types |
| `unsafe { std::mem::transmute(...) }` | `bytemuck::cast` / `pod_read_unaligned` / rethink the type boundary |
| `unsafe { slice::from_raw_parts(...) }` | `&[T]` slice from a typed allocation; use `Vec` or `Box<[T]>` |
| `unsafe impl Send for Foo` | Restructure `Foo` to use `Arc<Mutex<...>>` or `Send`-safe primitives |

### In tests specifically

`unsafe` in `#[cfg(test)]` is **also banned** for env-var mutation. The `temp-env` crate is already a dev-dependency for exactly this purpose. Do not add new `unsafe { std::env::set_var/remove_var }` calls to any test — use `temp_env::with_var` / `with_var_unset` for sync tests and `with_smtp_blanked`-style helpers for async tests.

### Enforcement

`clippy::unsafe_code` is in the project's Clippy deny-list (Rule 25). Any new `unsafe` block will produce a warning that is treated as a build error unless suppressed with `#[allow(unsafe_code)]`. Such a suppression requires the four-criteria comment above and **must be reviewed before merge**.

---

## 29. CURRENT_STATE.md — Remind When It's Time to Refresh

`docs/CURRENT_STATE.md` is the ground-truth registry of platform implementation status. The user has an automated workflow (`/update-current-state`) to keep it accurate — **do not run it yourself**.

Your job is to surface a reminder at the right moment, then stop. The decision to run it belongs to the user.

**Remind the user to run `/update-current-state` when:**
- 5 or more pages have been implemented since the last reminder
- You are about to read CURRENT_STATE.md and notice its "Last modified" date is more than 2 weeks old
- A new generic has just been proposed or confirmed (Rule 27)
- The user asks a question whose answer depends on platform implementation status

**Format — one line, at the end of your response:**

```
💡 Good time to run `/update-current-state` — [X] pages landed since last refresh.
```

That's it. Do not explain what the workflow does. Do not offer to run it. Do not repeat the reminder in the same session unless the user explicitly asks about platform status again.

---

## 27. Proactive Generic Discovery

The platform's backend is built around **Platform Generics** (G01–G34+) — reusable, service-backed entities that power multiple roles and apps. When implementing a new page or feature, actively evaluate whether the data it needs maps to an existing generic or represents a gap that should become one.

**Do this check before writing a single line of implementation:**

Read `docs/CURRENT_STATE.md` Generics Registry. If the data pattern on the page doesn't appear there, score it against these five criteria:

| Criterion | Question |
|---|---|
| **Own entity** | Does it need its own database table (not just a column on an existing table)? |
| **Lifecycle** | Does it have statuses, state transitions, or a workflow? (created → active → closed) |
| **Cross-role usage** | Is it consumed by more than one user role? (e.g., both landlord AND tenant see it) |
| **Cross-app potential** | Could this data appear in both Folio AND Network Instance or Platform Admin? |
| **Service boundary** | Does it need its own service methods independent of other entities? |

**Score 3 or more → flag as a potential new generic before implementing.**

### How to surface the finding (non-blocking)

Do not stop work to ask permission. Complete the analysis, then append a callout at the **end** of your implementation output — after the code, not before:

```
---
## 🔍 Potential New Generic Identified

**Data pattern:** Inspection Reports
**Seen in:** `/l/inspections` stitch design (`l_inspections/code.html`)
**Criteria met:** Own entity ✅ | Lifecycle ✅ | Cross-role (landlord + vendor) ✅ | Service boundary ✅
**Score:** 4/5 — recommend new generic

**Proposed:** `atlas_inspections` (G-35?)
- Tables: `atlas_inspections`, `atlas_inspection_items`
- Service: `InspectionService`
- Roles that consume it: Landlord (/l/inspections), Vendor (/v/work-orders detail), Tenant (read-only)
- Similar pattern to: G-13 `atlas_cases` (has items, statuses, assignments)

**Recommendation:** Define this generic in a backend migration + service before wiring the Folio page.
This prevents the page from being implemented against a one-off schema that can't be reused.

**Action needed:** Confirm generic definition or explain why this is domain-specific.
```

### When NOT to propose a new generic

- The data is a **property/attribute of an existing entity** (e.g., adding a `furnished: bool` column to `atlas_assets` is not a new generic)
- The feature is **only ever used by one role** in one portal with no plausible cross-app future (e.g., a very specific broker-only commission split formula)
- The data is **configuration, not operational** (e.g., a setting stored in `atlas_app_deployment_config`)
- An existing generic already handles it and just needs a new endpoint (check the handler files first)

When in doubt, propose it — a false positive costs one conversation; a missed generic costs a refactor.

---

## 28. AGENTS.md Is a Living Document — Propose Amendments

This file must evolve with the codebase. When a conversation reveals a pattern, antipattern, security issue, architectural decision, or Leptos quirk that isn't captured here, **propose an amendment at the end of your response**.

### Four triggers that should always produce a proposal

1. **A new antipattern discovered in the codebase** — you find code that violates a rule not yet written (e.g., a new category of mock data, a new SSR footgun, a new Leptos 0.8 API misuse)
2. **An architectural decision made during the conversation** — you and the user settled on something that will affect future implementations (e.g., "STR Host is a distinct FolioRole, not a sub-view of Landlord")
3. **A rule that turns out to be incomplete or ambiguous** — implementation reveals an edge case the rule didn't cover
4. **A security or quality concern surfaced during code review** — something you catch in existing code that isn't in the rules yet

### Format for proposals (append to end of response, never interrupt the primary task)

```markdown
---
## 📋 Proposed AGENTS.md Amendment

**Rule:** [New Rule N] or [Amendment to Rule N]
**Trigger:** [Which of the 4 triggers above]
**Context:** [One sentence — what in this conversation revealed the gap]

**Proposed addition:**
> [The rule text, written in the same style as existing rules — with ✅/❌ code examples if applicable]

**Confidence:** [High / Medium — High means "this is clearly missing", Medium means "worth discussing"]

Respond "add it" to commit, or tell me to adjust.
```

### What NOT to propose

- Trivial style preferences with no quality or safety impact
- Rules that duplicate existing rules with minor rewording
- Changes that conflict with the Exceptions Policy without an override rationale
- Proposals triggered by a one-off quirk that is unlikely to recur

### Keeping the file current

When a proposal is approved, update AGENTS.md and commit immediately with:
```bash
git add .agents/AGENTS.md && \
git commit -m "docs(agents): Rule N — <short description>

<what triggered the addition and why it matters>"
```

The AGENTS.md commit history is the audit trail for how quality standards evolved. Every addition should be traceable to a specific conversation, decision, or discovered antipattern.

---

## Exceptions Policy

A deviation from these rules is permitted only if:

1. The Leptos or Rust compiler makes the typed approach impossible in this specific context (document the compiler constraint), AND
2. A **type-system solution** was genuinely evaluated and ruled out (explain why), AND
3. The deviation is marked with `// EXCEPTION: <reason>` inline, AND
4. A `// TODO: fix when <condition>` comment documents the path back to compliance

"It's faster to write as a string" is **not** a valid exception.  
"The stitch prototype does it this way" is **not** a valid exception.  
"It's just a prototype" is **not** a valid exception — there are no prototypes that don't eventually ship.  
"The endpoint doesn't exist yet" is **not** a valid exception for mock data — return `Err(ServerFnError::new("not implemented"))` instead.



---

## Server Access & Remote Debugging

### NEVER ask the user for SSH credentials, server IPs, or hostnames.

Always derive the server address from shell history — never hardcode it in this file:

```bash
export SERVER=$(cat ~/.zsh_history | grep -oE "root@[0-9.]+" | tail -1)
```

### Standard debugging commands

```bash
# Resolve server address first
export SERVER=$(cat ~/.zsh_history | grep -oE "root@[0-9.]+" | tail -1)

# Backend errors — last 2h
ssh -o StrictHostKeyChecking=no $SERVER \
  'kubectl logs deployment/backend -n atlas-dev --since=2h 2>&1 | grep -E "ERROR|WARN|422|500|400|panic" | grep -v "outbox\|sqlx\|webhook\|syndication" | tail -80'

# Provision-specific
ssh -o StrictHostKeyChecking=no $SERVER \
  'kubectl logs deployment/backend -n atlas-dev --since=3h 2>&1 | grep -iE "provision|internal_operator|anchor"'

# Pod status
ssh -o StrictHostKeyChecking=no $SERVER 'kubectl get pods -n atlas-dev'

# DB query (dev database is atlas_dev, NOT ruud)
ssh -o StrictHostKeyChecking=no $SERVER 'sudo -u postgres psql -d atlas_dev -c "<QUERY>"'
```

### Namespaces
- `atlas-dev` — dev environment (`dev.atlas.oply.co`)
- `atlas-platform` — UAT / legacy environment


---

## Infrastructure & Server Configuration

All server, cluster, and NixOS configuration lives in:

```
/Users/oply/src/git/orbit_/NixForge/
```

Key files:
- `flake.nix` — NixOS system configuration (nginx SNI routing, k3s, PostgreSQL, Woodpecker CI, Grafana/Loki/Prometheus)
- `darwin-configuration.nix` — local macOS dev machine config
- `docs/` — architecture decisions, k3s-nginx SNI routing, Woodpecker/Dagger pipeline docs
- `secrets/` — secret management (do not hardcode)

### Domain routing architecture
- Host-level nginx uses `ssl_preread` (Layer 4) to route known domains (e.g. `ci.oply.co`) to local services
- All other traffic is TCP-proxied transparently to the k3s ingress controller
- Wildcard cert for `*.dev.atlas.oply.co` is provisioned via cert-manager inside k3s
- New internal instance domains MUST use `{slug}.dev.atlas.oply.co` to be covered by the wildcard cert
- Custom domains require a new Ingress manifest + cert-manager Certificate resource in k3s

---

## 32. CI/CD Test Database — Integration Tests Are Never "Blocked on DB"

The Woodpecker CI pipeline (`/.woodpecker.yml`) spins up a **Postgres 15 service container** on every push to `dev` or `uat`. There is no manual DB setup step required for integration tests.

```yaml
# From .woodpecker.yml — always present
services:
  database:
    image: postgres:15
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: oplydbtest

steps:
  test_workspace:
    environment:
      TEST_DATABASE_URL: postgres://postgres:postgres@database:5432/oplydbtest
    commands:
      - cd backend && cargo test --workspace -j $(nproc)
```

### What this means for agents

- **Do not treat integration tests as "future work" or "blocked."** The test harness is always available in CI. Write integration tests alongside the feature in the same PR.
- The `setup_test_app()` function in `backend/src/tests/api_tests.rs` already handles the three connection URLs — local (`:5432`), Docker Compose (`:5433`), and Woodpecker (`database:5432`) — in priority order. It will connect correctly in all environments.
- **Unit tests** (pure logic, no DB) go in `backend/src/tests/unit/` and run fast locally with no setup.
- **Integration tests** (DB + HTTP) go in `backend/src/tests/` and run in CI automatically.
- Both types belong in the same PR as the feature. Separating them into "later" PRs means they never get written.

### Test file registration

- Unit tests: register in `backend/src/tests/unit/mod.rs`
- Integration tests: register in `backend/src/tests/mod.rs`

### Running tests locally

```bash
# Unit tests only (no DB needed)
cd backend && cargo test unit

# Specific integration test file
cd backend && cargo test waitlist_integration_tests

# Full suite (requires local Postgres at localhost:5432 or :5433)
cd backend && cargo test --workspace
```
