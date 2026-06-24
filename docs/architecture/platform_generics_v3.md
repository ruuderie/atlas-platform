# Atlas Platform — Generics v3

> **Status:** Implemented & Merged to dev (June 2026 — Rev 5)
> **Date:** June 2026
> **Supersedes:** [`platform_generics_v2.md`](./platform_generics_v2.md)
> **Branch Target:** `feat/platform-generics-v3` → merged to `dev`
> **Purpose:** Adds G-32 (`atlas_rbac`), G-33 (`atlas_app_deployment_config`), G-34 (`atlas_vendor_marketplace`), promotes field enhancements to G-01/G-10/G-05 (Syndication), and resolves all gaps identified in the PM/Folio verticals.
>
> **See also:** `../CURRENT_STATE.md` for the absolute latest high-level summary.
>
> **NOTE (Rev 5):** G-32 and G-33 as originally designed in this spec referred to `atlas_memberships` and `atlas_entitlements` (Agency Provisioning vertical). The **actual implemented** G-32 is `atlas_rbac` (platform-generic Role Based Access Control) and the **actual implemented** G-33 is `atlas_app_deployment_config` (multi-tenant deployment topology). The original G-32/G-33 design content is preserved below for reference but marked as pending. See CURRENT_STATE.md for ground truth.

---

## 1. Philosophy & Rule 7

**Core Principle:** Before any AtlasApp writes a net-new table, it must prove that none of the platform generics can satisfy the need.

This rule exists to prevent the platform from becoming a collection of 13 slightly different CRMs, asset systems, case systems, and document stores.

**The Fitness Test (updated for v3):**

When an app author wants to introduce a new table, they must answer in `atlas_app_integration.md` style:

1. Which existing generic comes closest?
2. What specific field or behavior is missing?
3. Can it be modeled as `*_type` + `*_metadata JSONB` + app-level service typing?
4. **NEW v3:** Does the missing concept involve _platform-level roles and permissions_? → Consider G-32 `atlas_rbac`.
5. **NEW v3:** Does the missing concept involve _multi-tenant deployment topology and instance operational mode_? → Consider G-33 `atlas_app_deployment_config`.
6. **NEW v3:** Does the missing concept involve _cross-tenant vendor/provider discovery_? → Consider G-34 `atlas_vendor_marketplace`.
7. If truly not, what is the cross-vertical benefit that justifies promoting it to a new generic?

Only after passing the Fitness Test may a new migration be added to an `AtlasApp::migrations()`.

---

## 2. All Generics — Quick Reference

### Infrastructure Layer (G01–G08) — All Deployed ✅

| ID | Name | Core Need | Key Tables | Backend Status | Frontend Status |
|----|------|-----------|------------|----------------|-----------------|
| 01 | `atlas_geo` | Spatial / PostGIS queries + jurisdiction reference data | `geo_service_areas` | Deployed with API | Partial UI |
| 02 | `atlas_vault` | Secure file storage + sharing | `attachment`, `attachment_share_tokens`, `attachment_multipart_uploads` | Deployed with API | Partial UI |
| 03 | `atlas_payments` | Multi-rail payment ledger | `atlas_ledger_entries`, `atlas_ledger_splits`, `atlas_payment_credentials` | Deployed with API | **Full UI** |
| 04 | `atlas_subscriptions` | B2C recurring billing | `atlas_subscriptions` | Deployed with API | Partial UI |
| 05 | `atlas_syndication` | Outbound syndication + external integration event bus | `atlas_external_integrations`, `atlas_syndication_offer`, `atlas_app_instance_syndication`, `atlas_syndication_outbox`, `atlas_integration_events` | **Deployed with API** | **Full UI** |
| 06 | `atlas_verification_queue` | Human + automated trust workflows | `atlas_verification_requests` | Deployed with API | Partial UI |
| 07 | `atlas_realtime` | WebSocket entity rooms | `atlas_ws_rooms`, `atlas_ws_messages` | Deployed with API | No UI |
| 08 | `atlas_ai_tasks` | Async LLM / model work queue | `atlas_ai_tasks` | Deployed with API | No UI |

> **G-05 expansion (Rev 5):** Originally scoped to `atlas_external_integrations` (passive connector registry). Now extended with the full Syndication Event Bus: transactional outbox, per-instance offer links, HMAC-signed webhook delivery, exponential back-off retry, dead-letter, and integration event audit log.

### Domain Object Layer (G09–G18) — All Deployed ✅

| ID | Name | Core Need | Key Tables | Backend Status | Frontend Status |
|----|------|-----------|------------|----------------|-----------------|
| 09 | `atlas_portfolios` | Grouping of assets for reporting/billing/access | `atlas_portfolios` | Deployed with API | **Full UI** |
| 10 | `atlas_assets` | Physical or digital ledger items | `atlas_assets` (+ `asset_type`, `parent_asset_id`, `attributes JSONB`, `listing_mode`) | Deployed with API | **Full UI** |
| 11 | `atlas_contracts` | Legal agreements (leases, policies, rate agreements, SLAs) | `atlas_contracts` (+ `contract_type`, `terms_metadata JSONB`) | Deployed with API | **Full UI** |
| 12 | `atlas_service_providers` | Vendors, contractors, agents, adjusters | `atlas_service_providers` (+ `scope`, `service_categories`) | Deployed with API | **Full UI** |
| 13 | `atlas_cases` | Work items, tickets, claims, tasks, compliance alerts | `atlas_cases` (+ `case_type`, `case_metadata JSONB`) | Deployed with API | **Full UI** |
| 14 | `atlas_documents` | Documents with metadata, e-sig, versioning, polymorphic linkage | `atlas_documents` (+ `document_category`, `app_namespace`) | Deployed with API | No UI |
| 15 | `atlas_opportunities` | Deal / pipeline objects with financial modeling | `atlas_opportunities` (+ `opportunity_type`, `financial_inputs/outputs JSONB`) | Deployed with API | Partial UI |
| 16 | `atlas_regulatory_registrations` | Government permits, licenses, STR registrations | `atlas_regulatory_registrations` | Deployed with API | **Full UI** |
| 17 | `atlas_tax_events` + `atlas_tax_filings` | Taxable revenue events and periodic filings | Two tables for event-level + periodic filing | Deployed with API | No UI |
| 18 | `atlas_applications` | Structured multi-step intake, screening, and onboarding | `atlas_applications` (+ `application_type`, `application_metadata JSONB`) | Deployed with API | No UI |

### Round 1 Gap-Fill Additions (G19–G26) — All Deployed ✅

| ID | Name | Core Need | Key Tables | Backend Status | Frontend Status |
|----|------|-----------|------------|----------------|-----------------|
| 19 | `atlas_campaigns` | Marketing campaign enrollment + event tracking | `atlas_campaigns`, `atlas_campaign_enrollments`, `atlas_campaign_events` | Deployed with API | **Full UI** |
| 20 | `atlas_attribution` | Marketing touchpoint attribution | `atlas_attribution_touchpoints` | Deployed with API | No UI |
| 21 | `atlas_events` | Managed events with ticketing + registration | `atlas_events`, `atlas_event_registrations`, `atlas_event_ticket_types` | Deployed with API | No UI |
| 22 | `atlas_record_relationships` | Polymorphic typed edge between any two entity types | `atlas_record_relationships` | Deployed with API | No UI |
| 23 | `atlas_reservations` | Time-bound reservations with slot-conflict detection | `atlas_reservations`, `atlas_availability` | Deployed with API | **Full UI** |
| 24 | `atlas_quotes` | Quote + line-item pricing proposals | `atlas_quotes`, `atlas_quote_line_items` | Deployed with API | No UI |
| 25 | `atlas_commission_plans` | Commission agreement governing ledger splits | `atlas_commission_plans`, `atlas_commission_plan_splits` | Deployed with API | No UI |
| 26 | `atlas_catalog` | Product/service catalog with availability + dynamic pricing rules | `atlas_catalog_entries`, `atlas_catalog_availability`, `atlas_catalog_rate_rules` | Deployed with API | **Full UI** |

### Round 2 CRM & Intelligence Layer (G27–G31) — All Deployed ✅

| ID | Name | Core Need | Key Tables | Backend Status | Frontend Status |
|----|------|-----------|------------|----------------|-----------------|
| 27 | `atlas_scorecards` | Universal Structured Evaluation Engine + The Combinator similarity search | `atlas_scorecard_templates`, `atlas_scorecard_dimensions`, `atlas_scorecard_dimension_options`, `atlas_scorecards`, `atlas_rating_sessions`, `atlas_scorecard_entries`, `atlas_scorecard_dimension_aggregates`, `atlas_scorecard_poll_aggregates`, `atlas_scorecard_time_series`, `atlas_scorecard_targets`, `atlas_scorecard_target_criteria`, `atlas_scorecard_display_rules`, `atlas_scorecard_contributor_calibrations` | Deployed with API | Partial UI |
| 28 | `atlas_note` | Universal polymorphic note with threading + visibility | `atlas_notes` | Entity defined | Partial UI |
| 29 | `atlas_activity` | Universal polymorphic activity log | `activity` | Entity defined | Partial UI |
| 31 | `atlas_lead` | Canonical lead/prospect with full import→qualify→convert→disqualify lifecycle | `atlas_lead`, `atlas_lead_compat_view` | Deployed with API | **Full UI** |

### Round 3 — Deployed Generics (G32–G34) ✅

> **Implementation note:** The original v3 design proposed G-32 as `atlas_memberships` and G-33 as `atlas_entitlements` for the Agency Provisioning vertical. The actual implemented generics in the current codebase are different — they address the PM/Folio multi-tenant operational needs that proved more immediately critical. The original memberships/entitlements design is preserved in §9 for future implementation.

| ID | Name | Core Need | Key Tables | Backend Status | Frontend Status |
|----|------|-----------|------------|----------------|-----------------|
| 32 | `atlas_rbac` | Platform-generic Role Based Access Control | `atlas_role_profiles`, `atlas_role_profile_permissions`, `atlas_user_app_roles` | Deployed with API | Partial UI |
| 33 | `atlas_app_deployment_config` | Multi-tenant deployment topology + operational instance config | `atlas_app_deployment_config` (`folio_mode`: standard\|pmc\|brokerage; `deployment_mode`: standard\|internal_operator) | Deployed with API | Partial UI |
| 34 | `atlas_vendor_marketplace` | Opt-in cross-tenant vendor/service provider discovery | `atlas_service_providers` extension (`is_marketplace_visible`, `marketplace_bio`, `marketplace_trade_types`, `marketplace_location` PostGIS point) | Deployed | No UI |

---

## 3. Implemented Generic Detail — G-32: `atlas_rbac`

**Problem this solves:** Every app instance requires a role-based permission system where users have app-scoped roles, and those roles confer specific permissions. A platform-level RBAC system prevents each vertical from rebuilding permission checks independently.

### Tables

```
atlas_role_profiles           — named role definitions per tenant + app
atlas_role_profile_permissions — permission slugs associated with a role profile
atlas_user_app_roles          — active role assignment for (user, tenant, app)
```

### Service API (`backend/src/services/rbac.rs`)

```rust
get_user_app_role(db, user_id, tenant_id, app_slug) → Option<String>
  └─ resolves role_slug from active role profile for app+tenant
has_permission(db, user_id, tenant_id, app_slug, permission_slug) → bool
  └─ checks both role profile permissions (wildcards supported: "billing:*") and user-specific overrides
assign_role(db, user_id, tenant_id, app_slug, role_slug, granted_by) → Result<Uuid, DbErr>
  └─ idempotent upsert; swaps active role in app+tenant context
revoke_role(db, user_id, tenant_id, app_slug) → Result<u64, DbErr>
  └─ sets is_active = false for user app role
list_role_profiles(db, tenant_id, app_slug) → Result<Vec<atlas_role_profiles::Model>, DbErr>
  └─ lists all platform defaults + tenant-scoped custom role profiles
```

### Migrations
- `m20260811_g32_atlas_rbac` — full tables + indexes
- `m20260812_g32_folio_role_seed` — seeds standard landlord role
- `m20260813_g32_migrate_folio_roles` — migrates legacy folio user roles
- `m20260814_g32_drop_folio_role_column` — drops legacy column post-migration

---

## 4. Implemented Generic Detail — G-33: `atlas_app_deployment_config`

**Problem this solves:** Platform operators need to configure how each app instance is deployed — its operational identity (is it a standard folio, a PMC-mode folio, or a brokerage-mode folio?) and its deployment topology. This cannot be a tenant setting because different app instances for the same tenant can have different modes.

### Table

```sql
atlas_app_deployment_config (
  id                UUID PRIMARY KEY,
  tenant_id         UUID NOT NULL,
  app_instance_id   UUID NOT NULL,
  folio_mode        VARCHAR CHECK (folio_mode IN ('standard', 'pmc', 'brokerage')) DEFAULT 'standard',
  deployment_mode   VARCHAR CHECK (deployment_mode IN ('standard', 'internal_operator')) DEFAULT 'standard',
  ...
)
```

**Key invariant:** A single folio instance cannot be `pmc` AND `brokerage` simultaneously. The DB CHECK constraint on `folio_mode` enforces this at the database level.

### Axum Extractor

`extractors/app_config.rs` — `AppDeploymentConfig` extractor resolves the config for the current request's app instance. Defaults to `"folio"` app slug.

### API
- `PATCH /admin/app-instances/{id}/operational-config` — update folio_mode, billing_tier, portal toggles

### Migrations
- `m20260815_g33_app_deployment_config` — initial table
- `m20260816_g33_folio_pmc_seed` — seeds PMC rbac role profiles
- `m20260817_folio_managed_account_id` — managed account isolation
- `m20260818_folio_client_role_scope` — client scope on user roles
- `m20260909_folio_instance_mode` — typed `folio_mode` column with DB CHECK constraint

---

## 5. Implemented Generic Detail — G-34: `atlas_vendor_marketplace`

**Problem this solves:** Service providers managed within a Folio instance need an opt-in mechanism to appear in a cross-tenant vendor discovery marketplace, with geographic matching via PostGIS.

### Database Extension (extends G-12)

```sql
-- Additive columns on atlas_service_providers:
ALTER TABLE atlas_service_providers
  ADD COLUMN is_marketplace_visible  BOOLEAN NOT NULL DEFAULT FALSE,
  ADD COLUMN marketplace_bio         TEXT,
  ADD COLUMN marketplace_trade_types TEXT[],
  ADD COLUMN marketplace_location    geometry(Point, 4326);

CREATE INDEX idx_service_providers_marketplace_location
  ON atlas_service_providers USING GIST (marketplace_location)
  WHERE is_marketplace_visible = TRUE;
```

### Geographic Matching

PostGIS `ST_DWithin` and `ST_Distance` used for radius search. Coordinates derived from property/service area context.

### Migration
- `m20260819_g34_vendor_marketplace` — all columns + PostGIS index

---

## 6. v3 Field Enhancements to Existing Generics (Deployed)

### G-10: `atlas_assets` — Universal Lifecycle Extension

**Migration:** `m20260900_g10_asset_lifecycle` ✅ DEPLOYED

```sql
ALTER TABLE atlas_assets
    ADD COLUMN scheduled_service_date  DATE,
    ADD COLUMN expiry_date             DATE,
    ADD COLUMN condition               VARCHAR(30),
    ADD COLUMN lifecycle_metadata      JSONB;
```

**Column semantics:**

| Column | Universal meaning | Example values |
|---|---|---|
| `scheduled_service_date` | Next scheduled maintenance / calibration / inspection | Annual boiler service, DOT inspection |
| `expiry_date` | Warranty / certificate / registration expiry | Manufacturer warranty, vehicle registration |
| `condition` | Current operational state | `excellent` \| `good` \| `fair` \| `poor` \| `retired` |
| `lifecycle_metadata` | App-owned typed sidecar — make, model, serial | Varies per `asset_type` |

### G-10: `atlas_assets` — Listing Mode Extension

**Migration:** `m20260911_asset_listing_mode` ✅ DEPLOYED

Adds `listing_mode` column to control how assets are surfaced in marketplace/syndication contexts.

### G-05: Syndication Event Bus

**Migrations:** `m20260910`, `m20260912`, `m20260913`, `m20260915` ✅ DEPLOYED

See §2 G-05 for full detail.

---

## 7. Implementation Order (Authoritative — v3 Complete)

All v3 generics are deployed. Migration sequence (chronological):

```
m20260811_g32_atlas_rbac
m20260812_g32_folio_role_seed
m20260813_g32_migrate_folio_roles
m20260814_g32_drop_folio_role_column
m20260815_g33_app_deployment_config
m20260816_g33_folio_pmc_seed
m20260817_folio_managed_account_id
m20260818_folio_client_role_scope
m20260819_g34_vendor_marketplace
m20260900_g10_asset_lifecycle
m20260907_feature_flags
m20260908_platform_invitations
m20260909_folio_instance_mode
m20260910_folio_instance_syndication
m20260911_asset_listing_mode
m20260912_atlas_syndication_offer
m20260913_atlas_app_instance_syndication
m20260914_atlas_listing_asset_fk
m20260915_atlas_syndication_outbox
```

---

## 8. Cross-App Benefit Matrix (v3 Updated)

| Generic | Primary Driver | Strong Secondary Benefits |
|---------|----------------|---------------------------|
| `atlas_portfolios` | PM | ClaimSwift (adjuster pools), AgentLink (agency alliances) |
| `atlas_assets` | PM | ClaimSwift (damaged asset), Direct Booking (hotel rooms) |
| `atlas_contracts` | PM | CoverFlow (insurance policies), AgentLink (distribution agreements) |
| `atlas_service_providers` | PM | ClaimSwift (adjusters), AgentLink (agents), **G-34 Marketplace** |
| `atlas_cases` | PM | ClaimSwift (claims), compliance |
| `atlas_documents` | PM | CoverFlow, ClaimSwift, AgentLink, Famtasm |
| `atlas_opportunities` | PM | CoverFlow (submissions), Direct Booking (pipelines) |
| `atlas_regulatory_registrations` | PM (STR) | ClaimSwift (adjuster licenses), AgentLink |
| `atlas_tax_events / filings` | PM (TDT) | CoverFlow (premium tax), Clipping (1099-K) |
| `atlas_applications` | PM | AgentLink, Famtasm, Direct Booking |
| `atlas_catalog` | Direct Booking | PM (rate rules) |
| `atlas_record_relationships` | CRM | Campaign ↔ lead, event ↔ contact cross-links |
| **`atlas_rbac` (G-32)** | **Platform** | All apps (Folio landlord/tenant/vendor/agent/broker/PMC role shells) |
| **`atlas_app_deployment_config` (G-33)** | **Platform/Folio** | Enables standard/PMC/brokerage operational modes per instance |
| **`atlas_vendor_marketplace` (G-34)** | **Folio** | Any vertical with a cross-tenant vendor discovery need |
| `atlas_syndication (G-05)` | **Folio/NI** | Any vertical needing outbound webhook delivery with reliability guarantees |

---

## 9. Open Items & Next Build Priorities (v3 Status)

### Deployed (✅ Complete)

- G-32 `atlas_rbac` — tables, service, handlers, folio auth shells
- G-33 `atlas_app_deployment_config` — tables, extractor, operational config PATCH, platform-admin panel
- G-34 `atlas_vendor_marketplace` — PostGIS columns, index
- G-05 Syndication Event Bus — outbox, retry, HMAC, delivery, audit
- G-10 asset lifecycle fields, listing mode
- Feature flags registry, platform invitations

### Pending (Highest Value)

1. **G-34 Public Marketplace UI** — No public-facing vendor search directory page yet.
2. **G-24 Quote Builder UI** — Backend deployed, no frontend.
3. **G-21 Event Management UI** — Backend deployed, no frontend.
4. **G-14 Document Browser UI** — Backend deployed, no frontend.
5. **G-28/G-29 Full Services** — Notes/Activities use handler-only pattern; should be promoted to `NoteService`/`ActivityService`.
6. **G-05 Inbound Webhook Handler** — Outbound delivery complete; inbound receiver not yet built.

### Original v3 Design (Agency Provisioning) — Future

The original v3 design proposed for Agency Provisioning vertical:
- `atlas_memberships` — Person↔organization membership with roles and lifecycle
- `atlas_entitlements` — Scoped external system access grants

These remain valid generic candidates for future verticals (insurance, telecom, fintech). The detailed DDL, Rust patterns, and cross-app benefit analysis from the original v3 spec are preserved in the git history and remain the reference design for when those generics are implemented. See the original v3 spec content for implementation guidance.

---

## 10. Open Risks & Questions

1. **JSONB Ergonomics** (carried from v2) — Heavy reliance on `*_metadata JSONB` in Rust service layers and Leptos forms.
2. **Polymorphic Query Performance** (carried from v2) — Index strategy review needed for high-cardinality JSONB queries.
3. **G-33 `folio_mode` CHECK constraint** — Enforced at DB level. Adding new modes requires a migration to extend the CHECK constraint.
4. **G-32 cascade patterns** — RbacService `revoke_role` soft-deletes (sets `is_active = false`). Hard deletion of role profiles must cascade-update or soft-delete linked user app roles.
5. **G-34 coordinate updates** — `marketplace_location` must be kept in sync with asset/property location updates. No trigger exists; service layer responsibility.
6. **PostGIS in CI** — `ST_DWithin` and `ST_Distance` queries are skipped when PostGIS binary is absent. CI should enforce PostGIS.

---

**References**

- Superseded: [`platform_generics_v2.md`](./platform_generics_v2.md)
- Original 8 generics: [`platform_generics.md`](./platform_generics.md)
- Current State: [`../CURRENT_STATE.md`](../CURRENT_STATE.md)
- Layer Map: [`platform_layer_map.md`](./platform_layer_map.md)
- Asset metadata shapes: [`asset_metadata_shapes.md`](./asset_metadata_shapes.md)

---

© Copyright Ruud Salym Erie & Oplyst International, LLC. All Rights Reserved.
