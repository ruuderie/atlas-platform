# Folio Documentation Index

> **This folder is the canonical reference for building, extending, and architecting the Folio app.**  
> Read in the order listed before starting any implementation work.

---

## Architecture Decision Records

| Doc | Purpose | Read when |
|---|---|---|
| [folio_vs_network_instance.md](folio_vs_network_instance.md) | Defines which app each user type uses (landlord, tenant, vendor, PMC, brokerage) | Before any Folio page work |
| [brokerage_architecture.md](brokerage_architecture.md) | How brokerage mode coexists with Network Instance; shared-ui boundary; data flow | Before implementing `/a/**` or `/br/**` |

---

## Implementation Guides

| Doc | Purpose | Read when |
|---|---|---|
| [stitch_to_leptos_prompt.md](stitch_to_leptos_prompt.md) | **The implementation prompt.** Copy-paste this when converting any stitch HTML page to a Leptos component. Enforces API-first, mock-data-free, token-compliant implementation. | Every page implementation session |

---

## Quick Links to Key Files

| What | Path |
|---|---|
| Stitch designs (all pages) | [`designs/stitch/project_pm/folio/`](../../designs/stitch/project_pm/folio/) |
| Route → Leptos module map | [`designs/stitch/project_pm/folio/ROUTES.md`](../../designs/stitch/project_pm/folio/ROUTES.md) |
| Component manifest | [`designs/stitch/project_pm/folio/COMPONENTS.md`](../../designs/stitch/project_pm/folio/COMPONENTS.md) |
| Folio router | [`apps/folio/src/app.rs`](../../apps/folio/src/app.rs) |
| Folio design tokens (CSS) | [`apps/folio/style/main.css`](../../apps/folio/style/main.css) |
| Folio Tailwind config | [`apps/folio/tailwind.config.js`](../../apps/folio/tailwind.config.js) |
| Shared UI components | [`apps/shared-ui/src/components/`](../../apps/shared-ui/src/components/) |
| Backend Folio handlers | [`backend/src/handlers/folio/`](../../backend/src/handlers/folio/) |
| Platform CURRENT_STATE | [`docs/CURRENT_STATE.md`](../CURRENT_STATE.md) |

---

## Design Token Discrepancy — Stitch vs Real App

> **IMPORTANT:** The stitch HTML files use a **different Tailwind color palette** than the real Folio app.

| Stitch uses | Real app uses | Notes |
|---|---|---|
| `bg-slate-*`, `text-slate-*` | `bg-surface-container-*`, `text-on-surface*` | Stitch used slate for simplicity |
| `bg-white` | `bg-surface-container-lowest` | = `#ffffff` in light mode |
| `text-black` / `bg-black` | `text-on-surface` / `bg-primary` | |
| Custom color palette in `<script id="tailwind-config">` | `apps/folio/tailwind.config.js` | Stitch has inline config; real app uses file |
| Inline `style=""` with gradients | CSS class in `main.css` | Never port inline styles |

The canonical token mapping is documented in the implementation prompt: [`stitch_to_leptos_prompt.md` — Step 2](stitch_to_leptos_prompt.md).

---

## Implementation Status

See [`designs/stitch/project_pm/folio/ROUTES.md`](../../designs/stitch/project_pm/folio/ROUTES.md) for the full page-by-page status (stub / missing).

**Summary as of ROUTES.md generation:**

| Namespace | Total pages | Stubs | Missing |
|---|---|---|---|
| Landlord `/l/**` | 25 | 11 | 14 |
| Tenant `/t/**` | 10 | 5 | 5 |
| STR Host `/s/**` | 5 | 0 | 5 |
| Owner `/o/**` | 2 | 0 | 2 |
| Vendor `/v/**` | 3 | 3 | 0 |
| PMC `/pmc/**` | 3 | 0 | 3 |
| Wizards | 4 | 0 | 4 |
| Public | 5 | 1 | 4 |
| **Total** | **57** | **20** | **37** |
