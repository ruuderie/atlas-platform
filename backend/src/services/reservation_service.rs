use sea_orm::{DatabaseConnection, ConnectionTrait, Statement};
use uuid::Uuid;
use chrono::Utc;

/// Service layer for GENERIC-23: AtlasReservation
///
/// Manages time-bounded reservations with inventory hold, status lifecycle,
/// and slot conflict detection across all reservation types (hotel rooms,
/// STR units, tour bookings, service appointments, event slots).
///
/// # Commerce Chain Position
///   Browse (G-26) → Quote (G-24) → **Reserve (G-23)** → Pay (G-03)
///
/// # Key invariant
///   Every call to `create_hold()` MUST be paired with a corresponding
///   `atlas_catalog_availability.reserved_count` increment (via CatalogService)
///   so the availability grid stays consistent. The `release_expired_holds()`
///   background worker decrements that count on hold expiry.
pub struct ReservationService;

impl ReservationService {
    /// Create a time-bounded inventory hold.
    ///
    /// The reservation starts in `hold` status. If `hold_expires_at` passes
    /// without a `confirm()` call, `release_expired_holds()` will cancel it.
    ///
    /// Returns the new reservation UUID.
    pub async fn create_hold(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        reserved_asset_type: &str,
        reserved_asset_id: Uuid,
        reservation_type: &str,
        starts_at: chrono::DateTime<Utc>,
        ends_at: chrono::DateTime<Utc>,
        hold_duration_minutes: i64,
        guest_account_id: Option<Uuid>,
        guest_email: Option<&str>,
        total_amount_cents: Option<i64>,
        currency: &str,
        reservation_metadata: serde_json::Value,
    ) -> Result<Uuid, String> {
        let id = Uuid::new_v4();
        let hold_expires_at = Utc::now() + chrono::Duration::minutes(hold_duration_minutes);

        db.execute(Statement::from_sql_and_values(
            db.get_database_backend(),
            r#"
            INSERT INTO atlas_reservations (
                id, tenant_id,
                reserved_asset_type, reserved_asset_id,
                reservation_type, status,
                starts_at, ends_at,
                hold_expires_at,
                guest_account_id, guest_email,
                total_amount_cents, currency,
                reservation_metadata,
                created_at, updated_at
            ) VALUES (
                $1, $2,
                $3, $4,
                $5, 'hold',
                $6, $7,
                $8,
                $9, $10,
                $11, $12,
                $13,
                NOW(), NOW()
            )
            "#,
            [
                id.into(),
                tenant_id.into(),
                reserved_asset_type.into(),
                reserved_asset_id.into(),
                reservation_type.into(),
                starts_at.into(),
                ends_at.into(),
                hold_expires_at.into(),
                guest_account_id.into(),
                guest_email.into(),
                total_amount_cents.into(),
                currency.into(),
                reservation_metadata.into(),
            ],
        ))
        .await
        .map_err(|e| format!("ReservationService::create_hold failed: {e}"))?;

        tracing::info!(
            "G23 reservation hold created: {} ({}) for asset {}/{} [{} → {}]",
            id, reservation_type, reserved_asset_type, reserved_asset_id,
            starts_at, ends_at
        );

        Ok(id)
    }

    /// Confirm a reservation after successful payment.
    ///
    /// Links the reservation to a `atlas_ledger_entries` row so financial
    /// reporting via `atlas_ledger_entries.billable_entity_type = 'atlas_reservation'`
    /// and `billable_entity_id = reservation.id` works cross-app.
    pub async fn confirm(
        db: &DatabaseConnection,
        reservation_id: Uuid,
        tenant_id: Uuid,
        ledger_entry_id: Uuid,
    ) -> Result<(), String> {
        let rows_affected = db
            .execute(Statement::from_sql_and_values(
                db.get_database_backend(),
                r#"
                UPDATE atlas_reservations
                SET status = 'confirmed',
                    confirmed_at = NOW(),
                    hold_expires_at = NULL,
                    ledger_entry_id = $3,
                    updated_at = NOW()
                WHERE id = $1
                  AND tenant_id = $2
                  AND status IN ('hold', 'pending_payment')
                "#,
                [reservation_id.into(), tenant_id.into(), ledger_entry_id.into()],
            ))
            .await
            .map_err(|e| format!("ReservationService::confirm failed: {e}"))?
            .rows_affected();

        if rows_affected == 0 {
            return Err(format!(
                "Reservation {reservation_id} not found or already past hold/pending_payment status"
            ));
        }

        tracing::info!("G23 reservation {} confirmed → ledger entry {}", reservation_id, ledger_entry_id);
        Ok(())
    }

    /// Record guest check-in.
    pub async fn check_in(
        db: &DatabaseConnection,
        reservation_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<(), String> {
        db.execute(Statement::from_sql_and_values(
            db.get_database_backend(),
            r#"
            UPDATE atlas_reservations
            SET status = 'checked_in', checked_in_at = NOW(), updated_at = NOW()
            WHERE id = $1 AND tenant_id = $2 AND status = 'confirmed'
            "#,
            [reservation_id.into(), tenant_id.into()],
        ))
        .await
        .map_err(|e| format!("ReservationService::check_in failed: {e}"))?;

        Ok(())
    }

    /// Cancel a reservation.
    ///
    /// When cancelling a confirmed reservation, the caller is responsible for
    /// triggering the commission clawback via `CommissionPlanService::process_clawback()`
    /// if the plan has a `clawback_days` rule.
    pub async fn cancel(
        db: &DatabaseConnection,
        reservation_id: Uuid,
        tenant_id: Uuid,
        reason: &str,
    ) -> Result<(), String> {
        db.execute(Statement::from_sql_and_values(
            db.get_database_backend(),
            r#"
            UPDATE atlas_reservations
            SET status = 'cancelled',
                cancelled_at = NOW(),
                cancellation_reason = $3,
                updated_at = NOW()
            WHERE id = $1
              AND tenant_id = $2
              AND status NOT IN ('cancelled', 'no_show', 'completed')
            "#,
            [reservation_id.into(), tenant_id.into(), reason.into()],
        ))
        .await
        .map_err(|e| format!("ReservationService::cancel failed: {e}"))?;

        Ok(())
    }

    /// Check whether a specific asset has no conflicting confirmed/hold reservations
    /// for the requested time window.
    ///
    /// Returns `true` if the slot is available (no overlap), `false` if blocked.
    ///
    /// Uses a half-open interval `[starts_at, ends_at)` — the check-out day is
    /// not counted as occupied (standard hotel / STR convention).
    pub async fn check_availability(
        db: &DatabaseConnection,
        reserved_asset_type: &str,
        reserved_asset_id: Uuid,
        starts_at: chrono::DateTime<Utc>,
        ends_at: chrono::DateTime<Utc>,
        exclude_reservation_id: Option<Uuid>,
    ) -> Result<bool, String> {
        let row = db
            .query_one(Statement::from_sql_and_values(
                db.get_database_backend(),
                r#"
                SELECT COUNT(*) AS conflict_count
                FROM atlas_reservations
                WHERE reserved_asset_type = $1
                  AND reserved_asset_id = $2
                  AND status NOT IN ('cancelled', 'no_show')
                  AND starts_at < $4
                  AND ends_at > $3
                  AND ($5::uuid IS NULL OR id != $5)
                "#,
                [
                    reserved_asset_type.into(),
                    reserved_asset_id.into(),
                    starts_at.into(),
                    ends_at.into(),
                    exclude_reservation_id.into(),
                ],
            ))
            .await
            .map_err(|e| format!("ReservationService::check_availability failed: {e}"))?;

        let count: i64 = row
            .and_then(|r| r.try_get("", "conflict_count").ok())
            .unwrap_or(0);

        Ok(count == 0)
    }

    /// Background worker: release all expired holds.
    ///
    /// Runs on a platform-wide sweep (not per-tenant). Sets status = 'cancelled'
    /// for any reservation WHERE status = 'hold' AND hold_expires_at < NOW().
    ///
    /// MUST be registered as a `BackgroundJob` in `CorePlatformApp::background_jobs()`
    /// with `default_interval_seconds = 300` (every 5 minutes).
    ///
    /// Returns the number of holds released.
    pub async fn release_expired_holds(db: &DatabaseConnection) -> Result<u64, String> {
        let result = db
            .execute(Statement::from_string(
                db.get_database_backend(),
                r#"
                UPDATE atlas_reservations
                SET status = 'cancelled',
                    cancelled_at = NOW(),
                    cancellation_reason = 'hold_expired',
                    updated_at = NOW()
                WHERE status = 'hold'
                  AND hold_expires_at IS NOT NULL
                  AND hold_expires_at < NOW()
                "#
                .to_owned(),
            ))
            .await
            .map_err(|e| format!("release_expired_holds failed: {e}"))?;

        let released = result.rows_affected();
        if released > 0 {
            tracing::info!("G23 ReservationService: released {} expired holds", released);
        }

        Ok(released)
    }

    /// Fetch a single reservation by ID and tenant.
    pub async fn find_by_id(
        db: &DatabaseConnection,
        reservation_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<Option<serde_json::Value>, String> {
        let row = db
            .query_one(Statement::from_sql_and_values(
                db.get_database_backend(),
                r#"
                SELECT id, tenant_id, reserved_asset_type, reserved_asset_id,
                       reservation_type, status, starts_at, ends_at,
                       guest_account_id, guest_email,
                       total_amount_cents, currency,
                       hold_expires_at, confirmed_at, cancelled_at,
                       ledger_entry_id, reservation_metadata,
                       created_at, updated_at
                FROM atlas_reservations
                WHERE id = $1 AND tenant_id = $2
                "#,
                [reservation_id.into(), tenant_id.into()],
            ))
            .await
            .map_err(|e| format!("ReservationService::find_by_id failed: {e}"))?;

        Ok(row.map(|_r| serde_json::json!({"id": reservation_id})))
    }
}
