# Atlas Platform — Generics v3

> **Status:** Design Complete — Implementation Pending (June 2026)
> **Date:** June 2026
> **Supersedes:** [`platform_generics_v2.md`](./platform_generics_v2.md)
> **Branch Target:** `feat/platform-generics-v3`
> **Purpose:** Adds G-32 (`atlas_memberships`) and G-33 (`atlas_entitlements`), promotes field enhancements to G-01/G-16/Party model, and resolves all gaps identified in the Agency Provisioning vertical analysis.
>
> **See also:** `../CURRENT_STATE.md` for the absolute latest high-level summary.
> **Triggered by:** [`../agprov/agency_provisioning_spec.md`](../agprov/agency_provisioning_spec.md) — generic gap analysis.

---

## 1. Philosophy & Rule 7

**Core Principle:** Before any AtlasApp writes a net-new table, it must prove that none of the platform generics can satisfy the need.

This rule exists to prevent the platform from becoming a collection of 13 slightly different CRMs, asset systems, case systems, and document stores.

**The Fitness Test (updated for v3):**

When an app author wants to introduce a new table, they must answer in `atlas_app_integration.md` style:

1. Which existing generic comes closest?
2. What specific field or behavior is missing?
3. Can it be modeled as `*_type` + `*_metadata JSONB` + app-level service typing?
4. **NEW v3:** Does the missing concept involve _roles on a person↔org link_? → Consider G-32 `atlas_memberships`.
5. **NEW v3:** Does the missing concept involve _scoped access grants to an external system_? → Consider G-33 `atlas_entitlements`.
6. If truly not, what is the cross-vertical benefit that justifies promoting it to a new generic?

Only after passing the Fitness Test may a new migration be added to an `AtlasApp::migrations()`.

---

## 2. All Generics — Quick Reference

### Infrastructure Layer (G01–G08) — All Deployed ✅

| ID | Name | Core Need | Key Tables |
|----|------|-----------|------------|
| 01 | `atlas_geo` | Spatial / PostGIS queries + jurisdiction reference data | `geo_service_areas` (**v3: + `regulatory_status`, `regulatory_notes`**) |
| 02 | `atlas_vault` | Secure file storage + sharing | `attachment`, `attachment_share_tokens`, `attachment_multipart_uploads` |
| 03 | `atlas_payments` | Multi-rail payment ledger | `atlas_ledger_entries`, `atlas_ledger_splits`, `atlas_payment_credentials` |
| 04 | `atlas_subscriptions` | B2C recurring billing | `atlas_subscriptions` |
| 05 | `atlas_external_integrations` | Third-party connector registry (PMS/OTA/AMS) | `atlas_external_integrations`, `atlas_integration_events` |
| 06 | `atlas_verification_queue` | Human + automated trust workflows | `atlas_verification_requests` |
| 07 | `atlas_realtime` | WebSocket entity rooms | `atlas_ws_rooms`, `atlas_ws_messages` |
| 08 | `atlas_ai_tasks` | Async LLM / model work queue | `atlas_ai_tasks` |

### Domain Object Layer (G09–G18) — All Deployed ✅

| ID | Name | Core Need | Key Tables |
|----|------|-----------|------------|
| 09 | `atlas_portfolios` | Grouping of assets for reporting/billing/access | `atlas_portfolios` |
| 10 | `atlas_assets` | Physical or digital ledger items | `atlas_assets` (+ `asset_type`, `parent_asset_id`, `attributes JSONB`) |
| 11 | `atlas_contracts` | Legal agreements (leases, policies, rate agreements, SLAs, LOB agreements) | `atlas_contracts` (+ `contract_type`, `terms_metadata JSONB`) |
| 12 | `atlas_service_providers` | Vendors, contractors, agents, adjusters | `atlas_service_providers` (+ `scope`, `service_categories`) |
| 13 | `atlas_cases` | Work items, tickets, claims, tasks, compliance alerts, approval workflows | `atlas_cases` (+ `case_type`, `case_metadata JSONB`) |
| 14 | `atlas_documents` | Documents with metadata, e-sig, versioning, polymorphic linkage | `atlas_documents` (+ `document_category`, `app_namespace`) |
| 15 | `atlas_opportunities` | Deal / pipeline objects with financial modeling | `atlas_opportunities` (+ `opportunity_type`, `financial_inputs/outputs JSONB`) |
| 16 | `atlas_regulatory_registrations` | Government permits, licenses, STR registrations, state appointments | `atlas_regulatory_registrations` (**v3: + `account_id`, `contact_id`, `npn`, `license_type`, `lines_of_authority TEXT[]`**) |
| 17 | `atlas_tax_events` + `atlas_tax_filings` | Taxable revenue events and periodic filings | Two tables for event-level + periodic filing |
| 18 | `atlas_applications` | Structured multi-step intake, screening, and onboarding | `atlas_applications` (+ `application_type`, `application_metadata JSONB`) |

### Round 1 Gap-Fill Additions (G19–G26) — All Deployed ✅

| ID | Name | Core Need | Key Tables |
|----|------|-----------|------------|
| 19 | `atlas_campaigns` | Marketing campaign enrollment + event tracking | `atlas_campaigns`, `atlas_campaign_enrollments`, `atlas_campaign_events` |
| 20 | `atlas_attribution` | Marketing touchpoint attribution | `atlas_attribution_touchpoints` |
| 21 | `atlas_events` | Managed events with ticketing + registration | `atlas_events`, `atlas_event_registrations`, `atlas_event_ticket_types` |
| 22 | `atlas_record_relationships` | Polymorphic typed edge between any two entity types | `atlas_record_relationships` |
| 23 | `atlas_reservations` | Time-bound reservations with slot-conflict detection | `atlas_reservations`, `atlas_availability` |
| 24 | `atlas_quotes` | Quote + line-item pricing proposals | `atlas_quotes`, `atlas_quote_line_items` |
| 25 | `atlas_commission_plans` | Commission agreement governing ledger splits | `atlas_commission_plans`, `atlas_commission_plan_splits` |
| 26 | `atlas_catalog` | Product/service catalog with availability + dynamic pricing rules | `atlas_catalog_entries`, `atlas_catalog_availability`, `atlas_catalog_rate_rules` |

### Round 2 CRM & Intelligence Layer (G27–G31) — All Deployed ✅

| ID | Name | Core Need | Key Tables |
|----|------|-----------|------------|
| 27 | `atlas_scorecards` | Universal Structured Evaluation Engine + The Combinator similarity search | `atlas_scorecard_templates`, `atlas_scorecard_dimensions`, `atlas_scorecard_dimension_options`, `atlas_scorecards`, `atlas_rating_sessions`, `atlas_scorecard_entries`, `atlas_scorecard_dimension_aggregates`, `atlas_scorecard_poll_aggregates`, `atlas_scorecard_time_series`, `atlas_scorecard_targets`, `atlas_scorecard_target_criteria`, `atlas_scorecard_display_rules`, `atlas_scorecard_contributor_calibrations` |
| 28 | `atlas_note` | Universal polymorphic note with threading + visibility | `atlas_notes` |
| 29 | `atlas_activity` | Universal polymorphic activity log | `activity` |
| 31 | `atlas_lead` | Canonical lead/prospect with full import→qualify→convert→disqualify lifecycle | `atlas_lead`, `atlas_lead_compat_view` |

### Round 3 Access & Relationship Layer (G32–G33) — **NEW in v3, Implementation Pending** 🔲

| ID | Name | Core Need | Key Tables |
|----|------|-----------|------------|
| 32 | `atlas_memberships` | Person↔organization membership with roles, lifecycle status, and active switch | `atlas_memberships` |
| 33 | `atlas_entitlements` | Scoped external system access grants with idempotent keying, tier, and approval gating | `atlas_entitlements` |

---

## 3. New in v3 — Detailed Specifications

### G-32: `atlas_memberships`

**Problem this solves:** Every vertical has a "person belongs to an organization with a role" concept: insurance agents belong to agencies, property managers belong to portfolios, SaaS users belong to teams, healthcare providers belong to practices. G-22 (`atlas_record_relationships`) models the _existence_ of a link but cannot support first-class queryable role arrays, lifecycle status transitions, or the indexed `is_active` column required for bulk cascade operations.

**The cascade invariant** is the critical differentiator: deactivating a membership must atomically deactivate all downstream entitlements (G-33) via a direct `UPDATE ... WHERE account_id = X AND contact_id = Y AND is_active = TRUE`. This cannot be done efficiently against JSONB fields.

#### Table: `atlas_memberships`

```sql
CREATE TABLE atlas_memberships (
    id                   UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id            UUID         NOT NULL,

    -- The organization (agency, team, practice, portfolio owner)
    parent_account_id    UUID         NOT NULL REFERENCES atlas_accounts(id) ON DELETE CASCADE,

    -- The person (agent, team member, provider, manager)
    member_contact_id    UUID         NOT NULL REFERENCES atlas_contacts(id) ON DELETE CASCADE,

    -- Membership classification. Open string — e.g. "AGENCY_AGENT", "TEAM_MEMBER",
    -- "FRANCHISE_LOCATION", "HEALTHCARE_PROVIDER", "PROPERTY_MANAGER"
    membership_type      VARCHAR(100) NOT NULL,

    -- Structured role array. Values are domain-specific, validated at the service layer.
    -- e.g. ["CSR", "LICENSED_PRODUCER", "AGENCY_ADMIN"] for insurance
    -- e.g. ["ADMIN", "EDITOR", "VIEWER"] for SaaS teams
    roles                TEXT[]       NOT NULL DEFAULT '{}',

    -- Lifecycle status. Open string — e.g. "ACTIVE", "PENDING_REVIEW", "INACTIVE"
    status               VARCHAR(50)  NOT NULL DEFAULT 'ACTIVE',

    -- Master deactivation switch. Cascade to atlas_entitlements is one-directional.
    -- Reactivating this field does NOT automatically reactivate linked entitlements.
    is_active            BOOLEAN      NOT NULL DEFAULT TRUE,

    -- First-class flag for document-signing signer resolution. Must be queryable directly.
    is_authorized_signer BOOLEAN      NOT NULL DEFAULT FALSE,

    -- Domain-specific extension bag.
    membership_metadata  JSONB,

    created_at           TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at           TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    UNIQUE (tenant_id, parent_account_id, member_contact_id, membership_type)
);

-- Hot path: cascade deactivation query
CREATE INDEX idx_memberships_account_contact_active
    ON atlas_memberships (tenant_id, parent_account_id, member_contact_id)
    WHERE is_active = TRUE;

-- Authorized signer lookup (single-row queries)
CREATE INDEX idx_memberships_authorized_signer
    ON atlas_memberships (tenant_id, parent_account_id, is_authorized_signer)
    WHERE is_authorized_signer = TRUE AND is_active = TRUE;
```

#### Rust Entity Pattern

```rust
pub struct Model {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub parent_account_id: Uuid,
    pub member_contact_id: Uuid,
    pub membership_type: String,
    pub roles: Vec<String>,
    pub status: String,
    pub is_active: bool,
    pub is_authorized_signer: bool,
    pub membership_metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

#### Cascade Deactivation Service Pattern

```rust
/// One-directional cascade: membership → entitlements.
/// Reactivation of the membership does NOT reverse this.
pub async fn deactivate_membership(db: &DbConn, id: Uuid, tenant_id: Uuid) -> Result<()> {
    // 1. Deactivate the membership itself
    atlas_memberships::Entity::update_many()
        .col_expr(atlas_memberships::Column::IsActive, Expr::value(false))
        .filter(atlas_memberships::Column::Id.eq(id))
        .filter(atlas_memberships::Column::TenantId.eq(tenant_id))
        .exec(db).await?;

    // 2. Cascade to all active entitlements (one-directional)
    atlas_entitlements::Entity::update_many()
        .col_expr(atlas_entitlements::Column::IsActive, Expr::value(false))
        .col_expr(atlas_entitlements::Column::ProvisionStatus, Expr::value("PENDING_DEPROVISION"))
        .filter(atlas_entitlements::Column::MembershipId.eq(id))
        .filter(atlas_entitlements::Column::IsActive.eq(true))
        .exec(db).await?;
    Ok(())
}
```

#### Cross-App Benefit

| Vertical | `membership_type` | `roles` example | Key `membership_metadata` fields |
|---|---|---|---|
| Insurance Distribution | `"AGENCY_AGENT"` | `["LICENSED_PRODUCER", "CSR"]` | `{ "agent_status": "PENDING_REVIEW", "generic_onboarding": false }` |
| Property Management | `"PORTFOLIO_MANAGER"` | `["OWNER", "MANAGER"]` | `{ "management_fee_pct": 8.5 }` |
| SaaS / Team | `"TEAM_MEMBER"` | `["ADMIN", "EDITOR"]` | `{ "invited_by": "uuid", "seat_type": "full" }` |
| Marketplace / Franchise | `"FRANCHISE_LOCATION"` | `["OPERATOR"]` | `{ "territory_code": "SE-FL" }` |
| Healthcare | `"PROVIDER_PRACTICE"` | `["ATTENDING", "BILLING"]` | `{ "npi": "1234567890", "specialty": "cardiology" }` |

---

### G-33: `atlas_entitlements`

**Problem this solves:** When a person's membership grants them access to external downstream systems — scoped by geography, carrier, or product — there must be a first-class record that: (a) serves as the on/off provisioning switch, (b) stores confirmed identity from the external system, (c) supports approval-gated activation, (d) computes a tier from a configurable role matrix, and (e) is idempotently upsertable via a deterministic key.

No existing generic satisfies this:
- G-05 (`atlas_external_integrations`) tracks a **tenant's** credentials to an external system — not a **person's** access grant within a scope.
- G-04 (`atlas_subscriptions`) is billing — not access provisioning.
- G-11 (`atlas_contracts`) tracks signed agreements — not active system provisioning state.

#### Table: `atlas_entitlements`

```sql
CREATE TABLE atlas_entitlements (
    id                    UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id             UUID         NOT NULL,

    -- Link to the membership that created this entitlement (optional but recommended).
    -- Supports cascade deactivation queries.
    membership_id         UUID         REFERENCES atlas_memberships(id) ON DELETE SET NULL,

    -- Direct entity references for queries that bypass membership join.
    account_id            UUID         NOT NULL REFERENCES atlas_accounts(id),
    contact_id            UUID         NOT NULL REFERENCES atlas_contacts(id),

    -- The counterparty organization providing the system access
    -- (e.g. carrier, software vendor, platform operator).
    provider_account_id   UUID         REFERENCES atlas_accounts(id),

    -- The downstream system identifier. Open string.
    -- e.g. "SYSTEM_A", "VIOLET", "IPX", "SALESFORCE", "GUIDEWIRE"
    system_identifier     VARCHAR(100) NOT NULL,

    -- Geographic / jurisdictional scope. Use either scope_code (loose)
    -- or scope_entity_id (FK to geo_service_areas, strict).
    scope_code            VARCHAR(10),
    scope_entity_id       UUID         REFERENCES geo_service_areas(id),

    -- Deterministic idempotency key. Format is caller-defined but must be unique.
    -- Recommended: lower("{account_id}:{contact_id}:{provider_id}:{system}:{scope_code}")
    -- Used for ON CONFLICT (entitlement_key) DO UPDATE upserts.
    entitlement_key       VARCHAR(500) NOT NULL,

    -- Classification of the entitlement within the system.
    -- e.g. "LICENSED_PRODUCER", "CSR", "COMMISSION", "READ_ONLY"
    entitlement_type      VARCHAR(100) NOT NULL,

    -- Provisioning on/off switch. The ground truth for whether the external system
    -- should have this entity provisioned. Downstream reconciliation matches this.
    is_active             BOOLEAN      NOT NULL DEFAULT FALSE,

    -- Computed tier from a configurable role-tier mapping (G-26 rate rules or tenant config).
    -- Written by the service layer before the record is dispatched to external systems.
    tier                  SMALLINT,

    -- When TRUE, activation requires an approval Case (G-13) to be resolved first.
    -- The linked_approval_id must reference an atlas_case with status = 'APPROVED'
    -- before is_active may be set to TRUE.
    is_gated              BOOLEAN      NOT NULL DEFAULT FALSE,
    linked_approval_id    UUID         REFERENCES atlas_cases(id),

    -- The approved regulatory registration (e.g. state license / appointment).
    -- FK to G-16. Set when resolving license credentials for this entitlement.
    regulatory_registration_id UUID    REFERENCES atlas_regulatory_registrations(id),

    -- Stamped by the downstream adapter upon successful provisioning.
    external_id           VARCHAR(255),
    external_roles        TEXT[],

    -- Provisioning lifecycle: PENDING / PROVISIONING / ACTIVE / DEPROVISIONING /
    --                         FAILED / DEPROVISIONED
    provision_status      VARCHAR(50)  NOT NULL DEFAULT 'PENDING',

    -- Domain-specific extension bag.
    entitlement_metadata  JSONB,

    created_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_entitlement_key UNIQUE (entitlement_key)
);

-- Cascade deactivation: all active entitlements for an account+contact pair
CREATE INDEX idx_entitlements_account_contact_active
    ON atlas_entitlements (tenant_id, account_id, contact_id)
    WHERE is_active = TRUE;

-- Provisioning dispatch: entitlements needing outbox processing
CREATE INDEX idx_entitlements_provision_pending
    ON atlas_entitlements (tenant_id, provision_status)
    WHERE provision_status IN ('PENDING', 'PROVISIONING', 'DEPROVISIONING', 'FAILED');

-- Membership cascade path
CREATE INDEX idx_entitlements_membership
    ON atlas_entitlements (membership_id)
    WHERE is_active = TRUE;
```

#### Rust Entity Pattern

```rust
pub struct Model {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub membership_id: Option<Uuid>,
    pub account_id: Uuid,
    pub contact_id: Uuid,
    pub provider_account_id: Option<Uuid>,
    pub system_identifier: String,
    pub scope_code: Option<String>,
    pub scope_entity_id: Option<Uuid>,
    pub entitlement_key: String,
    pub entitlement_type: String,
    pub is_active: bool,
    pub tier: Option<i16>,
    pub is_gated: bool,
    pub linked_approval_id: Option<Uuid>,
    pub regulatory_registration_id: Option<Uuid>,
    pub external_id: Option<String>,
    pub external_roles: Option<Vec<String>>,
    pub provision_status: String,
    pub entitlement_metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

#### Idempotent Upsert Pattern

```rust
/// Deterministic key format (caller defines, but must be stable and unique):
/// lower("{account_id}:{contact_id}:{provider_id}:{system}:{scope_code}")
pub fn compute_entitlement_key(
    account_id: Uuid, contact_id: Uuid, provider_id: Uuid,
    system: &str, scope: &str,
) -> String {
    format!("{}:{}:{}:{}:{}", account_id, contact_id, provider_id, system, scope)
        .to_lowercase()
}

pub async fn upsert_entitlement(db: &DbConn, input: EntitlementInput) -> Result<Uuid> {
    let key = compute_entitlement_key(/* ... */);
    // INSERT ... ON CONFLICT (entitlement_key) DO UPDATE SET
    //   is_active = EXCLUDED.is_active,
    //   tier = EXCLUDED.tier,
    //   provision_status = EXCLUDED.provision_status,
    //   updated_at = NOW()
    // RETURNING id
}
```

#### OutboxWorker Job Types (extend existing worker)

```
provision_entitlement      → calls downstream adapter for system_identifier
deprovision_entitlement    → calls downstream adapter with DEPROVISION action
reconcile_entitlement      → drift check: Atlas is_active vs external system state
```

#### Cross-App Benefit

| Vertical | `system_identifier` | `entitlement_type` | `is_gated` | `scope_code` |
|---|---|---|---|---|
| Insurance Distribution | `"VIOLET"`, `"IPX"`, `"SPEEDBUILDER"` | `"LICENSED_PRODUCER"` | `true` | `"FL"`, `"GA"` |
| SaaS Reseller Network | `"SALESFORCE"`, `"HUBSPOT"` | `"SEAT_FULL"` | `false` | — |
| Fintech / Advisor | `"BLOOMBERG"`, `"REFINITIV"` | `"TRADING_AUTHORIZED"` | `true` | `"NY"`, `"CA"` |
| Telecom Provisioning | `"VOICE_SYSTEM"`, `"SMS_GATEWAY"` | `"SUBSCRIBER_ACTIVE"` | `false` | `"REGION_SE"` |
| Healthcare HIT | `"EMR_SYSTEM"`, `"PRESCRIPTION_GATEWAY"` | `"PRESCRIBING"` | `true` | `"FL"` |

---

## 4. v3 Field Enhancements to Existing Generics

These are **additive, non-breaking column additions** to existing deployed generics. Each is a new migration.

### G-10: `atlas_assets` — Universal Lifecycle Extension

**Migration:** `m20260900_g10_asset_lifecycle`

```sql
ALTER TABLE atlas_assets
    ADD COLUMN scheduled_service_date  DATE,
    ADD COLUMN expiry_date             DATE,
    ADD COLUMN condition               VARCHAR(30),
    ADD COLUMN lifecycle_metadata      JSONB;

-- Universal alert index — works for every asset_type, every vertical
CREATE INDEX idx_assets_service_due
    ON atlas_assets (tenant_id, scheduled_service_date)
    WHERE scheduled_service_date IS NOT NULL;

CREATE INDEX idx_assets_expiry
    ON atlas_assets (tenant_id, expiry_date)
    WHERE expiry_date IS NOT NULL;

CREATE INDEX idx_assets_condition
    ON atlas_assets (tenant_id, condition)
    WHERE condition IS NOT NULL;
```

**Column semantics:**

| Column | Type | Universal meaning | Example values |
|---|---|---|---|
| `scheduled_service_date` | `DATE` | Next scheduled maintenance / calibration / inspection | Annual boiler service, DOT inspection, MRI calibration |
| `expiry_date` | `DATE` | Warranty / certificate / license / registration expiry | Manufacturer warranty, FDA cert, vehicle registration |
| `condition` | `VARCHAR(30)` | Current operational state | `excellent` \| `good` \| `fair` \| `poor` \| `retired` |
| `lifecycle_metadata` | `JSONB` | App-owned typed sidecar — make, model, serial, domain-specific identity | Varies per `asset_type`; see app service layer for shape |

**Why now:** Every vertical that tracks physical or managed assets needs scheduled-maintenance alerts and expiry notifications. Without indexed columns for these dates, each app would write JSONB expression queries that cannot use standard B-tree indexes. Adding these four columns once gives every current and future vertical efficient date-range queries with zero additional migrations.

**Cross-app benefit:**

| Vertical | `asset_type` | `scheduled_service_date` | `expiry_date` | `lifecycle_metadata` shape |
|---|---|---|---|---|
| Property Mgmt (Folio) | `appliance` | Annual service | Manufacturer warranty | `ApplianceMetadata` |
| Healthcare | `medical_device` | Calibration due | FDA cert expiry | `MedicalDeviceMetadata` |
| Fleet / Logistics | `vehicle` | DOT inspection | Registration expiry | `VehicleMetadata` |
| IT / SaaS | `it_device` | Patch cycle | Warranty / OS EOL | `ItDeviceMetadata` |
| Energy / Utilities | `meter` | Calibration due | Manufacturer warranty | `MeterMetadata` |
| Insurance | `insured_item` | Appraisal renewal | Policy expiry | `InsuredItemMetadata` |

**App-layer Rust pattern:**

Each AtlasApp defines its own typed struct for `lifecycle_metadata` and implements `TryFrom<&AssetModel>` to deserialize and validate domain rules:

```rust
// Each app owns its metadata shape — DB enforces nothing beyond JSONB validity
#[derive(Serialize, Deserialize)]
pub struct ApplianceMetadata {           // Folio
    pub appliance_type: ApplianceType,
    pub make: String,
    pub model: String,
    pub fuel_type: Option<FuelType>,
    // ...
}

#[derive(Serialize, Deserialize)]
pub struct VehicleMetadata {             // FleetOps
    pub vin: String,                     // validated: must be 17 chars
    pub make: String,
    pub vehicle_class: VehicleClass,
    pub dot_number: Option<String>,      // required if Commercial
    // ...
}

// The universal alert query — identical for every vertical
SELECT * FROM atlas_assets
WHERE tenant_id = $1
  AND (scheduled_service_date < NOW() + INTERVAL '30 days'
       OR expiry_date          < NOW() + INTERVAL '30 days')
ORDER BY LEAST(scheduled_service_date, expiry_date) ASC;
```

> **See §8 Risk #8** for documented tradeoffs of this approach and the conditions under which
> typed extension tables should be preferred.

### Party Model: `atlas_accounts`

**Migration:** `m20260901_party_account_hierarchy`

```sql
ALTER TABLE atlas_accounts
    ADD COLUMN parent_id       UUID REFERENCES atlas_accounts(id),
    ADD COLUMN entity_subtype  VARCHAR(50);  -- e.g. AGENCY / CARRIER / SUB_AGENCY / FRANCHISE

-- Sub-entity duplicate prevention (address-based uniqueness for sub-orgs)
CREATE UNIQUE INDEX idx_accounts_sub_entity_address
    ON atlas_accounts (parent_id, lower(billing_street), billing_postal_code)
    WHERE parent_id IS NOT NULL AND entity_subtype IS NOT NULL;
```

**Why now:** The Agency Provisioning spec requires a self-referential Agency hierarchy (parent agency → sub-agencies) and carrier distinction. This is universally needed for franchise networks, subsidiary management, and multi-level organization trees in any vertical.

### G-01: `geo_service_areas`

**Migration:** `m20260902_geo_regulatory_status`

```sql
ALTER TABLE geo_service_areas
    ADD COLUMN regulatory_status VARCHAR(50),   -- e.g. APPOINTED / UNAPPOINTED / RESTRICTED
    ADD COLUMN regulatory_notes  TEXT;
```

**Why now:** G-01 is the platform's geographic reference layer. Regulatory status per jurisdiction (whether a carrier is appointed in a state) is geographic metadata — it belongs here, not in a domain-specific table.

### G-16: `atlas_regulatory_registrations`

**Migration:** `m20260903_regulatory_registrations_extend`

```sql
ALTER TABLE atlas_regulatory_registrations
    ADD COLUMN account_id          UUID REFERENCES atlas_accounts(id),  -- the agency/org
    ADD COLUMN contact_id          UUID REFERENCES atlas_contacts(id),  -- the individual
    ADD COLUMN npn                 VARCHAR(50),   -- National Producer Number
    ADD COLUMN license_type        VARCHAR(100),  -- e.g. "Resident", "Non-Resident"
    ADD COLUMN lines_of_authority  TEXT[];        -- e.g. ["PROPERTY", "CASUALTY"]
```

**Why now:** G-16 was designed around asset/property permits (STR licenses, building permits). Extending it to cover individual professional licenses (insurance, financial advisor, healthcare) requires entity references at the person and organization level. The `npn` + `license_type` + `lines_of_authority` fields are insurance-domain names but the concepts (issuer ID, license class, authorized scope) are universal.

---

## 5. Agency Provisioning Spec — Full Generic Mapping

This table maps every entity in [`../agprov/agency_provisioning_spec.md`](../agprov/agency_provisioning_spec.md) to its Atlas generic without domain-specific objects.

| Spec Entity | §Ref | Atlas Generic | Mapping Notes |
|---|---|---|---|
| `Agency` | §3.1 | `atlas_accounts` + `entity_subtype` | `entity_subtype = "AGENCY"/"CARRIER"/"SUB_AGENCY"`. `parent_id` for hierarchy. |
| `Agent` | §3.2 | `atlas_contacts` (Party model) | Direct fit. |
| `AgentAgencyRelationship` | §3.3 | **G-32 `atlas_memberships`** | `membership_type = "AGENCY_AGENT"`. `roles`, `is_active`, `is_authorized_signer` are first-class columns. |
| `StateConfiguration` | §3.4 | **G-01 `geo_service_areas`** | `regulatory_status` + `regulatory_notes` added by v3 enhancement. |
| `Appointment` | §3.5 | **G-16 `atlas_regulatory_registrations`** | `account_id`, `contact_id`, `npn`, `license_type`, `lines_of_authority` added by v3 enhancement. |
| `SystemEntitlement` | §3.6 | **G-33 `atlas_entitlements`** | `system_identifier`, `scope_code`, `entitlement_key`, `is_gated`, `is_active`, `tier`, `external_id`. |
| `DistributionNetwork` | §3.7 | G-11 `atlas_contracts` | `contract_type = "DISTRIBUTION_NETWORK"`. `base_agreement_id` in `terms_metadata`. |
| `CarrierProduct` | §3.8 | G-26 `atlas_catalog_entries` | `catalog_metadata: { lob_category, system }`. |
| `AgencyStateCarrier` | §3.9 | G-11 `atlas_contracts` | `contract_type = "LOB_AGREEMENT"`. `terms_metadata: { personal_signed, commercial_signed }`. Or two contract rows per LOB. |
| `StateCarrier` (doc templates) | §3.10 | G-11 `atlas_contracts` | `contract_type = "STATE_CARRIER_TEMPLATE"`. Template IDs in `terms_metadata`. |
| `UserRoleTierMatrix` | §3.11 | G-26 `atlas_catalog_rate_rules` | `rule_name` = role string, `priority` = tier value, `channel` = system identifier. |
| `ProductSystemMap` | §3.12 | G-26 `atlas_catalog` | `catalog_metadata: { state_system_map: { "FL": "SYSTEM_A" } }` per catalog entry. |
| `GlobalProvisioningSettings` | §3.13 | `tenant_settings` | `w9_form_template_id` as a tenant setting key. |
| `Case` (LP approval) | §3.14 | G-13 `atlas_cases` | `case_type = "LP_PROVISIONING"/"LP_REMOVAL"`. `case_metadata: { linked_entitlement_id }`. Zero schema changes. |
| `Task` (unappointed state) | §3.15 | G-13 `atlas_cases` | `case_type = "UNAPPOINTED_STATE_INTEREST"`, `priority = "LOW"`. Zero schema changes. |

---

## 6. Implementation Order (Authoritative — v3 Additions)

> Phases 0-A and 0-B from v2 are complete. New ordering below covers v3 only.

### Phase v3-A: G-10 Lifecycle Enhancement (no dependency — run first)

1. `m20260900_g10_asset_lifecycle` — G-10: add `scheduled_service_date`, `expiry_date`, `condition`, `lifecycle_metadata` + indexes

### Phase v3-B: Party Model Enhancement

2. `m20260901_party_account_hierarchy` — `atlas_accounts`: add `parent_id`, `entity_subtype`

### Phase v3-C: Existing Generic Enhancements

3. `m20260902_geo_regulatory_status` — G-01: add `regulatory_status`, `regulatory_notes`
4. `m20260903_regulatory_registrations_extend` — G-16: add `account_id`, `contact_id`, `npn`, `license_type`, `lines_of_authority`

### Phase v3-D: New Generics

5. `m20260904_g32_atlas_memberships` — G-32: full table + indexes
6. `m20260905_g33_atlas_entitlements` — G-33: full table + indexes

### Phase v3-E: OutboxWorker Extensions

7. Register job types: `provision_entitlement`, `deprovision_entitlement`, `reconcile_entitlement`
8. Seed `UserRoleTierMatrix` config rows via `atlas_catalog_rate_rules` for all known downstream systems

**Rule:** v3 generics must be registered in `CorePlatformApp::migrations()` after all v2 generics. G-33 depends on G-32 (via `membership_id` FK). G-10 lifecycle enhancement has no inter-generic dependency and may be applied independently.

---

## 7. Cross-App Benefit Matrix (v3 Updated)

| Generic | Primary Driver | Strong Secondary Benefits |
|---------|----------------|---------------------------|
| `atlas_portfolios` | PM | ClaimSwift (adjuster pools), AgentLink (agency alliances) |
| `atlas_assets` | PM | ClaimSwift (damaged asset), Direct Booking (hotel rooms) |
| `atlas_contracts` | PM | CoverFlow (insurance policies), AgentLink (distribution agreements, LOB agreements) |
| `atlas_service_providers` | PM | ClaimSwift (adjusters), AgentLink (agents), Guest Comms (housekeeping) |
| `atlas_cases` | PM | ClaimSwift (claims), **AgencyProv (LP approval + unappointed state tasks)**, compliance |
| `atlas_documents` | PM | CoverFlow, ClaimSwift, AgentLink, Famtasm |
| `atlas_opportunities` | PM | CoverFlow (submissions), Direct Booking (pipelines) |
| `atlas_regulatory_registrations` | PM (STR) | **AgencyProv (state appointments, NPN)**, ClaimSwift (adjuster licenses), AgentLink |
| `atlas_tax_events / filings` | PM (TDT) | CoverFlow (premium tax), Clipping (1099-K) |
| `atlas_applications` | PM | AgentLink, Famtasm, Direct Booking |
| `atlas_catalog` | Direct Booking | **AgencyProv (product-system map, role-tier matrix)**, PM (rate rules) |
| `atlas_record_relationships` | CRM | Campaign ↔ lead, event ↔ contact, **entitlement ↔ registration** cross-links |
| **`atlas_memberships` (G-32)** | **AgencyProv** | PM (portfolio managers), SaaS teams, franchise networks, healthcare provider practices |
| **`atlas_entitlements` (G-33)** | **AgencyProv** | SaaS reseller provisioning, fintech advisor licensing, telecom subscriber provisioning |
| `geo_service_areas` | PM (geo) | **AgencyProv (state regulatory status)**, Direct Booking (market areas) |

---

## 8. Open Risks & Questions

1. **JSONB Ergonomics** (carried from v2) — Heavy reliance on `*_metadata JSONB` remains a concern in Rust service layers and Leptos forms.
2. **Polymorphic Query Performance** (carried from v2) — Index strategy review needed for high-cardinality JSONB queries.
3. **G-33 `entitlement_key` collation** — The unique index must use case-insensitive collation (`LOWER()` functional index or `CITEXT`). Verify PostgreSQL collation handling in the migration.
4. **G-32 cascade vs. G-33 cascade** — The membership cascade to entitlements is one-directional. The service layer must enforce that reactivating a membership does NOT auto-reactivate entitlements. This invariant must be covered by tests.
5. **G-16 backward compatibility** — Adding `account_id` / `contact_id` as nullable columns to `atlas_regulatory_registrations` is non-breaking. However, PM services that currently key regulatory registrations off `asset_id` will coexist with insurance provisioning that keys off `account_id` + `contact_id`. The `registration_type` field disambiguates.
6. **`atlas_accounts` parent_id cycle guard** — Self-referential FKs can create cycles. Enforce a maximum hierarchy depth (e.g. 3 levels: Carrier → Agency → Sub-Agency) at the service layer, not the DB.
7. **`atlas_catalog_rate_rules` semantic stretch** — Using G-26 rate rules as the `UserRoleTierMatrix` is structurally sound but semantically indirect. If this causes confusion in admin UIs, consider a purpose-built `atlas_scoring_rules` sub-table or a `tenant_settings` JSON blob.
8. **G-10 `lifecycle_metadata` JSONB — deliberate tradeoff, optimization triggers documented.**

   The decision to store domain-specific asset fields in `lifecycle_metadata JSONB` rather than typed extension tables is a deliberate tradeoff. It is the correct choice for the current stage of the platform but has known future costs:

   | Risk | Impact | Mitigation in place |
   |---|---|---|
   | DB-level type enforcement is absent | A serialization bug stores corrupt JSONB silently; discovered at read time | `TryFrom<&AssetModel>` in every app service; panic-on-deserialize in staging |
   | Schema is opaque to non-Rust tooling | BI tools, data analysts, admin scripts cannot introspect field structure | Per-app `lifecycle_metadata` shape documented in `docs/architecture/asset_metadata_shapes.md` (to be created) |
   | JSONB key renames require data backfills | Renaming `fuel_type` → `energy_source` across 50k rows needs a script | Treat JSONB keys as stable public API; version with `metadata_version` field |
   | Cross-app field queries are fragile | Platform-level queries on metadata fields need expression indexes and return TEXT not typed values | Only `scheduled_service_date`, `expiry_date`, `condition` are queried platform-wide; metadata is app-private |
   | Compliance tooling expects typed schema | Audit, SOC 2, HIPAA tooling may require queryable typed fields | Not a current requirement; re-evaluate at compliance certification stage |

   **Optimization triggers — convert `lifecycle_metadata` fields to typed extension tables when ANY of these occur:**

   - A platform-level feature needs to `WHERE` on a `lifecycle_metadata` field across multiple `asset_type` values
   - A compliance audit requires typed, queryable schema documentation
   - More than 3 apps need to share a subset of lifecycle fields (at that point the field is generic, not app-specific)
   - `lifecycle_metadata` deserialization errors appear in production error tracking
   - A vertical reaches >500k asset rows and JSONB write amplification becomes measurable

   The migration path when a trigger fires: create `atlas_asset_{capability}` table (e.g. `atlas_asset_lifecycle`), backfill from JSONB, add FK, deprecate JSONB key. The `asset_type` discriminator on `atlas_assets` ensures the backfill scope is bounded per vertical.

---

## 9. Implementation Notes

**G-32 `atlas_memberships`:**
- `roles` is validated at the service layer per `membership_type`. The DB stores any `TEXT[]`; the service enforces `["CSR", "LICENSED_PRODUCER", "AGENCY_ADMIN"]` for insurance, `["ADMIN", "EDITOR"]` for SaaS, etc.
- `is_authorized_signer` is a first-class column (not metadata) because it is queried in document dispatch with `LIMIT 1` — must be indexed.
- The `UNIQUE` constraint is on `(tenant_id, parent_account_id, member_contact_id, membership_type)` to allow a contact to have multiple membership types within the same org (e.g., both `"AGENCY_AGENT"` and `"BILLING_CONTACT"`).

**G-33 `atlas_entitlements`:**
- `entitlement_key` format is caller-defined. The recommended format is `lower("{account_id}:{contact_id}:{provider_id}:{system}:{scope_code}")`. All UUIDs should be in canonical lowercase UUID format before concatenation.
- `provision_status` transitions: `PENDING → PROVISIONING → ACTIVE → DEPROVISIONING → DEPROVISIONED`. Failed states: `FAILED`. Retries reset to `PROVISIONING`.
- `linked_approval_id` (FK → `atlas_cases`) enforces the approval gate: a service guard must verify `atlas_cases.status = 'APPROVED'` before flipping `is_active = TRUE` on any entitlement where `is_gated = TRUE`.
- External adapters are **not** defined in this generic. Each downstream system has an adapter module that accepts the canonical entitlement payload and translates to the target API.

**G-01 `geo_service_areas` enhancement:**
- `regulatory_status` valid values: `APPOINTED`, `UNAPPOINTED`, `RESTRICTED`. Nullable for non-insurance use cases.
- Existing PM service code reading `geo_service_areas` is unaffected — columns are additive.

**G-16 `atlas_regulatory_registrations` enhancement:**
- `jurisdiction_code` continues to carry the state code (e.g., `"FL"`) for non-FK use cases.
- `scope_entity_id` on G-33 provides the strict FK path to `geo_service_areas` when referential integrity is needed.

---

## 10. Current Gaps & Next Build Priorities (v3)

After G-32 and G-33:

1. **G-32/G-33 Migrations** — Author and run the 5 migrations in v3 Phase A–C order.
2. **MembershipService** — `upsert_membership`, `deactivate_membership` (with G-33 cascade), `find_authorized_signer`.
3. **EntitlementService** — `upsert_entitlement` (idempotent key), `activate_entitlement` (approval guard), `cascade_deactivate`.
4. **OutboxWorker extension** — Register 3 new job types: `provision_entitlement`, `deprovision_entitlement`, `reconcile_entitlement`.
5. **Agency Provisioning REST API** — Wire `POST /api/v1/agent-management/provision`, `POST /api/v1/agent-management/remove-lp`, `POST /api/v1/agencies/sub-agencies` against G-32/G-33 services.
6. **Admin UI for `UserRoleTierMatrix`** — Surface G-26 rate rules configured for entitlement tier computation in platform-admin.
7. **PM Frontend App** (carried from CURRENT_STATE) — Backend fully deployed, no Leptos frontend yet.
8. **G-28/G-29 Handler Migration** — `notes.rs` / `activities.rs` still reference legacy entity files.
9. **Legacy CRM Teardown** — Drop dual-write bridge after PM app validation.
10. **PostGIS in CI** — Switch to `postgis/postgis:16-3.4` Docker image.

---

**References**

- Superseded: [`platform_generics_v2.md`](./platform_generics_v2.md)
- Original 8 generics: [`platform_generics.md`](./platform_generics.md)
- Agency Provisioning RFP (trigger for G-32/G-33): [`../agprov/agency_provisioning_spec.md`](../agprov/agency_provisioning_spec.md)
- Generic mapping analysis: see conversation artifact `agency_provisioning_generic_mapping.md`
- Property Management Generics: [`../property-management/23_second_round_generics.md`](../property-management/23_second_round_generics.md)
- Payment Rails Extension: [`../property-management/25_payment_rails_architecture.md`](../property-management/25_payment_rails_architecture.md)
- AtlasApp Integration Protocol: [`../atlas_app_integration.md`](../atlas_app_integration.md)
- Current State: [`../CURRENT_STATE.md`](../CURRENT_STATE.md)

---

© Copyright Ruud Salym Erie & Oplyst International, LLC. All Rights Reserved.
