# Folio — Page Implementation Queue

> **How to use:** Pick up the next `[ ]` item. When done, mark `[x]` and commit.  
> Ordering is by implementation priority (highest value to operator first).  
> See [`stitch_to_leptos_prompt.md`](stitch_to_leptos_prompt.md) for the implementation workflow.

---

## P0 — Landlord Core (`/l/**`)
_The primary operator. Nothing else works until this works._

| Status | Page | Stitch dir | Leptos module | Route | Backend handler |
|--------|------|-----------|---------------|-------|-----------------|
| `[x]` | Dashboard | `l_dashboard` | `pages/landlord/dashboard.rs` | `/l` | `portfolio.rs` (summary) |
| `[x]` | Portfolio | `l_portfolio` | `pages/landlord/portfolio.rs` | `/l/portfolio` | `portfolio.rs` |
| `[x]` | Assets | `l_assets` | `pages/landlord/assets.rs` | `/l/assets` | `asset.rs` |
| `[x]` | Leases | `l_leases` | `pages/landlord/leases.rs` | `/l/leases` | `lease.rs` |
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
| `[ ]` | Maintenance Queue | `l_maintenance_queue` | `pages/landlord/maintenance_queue.rs` | `/l/maintenance` | `case.rs` |
| `[ ]` | Meridian Analytics | `l_meridian` | `pages/landlord/meridian.rs` | `/l/meridian` | `analytics` (G-27) |
| `[ ]` | Ledger | `l_ledger` | `pages/landlord/ledger.rs` | `/l/ledger` | `billing.rs` |
| `[ ]` | Communications | `l_communications` | `pages/landlord/communications.rs` | `/l/comms` | `comms.rs` |
| `[ ]` | Map Portfolio | `l_map_portfolio` | `pages/landlord/map_portfolio.rs` | `/l/map` | `portfolio.rs` |
| `[ ]` | Digital Vault | `l_digital_vault` | `pages/landlord/digital_vault.rs` | `/l/vault` | `file_attachments` |
| `[ ]` | Inspections | `l_inspections` | `pages/landlord/inspections.rs` | `/l/inspections` | `case.rs` |
| `[ ]` | Violations | `l_violations` | `pages/landlord/violations.rs` | `/l/violations` | `case.rs` |
| `[ ]` | Building Systems | `l_building_systems` | `pages/landlord/building_systems.rs` | `/l/systems` | `asset.rs` |
| `[ ]` | Unit Appliances | `l_unit_appliances` | `pages/landlord/unit_appliances.rs` | `/l/assets/:id/appliances` | `asset.rs` |
| `[ ]` | Syndication | `l_syndication` | `pages/landlord/syndication.rs` | `/l/syndication` | `syndication_admin.rs` |
| `[ ]` | Wholesaling | `l_wholesaling` | `pages/landlord/wholesaling.rs` | `/l/wholesaling` | TBD |
| `[ ]` | Listing Network Preview | `l_listing_network_preview` | `pages/landlord/listing_preview.rs` | `/l/assets/:id/preview` | `catalog.rs` |
| `[ ]` | Contractor Marketplace | `l_contractor_marketplace` | `pages/landlord/contractor_marketplace.rs` | `/l/marketplace` | `vendor.rs` |
| `[ ]` | Account Billing | `l_account_billing` | `pages/landlord/account_billing.rs` | `/l/account/billing` | `billing.rs` |
| `[ ]` | G27 Configurator | `l_g27_configurator` | `pages/landlord/meridian_config.rs` | `/l/meridian/configure` | G-27 analytics |
| `[ ]` | Asset Alerts | `l_asset_alerts` | `pages/landlord/asset_alerts.rs` | `/l/assets/:id/alerts` | `asset.rs` |

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
| `[ ]` | Inbox | `t_inbox` | `pages/tenant/inbox.rs` | `/t/inbox` | `comms.rs` |
| `[ ]` | Documents | `t_documents` | `pages/tenant/documents.rs` | `/t/docs` | file attachments |
| `[ ]` | Household | `t_household` | `pages/tenant/household.rs` | `/t/household` | `lease.rs` |
| `[ ]` | Payment History | `t_payment_history` | `pages/tenant/payment_history.rs` | `/t/payments/history` | `billing.rs` |
| `[ ]` | Profile | `t_profile` | `pages/tenant/profile.rs` | `/t/profile` | `user_accounts` |
| `[ ]` | Violations | `t_violations` | `pages/tenant/violations.rs` | `/t/violations` | `case.rs` |
| `[ ]` | Reports | `t_reports` | `pages/tenant/reports.rs` | `/t/reports` | `billing.rs` |
| `[ ]` | Maintenance Detail | `t_maintenance_detail` | `pages/tenant/maintenance_detail.rs` | `/t/maintenance/:id` | `case.rs` |
| `[ ]` | Application Status | `t_application_status` | `pages/tenant/application_status.rs` | `/t/application` | `application.rs` |

---

## P2 — Vendor (`/v/**`)
_Stubs exist — wire to real data._

| Status | Page | Stitch dir | Leptos module | Route | Backend handler |
|--------|------|-----------|---------------|-------|-----------------|
| `[x]` | Dashboard | `v_dashboard` | `pages/vendor/dashboard.rs` | `/v` | `vendor.rs` |
| `[x]` | Work Orders | `v_work_orders` | `pages/vendor/work_orders.rs` | `/v/work-orders` | `case.rs` |
| `[x]` | Invoices | `v_invoices` | `pages/vendor/invoices.rs` | `/v/invoices` | `billing.rs` |
| `[ ]` | Network Profile | `v_network_profile` | `pages/vendor/network_profile.rs` | `/v/profile` | `vendor.rs` |
| `[ ]` | Schedule | `v_schedule` | `pages/vendor/schedule.rs` | `/v/schedule` | `case.rs` |

---

## P3 — PMC (`/pmc/**`)
_Unlocks enterprise accounts. Requires `folio_mode = pmc`._

| Status | Page | Stitch dir | Leptos module | Route | Backend handler |
|--------|------|-----------|---------------|-------|-----------------|
| `[x]` | Dashboard | `p_analytics` | `pages/pmc/dashboard.rs` | `/pmc` | `portfolio.rs` |
| `[x]` | Client Book | `p_client_book` | `pages/pmc/client_book.rs` | `/pmc/clients` | `portfolio.rs` |
| `[ ]` | Client Detail | `p_client_detail` | `pages/pmc/client_detail.rs` | `/pmc/clients/:id` | `portfolio.rs` |
| `[ ]` | Maintenance Dispatch | `p_maintenance_dispatch` | `pages/pmc/maintenance_dispatch.rs` | `/pmc/maintenance` | `case.rs` |
| `[ ]` | Portfolio Map | `p_portfolio_map` | `pages/pmc/portfolio_map.rs` | `/pmc/map` | `portfolio.rs` |
| `[ ]` | Owner Statement Batch | `p_owner_statement_batch` | `pages/pmc/owner_statements.rs` | `/pmc/statements` | `billing.rs` |

---

## P4 — STR Host (`/s/**`)
_Standard mode, `listing_mode = str`. Folio hosts who run short-term rentals._

| Status | Page | Stitch dir | Leptos module | Route | Backend handler |
|--------|------|-----------|---------------|-------|-----------------|
| `[ ]` | Dashboard | `s_dashboard` | `pages/str_host/dashboard.rs` | `/s` | `reservation.rs` |
| `[ ]` | Calendar | `s_calendar` | `pages/str_host/calendar.rs` | `/s/calendar` | `reservation.rs` |
| `[ ]` | Reservation Manifest | `s_reservation_manifest` | `pages/str_host/reservations.rs` | `/s/reservations` | `reservation.rs` |
| `[ ]` | Listing Detail | `s_listing_detail` | `pages/str_host/listing.rs` | `/s/listings/:id` | `catalog.rs` |
| `[ ]` | Pricing Rules | `s_pricing_rules` | `pages/str_host/pricing.rs` | `/s/pricing` | `catalog.rs` |
| `[ ]` | Channel Manager | `s_channel_manager` | `pages/str_host/channels.rs` | `/s/channels` | `syndication_admin.rs` |
| `[ ]` | Guest Messaging | `s_guest_messaging` | `pages/str_host/messages.rs` | `/s/messages` | `comms.rs` |
| `[ ]` | Reviews | `s_reviews` | `pages/str_host/reviews.rs` | `/s/reviews` | TBD |
| `[ ]` | Syndication | `s_syndication` | `pages/str_host/syndication.rs` | `/s/syndication` | `syndication_admin.rs` |
| `[ ]` | Incidents / Violations | `s_incidents` | `pages/str_host/incidents.rs` | `/s/incidents` | `case.rs` |
| `[ ]` | Violation Filing | `s_violation_filing` | `pages/str_host/violation_file.rs` | `/s/violations/new` | `case.rs` |

---

## P5 — Owner (Passive Investor) (`/o/**`)
_Read-only financial visibility._

| Status | Page | Stitch dir | Leptos module | Route | Backend handler |
|--------|------|-----------|---------------|-------|-----------------|
| `[ ]` | Dashboard | `o_dashboard` | `pages/owner/dashboard.rs` | `/o` | `portfolio.rs` |
| `[ ]` | Property Detail | `o_property_detail` | `pages/owner/property.rs` | `/o/properties/:id` | `asset.rs` |
| `[ ]` | Statements | `o_statements` | `pages/owner/statements.rs` | `/o/statements` | `billing.rs` |
| `[ ]` | Distributions | `o_distributions` | `pages/owner/distributions.rs` | `/o/distributions` | `billing.rs` |
| `[ ]` | Maintenance Approval | `o_maintenance_approval` | `pages/owner/maintenance.rs` | `/o/maintenance` | `case.rs` |

---

## P6 — Wizards (Public, no auth shell)
_Onboarding flows. Public-facing but Folio-hosted._

| Status | Page | Stitch dir | Leptos module | Route | Notes |
|--------|------|-----------|---------------|-------|-------|
| `[ ]` | Renter Application | `wiz_renter_application` | `pages/marketing/renter_application.rs` | `/apply/:property_id` | Public |
| `[ ]` | Vendor Onboard | `wiz_vendor_onboard` | `pages/vendor/onboard.rs` | `/v/onboard` | Token-gated |
| `[ ]` | PMC Onboard | `wiz_pmc_onboard` | `pages/pmc/onboard.rs` | `/pmc/onboard` | Admin-initiated |
| `[ ]` | Maintenance Triage | `wiz_maintenance_triage` | `pages/tenant/maintenance_triage.rs` | `/t/maintenance/new` | Tenant-initiated |

---

## P7 — Public Pages
_Folio-hosted public surfaces (not Network Instance)._

| Status | Page | Stitch dir | Leptos module | Route | Notes |
|--------|------|-----------|---------------|-------|-------|
| `[x]` | Login | `pub_login` | `pages/login.rs` | `/login` | Done |
| `[ ]` | Marketing Landing | `pub_marketing` | `pages/marketing/market_landing_page.rs` | `/lp` | Folio brand page |
| `[ ]` | LTR Listings (embedded) | `pub_ltr_listings` | — | — | → Network Instance territory |
| `[ ]` | STR Listings (embedded) | `pub_str_listings` | — | — | → Network Instance territory |
| `[ ]` | Lead Portal | `pub_lead_portal` | `pages/marketing/lead_portal.rs` | `/leads/:token` | Token-gated |
| `[ ]` | Inquiry Confirm | `pub_inquiry_confirm` | `pages/marketing/inquiry_confirm.rs` | `/inquiry/thanks` | Post-form |
| `[ ]` | Vendor Job Link | `pub_vendor_job_link` | `pages/vendor/job_link.rs` | `/jobs/:token` | Token-gated |
| `[ ]` | NI Signup | `pub_network_instance_signup` | — | — | → Network Instance territory |

---

## Progress Summary

```
P0 Landlord:  14 done / 31 total   ██████░░░░░░░░░░░░░░░░░░  45%
P1 Tenant:     5 done / 14 total   ████░░░░░░░░░░░░░░░░░░░░  36%
P2 Vendor:     3 done /  5 total   ████████████░░░░░░░░░░░░  60%
P3 PMC:        2 done /  6 total   ████░░░░░░░░░░░░░░░░░░░░  33%
P4 STR Host:   0 done / 11 total   ░░░░░░░░░░░░░░░░░░░░░░░░   0%
P5 Owner:      0 done /  5 total   ░░░░░░░░░░░░░░░░░░░░░░░░   0%
P6 Wizards:    0 done /  4 total   ░░░░░░░░░░░░░░░░░░░░░░░░   0%
P7 Public:     1 done /  8 total   ██░░░░░░░░░░░░░░░░░░░░░░  12%
─────────────────────────────────────────────────────────
Total:        25 done / 84 total   ██████░░░░░░░░░░░░░░░░░░  30%
```

---

*Last updated: 2026-06-27. Source: `designs/stitch/project_pm/folio/ROUTES.md`*

<!-- session 2026-06-27: asset_detail.rs (G-13 + G-21 timeline, G-22 contractor panel), vendors.rs (full vendor grid + asset picker) -->
