//! PM — Reservation Service (STR unit bookings).
//!
//! Manages the full STR booking lifecycle against `atlas_reservations` (G-23).
//!
//! # Lifecycle
//!
//! ```text
//! create_hold  →  confirm  →  check_in  →  check_out
//!      │              │
//!      └─ cancel      └─ cancel
//! ```
//!
//! A background job (`pm_str_hold_expiry_sweeper`) calls `expire_stale_holds()` every
//! 5 minutes to auto-release holds past their `hold_expires_at` deadline.
//!
//! # Conflict detection
//!
//! `create_hold` runs a conflict guard before insertion:
//! ```sql
//! SELECT EXISTS (
//!   SELECT 1 FROM atlas_reservations
//!   WHERE reserved_asset_id = $asset_id
//!     AND status NOT IN ('cancelled', 'no_show', 'hold_expired')
//!     AND starts_at < $ends_at
//!     AND ends_at > $starts_at
//! )
//! ```
//! Returns [`ReservationError::Conflict`] on overlap without inserting.
//!
//! # OTA platform sync
//!
//! `external_hold_id` stores the OTA reservation code (Airbnb, Vrbo, Booking.com).
//! The `pm_ota_calendar_sync` background job (Phase 7) reads this field to push
//! calendar blocks back to OTA channels.
//!
//! # TDT tax
//!
//! `confirm()` calls `PmTaxService::record_ota_revenue_full()` with the booking's
//! gross amount to record the Tourist Development Tax obligation.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use serde_json::json;
use uuid::Uuid;

use crate::entities::atlas_reservation;
use crate::types::pm::OtaPlatform;

// ── Public input types ────────────────────────────────────────────────────────

/// Input for creating an STR booking hold.
pub struct CreateHoldInput {
    /// The `atlas_assets` id for the STR unit.
    pub asset_id: Uuid,
    /// FK to `atlas_accounts` — the guest's account.
    pub guest_account_id: Uuid,
    /// Inclusive check-in timestamp (guest's local midnight in UTC preferred).
    pub check_in: DateTime<Utc>,
    /// Exclusive check-out timestamp.
    pub check_out: DateTime<Utc>,
    /// Nightly rate in minor currency units.
    pub nightly_rate_cents: i64,
    /// ISO 4217 currency code.
    pub currency: String,
    /// Guest headcount — stored in `reservation_metadata`.
    pub guest_count: u32,
    /// OTA platform the booking came through.
    pub ota_platform: OtaPlatform,
    /// External OTA reservation code (Airbnb HMJKX9, Vrbo ABC123, etc.).
    pub external_hold_id: Option<String>,
    /// How long to hold the slot before auto-releasing (default: 30 minutes).
    pub hold_minutes: Option<u32>,
    /// Jurisdiction code for STR permit validation.
    pub jurisdiction_code: String,
}

/// Summary returned from most reservation operations.
#[derive(Debug)]
pub struct ReservationSummary {
    pub id: Uuid,
    pub status: String,
    pub check_in: DateTime<Utc>,
    pub check_out: DateTime<Utc>,
    pub total_price_cents: i64,
    pub currency: String,
    pub hold_expires_at: Option<DateTime<Utc>>,
}

// ── Error types ───────────────────────────────────────────────────────────────

/// Typed reservation errors returned from service methods.
#[derive(Debug)]
pub enum ReservationError {
    Conflict { asset_id: Uuid },
    TerminalState { id: Uuid, status: String },
    NotFound { id: Uuid, tenant_id: Uuid },
    InvalidWindow { check_in: DateTime<Utc>, check_out: DateTime<Utc> },
    Db(sea_orm::DbErr),
    Other(anyhow::Error),
}

impl std::fmt::Display for ReservationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReservationError::Conflict { asset_id } => {
                write!(f, "Time slot conflict: asset {asset_id} is already booked")
            }
            ReservationError::TerminalState { id, status } => {
                write!(f, "Reservation {id} is in terminal state '{status}'")
            }
            ReservationError::NotFound { id, tenant_id } => {
                write!(f, "Reservation {id} not found for tenant {tenant_id}")
            }
            ReservationError::InvalidWindow { check_in, check_out } => {
                write!(f, "Invalid window: check_in {check_in} >= check_out {check_out}")
            }
            ReservationError::Db(e) => write!(f, "DB error: {e}"),
            ReservationError::Other(e) => write!(f, "{e:#}"),
        }
    }
}

impl std::error::Error for ReservationError {}

impl From<sea_orm::DbErr> for ReservationError {
    fn from(e: sea_orm::DbErr) -> Self {
        ReservationError::Db(e)
    }
}

impl From<anyhow::Error> for ReservationError {
    fn from(e: anyhow::Error) -> Self {
        ReservationError::Other(e)
    }
}

// ── Service ───────────────────────────────────────────────────────────────────

pub struct ReservationService;

impl ReservationService {
    // ── Write operations ──────────────────────────────────────────────────────

    /// Create a time-bounded hold on an STR unit.
    ///
    /// Runs conflict detection before inserting. Returns [`ReservationError::Conflict`]
    /// if any active reservation overlaps the requested window.
    pub async fn create_hold(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        input: CreateHoldInput,
    ) -> Result<Uuid, ReservationError> {
        // Basic window validation.
        if input.check_in >= input.check_out {
            return Err(ReservationError::InvalidWindow {
                check_in: input.check_in,
                check_out: input.check_out,
            });
        }

        // Conflict detection — overlapping active reservations on the same asset.
        let conflict_exists = Self::has_conflict(db, input.asset_id, input.check_in, input.check_out).await?;
        if conflict_exists {
            tracing::warn!(
                %tenant_id, asset_id = %input.asset_id,
                "create_hold: time slot conflict detected"
            );
            return Err(ReservationError::Conflict { asset_id: input.asset_id });
        }

        // Compute derived fields.
        let nights = (input.check_out - input.check_in).num_days().max(1);
        let total_price_cents = input.nightly_rate_cents * nights;
        let hold_minutes = input.hold_minutes.unwrap_or(30) as i64;
        let hold_expires_at = Utc::now() + chrono::Duration::minutes(hold_minutes);

        // Build metadata.
        let metadata = json!({
            "guest_count": input.guest_count,
            "ota_platform": input.ota_platform.to_string(),
            "nights": nights,
            "jurisdiction_code": input.jurisdiction_code,
        });

        let id = Uuid::new_v4();

        let active = atlas_reservation::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_id),
            reservation_type: Set("str_unit".to_string()),
            reserved_asset_type: Set(Some("atlas_assets".to_string())),
            reserved_asset_id: Set(Some(input.asset_id)),
            guest_account_id: Set(Some(input.guest_account_id)),
            starts_at: Set(input.check_in),
            ends_at: Set(input.check_out),
            status: Set("hold".to_string()),
            external_hold_id: Set(input.external_hold_id),
            hold_expires_at: Set(Some(hold_expires_at)),
            total_price_cents: Set(Some(total_price_cents)),
            nightly_rate_cents: Set(Some(input.nightly_rate_cents)),
            currency: Set(Some(input.currency)),
            reservation_metadata: Set(metadata),
            quote_id: Set(None),
            ledger_entry_id: Set(None),
            confirmed_at: Set(None),
            cancelled_at: Set(None),
            cancellation_reason: Set(None),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        };

        active.insert(db).await?;

        tracing::info!(%id, %tenant_id, asset_id = %input.asset_id, "create_hold: STR hold created");
        Ok(id)
    }

    /// Confirm a held reservation.
    ///
    /// Transitions `hold` → `confirmed`. Sets `confirmed_at` timestamp.
    /// Records Tourist Development Tax obligation via `PmTaxService`.
    pub async fn confirm(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        reservation_id: Uuid,
    ) -> Result<ReservationSummary, ReservationError> {
        let reservation = Self::fetch_owned(db, tenant_id, reservation_id).await?;

        match reservation.status.as_str() {
            "hold" => {},
            "confirmed" => {
                // Idempotent — already confirmed.
                return Ok(to_summary(reservation));
            }
            s => {
                return Err(ReservationError::TerminalState {
                    id: reservation_id,
                    status: s.to_string(),
                });
            }
        }

        let mut active: atlas_reservation::ActiveModel = reservation.clone().into();
        active.status = Set("confirmed".to_string());
        active.confirmed_at = Set(Some(Utc::now()));
        active.hold_expires_at = Set(None); // Clear hold expiry on confirm.

        let updated = active.update(db).await?;

        // Record TDT / Tourist Development Tax obligation.
        // This is best-effort — a failure here does not roll back the confirmation.
        if let Some(total_cents) = updated.total_price_cents {
            let jurisdiction_code = updated.reservation_metadata
                .get("jurisdiction_code")
                .and_then(|v| v.as_str())
                .unwrap_or("UNKNOWN")
                .to_string();

            if let Err(e) = crate::services::pm::tax::PmTaxService::record_ota_revenue_simple(
                db,
                tenant_id,
                updated.id,
                total_cents,
                updated.currency.as_deref().unwrap_or("USD"),
                &jurisdiction_code,
            ).await {
                tracing::error!(
                    %tenant_id, %reservation_id,
                    "confirm: TDT record failed (non-fatal): {e:#}"
                );
            }
        }

        tracing::info!(%reservation_id, %tenant_id, "confirm: reservation confirmed");
        Ok(to_summary(updated))
    }

    /// Record guest check-in. Transitions `confirmed` → `checked_in`.
    pub async fn check_in(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        reservation_id: Uuid,
    ) -> Result<ReservationSummary, ReservationError> {
        let reservation = Self::fetch_owned(db, tenant_id, reservation_id).await?;

        if reservation.status != "confirmed" {
            return Err(ReservationError::TerminalState {
                id: reservation_id,
                status: reservation.status,
            });
        }

        let mut active: atlas_reservation::ActiveModel = reservation.into();
        active.status = Set("checked_in".to_string());

        let updated = active.update(db).await?;
        tracing::info!(%reservation_id, %tenant_id, "check_in: guest checked in");
        Ok(to_summary(updated))
    }

    /// Record guest check-out. Transitions `checked_in` → `checked_out`.
    pub async fn check_out(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        reservation_id: Uuid,
    ) -> Result<ReservationSummary, ReservationError> {
        let reservation = Self::fetch_owned(db, tenant_id, reservation_id).await?;

        if reservation.status != "checked_in" {
            return Err(ReservationError::TerminalState {
                id: reservation_id,
                status: reservation.status,
            });
        }

        let mut active: atlas_reservation::ActiveModel = reservation.into();
        active.status = Set("checked_out".to_string());

        let updated = active.update(db).await?;
        tracing::info!(%reservation_id, %tenant_id, "check_out: guest checked out");
        Ok(to_summary(updated))
    }

    /// Cancel a reservation. Allowed from `hold` or `confirmed`.
    pub async fn cancel(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        reservation_id: Uuid,
        reason: Option<String>,
    ) -> Result<ReservationSummary, ReservationError> {
        let reservation = Self::fetch_owned(db, tenant_id, reservation_id).await?;

        match reservation.status.as_str() {
            "hold" | "confirmed" => {},
            "cancelled" => return Ok(to_summary(reservation)), // Idempotent.
            s => {
                return Err(ReservationError::TerminalState {
                    id: reservation_id,
                    status: s.to_string(),
                });
            }
        }

        let mut active: atlas_reservation::ActiveModel = reservation.into();
        active.status = Set("cancelled".to_string());
        active.cancelled_at = Set(Some(Utc::now()));
        active.cancellation_reason = Set(reason);

        let updated = active.update(db).await?;
        tracing::info!(%reservation_id, %tenant_id, "cancel: reservation cancelled");
        Ok(to_summary(updated))
    }

    /// Sweep and expire holds past their `hold_expires_at` deadline.
    ///
    /// Called by the `pm_str_hold_expiry_sweeper` background job.
    /// Returns the count of reservations transitioned to `hold_expired`.
    pub async fn expire_stale_holds(db: &DatabaseConnection) -> Result<u32> {
        use sea_orm::sea_query::Expr;

        let now = Utc::now();

        // Bulk-update all holds past their deadline across all tenants.
        let result = atlas_reservation::Entity::update_many()
            .col_expr(
                atlas_reservation::Column::Status,
                Expr::val("hold_expired").into(),
            )
            .col_expr(
                atlas_reservation::Column::UpdatedAt,
                Expr::val(now).into(),
            )
            .filter(atlas_reservation::Column::Status.eq("hold"))
            .filter(atlas_reservation::Column::HoldExpiresAt.lt(now))
            .exec(db)
            .await
            .map_err(|e| anyhow!("expire_stale_holds DB error: {e:#}"))?;

        let count = result.rows_affected as u32;
        if count > 0 {
            tracing::info!(count, "expire_stale_holds: released {} stale holds", count);
        }
        Ok(count)
    }

    // ── Read operations ───────────────────────────────────────────────────────

    /// List all STR reservations for a tenant, optionally filtered by asset.
    pub async fn list(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        asset_id: Option<Uuid>,
    ) -> Result<Vec<atlas_reservation::Model>> {
        let mut query = atlas_reservation::Entity::find()
            .filter(atlas_reservation::Column::TenantId.eq(tenant_id))
            .filter(atlas_reservation::Column::ReservationType.eq("str_unit"));

        if let Some(aid) = asset_id {
            query = query.filter(atlas_reservation::Column::ReservedAssetId.eq(aid));
        }

        let rows = query.all(db).await
            .map_err(|e| anyhow!("list reservations DB error: {e:#}"))?;
        Ok(rows)
    }

    /// Fetch a single reservation, verifying tenant ownership.
    pub async fn get(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        reservation_id: Uuid,
    ) -> Result<atlas_reservation::Model, ReservationError> {
        Self::fetch_owned(db, tenant_id, reservation_id).await
    }

    // ── Conflict detection ────────────────────────────────────────────────────

    /// Check whether any active reservation overlaps the given time window for an asset.
    pub async fn has_conflict(
        db: &DatabaseConnection,
        asset_id: Uuid,
        check_in: DateTime<Utc>,
        check_out: DateTime<Utc>,
    ) -> Result<bool, ReservationError> {
        let terminal: Vec<String> = vec![
            "cancelled".to_string(),
            "no_show".to_string(),
            "hold_expired".to_string(),
        ];

        let conflicts = atlas_reservation::Entity::find()
            .filter(atlas_reservation::Column::ReservedAssetId.eq(asset_id))
            .filter(atlas_reservation::Column::Status.is_not_in(terminal))
            // Overlap: existing.starts_at < check_out AND existing.ends_at > check_in
            .filter(atlas_reservation::Column::StartsAt.lt(check_out))
            .filter(atlas_reservation::Column::EndsAt.gt(check_in))
            .count(db)
            .await?;

        Ok(conflicts > 0)

    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    async fn fetch_owned(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        reservation_id: Uuid,
    ) -> Result<atlas_reservation::Model, ReservationError> {
        let row = atlas_reservation::Entity::find_by_id(reservation_id)
            .one(db)
            .await?
            .ok_or(ReservationError::NotFound {
                id: reservation_id,
                tenant_id,
            })?;

        if row.tenant_id != tenant_id {
            return Err(ReservationError::NotFound {
                id: reservation_id,
                tenant_id,
            });
        }

        Ok(row)
    }
}

// ── Conversion ────────────────────────────────────────────────────────────────

fn to_summary(r: atlas_reservation::Model) -> ReservationSummary {
    ReservationSummary {
        id: r.id,
        status: r.status,
        check_in: r.starts_at,
        check_out: r.ends_at,
        total_price_cents: r.total_price_cents.unwrap_or(0),
        currency: r.currency.unwrap_or_else(|| "USD".to_string()),
        hold_expires_at: r.hold_expires_at,
    }
}
