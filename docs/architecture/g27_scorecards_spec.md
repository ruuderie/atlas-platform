# G-27: Atlas Scorecards
## Comprehensive Technical Specification & Business Analysis

> **Status:** Proposed — Pending Implementation  
> **Generic ID:** G-27  
> **Tables:** `atlas_scorecard_templates`, `atlas_scorecard_dimensions`,
> `atlas_scorecard_dimension_options`, `atlas_scorecards`,
> `atlas_rating_sessions`, `atlas_scorecard_entries`,
> `atlas_scorecard_dimension_aggregates`, `atlas_scorecard_poll_aggregates`,
> `atlas_scorecard_time_series`, `atlas_scorecard_targets`,
> `atlas_scorecard_target_criteria`  
> **Rule 7 Status:** PASSES — 8+ confirmed cross-app use cases identified

---

## 1. Executive Summary

G-27 is a universal structured evaluation engine. It solves a single recurring
problem: **any entity on the platform needs to be evaluated across standardized
dimensions by multiple contributors, with results aggregated into comparable,
benchmarked scores over time.**

The same data structure powers:
- A NomadList-style city comparison engine
- Bridgewater-style employee Baseball Cards and the Combinator
- Restaurant and venue quality tracking (per-visit, longitudinal)
- Beauty and consumer product reviews
- Contractor performance scoring across jobs
- Event staff performance per shift
- Carrier/MGA ratings in insurance
- Agent ratings in real estate

The key insight is that the **entity changes but the engine does not**. A city,
a person, a restaurant, a product, and a contractor job are all just different
subjects of the same scorecard primitive.

---

## 2. Core Concepts

### 2.1 The Five Objects

```
Template ──── defines what traits exist for an entity type
   │
   └── Dimensions ──── individual traits with scale, benchmarks, options
         │
         └── Scorecard ──── template applied to one specific entity instance
               │
               ├── Rating Sessions ──── one per discrete occurrence
               │       │               (job, visit, stay, event shift)
               │       └── Entries ──── sparse scores per dimension per session
               │
               ├── Dimension Aggregates ──── rolled-up community scores
               ├── Poll Aggregates ──── vote counts for categorical dimensions
               └── Time Series ──── monthly/quarterly trend buckets
```

### 2.2 The Five Response Types

All community input flows through `atlas_scorecard_entries.score` (numeric) or
`atlas_scorecard_entries.option_id` (categorical). The `scale_type` on the
dimension determines which field is used and how aggregation works.

| `scale_type` | Input | Aggregation | Display example |
|---|---|---|---|
| `rating` | Numeric 1–10 | Weighted mean | `"Good"` bar at 75% fill |
| `absolute` | Real-world value | Mean of actuals | `"Fast: 16 Mbps (avg)"` |
| `boolean` | Yes=1.0 / No=0.0 | `% true` | `"83% say clean"` |
| `poll_single` | Pick one option | Vote count → rank | `"Telkomsel 65%"` |
| `poll_multi` | Pick many options | Vote count per option | `"Dry 78%, Sensitive 61%"` |

### 2.3 Benchmark Tiers — "The Bar"

Every dimension carries a `benchmark_tiers` JSONB array that defines what each
score level *means* in plain language. This is what converts a raw aggregate
of 7.3 into "Above the bar" or 102 μg/m³ into "Stay inside."

```json
// Rating dimension: food quality
[
  {"label": "Outstanding",    "min_score": 9.0, "color": "#00cc44"},
  {"label": "Above the bar",  "min_score": 7.5, "color": "#88cc00"},
  {"label": "At the bar",     "min_score": 6.0, "color": "#ffaa00"},
  {"label": "Below the bar",  "min_score": 4.0, "color": "#ff6600"},
  {"label": "Avoid",          "min_score": 0.0, "color": "#ff0000"}
]

// Absolute dimension: air quality (μg/m³) — lower is better
[
  {"label": "Excellent",    "max_score": 12,  "color": "#00cc44"},
  {"label": "Good",         "max_score": 35,  "color": "#88cc00"},
  {"label": "Sensitive",    "max_score": 55,  "color": "#ffaa00"},
  {"label": "Unhealthy",    "max_score": 150, "color": "#ff6600",
   "prefix": "Unhealthy:", "show_value": true},
  {"label": "Stay inside",  "max_score": 999, "color": "#ff0000",
   "prefix": "Stay inside:", "show_value": true}
]

// Boolean dimension: bathroom cleanliness
[
  {"label": "Most say clean",  "min_pct": 70, "color": "#00cc44"},
  {"label": "Mixed reports",   "min_pct": 40, "color": "#ffaa00"},
  {"label": "Most say dirty",  "min_pct": 0,  "color": "#ff0000"}
]
```

The `global_reference_value` defines the comparative baseline — the "industry
standard" or "global average" against which every entity's score is compared.
This produces the `vs_global_label` field: `'above'`, `'at'`, or `'below'`.

### 2.4 Rating Sessions — Longitudinal vs. Static

The difference between a static profile and a living record is the **rating
session**. Without sessions, you can only rate a contractor once, ever. With
sessions, every job becomes a data point.

```
STATIC (no sessions):
  UNIQUE(scorecard_id, dimension_id, contributor_user_id)
  → One rating per person per trait per entity, forever.
  → Use case: NomadList city score (one person rates one city once)

LONGITUDINAL (with sessions):
  Rating Session → one per discrete occurrence
  UNIQUE(session_id, dimension_id, rater_user_id)
  → Multiple sessions per entity (one per job, per visit, per event shift)
  → Use case: Contractor job quality over 47 jobs; employee per event
```

Sessions link back to existing platform records via `context_entity_type` and
`context_entity_id`, avoiding data duplication:

| Session context | Links to | Generic |
|---|---|---|
| Contractor job | `atlas_case` | G-13 |
| Hotel stay / booking | `atlas_reservation` | G-23 |
| Event shift | `atlas_events` | G-21 (Round 2) |
| Service appointment | `atlas_case` | G-13 |
| Product purchase | `atlas_catalog_entry` | G-26 |

### 2.5 Official Data vs. Community Data

Not all entity attributes are community-sourced. The platform uses two
complementary stores:

| Data type | Home | Examples | Updated by |
|---|---|---|---|
| Official/factual | `atlas_assets.asset_metadata` JSONB | Population, timezone, currency | Background job syncing external API |
| Computed/derived | `atlas_assets.asset_metadata` JSONB | Return rate, avg stay | Background job from behavioral data |
| Community rated | G-27 `atlas_scorecard_entries` | Safety feeling, food quality | User submissions via sessions |
| Community polled | G-27 `atlas_scorecard_entries` (option_id) | Best carrier, best hospital | User poll votes |

The entity page aggregates all four sources without the user knowing which
system produced which data point.

---

## 3. Full DDL

### 3.1 Template & Dimensions

```sql
-- ── Template ──────────────────────────────────────────────────────────────
CREATE TABLE atlas_scorecard_templates (
    id                      UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id               UUID        NOT NULL,
    name                    VARCHAR(255) NOT NULL,
    -- Discriminator: 'city','person','restaurant','product','contractor',
    --                'airline','property','hotel','agent','carrier','event'
    entity_type             VARCHAR(50) NOT NULL,
    description             TEXT,
    -- 'weighted_mean' | 'simple_mean' | 'percentile_rank'
    scoring_method          VARCHAR(30) NOT NULL DEFAULT 'weighted_mean',
    default_scale_min       DECIMAL(6,2) NOT NULL DEFAULT 1,
    default_scale_max       DECIMAL(6,2) NOT NULL DEFAULT 10,
    min_entries_to_publish  INT         NOT NULL DEFAULT 5,
    is_published            BOOLEAN     NOT NULL DEFAULT false,
    created_by_user_id      UUID,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── Dimensions ────────────────────────────────────────────────────────────
CREATE TABLE atlas_scorecard_dimensions (
    id                      UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id             UUID        NOT NULL
                                REFERENCES atlas_scorecard_templates(id),
    tenant_id               UUID        NOT NULL,
    slug                    VARCHAR(100) NOT NULL,
    name                    VARCHAR(255) NOT NULL,
    description             TEXT,
    category                VARCHAR(50),
    weight                  DECIMAL(5,4) NOT NULL DEFAULT 1.0,

    -- Response type
    -- 'rating'      → subjective 1–10 (or custom scale)
    -- 'absolute'    → real-world unit (Mbps, USD, °C, hrs, minutes)
    -- 'boolean'     → yes/no, has/doesn't-have, pass/fail
    -- 'poll_single' → pick one from predefined options
    -- 'poll_multi'  → pick multiple from predefined options
    scale_type              VARCHAR(20) NOT NULL DEFAULT 'rating',
    scale_min               DECIMAL(10,2) NOT NULL DEFAULT 1,
    scale_max               DECIMAL(10,2) NOT NULL DEFAULT 10,
    unit_label              VARCHAR(30),        -- 'Mbps', 'USD/mo', '°C', 'hrs'

    -- Benchmark tier configuration (see section 2.3)
    benchmark_tiers         JSONB       NOT NULL DEFAULT '[]',

    -- Global reference baseline for above/below-bar comparison
    global_reference_value  DECIMAL(10,2),
    global_reference_label  VARCHAR(100),

    min_entries_to_show     INT         NOT NULL DEFAULT 3,
    is_community_ratable    BOOLEAN     NOT NULL DEFAULT true,
    is_active               BOOLEAN     NOT NULL DEFAULT true,
    sort_order              INT         NOT NULL DEFAULT 0,
    UNIQUE (template_id, slug)
);

-- ── Poll Options ──────────────────────────────────────────────────────────
-- Only used for dimensions where scale_type IN ('poll_single','poll_multi')
CREATE TABLE atlas_scorecard_dimension_options (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    dimension_id    UUID        NOT NULL
                        REFERENCES atlas_scorecard_dimensions(id),
    tenant_id       UUID        NOT NULL,
    label           VARCHAR(255) NOT NULL,   -- "Telkomsel", "BIMC Kuta Hospital"
    value_key       VARCHAR(100),            -- stable slug: 'telkomsel'
    description     TEXT,
    image_url       TEXT,
    sort_order      INT         NOT NULL DEFAULT 0,
    is_write_in     BOOLEAN     NOT NULL DEFAULT false
);
```

### 3.2 Scorecards & Sessions

```sql
-- ── Scorecard ─────────────────────────────────────────────────────────────
CREATE TABLE atlas_scorecards (
    id                      UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id               UUID        NOT NULL,
    template_id             UUID        NOT NULL
                                REFERENCES atlas_scorecard_templates(id),
    -- Polymorphic subject
    subject_entity_type     VARCHAR(50) NOT NULL,
    subject_entity_id       UUID        NOT NULL,
    -- Computed composite (recomputed by background job)
    composite_score         DECIMAL(5,2),
    -- 'insufficient'(<5) | 'low'(<10) | 'medium'(<50) | 'high'(<200) | 'very_high'
    confidence_level        VARCHAR(20) NOT NULL DEFAULT 'insufficient',
    total_contributors      INT         NOT NULL DEFAULT 0,
    total_sessions          INT         NOT NULL DEFAULT 0,
    total_entries           INT         NOT NULL DEFAULT 0,
    -- Vector for similarity search (The Combinator)
    -- Ordered array of weighted scores, one per dimension (sort_order sequence)
    dimension_vector        DECIMAL(5,2)[],
    last_computed_at        TIMESTAMPTZ,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (template_id, subject_entity_type, subject_entity_id)
);

CREATE INDEX idx_atlas_scorecards_entity
    ON atlas_scorecards (subject_entity_type, subject_entity_id);

-- ── Rating Sessions ───────────────────────────────────────────────────────
-- One per discrete occurrence. City visit. Contractor job. Event shift.
-- Product use. Hotel stay. Enables longitudinal tracking.
CREATE TABLE atlas_rating_sessions (
    id                      UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    scorecard_id            UUID        NOT NULL REFERENCES atlas_scorecards(id),
    tenant_id               UUID        NOT NULL,
    rater_user_id           UUID        NOT NULL,
    occurred_at             TIMESTAMPTZ NOT NULL,
    -- 'job'|'stay'|'visit'|'event_shift'|'purchase'|'flight'|'meeting'
    session_type            VARCHAR(30) NOT NULL,
    -- Links back to existing platform records (avoids data duplication)
    context_entity_type     VARCHAR(50),   -- 'atlas_case','atlas_reservation',...
    context_entity_id       UUID,
    session_label           TEXT,
    -- 'draft' | 'submitted' | 'verified' | 'disputed'
    status                  VARCHAR(20) NOT NULL DEFAULT 'submitted',
    verification_request_id UUID,          -- G-06 gate when needed
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_rating_sessions_scorecard_time
    ON atlas_rating_sessions (scorecard_id, occurred_at DESC);

CREATE INDEX idx_rating_sessions_context
    ON atlas_rating_sessions (context_entity_type, context_entity_id);

CREATE INDEX idx_rating_sessions_rater
    ON atlas_rating_sessions (tenant_id, rater_user_id, occurred_at DESC);
```

### 3.3 Entries

```sql
-- ── Entries ───────────────────────────────────────────────────────────────
-- Sparse: contributor submits only the dimensions they have experience with.
-- One row per (session, dimension, rater). Hard unique constraint at DB level.
CREATE TABLE atlas_scorecard_entries (
    id                      UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id              UUID        NOT NULL
                                REFERENCES atlas_rating_sessions(id),
    scorecard_id            UUID        NOT NULL REFERENCES atlas_scorecards(id),
    dimension_id            UUID        NOT NULL
                                REFERENCES atlas_scorecard_dimensions(id),
    tenant_id               UUID        NOT NULL,
    contributor_user_id     UUID        NOT NULL,

    -- For rating/absolute/boolean: the numeric value
    score                   DECIMAL(8,2),
    -- For poll_single/poll_multi: the selected option
    option_id               UUID REFERENCES atlas_scorecard_dimension_options(id),
    -- Exactly one of score or option_id must be non-null (enforced in service layer)

    -- Evidence source type
    -- 'community_rating'  → public user rates entity
    -- 'peer_review'       → colleague rates colleague (Bridgewater)
    -- 'self_assessment'   → subject rates themselves
    -- 'manager_review'    → manager rates direct report
    -- 'test_result'       → objective scored test
    -- 'behavioral_signal' → inferred from recorded choices/actions
    -- 'official_data'     → external API feed (Speedtest, Numbeo, weather)
    source_type             VARCHAR(30) NOT NULL DEFAULT 'community_rating',

    -- Credibility and weighting context (source_type-specific)
    -- Community: {"visit_start":"2024-03","duration_days":90,"purpose":"work"}
    -- Peer review: {"relationship":"peer","worked_together_months":18}
    -- Test result: {"test_name":"CRT","date":"2024-01","administered_by":"HR"}
    context                 JSONB,

    -- Optional written note for this dimension
    note                    TEXT,

    -- G-06 verification gate
    is_verified             BOOLEAN     NOT NULL DEFAULT false,
    verification_request_id UUID,

    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (session_id, dimension_id, contributor_user_id)
);

CREATE INDEX idx_scorecard_entries_scorecard_verified
    ON atlas_scorecard_entries (scorecard_id, is_verified, dimension_id);
```

### 3.4 Aggregates & Time Series

```sql
-- ── Dimension Aggregates ──────────────────────────────────────────────────
CREATE TABLE atlas_scorecard_dimension_aggregates (
    scorecard_id            UUID        NOT NULL REFERENCES atlas_scorecards(id),
    dimension_id            UUID        NOT NULL
                                REFERENCES atlas_scorecard_dimensions(id),
    mean_score              DECIMAL(5,2),
    weighted_mean_score     DECIMAL(5,2),
    -- For boolean dimensions
    percent_true            DECIMAL(5,2),
    -- Resolved benchmark tier for this aggregate
    benchmark_label         VARCHAR(100),
    benchmark_color         VARCHAR(7),
    -- Human-readable display: "Fast: 16 Mbps", "$1,183/mo", "83% say clean"
    display_value           VARCHAR(150),
    std_deviation           DECIMAL(5,2),
    -- 'strong_consensus'|'consensus'|'mixed'|'disputed'
    consensus_level         VARCHAR(20),
    min_score               DECIMAL(5,2),
    max_score               DECIMAL(5,2),
    contributor_count       INT         NOT NULL DEFAULT 0,
    session_count           INT         NOT NULL DEFAULT 0,
    -- Delta from global_reference_value
    vs_global_delta         DECIMAL(8,2),
    vs_global_label         VARCHAR(10),   -- 'above'|'at'|'below'
    last_computed_at        TIMESTAMPTZ,
    PRIMARY KEY (scorecard_id, dimension_id)
);

-- ── Poll Aggregates ───────────────────────────────────────────────────────
CREATE TABLE atlas_scorecard_poll_aggregates (
    scorecard_id            UUID        NOT NULL REFERENCES atlas_scorecards(id),
    dimension_id            UUID        NOT NULL
                                REFERENCES atlas_scorecard_dimensions(id),
    option_id               UUID        NOT NULL
                                REFERENCES atlas_scorecard_dimension_options(id),
    vote_count              INT         NOT NULL DEFAULT 0,
    vote_pct                DECIMAL(5,2),
    rank                    INT         NOT NULL,
    total_voters            INT         NOT NULL DEFAULT 0,
    last_computed_at        TIMESTAMPTZ,
    PRIMARY KEY (scorecard_id, dimension_id, option_id)
);

-- ── Time Series ───────────────────────────────────────────────────────────
CREATE TABLE atlas_scorecard_time_series (
    scorecard_id            UUID        NOT NULL REFERENCES atlas_scorecards(id),
    dimension_id            UUID        NOT NULL
                                REFERENCES atlas_scorecard_dimensions(id),
    period_start            DATE        NOT NULL,
    period_type             VARCHAR(10) NOT NULL DEFAULT 'monthly',
    mean_score              DECIMAL(5,2),
    session_count           INT         NOT NULL DEFAULT 0,
    contributor_count       INT         NOT NULL DEFAULT 0,
    delta_from_prior        DECIMAL(5,2),
    -- 'improving'|'stable'|'declining'|'insufficient_data'
    trend_direction         VARCHAR(20),
    PRIMARY KEY (scorecard_id, dimension_id, period_type, period_start)
);

-- ── Target Profiles (The Combinator) ──────────────────────────────────────
CREATE TABLE atlas_scorecard_targets (
    id                      UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id             UUID        NOT NULL
                                REFERENCES atlas_scorecard_templates(id),
    tenant_id               UUID        NOT NULL,
    name                    VARCHAR(255) NOT NULL,
    -- 'search_filter'|'job_specification'|'ideal_profile'
    target_type             VARCHAR(30) NOT NULL,
    description             TEXT,
    seed_entity_ids         UUID[],       -- source entities for ideal_profile
    target_vector           DECIMAL(5,2)[],
    created_by_user_id      UUID,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE atlas_scorecard_target_criteria (
    id                      UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    target_id               UUID        NOT NULL
                                REFERENCES atlas_scorecard_targets(id),
    dimension_id            UUID        NOT NULL
                                REFERENCES atlas_scorecard_dimensions(id),
    min_score               DECIMAL(5,2),
    max_score               DECIMAL(5,2),
    ideal_score             DECIMAL(5,2),
    is_dealbreaker          BOOLEAN     NOT NULL DEFAULT false,
    search_weight           DECIMAL(5,4)
);
```

---

## 4. Use Case Matrix

| App | Entity | Template | Session type | Key dimensions | Combinator use |
|---|---|---|---|---|---|
| **NomadList** | City (G-10) | Nomad City | `visit` (linked G-23) | internet_speed, cost, safety, nightlife, air_quality | "Find cities like Medellín" |
| **NomadList** | Airline | Airline Experience | `flight` | legroom, wifi, punctuality, food, service | "Best airlines for long-haul remote work" |
| **NomadList** | Neighborhood | Neighborhood | `visit` | walkability, noise, cafe_density, safety | "Most nomad-friendly area in Bangkok" |
| **Bridgewater OS** | Person (accounts) | Employee Baseball Card | `meeting`, `project` | reliability, conceptual_thinking, communication, follow-through | Combinator: match to role spec |
| **PM / STR** | Property (G-10) | STR Property | `stay` (linked G-23) | cleanliness, host_responsiveness, value, location_accuracy | "Find properties like our best-rated unit" |
| **Direct Booking** | Hotel room (G-26) | Hotel Assessment | `stay` (linked G-23) | room_cleanliness, bathroom_clean, wifi_speed, noise, value | — |
| **Restaurant OS** | Restaurant (G-10) | Restaurant | `visit` | food_quality, bathroom_clean, wait_time, price_vs_value, would_return | "Find restaurants like our top 3" |
| **Beauty/Consumer** | Product (G-26) | Beauty Product | `purchase` | scent, longevity, caused_irritation, price_per_use, would_repurchase | "Find products like this one for dry skin" |
| **Contractor OS** | Contractor (G-12) | Contractor Assessment | `job` (linked G-13) | quality, punctuality, cleanup, communication | "Find contractors with profile like our best performer" |
| **Event Staffing** | Employee (accounts) | Event Staff | `event_shift` (linked G-21) | energy, reliability, guest_interaction, setup_speed | "Find staff like our top 5 for gala events" |
| **AgentLink** | Agent (G-12) | Agent Performance | `deal` (linked G-15) | market_knowledge, responsiveness, negotiation, follow-through | "Find agents matching this ICP" |
| **Insurance (CoverFlow)** | Carrier / MGA | Carrier Rating | `claim` | claim_speed, communication, payout_accuracy, support | "Carriers similar to top performers" |

---

## 5. Service Layer

### 5.1 Core Service Operations

```rust
pub struct ScorecardService;

impl ScorecardService {
    /// Create a scorecard for an entity if one doesn't exist.
    /// Idempotent — returns existing scorecard if already present.
    pub async fn get_or_create(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        template_id: Uuid,
        subject_entity_type: &str,
        subject_entity_id: Uuid,
    ) -> Result<Uuid, String>;

    /// Open a rating session for a discrete occurrence.
    /// Optionally links to an existing platform record (job, booking, event).
    pub async fn open_session(
        db: &DatabaseConnection,
        scorecard_id: Uuid,
        rater_user_id: Uuid,
        occurred_at: DateTime<Utc>,
        session_type: &str,
        context_entity_type: Option<&str>,
        context_entity_id: Option<Uuid>,
    ) -> Result<Uuid, String>;

    /// Submit a score for one dimension within a session.
    /// Enforces the UNIQUE(session, dimension, rater) constraint.
    /// Queues for G-06 verification if required by template config.
    pub async fn submit_entry(
        db: &DatabaseConnection,
        session_id: Uuid,
        dimension_id: Uuid,
        score: Option<f64>,         // for rating/absolute/boolean
        option_id: Option<Uuid>,    // for poll_single/poll_multi
        source_type: &str,
        context: serde_json::Value,
        note: Option<&str>,
    ) -> Result<Uuid, String>;

    /// Recompute all aggregates for a scorecard after verified entries change.
    /// Called by background job or G-06 verification webhook.
    pub async fn recompute_aggregates(
        db: &DatabaseConnection,
        scorecard_id: Uuid,
    ) -> Result<(), String>;

    /// Run the Combinator: find entities most similar to a target vector.
    /// Returns (scorecard_id, entity_id, similarity_score) ordered by similarity.
    pub async fn find_similar(
        db: &DatabaseConnection,
        template_id: Uuid,
        target_vector: Vec<f64>,
        limit: u64,
        min_confidence: &str,       // 'low'|'medium'|'high'
    ) -> Result<Vec<SimilarityResult>, String>;
}
```

### 5.2 Aggregation Engine (Background Job)

The `recompute_scorecard` background job fires after every verified entry
and once nightly for all scorecards with uncomputed sessions.

```rust
// Pseudo-code for the aggregation branch logic
fn recompute_dimension_aggregate(
    entries: &[ScorecardEntry],
    dimension: &ScorecardDimension,
) -> DimensionAggregate {
    match dimension.scale_type.as_str() {
        "rating" | "absolute" => {
            let context_weight = |e: &ScorecardEntry| -> f64 {
                // Weight by credibility signal in context JSONB
                // duration_days for visits, worked_together_months for peer reviews
                e.context["duration_days"].as_f64()
                    .or(e.context["worked_together_months"].as_f64())
                    .map(|v| (v / 30.0).min(3.0).max(0.5))  // cap at 3x weight
                    .unwrap_or(1.0)
            };
            let weighted_mean = weighted_average(entries, context_weight);
            let benchmark = resolve_tier(weighted_mean, &dimension.benchmark_tiers);
            let display = format_display(weighted_mean, dimension);
            // ...
        }
        "boolean" => {
            let pct = entries.iter().filter(|e| e.score >= 1.0).count() as f64
                / entries.len() as f64;
            let benchmark = resolve_boolean_tier(pct, &dimension.benchmark_tiers);
            // ...
        }
        "poll_single" | "poll_multi" => {
            // Count votes per option, compute percentages, rank
            // → atlas_scorecard_poll_aggregates (separate table)
        }
    }
}

fn build_dimension_vector(
    aggregates: &[DimensionAggregate],
    dimensions: &[ScorecardDimension],
) -> Vec<f64> {
    // Ordered by dimension.sort_order
    // Normalized to 0.0–1.0 range: (score - scale_min) / (scale_max - scale_min)
    // Weighted: normalized_score * dimension.weight
    // Zero-filled for dimensions with no entries
    dimensions.iter()
        .map(|d| {
            aggregates.iter()
                .find(|a| a.dimension_id == d.id)
                .and_then(|a| a.weighted_mean_score)
                .map(|s| (s - d.scale_min) / (d.scale_max - d.scale_min) * d.weight)
                .unwrap_or(0.0)
        })
        .collect()
}
```

### 5.3 Background Jobs to Register

Two jobs in `CorePlatformApp::background_jobs()`:

| Job type | Interval | What it does |
|---|---|---|
| `recompute_scorecard_aggregates` | 5 min | Processes sessions verified since last run; recomputes aggregates and vectors |
| `refresh_scorecard_time_series` | 1 hr | Buckets session data into monthly/quarterly periods; computes trend directions |

---

## 6. The Benchmark Standard System

The `global_reference_value` + `benchmark_tiers` combination is what transforms
raw numbers into actionable intelligence. Here is how it works end-to-end for
a restaurant's bathroom cleanliness dimension:

```
1. Template admin sets:
   dimension: bathroom_clean
   scale_type: boolean
   benchmark_tiers: [
     {"label":"Most say clean",  "min_pct":70, "color":"#00cc44"},
     {"label":"Mixed reports",   "min_pct":40, "color":"#ffaa00"},
     {"label":"Most say dirty",  "min_pct":0,  "color":"#ff0000"}
   ]
   global_reference_value: 0.75  ← 75% is the "at the bar" standard
                                    for full-service restaurants

2. After 47 verified entries:
   percent_true = 0.83             → "above the bar" (83% > 75% reference)
   benchmark_label = "Most say clean"
   display_value = "83% say clean"
   vs_global_label = "above"

3. Time series (monthly):
   Jan: 91%  Feb: 88%  Mar: 83%  Apr: 71%  May: 62%
   trend_direction = "declining"   ← actionable signal for owner dashboard

4. Owner sees: bathroom cleanliness was a strength, now approaching "at the bar."
   The platform can trigger an alert when it crosses below reference.
```

---

## 7. Most Valuable Datasets

Ranked by commercial value and strategic defensibility.

### Tier 1 — High Value, High Defensibility

#### 7.1 Verified Human Capital Intelligence

**What it is:** Structured, multi-source performance profiles of individuals
across behavioral dimensions — aggregated across peer reviews, manager
assessments, test results, and self-assessments.

**Why it's valuable:** This is validated, longitudinal, multi-rater data on
human performance. It is extraordinarily difficult to replicate.

**Who pays:**
- **Enterprise HR platforms** — $50K–$500K/yr API licenses for talent benchmarking
- **Recruiting firms** — pay per candidate assessed vs. job specification vector
- **Workforce analytics companies** — Visier, Workday pay for benchmark datasets
- **Insurance underwriters** — key-person risk underwriting for D&O policies

**Revenue model:** B2B data license. Anonymous, aggregated by role/industry.
The individual profiles stay private; the benchmarks (what does a top-quartile
operations manager look like across these 40 dimensions) are the product.

#### 7.2 Hyperlocal Structured Quality Intelligence

**What it is:** Structured, dimensional community scores for cities, neighborhoods,
restaurants, and venues — with per-dimension benchmark comparisons.

**Why it's valuable:** Google Reviews gives you unstructured text and a star
count. This gives you `internet_speed: 7.3/10 (above global avg), bathroom_clean:
83% (above category bar), wait_time: avg 8 min (well below bar)` — structured,
comparable, benchmarked. This doesn't exist at scale anywhere.

**Who pays:**
- **Government tourism boards** — $25K–$200K/yr for city quality dashboards
  and competitive benchmarking against rival cities
- **Commercial real estate** — site selection for restaurant chains, coworkings;
  pay for neighborhood quality data
- **Remote work platforms** — Deel, Remote.com, Oyster pay for "nomad
  friendliness" city scores for employee relocation tools
- **Airlines and hospitality** — route planning, marketing positioning
- **Real estate portals** — Zillow, Idealista pay for neighborhood quality layers

**Revenue model:** API licensing at city/region/category level. Government
contracts for destination marketing. White-label dashboards.

#### 7.3 Consumer Product Structured Review Data

**What it is:** Structured per-dimension product assessments (longevity, skin
reaction, scent, coverage, price-per-use) collected across product categories,
with skin-type and demographic segmentation from contributor context.

**Why it's valuable:** No one has structured cosmetic/personal care review data
at dimension level. Consumer Reports is the closest analog and it's very limited.

**Who pays:**
- **CPG brands** — competitive intelligence on rivals' products ($10K–$100K/yr)
- **Retailers** — Sephora, ULTA pay for structured product quality signals
  to improve recommendation engines
- **Ingredient safety databases** — EWG, Think Dirty pay for irritation data
  correlated to ingredients
- **Market research firms** — Nielsen, IRI pay for structured primary research
- **Patent attorneys** — cosmetic formulation disputes use consumer data as evidence

**Revenue model:** Per-brand subscription for competitive benchmarking.
Dataset licensing to research firms. Ingredient-level safety signal API.

### Tier 2 — High Value, Moderate Defensibility

#### 7.4 Contractor & Service Provider Performance Longitudinal Records

**What it is:** Per-job dimensional performance data on contractors across
quality, punctuality, cleanup, and communication — with trend direction.

**Who pays:**
- **Insurance underwriters** — contractor liability pricing; poor longitudinal
  quality scores = higher risk = adjustable premium
- **Home warranty companies** — network contractor qualification
- **Background check providers** — Checkr, Sterling Screening pay for
  structured performance history to complement criminal checks
- **Franchise operators** — service quality compliance monitoring

#### 7.5 Venue & Hospitality Quality Time Series

**What it is:** Per-dimension quality scores for hotels, restaurants, coworking
spaces — tracked monthly, with trend direction on each dimension.

**Who pays:**
- **Health departments** — restaurant inspection supplementation; community
  bathroom cleanliness data predicts inspection outcomes
- **Food delivery platforms** — DoorDash, Uber Eats pay for restaurant quality
  signals to improve ranking algorithms
- **Hospitality groups** — multi-location chains pay for competitive benchmarking
  across their own locations and rivals

### Tier 3 — Strategic / Platform Value

#### 7.6 Cross-Category Benchmark Standards

**What it is:** The benchmark reference values themselves — "what does excellent
internet speed mean globally, in Southeast Asia, in Tier 2 cities." The *bar*
data, not the entity scores.

**Why it's valuable:** Once you have enough data to set credible benchmarks,
the benchmarks become a citation source. Academic papers cite them. Governments
reference them. This creates a moat — the platform becomes the authority on
what "good" looks like across domains.

**Who pays:** Indirectly — the benchmark authority drives media coverage,
inbound enterprise inquiries, and data licensing demand.

#### 7.7 Similarity & Matching Intelligence

**What it is:** The dimension vectors and target profiles powering the Combinator —
"entities most similar to X" across cities, people, products, venues.

**Why it's valuable:** Recommendation and matching is one of the highest-value
ML functions. Having pre-built structured vectors is far more valuable than
raw text for this purpose.

**Who pays:**
- **Job boards** — LinkedIn, Indeed pay for structured candidate-to-role match data
- **Travel platforms** — "cities you'd love based on cities you've rated"
- **Dating/network apps** — behavioral compatibility vectors
- **Private equity** — "companies with operational profiles similar to our
  best acquisitions" as a deal sourcing tool

---

## 8. Monetization Strategy

### 8.1 Data Licensing Tiers

```
Free tier (lead generation):
  → Public access to top-level composite scores
  → "Canggu: Nomad Score 7.4/10"
  → "Bob's Contracting: Overall 8.1/10"

Pro tier ($99–$499/mo per tenant):
  → Full dimensional breakdown
  → Per-session history
  → Benchmark comparisons
  → CSV export

Enterprise API ($2K–$50K/mo):
  → Raw dimension data via API
  → Custom benchmark tiers for their category
  → Bulk entity ingestion
  → Webhook on score change

Dataset License ($25K–$500K/yr):
  → Anonymized, aggregated benchmark dataset
  → By category, geography, time period
  → Updated quarterly
  → Exclusive windows for premium buyers
```

### 8.2 The Network Effect Moat

The system is self-reinforcing in a way most SaaS products are not:

1. **More contributors → better benchmarks**
2. **Better benchmarks → more credible ratings**
3. **More credible ratings → more enterprise licensing value**
4. **Licensing revenue → funds more template development**
5. **More templates → more use cases → more contributors**

Once a template (e.g., "Nomad City") reaches ~50 rated entities and ~500
contributors, the benchmark reference values become statistically meaningful.
At that point the dataset has standalone commercial value independent of the
platform that collected it.

### 8.3 Sponsored Placement (Opt-in Revenue)

Entities with verified high scores can pay for featured placement in search
results and comparison tools. Unlike traditional advertising, the placement
is credibility-correlated — a restaurant with a bad bathroom score cannot
buy its way into the "clean venues" filter. This creates an incentive for
quality improvement, not just payment.

---

## 9. Implementation Roadmap

### Phase 1 — Migration & Core Engine (2 weeks)
- [ ] Write migration `m20260701_g27_scorecards.rs`
- [ ] Register in `migration/mod.rs` and `core_platform.rs`
- [ ] Write `ScorecardService` (get_or_create, open_session, submit_entry)
- [ ] Write aggregation engine (rating, absolute, boolean branches)
- [ ] Register `recompute_scorecard_aggregates` background job (5-min interval)

### Phase 2 — Poll Support & Time Series (1 week)
- [ ] Wire poll aggregation branch (poll_single, poll_multi)
- [ ] Populate `atlas_scorecard_poll_aggregates` on recompute
- [ ] Register `refresh_scorecard_time_series` background job (1-hr interval)
- [ ] Trend direction calculation (improving/stable/declining per dimension)

### Phase 3 — The Combinator / Similarity (1 week)
- [ ] `build_dimension_vector()` service function
- [ ] `find_similar()` implementation (Euclidean distance, Rust-side for <10K entities)
- [ ] `atlas_scorecard_targets` + criteria tables
- [ ] Target vector construction from seed entity IDs

### Phase 4 — First App Integration — NomadList (2 weeks)
- [ ] "Nomad City Template" with 25 initial dimensions
- [ ] City scorecard provisioning from `atlas_assets` (asset_type='city')
- [ ] Contributor submission flow with visit context
- [ ] G-06 verification gate for new contributor accounts
- [ ] Public city scorecard page

### Phase 5 — Official Data Integration (1 week)
- [ ] Background job syncing `asset_metadata` from Numbeo API
- [ ] Speedtest/OOKLA integration for internet speed reference values
- [ ] Weather API integration for temperature/humidity current values
- [ ] Return rate computation from reservation session data (G-23)

---

## 10. Rule 7 Generic Fitness Test

| Criterion | Assessment |
|---|---|
| Used by ≥3 apps | ✅ 8 confirmed: NomadList, Bridgewater OS, PM/STR, Direct Booking, Restaurant OS, Beauty Products, Contractor OS, AgentLink |
| Not app-specific behavior | ✅ The engine is pure structure + aggregation math |
| JSONB would be abused otherwise | ✅ Every app would build private `*_ratings` tables |
| Cross-app comparison value | ✅ Templates are portable across apps; vendor can be rated across platforms |
| Salesforce analog | ✅ Salesforce Surveys + Questions + Responses — identical structure |
| Promotes before competing tables exist | ✅ No current active migration duplicating this pattern |

**Verdict: PASSES. Promote to G-27.**

---

## 11. Shared UI Configurator Component

G-27 ships with a shared frontend configurator that any Atlas app can embed
in its admin settings. It is a self-contained component tree requiring only
a `templateId` (or `null` to create new) and an `entityType` label.

### 11.1 Component Architecture

```
<ScorecardConfigurator>              ← root, handles template CRUD
  ├── <TemplateHeader>               ← name, entity_type, scoring_method
  ├── <DimensionList>                ← sortable list of all dimensions
  │     └── <DimensionCard>          ← per-dimension summary + edit button
  ├── <DimensionEditor>              ← slide-over panel for one dimension
  │     ├── <ScaleTypeSelector>      ← rating / absolute / boolean / poll_*
  │     ├── <BenchmarkTierBuilder>   ← visual tier editor (see 11.3)
  │     ├── <ReferenceValueInput>    ← global_reference_value + label
  │     └── <PollOptionsList>        ← visible only for poll_single/poll_multi
  └── <PublishToggle>                ← draft → published gate
```

All components talk to a single `useScorecardTemplate` hook that manages
optimistic state and syncs to the backend API:

```
POST   /api/admin/scorecard-templates               create
PATCH  /api/admin/scorecard-templates/:id           update name/method/settings
POST   /api/admin/scorecard-templates/:id/publish   flip is_published
GET    /api/admin/scorecard-templates/:id/dimensions list dimensions
POST   /api/admin/scorecard-dimensions              add dimension to template
PATCH  /api/admin/scorecard-dimensions/:id          update dimension config
DELETE /api/admin/scorecard-dimensions/:id          remove dimension
POST   /api/admin/scorecard-dimension-options       add poll option
DELETE /api/admin/scorecard-dimension-options/:id   remove poll option
```

### 11.2 Embedding in Any App

Any app drops the configurator in with two props:

```tsx
// In the app's admin settings panel
import { ScorecardConfigurator } from '@atlas/scorecards';

// New template from scratch
<ScorecardConfigurator
  entityType="restaurant"
  label="Restaurant Assessment"
/>

// Edit an existing template
<ScorecardConfigurator
  templateId="f3a1b2c4-..."
  entityType="restaurant"
/>

// Read-only preview (for end-user scorecard display, not editing)
<ScorecardDisplay
  scorecardId="..."
  showTimeSeries={true}
  showComparison={true}
/>
```

The component handles its own routing, state, and API calls. The embedding
app doesn't need to know anything about the underlying tables.

### 11.3 Benchmark Tier Builder — Visual Editor

The benchmark tier builder renders a color-coded ruler that the admin
drags to define tier boundaries. Crucially it works across all three
numeric types and gives immediate visual feedback.

```
Rating dimension (1–10 scale):
┌─────────────────────────────────────────────────────────┐
│  1    2    3    4    5    6    7    8    9    10         │
│  [████████BAD████████|████OKAY████|███GOOD███|GREAT██]  │
│          ↑ drag      ↑ drag       ↑ drag                │
│         3.5         5.0          7.0                    │
│  Tier labels: [Bad] [Okay] [Good] [Great]  + colors     │
└─────────────────────────────────────────────────────────┘

Absolute dimension (0–200 μg/m³ air quality, lower=better):
┌─────────────────────────────────────────────────────────┐
│  0    25    55    100    150    200                      │
│  [EXCELLENT|GOOD|SENSITIVE|UNHEALTHY|STAY INSIDE]       │
│  ✓ "Invert: lower is better"  checkbox                  │
│  ✓ "Show value in label"      checkbox                  │
│  ✓ "Show global reference"    value: 35 μg/m³           │
└─────────────────────────────────────────────────────────┘

Boolean dimension (0–100% true):
┌─────────────────────────────────────────────────────────┐
│  0%          40%          70%         100%              │
│  [MOST SAY DIRTY | MIXED REPORTS | MOST SAY CLEAN]      │
│  ✓ "Invert: lower % is better" checkbox (for irritation)│
└─────────────────────────────────────────────────────────┘
```

The builder serializes this directly to the `benchmark_tiers` JSONB on save.

### 11.4 End-User Submission Widget

The contributor-facing widget adapts its UI to `scale_type`:

```
rating      → horizontal slider 1–10 with snap points + label preview
absolute    → numeric input with unit label + shows where input falls
               on the benchmark tier ruler ("Your value: 45 Mbps → Good")
boolean     → large YES / NO toggle buttons
poll_single → radio card list (one choice)
poll_multi  → checkbox card list (multiple choices)
```

All input types show the benchmark tier the current value resolves to in
real-time, giving contributors immediate context: "you're about to vote
this restaurant's wait time as 'Long wait.'"

The session context inputs (when was your visit, how many days, purpose)
appear as optional but encouraged fields above the dimension grid.

### 11.5 Admin Module Type

G-27 requires a new `AdminModuleType` variant to appear in app admin panels:

```rust
// In models/admin_module.rs — add:
#[sea_orm(string_value = "Scorecards")]
Scorecards,
```

Apps that want the configurator in their admin declare it in `default_modules()`:

```rust
(M::Scorecards, "Scorecards", 45, false),
```

---

## 12. Polymorphic Subject Entities — What Can Be Rated

G-27 is **not limited to any specific entity type**. The `subject_entity_type`
field is a plain `VARCHAR(50)` — the scorecard engine does not care what the
entity is, only that it has a UUID. Any entity in the platform can have a
scorecard attached to it.

### 12.1 Supported Out of the Box

| Entity | Table name | `subject_entity_type` value | Notes |
|---|---|---|---|
| City / Property / Vehicle / Equipment | `atlas_asset` | `'atlas_asset'` | G-10, primary target |
| Marketplace Listing | `listing` | `'listing'` | ✅ existing legacy entity — works now |
| Catalog Entry / Product | `atlas_catalog_entry` | `'atlas_catalog_entry'` | G-26 |
| Person / User | `atlas_account` | `'atlas_account'` | Baseball Card use case |
| Contractor / Agent | `atlas_service_provider` | `'atlas_service_provider'` | G-12 |
| Profile (Business/Individual) | `profile` | `'profile'` | existing legacy entity |
| Customer (CRM) | `customer` | `'customer'` | existing legacy entity |
| Opportunity / Deal | `atlas_opportunity` | `'atlas_opportunity'` | G-15 |
| Portfolio | `atlas_portfolio` | `'atlas_portfolio'` | G-09 |

### 12.2 The `listing` Entity — How It Works Now

The `listing` entity has `properties JSONB` for unstructured attributes and a
`based_on_template_id` that links to the legacy `template` entity. G-27
complements this rather than replacing it:

```
listing (existing)
  ├── properties JSONB         ← unstructured fields (bedrooms, sqft, etc.)
  ├── based_on_template_id     ← legacy template for the listing form
  └── [G-27 scorecard]        ← community ratings via atlas_scorecards
        subject_entity_type = 'listing'
        subject_entity_id   = listing.id
```

A rental listing on a property marketplace gets:
- `properties JSONB`: bedrooms=3, bathrooms=2, parking=true, sqft=1200
- G-27 scorecard: cleanliness=8.1, noise=6.4, host_responsiveness=9.2,
  value=7.8, wifi_speed=6.0, would_return=78%

No migration to the `listing` table needed. The scorecard is created on
first rating submission with `get_or_create(template_id, 'listing', listing.id)`.

### 12.3 Adding a Scorecard to Any New Entity

The pattern is always the same three lines in the submission handler:

```rust
// 1. Get or create the scorecard for this entity
let scorecard_id = ScorecardService::get_or_create(
    db, tenant_id, city_template_id, "listing", listing_id
).await?;

// 2. Open a rating session
let session_id = ScorecardService::open_session(
    db, scorecard_id, rater_user_id, Utc::now(),
    "stay",              // session type
    Some("atlas_reservation"), Some(reservation_id), // optional context link
).await?;

// 3. Submit entries sparsely
ScorecardService::submit_entry(db, session_id, cleanliness_dim_id,
    Some(8.5), None, "community_rating", json!({}), None).await?;
```

---

## 13. Legacy Entity Promotion Audit

Several entities in the current platform are app-specific implementations
of concepts that would pass Rule 7 if promoted to platform generics.
This section identifies the strongest candidates.

### 13.1 `note` → **Candidate: `atlas_note` (G-28)**

**Current state:** `notes` table with `entity_type VARCHAR` + `entity_id UUID`
— already polymorphic. Used on deals, customers, leads, contacts, cases.

**Problem:** Notes are already a generic pattern but live in the legacy layer.
Every app that needs contextual notes (PM, Contractor OS, AgentLink) will
either re-use this table ad-hoc or build their own.

**Promotion case:** Promote to `atlas_note` with `tenant_id` (required, not
nullable), soft-delete, visibility scoping (`visibility: 'private'|'team'|'public'`),
and pinning. This is a clean one-table promotion.

**Estimated effort:** 1 migration, 1 service, update `notes.tenant_id` to NOT NULL.

---

### 13.2 `activity` → **Candidate: `atlas_activity` (G-29)**

**Current state:** An activity/event log table. Likely polymorphic (entity_type,
entity_id). Used to track what happened on a deal, case, or lead.

**Problem:** Every app needs an audit trail / activity feed. AgentLink needs
"call logged", "email sent", "showing scheduled" on each deal. PM needs "lease
renewed", "payment received", "maintenance completed" on each unit.

**Promotion case:** Promote to `atlas_activity` with `activity_type VARCHAR`,
`actor_user_id`, `subject_entity_type/id` (polymorphic), `activity_metadata JSONB`.
This becomes the platform's universal timeline/feed primitive.

**Estimated effort:** 1 migration + backfill, 1 service.

---

### 13.3 `category` → **Candidate: `atlas_category` (G-30)**

**Current state:** Hierarchical categories (`parent_category_id` self-reference,
`is_custom`, `tenant_id` nullable for global categories). Already used for
listings and templates.

**Problem:** Category systems are needed across NomadList (city categories),
Restaurant OS (cuisine types), Clipping (content categories), Job Board
(job categories). Each app would build its own.

**Promotion case:** The `category` table is already 80% correct for promotion.
Main changes: make `tenant_id` NOT NULL (use a platform tenant for global
categories), add `entity_scope VARCHAR` to discriminate which entity type the
category applies to. Rename to `atlas_category`.

**Estimated effort:** 1 migration (rename + add column), update all FK references.

---

### 13.4 `deal` + `lead` → **Migration target: G-15 `atlas_opportunity`**

**Current state:** `deal` (amount, status, stage, close_date) and `lead`
(name, email, listing_id, lead_status, is_converted) are separate tables
that model the same pipeline concept at different stages.

**Observation:** These are not generic promotion candidates — they are **legacy
duplicates** of G-15 `atlas_opportunity` that should be migrated into it.
- `lead` = opportunity at `stage = 'lead'`
- `deal` = opportunity at `stage ∈ ['qualification', 'proposal', 'closed_won']`

**Action:** Write a data migration that moves `lead` and `deal` rows into
`atlas_opportunity`, preserving `properties JSONB` as `opportunity_metadata`.
Keep the old tables as read-only views for backward compatibility during the
transition window.

---

### 13.5 `customer` → **Migration target: `atlas_contact` (G-01 area)**

**Current state:** `customer` has CPF/CNPJ/TIN (Brazilian/international tax IDs),
contact channels (email, phone, WhatsApp, Instagram), `customer_type`, and
`attributes JSON`.

**Observation:** This overlaps heavily with `atlas_contact`. The tax ID fields
(CPF, CNPJ, TIN) are the meaningful delta — `atlas_contact` doesn't have them.

**Action:** Add `tax_id_primary VARCHAR` + `tax_id_secondary VARCHAR` +
`tax_id_type VARCHAR` to `atlas_contact`, then migrate customer rows in.
Do NOT try to add Brazilian-specific fields to the contact generic — use
`contact_metadata JSONB` for locale-specific fields.

---

### 13.6 `profile` — **Keep as App-Specific**

**Current state:** `profile` has `profile_type` (Individual/Business),
`service_area_zips`, `properties JSONB`, and links to `account`.

**Assessment:** This is too tightly coupled to the marketplace/listing flow
to promote cleanly. The `service_area_zips` field is highly specific to
service-area-based businesses. Keep as-is; apps that need a richer profile
concept should use `atlas_service_provider` (G-12) which has similar semantics
and is already generic.

---

### 13.7 Promotion Priority Summary

| Entity | Action | Priority | Effort |
|---|---|---|---|
| `notes` | Promote → `atlas_note` (G-28) | **High** — needed by 6+ active apps | Low (1 migration) |
| `activity` | Promote → `atlas_activity` (G-29) | **High** — universal timeline need | Medium |
| `category` | Promote → `atlas_category` (G-30) | **Medium** — NomadList blocks on this | Low (rename + 1 col) |
| `lead` | Promote → `atlas_lead` (G-31) | **High** — pre-qualification intake, NOT the same as G-15 | Low (1 migration) |
| `deal` | Migrate → G-15 `atlas_opportunity` | **Medium** — deal IS a qualified opportunity | Medium (data migration) |
| `customer` | Migrate → `atlas_contact` | **Low** — no active app competing yet | Medium |
| `profile` | Keep app-specific | N/A | N/A |

> [!IMPORTANT]
> G-28 (`atlas_note`) and G-29 (`atlas_activity`) should be considered
> Round 2 generics alongside G-20, G-21, G-22, G-24. They are lower risk
> than G-27 (simpler tables) and unblock every app that needs contextual
> notes and timeline feeds.

---

## 14. Sales Engagement — G-27 as the CRM Intelligence Layer

G-27 is the dimensional intelligence layer that sits across the full sales
pipeline. It does not replace CRM records — it scores them, tracks them over
time, and makes them comparable and predictable.

### 14.1 Why Lead Stays Separate from `atlas_opportunity`

Lead and opportunity are architecturally distinct objects:

```
lead          → "someone showed up. do we even want to talk to them?"
                 no committed value, no pipeline stage
                 may be disqualified and discarded entirely
                 conversion is a deliberate gate event, not a stage change

atlas_opportunity → "we have decided this is worth pursuing"
                 defined value, defined stage, defined probability
                 someone made an explicit choice to open this
```

The same lead-as-intake pattern recurs across every app:

| App | Lead type |
|---|---|
| AgentLink | Inbound buyer/seller inquiry before agent assignment |
| Contractor OS | Inbound service request before site estimate |
| Insurance brokerage | Inbound quote request before underwriting review |
| PM / STR | Rental inquiry before tenant screening |
| Commercial loan | Borrower inquiry before creditworthiness check |

**Action:** Remove the `lead`→G-15 merge suggestion. Promote `lead` to
`atlas_lead` (G-31) as its own generic. The bridge is:

```sql
-- On atlas_lead:
is_converted            BOOLEAN NOT NULL DEFAULT false,
converted_at            TIMESTAMPTZ,
converted_opportunity_id UUID,   -- FK atlas_opportunity (G-15)
converted_contact_id    UUID,    -- FK atlas_contact (G-01)
disqualified_at         TIMESTAMPTZ,
disqualification_reason TEXT
```

The G-27 lead qualification scorecard (section 14.2) produces the composite
score that drives the conversion decision. High score → rep clicks "Convert."
Low score after N sessions → rep clicks "Disqualify."

---

### 14.2 Use Case 1 — Lead Qualification Scoring

The BANT / MEDDIC qualification framework as a G-27 scorecard:

```
Template: "Lead Qualification"    entity_type='atlas_lead'
Scoring method: weighted_mean

Dimensions:

  budget_confirmed       scale_type='boolean'
                         weight=1.5
                         — "Has the prospect confirmed an allocated budget?"

  decision_authority     scale_type='rating', scale_min=1, scale_max=10
                         weight=2.0
                         benchmark_tiers: [
                           {label:"Economic buyer",  min_score:8.5},
                           {label:"Strong influencer",min_score:6.0},
                           {label:"Influencer only",  min_score:4.0},
                           {label:"No authority",     min_score:0.0}
                         ]

  pain_urgency           scale_type='rating'
                         weight=2.0
                         — "How acute is their problem? Are they actively suffering?"

  timeline_defined       scale_type='boolean'
                         weight=1.0
                         — "Is there a decision date or triggering event?"

  competition_risk       scale_type='rating'
                         weight=1.0     [inverted: lower=better]
                         — "Are we in a competitive evaluation?"

  fit_to_icp             scale_type='rating'
                         weight=2.0
                         — "How well does this lead match our ICP?"

  engagement_quality     scale_type='rating'
                         — "How engaged and responsive have they been?"
```

Each qualification call is a **session**. The rep submits only the dimensions
they learned in that interaction — sparse by design.

```
Lead: Acme Corp (Jane Smith, VP Operations)

  Session 1 — Discovery call (Jan 10)
    source_type='community_rating'  (rep's assessment of the call)
    engagement_quality: 8.0
    pain_urgency: 7.5
    decision_authority: 5.0  ← VP Ops, not CFO — unclear budget authority
    composite: 4.8 / 10 — insufficient data, keep qualifying

  Session 2 — Follow-up call (Jan 17)
    budget_confirmed: true  ← CFO joined the call
    decision_authority: 9.0  ← CFO is the economic buyer
    timeline_defined: true   ← board review March 1
    composite: 7.9 / 10
    → threshold crossed: "Convert to Opportunity?" prompt surfaced

  Session 3 — Demo (Jan 24)
    competition_risk: 7.0   ← Salesforce also in evaluation
    fit_to_icp: 8.5
    composite: 7.6 / 10     ← slight drop on competition risk
    → rep converts anyway, notes competitive threat in opportunity
```

#### The Combinator: Predictive Lead Scoring

After 200+ converted leads have qualification scorecards:

```
New lead arrives: dimension_vector = [8.2, 7.0, 1.0, 6.5, 3.0, 8.0, 7.5]
                  (budget, authority, timeline, urgency, competition, fit, engagement)

ScorecardService::find_similar(
    template_id: lead_qualification_template,
    target_vector: new_lead_vector,
    min_confidence: "medium",
    limit: 20
)
→ Returns: 23 historically similar leads
   → 17 converted (74% conversion rate)
   → avg deal size: $38,200
   → median time-to-close: 47 days
   → top industry match: manufacturing (61%)

Surface to rep: "Based on similar leads, this profile closes ~74% of the time
               at an average of $38K. Prioritize."
```

No black-box ML required. The scorecard vectors are the features. The
historical conversion outcomes are the labels. The Combinator IS the model.

---

### 14.3 Use Case 2 — Deal Health Tracking (Per Pipeline Review)

```
Template: "Opportunity Health"    entity_type='atlas_opportunity'
Session type: 'pipeline_review'   — one per weekly/bi-weekly review

Dimensions:

  champion_strength      scale_type='rating'
                         weight=2.0
                         — internal sponsor who actively fights for the deal

  exec_sponsor_engaged   scale_type='boolean'
                         weight=1.5

  budget_protected       scale_type='boolean'
                         weight=1.5

  timeline_slippage      scale_type='rating'
                         weight=1.0   [inverted: lower=better]
                         benchmark_tiers: [
                           {label:"On track",     max_score:2.0, color:"#00cc44"},
                           {label:"Minor slip",   max_score:5.0, color:"#ffaa00"},
                           {label:"Significant",  max_score:8.0, color:"#ff6600"},
                           {label:"At risk",      max_score:10,  color:"#ff0000"}
                         ]

  competitive_risk       scale_type='rating'
                         weight=1.0   [inverted]

  mutual_action_plan     scale_type='boolean'
                         weight=2.0
                         — clear next steps with dates agreed by both sides

  forecast_confidence    scale_type='absolute'  unit='%'  scale_min=0, scale_max=100
```

Time series reveals deal trajectory the rep's verbal status cannot:

```
Deal: Acme Corp — $42K  (target close Feb 28)

  Review 1 (Jan 15): composite=6.2  trend=—
  Review 2 (Jan 29): composite=7.4  trend='improving'   ← exec joined
  Review 3 (Feb 12): composite=5.1  trend='declining' ⚠️
    timeline_slippage: 8.0   ← close date moved to April
    mutual_action_plan: false ← no confirmed next step
    → ALERT to manager: "Deal health declining — 2 risk signals active"
    → Add to at-risk pipeline report
```

The aggregate across all open deals gives the manager a forecast that is
correlated to deal health scores, not just rep-reported probability percentages.

---

### 14.4 Use Case 3 — Sales Rep Performance (Baseball Card for Sellers)

```
Template: "Sales Rep Performance"  entity_type='atlas_account'
Session type: 'monthly_review' | 'quarterly_review'
Multiple source_types on the same dimension:

Dimensions:

  activity_volume        scale_type='absolute'  unit='touches/wk'
                         source_type='behavioral_signal'  (from G-29 activity log)
                         global_reference_value=25
                         global_reference_label='Team average: 25 touches/wk'

  call_quality           scale_type='rating'
                         source_type='manager_review'  (recorded call review)

  pipeline_coverage      scale_type='absolute'  unit='x'  (pipeline/quota ratio)
                         benchmark_tiers: [
                           {label:"Covered",    min_score:3.5},
                           {label:"Tight",      min_score:2.0},
                           {label:"At risk",    min_score:0}
                         ]
                         global_reference_value=3.5

  close_rate             scale_type='absolute'  unit='%'
  avg_deal_size          scale_type='absolute'  unit='USD'
  forecast_accuracy      scale_type='absolute'  unit='%'
  ramp_to_close          scale_type='absolute'  unit='days'  [inverted]
```

Multiple source types on `call_quality` surface coaching signals:

```
Rep: Marcus T.

  call_quality (manager_review):   7.2  ← manager scored 4 recorded calls
  call_quality (self_assessment):  8.8  ← Marcus thinks he's doing great
  gap: 1.6 points
  → coaching flag: rep is overconfident relative to observed quality
    specifically: low next-step confirmation rate (behavioral_signal)
```

The Combinator for hiring: "We need a new AE. Find external candidates whose
profiles match our top 3 reps." → average the top reps' dimension vectors
→ `find_similar()` against a candidate pool whose assessment scorecards have
been collected during the interview process.

---

### 14.5 Use Case 4 — Touch Quality Scoring (Per Interaction)

The most granular use case: individual call/email/meeting quality, scored
immediately after by the rep or manager. Links to G-29 `atlas_activity`.

```
Template: "Sales Touch Quality"    entity_type='atlas_lead' or 'atlas_opportunity'
Session type: 'call' | 'email_thread' | 'demo' | 'meeting'
context_entity_type='atlas_activity'  ← links to the activity log record

Dimensions:
  prospect_engagement    scale_type='rating'
  pain_confirmed         scale_type='boolean'   — did they articulate the pain?
  objections_surfaced    scale_type='boolean'   — did we learn real concerns?
  next_step_confirmed    scale_type='boolean'   — date + agenda agreed?
  rep_preparation        scale_type='rating'    source_type='manager_review'
```

Across 50+ touches on a deal, patterns emerge:

```
Rep Marcus on Acme deal — touch quality time series:
  prospect_engagement: stable at 7–8  ← prospect is interested
  next_step_confirmed: FALSE on 8/12 touches  ← rep never confirms next step
  → specific coaching target, not generic "needs improvement"
```

---

### 14.6 Full Pipeline — Generics Map

```
┌─────────────────────────────────────────────────────────────────────┐
│  INTAKE                                                             │
│  atlas_lead (G-31)                                                  │
│    ↕ G-27 scorecard: Lead Qualification (BANT/MEDDIC)               │
│      Sessions: each call, demo, discovery meeting                   │
│      Composite ≥ 7.5 → surfaces "Convert" prompt                   │
│      Composite < 3.0 after 3 sessions → surfaces "Disqualify"      │
│      Combinator: predict close rate from historical similar leads   │
├─────────────────────────────────────────────────────────────────────┤
│  QUALIFICATION GATE  (deliberate human decision)                    │
│    → CREATE atlas_contact (G-01)                                    │
│    → CREATE atlas_opportunity (G-15)                                │
│    → lead.is_converted = true                                       │
│    → lead.converted_opportunity_id = opp.id                        │
├─────────────────────────────────────────────────────────────────────┤
│  PIPELINE                                                           │
│  atlas_opportunity (G-15)                                           │
│    ↕ G-27 scorecard: Deal Health                                    │
│      Sessions: each pipeline review (weekly)                        │
│      Time series: composite trend drives forecast confidence        │
│      Alert on 2+ consecutive declining sessions                     │
│                                                                     │
│  atlas_activity (G-29)   ← what actually happened                  │
│    ↕ G-27 scorecard: Touch Quality                                  │
│      Sessions: one per call/email/demo                              │
│      Pattern analysis: which reps confirm next steps?               │
├─────────────────────────────────────────────────────────────────────┤
│  REP PERFORMANCE                                                    │
│  atlas_account (the rep)                                            │
│    ↕ G-27 scorecard: Sales Rep Performance                          │
│      Sessions: monthly/quarterly review                             │
│      Source types: manager_review + self_assessment + behavioral    │
│      Time series: improving/declining by dimension                  │
│      Combinator: match reps to roles, find similar talent           │
└─────────────────────────────────────────────────────────────────────┘
```

### 14.7 What G-27 Does NOT Replace

G-27 is the **intelligence layer**, not the operational layer. These still
belong in their own records:

| Operational data | Belongs in | NOT in G-27 |
|---|---|---|
| Call log (who called when, duration) | `atlas_activity` (G-29) | |
| Email thread content | `atlas_activity` or integration event | |
| Lead contact details | `atlas_lead` (G-31) | |
| Deal amount, close date, stage | `atlas_opportunity` (G-15) | |
| Contract terms | `atlas_contract` (G-08) | |

G-27 scores the quality of what happened. G-29 records that it happened.
G-15 tracks what the deal is worth. They are complementary, not competing.
