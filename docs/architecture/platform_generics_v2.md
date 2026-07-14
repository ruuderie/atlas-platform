# Atlas Platform — Generics v2 (Consolidated)

> [!WARNING]
> **SUPERSEDED for G32+ design.** Prefer [`platform_generics_v3.md`](./platform_generics_v3.md) for G-32+.  
> **Ground-truth status** (backend + frontend) is always [`../CURRENT_STATE.md`](../private/archive/CURRENT_STATE.md) (Rev 11 — July 11, 2026).

> **Status:** Implemented & Merged to `dev` — status columns synced July 11, 2026 (Rev 11)
> **Date:** 2026-05-27 (original design) → June 2026 (G01–G31) → July 11, 2026 (status sync)
> **Branch History:** `feat/platform-generics-v2` → merged to `dev`
> **Purpose:** Historical + status registry for G01–G31 design rationale. Do not invent new G-numbers here — use Rule 7 in [`generic_fitness_test.md`](./generic_fitness_test.md).
>
> **See also:** [`../CURRENT_STATE.md`](../private/archive/CURRENT_STATE.md) — authoritative implementation registry.

---

## 1. Philosophy & Rule 7

**Core Principle:** Before any AtlasApp writes a net-new table, it must prove that none of the existing platform generics can satisfy the need.

This rule exists to prevent the platform from becoming a collection of slightly different CRMs, asset systems, case systems, and document stores.

**Living fitness procedure:** [`generic_fitness_test.md`](./generic_fitness_test.md) (includes USE EXISTING / EXTEND JSONB / EXTEND COMPANION). The questions below are preserved for history:

1. Which existing generic comes closest?
2. What specific field or behavior is missing?
3. Can it be modeled as `*_type` + `*_metadata JSONB` + app-level service typing?
4. If truly not, what is the cross-app benefit that justifies promoting it to a new generic?

Only after passing the Fitness Test may a new migration be added to an `AtlasApp::migrations()`.

---

## 2. All Generics — Quick Reference (G01–G31+)

Status columns match [`../CURRENT_STATE.md`](../private/archive/CURRENT_STATE.md) Rev 11.

### Infrastructure Layer (G01–G08) — All Deployed ✅

| ID | Name | Core Need | Key Tables | Backend Status | Frontend Status |
|----|------|-----------|------------|----------------|-----------------|
| 01 | `atlas_geo` | Spatial / PostGIS queries | `geo_service_areas`, `atlas_geo` | Deployed with API | Partial UI |
| 02 | `atlas_vault` | Secure file storage + sharing | `attachment`, `attachment_share_tokens`, `attachment_multipart_uploads` | Deployed with API | Partial UI |
| 03 | `atlas_payments` | Multi-rail payment ledger | `atlas_ledger_entries`, `atlas_ledger_splits`, `atlas_payment_credentials` | Deployed with API | **Full UI** |
| 04 | `atlas_subscriptions` | B2C recurring billing | `atlas_subscriptions` | Deployed with API | Partial UI |
| 05 | `atlas_syndication` | Outbound syndication + external integration event bus | `atlas_external_integrations`, `atlas_syndication_offer`, `atlas_app_instance_syndication`, `atlas_syndication_outbox`, `atlas_integration_events` | Deployed with API | **Full UI** |
| 06 | `atlas_verification_queue` | Human + automated trust workflows | `atlas_verification_requests` (+ reviewer notes) | Deployed with API | **Full UI** |
| 07 | `atlas_realtime` | WebSocket entity rooms | `atlas_ws_rooms`, `atlas_ws_messages` | Deployed with API | No UI |
| 08 | `atlas_ai_tasks` | Async LLM / model work queue | `atlas_ai_tasks` | Deployed with API | **Full UI** |

### Domain Object Layer (G09–G18) — All Deployed ✅

| ID | Name | Core Need | Key Tables | Backend Status | Frontend Status |
|----|------|-----------|------------|----------------|-----------------|
| 09 | `atlas_portfolios` | Grouping of assets for reporting/billing/access | `atlas_portfolios` | Deployed with API | **Full UI** |
| 10 | `atlas_assets` | Physical or digital ledger items | `atlas_assets` | Deployed with API | **Full UI** |
| 11 | `atlas_contracts` | Legal agreements | `atlas_contracts` | Deployed with API | **Full UI** |
| 12 | `atlas_service_providers` | Vendors, contractors, agents, adjusters | `atlas_service_providers` | Deployed with API | **Full UI** |
| 13 | `atlas_cases` | Work items, tickets, claims, tasks | `atlas_cases` | Deployed with API | **Full UI** |
| 14 | `atlas_documents` | Documents with metadata, e-sig, versioning | `atlas_documents` | Deployed with API | Partial UI |
| 15 | `atlas_opportunities` | Deal / pipeline objects | `atlas_opportunities` | Deployed with API | Partial UI |
| 16 | `atlas_regulatory_registrations` | Government permits, licenses, STR registrations | `atlas_regulatory_registrations` | Deployed with API | **Full UI** |
| 17 | `atlas_tax_events` + `atlas_tax_filings` | Taxable revenue events and periodic filings | `atlas_tax_events`, `atlas_tax_filings` | Deployed with API | Partial UI |
| 18 | `atlas_applications` | Structured multi-step intake / onboarding | `atlas_applications` | Deployed with API | Partial UI |

### Round 1 Gap-Fill Additions (G19–G26) — All Deployed ✅

| ID | Name | Core Need | Key Tables | Backend Status | Frontend Status |
|----|------|-----------|------------|----------------|-----------------|
| 19 | `atlas_campaigns` | Marketing campaign + enrollments + events | `atlas_campaigns` (+ `global_name`), `atlas_campaign_enrollments`, `atlas_campaign_events` | Deployed with API | **Full UI** |
| 20 | `atlas_attribution` | Marketing touchpoint attribution | `atlas_attribution_touchpoints` | Deployed with API | No UI |
| 21 | `atlas_events` | Managed events with ticketing + registration | `atlas_events`, `atlas_event_registrations`, `atlas_event_ticket_types` | Deployed with API | No UI |
| 22 | `atlas_record_relationships` | Polymorphic typed edge between entity types | `atlas_record_relationships` | Deployed with API | No UI |
| 23 | `atlas_reservations` | Time-bound reservations + availability | `atlas_reservations`, `atlas_availability` (+ `atlas_bookings`) | Deployed with API | **Full UI** |
| 24 | `atlas_quotes` | Quote + line-item pricing proposals | `atlas_quotes`, `atlas_quote_line_items` | Deployed with API | No UI |
| 25 | `atlas_commission_plans` | Commission agreement governing ledger splits | `atlas_commission_plans`, `atlas_commission_plan_splits` | Deployed with API | No UI |
| 26 | `atlas_catalog` | Product/service catalog + rate rules | `atlas_catalog_entries`, `atlas_catalog_availability`, `atlas_catalog_rate_rules` | Deployed with API | **Full UI** |

### Round 2 CRM & Intelligence Layer (G27–G31) — Implemented ✅

| ID | Name | Core Need | Key Tables | Backend Status | Frontend Status |
|----|------|-----------|------------|----------------|-----------------|
| 27 | `atlas_scorecards` | Universal Structured Evaluation Engine | `atlas_scorecard_*` (+ template deployments) | Deployed with API | **Full UI** |
| 28 | `atlas_note` | Universal polymorphic note | `atlas_notes` | Entity defined | Partial UI |
| 29 | `atlas_activity` | Universal polymorphic activity log | `activity` | Entity defined | Partial UI |
| 31 | `atlas_lead` | Canonical lead/prospect lifecycle | `atlas_lead`, `atlas_lead_compat_view` | Deployed with API | **Full UI** |

> **Note:** G-30 was not assigned. For G-32+ see [`platform_generics_v3.md`](./platform_generics_v3.md) and CURRENT_STATE (G32–G37 deployed).

### Party Model (replaces legacy CRM)

| Name | Core Need | Key Tables | Backend Status | Frontend Status |
|------|-----------|------------|----------------|-----------------|
| `atlas_accounts` | Top-level party (individual \| organization) | `atlas_accounts` | Deployed with API | **Full UI** |
| `atlas_contacts` | Lightweight people records on an Account | `atlas_contacts` | Deployed with API | **Full UI** |

**Services:** `AccountService`, `ContactService` — see CURRENT_STATE Key Service Layer Facts.

---

## 3. Detailed Specifications

> The full DDL, Rust service patterns, and cross-app usage examples for GENERIC-01–08 live in the original document.
>
> **See:** [`platform_generics.md`](./platform_generics.md)

The detailed specifications for GENERIC-09 through GENERIC-18 live in the Property Management analysis.

**See:** [`../property-management/23_second_round_generics.md`](../property-management/23_second_round_generics.md)

**Payment-specific extension** (GENERIC-03):

**See:** [`../property-management/25_payment_rails_architecture.md`](../property-management/25_payment_rails_architecture.md)

---

## 4. Implementation Order (Authoritative — Complete)

### Phase 0-A: Infrastructure (G01–G08) — ✅ Deployed

### Phase 0-B: Domain Objects (G09–G18) — ✅ Deployed

### Phase Round 1–2: G19–G31 — ✅ Deployed (G28/G29 Entity defined)

**Rule:** Each generic must be registered in `CorePlatformApp::migrations()` in dependency order. App-specific migrations must come *after* all required generics.

---

## 5. Cross-App Benefit Matrix (as of 2026-05-26)

| Generic | Primary Driver | Strong Secondary Benefits |
|---------|----------------|---------------------------|
| `atlas_portfolios` | PM | ClaimSwift (adjuster pools), AgentLink (agency alliances), fleet/equipment apps |
| `atlas_assets` | PM | ClaimSwift (damaged asset), Direct Booking (hotel rooms), future fleet |
| `atlas_contracts` | PM | CoverFlow (insurance policies), Direct Booking (corp agreements), AgentLink |
| `atlas_service_providers` | PM | ClaimSwift (adjusters), AgentLink (agents), Guest Comms (housekeeping) |
| `atlas_cases` | PM | ClaimSwift (claims), Guest Comms (tasks), Nomad List (reports), compliance |
| `atlas_documents` | PM | CoverFlow, ClaimSwift, AgentLink, Famtasm |
| `atlas_opportunities` | PM | CoverFlow (submissions), Direct Booking (pipelines), AgentLink (onboarding) |
| `atlas_regulatory_registrations` | PM (STR) | ClaimSwift (licenses), AgentLink (licenses), Direct Booking (permits) |
| `atlas_tax_events` / `filings` | PM (TDT) | CoverFlow (premium tax), Clipping (1099-K), Direct Booking (occupancy tax) |
| `atlas_applications` | PM | AgentLink, Famtasm, Direct Booking, future mortgage/employment flows |

---

## 6. Open Risks & Questions (historical — still relevant)

1. **JSONB Ergonomics** — Heavy reliance on `*_metadata JSONB` in Rust service layers and Leptos forms.
2. **Polymorphic Query Performance** — Index strategy review for high-cardinality JSONB queries.
3. **Payment Liability & Key Management** — MOR + per-tenant credential encryption requires ongoing security review.
4. **Migration Ordering Complexity** — Long migration chain; strong test coverage required.
5. **Over-abstraction** — Prefer EXTEND COMPANION under an existing G-id before inventing a new G-number (Rule 7).

---

## 7. Implementation Notes & Test Environment Considerations

**G01 (Geo/PostGIS)**: The `CREATE EXTENSION postgis` step is tolerant of environments without PostGIS binaries. Production and CI should still enable PostGIS (recommended, not yet enforced in CI — see CURRENT_STATE).

---

## 8. Current Gaps & Next Build Priorities (synced Rev 11)

After G-27 through G-37, highest-value open items (see CURRENT_STATE Recommended Follow-Up):

1. **G-20 / G-21 / G-22 / G-24 / G-25 UI** — Backend deployed; no dedicated frontend.
2. **G-28/G-29 Full Services** — Promote handler-only notes/activities to `NoteService` / `ActivityService`.
3. **G-05 Inbound Webhook Handler** — Outbound delivery complete; inbound receiver not built.
4. **G-35 Notification bell** — Landlord inbox exists; other portals still lack a bell.
5. **Legacy CRM Teardown** — Dual-write bridge still active; drop after full deprecation.

**Resolved since original v2 gaps list:**
- ✅ shared-ui `configurator.rs` — built (G-27)
- ✅ G-27 HTTP handler layer — `scorecard_admin.rs` / `scorecard_entries.rs` deployed
- ✅ G-06 / G-08 Full UI — platform-admin + Folio paths (Rev 11)

---

**References**

- Ground truth: [`../CURRENT_STATE.md`](../private/archive/CURRENT_STATE.md)
- v3 (G32+): [`platform_generics_v3.md`](./platform_generics_v3.md)
- Original 8 generics: [`platform_generics.md`](./platform_generics.md)
- Fitness test: [`generic_fitness_test.md`](./generic_fitness_test.md)
- Layer Map: [`platform_layer_map.md`](./platform_layer_map.md)

---

&copy; Copyright Ruud Salym Erie & Oplyst International, LLC. All Rights Reserved.
