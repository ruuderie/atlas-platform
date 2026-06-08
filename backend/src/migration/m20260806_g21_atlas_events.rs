//! # Migration: G21 `atlas_events` — Event Management, Ticketing & Check-In
//!
//! ## Tables created
//!
//! | Table | Purpose |
//! |-------|---------|
//! | `atlas_events` | Event definition — type, venue, capacity, schedule, campaign link |
//! | `atlas_event_ticket_types` | Ticket tiers per event (free, paid, VIP) |
//! | `atlas_event_registrations` | Individual attendee registration + QR check-in token |
//!
//! ## Design
//!
//! ### Campaign linkage
//! `atlas_events.campaign_id` → `atlas_campaigns(id)` connects every event to
//! the campaign that promoted it, enabling full-funnel attribution:
//! Campaign → Event → Registration → Conversion.
//!
//! ### Polymorphic subject entity
//! `subject_entity_type` + `subject_entity_id` links the event to any platform
//! entity — most commonly `atlas_assets` (open house at a managed property) or
//! `atlas_opportunities` (showing tied to a deal).
//!
//! ### QR check-in
//! `atlas_event_registrations.check_in_token` is a 32-byte random hex string
//! generated at insert time. The `EventService::check_in(token)` method resolves
//! it to a registration, transitions status to `CheckedIn`, and increments the
//! event's `attended_count`.
//!
//! ### G03 ledger integration
//! `atlas_event_registrations.ledger_entry_id` → `atlas_ledger_entries(id)` ties
//! paid ticket revenue to the platform's general ledger for financial reporting.
//!
//! ### G20 attribution integration
//! `atlas_event_registrations.attribution_touchpoint_id` links the registration
//! to the specific G20 touchpoint that drove the attendee to register, enabling
//! event-driven attribution.

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260806_g21_atlas_events"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        use sea_orm::ConnectionTrait;

        // ── atlas_events ──────────────────────────────────────────────────────

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("atlas_events"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).uuid().primary_key().not_null())
                    .col(ColumnDef::new(Alias::new("tenant_id")).uuid().not_null())
                    // ── Identity ──────────────────────────────────────────────
                    .col(ColumnDef::new(Alias::new("name")).string_len(255).not_null())
                    // URL-safe slug for public event pages (unique per tenant)
                    .col(ColumnDef::new(Alias::new("slug")).string_len(255).null())
                    // VARCHAR — validated as `EventType` enum at service layer
                    .col(ColumnDef::new(Alias::new("event_type")).string_len(50).not_null())
                    // VARCHAR — validated as `EventStatus` enum at service layer
                    .col(
                        ColumnDef::new(Alias::new("status"))
                            .string_len(30)
                            .not_null()
                            .default("draft"),
                    )
                    // ── Location ──────────────────────────────────────────────
                    .col(
                        ColumnDef::new(Alias::new("is_virtual"))
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    // URL for Zoom, Google Meet, Hopin, etc.
                    .col(ColumnDef::new(Alias::new("virtual_url")).string_len(512).null())
                    .col(ColumnDef::new(Alias::new("venue_name")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("venue_address")).text().null())
                    // FK to atlas_assets if event is at a managed property
                    .col(ColumnDef::new(Alias::new("venue_asset_id")).uuid().null())
                    // ── Capacity ──────────────────────────────────────────────
                    .col(ColumnDef::new(Alias::new("max_capacity")).integer().null())
                    .col(
                        ColumnDef::new(Alias::new("waitlist_enabled"))
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    // ── Schedule ──────────────────────────────────────────────
                    .col(
                        ColumnDef::new(Alias::new("starts_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("ends_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("registration_opens_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("registration_closes_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    // ── Campaign linkage ──────────────────────────────────────
                    .col(ColumnDef::new(Alias::new("campaign_id")).uuid().null())
                    // ── Subject entity (polymorphic FK) ───────────────────────
                    .col(ColumnDef::new(Alias::new("subject_entity_type")).string_len(100).null())
                    .col(ColumnDef::new(Alias::new("subject_entity_id")).uuid().null())
                    // ── Visibility ────────────────────────────────────────────
                    .col(
                        ColumnDef::new(Alias::new("is_public"))
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    // ── Computed counters (updated by EventService) ───────────
                    .col(
                        ColumnDef::new(Alias::new("registered_count"))
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Alias::new("attended_count"))
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Alias::new("revenue_cents"))
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    // ── Audit ─────────────────────────────────────────────────
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("NOW()")),
                    )
                    .col(
                        ColumnDef::new(Alias::new("updated_at"))
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
                "ALTER TABLE atlas_events
                 ADD CONSTRAINT atlas_events_tenant_fk
                 FOREIGN KEY (tenant_id) REFERENCES tenant(id) ON DELETE CASCADE,
                 ADD CONSTRAINT atlas_events_campaign_fk
                 FOREIGN KEY (campaign_id) REFERENCES atlas_campaigns(id) ON DELETE SET NULL,
                 ADD CONSTRAINT atlas_events_slug_tenant_unique
                 UNIQUE (tenant_id, slug);

                 CREATE INDEX atlas_events_tenant_status_starts
                     ON atlas_events(tenant_id, status, starts_at);
                 CREATE INDEX atlas_events_campaign
                     ON atlas_events(campaign_id)
                     WHERE campaign_id IS NOT NULL;
                 CREATE INDEX atlas_events_subject
                     ON atlas_events(tenant_id, subject_entity_type, subject_entity_id)
                     WHERE subject_entity_type IS NOT NULL;",
            )
            .await?;

        // ── atlas_event_ticket_types ──────────────────────────────────────────

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("atlas_event_ticket_types"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).uuid().primary_key().not_null())
                    .col(ColumnDef::new(Alias::new("event_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("tenant_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("name")).string_len(100).not_null())
                    // 0 = free ticket
                    .col(
                        ColumnDef::new(Alias::new("price_cents"))
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Alias::new("currency"))
                            .char_len(3)
                            .not_null()
                            .default("USD"),
                    )
                    // NULL = unlimited
                    .col(ColumnDef::new(Alias::new("quantity_available")).integer().null())
                    .col(
                        ColumnDef::new(Alias::new("quantity_sold"))
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Alias::new("is_active"))
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE atlas_event_ticket_types
                 ADD CONSTRAINT atlas_ticket_types_event_fk
                 FOREIGN KEY (event_id) REFERENCES atlas_events(id) ON DELETE CASCADE,
                 ADD CONSTRAINT atlas_ticket_types_tenant_fk
                 FOREIGN KEY (tenant_id) REFERENCES tenant(id) ON DELETE CASCADE;

                 CREATE INDEX atlas_ticket_types_event
                     ON atlas_event_ticket_types(event_id, is_active);",
            )
            .await?;

        // ── atlas_event_registrations ─────────────────────────────────────────

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("atlas_event_registrations"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).uuid().primary_key().not_null())
                    .col(ColumnDef::new(Alias::new("event_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("ticket_type_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("tenant_id")).uuid().not_null())
                    // ── Attendee identity ─────────────────────────────────────
                    .col(ColumnDef::new(Alias::new("attendee_email")).string_len(255).not_null())
                    .col(ColumnDef::new(Alias::new("attendee_name")).string_len(200).null())
                    // Optional FK to platform user (if registered attendee has an account)
                    .col(ColumnDef::new(Alias::new("attendee_user_id")).uuid().null())
                    // ── Booking ───────────────────────────────────────────────
                    .col(ColumnDef::new(Alias::new("quantity")).integer().not_null().default(1))
                    // FK to atlas_ledger_entries (G03) for paid tickets
                    .col(ColumnDef::new(Alias::new("ledger_entry_id")).uuid().null())
                    // ── Check-in ──────────────────────────────────────────────
                    // 32-byte random hex, unique — used for QR code generation
                    .col(
                        ColumnDef::new(Alias::new("check_in_token"))
                            .string_len(128)
                            .not_null()
                            .unique_key(),
                    )
                    // VARCHAR — validated as `RegistrationStatus` at service layer
                    .col(
                        ColumnDef::new(Alias::new("status"))
                            .string_len(30)
                            .not_null()
                            .default("pending_payment"),
                    )
                    .col(
                        ColumnDef::new(Alias::new("confirmed_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("checked_in_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    // FK to atlas_attribution_touchpoints (G20)
                    .col(ColumnDef::new(Alias::new("attribution_touchpoint_id")).uuid().null())
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
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
                "ALTER TABLE atlas_event_registrations
                 ADD CONSTRAINT atlas_registrations_event_fk
                 FOREIGN KEY (event_id) REFERENCES atlas_events(id) ON DELETE CASCADE,
                 ADD CONSTRAINT atlas_registrations_ticket_type_fk
                 FOREIGN KEY (ticket_type_id) REFERENCES atlas_event_ticket_types(id),
                 ADD CONSTRAINT atlas_registrations_tenant_fk
                 FOREIGN KEY (tenant_id) REFERENCES tenant(id) ON DELETE CASCADE,
                 ADD CONSTRAINT atlas_registrations_attribution_fk
                 FOREIGN KEY (attribution_touchpoint_id)
                     REFERENCES atlas_attribution_touchpoints(id) ON DELETE SET NULL;

                 -- Populate check_in_token with a secure random default via trigger or app layer.
                 -- The application generates the token; this index enables fast QR lookup.
                 CREATE UNIQUE INDEX atlas_registrations_check_in_token
                     ON atlas_event_registrations(check_in_token);
                 CREATE INDEX atlas_registrations_event_status
                     ON atlas_event_registrations(event_id, status);
                 CREATE INDEX atlas_registrations_attendee_email
                     ON atlas_event_registrations(tenant_id, attendee_email);",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        use sea_orm::ConnectionTrait;
        manager
            .get_connection()
            .execute_unprepared(
                "DROP TABLE IF EXISTS atlas_event_registrations CASCADE;
                 DROP TABLE IF EXISTS atlas_event_ticket_types CASCADE;
                 DROP TABLE IF EXISTS atlas_events CASCADE;",
            )
            .await?;
        Ok(())
    }
}
