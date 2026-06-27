# Atlas Platform — Agent Rules

> **Scope:** All code generation, refactoring, and documentation in this workspace.  
> These rules are non-negotiable. If you believe a rule prevents correct implementation, document why explicitly and propose an exception — do not silently violate it.

---

## 1. Rust Type System — Use It Fully

**Enums over strings for any finite set of values.**

Every finite, bounded set of values in this codebase MUST be an enum. This is non-negotiable.

```rust
// ❌ BANNED
let icon: &'static str = "apartment";
let route: &'static str = "/l/assets";
let status: &str = "active";

// ✅ REQUIRED
let icon: NavIcon = NavIcon::Apartment;
let route: FolioRoute = FolioRoute::LandlordAssets;
let status: AssetStatus = AssetStatus::Active;
```

**The compiler must enforce correctness, not tests or code review.**

If you add a new variant to an enum, the compiler will point to every match arm that needs updating. This is the goal. Never use `_ => {}` wildcard arms in exhaustive matches on enums you own — it defeats this protection.

```rust
// ❌ BANNED in owned enums
match status {
    AssetStatus::Active => ...,
    _ => {} // silently ignores new variants
}

// ✅ REQUIRED
match status {
    AssetStatus::Active   => ...,
    AssetStatus::Inactive => ...,
    AssetStatus::Pending  => ...,
    // compiler forces this arm when Archived is added
}
```

---

## 2. Zero-Cost Abstractions — const fn and &'static str

Use `const fn` for methods on enums that return `&'static str`. This produces no runtime allocation and no indirection.

```rust
// ✅ Correct — no allocation, resolved at compile time
impl NavIcon {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Apartment => "apartment",
            // ...
        }
    }
}

// ❌ Wrong — allocates a String unnecessarily
impl NavIcon {
    pub fn to_string(self) -> String {
        match self {
            Self::Apartment => "apartment".to_string(),
        }
    }
}
```

Implement `std::fmt::Display` by delegating to `as_str()` — never by constructing a `String`.

---

## 3. No Panics on Server Paths

`unwrap()`, `expect()`, and `panic!()` are **banned in all server function bodies and backend handlers**. No exceptions.

```rust
// ❌ BANNED in #[server] fns
let val = something.unwrap();
let val = something.expect("this should never happen");

// ✅ REQUIRED — propagate with ?
let val = something.map_err(|e| ServerFnError::new(e.to_string()))?;
```

`unwrap()` in tests is acceptable. `unwrap()` on `Option` in view macros is acceptable only when the value is provably non-None by construction (document why inline).

---

## 4. No Mock Data in Production Code Paths

No hardcoded business data (names, amounts, addresses, IDs) anywhere in `apps/folio/src/` or `apps/network-instance/src/`.

```rust
// ❌ BANNED
let items = vec![
    Asset { name: "Maple Blvd".to_string(), revenue: 14400 },
];

// ✅ REQUIRED — return Err if endpoint not ready yet
#[server]
async fn get_assets() -> Result<Vec<Asset>, ServerFnError> {
    // If the endpoint doesn't exist yet:
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
Redirect::to("/l");

// ✅ REQUIRED
<a href=FolioRoute::LandlordAssets.path()>
window.location.set_href(FolioRoute::Login.path());
view! { <Redirect path=FolioRole::Landlord.home_path()/> }
```

When adding a new route to `app.rs`, simultaneously add it to `FolioRoute` in `nav.rs`. The route string appears in exactly one place.

---

## 6. Typed Icons — NavIcon Enum

All Material Symbols icon names MUST be `NavIcon::Variant.as_str()`. String literals for icon names are banned.

```rust
// ❌ BANNED
<span class="material-symbols-outlined">{"apartment"}</span>

// ✅ REQUIRED
<span class="material-symbols-outlined">{NavIcon::Apartment.as_str()}</span>
```

When the design calls for a new icon, add it to the `NavIcon` enum first, then use it.

---

## 7. SSR Safety — No Window Calls in Server Context

`web_sys::window()`, `document()`, `localStorage`, and any browser API MUST be gated:

```rust
// ✅ For hydrate-only logic
#[cfg(feature = "hydrate")]
fn client_only_thing() { ... }

// ✅ For event handlers (already WASM-only by nature)
on:click=move |_| { web_sys::window()... }  // OK — event handlers only run client-side

// ❌ BANNED — will panic on SSR
#[server]
async fn my_server_fn() -> Result<(), ServerFnError> {
    let win = web_sys::window().unwrap(); // PANIC on server
}
```

---

## 8. Role → Config via FolioRole, Not Imports

Layouts and components derive nav, theme, and permissions from the `FolioRole` enum, not from importing specific static configs.

```rust
// ❌ BANNED — import couples layout to specific config
use crate::components::nav::LANDLORD_NAV;
<SidebarNav config=&LANDLORD_NAV/>

// ✅ REQUIRED — role IS the index
let config = session_info.folio_role.nav_config();
<SidebarNav config=config/>
```

This means adding a new `FolioRole` variant automatically triggers a compiler error at the `nav_config()` match arm — forcing nav config creation before the new role can compile.

---

## 9. Design Tokens Over Literals

No hardcoded color values in Rust view macros or CSS. All colors must come from the design token system defined in `apps/folio/tailwind.config.js`.

```rust
// ❌ BANNED
<div style="background: #131b2e; color: white;">

// ✅ REQUIRED
<div class="bg-primary-container text-on-primary">
```

Banned CSS classes (stitch artifacts that must not enter production code):
- `bg-slate-*`, `text-slate-*` → use `surface-container-*` / `on-surface*`
- `bg-white` → `bg-surface-container-lowest`
- Inline `style=""` with color values → extract to `apps/folio/style/main.css`
- Hardcoded `#hex` anywhere in Rust strings

---

## 10. Copy for Small Enums, Clone for Data

Enums that are purely descriptive (no heap data) MUST derive `Copy`. Structs containing `String`, `Vec`, or other heap types derive `Clone` only.

```rust
// ✅ Small descriptive enum — derives Copy
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavIcon { Home, Apartment, ... }

// ✅ Data-bearing struct — Clone only
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Asset { pub name: String, pub address: String }
```

---

## 11. Explicit Error Types in Public APIs

Public-facing server functions return `Result<T, ServerFnError>`. Internal functions between modules return domain-specific `Result<T, MyError>` where `MyError` is a typed enum, not a string.

```rust
// ❌ Acceptable only in server fn boundary (ServerFnError is the wire type)
Err(ServerFnError::new("something went wrong"))

// ✅ Preferred for internal module boundaries
#[derive(Debug, thiserror::Error)]
pub enum AssetError {
    #[error("asset not found: {0}")]
    NotFound(Uuid),
    #[error("permission denied")]
    Forbidden,
}
```

---

## 12. Consistency — The One-Place Rule

Every piece of information that could drift has exactly one source of truth:

| Information | Single source |
|---|---|
| Route paths | `FolioRoute::path()` in `nav.rs` |
| Icon names | `NavIcon::as_str()` in `nav.rs` |
| Nav items per role | `*_NAV` statics in `nav.rs` |
| Role → nav config binding | `FolioRole::nav_config()` |
| Role → home path | `FolioRole::home_path()` in `auth.rs` |
| Design tokens | `apps/folio/tailwind.config.js` |
| Backend routes | `backend/src/handlers/folio/*.rs` |
| Page implementation status | `docs/folio/page_queue.md` |

If you find information duplicated across two files, consolidate before adding more.

---

## Exceptions Policy

A deviation from these rules is permitted only if:
1. The Leptos or Rust compiler makes the typed approach impossible in this specific context, AND
2. The deviation is documented with a `// EXCEPTION: <reason>` comment inline, AND
3. A GitHub issue or TODO links to the future fix

"It's faster to write as a string" is not a valid exception.
