# Atlas App Registry — A-Number Convention

> Mirrors the **G-Generic numbering system** (G01–G31+).
> Every AtlasApp gets a permanent, immutable A-number. The number is the source of truth for
> branch names, internal references, and cross-app documentation. Human-readable names are
> aliases — the A-number never changes even if an app is renamed.

---

## Registered Apps

| A# | Internal Name | Rust Module | App Type | Status |
|---|---|---|---|---|
| **A1** | Core Platform | `core_platform` | Headless API — cross-cutting platform services, auth, CMS routes | ✅ Live |
| **A2** | Anchor | `anchor` | Leptos SSR — public-facing CMS / site builder | ✅ Live |
| **A3** | Network Instance | `network_instance` | Leptos SSR — multi-tenant network community app | ✅ Live |
| **A4** | Property Management | `property_management` | Leptos SSR — cross-border real estate OS (Miami + Brazil) | 🏗 In development |

---

## Naming Conventions

### Branch Names
Use the A-number prefix. Do **not** spell out the app name — the number is the identifier.

```
feat/a4               ← primary feature branch for A4 development
feat/a4-payment-rails ← sub-feature branch scoped to A4
fix/a4-mempool-poll   ← bugfix scoped to A4
chore/a4-seed-templates
test/a4-lease-service
```

The branch name `feat/a4` alone is enough. Anyone who needs to know which app it is
checks this registry.

### Rust Modules
```
backend/src/atlas_apps/property_management.rs   ← A4 AtlasApp impl
backend/src/services/pm/                        ← A4 service modules
apps/property-management/                       ← A4 Leptos frontend
```

### API Route Prefix
Each app gets a namespaced API prefix:
```
A1 CorePlatform  → /api/           (platform-level, no app prefix)
A2 Anchor        → /api/anchor/
A3 NetworkInst   → /api/net/
A4 PropertyMgmt  → /api/pm/
```

### Migration Naming
Migrations scoped to an app use the A-number in the filename:
```
m20260801_a4_pm_g27_template_scope.rs
m20260803_a4_atlas_payment_credentials.rs
```
> **Exception:** Platform-level migrations (e.g., adding a column to a G-generic table) use
> the G-number: `m20260801_g27_template_scope.rs`. A-prefixed migrations are only for
> app-specific tables (which A4 has very few of — see Rule 7 in `CURRENT_STATE.md`).

---

## Assigning New A-Numbers

When a new app is added:
1. Claim the next A-number by adding a row to the **Registered Apps** table above.
2. Update `backend/src/atlas_apps/mod.rs` and `get_active_apps()`.
3. Follow `docs/adding_a_new_app.md` for the full registration checklist.
4. Use the A-number in all branch names and internal references from day one.

**Next available:** A5

---

## Why A-Numbers?

- **Obfuscation** — branch names like `feat/a4` don't telegraph product strategy to anyone
  who shouldn't have it (external contributors, CI logs, public repos).
- **Stability** — if an app is renamed (`PropertyManagement` → `RealEstateOS`), the A-number
  and all branch conventions stay identical. Zero refactor.
- **Mirrors the G-system** — the platform already uses G-numbers for generics
  (G01 = accounts, G27 = scorecards). A-numbers give apps the same stable identity layer.
- **Scannability** — `feat/a4-payment-rails` and `feat/a4-lease-service` are instantly
  sortable, grouped, and identifiable in any branch list.

---

*Established: June 2026 · Atlas Platform*
