# Atlas Platform — Generics v2 (Consolidated)

> **Status:** Implemented & Merged to dev (June 2026)
> **Date:** 2026-05-27 (original design) → June 2026 (implementation complete)
> **Branch History:** `feat/platform-generics-v2` → merged to `dev`
> **Purpose:** Single source of truth for all 18 platform generics. Supersedes the original 8 + the Round 2/3 analysis from Property Management.
>
> **See also:** `../CURRENT_STATE.md` for the absolute latest high-level summary.

---

## 1. Philosophy & Rule 7

**Core Principle:** Before any AtlasApp writes a net-new table, it must prove that none of the 18 platform generics can satisfy the need.

This rule exists to prevent the platform from becoming a collection of 13 slightly different CRMs, asset systems, case systems, and document stores.

**The Fitness Test (new for v2):**

When an app author wants to introduce a new table, they must answer in `atlas_app_integration.md` style:

1. Which of the 18 generics comes closest?
2. What specific field or behavior is missing?
3. Can it be modeled as `*_type` + `*_metadata JSONB` + app-level service typing?
4. If truly not, what is the cross-app benefit that justifies promoting it to a new generic?

Only after passing the Fitness Test may a new migration be added to an `AtlasApp::migrations()`.

---

## 2. The 18 Generics — Quick Reference

### Infrastructure Layer (GENERIC-01–08)

| ID | Name | Core Need | Key Tables |
|----|------|-----------|------------|
| 01 | `atlas_geo` | Spatial / PostGIS queries | `geo_service_areas` |
| 02 | `atlas_vault` | Secure file storage + sharing | `attachment` (extended), `attachment_share_tokens`, `attachment_multipart_uploads` |
| 03 | `atlas_payments` | Multi-rail payment ledger | `atlas_ledger_entries`, `atlas_ledger_splits` |
| 04 | `atlas_subscriptions` | B2C recurring billing | `atlas_subscriptions` |
| 05 | `atlas_external_integrations` | Third-party connectors (PMS/OTA/AMS) | `atlas_external_integrations`, `atlas_integration_events` |
| 06 | `atlas_verification_queue` | Human + automated trust workflows | `atlas_verification_requests` |
| 07 | `atlas_realtime` | WebSocket entity rooms | `atlas_ws_rooms`, `atlas_ws_messages` |
| 08 | `atlas_ai_tasks` | Async LLM / model work queue | `atlas_ai_tasks` |

### Domain Object Layer (GENERIC-09–17)

| ID | Name | Core Need | Key Tables |
|----|------|-----------|------------|
| 09 | `atlas_portfolios` | Grouping of assets for reporting/billing/access | `atlas_portfolios` |
| 10 | `atlas_assets` | Physical or digital ledger items (properties, units, vehicles, equipment, hotel rooms) | `atlas_assets` (with `asset_type` + `parent_asset_id` + `attributes JSONB`) |
| 11 | `atlas_contracts` | Legal agreements (leases, policies, rate agreements, SLAs) | `atlas_contracts` (with `contract_type` + `terms_metadata JSONB`) |
| 12 | `atlas_service_providers` | Vendors, contractors, agents, adjusters | `atlas_service_providers` (with `scope` + `service_categories`) |
| 13 | `atlas_cases` | Work items, tickets, claims, tasks, compliance alerts | `atlas_cases` (with `case_type` + `case_metadata JSONB`) |
| 14 | `atlas_documents` | Documents with metadata, e-sig, versioning, polymorphic linkage | `atlas_documents` (with `document_category` + `app_namespace`) |
| 15 | `atlas_opportunities` | Deal / pipeline objects with financial modeling | `atlas_opportunities` (with `opportunity_type` + `financial_inputs/outputs JSONB`) |
| 16 | `atlas_regulatory_registrations` | Government permits, licenses, STR registrations | `atlas_regulatory_registrations` (with `registration_type` + `jurisdiction_metadata JSONB`) |
| 17 | `atlas_tax_events` + `atlas_tax_filings` | Taxable revenue events and periodic filings | Two tables for event-level + periodic filing |

### Intake & Onboarding Layer (GENERIC-18)

| ID | Name | Core Need | Key Tables |
|----|------|-----------|------------|
| 18 | `atlas_applications` | Structured multi-step intake, screening, and onboarding workflows | `atlas_applications` (with `application_type` + `application_metadata JSONB`) |

---

## 3. Detailed Specifications

> The full DDL, Rust service patterns, and cross-app usage examples for GENERIC-01–08 live in the original document.
>
> **See:** [`platform_generics.md`](./platform_generics.md)

The detailed specifications for GENERIC-09 through GENERIC-18 (including all DDL, index strategies, polymorphic patterns, and PM/ClaimSwift/Direct Booking mapping examples) live in the Property Management analysis.

**See:** [`../property-management/23_second_round_generics.md`](../property-management/23_second_round_generics.md)

**Payment-specific extension** (new table + adapter trait for GENERIC-03):

**See:** [`../property-management/25_payment_rails_architecture.md`](../property-management/25_payment_rails_architecture.md) — `atlas_payment_credentials` + `PaymentRailAdapter` trait.

---

## 4. Implementation Order (Authoritative)

### Phase 0-A: Infrastructure (Blocker for everything)

1. `atlas_vault` (G-02)
2. `atlas_payments` (G-03) — including the `atlas_payment_credentials` extension
3. `atlas_geo` (G-01) — PostGIS extension
4. `atlas_external_integrations` (G-05)
5. `atlas_verification_queue` (G-06)
6. `atlas_realtime` (G-07)
7. `atlas_subscriptions` (G-04)
8. `atlas_ai_tasks` (G-08)

### Phase 0-B: Domain Objects (Blocker for PM + several other apps)

9. `atlas_portfolios` (G-09)
10. `atlas_assets` (G-10)
11. `atlas_contracts` (G-11)
12. `atlas_service_providers` (G-12)
13. `atlas_cases` (G-13)
14. `atlas_documents` (G-14)
15. `atlas_opportunities` (G-15)
16. `atlas_regulatory_registrations` (G-16)
17. `atlas_tax_events` + `atlas_tax_filings` (G-17)
18. `atlas_applications` (G-18)

**Rule:** Each generic must be registered in `CorePlatformApp::migrations()` in the exact order above. App-specific migrations must come *after* all required generics.

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

## 6. Open Risks & Questions

1. **JSONB Ergonomics** — How painful will the heavy reliance on `*_metadata JSONB` be in Rust service layers and Leptos forms? (Needs POC validation)
2. **Polymorphic Query Performance** — Heavy use of `*_type` columns + JSONB queries on hot tables. Index strategy and query patterns need review.
3. **Payment Liability & Key Management** — The two-layer MOR + per-tenant credential encryption design is powerful but high-risk. Requires security review before production use.
4. **Migration Ordering Complexity** — 18 generics + app migrations creates a long, fragile startup sequence. We need strong test coverage and clear failure messages.
5. **Over-abstraction** — Is 18 the right number, or will some of these later be found to need app-specific extensions anyway?

---

## 7. Implementation Notes & Test Environment Considerations

**G01 (Geo/PostGIS)**: The `CREATE EXTENSION postgis` step is now tolerant of environments without the PostGIS binaries installed. In such cases it logs a warning and skips table creation. This was necessary to keep the test suite healthy after the v2 merge. Production and proper test environments should still have PostGIS enabled.

---

## 8. Next Steps (This Branch)

This document is the first deliverable on `feat/platform-generics-v2`.

Subsequent work on this branch (in priority order):

1. Finish and review this consolidated v2 spec.
2. Implement the first 3–4 highest-value new generics as a proof-of-concept (recommended: `atlas_assets`, `atlas_cases`, `atlas_contracts`, `atlas_documents`).
3. Validate the JSONB + service-layer typing pattern in real code.
4. Update `atlas_app_integration.md` with the formal "Generic Fitness Test".
5. Update the most referenced Property Management requirements documents with correct entity names.
6. Lightweight Postgres performance review of the new generic DDLs.
7. Define clear ownership boundaries for the generics.

---

**References**

- Original 8 generics: [`platform_generics.md`](./platform_generics.md)
- Property Management Generics Challenge (Round 2/3): [`../property-management/23_second_round_generics.md`](../property-management/23_second_round_generics.md)
- Payment Rails Extension: [`../property-management/25_payment_rails_architecture.md`](../property-management/25_payment_rails_architecture.md)
- PM Implementation Roadmap: [`../property-management/24_implementation_roadmap.md`](../property-management/24_implementation_roadmap.md)
- AtlasApp Integration Protocol: [`../atlas_app_integration.md`](../atlas_app_integration.md)
- Layer Map: [`platform_layer_map.md`](./platform_layer_map.md)

---

&copy; Copyright Ruud Salym Erie & Oplyst International, LLC. All Rights Reserved.