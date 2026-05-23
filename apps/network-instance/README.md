# Network-Instance App — The Curated Multi-Tenant Portal

`network-instance` is a premium, high-performance Leptos SSR frontend app designed to power dynamic, user-facing tenant business networks and directory sites. Operating as a single, multi-tenant codebase, it dynamically resolves its branding, configurations, and catalogs at runtime based on the incoming request subdomain or domain context.

---

## 1. Architectural Strategy & Design System

The creative north star for this design system is **"The Architectural Curator"**. The visual language rejects heavy solid lines, boxes-within-boxes, and crowded layouts in favor of:

*   **Tonal Layering ("No-Line" Rule)**: Explicitly forbids 1px solid borders for defining cards or sections. Structure is achieved purely through background color shifts (`surface-container-low` background with elevated `surface-container-lowest` cards).
*   **Asymmetric Balance**: Intentional white space and asymmetric grid proportions (e.g., 2/3 main listings feed paired with a 1/3 featured sidebar) create a premium digital editorial aesthetic resembling a bespoke magazine.
*   **Atmospheric Shadows**: Ambient, low-opacity shadows computed using a tint of `on-surface` (never pure black) for modals, menus, and elevated sections:
    `box-shadow: 0 10px 30px rgba(25, 28, 29, 0.04), 0 4px 8px rgba(25, 28, 29, 0.02);`
*   **Typography Rhythm**: Interplay between **Manrope** (stable, modern, authoritative Display and Headlines) and **Inter** (high-legibility small labels and body copy).

---

## 2. Directory Layout & Module Structure

```
apps/network-instance/
├── src/
│   ├── main.rs              # App entrypoint initializing Trunk / Hydration
│   ├── lib.rs               # Library registration and WASM hookups
│   ├── app.rs               # Root router, shell template, and sub-pages mapping
│   ├── auth.rs              # Client session authorization & cookies
│   ├── components/          # Reusable local page parts
│   └── pages/               # Top-level page routes
│       ├── mod.rs
│       ├── admin.rs         # Operator-facing admin console shell (/admin)
│       ├── search.rs        # Premium Search & Curated Results feed
│       ├── auth/            # Sign-in and onboarding pages
│       └── dashboard/       # Client member area
├── Design.md                # Tonal color system and components spec
├── Trunk.toml               # Cargo Trunk compiler settings
├── tailwind.config.js       # Editorial theme Tailwind configurations
└── Dockerfile               # Production container build manifest
```

---

## 3. The `/admin` Portal & RBAC Integration

The network operator interface is routed at `/admin`. It uses an SSR-blocking resource gate to prevent flashing unauthenticated UI before hydration.

### Authentication & Authorization Gates
1.  **Backend Token Verification**: The page uses `check_admin_access()`, a backend server function that checks the `session` cookie against `/api/admin/modules` to verify if the operator possesses the `Owner`, `Admin`, or `PlatformSuperAdmin` roles.
2.  **Role Verification**: This gate completely bypasses the stale `user.is_admin` boolean (which was deprecated and removed in database migration `m20260504_000002`).
3.  **Module Load**: Pushes authorized modules through a custom `<AdminModuleSidebar>` using `SidebarTheme::Network` rules.

---

## 4. Current CRM Alignment Status & Roadmap (Gap Analysis)

As the Atlas core platform expands its tenant CRM functionalities, several placeholders in the current `network-instance` admin console will be aligned with the backend in future releases:

### Current Standalone CRM Gaps:
*   **Leads Panel**: The Leads panel currently serves as a static informational placeholder. It is planned to be replaced with the full [leads::LeadTable](file:///Users/oply/src/git/orbit_/atlas-platform/apps/anchor/src/pages/admin/leads.rs) and split CRM panels.
*   **Contacts Panel**: The Contacts list is currently a static placeholder and does not pull live database contact structures. It will be upgraded to the full [contacts::ContactTable](file:///Users/oply/src/git/orbit_/atlas-platform/apps/anchor/src/pages/admin/contacts.rs) rendering.
*   **Headless Widgets**: Integrating the dynamic `CrmStageBar`, `CrmTimeline` (real-time notes logging and chronological activity stream), and `PropertiesEditor` (editable dynamic metadata fields) from the `shared-ui` package.

---

## 5. Development & Trunk Build commands

To compile and launch the `network-instance` app locally:
1.  Ensure you have Trunk installed: `cargo install --locked trunk`
2.  Run the application in development mode:
    ```bash
    trunk serve --port 8081
    ```
3.  Production optimized build:
    ```bash
    trunk build --release
    ```
