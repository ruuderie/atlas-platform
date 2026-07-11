//! Folio — STR Guest Portal handler
//!
//! HTTP endpoints for managing guests, vehicles, and special requests on
//! STR bookings. All write routes require `FolioRole::Landlord` (the
//! property manager files guests on behalf of / during check-in). Read
//! routes are available to the guest themselves via shared router.
//!
//! # Routes
//!
//! ## Landlord / operator (gated by landlord_router)
//!
//! ```ignore
//! GET  /api/folio/reservations/{id}/manifest
//!      Full guest + vehicle + special requests manifest.
//!      -> 200 ReservationManifest
//!
//! POST /api/folio/reservations/{id}/guests
//!      Register a guest on this booking.
//!      Body: { full_name, nationality, date_of_birth, document_type, document_number, is_lead_guest }
//!      -> 201 { rel_id }
//!
//! DELETE /api/folio/reservations/{id}/guests/{rel_id}
//!      Remove a guest registration (hard delete).
//!      -> 204
//!
//! POST /api/folio/reservations/{id}/vehicles
//!      Register a vehicle on this booking.
//!      Body: { license_plate, make?, model?, color?, parking_spot? }
//!      -> 201 { rel_id }
//!
//! DELETE /api/folio/reservations/{id}/vehicles/{rel_id}
//!      Remove a vehicle registration (hard delete).
//!      -> 204
//!
//! PUT /api/folio/reservations/{id}/special-requests
//!      Overwrite the special requests list.
//!      Body: { requests: ["Late check-in", "Baby cot"] }
//!      -> 200
//! ```
//!
//! ## Violations on STR bookings (filed via violations handler, not this one)
//! Use `POST /api/folio/violations` with `reservation_id` set in the body.
//! The violations handler already accepts this field.

use axum::{
    Router,
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use chrono::NaiveDate;
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use uuid::Uuid;

use crate::services::pm::str_guest::{
    DocumentNumber, DocumentType, NationalityCode, RegisterStrGuestInput, RegisterStrVehicleInput,
    StrGuestService,
};

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/reservations/{id}/manifest", get(get_manifest))
        .route("/api/folio/reservations/{id}/guests", post(register_guest))
        .route(
            "/api/folio/reservations/{id}/guests/{rel_id}",
            delete(remove_guest),
        )
        .route(
            "/api/folio/reservations/{id}/vehicles",
            post(register_vehicle),
        )
        .route(
            "/api/folio/reservations/{id}/vehicles/{rel_id}",
            delete(remove_vehicle),
        )
        .route(
            "/api/folio/reservations/{id}/special-requests",
            put(set_special_requests),
        )
}

// ── HTTP input types ──────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RegisterGuestBody {
    pub full_name: String,
    /// ISO 3166-1 alpha-2, e.g. "BR", "US"
    pub nationality: String,
    /// "YYYY-MM-DD"
    pub date_of_birth: NaiveDate,
    /// "passport" | "national_id" | "driver_licence" | "residence_permit" | "visa" | "other_gov_id"
    pub document_type: String,
    pub document_number: String,
    #[serde(default)]
    pub is_lead_guest: bool,
    pub user_account_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterVehicleBody {
    pub license_plate: String,
    pub make: Option<String>,
    pub model: Option<String>,
    pub color: Option<String>,
    pub parking_spot: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SpecialRequestsBody {
    pub requests: Vec<String>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn get_manifest(
    Extension(db): Extension<DatabaseConnection>,
    Extension(tenant_id): Extension<Uuid>,
    Path(reservation_id): Path<Uuid>,
) -> impl IntoResponse {
    match StrGuestService::get_manifest(&db, tenant_id, reservation_id).await {
        Ok(m) => (StatusCode::OK, Json(m)).into_response(),
        Err(e) => {
            tracing::error!("get_manifest: {e:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

async fn register_guest(
    Extension(db): Extension<DatabaseConnection>,
    Extension(user_id): Extension<Uuid>,
    Extension(tenant_id): Extension<Uuid>,
    Path(reservation_id): Path<Uuid>,
    Json(body): Json<RegisterGuestBody>,
) -> impl IntoResponse {
    // Validate newtypes at the HTTP boundary → 422 on invalid input
    let nationality = match NationalityCode::new(&body.nationality) {
        Ok(n) => n,
        Err(e) => return (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response(),
    };
    let document_type = match DocumentType::try_from(body.document_type) {
        Ok(d) => d,
        Err(e) => return (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
    };
    let document_number = match DocumentNumber::new(&body.document_number) {
        Ok(n) => n,
        Err(e) => return (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response(),
    };

    let input = RegisterStrGuestInput {
        full_name: body.full_name,
        nationality,
        date_of_birth: body.date_of_birth,
        document_type,
        document_number,
        is_lead_guest: body.is_lead_guest,
        user_account_id: body.user_account_id,
    };

    match StrGuestService::register_guest(&db, tenant_id, reservation_id, user_id, input).await {
        Ok(rel_id) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "rel_id": rel_id })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("register_guest: {e:#}");
            (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()
        }
    }
}

async fn remove_guest(
    Extension(db): Extension<DatabaseConnection>,
    Extension(tenant_id): Extension<Uuid>,
    Path((reservation_id, rel_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    match StrGuestService::remove_guest(&db, tenant_id, reservation_id, rel_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("remove_guest: {e:#}");
            (StatusCode::NOT_FOUND, e.to_string()).into_response()
        }
    }
}

async fn register_vehicle(
    Extension(db): Extension<DatabaseConnection>,
    Extension(user_id): Extension<Uuid>,
    Extension(tenant_id): Extension<Uuid>,
    Path(reservation_id): Path<Uuid>,
    Json(body): Json<RegisterVehicleBody>,
) -> impl IntoResponse {
    // Validate plate at the HTTP boundary
    if let Err(e) = RegisterStrVehicleInput::validate_plate(&body.license_plate) {
        return (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response();
    }

    let input = RegisterStrVehicleInput {
        license_plate: body.license_plate,
        make: body.make,
        model: body.model,
        color: body.color,
        parking_spot: body.parking_spot,
    };

    match StrGuestService::register_vehicle(&db, tenant_id, reservation_id, user_id, input).await {
        Ok(rel_id) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "rel_id": rel_id })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("register_vehicle: {e:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

async fn remove_vehicle(
    Extension(db): Extension<DatabaseConnection>,
    Extension(tenant_id): Extension<Uuid>,
    Path((reservation_id, rel_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    match StrGuestService::remove_vehicle(&db, tenant_id, reservation_id, rel_id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            tracing::error!("remove_vehicle: {e:#}");
            (StatusCode::NOT_FOUND, e.to_string()).into_response()
        }
    }
}

async fn set_special_requests(
    Extension(db): Extension<DatabaseConnection>,
    Extension(tenant_id): Extension<Uuid>,
    Path(reservation_id): Path<Uuid>,
    Json(body): Json<SpecialRequestsBody>,
) -> impl IntoResponse {
    match StrGuestService::set_special_requests(&db, tenant_id, reservation_id, body.requests).await
    {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => {
            tracing::error!("set_special_requests: {e:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}
