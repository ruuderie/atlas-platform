//! # Migration: G20 `atlas_attribution` — Multi-Channel Attribution Touchpoints
//!
//! ## Tables created
//!
//! | Table | Purpose |
//! |-------|---------|
//! | `atlas_attribution_touchpoints` | Every marketing interaction on the path to a conversion |
//!
//! ## Design
//!
//! ### Identity resolution
//! A visitor arrives anonymously (`anonymous_id` = client cookie/fingerprint).
//! When they sign in or submit a form, `AttributionService::resolve_identity()`
//! back-fills all prior touchpoints for that `anonymous_id` with the resolved
//! `user_id`. This is the industry-standard approach (Segment, Rudderstack,
//! Mixpanel all do the same).
//!
//! ### Attribution window
//! The FK to `atlas_campaigns` and `atlas_campaign_enrollments` allows tracing
//! which campaign drove a touchpoint. `atlas_campaigns.attribution_window_days`
//! is respected by `record_conversion()` when deciding which touchpoints
//! to credit.
//!
//! ### Model-specific credit distribution
//! `record_conversion()` matches on `AttributionModel` (enum) and applies
//! a different credit algorithm per variant — compiler-enforced exhaustiveness
//! guarantees no model falls through to a default.
//!
//! ### Indexes
//! Three covering indexes for the three main read patterns:
//!   - User touchpoint timeline (user_id + occurred_at)
//!   - Anonymous session lookup (anonymous_id + occurred_at)
//!   - Conversion path lookup (conversion_entity_type + conversion_entity_id)

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260805_g20_atlas_attribution"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        use sea_orm::ConnectionTrait;

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("atlas_attribution_touchpoints"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .uuid()
                            .primary_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("tenant_id")).uuid().not_null())
                    // ── Visitor identity (one or more populated) ──────────────
                    // Resolved user (set immediately if logged in, or on identity resolution)
                    .col(ColumnDef::new(Alias::new("user_id")).uuid().null())
                    // External contact (email captured from form fill before login)
                    .col(
                        ColumnDef::new(Alias::new("contact_email"))
                            .string_len(255)
                            .null(),
                    )
                    // Client-side cookie / device fingerprint — resolved to user_id later
                    .col(
                        ColumnDef::new(Alias::new("anonymous_id"))
                            .string_len(128)
                            .null(),
                    )
                    // ── Channel ───────────────────────────────────────────────
                    // VARCHAR — validated as `AttributionChannel` enum at service layer
                    // 'organic_search' | 'paid_search' | 'paid_social' | 'cold_email' | ...
                    .col(
                        ColumnDef::new(Alias::new("channel"))
                            .string_len(50)
                            .not_null(),
                    )
                    // ── UTM parameters ────────────────────────────────────────
                    .col(
                        ColumnDef::new(Alias::new("utm_source"))
                            .string_len(100)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("utm_medium"))
                            .string_len(100)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("utm_campaign"))
                            .string_len(100)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("utm_content"))
                            .string_len(100)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("utm_term"))
                            .string_len(100)
                            .null(),
                    )
                    // ── Platform entity references ────────────────────────────
                    // Which campaign generated this touchpoint (if via a tracked campaign)
                    .col(ColumnDef::new(Alias::new("campaign_id")).uuid().null())
                    // Which enrollment in that campaign (if via a sequence step)
                    .col(ColumnDef::new(Alias::new("enrollment_id")).uuid().null())
                    // Which event (if via G21 event registration)
                    .col(ColumnDef::new(Alias::new("event_id")).uuid().null())
                    // ── Conversion (set when touchpoint is credited) ──────────
                    // What was converted ('atlas_reservations', 'atlas_applications', etc.)
                    .col(
                        ColumnDef::new(Alias::new("conversion_entity_type"))
                            .string_len(100)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("conversion_entity_id"))
                            .uuid()
                            .null(),
                    )
                    // GMV of the conversion in cents
                    .col(
                        ColumnDef::new(Alias::new("conversion_value_cents"))
                            .big_integer()
                            .null(),
                    )
                    // Credit allocated to this touchpoint by the attribution model
                    .col(
                        ColumnDef::new(Alias::new("attributed_revenue_cents"))
                            .big_integer()
                            .null(),
                    )
                    // VARCHAR — validated as `AttributionModel` enum at service layer
                    // 'first_touch' | 'last_touch' | 'linear' | 'time_decay' | 'position_based'
                    .col(
                        ColumnDef::new(Alias::new("attribution_model"))
                            .string_len(30)
                            .null()
                            .default("last_touch"),
                    )
                    // ── Visit context ─────────────────────────────────────────
                    .col(ColumnDef::new(Alias::new("landing_page_url")).text().null())
                    .col(ColumnDef::new(Alias::new("referrer_url")).text().null())
                    .col(
                        ColumnDef::new(Alias::new("occurred_at"))
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("NOW()")),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE atlas_attribution_touchpoints
                 ADD CONSTRAINT atlas_attribution_tenant_fk
                 FOREIGN KEY (tenant_id) REFERENCES tenant(id) ON DELETE CASCADE,
                 ADD CONSTRAINT atlas_attribution_campaign_fk
                 FOREIGN KEY (campaign_id) REFERENCES atlas_campaigns(id) ON DELETE SET NULL,
                 ADD CONSTRAINT atlas_attribution_enrollment_fk
                 FOREIGN KEY (enrollment_id) REFERENCES atlas_campaign_enrollments(id) ON DELETE SET NULL;

                 -- Per-user chronological touchpoint timeline
                 CREATE INDEX atlas_attribution_user
                     ON atlas_attribution_touchpoints(tenant_id, user_id, occurred_at DESC)
                     WHERE user_id IS NOT NULL;

                 -- Anonymous session lookup (for identity resolution)
                 CREATE INDEX atlas_attribution_anon
                     ON atlas_attribution_touchpoints(tenant_id, anonymous_id, occurred_at DESC)
                     WHERE anonymous_id IS NOT NULL;

                 -- Conversion path lookup — 'show me every touchpoint that led to booking X'
                 CREATE INDEX atlas_attribution_conversion
                     ON atlas_attribution_touchpoints(tenant_id, conversion_entity_type, conversion_entity_id)
                     WHERE conversion_entity_id IS NOT NULL;

                 -- Campaign-level touchpoint aggregation
                 CREATE INDEX atlas_attribution_campaign
                     ON atlas_attribution_touchpoints(campaign_id, occurred_at DESC)
                     WHERE campaign_id IS NOT NULL;",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        use sea_orm::ConnectionTrait;
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS atlas_attribution_touchpoints CASCADE;")
            .await?;
        Ok(())
    }
}
