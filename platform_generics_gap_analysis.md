# Atlas Platform — Horizontal Gap Analysis: Missing Platform Generics
## Principal Architect Review — June 2026

> **Scope:** All 14 roadmap apps × 5 analytical lenses × G01–G18 current state
> **Goal:** Identify platform primitives that multiple apps will build independently if not promoted to generics now
> **Outcome:** 8 new generic candidates (G19–G26), scored and sequenced

---

## Preface: What the Growth Platform Document Already Told Us

Before the five-lens analysis, note that the Growth Platform spec (`docs/growth-platform/01_campaign_events_related_lists.md`) already proposed G19–G22 with full DDL. This analysis **confirms those four, extends them with scoring, and identifies four additional candidates** (G23–G26) that the Growth Platform analysis did not cover. All eight are analyzed uniformly below.

---

## Section 1: Cross-App JSONB Abuse Inventory

The following patterns are the highest-risk signals — data that will be stuffed into JSONB columns on existing generics rather than getting proper tables. Each represents a future migration nightmare.

| App | Generic Being Abused | JSONB Field | Data Being Stored | Missing Generic |
|-----|---------------------|-------------|-------------------|-----------------|
| **Direct Booking Engine** | G13 `atlas_cases` (`case_metadata`) | `check_in`, `check_out`, `room_type`, `guest_count`, `rate_applied`, `hold_expires_at` | A time-bounded room reservation with inventory hold | G23 `atlas_reservations` |
| **Event-Aware Revenue Manager** | G13 `atlas_cases` (`case_metadata`) | `property_id`, `date`, `suggested_rate_cents`, `floor_cents`, `ceiling_cents`, `autopilot` | A rate recommendation for a specific date slot | G23 `atlas_reservations` (rate slot variant) |
| **Flight + Hotel Package Builder** | G13 `atlas_cases` (`case_metadata`) | `flight_offer`, `hotel_offer`, `bundle_price_cents`, `lock_expires_at`, `duffel_hold_id` | A time-bounded multi-item price lock with external hold IDs | G23 `atlas_reservations` |
| **CoverFlow** | G15 `atlas_opportunities` (`financial_inputs`) | `base_premium`, `revenue_mult`, `loss_mult`, `limits`, `deductible`, `prior_losses` | An insurance rating engine's input/output calculation — a quote | G24 `atlas_quotes` |
| **AgentLink** | G15 `atlas_opportunities` (`financial_inputs`) | `commission_pct`, `split_to_account_id`, `split_type`, `override_pct` | A commission plan governing how a split is calculated | G25 `atlas_commission_plans` |
| **Direct Booking Engine** | G11 `atlas_contracts` (`terms_metadata`) | `negotiated_rate`, `block_rooms`, `cutoff_date`, `release_policy`, `commission_to_travel_agent` | A corporate rate agreement with GDS codes and block inventory | G24 `atlas_quotes` (quote line item variant) |
| **Flight + Hotel Package Builder** | G15 `atlas_opportunities` (`financial_inputs`) | `flight_segment[]`, `hotel_night[]`, `fx_rate`, `markup_pct`, `bundle_discount_pct` | A multi-line-item priced proposal for a travel package | G24 `atlas_quotes` + Quote Line Items |
| **Growth Platform / All Apps** | G15 `atlas_opportunities` | (would need new column) | "Which campaign brought in this lead?" — attribution link | G20 `atlas_attribution` |
| **Clipping Marketplace** | G13 `atlas_cases` (`case_metadata`) | `campaign_id`, `brand_id`, `submission_url`, `view_count`, `payout_rate_cpm`, `fraud_score` | A campaign submission workflow — a structured work item tied to a campaign | G19 `atlas_campaigns` |
| **Multilingual Guest Comms** | G13 `atlas_cases` (`case_metadata`) | `guest_phone`, `reservation_id`, `language`, `pms_folio`, `check_in_date`, `check_out_date` | A guest communication context — this is a reservation, not a case | G23 `atlas_reservations` |
| **Revenue Manager** | G10 `atlas_assets` (`attributes`) | `rate_calendar`, `availability_windows`, `min_stay`, `channel_rules` | A rate and availability calendar — a product pricebook, not an asset attribute | G26 `atlas_catalog` (availability/pricebook) |
| **AgentLink** | G10 `atlas_assets` (`attributes`) | `license_number`, `license_state`, `license_type`, `expires_at`, `ce_credits_required` | Agent license data — already covered by G16, but being jammed into assets instead | G16 already exists — Rule 7 enforcement gap |
| **All sales prospects (insurance brokerage, HVAC, loan broker)** | G11 `atlas_contracts` (`terms_metadata`) | `commission_schedule[]`, `split_tiers`, `override_threshold`, `clawback_days` | Multi-tier commission plans for producers, brokers, agents | G25 `atlas_commission_plans` |
| **PM + Direct Booking + Revenue Manager** | G03 `atlas_ledger_entries` (`billable_entity_type = 'booking'`) | booking detail in free-form comment field | There is no canonical booking/reservation entity to FK to | G23 `atlas_reservations` |

> **Critical observation:** The `atlas_ledger_entries` table uses a polymorphic `billable_entity_type` / `billable_entity_id` pattern. Right now, multiple apps are creating their own private `direct_bookings`, `guest_reservations`, and `booking_items` tables that will each become the `billable_entity_id`. Once two apps create different reservation tables, cross-platform financial reporting becomes impossible without a UNION across app-specific schemas. This is the single most dangerous technical debt vector on the roadmap.

---

## Section 2: App-Specific Tables That Should Be Generics

Every table below was explicitly called out in an `atlas_integration_mapping.md` or product spec. Tables appearing in 2+ apps in equivalent form are generic candidates.

| App-Specific Table Name | App(s) That Propose It | Cross-App Pattern | Missing Generic |
|------------------------|----------------------|-------------------|----------------|
| `direct_bookings`, `booking_items` | Direct Booking Engine, Flight + Hotel Package Builder | Time-bounded reservation with line items, hold, and status lifecycle | **G23 `atlas_reservations`** |
| `guest_reservations` | Multilingual Guest Comms, Direct Booking Engine | PMS-synced reservation record tied to a ledger entry | **G23 `atlas_reservations`** |
| `room_rates`, `rate_recommendations`, `rate_pushes`, `tenant_room_inventories` | Event-Aware Revenue Manager, Direct Booking Engine, PM (STR) | Availability calendar + rate per slot per asset | **G26 `atlas_catalog`** |
| `price_locks`, `booking_items`, `commission_ledger` | Flight + Hotel Package Builder, Direct Booking Engine | Quote with line items + time-bounded price hold | **G24 `atlas_quotes`** |
| `risk_submissions`, `rating_tables` | CoverFlow, AgentLink (insurance) | Structured intake with calculated financial output (premium, commission) | **G24 `atlas_quotes`** |
| `campaigns`, `campaign_submissions`, `metric_history` | Clipping Marketplace, Growth Platform, PM, AgentLink | Multi-channel campaign with enrollments and events | **G19 `atlas_campaigns`** |
| `miami_events`, `competitor_sets`, `comp_rate_logs` | Revenue Manager | Event-aware pricing inputs — the events themselves are a generic | **G21 `atlas_events`** |
| `comms_tenants`, `guest_profiles`, `message_logs` | Multilingual Guest Comms | Guest contact profiles tied to reservations — these are atlas_accounts with a `check_in`/`check_out` lifecycle | **G23 `atlas_reservations`** (+ existing atlas_accounts) |
| `agency_seats`, `commission_ledger` | Flight + Hotel Package Builder, AgentLink, CoverFlow | Agent/broker who earns a commission split; the plan governing that split | **G25 `atlas_commission_plans`** |
| `social_channels`, `clip_submissions` | Clipping Marketplace, Famtasm | Social OAuth connection + media submission workflow tied to a campaign | G19 `atlas_campaigns` + G05 (integration) |
| `attribute_score_history`, `member_entity_associations` | Nomad List | Time-series rating log — this is an attribution touchpoint pattern | G20 `atlas_attribution` (partial) |
| `work_tasks`, `recipient_staff` | Multilingual Guest Comms | Internal task assignment — this is G13 `atlas_cases` with `case_type = 'task'` | Rule 7 enforcement gap, not a new generic |

---

## Section 3: Missing Generic Candidates — Scored

### G-19: `atlas_campaigns` — Multi-Channel Campaign Management

**One-line description:** A campaign is a coordinated outreach effort across one or more channels (email, SMS, PPC, events, referrals) aimed at driving a specific conversion — enrollment, booking, application, sale, or registration.

**Salesforce analog:** `Campaign` + `CampaignMember` objects. Every enterprise CRM has this. Salesforce's Campaign is one of its oldest core objects (since 2001).

**Lens scores:**
- **Lens 1 (Rule 7 cross-app divergence):** STRONG PASS — PM needs open-house campaigns, AgentLink needs agent recruiting campaigns, Clipping needs brand campaigns, Direct Booking needs hotel marketing campaigns. If each builds independently, the schema of what a "campaign" is will diverge immediately (different goal fields, different metric tracking), making cross-tenant reporting impossible.
- **Lens 2 (Enterprise platform precedent):** STRONG PASS — Salesforce Campaign, HubSpot Campaigns, Marketo Programs. Every enterprise marketing platform has this as a first-class object.
- **Lens 3 (JSONB abuse detected):** STRONG PASS — Clipping Marketplace has a `campaigns` table already planned app-specifically. Growth Platform spec explicitly documents the gap with full DDL. Without this generic, Clipping builds it first and Direct Booking reinvents it 6 months later.
- **Lens 4 (Who builds it first / debt timeline):** **Immediate** — Clipping Marketplace is active development; Growth Platform document was written May 2026. Both apps will build conflicting campaign tables within weeks of each other without this generic.
- **Lens 5 (Commerce chain completeness):** Fills the **Awareness / Attribution** step — the part of the chain that drives prospects into the Browse step. Without it, "which campaign drove this booking?" is unanswerable.

**Score: 9.5/10** — Maximum cross-app impact (7 apps benefit immediately), most complete existing spec (Growth Platform doc already provides production DDL), and the debt timeline is already here.

**Apps that benefit immediately:** Clipping Marketplace, PM, AgentLink, Nomad List, Direct Booking Engine, ClaimSwift, Famtasm

**Proposed tables (from Growth Platform spec — already finalized):**
```sql
-- Campaign definition
CREATE TABLE atlas_campaigns (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    campaign_type atlas_campaign_type NOT NULL,  -- ENUM: cold_email, ppc, social, event_based, sms, referral
    status atlas_campaign_status NOT NULL DEFAULT 'draft',
    goal_type VARCHAR(50),         -- 'lead_capture', 'booking', 'application', 'sale', 'registration'
    goal_entity_type VARCHAR(50),  -- what a conversion creates: 'atlas_applications', 'atlas_event_registrations'
    budget_cents BIGINT,
    integration_id UUID REFERENCES atlas_external_integrations(id), -- Instantly, Google Ads, etc.
    subject_entity_type VARCHAR(50),  -- what this campaign is FOR
    subject_entity_id UUID,
    starts_at TIMESTAMPTZ,
    ends_at TIMESTAMPTZ,
    utm_source VARCHAR(100),
    utm_medium VARCHAR(100),
    utm_campaign VARCHAR(100),
    total_contacts INT DEFAULT 0,
    total_conversions INT DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Sequence steps
CREATE TABLE atlas_sequence_steps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    campaign_id UUID NOT NULL REFERENCES atlas_campaigns(id) ON DELETE CASCADE,
    step_number INT NOT NULL,
    step_type VARCHAR(30) NOT NULL,  -- 'email', 'sms', 'wait', 'condition', 'linkedin'
    subject_template TEXT,
    body_template TEXT,
    wait_days INT,
    condition_type VARCHAR(50),
    exit_on_reply BOOLEAN DEFAULT TRUE,
    UNIQUE(campaign_id, step_number)
);

-- Contact enrollment in a campaign
CREATE TABLE atlas_campaign_enrollments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    campaign_id UUID NOT NULL REFERENCES atlas_campaigns(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
    contact_email VARCHAR(255),
    contact_name VARCHAR(200),
    status atlas_enrollment_status NOT NULL DEFAULT 'active',
    current_step INT DEFAULT 1,
    exit_reason VARCHAR(50),
    converted_at TIMESTAMPTZ,
    conversion_entity_type VARCHAR(50),
    conversion_entity_id UUID,
    external_enrollment_id VARCHAR(255),  -- Instantly lead ID, etc.
    enrolled_at TIMESTAMPTZ DEFAULT NOW(),
    next_step_at TIMESTAMPTZ
);

-- Per-enrollment events (opens, clicks, replies, bounces)
CREATE TABLE atlas_campaign_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    enrollment_id UUID NOT NULL REFERENCES atlas_campaign_enrollments(id) ON DELETE CASCADE,
    campaign_id UUID NOT NULL REFERENCES atlas_campaigns(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL,  -- 'sent', 'opened', 'clicked', 'replied', 'bounced', 'converted'
    channel VARCHAR(30) NOT NULL,     -- 'email', 'sms', 'ppc_click', 'social', 'event'
    metadata JSONB,
    occurred_at TIMESTAMPTZ DEFAULT NOW()
);
```

**Rust service signature:**
```rust
pub struct CampaignService;

impl CampaignService {
    pub async fn create(db: &DatabaseConnection, tenant_id: Uuid, payload: CreateCampaignPayload) -> Result<AtlasCampaign>;
    pub async fn get(db: &DatabaseConnection, id: Uuid, tenant_id: Uuid) -> Result<AtlasCampaign>;
    pub async fn list(db: &DatabaseConnection, filter: CampaignFilter) -> Result<Vec<AtlasCampaign>>;
    pub async fn enroll(db: &DatabaseConnection, campaign_id: Uuid, contact: CampaignContact) -> Result<AtlasCampaignEnrollment>;
    pub async fn record_event(db: &DatabaseConnection, enrollment_id: Uuid, event_type: &str, channel: &str) -> Result<AtlasCampaignEvent>;
    pub async fn find_by_subject(db: &DatabaseConnection, tenant_id: Uuid, entity_type: &str, entity_id: Uuid) -> Result<Vec<AtlasCampaign>>;
}
```

**Recommendation: PROMOTE NOW**

**Justification:** Clipping Marketplace and Growth Platform are both in active development and will create conflicting `campaigns` tables immediately. The Growth Platform document has already done the full DDL design work — this is a near-zero-effort promotion. Waiting even one sprint risks a permanent cross-app schema divergence.

---

### G-20: `atlas_attribution` — Multi-Channel Attribution

**One-line description:** Captures every marketing touchpoint (UTM click, email open, event attendance, referral) and attributes conversion revenue back to the channels that influenced it — answering "which ad/campaign/event drove this booking?"

**Salesforce analog:** Salesforce doesn't have a native Attribution object — this is the gap that products like Dreamdata and Bizible (now Marketo Measure, $30K+/year) fill. Its absence from Salesforce is why the market exists.

**Lens scores:**
- **Lens 1 (Rule 7 cross-app divergence):** STRONG PASS — Every app that runs paid acquisition needs this. If Direct Booking tracks Google Ad conversions in a private table while PM tracks open-house ad conversions in a different private table, the CMO of a multi-vertical operator can never answer "what is our total cost per booking across all products?" Cross-app reporting is broken.
- **Lens 2 (Enterprise platform precedent):** STRONG PASS — Entire $5B martech industry exists because platforms don't have this. Bizible, Dreamdata, Triple Whale all exist to fill this gap in Salesforce/HubSpot.
- **Lens 3 (JSONB abuse detected):** PASSES — Without this table, UTM parameters from PPC clicks will be stored in `atlas_opportunities.financial_inputs` or dropped entirely. Nomad List's `attribute_score_history` is a crude version of this for ratings data.
- **Lens 4 (Who builds it first / debt timeline):** **Near-term** — The Direct Booking Engine and Growth Platform are the first apps that will run paid acquisition. Once they each build their own UTM capture tables, stitching them is a multi-quarter project.
- **Lens 5 (Commerce chain completeness):** Fills the **Attribution step** — connecting every touchpoint in the awareness phase back to a confirmed conversion. This is the analytics layer that tells operators their true CAC.

**Score: 9/10** — Universal cross-app benefit (every app that acquires users needs this), high Lens 2 precedent, and it is a dependency for G19 (campaigns reference attribution touchpoints).

**Apps that benefit immediately:** Direct Booking Engine, Growth Platform, PM, AgentLink, Nomad List, Famtasm, Clipping Marketplace

**Proposed tables:**
```sql
CREATE TABLE atlas_attribution_touchpoints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
    -- Who had this touchpoint (one of):
    user_id UUID REFERENCES "user"(id),
    contact_email VARCHAR(255),
    anonymous_id VARCHAR(128),  -- client-side cookie/fingerprint, identity-resolved later
    -- Channel:
    channel VARCHAR(50) NOT NULL,   -- 'organic_search', 'paid_search', 'paid_social', 'cold_email', 'event', 'referral', 'direct'
    -- UTM params:
    utm_source VARCHAR(100),
    utm_medium VARCHAR(100),
    utm_campaign VARCHAR(100),
    utm_content VARCHAR(100),
    utm_term VARCHAR(100),
    -- Referencing platform entities:
    campaign_id UUID REFERENCES atlas_campaigns(id),
    enrollment_id UUID REFERENCES atlas_campaign_enrollments(id),
    event_id UUID,  -- FK to atlas_events (G21)
    -- Conversion (set on conversion event):
    conversion_entity_type VARCHAR(50),
    conversion_entity_id UUID,
    conversion_value_cents BIGINT,
    attributed_revenue_cents BIGINT,
    attribution_model VARCHAR(30) DEFAULT 'last_touch',  -- 'first_touch', 'linear', 'time_decay'
    -- Context:
    landing_page_url TEXT,
    referrer_url TEXT,
    occurred_at TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX atlas_attribution_user ON atlas_attribution_touchpoints(tenant_id, user_id, occurred_at DESC);
CREATE INDEX atlas_attribution_anon ON atlas_attribution_touchpoints(tenant_id, anonymous_id, occurred_at DESC);
CREATE INDEX atlas_attribution_conversion ON atlas_attribution_touchpoints(tenant_id, conversion_entity_type, conversion_entity_id);
```

**Rust service signature:**
```rust
pub struct AttributionService;

impl AttributionService {
    pub async fn capture_touchpoint(db: &DatabaseConnection, tenant_id: Uuid, params: &UtmParams, session: Option<&UserSession>, anonymous_id: &str) -> Result<Uuid>;
    pub async fn resolve_identity(db: &DatabaseConnection, tenant_id: Uuid, anonymous_id: &str, user_id: Uuid) -> Result<u64>;
    pub async fn record_conversion(db: &DatabaseConnection, tenant_id: Uuid, user_id_or_email: Either<Uuid, &str>, conversion_entity_type: &str, conversion_entity_id: Uuid, conversion_value_cents: i64, model: AttributionModel) -> Result<Vec<Uuid>>;
    pub async fn get_conversion_path(db: &DatabaseConnection, tenant_id: Uuid, conversion_entity_id: Uuid) -> Result<Vec<AtlasAttributionTouchpoint>>;
}
```

**Recommendation: PROMOTE NOW (depends on G19)**

**Justification:** G20 references G19 (campaigns) and G21 (events). It must be built after those two. However, the DDL design is complete (Growth Platform document), and the earliest app to run paid acquisition (Direct Booking, Growth Platform) needs it immediately. Deprioritizing attribution means operators lose the ability to measure CAC across channels — one of the platform's key sales promises.

---

### G-21: `atlas_events` — Event Management, Ticketing & Check-In

**One-line description:** A public or private event with capacity, ticket types, registrations, QR check-in, and post-event attribution — covering open houses, webinars, CE seminars, creator meet-and-greets, hotel conference bookings, and brand activations.

**Salesforce analog:** Salesforce doesn't have a native Events object (Campaign can model it loosely). Event management is handled by AppExchange tools like Fonteva Events ($20K+/yr) or Cvent. The absence is why Eventbrite ($500M+ revenue) exists.

**Lens scores:**
- **Lens 1 (Rule 7 cross-app divergence):** STRONG PASS — PM runs open houses, AgentLink runs CE seminars, Nomad List runs city meetups, Famtasm runs creator live events, Direct Booking runs hotel conference facilities. Each is structurally identical: a scheduled gathering with registrants and capacity. If each app builds its own events table, a single operator running PM + AgentLink + Nomad List can never see a unified event calendar.
- **Lens 2 (Enterprise platform precedent):** STRONG PASS — Eventbrite, Luma, Splash, Hopin. The entire event-tech category exists because this primitive is universally needed and universally missing from vertical software.
- **Lens 3 (JSONB abuse detected):** PASSES — Revenue Manager's `miami_events` table is already an app-specific events table. Without G21, Revenue Manager builds it app-specifically, and then PM builds a different one for open houses, and they can never be joined.
- **Lens 4 (Who builds it first / debt timeline):** **Near-term** — Revenue Manager's `miami_events` is already in the spec. PM will need open-house events during its active development phase.
- **Lens 5 (Commerce chain completeness):** Fills the **Browse + Quote** step — events generate registrations which generate revenue (ticket sales through G03), and feed back into G20 attribution.

**Score: 9/10** — 7 apps benefit immediately, complete DDL exists in Growth Platform doc, and the Revenue Manager is already building a private version.

**Apps that benefit immediately:** PM, AgentLink, Nomad List, Direct Booking Engine, ClaimSwift (training), Famtasm, Clipping Marketplace

**Proposed tables (from Growth Platform spec):**
```sql
CREATE TABLE atlas_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255),
    event_type VARCHAR(50) NOT NULL,  -- 'open_house', 'webinar', 'conference', 'meetup', 'training', 'live_experience', 'brand_activation'
    status VARCHAR(30) NOT NULL DEFAULT 'draft',
    is_virtual BOOLEAN DEFAULT FALSE,
    virtual_url VARCHAR(512),
    venue_name VARCHAR(255),
    venue_address TEXT,
    venue_geo_point GEOGRAPHY(Point, 4326),   -- uses G01
    venue_asset_id UUID,                       -- FK to atlas_assets if at a managed property
    max_capacity INT,
    waitlist_enabled BOOLEAN DEFAULT TRUE,
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ NOT NULL,
    registration_opens_at TIMESTAMPTZ,
    registration_closes_at TIMESTAMPTZ,
    campaign_id UUID REFERENCES atlas_campaigns(id),  -- which campaign promoted this event
    subject_entity_type VARCHAR(50),   -- 'atlas_asset', 'atlas_opportunities'
    subject_entity_id UUID,
    is_public BOOLEAN DEFAULT TRUE,
    registered_count INT DEFAULT 0,
    attended_count INT DEFAULT 0,
    revenue_cents BIGINT DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE(tenant_id, slug)
);

CREATE TABLE atlas_event_ticket_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL REFERENCES atlas_events(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    price_cents BIGINT NOT NULL DEFAULT 0,
    currency CHAR(3) DEFAULT 'USD',
    quantity_available INT,
    quantity_sold INT DEFAULT 0,
    is_active BOOLEAN DEFAULT TRUE
);

CREATE TABLE atlas_event_registrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id UUID NOT NULL REFERENCES atlas_events(id) ON DELETE CASCADE,
    ticket_type_id UUID NOT NULL REFERENCES atlas_event_ticket_types(id),
    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
    attendee_email VARCHAR(255) NOT NULL,
    attendee_name VARCHAR(200),
    quantity INT NOT NULL DEFAULT 1,
    ledger_entry_id UUID REFERENCES atlas_ledger_entries(id),   -- G03
    check_in_token VARCHAR(128) NOT NULL UNIQUE DEFAULT encode(gen_random_bytes(32), 'hex'),
    status VARCHAR(30) NOT NULL DEFAULT 'pending_payment',   -- 'confirmed', 'waitlisted', 'cancelled', 'checked_in', 'no_show'
    confirmed_at TIMESTAMPTZ,
    checked_in_at TIMESTAMPTZ,
    attribution_touchpoint_id UUID,   -- G20
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

**Rust service signature:**
```rust
pub struct EventService;

impl EventService {
    pub async fn create(db: &DatabaseConnection, tenant_id: Uuid, payload: CreateEventPayload) -> Result<AtlasEvent>;
    pub async fn get(db: &DatabaseConnection, id: Uuid, tenant_id: Uuid) -> Result<AtlasEvent>;
    pub async fn list(db: &DatabaseConnection, filter: EventFilter) -> Result<Vec<AtlasEvent>>;
    pub async fn register(db: &DatabaseConnection, event_id: Uuid, attendee: RegistrationPayload) -> Result<AtlasEventRegistration>;
    pub async fn check_in(db: &DatabaseConnection, check_in_token: &str) -> Result<AtlasEventRegistration>;
    pub async fn find_by_subject(db: &DatabaseConnection, tenant_id: Uuid, entity_type: &str, entity_id: Uuid) -> Result<Vec<AtlasEvent>>;
}
```

**Recommendation: PROMOTE NOW (depends on G19 for campaign linkage; can build concurrently)**

**Justification:** The Revenue Manager already plans to create `miami_events` as an app-specific table. That table needs to be G21 instead. PM will need open-house events within its active development timeline. This is near-zero-risk to promote given the complete DDL exists.

---

### G-22: `atlas_record_relationships` — Junction Table / Salesforce Related Lists

**One-line description:** A universal M:M junction table that connects any two generic records with a labeled relationship type — enabling Salesforce-style Related Lists UI and cross-entity M:M reporting without per-combination join tables.

**Salesforce analog:** Salesforce Junction Object pattern. Used to implement M:M between standard and custom objects. The Related Lists panel in every Salesforce record view depends on this.

**Lens scores:**
- **Lens 1 (Rule 7 cross-app divergence):** PASSES — Without this, each app builds its own junction tables: `campaign_assets`, `event_service_providers`, `case_contracts`. They diverge immediately and are unqueryable uniformly.
- **Lens 2 (Enterprise platform precedent):** STRONG PASS — Salesforce Junction Object is a fundamental architectural primitive. Every CRM platform has some form of this.
- **Lens 3 (JSONB abuse detected):** PASSES — Apps will store array-of-IDs in JSONB on existing generics rather than build junction tables: `campaign.asset_ids JSONB` on `atlas_campaigns`.
- **Lens 4 (Who builds it first / debt timeline):** **Near-term** — The first app to need a campaign-to-multiple-assets relationship will build it privately.
- **Lens 5 (Commerce chain completeness):** Infrastructure primitive — enables reporting across the commerce chain by connecting entities that don't have explicit FKs.

**Score: 7.5/10** — High architectural value but lower urgency than G23-G25 because the implicit polymorphic pattern (already present in G01-G18) handles 90% of use cases. This is the connective tissue, not a blocker.

**Apps that benefit immediately:** All apps (infrastructure primitive)

**Proposed tables:**
```sql
CREATE TABLE atlas_record_relationships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
    source_entity_type VARCHAR(50) NOT NULL,
    source_entity_id UUID NOT NULL,
    target_entity_type VARCHAR(50) NOT NULL,
    target_entity_id UUID NOT NULL,
    relationship_type VARCHAR(100) NOT NULL,
    -- e.g. 'featured_in_campaign', 'attended_by', 'referenced_in_case', 'generated_from'
    relationship_metadata JSONB,
    inverse_label VARCHAR(100),
    created_by_user_id UUID REFERENCES "user"(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(tenant_id, source_entity_type, source_entity_id, target_entity_type, target_entity_id, relationship_type)
);
CREATE INDEX atlas_record_rel_source ON atlas_record_relationships(tenant_id, source_entity_type, source_entity_id, relationship_type);
CREATE INDEX atlas_record_rel_target ON atlas_record_relationships(tenant_id, target_entity_type, target_entity_id, relationship_type);
```

**Rust service signature:**
```rust
pub struct RelatedListService;

impl RelatedListService {
    pub async fn create_relationship(db: &DatabaseConnection, tenant_id: Uuid, source: EntityRef, target: EntityRef, relationship_type: &str, metadata: Option<serde_json::Value>) -> Result<AtlasRecordRelationship>;
    pub async fn get_related_records(db: &DatabaseConnection, tenant_id: Uuid, entity_type: &str, entity_id: Uuid) -> Result<HashMap<String, Vec<RelatedListItem>>>;
    pub async fn delete_relationship(db: &DatabaseConnection, id: Uuid, tenant_id: Uuid) -> Result<()>;
}
```

**Recommendation: PROMOTE NOW (after G19/G21; zero DDL complexity)**

**Justification:** Tiny schema, huge architectural leverage. Once G19 (campaigns) and G21 (events) exist, apps immediately need M:M connections like campaign ↔ multiple assets, event ↔ multiple service providers. This prevents a proliferation of private junction tables.

---

### G-23: `atlas_reservations` — Time-Bounded Reservation with Inventory Hold

**One-line description:** A confirmed or pending booking of an asset (room, seat, time slot, flight) for a specific time window, with external hold IDs, slot conflict detection, and a status lifecycle from `hold` → `confirmed` → `completed` / `cancelled`.

**Salesforce analog:** There is no native Salesforce object for this. Field Service Lightning (FSL) `ServiceAppointment` + `WorkOrder` come closest. The absence of a native Reservation object is why VerticalResponse, Cloudbeds, Mews, Opera PMS, and Duffel all exist — they fill this exact gap.

**Lens scores:**
- **Lens 1 (Rule 7 cross-app divergence):** STRONG PASS — This is the most dangerous divergence risk on the platform. Direct Booking needs `direct_bookings`, Flight + Hotel needs `package_bookings` + `booking_items`, Guest Comms needs `guest_reservations`, Revenue Manager needs `tenant_room_inventories`. These are all structurally the same object: a time-bounded claim on an asset. If four apps build four private reservation tables, `atlas_ledger_entries` will have four different `billable_entity_type` values for the same concept, making financial cross-reporting impossible without a UNION.
- **Lens 2 (Enterprise platform precedent):** STRONG PASS — Booking.com's `reservation` object, Airbnb's reservation, Cloudbeds PMS's reservation, Duffel's `order` object. Every property management, hotel, and travel system has this as its central domain object.
- **Lens 3 (JSONB abuse detected):** STRONG PASS — Explicitly documented: `atlas_cases.case_metadata` in both Direct Booking and Revenue Manager integration specs will store check-in/check-out dates and room IDs before a proper reservation table exists.
- **Lens 4 (Who builds it first / debt timeline):** **Immediate** — Direct Booking Engine and Flight + Hotel Package Builder are both in active development. The Phase 1 migration script for Direct Booking Engine explicitly includes `direct_bookings` and `booking_items`. This debt clock has already started.
- **Lens 5 (Commerce chain completeness):** Fills the **Commit step** — the single most critical gap in the commerce chain. Without a reservation primitive, the platform has no canonical representation of a "confirmed purchase with a time slot." The ledger records the payment (G03), but there is no entity to FK to.

**Score: 10/10** — The highest-scoring missing generic on the platform. Immediate debt timeline, strongest cross-app divergence risk, largest commerce chain gap, Lens 2 universal precedent. This is the most important thing to build.

**Apps that benefit immediately:** Direct Booking Engine, Flight + Hotel Package Builder, Multilingual Guest Comms, Event-Aware Revenue Manager, PM (STR mode), Property Management (any time-based booking)

**Proposed tables:**
```sql
CREATE TYPE atlas_reservation_status AS ENUM (
    'hold',           -- inventory locked, payment not confirmed
    'pending_payment',-- hold confirmed, awaiting payment
    'confirmed',      -- payment confirmed
    'checked_in',     -- guest/attendee arrived
    'completed',      -- stay/service completed
    'cancelled',      -- cancelled by guest or operator
    'no_show'         -- confirmed but never checked in
);

CREATE TABLE atlas_reservations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
    -- What is reserved (polymorphic — the asset being booked):
    reserved_asset_type VARCHAR(50) NOT NULL,
    -- 'atlas_asset' (room, unit, seat), 'atlas_event_ticket_type', 'atlas_service_slot'
    reserved_asset_id UUID NOT NULL,
    -- Who is reserving:
    guest_account_id UUID REFERENCES atlas_accounts(id),
    guest_email VARCHAR(255),
    -- Time bounds:
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ NOT NULL,
    -- Reservation type:
    reservation_type VARCHAR(50) NOT NULL,
    -- 'hotel_room', 'str_unit', 'flight_seat', 'package', 'service_appointment', 'event_slot'
    -- Line items (for multi-item reservations like packages):
    reservation_metadata JSONB NOT NULL DEFAULT '{}',
    -- {night_count, room_type, pax_count, meal_plan, add_ons[], flight_segments[], package_items[]}
    -- External system references:
    external_hold_id VARCHAR(255),    -- GDS/Duffel hold ID, PMS folio number
    external_confirmation_no VARCHAR(255),
    pms_integration_id UUID REFERENCES atlas_external_integrations(id),
    -- Status lifecycle:
    status atlas_reservation_status NOT NULL DEFAULT 'hold',
    hold_expires_at TIMESTAMPTZ,     -- auto-release if payment not received
    confirmed_at TIMESTAMPTZ,
    checked_in_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ,
    cancellation_reason TEXT,
    -- Financial link:
    ledger_entry_id UUID REFERENCES atlas_ledger_entries(id),  -- G03
    quote_id UUID,                    -- FK to atlas_quotes (G24) — the proposal that became this booking
    total_amount_cents BIGINT,
    currency CHAR(3) DEFAULT 'USD',
    -- Attribution:
    campaign_enrollment_id UUID,      -- G19
    attribution_touchpoint_id UUID,   -- G20
    -- Timestamps:
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX atlas_reservations_asset ON atlas_reservations(tenant_id, reserved_asset_type, reserved_asset_id, starts_at, ends_at);
CREATE INDEX atlas_reservations_guest ON atlas_reservations(tenant_id, guest_account_id, status);
CREATE INDEX atlas_reservations_status ON atlas_reservations(tenant_id, status, hold_expires_at);
CREATE INDEX atlas_reservations_dates ON atlas_reservations(tenant_id, reservation_type, starts_at, ends_at);

-- Slot conflict detection function:
-- Query: SELECT id FROM atlas_reservations WHERE reserved_asset_id = $1 AND status NOT IN ('cancelled', 'no_show') AND tsrange(starts_at, ends_at) && tsrange($2, $3)
```

**Rust service signature:**
```rust
pub struct ReservationService;

impl ReservationService {
    pub async fn create_hold(db: &DatabaseConnection, tenant_id: Uuid, payload: CreateReservationPayload, hold_duration_minutes: i64) -> Result<AtlasReservation>;
    pub async fn confirm(db: &DatabaseConnection, id: Uuid, tenant_id: Uuid, ledger_entry_id: Uuid) -> Result<AtlasReservation>;
    pub async fn check_in(db: &DatabaseConnection, id: Uuid, tenant_id: Uuid) -> Result<AtlasReservation>;
    pub async fn cancel(db: &DatabaseConnection, id: Uuid, tenant_id: Uuid, reason: &str) -> Result<AtlasReservation>;
    pub async fn check_availability(db: &DatabaseConnection, tenant_id: Uuid, asset_id: Uuid, starts_at: DateTime<Utc>, ends_at: DateTime<Utc>) -> Result<bool>;
    pub async fn release_expired_holds(db: &DatabaseConnection) -> Result<u64>;  -- background worker
    pub async fn get(db: &DatabaseConnection, id: Uuid, tenant_id: Uuid) -> Result<AtlasReservation>;
    pub async fn list(db: &DatabaseConnection, filter: ReservationFilter) -> Result<Vec<AtlasReservation>>;
}
```

**Recommendation: PROMOTE IMMEDIATELY — this is the top priority**

**Justification:** Direct Booking Engine's Phase 1 migration script already names `direct_bookings` and `booking_items`. If this generic is not promoted before Direct Booking begins development, two app-specific reservation tables will exist within weeks. The `atlas_ledger_entries` table's `billable_entity_type` field will have `'direct_booking'` and `'package_booking'` as separate types for the same concept — a data model catastrophe that will require a production migration to fix. The `release_expired_holds()` background worker must be registered as a platform-level BackgroundJob from day one.

---

### G-24: `atlas_quotes` — Pre-Purchase Pricing Proposal with Line Items

**One-line description:** A structured pricing proposal sent to a prospect before commitment — with one or more line items, a calculated total, a validity window, and a status lifecycle from `draft` → `sent` → `accepted` / `rejected` / `expired`. When accepted, it becomes a Reservation or Contract.

**Salesforce analog:** `Quote` + `QuoteLineItem` objects. Native Salesforce since API v16. Revenue Cloud quote-to-order flow.

**Lens scores:**
- **Lens 1 (Rule 7 cross-app divergence):** STRONG PASS — CoverFlow needs risk_submissions with calculated premiums, Direct Booking needs corporate negotiated rate proposals with room block line items, Flight + Hotel needs multi-component package price proposals. Three identical "here is the priced breakdown before you commit" use cases, three different private schemas.
- **Lens 2 (Enterprise platform precedent):** STRONG PASS — Salesforce Quote/QuoteLineItem, HubSpot Quotes, Stripe Payment Links, PandaDoc. Every B2B commerce platform has a Quote object.
- **Lens 3 (JSONB abuse detected):** STRONG PASS — CoverFlow `risk_submissions` will store the calculated premium, limits, and deductible in JSONB. Direct Booking `corporate_contracts` will store block room pricing in JSONB. Both are quotes.
- **Lens 4 (Who builds it first / debt timeline):** **Near-term** — CoverFlow's rating engine is Gap 2 in its integration mapping. Direct Booking's `corp_contracts.rs` is Gap 4. Both will build private quote tables.
- **Lens 5 (Commerce chain completeness):** Fills the **Quote step** — the explicit gap identified in the commerce chain analysis. `atlas_opportunities` (G15) represents the pipeline stage; `atlas_quotes` represents the formal priced proposal that converts that opportunity into a reservation or contract.

**Score: 8.5/10** — Completes the commerce chain, resolves a documented G15 inadequacy, and covers 4+ high-value apps.

**Apps that benefit immediately:** Direct Booking Engine, CoverFlow, Flight + Hotel Package Builder, AgentLink (producer recruitment packages), Clipping Marketplace (campaign pricing proposals)

**Proposed tables:**
```sql
CREATE TYPE atlas_quote_status AS ENUM (
    'draft', 'sent', 'viewed', 'accepted', 'rejected', 'expired', 'converted'
);

CREATE TABLE atlas_quotes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
    quote_type VARCHAR(50) NOT NULL,
    -- 'hotel_room_block', 'insurance_premium', 'travel_package', 'service_agreement', 'campaign_proposal'
    status atlas_quote_status NOT NULL DEFAULT 'draft',
    -- Who this quote is for:
    account_id UUID REFERENCES atlas_accounts(id),
    contact_email VARCHAR(255),
    -- What this quote is based on:
    opportunity_id UUID REFERENCES atlas_opportunities(id),   -- G15 — the pipeline stage
    -- Financial summary (computed from line items):
    subtotal_cents BIGINT NOT NULL DEFAULT 0,
    discount_cents BIGINT NOT NULL DEFAULT 0,
    tax_cents BIGINT NOT NULL DEFAULT 0,
    total_cents BIGINT NOT NULL DEFAULT 0,
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    -- Validity:
    valid_until TIMESTAMPTZ,
    -- Conversion:
    accepted_at TIMESTAMPTZ,
    reservation_id UUID,   -- FK to atlas_reservations (G23) when accepted
    contract_id UUID REFERENCES atlas_contracts(id),  -- G11 when accepted for service agreements
    -- Quote metadata (app-specific rating inputs, etc.):
    quote_metadata JSONB NOT NULL DEFAULT '{}',
    -- e.g. CoverFlow: {base_premium, revenue_mult, loss_mult, limits, deductible}
    -- e.g. Direct Booking: {cutoff_date, release_policy, corporate_rate_code}
    created_by_user_id UUID REFERENCES "user"(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE atlas_quote_line_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    quote_id UUID NOT NULL REFERENCES atlas_quotes(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    -- What is being priced:
    item_type VARCHAR(50) NOT NULL,  -- 'room_night', 'flight_segment', 'insurance_coverage', 'service_fee', 'add_on'
    item_description VARCHAR(500),
    catalog_entry_id UUID,  -- FK to atlas_catalog (G26) when items come from a product catalog
    -- Pricing:
    quantity NUMERIC(10, 4) NOT NULL DEFAULT 1,
    unit_price_cents BIGINT NOT NULL,
    discount_pct NUMERIC(5, 2) DEFAULT 0,
    line_total_cents BIGINT NOT NULL,
    -- Dates (for time-based items):
    service_starts_at TIMESTAMPTZ,
    service_ends_at TIMESTAMPTZ,
    -- Metadata:
    line_metadata JSONB NOT NULL DEFAULT '{}',
    UNIQUE(quote_id, line_number)
);
```

**Rust service signature:**
```rust
pub struct QuoteService;

impl QuoteService {
    pub async fn create(db: &DatabaseConnection, tenant_id: Uuid, payload: CreateQuotePayload) -> Result<AtlasQuote>;
    pub async fn add_line_item(db: &DatabaseConnection, quote_id: Uuid, item: QuoteLineItemPayload) -> Result<AtlasQuoteLineItem>;
    pub async fn recalculate_totals(db: &DatabaseConnection, quote_id: Uuid) -> Result<AtlasQuote>;
    pub async fn send(db: &DatabaseConnection, id: Uuid, tenant_id: Uuid) -> Result<AtlasQuote>;
    pub async fn accept(db: &DatabaseConnection, id: Uuid, tenant_id: Uuid) -> Result<AtlasQuote>;
    pub async fn convert_to_reservation(db: &DatabaseConnection, quote_id: Uuid, tenant_id: Uuid) -> Result<AtlasReservation>;
    pub async fn get(db: &DatabaseConnection, id: Uuid, tenant_id: Uuid) -> Result<AtlasQuote>;
    pub async fn list(db: &DatabaseConnection, filter: QuoteFilter) -> Result<Vec<AtlasQuote>>;
}
```

**Recommendation: PROMOTE BEFORE DIRECT BOOKING AND COVERFLOW START**

**Justification:** The commerce chain is incomplete without Quote + Line Items between Opportunity (G15) and Reservation (G23). CoverFlow's rating engine and Direct Booking's corporate contracts both represent quotes — they will build incompatible private schemas. The dependency chain is: G15 (Opportunity) → G24 (Quote) → G23 (Reservation) → G03 (Payment).

---

### G-25: `atlas_commission_plans` — Commission Plan & Split Governance

**One-line description:** A reusable plan that defines how revenue is split among multiple recipients (platform, broker, agent, carrier, MGA, co-broker) — including tiered rates, overrides, clawback rules, and caps — governing the behavior of `atlas_ledger_splits` (G03).

**Salesforce analog:** Revenue Cloud `CommissionSchedule` + `SalesAgreement`. Financial Services Cloud has commission tracking as a first-class feature.

**Lens scores:**
- **Lens 1 (Rule 7 cross-app divergence):** STRONG PASS — AgentLink (lead referral commissions), CoverFlow (carrier/MGA/broker premium splits), Clipping Marketplace (CPM payout rates), Direct Booking (travel agent commissions), PM (property management fee splits), Commercial Loan Broker, Insurance Brokerage, Freight Dispatch (all from sales analyses). Every vertical has a commission structure. Every app will encode their commission logic differently — some in `terms_metadata` on `atlas_contracts`, some in handler code, some in JSONB.
- **Lens 2 (Enterprise platform precedent):** STRONG PASS — SAP Commission Management, Salesforce Revenue Cloud, Xactly Incent ($500+/user/month). The commission management software category exists because this is universally needed and universally complex.
- **Lens 3 (JSONB abuse detected):** STRONG PASS — Currently, `atlas_ledger_splits` records the resulting split but has no reference to what plan governed it. AgentLink and CoverFlow integration specs both describe commission logic that will end up in `terms_metadata` on `atlas_contracts` or hardcoded in handlers. The sales analyses for insurance brokerage, loan broker, and freight dispatch all reference commission split complexity as a key platform benefit — and that benefit is undelivered without this generic.
- **Lens 4 (Who builds it first / debt timeline):** **Immediate** — CoverFlow's Stripe Connect split implementation (Gap 1 in its integration spec) hardcodes commission percentages in the handler. Without G25, the first app to implement commissions bakes the rate into Rust code instead of a configurable plan table.
- **Lens 5 (Commerce chain completeness):** Fills the **Commission step** — explicitly identified as a gap in the prompt's commerce chain table: "Commission: G03 `atlas_ledger_splits` — Partial — records the split, but no plan that governs it."

**Score: 9/10** — Resolves an explicitly identified commerce chain gap, highest frequency across sales prospects (every B2B vertical needs commission management), and immediate debt from CoverFlow's hardcoded split logic.

**Apps that benefit immediately:** CoverFlow, AgentLink, Direct Booking Engine, Clipping Marketplace, PM, Flight + Hotel Package Builder

**Proposed tables:**
```sql
CREATE TYPE atlas_commission_basis AS ENUM (
    'percentage',      -- X% of gross transaction
    'flat_per_unit',   -- $X per unit (room night, policy, load)
    'cpm',             -- per 1,000 views/impressions (Clipping)
    'tiered'           -- rate changes at volume thresholds
);

CREATE TABLE atlas_commission_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    plan_type VARCHAR(50) NOT NULL,
    -- 'broker_split', 'agent_override', 'carrier_remittance', 'platform_fee', 'cpm_payout', 'referral'
    is_active BOOLEAN DEFAULT TRUE,
    -- Default split structure:
    commission_basis atlas_commission_basis NOT NULL,
    default_rate NUMERIC(8, 4),     -- percentage (e.g. 15.00 for 15%) or flat amount in cents
    currency CHAR(3) DEFAULT 'USD',
    -- Tiered rates (used when commission_basis = 'tiered'):
    tiers JSONB,
    -- [{min_volume_cents: 0, max_volume_cents: 100000, rate: 10.00}, {min: 100001, rate: 12.00}]
    -- Caps and minimums:
    cap_cents BIGINT,               -- maximum commission per transaction
    minimum_cents BIGINT,           -- minimum commission per transaction
    -- Clawback:
    clawback_days INT,              -- if cancelled within N days, commission is reversed
    -- What entity this plan applies to:
    applies_to_entity_type VARCHAR(50),  -- 'atlas_service_providers', 'atlas_accounts'
    applies_to_entity_id UUID,
    -- What transaction type this plan governs:
    applies_to_ledger_type VARCHAR(50),  -- 'hotel_booking', 'insurance_policy', 'campaign_payout'
    -- Recipient:
    recipient_type VARCHAR(50),     -- 'platform', 'broker', 'agent', 'carrier', 'creator'
    recipient_account_id UUID REFERENCES atlas_accounts(id),
    -- Timestamps:
    effective_from DATE NOT NULL,
    effective_to DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX atlas_commission_plans_tenant ON atlas_commission_plans(tenant_id, plan_type, is_active);
CREATE INDEX atlas_commission_plans_entity ON atlas_commission_plans(tenant_id, applies_to_entity_type, applies_to_entity_id);

-- Commission plan line items (for complex multi-party splits):
CREATE TABLE atlas_commission_plan_splits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id UUID NOT NULL REFERENCES atlas_commission_plans(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
    recipient_type VARCHAR(50) NOT NULL,   -- 'platform', 'broker', 'carrier', 'mga'
    recipient_account_id UUID REFERENCES atlas_accounts(id),
    split_basis atlas_commission_basis NOT NULL,
    split_rate NUMERIC(8, 4) NOT NULL,
    priority INT DEFAULT 0,   -- order of calculation for cascading splits
    is_remainder BOOLEAN DEFAULT FALSE  -- TRUE = gets whatever is left after other splits
);
```

**Rust service signature:**
```rust
pub struct CommissionPlanService;

impl CommissionPlanService {
    pub async fn create(db: &DatabaseConnection, tenant_id: Uuid, payload: CreateCommissionPlanPayload) -> Result<AtlasCommissionPlan>;
    pub async fn calculate(db: &DatabaseConnection, plan_id: Uuid, transaction_amount_cents: i64, context: &CommissionContext) -> Result<Vec<CommissionSplit>>;
    pub async fn apply_to_ledger_entry(db: &DatabaseConnection, plan_id: Uuid, ledger_entry_id: Uuid) -> Result<Vec<AtlasLedgerSplit>>;
    pub async fn find_applicable_plans(db: &DatabaseConnection, tenant_id: Uuid, entity_type: &str, entity_id: Uuid, ledger_type: &str) -> Result<Vec<AtlasCommissionPlan>>;
    pub async fn process_clawback(db: &DatabaseConnection, ledger_entry_id: Uuid, cancellation_date: DateTime<Utc>) -> Result<Vec<AtlasLedgerSplit>>;
    pub async fn get(db: &DatabaseConnection, id: Uuid, tenant_id: Uuid) -> Result<AtlasCommissionPlan>;
}
```

**Recommendation: PROMOTE BEFORE COVERFLOW STARTS**

**Justification:** CoverFlow's integration spec describes hardcoding commission percentages directly in the Stripe Connect transfer call. That is a configuration embedded in code — the most expensive form of technical debt to change. G25 moves this into data. The insurance brokerage, commercial loan broker, and freight dispatch sales analyses all promise commission management as a key value proposition. Without G25, that promise is delivered via hardcoded handler logic in each app.

---

### G-26: `atlas_catalog` — Product Catalog, Pricebook & Availability Grid

**One-line description:** A structured catalog of what can be sold (room types, package tiers, service slots, coverage options, creator subscription tiers) with associated prices, availability windows, and rate rules — the "what is for sale and at what price?" primitive that sits between Assets (G10) and Quotes (G24).

**Salesforce analog:** `Product2` + `Pricebook2` + `PricebookEntry` objects. This is the oldest commerce primitive in Salesforce and arguably the most universally needed.

**Lens scores:**
- **Lens 1 (Rule 7 cross-app divergence):** STRONG PASS — Direct Booking needs `hotel_room_types` + `room_rates`. Revenue Manager needs `tenant_room_inventories` + `rate_recommendations`. PM (STR) needs availability windows and nightly rates. Famtasm needs creator subscription tiers (Free, Standard, Premium). These are all product catalog entries with associated prices. Each app will build its own schema.
- **Lens 2 (Enterprise platform precedent):** STRONG PASS — Salesforce Product2/Pricebook2, Shopify Product/Variant/Price, Stripe Price object, Cloudbeds Room Types. Every commerce platform has a product catalog. Its absence from Atlas is the single largest gap relative to any enterprise commerce platform.
- **Lens 3 (JSONB abuse detected):** STRONG PASS — Revenue Manager's `tenant_room_inventories` and `room_rates` in its integration spec are a private pricebook. `atlas_assets.attributes` will store `rate_calendar`, `min_stay`, and `availability_windows` as JSONB abuse.
- **Lens 4 (Who builds it first / debt timeline):** **Immediate** — Direct Booking Engine Phase 1 includes `hotel_room_types` and `room_rates` migrations explicitly. Revenue Manager includes `tenant_room_inventories` and `rate_recommendations`. These are the same thing being built twice within weeks of each other.
- **Lens 5 (Commerce chain completeness):** Fills the **Browse step** — "What can be sold?" and "What is the price?" — the very first step in the commerce chain. Currently G10 (`atlas_assets`) covers "what you own" but not "what you sell and at what price."

**Score: 8.5/10** — Resolves the single largest commerce chain gap (Browse), immediate debt from two apps building competing room-type/rate schemas simultaneously, and strongest Salesforce/Shopify precedent.

**Apps that benefit immediately:** Direct Booking Engine, Event-Aware Revenue Manager, PM (STR), Famtasm (subscription tiers), Flight + Hotel Package Builder, CoverFlow (coverage options/limits)

**Proposed tables:**
```sql
CREATE TABLE atlas_catalog_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
    -- What type of product this is:
    entry_type VARCHAR(50) NOT NULL,
    -- 'room_type', 'package_tier', 'service_slot', 'coverage_option', 'subscription_tier', 'add_on'
    name VARCHAR(255) NOT NULL,
    description TEXT,
    -- The underlying asset this catalog entry is based on (optional):
    asset_id UUID REFERENCES atlas_assets(id),  -- G10 — hotel room type → room asset hierarchy
    -- Base pricing:
    base_price_cents BIGINT NOT NULL,
    currency CHAR(3) NOT NULL DEFAULT 'USD',
    billing_interval VARCHAR(20),  -- NULL for one-time, 'nightly', 'monthly', 'annually'
    -- Availability:
    is_available BOOLEAN DEFAULT TRUE,
    min_quantity INT DEFAULT 1,
    max_quantity INT,
    -- Catalog metadata (app-specific attributes):
    catalog_metadata JSONB NOT NULL DEFAULT '{}',
    -- e.g. room_type: {bed_type, max_occupancy, view_type, amenities[]}
    -- e.g. subscription_tier: {feature_flags[], max_uploads, ai_credits}
    -- Display:
    sort_order INT DEFAULT 0,
    cover_image_attachment_id UUID REFERENCES attachment(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX atlas_catalog_tenant_type ON atlas_catalog_entries(tenant_id, entry_type, is_available);
CREATE INDEX atlas_catalog_asset ON atlas_catalog_entries(tenant_id, asset_id);

-- Rate rules (overrides base price for specific date ranges, channels, or segments):
CREATE TABLE atlas_catalog_rate_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    catalog_entry_id UUID NOT NULL REFERENCES atlas_catalog_entries(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
    rule_name VARCHAR(100),
    -- When this rule applies:
    applies_from DATE,
    applies_to DATE,
    day_of_week_mask INT,  -- bitmask: 1=Mon, 2=Tue, 4=Wed, 8=Thu, 16=Fri, 32=Sat, 64=Sun
    min_stay_nights INT,
    channel VARCHAR(50),   -- 'direct', 'ota', 'gds', 'corporate' — NULL means all channels
    -- Pricing:
    price_override_cents BIGINT,   -- absolute override
    price_modifier_pct NUMERIC(6, 2),  -- percentage modifier (e.g. +20 for weekend premium, -10 for early bird)
    -- Priority (higher = applied first):
    priority INT DEFAULT 0,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX atlas_catalog_rate_rules_entry ON atlas_catalog_rate_rules(catalog_entry_id, applies_from, applies_to);

-- Availability grid (for slot-based inventory with hard capacity):
CREATE TABLE atlas_catalog_availability (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    catalog_entry_id UUID NOT NULL REFERENCES atlas_catalog_entries(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenant(id) ON DELETE CASCADE,
    slot_date DATE NOT NULL,
    total_inventory INT NOT NULL,
    reserved_count INT NOT NULL DEFAULT 0,
    available_count INT GENERATED ALWAYS AS (total_inventory - reserved_count) STORED,
    is_blocked BOOLEAN DEFAULT FALSE,  -- manual block by operator
    block_reason VARCHAR(255),
    override_price_cents BIGINT,       -- day-specific price override
    UNIQUE(catalog_entry_id, slot_date)
);
CREATE INDEX atlas_catalog_availability_entry ON atlas_catalog_availability(catalog_entry_id, slot_date);
CREATE INDEX atlas_catalog_availability_available ON atlas_catalog_availability(tenant_id, catalog_entry_id, slot_date) WHERE available_count > 0 AND NOT is_blocked;
```

**Rust service signature:**
```rust
pub struct CatalogService;

impl CatalogService {
    pub async fn create_entry(db: &DatabaseConnection, tenant_id: Uuid, payload: CreateCatalogEntryPayload) -> Result<AtlasCatalogEntry>;
    pub async fn get_effective_price(db: &DatabaseConnection, entry_id: Uuid, date: NaiveDate, channel: Option<&str>, nights: Option<i32>) -> Result<i64>;
    pub async fn check_availability(db: &DatabaseConnection, entry_id: Uuid, from_date: NaiveDate, to_date: NaiveDate) -> Result<Vec<AtlasCatalogAvailability>>;
    pub async fn reserve_slots(db: &DatabaseConnection, entry_id: Uuid, from_date: NaiveDate, to_date: NaiveDate, quantity: i32) -> Result<()>;
    pub async fn release_slots(db: &DatabaseConnection, entry_id: Uuid, from_date: NaiveDate, to_date: NaiveDate, quantity: i32) -> Result<()>;
    pub async fn apply_rate_rule(db: &DatabaseConnection, tenant_id: Uuid, rule: CreateRateRulePayload) -> Result<AtlasCatalogRateRule>;
    pub async fn push_rates_to_pms(db: &DatabaseConnection, entry_id: Uuid, integration_id: Uuid, from_date: NaiveDate, to_date: NaiveDate) -> Result<()>;
    pub async fn list(db: &DatabaseConnection, filter: CatalogFilter) -> Result<Vec<AtlasCatalogEntry>>;
}
```

**Recommendation: PROMOTE IMMEDIATELY (parallel to G23)**

**Justification:** Direct Booking Engine and Revenue Manager are simultaneously building `hotel_room_types`/`room_rates` and `tenant_room_inventories`/`rate_recommendations` respectively — these are the same catalog + availability concept. The Revenue Manager's entire value proposition (dynamic pricing) is built on top of a rate calendar. If that rate calendar is a private app table instead of G26, the Direct Booking Engine cannot consume Revenue Manager's pricing outputs without a cross-app JOIN into an app-specific schema.

---

## Section 4: Priority Matrix

| Rank | Generic | Score | Debt Timeline | Apps Unblocked | Recommendation |
|------|---------|-------|---------------|----------------|----------------|
| **1** | **G23 `atlas_reservations`** | 10/10 | **Immediate** | Direct Booking, Flight+Hotel, Guest Comms, Revenue Manager, PM | **PROMOTE NOW** |
| **2** | **G26 `atlas_catalog`** | 8.5/10 | **Immediate** | Direct Booking, Revenue Manager, PM (STR), Famtasm, CoverFlow | **PROMOTE NOW** |
| **3** | **G19 `atlas_campaigns`** | 9.5/10 | **Immediate** | Clipping, PM, AgentLink, Nomad List, Direct Booking, Famtasm | **PROMOTE NOW** |
| **4** | **G25 `atlas_commission_plans`** | 9/10 | **Immediate** | CoverFlow, AgentLink, Direct Booking, Clipping, PM | **PROMOTE BEFORE COVERFLOW STARTS** |
| **5** | **G20 `atlas_attribution`** | 9/10 | Near-term | Direct Booking, Growth Platform, PM, AgentLink, Nomad List | PROMOTE BEFORE GROWTH PLATFORM STARTS |
| **6** | **G24 `atlas_quotes`** | 8.5/10 | Near-term | Direct Booking, CoverFlow, Flight+Hotel, AgentLink | PROMOTE BEFORE DIRECT BOOKING STARTS |
| **7** | **G21 `atlas_events`** | 9/10 | Near-term | PM, AgentLink, Nomad List, Direct Booking, Famtasm, Clipping | PROMOTE BEFORE PM EVENTS FEATURE |
| **8** | **G22 `atlas_record_relationships`** | 7.5/10 | Near-term | All apps | PROMOTE AFTER G19 AND G21 |

---

## Section 5: Implementation Sequencing

### Dependency Graph

```
G19 (Campaigns)
  └── G20 (Attribution) — references G19 + G21
      └── G21 (Events) — references G01, G03, G02, G19
          └── G22 (Record Relationships) — no deps; pure junction

G23 (Reservations) — references G03, G10, G05
  └── G24 (Quotes) — references G15, G23, G11
      └── G26 (Catalog) — references G10, G02

G25 (Commission Plans) — references G03, G11, G12
```

---

### Round 1 — Build Immediately (Unblocks Active Development)

These must begin before Direct Booking Engine, Revenue Manager, or Clipping Marketplace write their first migration.

```
Round 1 (Immediate — start this sprint):

  G23 atlas_reservations
    — Blocks: Direct Booking Engine (direct_bookings migration), Flight+Hotel (package_bookings),
              Guest Comms (guest_reservations), Revenue Manager (rate_pushes)
    — Without this: atlas_ledger_entries gets 4+ different billable_entity_type values for "booking"
    — Estimated effort: 2 developer-weeks (schema + service + hold-expiry background worker)

  G26 atlas_catalog
    — Blocks: Direct Booking Engine (room_types + room_rates), Revenue Manager (rate calendar)
    — Without this: Direct Booking and Revenue Manager each build a private pricebook that can't be shared
    — Estimated effort: 2 developer-weeks (schema + service + rate rule evaluation)
    — Can be built in parallel with G23

  G19 atlas_campaigns
    — Blocks: Clipping Marketplace (campaigns table), Growth Platform, AgentLink (recruiting campaigns)
    — Growth Platform DDL is already written — this is near-zero-effort to promote
    — Estimated effort: 1 developer-week (schema exists; service layer + enrollment background worker)
    — Can be built in parallel with G23 and G26

  G25 atlas_commission_plans
    — Blocks: CoverFlow (hardcoded Stripe splits), AgentLink (lead referral commissions)
    — Without this: commission logic is baked into handler code and becomes a migration to fix
    — Estimated effort: 1.5 developer-weeks (schema + calculation engine + G03 integration)
```

**Round 1 total effort estimate: 7–8 developer-weeks across 4 generics (parallelizable to 3–4 weeks)**

---

### Round 2 — Build Before Next App Starts (1–2 Months)

```
Round 2 (Near-term — before CoverFlow, Flight+Hotel, or PM events begin):

  G24 atlas_quotes (Quote + Line Items)
    — Needed before: CoverFlow (rating engine), Direct Booking (corp contracts), Flight+Hotel
    — Depends on: G23 (Reservations) for conversion path
    — Estimated effort: 1.5 developer-weeks

  G21 atlas_events (Event Management + Ticketing)
    — Needed before: Revenue Manager builds miami_events as app-specific table
    — Needed before: PM builds open-house events feature
    — Growth Platform DDL is already written — near-zero-effort to promote
    — Depends on: G19 (Campaigns) for campaign linkage, G03 for ticket payments
    — Estimated effort: 1 developer-week (DDL exists; registration + check-in service)

  G20 atlas_attribution (Multi-Channel Attribution)
    — Needed before: Direct Booking Engine runs paid acquisition
    — Needed before: Growth Platform ships
    — Depends on: G19 (Campaigns), G21 (Events) for FK references
    — Estimated effort: 1.5 developer-weeks + frontend UTM capture middleware
```

**Round 2 total estimate: 4–5 developer-weeks across 3 generics**

---

### Round 3 — Document Now, Build When Second App Needs It

```
Round 3 (Defer — promote when scope is confirmed):

  G22 atlas_record_relationships
    — Candidate: promote when the first G19 campaign needs M:M with G10 assets
    — Schema is 15 lines of SQL — can be added in hours when needed
    — No urgency: the polymorphic pattern already handles 90% of use cases
```

---

## Section 6: Generics That Are Built But Incomplete

| Generic | What's Missing | Apps Affected | Recommended Resolution |
|---------|---------------|---------------|----------------------|
| **G03 `atlas_payments`** | No plan governing what split to compute — `atlas_ledger_splits` records the split but nothing defines *how* the split was calculated or who has the authority to change it. Commission plan hardcoding in handlers is the result. | CoverFlow, AgentLink, Direct Booking, Clipping, PM | **Build G25 `atlas_commission_plans`** with FK from `atlas_ledger_splits` to the plan that generated each split |
| **G10 `atlas_assets`** | No pricing or availability layer — assets represent what you own, not what you sell at what price on what dates. Apps abuse `attributes JSONB` to store rate calendars and availability windows. | Direct Booking, Revenue Manager, PM (STR), Famtasm | **Build G26 `atlas_catalog`** with FK to `atlas_assets.id` for the underlying physical asset |
| **G11 `atlas_contracts`** | No quote-to-contract conversion path. A contract is a signed agreement; a quote is the proposal before signing. Currently apps skip Quote entirely and jump from Opportunity (G15) to Contract (G11), losing the proposal stage. | Direct Booking (corp agreements), CoverFlow (bound policies), AgentLink (producer contracts) | **Build G24 `atlas_quotes`** with `accepted_quote_id` FK on `atlas_contracts` |
| **G13 `atlas_cases`** | Being overloaded as a reservation object. `case_metadata` stores `check_in`, `check_out`, `room_type`, `guest_count` in Direct Booking and Guest Comms integration specs. A reservation is not a case — it is a time-bounded asset claim. | Direct Booking, Guest Comms, Revenue Manager | **Build G23 `atlas_reservations`** and add `reservation_id` FK to `atlas_cases` for cases *about* a reservation (disputes, issues) |
| **G14 `atlas_documents`** | E-signature workflow is absent. Document model exists and stores PDFs, but the signing ceremony (signer tracking, document lock on completion, multi-party signing order) is missing. Multiple sales documents explicitly promise this capability. | PM (lease signing), CoverFlow (policy documents), AgentLink (consent documents), Insurance Defense (engagement letters) | Build `atlas_esig_requests` + `atlas_esig_events` companion tables, or integrate third-party (DocuSign embed) with G14 as the document store. This is an incomplete generic, not a missing one. |
| **G07 `atlas_realtime`** | No notification orchestration layer. WebSocket rooms and messages exist, but there is no rule engine, template system, suppression log, or outbound notification scheduler. Every app that needs "send a message when X happens" will build its own outbox worker or abuse the existing `outbox_jobs` table. | All apps — specifically Guest Comms (campaign triggers), Direct Booking (booking confirmations), PM (maintenance alerts) | Build `atlas_notification_rules` (trigger condition + template + channel) + `atlas_notification_log` (delivery record) as companion tables to G07 |
| **G15 `atlas_opportunities`** | No line items. Financial modeling is in JSONB on the opportunity record (`financial_inputs` / `financial_outputs`) with no structured line-item breakdown. This forces quote logic into JSONB. | Direct Booking (package line items), CoverFlow (coverage components), Flight+Hotel (flight + room line items) | **Build G24 `atlas_quotes`** — the line-item structure belongs on the Quote, not the Opportunity. Opportunity = pipeline stage; Quote = priced proposal. |
| **G04 `atlas_subscriptions`** | Stripe webhook to subscription status update is incomplete. `atlas_subscriptions` models the subscription, but the Stripe webhook handler that updates `status` on renewal, failure, and cancellation events is not yet wired. Every app building B2C subscriptions (Famtasm, Clipping) will implement their own Stripe webhook handler. | Famtasm, Clipping, Nomad List (city plans), Direct Booking (loyalty plans) | Platform-level Stripe webhook handler that maps `customer.subscription.updated` → `SubscriptionService::update_status()`. This is missing platform plumbing, not a schema gap. |

---

## Section 7: Executive Summary

### 1. Total New Generics Recommended

**8 new generics** (G19–G26):
- **Promote immediately (Round 1):** G23 `atlas_reservations`, G26 `atlas_catalog`, G19 `atlas_campaigns`, G25 `atlas_commission_plans` — 4 generics, ~7–8 developer-weeks effort
- **Promote in Round 2 (before next app starts):** G24 `atlas_quotes`, G21 `atlas_events`, G20 `atlas_attribution` — 3 generics, ~4–5 developer-weeks
- **Defer / build on demand:** G22 `atlas_record_relationships` — 1 generic, ~0.5 developer-week when needed

**Total promotion effort:** 12–14 developer-weeks across 8 generics, achievable in 6–8 calendar weeks with 2 developers working in parallel.

---

### 2. Biggest Rule 7 Risk Identified

**Direct Booking Engine** is the single highest Rule 7 risk on the platform.

Its Phase 1 migration script explicitly includes `hotel_room_types`, `room_rates`, `corporate_contracts`, `direct_bookings`, and `booking_items` — **five app-specific tables**, four of which are generic candidates:
- `hotel_room_types` + `room_rates` → **G26 `atlas_catalog`**
- `corporate_contracts` → **G24 `atlas_quotes`**
- `direct_bookings` + `booking_items` → **G23 `atlas_reservations`**

If Direct Booking proceeds with its Phase 1 migration as documented, the platform will have private versions of its three most critical missing generics embedded in an app-specific schema before the generics are promoted. The Event-Aware Revenue Manager has the same problem with `tenant_room_inventories` and `rate_recommendations`.

**The risk window:** Both apps appear to be in active or near-active development. Round 1 generics must be promoted before either app writes a migration.

---

### 3. Commerce Chain Completeness

**Current state (G01–G18 only):** 5 of 9 commerce chain steps are fully covered.

**After adding G19–G26:** 8 of 9 steps covered. The remaining partial coverage is in the **Reporting** step (no saved report definitions or dashboard builder).

| Commerce Step | Before (G01–G18) | After Adding G19–G26 | Gap Remaining |
|--------------|------------------|---------------------|---------------|
| Browse: what can be sold | ❌ Partial (G10 = what you own, not what you sell) | ✅ **G26 atlas_catalog** | None |
| Browse: pricing for that item | ❌ None | ✅ **G26 rate rules + availability grid** | None |
| Browse: is it available on these dates | ❌ None | ✅ **G26 atlas_catalog_availability** | None |
| Quote: priced list of items for a prospect | ❌ Partial (G15 = pipeline, no line items) | ✅ **G24 atlas_quotes + line items** | None |
| Commit: confirmed reservation with hold | ❌ None | ✅ **G23 atlas_reservations** | None |
| Pay | ✅ G03 complete | ✅ G03 complete | None |
| Deliver | ✅ G13 + G14 complete | ✅ G13 + G14 complete | None |
| Commission | ⚠️ Partial (G03 records split, no plan) | ✅ **G25 atlas_commission_plans** | None |
| Tax | ✅ G17 complete | ✅ G17 complete | None |
| Report | ⚠️ Partial (metrics aggregated, no dashboard builder) | ⚠️ Still partial | Reporting generic deferred |

**Commerce chain completeness after G19–G26: ~89% (8/9 steps fully covered)**

---

### 4. One-Sentence Verdict on Platform Scalability

**The platform cannot support 10 apps without each one reinventing reservation management, product catalogs, and commission logic — but it can, if G23, G26, and G25 are promoted to generics before Direct Booking Engine and CoverFlow write their first migration.**

---

*This analysis was performed against the complete integration mapping documentation for all 14 roadmap apps, all 5 architectural lens criteria, and all sales prospect analyses as of June 2026. All DDL provided is schema-level and not production-ready — indexes, constraints, and ENUM idempotency patterns should follow the conventions established in `backend/src/migration/`.*

*© Copyright Ruud Salym Erie & Oplyst International, LLC. All Rights Reserved.*
