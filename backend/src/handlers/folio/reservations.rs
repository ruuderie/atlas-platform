//! Folio — STR Reservation handler.
//!
//! # Route surface
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | POST | `/api/folio/reservations` | Create a hold on an STR unit |
//! | GET  | `/api/folio/reservations` | List reservations (optionally by asset) |
//! | GET  | `/api/folio/reservations/{id}` | Get a single reservation |
//! | POST | `/api/folio/reservations/{id}/confirm` | Confirm a held reservation |
//! | POST | `/api/folio/reservations/{id}/check-in` | Record guest check-in |
//! | POST | `/api/folio/reservations/{id}/check-out` | Record guest check-out |
//! | POST | `/api/folio/reservations/{id}/cancel` | Cancel a reservation |
//!
//! # Conflict response
//!
//! If the requested time window overlaps an active reservation, the hold endpoint
//! returns `409 Conflict` with a structured body explaining the conflict.
//!
//! # Data source
//!
//! `atlas_reservations` (G-23). No net-new tables.
//!
//! # TDT tax
//!
//! `POST /{id}/confirm` triggers `PmTaxService::record_ota_revenue_simple()` to
//! register the Tourist Development Tax obligation. This is best-effort and does
//! not block the confirmation response.

use axum::{
    Router,
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::user;
use crate::services::pm::reservation::{CreateHoldInput, ReservationError, ReservationService};
use crate::types::pm::OtaPlatform;

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route(
            "/api/folio/reservations",
            get(list_reservations).post(create_hold),
        )
        .route("/api/folio/reservations/{id}", get(get_reservation))
        .route(
            "/api/folio/reservations/{id}/confirm",
            post(confirm_reservation),
        )
        .route(
            "/api/folio/reservations/{id}/check-in",
            post(check_in_reservation),
        )
        .route(
            "/api/folio/reservations/{id}/check-out",
            post(check_out_reservation),
        )
        .route(
            "/api/folio/reservations/{id}/cancel",
            post(cancel_reservation),
        )
}

// ── Request / response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct CreateHoldHttpInput {
    /// The STR unit's atlas_assets.id.
    pub asset_id: Uuid,
    /// The guest's atlas_accounts.id.
    pub guest_account_id: Uuid,
    /// ISO 8601 datetime: "2026-07-04T15:00:00Z"
    pub check_in: chrono::DateTime<chrono::Utc>,
    /// ISO 8601 datetime: "2026-07-07T11:00:00Z"
    pub check_out: chrono::DateTime<chrono::Utc>,
    /// Nightly rate in minor currency units.
    pub nightly_rate_cents: i64,
    /// ISO 4217 currency: "USD", "BRL", etc.
    pub currency: String,
    /// Number of guests.
    pub guest_count: u32,
    /// OTA platform string: "airbnb", "vrbo", "direct", "booking_com", etc.
    pub ota_platform: String,
    /// External OTA reservation code.
    pub external_hold_id: Option<String>,
    /// How long to hold the slot (minutes). Defaults to 30.
    pub hold_minutes: Option<u32>,
    /// Jurisdiction for STR permit validation and TDT tax.
    pub jurisdiction_code: String,
}

#[derive(Debug, Serialize)]
struct HoldResponse {
    pub id: Uuid,
    pub status: &'static str,
    pub hold_expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub total_price_cents: i64,
}

#[derive(Debug, Serialize)]
struct ReservationResponse {
    pub id: Uuid,
    pub status: String,
    pub check_in: chrono::DateTime<chrono::Utc>,
    pub check_out: chrono::DateTime<chrono::Utc>,
    pub total_price_cents: i64,
    pub currency: String,
    pub hold_expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
struct ListReservationsQuery {
    /// Filter by STR unit asset ID.
    pub asset_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
struct CancelInput {
    pub reason: Option<String>,
}

#[allow(dead_code)] // Reserved for create_hold 409 body — not yet wired in the impl IntoResponse arm
#[derive(Debug, Serialize)]
struct ConflictResponse {
    pub error: &'static str,
    pub asset_id: Uuid,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// POST /api/folio/reservations
///
/// Create a time-bounded hold on an STR unit.
/// Returns 409 if the time slot is already taken.
async fn create_hold(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<CreateHoldHttpInput>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(status) => return status.into_response(),
    };

    let ota_platform =
        OtaPlatform::try_from(input.ota_platform.clone()).unwrap_or(OtaPlatform::Direct);

    let result = ReservationService::create_hold(
        &db,
        tenant_id,
        CreateHoldInput {
            asset_id: input.asset_id,
            guest_account_id: input.guest_account_id,
            check_in: input.check_in,
            check_out: input.check_out,
            nightly_rate_cents: input.nightly_rate_cents,
            currency: input.currency,
            guest_count: input.guest_count,
            ota_platform,
            external_hold_id: input.external_hold_id,
            hold_minutes: input.hold_minutes,
            jurisdiction_code: input.jurisdiction_code,
        },
    )
    .await;

    match result {
        Ok(id) => {
            // Fetch back the created row to include hold_expires_at + total.
            match ReservationService::get(&db, tenant_id, id).await {
                Ok(r) => (
                    StatusCode::CREATED,
                    axum::response::Json(HoldResponse {
                        id: r.id,
                        status: "hold",
                        hold_expires_at: r.hold_expires_at,
                        total_price_cents: r.total_price_cents.unwrap_or(0),
                    }),
                )
                    .into_response(),
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
        Err(ReservationError::Conflict { .. }) => StatusCode::CONFLICT.into_response(),
        Err(ReservationError::InvalidWindow { .. }) => {
            StatusCode::UNPROCESSABLE_ENTITY.into_response()
        }
        Err(e) => {
            tracing::error!(%tenant_id, "create_hold error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// GET /api/folio/reservations
async fn list_reservations(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(q): Query<ListReservationsQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let rows = ReservationService::list(&db, tenant_id, q.asset_id)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "list_reservations error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let response: Vec<ReservationResponse> = rows
        .into_iter()
        .map(|r| ReservationResponse {
            id: r.id,
            status: r.status,
            check_in: r.starts_at,
            check_out: r.ends_at,
            total_price_cents: r.total_price_cents.unwrap_or(0),
            currency: r.currency.unwrap_or_else(|| "USD".to_string()),
            hold_expires_at: r.hold_expires_at,
        })
        .collect();

    Ok(axum::response::Json(response))
}

/// GET /api/folio/reservations/{id}
async fn get_reservation(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(reservation_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let r = ReservationService::get(&db, tenant_id, reservation_id)
        .await
        .map_err(|e| match e {
            ReservationError::NotFound { .. } => StatusCode::NOT_FOUND,
            other => {
                tracing::error!(%tenant_id, %reservation_id, "get_reservation error: {other:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok(axum::response::Json(ReservationResponse {
        id: r.id,
        status: r.status,
        check_in: r.starts_at,
        check_out: r.ends_at,
        total_price_cents: r.total_price_cents.unwrap_or(0),
        currency: r.currency.unwrap_or_else(|| "USD".to_string()),
        hold_expires_at: r.hold_expires_at,
    }))
}

/// POST /api/folio/reservations/{id}/confirm
async fn confirm_reservation(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(reservation_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let summary = ReservationService::confirm(&db, tenant_id, reservation_id)
        .await
        .map_err(|e| reservation_err_to_status(e, reservation_id, tenant_id))?;

    Ok(axum::response::Json(ReservationResponse {
        id: summary.id,
        status: summary.status,
        check_in: summary.check_in,
        check_out: summary.check_out,
        total_price_cents: summary.total_price_cents,
        currency: summary.currency,
        hold_expires_at: summary.hold_expires_at,
    }))
}

/// POST /api/folio/reservations/{id}/check-in
async fn check_in_reservation(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(reservation_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let summary = ReservationService::check_in(&db, tenant_id, reservation_id)
        .await
        .map_err(|e| reservation_err_to_status(e, reservation_id, tenant_id))?;

    Ok(axum::response::Json(ReservationResponse {
        id: summary.id,
        status: summary.status,
        check_in: summary.check_in,
        check_out: summary.check_out,
        total_price_cents: summary.total_price_cents,
        currency: summary.currency,
        hold_expires_at: summary.hold_expires_at,
    }))
}

/// POST /api/folio/reservations/{id}/check-out
async fn check_out_reservation(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(reservation_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let summary = ReservationService::check_out(&db, tenant_id, reservation_id)
        .await
        .map_err(|e| reservation_err_to_status(e, reservation_id, tenant_id))?;

    // G-27: open post_checkout rating sessions (best-effort — never fail check-out).
    if let Err(e) =
        trigger_post_checkout_sessions(&db, tenant_id, reservation_id, current_user.id).await
    {
        tracing::warn!(
            %tenant_id,
            %reservation_id,
            "check_out: scorecard post_checkout trigger failed (non-fatal): {e:#}"
        );
    }

    Ok(axum::response::Json(ReservationResponse {
        id: summary.id,
        status: summary.status,
        check_in: summary.check_in,
        check_out: summary.check_out,
        total_price_cents: summary.total_price_cents,
        currency: summary.currency,
        hold_expires_at: summary.hold_expires_at,
    }))
}

/// POST /api/folio/reservations/{id}/cancel
async fn cancel_reservation(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(reservation_id): Path<Uuid>,
    Json(input): Json<CancelInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let summary = ReservationService::cancel(&db, tenant_id, reservation_id, input.reason)
        .await
        .map_err(|e| reservation_err_to_status(e, reservation_id, tenant_id))?;

    Ok(axum::response::Json(ReservationResponse {
        id: summary.id,
        status: summary.status,
        check_in: summary.check_in,
        check_out: summary.check_out,
        total_price_cents: summary.total_price_cents,
        currency: summary.currency,
        hold_expires_at: summary.hold_expires_at,
    }))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn reservation_err_to_status(e: ReservationError, id: Uuid, tenant_id: Uuid) -> StatusCode {
    match e {
        ReservationError::NotFound { .. } => StatusCode::NOT_FOUND,
        ReservationError::TerminalState { .. } => StatusCode::CONFLICT,
        ReservationError::Conflict { .. } => StatusCode::CONFLICT,
        ReservationError::InvalidWindow { .. } => StatusCode::UNPROCESSABLE_ENTITY,
        other => {
            tracing::error!(%tenant_id, %id, "reservation handler error: {other:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn resolve_tenant_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Uuid, StatusCode> {
    crate::extractors::tenant::resolve_tenant_id(db, user_id).await
}

/// Best-effort G-27 post_checkout session open after STR check-out.
async fn trigger_post_checkout_sessions(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    reservation_id: Uuid,
    rater_user_id: Uuid,
) -> anyhow::Result<()> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

    let reservation = crate::entities::atlas_reservation::Entity::find_by_id(reservation_id)
        .filter(crate::entities::atlas_reservation::Column::TenantId.eq(tenant_id))
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("reservation not found after check-out"))?;

    let asset_id = match reservation.reserved_asset_id {
        Some(id) => id,
        None => {
            tracing::debug!(%reservation_id, "post_checkout: no reserved_asset_id — skip");
            return Ok(());
        }
    };

    let app_instance_id = crate::entities::app_instance::Entity::find()
        .filter(crate::entities::app_instance::Column::TenantId.eq(tenant_id))
        .filter(crate::entities::app_instance::Column::AppType.eq("property_management"))
        .order_by_asc(crate::entities::app_instance::Column::CreatedAt)
        .one(db)
        .await?
        .map(|i| i.id);

    let Some(app_instance_id) = app_instance_id else {
        tracing::debug!(%tenant_id, "post_checkout: no Folio app_instance — skip");
        return Ok(());
    };

    let opened = crate::services::scorecard_triggers::on_str_checkout(
        db,
        tenant_id,
        app_instance_id,
        reservation_id,
        asset_id,
        rater_user_id,
    )
    .await?;

    tracing::info!(
        %tenant_id,
        %reservation_id,
        sessions = opened.len(),
        "post_checkout: rating sessions opened"
    );
    Ok(())
}
