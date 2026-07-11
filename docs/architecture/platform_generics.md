# Atlas Platform — Generic Subsystems (Original 8 — Historical Reference)

> **Important**: This document covers the original 8 infrastructure generics. The complete current architecture (all generics + Account/Contact unification + service layer) is documented in:
> - [`../CURRENT_STATE.md`](../CURRENT_STATE.md) — living registry
> - [`platform_generics_v2.md`](./platform_generics_v2.md) / [`platform_generics_v3.md`](./platform_generics_v3.md) — historical/implemented specs
>
> **Rule 7 — Generic Fitness Test (living):** [`generic_fitness_test.md`](./generic_fitness_test.md)

> See also:
> - [`docs/atlas_app_integration.md`](../atlas_app_integration.md) — AtlasApp trait (must run Rule 7 before new tables)
> - [`docs/architecture.md`](../architecture.md) — Full ERD including these tables
> - [`docs/architecture/auth_and_permissions.md`](./auth_and_permissions.md) — Permission model

---

## Overview

Eight structural patterns appear in 3 or more roadmap apps. Building each one
app-by-app would duplicate migrations, SQL, and service logic across the codebase.
These subsystems are promoted to the **Atlas base platform** — they are registered in
`CorePlatformApp::migrations()` and exposed as shared Rust services in
`backend/src/services/`.

**Rule:** Before writing a net-new app migration, complete Rule 7 in [`generic_fitness_test.md`](./generic_fitness_test.md), then check this table:

| Need | Use |
|------|-----|
| Store a file in R2 | `attachment` + `attachment_share_tokens` (→ GENERIC-02) |
| Record any payment | `atlas_ledger_entries` + `atlas_ledger_splits` (→ GENERIC-03) |
| Connect to PMS / AMS / OTA / GDS | `atlas_external_integrations` (→ GENERIC-05) |
| Spatial / geo query | `geo_service_areas` PostGIS index (→ GENERIC-01) |
| Human trust verification workflow | `atlas_verification_requests` (→ GENERIC-06) |
| B2C recurring subscription | `atlas_subscriptions` (→ GENERIC-04) |
| Real-time WebSocket room | `atlas_ws_rooms` + `atlas_ws_messages` (→ GENERIC-07) |
| Call an LLM or AI model asynchronously | `atlas_ai_tasks` (→ GENERIC-08) |

---

## GENERIC-01: `atlas_geo` — Spatial / PostGIS Foundation

**Apps:** AgentLink, ClaimSwift, Nomad List, Classified Directory, STR Compliance (PM)

### Migration (CorePlatformApp)

```sql
-- Enable once for the entire cluster:
CREATE EXTENSION IF NOT EXISTS postgis;

CREATE TABLE geo_service_areas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
    owner_entity_type VARCHAR(50) NOT NULL,  -- 'agency', 'adjuster', 'property', 'listing'
    owner_entity_id UUID NOT NULL,
    label VARCHAR(100),
    geom GEOMETRY(MultiPolygon, 4326),
    point GEOGRAPHY(Point, 4326),
    zip_codes TEXT[],
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX geo_service_areas_geom_idx ON geo_service_areas USING GIST(geom);
CREATE INDEX geo_service_areas_point_idx ON geo_service_areas USING GIST(point);
CREATE INDEX geo_service_areas_tenant_type_idx ON geo_service_areas(tenant_id, owner_entity_type);
```

### Rust Service

```rust
// backend/src/services/geo.rs
use sea_orm::{DatabaseConnection, Statement, DbBackend};
use uuid::Uuid;

pub struct GeoService;

impl GeoService {
    /// All service areas whose polygon contains the given lat/lng.
    pub async fn find_areas_containing_point(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        owner_type: &str,
        lat: f64,
        lng: f64,
    ) -> Result<Vec<GeoServiceAreaRow>, sea_orm::DbErr> {
        let wkt = format!("POINT({lng} {lat})");
        db.query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT id, owner_entity_id, label
               FROM geo_service_areas
               WHERE tenant_id = $1
                 AND owner_entity_type = $2
                 AND (
                   ST_Contains(geom, ST_GeomFromText($3, 4326))
                   OR $4 = ANY(zip_codes)
                 )"#,
            vec![tenant_id.into(), owner_type.into(), wkt.clone().into(), "".into()],
        )).await.map(|rows| rows.into_iter().map(GeoServiceAreaRow::from_query_result).collect())
    }

    /// Radius search in meters around a point.
    pub async fn find_within_radius(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        owner_type: &str,
        lat: f64,
        lng: f64,
        radius_meters: f64,
    ) -> Result<Vec<GeoServiceAreaRow>, sea_orm::DbErr> {
        db.query_all(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT id, owner_entity_id, label,
                      ST_Distance(point, ST_MakePoint($3, $4)::geography) AS distance_m
               FROM geo_service_areas
               WHERE tenant_id = $1
                 AND owner_entity_type = $2
                 AND ST_DWithin(point, ST_MakePoint($3, $4)::geography, $5)
               ORDER BY distance_m ASC"#,
            vec![tenant_id.into(), owner_type.into(), lng.into(), lat.into(), radius_meters.into()],
        )).await.map(|rows| rows.into_iter().map(GeoServiceAreaRow::from_query_result).collect())
    }
}
```

---

## GENERIC-02: `atlas_vault` — Secure File Storage

**Apps:** PM, ClaimSwift, CoverFlow, Branded Delivery, Famtasm, AgentLink

Extends the existing `attachment` table and adds:
- Presigned GET URL generation
- Guest / external share tokens (no platform login required)
- Multipart upload state tracking for large files

### Migration (CorePlatformApp — extends existing `attachment` table)

```sql
-- Extend existing attachment (idempotent — IF NOT EXISTS):
ALTER TABLE attachment
    ADD COLUMN IF NOT EXISTS access_level VARCHAR(30) DEFAULT 'private',
    ADD COLUMN IF NOT EXISTS r2_bucket VARCHAR(100),
    ADD COLUMN IF NOT EXISTS r2_key VARCHAR(512),
    ADD COLUMN IF NOT EXISTS mime_type VARCHAR(100),
    ADD COLUMN IF NOT EXISTS checksum_sha256 VARCHAR(64),
    ADD COLUMN IF NOT EXISTS upload_status VARCHAR(20) DEFAULT 'complete';
    -- 'pending_upload', 'uploading', 'complete', 'failed'

-- Guest/external share tokens:
CREATE TABLE IF NOT EXISTS attachment_share_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
    attachment_id UUID NOT NULL REFERENCES attachment(id) ON DELETE CASCADE,
    token VARCHAR(128) NOT NULL UNIQUE DEFAULT encode(gen_random_bytes(48), 'hex'),
    resource_type VARCHAR(50) NOT NULL,   -- 'lease', 'inspection_report', 'video', 'portfolio', 'permit'
    permissions TEXT[] NOT NULL DEFAULT '{read}',
    recipient_email VARCHAR(255),
    expires_at TIMESTAMPTZ NOT NULL,
    one_time_use BOOLEAN DEFAULT FALSE,
    used_at TIMESTAMPTZ,
    created_by_user_id UUID REFERENCES "user"(id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX attachment_share_tokens_token_idx ON attachment_share_tokens(token);
CREATE INDEX attachment_share_tokens_attachment_idx ON attachment_share_tokens(attachment_id);

-- Multipart upload state:
CREATE TABLE IF NOT EXISTS attachment_multipart_uploads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
    attachment_id UUID NOT NULL REFERENCES attachment(id) ON DELETE CASCADE,
    r2_upload_id VARCHAR(255) NOT NULL,
    total_parts INT,
    completed_parts INT DEFAULT 0,
    status VARCHAR(20) DEFAULT 'in_progress',
    -- 'in_progress', 'finalizing', 'complete', 'aborted'
    created_at TIMESTAMPTZ DEFAULT NOW()
);
```

### Rust Service

```rust
// backend/src/services/vault.rs
use aws_sdk_s3::presigning::PresigningConfig;
use std::time::Duration;

pub struct VaultService {
    pub r2_client: aws_sdk_s3::Client,
    pub r2_bucket: String,
}

impl VaultService {
    /// Initiate a multipart R2 upload. Returns (attachment_id, r2_upload_id).
    pub async fn initiate_upload(
        &self,
        db: &DatabaseConnection,
        tenant_id: Uuid,
        file_name: &str,
        mime_type: &str,
        size_bytes: i64,
    ) -> Result<(Uuid, String)>;

    /// Generate a short-lived presigned R2 GET URL for an attachment.
    pub async fn presign_get(
        &self,
        r2_key: &str,
        expires_secs: u64,
    ) -> Result<String> {
        let presigning = PresigningConfig::expires_in(Duration::from_secs(expires_secs))?;
        let req = self.r2_client
            .get_object()
            .bucket(&self.r2_bucket)
            .key(r2_key)
            .presigned(presigning)
            .await?;
        Ok(req.uri().to_string())
    }

    /// Create a share token granting external access to an attachment.
    pub async fn create_share_token(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        attachment_id: Uuid,
        resource_type: &str,
        expires_at: DateTime<Utc>,
        one_time_use: bool,
        created_by: Uuid,
    ) -> Result<AttachmentShareToken>;

    /// Validate a share token. Marks one-time tokens as used.
    pub async fn validate_share_token(
        db: &DatabaseConnection,
        token: &str,
    ) -> Result<AttachmentShareToken>;
}
```

### How apps use it

| App | `resource_type` | Notes |
|-----|----------------|-------|
| PM | `lease`, `permit`, `inspection` | Vault docs reference `attachment_id` |
| Branded Delivery | `portfolio` | Entire delivery portal is `attachment_share_tokens` |
| ClaimSwift | `inspection_report`, `loss_photo` | Inspector accesses via share token |
| Famtasm | `creator_video` | Cloudflare Stream UID stored in `r2_key` |
| AgentLink | `consent_document` | GLBA consent PDFs |

---

## GENERIC-03: `atlas_payments` — Multi-Rail Payment Ledger

**Apps:** PM, CoverFlow, Famtasm, Clipping Marketplace, Direct Booking Engine

> **Important:** This is NOT a replacement for the `transaction` table (which records
> operator-level SaaS billing). `atlas_ledger_entries` records payments *within* an app
> (rent, premiums, subscriptions, campaign payouts).

### Migration (CorePlatformApp)

```sql
CREATE TYPE atlas_payment_rail AS ENUM (
    'stripe', 'stripe_connect',
    'btc_onchain', 'btc_lightning',
    'zelle', 'cash_app', 'apple_pay', 'google_pay',
    'pix', 'wire', 'ach',
    'western_union', 'moneygram', 'cash'
);

CREATE TYPE atlas_ledger_status AS ENUM (
    'pending', 'processing', 'paid', 'partially_paid',
    'late', 'failed', 'refunded', 'waived', 'reconciled'
);

CREATE TABLE atlas_ledger_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
    -- What this payment is FOR (polymorphic):
    billable_entity_type VARCHAR(50) NOT NULL,
    -- 'pm_lease', 'booking', 'creator_subscription', 'insurance_policy', 'campaign_escrow'
    billable_entity_id UUID NOT NULL,
    -- Payer:
    payer_user_id UUID REFERENCES "user"(id),
    payer_email VARCHAR(255),
    -- Amount:
    gross_amount_cents BIGINT NOT NULL,
    fee_amount_cents BIGINT NOT NULL DEFAULT 0,
    net_amount_cents BIGINT GENERATED ALWAYS AS (gross_amount_cents - fee_amount_cents) STORED,
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    -- Payment:
    payment_rail atlas_payment_rail,
    external_tx_id VARCHAR(256),
    receipt_attachment_id UUID REFERENCES attachment(id),
    -- Status:
    status atlas_ledger_status NOT NULL DEFAULT 'pending',
    due_date DATE,
    paid_at TIMESTAMPTZ,
    -- Verification:
    verified_by_user_id UUID REFERENCES "user"(id),
    verified_at TIMESTAMPTZ,
    -- Reconciliation:
    reconciled_at TIMESTAMPTZ,
    reconciliation_note TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX atlas_ledger_entries_entity_idx
    ON atlas_ledger_entries(tenant_id, billable_entity_type, billable_entity_id);
CREATE INDEX atlas_ledger_entries_status_due_idx
    ON atlas_ledger_entries(tenant_id, status, due_date);

-- Split destinations (1 entry → N payout splits):
CREATE TABLE atlas_ledger_splits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ledger_entry_id UUID NOT NULL REFERENCES atlas_ledger_entries(id) ON DELETE CASCADE,
    recipient_type VARCHAR(50) NOT NULL,
    -- 'platform', 'vendor', 'creator', 'broker', 'carrier', 'mga'
    recipient_user_id UUID REFERENCES "user"(id),
    recipient_label VARCHAR(100),
    amount_cents BIGINT NOT NULL,
    payout_rail atlas_payment_rail,
    payout_status VARCHAR(30) DEFAULT 'pending',
    payout_tx_id VARCHAR(256),
    settled_at TIMESTAMPTZ
);
```

### How apps use it

| App | `billable_entity_type` | Split recipients |
|-----|------------------------|-----------------|
| PM (rent) | `pm_lease` | None (single landlord) |
| CoverFlow | `insurance_policy` | carrier, mga, broker |
| Famtasm | `creator_subscription` | creator (80%), platform (20%) |
| Clipping | `campaign_escrow` | clipper (CPM rate) |
| Direct Booking | `hotel_booking` | hotel (via Stripe Connect) |

---

## GENERIC-04: `atlas_subscriptions` — B2C Recurring Billing

**Apps:** Famtasm, Clipping Marketplace, Revenue Manager, PM STR Compliance OS

> `TenantSubscription` handles B2B SaaS (operator pays for their platform plan).
> `atlas_subscriptions` handles B2C (a user pays to subscribe to a creator, city plan, etc.)

### Migration (CorePlatformApp)

```sql
CREATE TYPE atlas_subscription_status AS ENUM (
    'trialing', 'active', 'past_due', 'canceled', 'paused', 'incomplete'
);

CREATE TABLE atlas_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
    subscriber_user_id UUID NOT NULL REFERENCES "user"(id),
    -- What is being subscribed to (polymorphic):
    subscribed_to_type VARCHAR(50) NOT NULL,
    -- 'creator', 'str_compliance_plan', 'revenue_mgr_plan', 'campaign_tier'
    subscribed_to_id UUID NOT NULL,
    -- Plan details:
    billing_plan_id UUID REFERENCES billing_plan(id),
    price_cents BIGINT NOT NULL,
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    billing_interval VARCHAR(20) NOT NULL DEFAULT 'monthly',
    -- External billing:
    stripe_subscription_id VARCHAR(100),
    stripe_customer_id VARCHAR(100),
    -- Status:
    status atlas_subscription_status NOT NULL DEFAULT 'active',
    trial_ends_at TIMESTAMPTZ,
    current_period_start TIMESTAMPTZ,
    current_period_end TIMESTAMPTZ,
    canceled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX atlas_subscriptions_subscriber_idx
    ON atlas_subscriptions(tenant_id, subscriber_user_id, status);
CREATE INDEX atlas_subscriptions_entity_idx
    ON atlas_subscriptions(tenant_id, subscribed_to_type, subscribed_to_id);
```

---

## GENERIC-05: `atlas_external_integrations` — Third-Party API Gateway

**Apps:** Direct Booking + Revenue Manager (Cloudbeds, Mews, Guesty, Hostaway),
Guest Comms (PMS + Twilio), AgentLink (AMS), ClaimSwift (Guidewire, Duck Creek),
CoverFlow (Applied, Salesforce), PM (OTA: Airbnb, VRBO, Booking.com)

### Migration (CorePlatformApp)

```sql
CREATE TABLE atlas_external_integrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
    integration_type VARCHAR(50) NOT NULL,
    -- 'pms_cloudbeds' | 'pms_mews' | 'pms_guesty' | 'pms_hostaway'
    -- 'ota_airbnb' | 'ota_vrbo' | 'ota_booking_com'
    -- 'ams_applied' | 'ams_vertafore' | 'ams_guidewire' | 'ams_duck_creek'
    -- 'gds_sabre' | 'gds_amadeus'
    -- 'telephony_twilio' | 'telephony_telnyx'
    label VARCHAR(100),
    credentials_encrypted JSONB NOT NULL,  -- encrypted at app layer before storage
    webhook_secret VARCHAR(100),
    webhook_url VARCHAR(512),              -- our endpoint to receive inbound events
    is_active BOOLEAN DEFAULT TRUE,
    last_synced_at TIMESTAMPTZ,
    last_error TEXT,
    config JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, integration_type)
);
CREATE INDEX atlas_integrations_tenant_type ON atlas_external_integrations(tenant_id, integration_type);

CREATE TABLE atlas_integration_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
    integration_id UUID NOT NULL REFERENCES atlas_external_integrations(id) ON DELETE CASCADE,
    event_type VARCHAR(100) NOT NULL,
    -- 'reservation.created', 'rate.pushed', 'claim.synced', 'booking.modified'
    direction VARCHAR(10) NOT NULL,         -- 'inbound', 'outbound'
    payload JSONB,
    status VARCHAR(20) DEFAULT 'pending',   -- 'pending', 'processed', 'failed', 'skipped'
    error_message TEXT,
    processed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX atlas_integration_events_status ON atlas_integration_events(integration_id, status, created_at);
```

### Rust Adapter Trait

```rust
// backend/src/services/integrations/mod.rs
#[async_trait]
pub trait ExternalIntegrationAdapter: Send + Sync {
    fn integration_type(&self) -> &'static str;

    async fn validate_credentials(
        &self,
        credentials: &serde_json::Value,
    ) -> Result<bool, IntegrationError>;

    async fn process_inbound_event(
        &self,
        db: &DatabaseConnection,
        tenant_id: Uuid,
        integration: &AtlasExternalIntegration,
        payload: serde_json::Value,
    ) -> Result<(), IntegrationError>;

    async fn execute_sync_cycle(
        &self,
        db: &DatabaseConnection,
        tenant_id: Uuid,
        integration: &AtlasExternalIntegration,
    ) -> Result<(), IntegrationError>;
}

// Registry — each adapter is a separate file:
// backend/src/services/integrations/
//   cloudbeds.rs    → impl ExternalIntegrationAdapter for CloudbedsAdapter
//   mews.rs
//   airbnb.rs
//   guidewire.rs
//   twilio.rs
//   ...

pub fn get_integration_adapters() -> HashMap<&'static str, Box<dyn ExternalIntegrationAdapter>> {
    [
        ("pms_cloudbeds", Box::new(CloudbedsAdapter) as Box<dyn ExternalIntegrationAdapter>),
        ("pms_mews", Box::new(MewsAdapter)),
        ("ota_airbnb", Box::new(AirbnbAdapter)),
        ("ams_guidewire", Box::new(GuidewireAdapter)),
        ("telephony_twilio", Box::new(TwilioAdapter)),
    ].into_iter().collect()
}
```

---

## GENERIC-06: `atlas_verification_queue` — Human-in-the-Loop Trust Verification

**Apps:** Classified (selfie OCR), ClaimSwift (GPS EXIF fraud), AgentLink (insurance license),
CoverFlow (ACORD docs), PM (vendor license, STR permit inspection)

### Migration (CorePlatformApp)

```sql
CREATE TYPE atlas_verification_status AS ENUM (
    'pending_upload',
    'auto_checking',
    'requires_manual_review',
    'approved',
    'rejected',
    'expired'
);

CREATE TABLE atlas_verification_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
    -- What is being verified (polymorphic):
    subject_type VARCHAR(50) NOT NULL,
    -- 'vendor_license', 'str_permit', 'insurance_license', 'listing_identity'
    -- 'adjuster_gps_photo', 'acord_document', 'selfie_identity'
    subject_id UUID NOT NULL,
    requested_by_user_id UUID NOT NULL REFERENCES "user"(id),
    -- Evidence:
    attachment_id UUID REFERENCES attachment(id),
    auto_check_result JSONB,
    -- e.g. {"gps_distance_m": 45, "passed": true}
    -- e.g. {"exif_found": true, "lat": 25.77, "lng": -80.19}
    auto_check_passed BOOLEAN,
    -- Manual review:
    status atlas_verification_status NOT NULL DEFAULT 'pending_upload',
    reviewed_by_user_id UUID REFERENCES "user"(id),
    reviewed_at TIMESTAMPTZ,
    rejection_reason TEXT,
    -- Optional expiry for licenses, permits:
    verified_value VARCHAR(255),           -- license number, permit number, etc.
    expires_at DATE,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX atlas_verification_status_idx
    ON atlas_verification_requests(tenant_id, subject_type, status);
```

---

## GENERIC-07: `atlas_realtime` — WebSocket Room Infrastructure

**Apps:** Nomad List (entity chat), Guest Comms (live chat + housekeeping board),
PM (maintenance ticket thread), Clipping (campaign leaderboard)

### Migration (CorePlatformApp)

```sql
CREATE TABLE atlas_ws_rooms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
    room_type VARCHAR(50) NOT NULL,
    -- 'entity_chat', 'ticket_thread', 'campaign_board', 'guest_inbox', 'task_board'
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, room_type, entity_type, entity_id)
);

CREATE TABLE atlas_ws_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_id UUID NOT NULL REFERENCES atlas_ws_rooms(id) ON DELETE CASCADE,
    sender_user_id UUID REFERENCES "user"(id),   -- NULL = system message
    message_type VARCHAR(30) DEFAULT 'text',
    -- 'text', 'system', 'translation', 'task_update', 'media'
    content TEXT NOT NULL,
    translated_content JSONB,
    -- {"es": "...", "ht": "...", "pt": "..."}
    attachment_id UUID REFERENCES attachment(id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX atlas_ws_messages_room_idx ON atlas_ws_messages(room_id, created_at DESC);
```

### Axum WebSocket Handler (registered once in platform router)

```rust
// backend/src/handlers/realtime.rs
use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use tokio::sync::broadcast;

// Platform-level broadcast map: room_id → broadcast::Sender<WsEvent>
pub type WsRoomRegistry = Arc<DashMap<Uuid, broadcast::Sender<WsEvent>>>;

pub async fn ws_upgrade_handler(
    ws: WebSocketUpgrade,
    Extension(session): Extension<UserSession>,
    Path(room_id): Path<Uuid>,
    State(db): State<DatabaseConnection>,
    Extension(registry): Extension<WsRoomRegistry>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_connection(socket, session, room_id, db, registry))
}

async fn handle_ws_connection(
    socket: WebSocket,
    session: UserSession,
    room_id: Uuid,
    db: DatabaseConnection,
    registry: WsRoomRegistry,
) {
    // Validate user has access to room's entity
    // Join or create broadcast channel for room_id
    // Forward messages to DB + broadcast to other subscribers
}
```

---

## GENERIC-08: `atlas_ai_tasks` — Async LLM / AI Processing Queue

**Apps:** Guest Comms (GPT-4o translation, concierge), Revenue Manager (pricing AI),
ClaimSwift (Whisper transcription), Clipping (fraud scoring)

**Rule:** Never call an LLM API inline from a handler or server function. Always enqueue
to `atlas_ai_tasks` and process via `TenantBackgroundJob`.

### Migration (CorePlatformApp)

```sql
CREATE TABLE atlas_ai_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
    task_type VARCHAR(50) NOT NULL,
    -- 'translate_message', 'transcribe_audio', 'price_recommendation'
    -- 'fraud_score', 'concierge_recommendation', 'document_extraction'
    model VARCHAR(50),
    -- 'gpt-4o', 'whisper-1', 'claude-3-5-sonnet', 'gemini-2.5-pro'
    input_payload JSONB NOT NULL,
    output_payload JSONB,
    -- Source context (polymorphic):
    source_entity_type VARCHAR(50),
    source_entity_id UUID,
    -- Callback: where to write the result
    callback_entity_type VARCHAR(50),
    callback_entity_id UUID,
    callback_field VARCHAR(100),
    -- Status:
    status VARCHAR(20) DEFAULT 'queued',
    -- 'queued', 'processing', 'completed', 'failed'
    error_message TEXT,
    retry_count INT DEFAULT 0,
    -- Timing:
    queued_at TIMESTAMPTZ DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    -- Cost tracking:
    input_tokens INT,
    output_tokens INT,
    estimated_cost_micro_usd INT       -- microdollars for precision
);

CREATE INDEX atlas_ai_tasks_status_idx ON atlas_ai_tasks(tenant_id, status, queued_at);
CREATE INDEX atlas_ai_tasks_entity_idx ON atlas_ai_tasks(tenant_id, source_entity_type, source_entity_id);
```

### Background Job Registration (per-app that uses AI)

```rust
// Each app that uses AI tasks declares a BackgroundJob:
BackgroundJob {
    job_type: "process_ai_tasks".to_string(),
    default_interval_seconds: 10,         // poll every 10 seconds
    is_active_by_default: true,
    default_config_payload: Some(json!({
        "task_types": ["translate_message", "concierge_recommendation"],
        "batch_size": 5
    })),
    executor: Box::new(|db, config| {
        Box::pin(async move {
            crate::services::ai_tasks::process_queued_tasks(&db, config).await
        })
    }),
}
```

---

## Migration Registration

All 8 generics are registered in `CorePlatformApp::migrations()` — **never in app-specific registries**:

```rust
// backend/src/atlas_apps/core_platform.rs
impl AtlasApp for CorePlatformApp {
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        vec![
            // Existing core migrations...
            Box::new(m20260101_000001_core_schema::Migration),

            // ── Generic Subsystem Migrations (ordered by dependency) ──
            Box::new(m20260601_g01_geo_postgis::Migration),
            Box::new(m20260601_g02_vault_extension::Migration),
            Box::new(m20260601_g03_ledger::Migration),
            Box::new(m20260601_g04_subscriptions::Migration),
            Box::new(m20260601_g05_external_integrations::Migration),
            Box::new(m20260601_g06_verification_queue::Migration),
            Box::new(m20260601_g07_realtime::Migration),
            Box::new(m20260601_g08_ai_tasks::Migration),
        ]
    }
}
```

## Build Priority

| Priority | Generic | Block on |
|----------|---------|----------|
| **1** | GENERIC-02 `atlas_vault` | PM, CoverFlow, Famtasm, ClaimSwift all need R2 immediately |
| **2** | GENERIC-03 `atlas_payments` | PM, CoverFlow, Famtasm — eliminates `pm_ledger_transactions` |
| **3** | GENERIC-01 `atlas_geo` | PostGIS must be enabled once — 5 apps need it |
| **4** | GENERIC-05 `atlas_external_integrations` | Direct Booking + Revenue Manager are next |
| **5** | GENERIC-06 `atlas_verification_queue` | Simplifies PM permits + ClaimSwift fraud |
| **6** | GENERIC-04 `atlas_subscriptions` | Famtasm and Clipping are B2C |
| **7** | GENERIC-07 `atlas_realtime` | Nomad List + Guest Comms need WS |
| **8** | GENERIC-08 `atlas_ai_tasks` | Guest Comms + Revenue Mgr — after outbox is hardened |
