# Folio — Page Implementation Queue

> **How to use:** Pick up the next `[ ]` item. When done, mark `[x]` and commit.  
> Ordering is by implementation priority (highest value to operator first).  
> See `docs/private/prompts/stitch_to_leptos_prompt.md` for the implementation workflow.

---

## P0 — Landlord Core (`/l/**`)
_The primary operator. Nothing else works until this works._

| Status | Page | Stitch dir | Leptos module | Route | Backend handler |
|--------|------|-----------|---------------|-------|-----------------|
| `[x]` | Dashboard (LTR/STR/All modes) | `l_dashboard` | `pages/landlord/dashboard.rs` | `/l` | `assets.rs`, `leases`, `maintenance` |
| `[x]` | Portfolio (redirect → Assets) | `l_portfolio` | `pages/landlord/portfolio.rs` | `/l/portfolio` → `/l/assets` | backend G-portfolio stays isolation-only |
| `[x]` | Assets (operator portfolio gallery) | `l_assets` | `pages/landlord/assets.rs` | `/l/assets` | `assets.rs` |
| `[x]` | Add property | — | `pages/landlord/asset_create.rs` | `/l/assets/new` | `assets.rs` POST |
| `[x]` | Leases | `l_leases` | `pages/landlord/leases.rs` | `/l/leases` | `lease.rs` |
| `[x]` | New lease | — | `pages/landlord/lease_create.rs` | `/l/leases/new` | `leases.rs` POST |
| `[x]` | Setup index | — | `pages/landlord/setup.rs` | `/l/setup` | (deep links) |
| `[x]` | Leads | `l_leads` | `pages/landlord/leads.rs` | `/l/leads` | `opportunity.rs` |
| `[x]` | Campaigns | `l_campaigns` | `pages/landlord/campaigns.rs` | `/l/campaigns` | `campaign.rs` |
| `[x]` | Billing | `l_billing` | `pages/landlord/billing.rs` | `/l/billing` | `billing.rs` |
| `[x]` | Vendors | `l_vendors` | `pages/landlord/vendors.rs` | `/l/vendors` | `vendor.rs` |
| `[x]` | STR Compliance | `l_str_compliance` | `pages/landlord/str_compliance.rs` | `/l/str` | `regulatory.rs` |
| `[x]` | Catalog | `l_catalog` | `pages/landlord/catalog.rs` | `/l/catalog` | `catalog.rs` |
| `[x]` | Reservations | `l_reservations` | `pages/landlord/reservations.rs` | `/l/reservations` | `reservation.rs` |
| `[x]` | Asset Detail | `l_asset_detail` | `pages/landlord/asset_detail.rs` | `/l/assets/:id` | `asset.rs`, `maintenance.rs` |
| `[x]` | Lease Detail | `l_lease_detail` | `pages/landlord/lease_detail.rs` | `/l/leases/:id` | `leases.rs` |
| `[x]` | Tenant Profile | `l_tenant_profile` | `pages/landlord/tenant_profile.rs` | `/l/tenants/:id` | `users.rs` (new) |
| `[x]` | Maintenance Queue | `l_maintenance_queue` | `pages/landlord/maintenance_queue.rs` | `/l/maintenance` | `maintenance.rs` |
| `[x]` | Meridian Analytics | `l_meridian` | `pages/landlord/meridian.rs` | `/l/meridian` | `reporting.rs` (`GET /api/folio/analytics/landlord`) |
| `[x]` | Ledger | `l_ledger` | `pages/landlord/ledger.rs` | `/l/ledger` | `billing.rs` |
| `[x]` | Communications | `l_communications` | `pages/landlord/communications.rs` | `/l/communications` | `atlas_ws_room.rs` |
| `[x]` | Notifications | `l_notifications` | `pages/landlord/notifications.rs` | `/l/notifications` | `atlas_notification.rs` |
| `[x]` | Ops Map (status / maintenance / STR layers) | `l_map_portfolio` | `pages/landlord/map_portfolio.rs` | `/l/map` | `assets.rs` (`GET /api/folio/assets/map`) |
| `[x]` | Digital Vault | `l_digital_vault` | `pages/landlord/digital_vault.rs` | `/l/vault` | `file_attachments` |
| `[x]` | Inspections | `l_inspections` | `pages/landlord/inspections.rs` | `/l/inspections` | `case.rs` |
| `[x]` | Violations | `l_violations` | `pages/landlord/violations.rs` | `/l/violations` | `violations.rs` |
| `[x]` | Building Systems | `l_building_systems` | `pages/landlord/building_systems.rs` | `/l/systems` | `asset.rs` |
| `[x]` | Deal Ops | `l_deal_workspace` / creative finance | `pages/landlord/deals.rs` | `/l/deals` | `wholesale` / deals |
| `[x]` | Deal Workspace | `l_deal_workspace` | `pages/landlord/deal_workspace.rs` | `/l/deals/:id` | deals |
| `[x]` | Deal Structure | `l_deal_structure` | `pages/landlord/deal_structure.rs` | `/l/deals/:id/structure` | deals |
| `[x]` | Buyers | `l_tenant_buyer_pipeline` | `pages/landlord/buyers.rs` | `/l/buyers` | deals |
| `[x]` | Unit Appliances | `l_unit_appliances` | `pages/landlord/unit_appliances.rs` | `/l/appliances` | `asset.rs` |
| `[x]` | Syndication | `l_syndication` | `pages/landlord/syndication.rs` | `/l/syndication` | `syndication_admin.rs` |
| `[x]` | Wholesaling | `l_wholesaling` | `pages/landlord/wholesaling.rs` | `/l/wholesaling` | TBD |
| `[x]` | Listing Network Preview | `l_listing_network_preview` | `pages/landlord/listing_preview.rs` | `/l/assets/:id/preview` | `catalog.rs` |
| `[x]` | Contractor Marketplace | `l_contractor_marketplace` | `pages/landlord/contractor_marketplace.rs` | `/l/marketplace` | `vendor.rs` |
| `[x]` | Account Billing | `l_account_billing` | `pages/landlord/account_billing.rs` | `/l/account/billing` | `billing.rs` |
| `[x]` | G27 Configurator | `l_g27_configurator` | `pages/landlord/meridian_config.rs` | `/l/meridian/configure` | G-27 analytics |
| `[x]` | Asset Alerts | `l_asset_alerts` | `pages/landlord/asset_alerts.rs` | `/l/assets/:id/alerts` | `asset.rs` |

---

## P0c — Landlord IA parity (operator-real) — done

> Assets = property list; `/l/portfolio` redirects; unit = first-class workspace; Deal Ops in nav; Dashboard LTR/STR/All; Map ops layers. Prompt: `docs/private/prompts/stitch_to_leptos_prompt.md`.

### Operator nav contract (2026-07)

- **Primary rail (~8):** Dashboard, Assets, Leases, Maintenance, Deals, Map, Messages, Billing.
- **Footer:** Setup (`/l/setup`), Analytics, Account, Settings — config/rare tools live in Setup, not the rail.
- **Hierarchy:** Related links on hubs (Assets → Vault/Systems/Appliances; Maintenance → Vendors/Marketplace/Inspections; Deals → Buyers; Billing → Ledger; Messages → Notifications).
- **Global search:** Cmd/Ctrl+K in landlord shell (nav, Setup, create actions, assets/leases/WOs).
- **Create surfaces:** `/l/leases/new`, `/l/assets/new` (+ CTAs on list/unit/dashboard).

### Action matrix (purpose → must-have write)

| Route | Job | Must-have | Status |
|-------|-----|-----------|--------|
| `/l/leases` | Manage contracts | Create lease | wired (`/l/leases/new`) |
| `/l/assets` | Holdings inventory | Add property | wired (`/l/assets/new`) |
| `/l/str` | STR permits | Register permit | wired |
| `/l/violations` | Compliance queue | File + status | wired |
| `/l/systems` | Systems registry | Add system | wired |
| `/l/appliances` | Appliance lifecycle | Add appliance | wired |
| `/l/vendors` / marketplace | Contractor network | Add vendor | wired (needs atlas `user_id`) |
| `/l/vault` | Document vault | Register document | wired (metadata / r2_key) |
| `/l/leads` | Prospect pipeline | Create + lifecycle | wired |
| `/l/campaigns` | Outreach | Create campaign | wired |
| `/l/ledger` | Audit trail | Post charge | wired |
| `/l/deals/:id` | Deal ops | Advance stage | wired |
| `/l/assets/:id/alerts` | Alert prefs | Persist prefs | wired (`GET`/`PUT …/alert-prefs`) |
| `/l/syndication` | Channel prefs | Read / deep-link only | **no persist API** (Save demoted) |
| `/l/reservations` | STR stay ops | Hold/confirm/check-in | wired (existing reservation APIs) |
| `/l/tenants/:id` | Applications | Approve/reject | wired (`PATCH …/decision`) |
| `/l/assets/:id` (Danger) | Soft archive | Type `DELETE` + blockers | wired (`POST …/archive`) |
| `/l/assets/:unitId/history` | Unit timeline | Historical lease / payments / maint | wired (`/history/lease`, `/history/payments`, `/history/maintenance`) |
| `/l/assets/:id` (hub) | Nested create | Add unit + New project | wired (`parent_asset_id`, `POST /projects`) |
| `/l/assets/:id` (unit Spaces) | Nested create | Add space | wired |
| `/l/assets/:id` (Lease & household) | Household writes | Add/depart occupant + vehicle | wired |
| `/l/maintenance/:id` · history expense | Expense ↔ WO | Link cost to unit WO (`related_case_id`) or standalone | wired |
| `/l/systems` · appliances | Retire → inactive | Reason + replace chain | wired (`POST …/retire`) |
| `/l/vault` | Document vault | Presign → R2 → register | wired (`POST …/vault/presign`) |

---

## P0b — Multi-unit hub / Projects / G-27 (production bar)

> Stitch is complete under `designs/stitch/project_pm/folio/`. Implement via `docs/private/prompts/stitch_to_leptos_prompt.md`: API mapping → token map → `Resource`/`Suspense`/skeleton → parity. No stubs, no CDN Tailwind ports, no mock data in `view!`. Quality bar: see implement plan **Production quality bar**.

| Status | Page | Stitch dir | Leptos module | Route | Backend handler |
|--------|------|-----------|---------------|-------|-----------------|
| `[x]` | Property Hub | `l_property_hub` | `pages/landlord/property_hub.rs` | `/l/assets/:id` (parent dispatch) | `assets.rs`, `maintenance.rs`, `projects.rs` |
| `[x]` | Unit workspace (first-class) | `l_unit_detail` | `pages/landlord/unit_detail.rs` | `/l/assets/:id` (unit dispatch) | `assets.rs`, `leases`, `household`, `maintenance` |
| `[x]` | Nested Building Systems | `l_building_systems` | `pages/landlord/property_systems.rs` | `/l/assets/:id/systems` | `building_systems.rs` |
| `[x]` | Property Documents | `l_property_documents` | `pages/landlord/property_documents.rs` | `/l/assets/:id/documents` | `assets.rs` (compose), `vault.rs` |
| `[x]` | Work Order Create | `l_work_order_create` | `pages/landlord/work_order_create.rs` | `/l/maintenance/new` | `maintenance.rs` |
| `[x]` | Work Order Detail | `l_work_order_detail` | `pages/landlord/work_order_detail.rs` | `/l/maintenance/:id` | `maintenance.rs` |
| `[x]` | Project Detail | `l_project_detail` | `pages/landlord/project_detail.rs` | `/l/projects/:id` | `projects.rs` |
| `[x]` | Landlord Ratings (full) | `l_ratings` | `pages/landlord/ratings.rs` | `/l/ratings` | `scorecards.rs` |
| `[x]` | Maintenance Queue polish | `l_maintenance_queue` | `pages/landlord/maintenance_queue.rs` | `/l/maintenance` | `maintenance.rs` |
| `[x]` | Tenant Portal stub | `l_tenant_portal_content` | `pages/landlord/tenant_portal_content.rs` | `/l/assets/:id/portal` | placeholder (CMS out of scope) |

**Shared Folio components (before page ports):** `PropertyTabBar`, `ActivityRail`, `StatusPill`, `InterruptibleSheet`, `PhotoLightbox` — `apps/folio/src/components/` + `.hub-*` / `.proj-*` / `.ratings-*` in `style/main.css`.

**Parity checklist:** [`docs/folio/multi_unit_parity_checklist.md`](multi_unit_parity_checklist.md)

---

## P1 — Tenant Core (`/t/**`)
_Tenant retention. Their UX determines renewal rates._

| Status | Page | Stitch dir | Leptos module | Route | Backend handler |
|--------|------|-----------|---------------|-------|-----------------|
| `[x]` | Dashboard | `t_dashboard` | `pages/tenant/dashboard.rs` | `/t` | `lease.rs` |
| `[x]` | My Lease | `t_lease_detail` | `pages/tenant/my_lease.rs` | `/t/my-lease` | `lease.rs` |
| `[x]` | Payments | `t_payments` | `pages/tenant/payments.rs` | `/t/payments` | `billing.rs` |
| `[x]` | Maintenance | `t_maintenance` | `pages/tenant/maintenance.rs` | `/t/maintenance` | `case.rs` |
| `[x]` | Reservations | `t_reservations` | `pages/tenant/reservations.rs` | `/t/reservations` | `reservation.rs` |
| `[x]` | Inbox | `t_inbox` | `pages/tenant/inbox.rs` | `/t/inbox` | `comms.rs` |
| `[x]` | Documents | `t_documents` | `pages/tenant/documents.rs` | `/t/docs` | file attachments |
| `[x]` | Household | `t_household` | `pages/tenant/household.rs` | `/t/household` | `lease.rs` |
| `[x]` | Payment History | `t_payment_history` | `pages/tenant/payment_history.rs` | `/t/payments/history` | `billing.rs` |
| `[x]` | Profile | `t_profile` | `pages/tenant/profile.rs` | `/t/profile` | `user_accounts` |
| `[x]` | Violations | `t_violations` | `pages/tenant/violations.rs` | `/t/violations` | `case.rs` |
| `[x]` | Reports | `t_reports` | `pages/tenant/reports.rs` | `/t/reports` | `billing.rs` |
| `[x]` | Maintenance Detail | `t_maintenance_detail` | `pages/tenant/maintenance_detail.rs` | `/t/maintenance/:id` | `case.rs` |
| `[x]` | Application Status | `t_application_status` | `pages/tenant/application_status.rs` | `/t/application` | `application.rs` |

---

## P2 — Vendor (`/v/**`)
_Stubs exist — wire to real data._

| Status | Page | Stitch dir | Leptos module | Route | Backend handler |
|--------|------|-----------|---------------|-------|-----------------|
| `[x]` | Dashboard | `v_dashboard` | `pages/vendor/dashboard.rs` | `/v` | `vendor.rs` |
| `[x]` | Work Orders | `v_work_orders` | `pages/vendor/work_orders.rs` | `/v/work-orders` | `case.rs` |
| `[x]` | Invoices | `v_invoices` | `pages/vendor/invoices.rs` | `/v/invoices` | `billing.rs` |
| `[x]` | Network Profile | `v_network_profile` | `pages/vendor/network_profile.rs` | `/v/profile` | `vendor.rs` |
| `[x]` | Schedule | `v_schedule` | `pages/vendor/schedule.rs` | `/v/schedule` | `case.rs` |

---

## P3 — PMC (`/pmc/**`)
_Unlocks enterprise accounts. Requires `folio_mode = pmc`._

| Status | Page | Stitch dir | Leptos module | Route | Backend handler |
|--------|------|-----------|---------------|-------|-----------------|
| `[x]` | Dashboard | `p_analytics` | `pages/pmc/dashboard.rs` | `/pmc` | `portfolio.rs` |
| `[x]` | Client Book | `p_client_book` | `pages/pmc/client_book.rs` | `/pmc/clients` | `portfolio.rs` |
| `[x]` | Client Detail | `p_client_detail` | `pages/pmc/client_detail.rs` | `/pmc/clients/:id` | `portfolio.rs` |
| `[x]` | Maintenance Dispatch | `p_maintenance_dispatch` | `pages/pmc/maintenance_dispatch.rs` | `/pmc/maintenance` | `case.rs` |
| `[x]` | Portfolio Map | `p_portfolio_map` | `pages/pmc/portfolio_map.rs` | `/pmc/map` | `portfolio.rs` |
| `[x]` | Owner Statement Batch | `p_owner_statement_batch` | `pages/pmc/owner_statements.rs` | `/pmc/statements` | `billing.rs` |

---

## P4 — STR Host (`/s/**`)
_Standard mode, `listing_mode = str`. Folio hosts who run short-term rentals._

| Status | Page | Stitch dir | Leptos module | Route | Backend handler |
|--------|------|-----------|---------------|-------|-----------------|
| `[x]` | Dashboard | `s_dashboard` | `pages/str_host/dashboard.rs` | `/s` | `reservation.rs` |
| `[x]` | Calendar | `s_calendar` | `pages/str_host/calendar.rs` | `/s/calendar` | `reservation.rs` |
| `[x]` | Reservation Manifest | `s_reservation_manifest` | `pages/str_host/reservations.rs` | `/s/reservations` | `reservation.rs` |
| `[x]` | Listing Detail | `s_listing_detail` | `pages/str_host/listing.rs` | `/s/listings/:id` | `catalog.rs` |
| `[x]` | Pricing Rules | `s_pricing_rules` | `pages/str_host/pricing.rs` | `/s/pricing` | `catalog.rs` |
| `[~]` | Channel Manager | `s_channel_manager` | `pages/str_host/channels.rs` | `/s/channels` | **blocked: no Folio OTA channels persistence API** |
| `[x]` | Guest Messaging | `s_guest_messaging` | `pages/str_host/messages.rs` | `/s/messages` | `comms.rs` |
| `[~]` | Reviews | `s_reviews` | `pages/str_host/reviews.rs` | `/s/reviews` | **blocked: no Folio STR reviews API** |
| `[x]` | Syndication | `s_syndication` | `pages/str_host/syndication.rs` | `/s/syndication` | `syndication_admin.rs` |
| `[x]` | Incidents / Violations | `s_incidents` | `pages/str_host/incidents.rs` | `/s/incidents` | `case.rs` |
| `[x]` | Violation Filing | `s_violation_filing` | `pages/str_host/violation_file.rs` | `/s/violations/new` | `case.rs` |

---

## P5 — Owner (Passive Investor) (`/o/**`)
_Read-only financial visibility._

| Status | Page | Stitch dir | Leptos module | Route | Backend handler |
|--------|------|-----------|---------------|-------|-----------------|
| `[x]` | Dashboard | `o_dashboard` | `pages/owner/dashboard.rs` | `/o` | `portfolio.rs` |
| `[x]` | Property Detail | `o_property_detail` | `pages/owner/property.rs` | `/o/properties/:id` | `asset.rs` |
| `[x]` | Statements | `o_statements` | `pages/owner/statements.rs` | `/o/statements` | `billing.rs` |
| `[x]` | Distributions | `o_distributions` | `pages/owner/distributions.rs` | `/o/distributions` | `billing.rs` |
| `[x]` | Maintenance Approval | `o_maintenance_approval` | `pages/owner/maintenance.rs` | `/o/maintenance` | `case.rs` |

---

## P6 — Wizards (Public, no auth shell)
_Onboarding flows. Public-facing but Folio-hosted._

| Status | Page | Stitch dir | Leptos module | Route | Notes |
|--------|------|-----------|---------------|-------|-------|
| `[x]` | Renter Application | `wiz_renter_application` | `pages/marketing/renter_application.rs` | `/apply/:property_id` | Public |
| `[x]` | Vendor Onboard | `wiz_vendor_onboard` | `pages/vendor/onboard.rs` | `/v/onboard` | Token-gated |
| `[x]` | PMC Onboard | `wiz_pmc_onboard` | `pages/pmc/onboard.rs` | `/pmc/onboard` | Admin-initiated |
| `[x]` | Maintenance Triage | `wiz_maintenance_triage` | `pages/tenant/maintenance_triage.rs` | `/t/maintenance/new` | Tenant-initiated |

---

## P7 — Public Pages
_Folio-hosted public surfaces (not Network Instance)._

| Status | Page | Stitch dir | Leptos module | Route | Notes |
|--------|------|-----------|---------------|-------|-------|
| `[x]` | Login | `pub_login` | `pages/login.rs` | `/login` | Done |
| `[x]` | Marketing Landing | `pub_marketing` | `pages/marketing/market_landing_page.rs` | `/lp` | Folio brand page |
| `[x]` | LTR Listings (embedded) | `pub_ltr_listings` | `pages/marketing/ltr_listings.rs` | `/listings/ltr` | Embeddable via ?embed=1 |
| `[x]` | STR Listings (embedded) | `pub_str_listings` | `pages/marketing/str_listings.rs` | `/listings/str` | Embeddable via ?embed=1 |
| `[x]` | Lead Portal | `pub_lead_portal` | `pages/marketing/lead_portal.rs` | `/leads/:token` | Token-gated |
| `[x]` | Inquiry Confirm | `pub_inquiry_confirm` | `pages/marketing/inquiry_confirm.rs` | `/inquiry/thanks` | Post-form |
| `[x]` | Vendor Job Link | `pub_vendor_job_link` | `pages/vendor/job_link.rs` | `/jobs/:token` | Token-gated |
| `[x]` | NI Signup | `pub_network_instance_signup` | `pages/marketing/ni_signup.rs` | `/ni/signup` | Self-serve operator onboarding |

---

## Parked — blocked: no Folio API (this epic)

Do **not** invent backends here. Keep nav/routes if already scaffolded; UI stays honest empty or marketing-only.

| Namespace | Routes | Notes |
|-----------|--------|-------|
| Agent | `/a/**` | Product + APIs out of scope |
| Broker | `/br/**` (nav `/b/**` orphans) | Product + APIs out of scope |
| Guest | `/g/**` | Nav orphans; guest page set not built |
| STR reviews / OTA channels | `/s/reviews`, `/s/channels`, `/s/pricing`, `/s/listings/:id` | Honest empty / read-only until persistence APIs |
| Cohost marketplace | (if present) | No marketplace API |
| Tenant portal CMS | `/l/assets/:id/portal` | Intentional stub — CMS out of scope |
| Syndication channel persist | `/l/syndication` Save | Explicitly excluded — no GET/PUT channels API |
| Deal lease-option create-on-page | `/l/buyers` | CTA to Deal Ops disposition (wired) |
| Communications start room | `/l/comms` | Messaging graph |
| Campaign enroll | Campaigns | Marketing |
| Vendor create without Atlas `user_id` | Vendors | Identity/onboarding |
| STR historical reservation import | Unit history | After LTR unit history ships |
| Hard physical purge of history | Archive | Soft-archive only |

---

## Progress Summary

```
P0 Landlord:  32 done / 32 total   ████████████████████████ 100%
P0b Multi-unit: 10 done / 10 total ████████████████████████ 100%
P1 Tenant:    14 done / 14 total   ████████████████████████ 100%
P2 Vendor:     6 done /  6 total   ████████████████████████ 100%
P3 PMC:        6 done /  6 total   ████████████████████████ 100%
P4 STR Host:   9 done / 11 (+2 parked) ██████████████████░░ 82%
P5 Owner:      5 done /  5 total   ████████████████████████ 100%
P6 Wizards:    4 done /  4 total   ████████████████████████ 100%
P7 Public:     8 done /  8 total   ████████████████████████ 100%
─────────────────────────────────────────────────────────
Total: wired cores complete; agent/broker/guest/STR reviews-channels parked
```

*Last updated: 2026-07-18. Gaps-close epic: unit History timelines + historical lease / payment / maint sheets; nested unit/space create; household writes; hub projects; honesty on syndication/billing/STR host.*

<!-- session 2026-06-28: meridian_config.rs (G-27 dashboard/rules/surfaces), ltr_listings.rs, str_listings.rs, ni_signup.rs -->

---

*Source: `designs/stitch/project_pm/folio/ROUTES.md`*

<!-- session 2026-06-27: asset_detail.rs (G-13 + G-21 timeline, G-22 contractor panel), vendors.rs (full vendor grid + asset picker) -->
