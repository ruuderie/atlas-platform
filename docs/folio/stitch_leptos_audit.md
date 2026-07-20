# Folio Stitch ↔ Leptos fidelity audit

Living inventory of design-mock vs shipped UI. Update when a surface moves class.

**Sources of truth**

| Role | Source |
|------|--------|
| Path ↔ stitch folder | [`ROUTES.md`](../../../designs/stitch/project_pm/folio/ROUTES.md) (path index only — **status column is stale**) |
| Wiring / parked honesty | [`page_queue.md`](page_queue.md), `apps/folio/src/app.rs` |
| Job / section density | `designs/stitch/project_pm/folio/*/code.html` |
| Chrome / IDs | `.cursor/rules/folio-landlord-chrome.mdc`, `no-opaque-ids-in-ui.mdc` |

## Classification legend

| Class | Meaning |
|-------|---------|
| **A** Aligned | Leptos matches Stitch jobs + major sections |
| **B** Leptos thin | Stitch has sections/CTAs/IA Leptos lacks |
| **C** Stitch thin / Leptos ahead | Leptos has real capability mock doesn’t show |
| **D** IA conflict | Both exist; chrome/IA differs (product call or intentional) |
| **E** One-sided | Stitch-only unwired, or Leptos-only (no stitch) |

## Cross-cutting

| Theme | Finding | Sev |
|-------|---------|-----|
| Destructive UI | Quiet header **More**: Archive… + Delete permanently… (PURGE); no Danger zone strip | P1 |
| UUID in UI | No paste fields; prefer labeled pickers; avoid raw UUID display in product chrome | P0–P1 |
| Chrome islands | Prefer `PageHeader` / `folio-*`; retire `map-*` / `bsys-*` / `insp-*` leftovers | P2 |
| Thin-shell | Tenant/vendor/STR/owner/PMC dashboards systematically thinner than Stitch | P2 |
| Honesty | Prefer “Not available” over fake Save / empty theatre | P1 |

## Landlord priority surfaces

| Route | Stitch | Leptos | Class | Top gaps | Sev |
|-------|--------|--------|-------|----------|-----|
| `/l` | `l_dashboard` | `dashboard.rs` | C | STR KPI `beach_access` + hardened icon CSS | P2 |
| `/l/assets` | `l_assets` | `assets.rs` | A | Street address primary; nickname secondary | P2 |
| `/l/assets/:id` parent | `l_property_hub` | `property_hub.rs` | A | Photos/map peek; More → Archive/Purge; activity photo slots | P2 |
| `/l/assets/:id` unit | `l_unit_detail` | `unit_detail.rs` | A/B | Occupancy pills (not asset.status); address subtitle | P2 |
| `/l/assets/:id` leaf | `l_asset_detail` | `asset_detail.rs` | B | Archive via shared modal | P2 |
| `/l/assets/:id/documents` | `l_property_documents` | `property_documents.rs` | A/B | Export honesty; expense peeks | P2 |
| `/l/assets/:id/systems` | `l_building_systems` | `property_systems.rs` | B | Nested thinner than `/l/systems` | P1 |
| `/l/assets/:id/portal` | `l_tenant_portal_content` | stub | E | Honest park | — |
| `/l/leases` | `l_leases` | `leases.rs` | A | — | P2 |
| `/l/maintenance` | `l_maintenance_queue` | `maintenance_queue.rs` | A | — | P2 |
| `/l/maintenance/:id` | `l_work_order_detail` | `work_order_detail.rs` | B | PhotoStrip empty; receipts thin | P1 |
| `/l/billing` | `l_billing` | `billing.rs` | B/C | Stitch overbuilt; Leptos honest preview | P1 |
| `/l/vault` | `l_digital_vault` | `digital_vault.rs` | A/B | Raw entity UUID in detail | P1 |
| `/l/ledger` | `l_ledger` | `ledger.rs` | A/B | Opaque id display | P1 |
| `/l/deals` | creative finance + hub | `deals.rs` | A | New acquisition modal + stage counts | P2 |
| `/l/deals/:id/assignment` | `l_wholesale_assignment` | — | E | Partial CTA on workspace | P1 |
| `/l/marketplace` | `l_contractor_marketplace` | `contractor_marketplace.rs` | A/B | Add vendor modal (no User ID paste) | P2 |
| `/l/inspections` | `l_inspections` | `inspections.rs` | A | `insp-*` chrome island | P2 |
| `/l/systems` | `l_building_systems` | `building_systems.rs` | A | `bsys-*` chrome; retire modal OK | P2 |
| `/l/map` | `l_map_portfolio` | `map_portfolio.rs` | A | `map-title` chrome | P2 |
| History sheets | `l_unit_history*` | history modules | A | Maint: 2-col timeline + WO radio + receipt honesty | P2 |
| Archive / Purge | quiet More menu | hub modals + `POST …/purge` | A | Soft archive + hard PURGE tree | P1 |
| `/l/campaigns` | `l_campaigns` | `campaigns.rs` | B | Enroll parked | P1 |
| `/l/syndication` | `l_syndication` | `syndication.rs` | C | Read-only honesty | — |
| `/l/communications` | `l_communications` | `communications.rs` | B | No start-tenant-room API; contrast OK | P1 |
| `/l/account/billing` | stitch billing theatre | `account_billing.rs` | E→honest | Plan/payment APIs not shipped | — |
| `/l/setup` | — | `setup.rs` | E | Leptos only | — |

## Other personas (pattern)

| Persona | Pattern | Notes |
|---------|---------|-------|
| Tenant `/t` | B on dashboards/lists | Detail routes closer to A |
| Vendor `/v` | B on hubs | Referrals shared with landlord module (D) |
| STR host `/s` | B on hubs; reviews honest empty | ROUTES “missing namespace” is wrong |
| Owner `/o` | B dashboards; A~ details | Stitch-only: cohost, network |
| PMC `/pmc` | B hubs; map A | Analytics stitch → dashboard IA |

### Stitch-only (examples)

`o_cohost_management`, `o_network`, `s_cohost_earnings`, `v_network`, `wiz_invite_cohost`

### Leptos-only (examples)

`tenant/ratings`, `tenant/reservations`, `str_host/listing_index`, `landlord/setup`

## Remediation order

1. P0 — marketplace UUID; property documents; property hub + archive modal; unit message honesty  
2. P1 — destructive modals everywhere; nested systems; WO photos/receipts; assignment; UUID display  
3. P2 — chrome islands; persona dashboard composition (separate epics)

## Remediation log (2026-07-19)

| Item | Status |
|------|--------|
| Living audit + ROUTES stale banner | Done |
| Marketplace User ID paste removed | Done |
| Property documents group-by + honest Export | Done |
| Property hub Stitch density + archive modal | Done |
| Unit Message household/guest honesty modal | Done |
| Unit + leaf archive → quiet entry + modal | Done |
| Nested systems Create WO + condition pills | Done |
| WO job photos honest empty (no fake strip) | Done |
| Deal assignment demoted (no theatrical assign) | Done |
| Vault / ledger UUID display cleanup | Done |
| Map + Building Systems → PageHeader; Meridian subtitle | Done |
| Persona hub composition epics | Queued below (not boiled into this pass) |

### Pass 2 (screenshot validation)

| Item | Status |
|------|--------|
| Shared `.modal-*` CSS (overlay + header close) | Done |
| Hub Management icons + Delegate Not available | Done |
| Unit Open WOs View all → folio-btn | Done |
| Archive closes when switching to Units tab | Done |
| Maintenance history: Log paid form first | Done |
| Deal Ops New acquisition modal + Material close | Done |
| Wholesale kanban counts visible on light theme | Done |
| Messages contrast + empty-state / no emoji | Done |
| Account Billing honesty subtitle | Done |

### Pass 3 (address / occupancy / media / purge)

| Item | Status |
|------|--------|
| Assets list: street address primary | Done |
| Unit pills: Vacant/Occupied + mode (not ACTIVE) | Done |
| Dashboard STR-eligible icon (no VACATION ligature) | Done |
| Maint history 2-col + WO radio + receipt Not available | Done |
| Hub Photos + map peek → portfolio map `?asset_id=` | Done |
| Activity / timeline photo layout slots | Done |
| More → Archive… / Delete permanently… (PURGE API) | Done |
| Stitch: no Danger zone; photos.html; timeline thumbs | Done |

### Pass 4 (places + people)

| Item | Status |
|------|--------|
| Ops map light tiles + beach_access STR KPI + occupancy Vacant | Done |
| Hub Leaflet mini-map + geocode / Add location | Done |
| Coords PUT + Nominatim geocode + create-time geocode | Done |
| Draft occupancy (Add tenant) + activate (Attach lease) | Done |
| Full New lease path + offline branch + query prefill | Done |
| Applications inbox + Offer lease after Approve | Done |
| Dual Availability: Seeking lease + STR open/listed | Done |
| Stitch: add tenant / lease create / attach / applications / dual Availability | Done |
| Purge deepest-order + uuid_in_list unit tests | Done |

## Persona hub backlog (P2)

Queue as separate epics (do not boil into one landlord PR):

- [ ] Tenant dashboard / payments / maintenance composition vs `t_*`
- [ ] Vendor dashboard / schedule / invoices vs `v_*`
- [ ] STR host dashboard / reservations / incidents vs `s_*`
- [ ] Owner dashboard vs `o_dashboard`
- [ ] PMC dashboard / client book vs `p_*`

### Pass 5 (hub / unit ops hardening)

| Item | Status |
|------|--------|
| WO create sheet: dismiss navigates (no blank page); Schedule date | Done |
| Applications landlord access (no 403) | Done |
| Asset picker labels: street · name | Done |
| Hub mini-map stable after geocode | Done |
| Household enums + folio-btn CTAs + Depart + occupant profile | Done |
| Photos honesty → Documents; preferred contractor assign | Done |
| Property details + est. value + capital/equity; NOI/cap derived | Done |
| Stitch: WO sheet, occupant profile, hub details/capital | Done |

*Last updated: 2026-07-19 (Pass 5 hub ops)*
