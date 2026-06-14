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
