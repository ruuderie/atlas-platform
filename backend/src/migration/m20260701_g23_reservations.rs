use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// GENERIC-23: atlas_reservations — Time-Bounded Reservation with Inventory Hold
///
/// A confirmed or pending booking of an asset (room, seat, time slot, flight,
/// service appointment) for a specific time window, with external hold IDs,
/// slot conflict detection, and a status lifecycle from `hold` → `confirmed`
/// → `completed` / `cancelled`.
///
/// This is the most critical missing generic on the platform. Without it,
/// every app (Direct Booking, Flight+Hotel, Guest Comms, Revenue Manager, PM STR)
/// will create a private reservation table, fragmenting `atlas_ledger_entries`
/// `billable_entity_type` across incompatible schemas.
///
/// Salesforce analog: Field Service Lightning ServiceAppointment (closest).
/// Industry analogs: Cloudbeds PMS reservation, Duffel order, Airbnb reservation.
///
/// Depends on: G-03 (atlas_ledger_entries), G-10 (atlas_assets), G-05 (atlas_external_integrations)
/// Referenced by: G-24 (atlas_quotes) via quote_id → reservation
///
/// BACKGROUND WORKER REQUIREMENT:
///   A `release_expired_holds` worker MUST be registered as a platform-level
///   BackgroundJob that fires every 5 minutes and sets status = 'cancelled'
///   for reservations WHERE status = 'hold' AND hold_expires_at < NOW().
///   This prevents ghost inventory locks after payment abandonment.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // ── Reservation status ENUM ───────────────────────────────────────────
        db.execute(sea_orm::Statement::from_string(
            db.get_database_backend(),
            r#"
            DO $$ BEGIN
                CREATE TYPE atlas_reservation_status AS ENUM (
                    'hold',
                    'pending_payment',
                    'confirmed',
                    'checked_in',
                    'completed',
                    'cancelled',
                    'no_show'
                );
            EXCEPTION WHEN duplicate_object THEN NULL; END $$;
            "#
            .to_owned(),
        ))
        .await?;

        // ── atlas_reservations ────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(AtlasReservations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasReservations::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasReservations::TenantId).uuid().not_null())
                    // What is reserved (polymorphic — the asset being booked)
                    // reserved_asset_type examples:
                    //   'atlas_asset'             — hotel room, STR unit, seat
                    //   'atlas_event_ticket_type' — event ticket (G-21)
                    //   'atlas_catalog_entry'     — catalog item (G-26)
                    .col(ColumnDef::new(AtlasReservations::ReservedAssetType).string_len(50).not_null())
                    .col(ColumnDef::new(AtlasReservations::ReservedAssetId).uuid().not_null())
                    // Who is reserving
                    .col(ColumnDef::new(AtlasReservations::GuestAccountId).uuid().null())
                    .col(ColumnDef::new(AtlasReservations::GuestEmail).string_len(255).null())
                    // Time bounds
                    .col(ColumnDef::new(AtlasReservations::StartsAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(AtlasReservations::EndsAt).timestamp_with_time_zone().not_null())
                    // Reservation type discriminator
                    // Examples: 'hotel_room', 'str_unit', 'flight_seat', 'package',
                    //           'service_appointment', 'event_slot', 'tour_booking'
                    .col(ColumnDef::new(AtlasReservations::ReservationType).string_len(50).not_null())
                    // App-specific metadata (night_count, room_type, pax_count, flight_segments[], etc.)
                    .col(
                        ColumnDef::new(AtlasReservations::ReservationMetadata)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'{}'")),
                    )
                    // External system references (GDS hold ID, PMS folio, Duffel order ID)
                    .col(ColumnDef::new(AtlasReservations::ExternalHoldId).string_len(255).null())
                    .col(ColumnDef::new(AtlasReservations::ExternalConfirmationNo).string_len(255).null())
                    .col(ColumnDef::new(AtlasReservations::PmsIntegrationId).uuid().null()) // FK atlas_external_integrations
                    // Status lifecycle
                    .col(
                        ColumnDef::new(AtlasReservations::Status)
                            .string_len(30)
                            .not_null()
                            .default(Expr::val("hold")),
                    )
                    .col(ColumnDef::new(AtlasReservations::HoldExpiresAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasReservations::ConfirmedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasReservations::CheckedInAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasReservations::CompletedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasReservations::CancelledAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasReservations::CancellationReason).text().null())
                    // Financial link
                    .col(ColumnDef::new(AtlasReservations::LedgerEntryId).uuid().null()) // FK atlas_ledger_entries (G-03)
                    .col(ColumnDef::new(AtlasReservations::QuoteId).uuid().null()) // FK atlas_quotes (G-24) — the proposal
                    .col(ColumnDef::new(AtlasReservations::CommissionPlanId).uuid().null()) // FK atlas_commission_plans (G-25)
                    .col(ColumnDef::new(AtlasReservations::TotalAmountCents).big_integer().null())
                    .col(
                        ColumnDef::new(AtlasReservations::Currency)
                            .char_len(3)
                            .not_null()
                            .default(Expr::val("USD")),
                    )
                    // Attribution
                    .col(ColumnDef::new(AtlasReservations::CampaignEnrollmentId).uuid().null()) // G-19
                    .col(ColumnDef::new(AtlasReservations::AttributionTouchpointId).uuid().null()) // G-20
                    // Timestamps
                    .col(
                        ColumnDef::new(AtlasReservations::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(AtlasReservations::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Index: slot-level conflict detection
        // Query pattern: WHERE reserved_asset_id = $1
        //   AND status NOT IN ('cancelled', 'no_show')
        //   AND starts_at < $end AND ends_at > $start
        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_reservations_asset_dates")
                    .table(AtlasReservations::Table)
                    .col(AtlasReservations::TenantId)
                    .col(AtlasReservations::ReservedAssetType)
                    .col(AtlasReservations::ReservedAssetId)
                    .col(AtlasReservations::StartsAt)
                    .col(AtlasReservations::EndsAt)
                    .to_owned(),
            )
            .await?;

        // Index: guest's reservation history
        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_reservations_guest")
                    .table(AtlasReservations::Table)
                    .col(AtlasReservations::TenantId)
                    .col(AtlasReservations::GuestAccountId)
                    .col(AtlasReservations::Status)
                    .to_owned(),
            )
            .await?;

        // Index: background worker — expired hold sweep
        // Query: WHERE status = 'hold' AND hold_expires_at < NOW()
        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_reservations_hold_expiry")
                    .table(AtlasReservations::Table)
                    .col(AtlasReservations::Status)
                    .col(AtlasReservations::HoldExpiresAt)
                    .to_owned(),
            )
            .await?;

        // Index: reservation type (e.g. all hotel bookings across a tenant)
        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_reservations_type")
                    .table(AtlasReservations::Table)
                    .col(AtlasReservations::TenantId)
                    .col(AtlasReservations::ReservationType)
                    .col(AtlasReservations::StartsAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasReservations::Table).to_owned())
            .await?;

        manager
            .get_connection()
            .execute(sea_orm::Statement::from_string(
                manager.get_connection().get_database_backend(),
                "DROP TYPE IF EXISTS atlas_reservation_status;".to_owned(),
            ))
            .await?;

        Ok(())
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Iden enum
// ══════════════════════════════════════════════════════════════════════════════

#[derive(DeriveIden)]
enum AtlasReservations {
    Table,
    Id,
    TenantId,
    ReservedAssetType,
    ReservedAssetId,
    GuestAccountId,
    GuestEmail,
    StartsAt,
    EndsAt,
    ReservationType,
    ReservationMetadata,
    ExternalHoldId,
    ExternalConfirmationNo,
    PmsIntegrationId,
    Status,
    HoldExpiresAt,
    ConfirmedAt,
    CheckedInAt,
    CompletedAt,
    CancelledAt,
    CancellationReason,
    LedgerEntryId,
    QuoteId,
    CommissionPlanId,
    TotalAmountCents,
    Currency,
    CampaignEnrollmentId,
    AttributionTouchpointId,
    CreatedAt,
    UpdatedAt,
}
