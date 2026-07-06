# Leptos Frontend Architecture & Best Practices Decisions

This document establishes the official frontend architecture decisions and coding guidelines for the Atlas Platform Leptos apps (`platform-admin`, `folio`, `anchor`, and `network-instance`).

---

## 1. Headless Shared Components & Extraction Rules

To avoid code duplication and maintain UI consistency, components must be systematically categorized into two tiers: **`shared-ui` (headless/generic)** vs. **App-Specific**.

### When to Extract to `shared-ui` (`apps/shared-ui/src/components/ui/`)
A UI pattern must be extracted to the shared library only if it meets **all** of the following:
1. **Zero Domain Logic**: It contains no references to database entities, platform roles, or domain models (e.g. no references to `LeaseId`, `TenantId`, `FolioRole`).
2. **Presentational or Headless**: It handles purely visual styles or layout interactions (e.g., dropdown portals, customized inputs, date pickers, or sonner toasts). It never makes direct `fetch()` calls or server functions.
3. **Cross-App Usage**: It is utilized across at least two separate platform apps (e.g., shared between `folio` and `platform-admin`).

### Component Spec Design
When designing shared components, developers must expose generic interfaces using Leptos signals, callbacks, and optional props.
* **Incorrect (Tight Coupling)**:
  ```rust
  #[component]
  pub fn SaveButton(tenant_id: Uuid) -> impl IntoView { ... }
  ```
* **Correct (Headless & Decoupled)**:
  ```rust
  #[component]
  pub fn Button<F>(
      label: String,
      on_click: F,
      #[prop(optional)] disabled: MaybeSignal<bool>
  ) -> impl IntoView 
  where F: Fn(MouseEvent) + 'static
  ```

---

## 2. CSR vs. SSR Architectural Decision

A hybrid architectural model has been selected for the Atlas Platform to optimize search discoverability and speed:

```
                  ┌──────────────────────────────┐
                  │   Atlas Platform Monorepo    │
                  └──────────────┬───────────────┘
                                 │
         ┌───────────────────────┴───────────────────────┐
         ▼                                               ▼
┌─────────────────────────────────┐             ┌─────────────────────────────────┐
│   CSR (Client-Side Rendering)   │             │  SSR + WASM (Server Hydration)  │
├─────────────────────────────────┤             ├─────────────────────────────────┤
│  • apps/platform-admin          │             │  • apps/folio                   │
│  • Behind strict login walls    │             │  • apps/anchor                  │
│  • Zero SEO indexing required   │             │  • apps/network-instance        │
│  • High-density admin tables    │             │  • High SEO indexing priority   │
│  • Rich dashboard interactions  │             │  • Public landing pages & blogs │
└─────────────────────────────────┘             └─────────────────────────────────┘
```

### Server-Side Rendering (SSR + WASM)
* **Target Apps**: `apps/folio`, `apps/anchor`, `apps/network-instance`.
* **Rationale**: 
  1. **SEO Optimization**: Public landing pages, blog entries, listings, and short-term rental compliance registries must be search-engine crawlable.
  2. **First-Load Performance**: Users receive immediately rendered HTML from the server, optimizing Largest Contentful Paint (LCP) before the WASM hydrator registers event listeners in the browser.
  3. **Multi-Tenant Host Resolution**: Resolves host headers dynamically (`GET /api/pub/resolve`) at route delivery time to serve custom branding files.

### Client-Side Rendering (CSR)
* **Target Apps**: `apps/platform-admin`.
* **Rationale**:
  1. **Zero SEO Value**: Admin routes are entirely behind credentials walls.
  2. **Reduced Server Overhead**: Server CPU cycles are not wasted rendering complex, high-density data tables or chart wrappers.
  3. **Instant Browser Navigation**: Standard single-page application bundling allows instant client-side route changes and quick data filtering.

---

## 3. Marketing Homepage Routing Strategy

The platform landing page (`marketing/index.html` mockup) must be the default homepage at root `/` for unauthenticated traffic.

### Routing Configuration (`apps/platform-admin/src/app.rs`)
To integrate this, the top-level routing boundaries are structured as follows:

```rust
<Routes fallback=|| "Not found.">
    // Public landing page (marketing index) at the root
    <Route path=path!("/") view=PublicMarketingPage />
    <Route path=path!("/login") view=Login />
    <Route path=path!("/verify-token/:token") view=VerifyToken />
    
    // Authenticated layout routing
    <Route path=path!("/*any") view=AuthenticatedLayout />
</Routes>
```

### Authenticated Redirect Invariant
* If an unauthenticated user attempts to access any subpath under `AuthenticatedLayout` (such as `/dashboard` or `/apps`), they are redirected to `/login`.
* If a logged-in user hits the root `/` page, the router dispatches a redirect hook checks the active user's session role context (G-32 RBAC), automatically routing the user to `/dashboard`.

---

## 4. Navigation & Link Best Practices (folio / SSR apps)

> **Context**: Established after debugging broken navigation on the Folio marketing pages (July 2026). Symptoms: links changing the URL correctly but rendering the wrong page component; links doing nothing; intermittent failures depending on WASM hydration timing.

### 4.1 How the Leptos Router intercepts clicks

`leptos_router 0.8` installs a **global document-level click listener** (`leptos_router/src/location/mod.rs → handle_anchor_click`). It intercepts **every `<a>` click** on a same-origin URL — including plain HTML `<a href="...">` tags, not just Leptos `<A>` components.

When intercepted the router:
1. Calls `ev.prevent_default()` — kills the browser's native navigation
2. Pushes a new entry to the History API (`pushState`)
3. Re-renders the client-side component tree for the new path

If the client-side route table doesn't correctly match the new path (e.g. due to a hydration error), the URL changes but the **wrong component is displayed** — the most confusing failure mode.

### 4.2 The `rel="external"` bypass

The router **skips interception** when it detects:

```rust
// leptos_router/src/location/mod.rs:347
if a.has_attribute("download") || rel.any(|p| p == "external") {
    return; // browser handles natively → full HTTP GET
}
```

Add `rel="external"` to force a full page load:

```rust
// ✅ Full page load — bypasses the global handler
<a href="/login" class="mktg-btn-signin" rel="external">"Sign in"</a>

// ⚠️  Intercepted — push-state only, no SSR fetch
<a href="/login" class="mktg-btn-signin">"Sign in"</a>
```

### 4.3 When to use `<A>` vs `<a>` vs `<a rel="external">`

| Scenario | Element | Reason |
|---|---|---|
| In-app navigation (authenticated `/l/**` routes) | `<A href="...">` | Push-state keeps session/state alive, no full WASM reload |
| Cross-marketing-page links (`/beta`, `/login`, `/founding`, etc.) | `<a href="..." rel="external">` | Each marketing page is an independent SSR response; push-state renders the wrong component |
| In-page hash anchors (`#features`, `#pricing`) | `<a href="#section">` | Fragment navigation is never intercepted by the router |
| File download | `<a href="/export.csv" download>` | Router already skips the `download` attribute |
| External URLs | `<a href="https://...">` | Router skips cross-origin links automatically |

### 4.4 `<A>` with `attr:class` — SSR caveat

The Leptos `<A>` component renders its `href` reactively. The `href` **IS** present in SSR output. However, the push-state click handler is only attached after WASM hydration. If hydration fails (e.g. a DOM mismatch from `<details>` elements), the `<A>` component loses its click handler and becomes a styled but inert element.

**Safest pattern for marketing CTAs**:

```rust
// ✅ Reliable regardless of hydration state
<a href="/beta" class="mktg-btn-accent" rel="external">"Apply now →"</a>

// ⚠️  Fragile if hydration fails
<A href="/beta" attr:class="mktg-btn-accent">"Apply now →"</A>
```

### 4.5 Native `<details>`/`<summary>` for nav dropdowns

Avoid `RwSignal<bool>` toggling a CSS class for nav dropdowns. The signal update requires WASM — before hydration, the button has no `on:click` handler and the dropdown is permanently closed.

Use **native HTML `<details>`/`<summary>`** instead:

```rust
<details class="mktg-nav-role-dropdown">
    <summary>"For your role"</summary>
    <div class="mktg-nav-role-panel">
        <a href="/brokers" class="mktg-nav-role-item" rel="external">
            "For Brokers"
        </a>
    </div>
</details>
```

CSS drives open/close via `details[open]` — **zero JS, works on first paint, SSR-safe**:

```css
.mktg-nav-role-panel { display: none; }

details.mktg-nav-role-dropdown[open] .mktg-nav-role-panel {
    display: flex;
    flex-direction: column;
    animation: roleDropIn .15s ease;
}
details.mktg-nav-role-dropdown[open] .mktg-nav-role-arrow {
    transform: rotate(180deg);
}
/* Remove browser default marker */
details > summary { list-style: none; }
details > summary::-webkit-details-marker { display: none; }
```

**Trade-off**: `<details>` does not auto-close on outside click without JS. Acceptable for marketing navs (navigation destroys the component). For authenticated app dropdowns where outside-click matters, use `leptos::window_event_listener` inside the authenticated shell (runs after WASM is loaded).

### 4.6 Quick diagnostic checklist

| Symptom | Likely cause | Fix |
|---|---|---|
| URL changes, wrong page renders | Router intercepted click; push-state matched wrong route | Add `rel="external"` to force full HTTP GET |
| Click does nothing at all | `<A>` lost handler due to hydration failure; or CSS blocking pointer events | Add `rel="external"`; check browser console for hydration errors |
| Works sometimes, not others | Race between click and WASM hydration completing | Add `rel="external"` to remove WASM dependency |
| Hash anchor doesn't scroll | Element `id` doesn't match `href` fragment | Verify `id` matches exactly |
| Dropdown never opens | Signal-based toggle requires WASM; hydration failed | Replace with `<details>`/`<summary>` |
