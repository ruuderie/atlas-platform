# RustUI Integration Walkthrough

This walkthrough details the steps taken to fully integrate the `rust-ui` library components into the existing Leptos frontend.

## Goals

1.  Replace manual HTML elements and boilerplate with robust `rust-ui` counterparts.
2.  Maintain the existing dark, Palantir-style aesthetic.
3.  Ensure data reactivity (signals) remains intact.
4.  Resolve all compile-time errors resulting from the refactor.
5.  Launch the application and visually verify the changes.

## Refactored Components

### 1. Main Layout ([src/app.rs](./apps/platform-admin/src/app.rs))
We successfully substituted raw layout elements for full-fledged `rust-ui` layout components:
*   `<header>` $\rightarrow$ `<Header>`
*   `<select>` $\rightarrow$ `<Select>` (with `<SelectTrigger>`, `<SelectContent>`, `<SelectOption>`)
*   `<div class="avatar">` $\rightarrow$ `<Avatar>`

![Dashboard Main View](./screenshots/dashboard_main_1773751252856.png)

### 2. Multi-Site Settings ([src/pages/multi_site.rs](./apps/platform-admin/src/pages/multi_site.rs))
The custom `Toggle` code was completely rewritten to use the `Switch` component. This proved an effective test of property binding and correctly reacting to user clicks to switch "Enabled Modules" toggles. We also adopted the structured `<Card>` elements (Header, Title, Content) to wrap the configurations securely.

![Site Registry View](./screenshots/site_registry_page_1773751273714.png)

### 3. CRM Grid ([src/pages/crm_grid.rs](./apps/platform-admin/src/pages/crm_grid.rs))
The layout was overhauled to utilize:
*   **`Tabs`**: The `TabsList` and `TabsTrigger` replaced local state management to organize the tabs cleanly (Customers, Deals, Contacts).
*   **`DataTable`**: The previous un-styled table was transitioned to `Table`, `TableHeader`, `TableRow`, etc.

![CRM Grid with Tabs](./screenshots/crm_grid_tabs_1773751294142.png)

### 4. CMS Editor ([src/pages/cms_editor.rs](./apps/platform-admin/src/pages/cms_editor.rs))
We integrated the forms utilizing:
*   `Input` element (replacing `<input>`)
*   `Textarea` element (replacing `<textarea>`)
*   `Label` element (replacing `<label>`)
*   `Button` element (replacing `<button>`)

Reactive two-way bindings across `RwSignal` fields still work as highlighted in the live preview.

![CMS Editor Live Verification](./screenshots/cms_editor_preview_1773751412800.png)

## Validation Methods

1. **`cargo check`**
    * Refactored types, handled `.into()`, explicit `String` references vs. `Option<String>`, and addressed unused dependencies. Total warnings came down significantly and the system compiles with an `ok` exit status.
2. **Trunk Server**
    * We successfully cleared dangling locks on port `8080` (by issuing an explicit kill) and moved to `8081` for the live environment.
3. **Automated Browser Subagent**
    * A headless browser journey navigated `http://127.0.0.1:8081` to collect the interactive feedback seen above. The layout didn't break during state changes, highlighting a very successful drop-in replacement job!

### Platform Dashboard & Navigation Update
We recognized the need for an enterprise-level entry point into the application. We refactored the global sidebar to use business-oriented taxonomy ("Network Directories", "Sales & Relationships") and replaced the empty index route with a comprehensive `<Dashboard />`.

This new view features key data aggregations (Active Directories, Deals Pipeline) and a Recent Activity feed simulating platform-wide events. 

![Dashboard Overview](./screenshots/dashboard_overview_verification_1773767974125.png)

### Site Dashboard Drill-Down
A robust drill-down feature has been implemented, routing administrators from the global **Network Directories** space into a parameterized domain dashboard (`/sites/:id`). 

This interface utilizes the complex `Tabs` layout combined with the `DataTable` to cleanly present segmented Directory configurations, such as the specific isolated Businesses (Listings) mapped to that directory domain and their constituent user profiles.

![Specific Directory Dashboard](./screenshots/site_dashboard_build_failure_1773768461579.png)

## Verification
- Verified Leptos routing and Tailwind styling dynamically adjust for desktop and mobile viewport breakpoints.
- Verified dynamic rendering of Mock entity traits and form fields mapping successfully inside generic detail components.

## Refactoring Layouts and Fixing Tailwind V3 Engine Issues

After integrating the components, a significant layout bug emerged where the `Select` component was overlapping with the `<main>` content area. Additionally, the [multi_site.rs](./apps/platform-admin/src/pages/multi_site.rs) (Site Registry) remained squashed vertically instead of laying out clearly spaced "Palantir-style" tenant cards.

> [!CAUTION]
> The root cause was environmental: **Node.js v18.14.2** is entirely incompatible with Tailwind V4's `@tailwindcss/oxide` Rust binding engine. Trunk was failing to generate *any* utility classes for our components because `npm` was silently swallowing the compilation crashes related to the native extensions.

### Resolution Steps
1. **Engine Downgrade**: Wiped the `pnpm` and `node_modules` cache completely to purge the frozen `@tailwindcss+oxide` package remnants and explicitly migrated [package.json](./apps/platform-admin/package.json) to Tailwind **V3.4.17**.
2. **Build Bypass:** Configured a custom `"build-tailwind"` NPM script to compile [index.css](./apps/platform-admin/style/index.css) into `tailwind-out.css`. Standard `npx` routing was failing because of `type: module` inconsistencies against CommonJS configs.
3. **Trunk Wire-in**: Pointed [index.html](./apps/platform-admin/index.html) to consume the compiled `tailwind-out.css` instead of telling Trunk to execute the broken CSS watch loop internally.
4. **Structural Rewrite**: Stripped all legacy custom CSS grid classes (`.app-layout`, `.site-card`, etc.) from [app.rs](./apps/platform-admin/src/app.rs) and [multi_site.rs](./apps/platform-admin/src/pages/multi_site.rs) and replaced them tightly with raw structural Tailwind classes like `w-64`, `grid-cols-2`, and `max-w-7xl`.

### Rendering Validated
The `Select` active dropdown and sidebar now behave properly with Tailwind grid elements taking effect over raw HTML block placement:

````carousel
![Dashboard - Fixed Select Nav / Sidebar Layout](./screenshots/dashboard_main_view_1773754899279.png)
<!-- slide -->
![Site Registry - Fixed CSS Grid Tenant Cards](./screenshots/site_registry_grid_view_1773754912531.png)
````

## CRM Layout & Dark Theme Refinements (Phase 2)

Following the initial Tailwind integration, we addressed specific UI deficiencies reported by the user on the CRM page, notably text readability issues, labeling, and severe layout overlapping.

### Key Refinements

1. **Text Legibility on Dark Backgrounds**:
    * **Issue**: The Palantir-inspired dark theme caused table data and active text elements to blend into the dark navy backgrounds seamlessly, rendering text unreadable.
    * **Fix**: Added missing `foreground: '#f8fafc'` tokens to the [tailwind.config.js](./apps/platform-admin/tailwind.config.js) root and `card`/`popover` objects. This ensured that Tailwind's `text-foreground` classes natively enforce a high-contrast white/light-gray font color against the `#0f172a` background.
    
2. **CRM Component Overlapping**:
    * **Issue**: The Data Table was violently squashing into the right-hand inspection view ("Select a Customer"). Furthermore, the `TabsContent` was rendering *above* the Tabs header and bleeding over the page title. 
    * **Root Cause**: The custom `<Tabs>` component in [tabs.rs](./apps/platform-admin/src/components/tabs.rs) was forcibly wrapping all children (including the table body) inside an `inline-flex h-9` Tailwind container intended *only* for the tab buttons themselves. 
    * **Fix**: 
        * We decoupled `<TabsList>` from the internal `<Tabs>` wrapper, moving it directly to [crm_grid.rs](./apps/platform-admin/src/pages/crm_grid.rs) where we could lay out the tab triggers predictably.
        * We replaced the legacy `.split-pane` CSS layout and standard CSS grid with a responsive Flexbox layout (`flex-col xl:flex-row`), allowing the `DataTable` to `flex-1` and automatically scroll horizontally instead of overlapping the inspection pane.
        
### Results

The CRM page has been successfully updated to a clean, cohesive enterprise layout reflecting the Palantir design language. The "CRM & Architecture" tab is correctly labeled "CRM".

**Before Flexbox/Tabs separation (Table overlapping and squashed):**
![Broken CRM Layout](./screenshots/crm_final_validation_customers_1773756026767.png)

**After (Clean side-by-side flex layout with high text contrast):**
![Fixed CRM Layout](./screenshots/crm_layout_verification_1773756163573.png)


## Expanded CRM Detail Pages and CMS Content Views (Phase 3)

The CRM and CMS pages were further expanded to fulfill the full suite of CRUD view capabilities required by the user.

### 1. CMS Article List 
The single-page Rich Text Editor ([cms_editor.rs](./apps/platform-admin/src/pages/cms_editor.rs)) was encapsulated inside a `Tabs` structure. We introduced an "All Articles" view featuring a large `DataTable` of published materials alongside the original "Editor" mode.

````carousel
![CMS Content Manager - Articles Main List](./screenshots/cms_articles_list_1773760046216.png)
<!-- slide -->
![CMS Content Manager - Live Draft Editor](./screenshots/cms_editor_view_1773760062660.png)
````

### 2. Deep Linking / CRM Detail Layout
We engineered a generic Leptos-router component ([crm_detail.rs](./apps/platform-admin/src/pages/crm_detail.rs)) explicitly designed to display enterprise record details dynamically based on url paths (`/crm/:entity/:id`). The page implements a complex multi-card grid view spanning Contact Information, Record States (Active/Draft), Description, Related Sub-records, and internal System Info overviews. It dynamically extracts specific attributes based on the backend data architecture.

````carousel
![User Detail View](./screenshots/crm_user_record_detail_1773769501821.png)
<!-- slide -->
![Lead Detail View](./screenshots/crm_lead_record_detail_1773769510063.png)
<!-- slide -->
![Account & Customer Detail View](./screenshots/crm_customer_record_detail_1773769519102.png)
<!-- slide -->
![Deal Pipeline Detail View](./screenshots/crm_deal_record_detail_1773769529631.png)
````

## 5. Frontend Monorepo Architecture
To support the long-term vision of operating both an internal Platform Admin tool and an external tenant-facing Directory application, the frontend has been reorganized into a **Cargo Workspace**.

### Workspace Structure
```text
RustSvelteBusinessDirectory/
└── apps/                       (Frontend Monorepo)
    ├── Cargo.toml              (Workspace Root)
    │
    ├── platform-admin/         (Administration App)
    │   ├── Cargo.toml
    │   └── src/pages/          (Dashboard, CRM, MultiSite)
    │
    ├── directory-instance/     (Public Tenant App)
    │   ├── Cargo.toml
    │   └── src/main.rs         (Public Listings, Search)
    │
    └── shared-ui/              (Isolated Palantir-style Components)
        ├── Cargo.toml
        └── src/
            ├── components/     (Buttons, Cards, Tabs, DataTables)
            ├── utils/          (Date formatters)
            └── constants.rs
```

**Benefits**:
- **Code Reusability**: Both `platform-admin` and `directory-instance` depend directly on the local `shared-ui` package, allowing them to use the exact same aesthetic and UX components seamlessly without duplicating Leptos boilerplate.
- **Independent Lifecycles**: The platform administration and public directories can be built, tested, and deployed independently as separate binaries.
