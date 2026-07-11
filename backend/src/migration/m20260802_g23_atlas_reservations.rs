use sea_orm_migration::prelude::*;

/// GENERIC-23: `atlas_reservations` — Time-Bounded Reservation with Inventory Hold
///
/// The canonical reservation primitive for all Atlas apps that deal with time-bounded
/// asset claims: hotel rooms, STR units, equipment rentals, truck parking spots,
/// service appointments, event slots, flight seats.
///
/// # Why this exists
///
/// Four apps (Direct Booking Engine, Flight+Hotel Builder, Multilingual Guest Comms,
/// Folio STR) would each independently build private booking tables (`direct_bookings`,
/// `guest_reservations`, `package_bookings`) that all resolve to `billable_entity_type`
/// in `atlas_ledger_entries`. Once two apps build diverging reservation tables, cross-tenant
/// financial reporting requires a UNION across app-specific schemas.
///
/// This generic eliminates that debt before it accumulates.
///
/// # Reservation lifecycle
///
/// ```text
/// hold → confirmed → checked_in → checked_out
///   └─> cancelled
///   └─> no_show          (guest never arrived)
///   └─> hold_expired     (auto-released after hold_expires_at)
/// ```
///
/// # Conflict detection
///
/// Time slot conflicts are detected with:
/// ```sql
/// SELECT id FROM atlas_reservations
/// WHERE reserved_asset_id = $1
///   AND status NOT IN ('cancelled', 'no_show', 'hold_expired')
///   AND starts_at < $end AND ends_at > $start
/// ```
/// The index `atlas_reservations_asset` covers this query efficiently.
///
/// # Reservation types
///
/// `reservation_type` discriminates the domain:
/// - `'str_unit'`            — STR/Airbnb unit booking (Folio)
/// - `'hotel_room'`          — Hotel room booking (Direct Booking Engine)
/// - `'package'`             — Multi-item travel package (Flight+Hotel Builder)
/// - `'flight_seat'`         — Flight seat (Flight+Hotel Builder via Duffel)
/// - `'equipment_rental'`    — Equipment rental slot (Equipment Rental app)
/// - `'truck_parking'`       — Truck parking spot (Truck Parking app)
/// - `'service_appointment'` — Beauty/hair/contractor appointment
/// - `'event_slot'`          — Ticketed event seat (Events)
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260802_g23_atlas_reservations"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ── Status enum ───────────────────────────────────────────────────────
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TYPE atlas_reservation_status AS ENUM (
                    'hold',
                    'confirmed',
                    'checked_in',
                    'checked_out',
                    'cancelled',
                    'no_show',
                    'hold_expired'
                );",
            )
            .await?;

        // ── Main table ────────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(AtlasReservation::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasReservation::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasReservation::TenantId).uuid().not_null())
                    // Discriminator: 'str_unit', 'hotel_room', 'equipment_rental', etc.
                    .col(
                        ColumnDef::new(AtlasReservation::ReservationType)
                            .string_len(50)
                            .not_null(),
                    )
                    // Polymorphic asset reference (atlas_assets.id, or external).
                    .col(
                        ColumnDef::new(AtlasReservation::ReservedAssetType)
                            .string_len(50)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasReservation::ReservedAssetId)
                            .uuid()
                            .null(),
                    )
                    // Guest / booker — references atlas_accounts.id.
                    .col(
                        ColumnDef::new(AtlasReservation::GuestAccountId)
                            .uuid()
                            .null(),
                    )
                    // Time window — stored as TIMESTAMPTZ for cross-timezone correctness.
                    .col(
                        ColumnDef::new(AtlasReservation::StartsAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasReservation::EndsAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    // Status lifecycle via enum.
                    .col(
                        ColumnDef::new(AtlasReservation::Status)
                            .custom(Alias::new("atlas_reservation_status"))
                            .not_null()
                            .default(Expr::cust("'hold'::atlas_reservation_status")),
                    )
                    // External OTA hold ID (Airbnb reservation ID, Vrbo code, Duffel hold).
                    .col(
                        ColumnDef::new(AtlasReservation::ExternalHoldId)
                            .string_len(200)
                            .null(),
                    )
                    // Auto-release deadline — reservation transitions to 'hold_expired' after this.
                    .col(
                        ColumnDef::new(AtlasReservation::HoldExpiresAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    // Financial summary (line-item detail lives in atlas_quotes G24 or reservation_metadata).
                    .col(
                        ColumnDef::new(AtlasReservation::TotalPriceCents)
                            .big_integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasReservation::NightlyRateCents)
                            .big_integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasReservation::Currency)
                            .string_len(3)
                            .null(),
                    )
                    // App-specific payload: guest_count, ota_platform, channel, notes, etc.
                    .col(
                        ColumnDef::new(AtlasReservation::ReservationMetadata)
                            .json_binary()
                            .not_null()
                            .default(Expr::cust("'{}'")),
                    )
                    // FK to atlas_quotes (G24) — the quote that became this booking. Nullable.
                    .col(ColumnDef::new(AtlasReservation::QuoteId).uuid().null())
                    // FK to atlas_ledger_entries (G03) — set after payment is collected.
                    .col(
                        ColumnDef::new(AtlasReservation::LedgerEntryId)
                            .uuid()
                            .null(),
                    )
                    // Timestamps.
                    .col(
                        ColumnDef::new(AtlasReservation::ConfirmedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasReservation::CancelledAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasReservation::CancellationReason)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasReservation::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("NOW()")),
                    )
                    .col(
                        ColumnDef::new(AtlasReservation::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::cust("NOW()")),
                    )
                    .to_owned(),
            )
            .await?;

        // ── Constraint: ends_at must be after starts_at ────────────────────
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE atlas_reservations
                 ADD CONSTRAINT atlas_reservations_valid_window
                 CHECK (ends_at > starts_at);",
            )
            .await?;

        // ── Indexes ───────────────────────────────────────────────────────────

        // Primary conflict-detection index: asset + time range overlap queries.
        manager
            .create_index(
                Index::create()
                    .name("atlas_reservations_asset")
                    .table(AtlasReservation::Table)
                    .col(AtlasReservation::TenantId)
                    .col(AtlasReservation::ReservedAssetType)
                    .col(AtlasReservation::ReservedAssetId)
                    .col(AtlasReservation::StartsAt)
                    .col(AtlasReservation::EndsAt)
                    .to_owned(),
            )
            .await?;

        // Guest-facing lookup: all reservations for a guest.
        manager
            .create_index(
                Index::create()
                    .name("atlas_reservations_guest")
                    .table(AtlasReservation::Table)
                    .col(AtlasReservation::TenantId)
                    .col(AtlasReservation::GuestAccountId)
                    .col(AtlasReservation::Status)
                    .to_owned(),
            )
            .await?;

        // Hold expiry sweep: find all un-expired holds for the background job.
        manager
            .create_index(
                Index::create()
                    .name("atlas_reservations_hold_expiry")
                    .table(AtlasReservation::Table)
                    .col(AtlasReservation::TenantId)
                    .col(AtlasReservation::Status)
                    .col(AtlasReservation::HoldExpiresAt)
                    .to_owned(),
            )
            .await?;

        // Reservation type + date range: list all STR unit bookings in a date range.
        manager
            .create_index(
                Index::create()
                    .name("atlas_reservations_type_dates")
                    .table(AtlasReservation::Table)
                    .col(AtlasReservation::TenantId)
                    .col(AtlasReservation::ReservationType)
                    .col(AtlasReservation::StartsAt)
                    .col(AtlasReservation::EndsAt)
                    .to_owned(),
            )
            .await?;

        // Ledger FK lookup: find reservation from a ledger entry.
        manager
            .create_index(
                Index::create()
                    .name("atlas_reservations_ledger")
                    .table(AtlasReservation::Table)
                    .col(AtlasReservation::LedgerEntryId)
                    .to_owned(),
            )
            .await?;

        // updated_at trigger for optimistic concurrency.
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE OR REPLACE FUNCTION set_updated_at_atlas_reservations()
                 RETURNS TRIGGER LANGUAGE plpgsql AS $$
                 BEGIN
                     NEW.updated_at = NOW();
                     RETURN NEW;
                 END;
                 $$;

                 CREATE TRIGGER trg_atlas_reservations_updated_at
                 BEFORE UPDATE ON atlas_reservations
                 FOR EACH ROW EXECUTE FUNCTION set_updated_at_atlas_reservations();",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP TRIGGER IF EXISTS trg_atlas_reservations_updated_at ON atlas_reservations;
                 DROP FUNCTION IF EXISTS set_updated_at_atlas_reservations();",
            )
            .await?;

        manager
            .drop_table(Table::drop().table(AtlasReservation::Table).to_owned())
            .await?;

        manager
            .get_connection()
            .execute_unprepared("DROP TYPE IF EXISTS atlas_reservation_status;")
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
pub enum AtlasReservation {
    // Explicit override: default Iden for enum `AtlasReservation` would produce
    // `atlas_reservation` (singular). All raw SQL and index names in this migration
    // use `atlas_reservations` (plural), so we pin the table name here.
    #[iden = "atlas_reservations"]
    Table,
    Id,
    TenantId,
    ReservationType,
    ReservedAssetType,
    ReservedAssetId,
    GuestAccountId,
    StartsAt,
    EndsAt,
    Status,
    ExternalHoldId,
    HoldExpiresAt,
    TotalPriceCents,
    NightlyRateCents,
    Currency,
    ReservationMetadata,
    QuoteId,
    LedgerEntryId,
    ConfirmedAt,
    CancelledAt,
    CancellationReason,
    CreatedAt,
    UpdatedAt,
}
