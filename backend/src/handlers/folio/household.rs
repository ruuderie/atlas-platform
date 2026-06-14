//! Folio — Household handler (vehicles + occupants)
//!
//! # Routes
//!
//! ```ignore
//! --- Tenant routes (FolioRole::Tenant, LTR only) ---
//!
//! GET    /api/folio/leases/{lease_id}/vehicles
//!        List registered vehicles on this lease.
//!        -> 200 [VehicleRecord]
//!
//! POST   /api/folio/leases/{lease_id}/vehicles
//!        Register a vehicle. Body: RegisterVehicleHttpInput
//!        -> 201 VehicleRecord
//!
//! PATCH  /api/folio/leases/{lease_id}/vehicles/{entry_id}
//!        Update vehicle details mid-lease (new car, plate change, parking spot).
//!        Body: UpdateVehicleHttpInput
//!        -> 200 VehicleRecord
//!
//! DELETE /api/folio/leases/{lease_id}/vehicles/{entry_id}
//!        Remove a vehicle (hard delete — no legal retention need).
//!        -> 204
//!
//! GET    /api/folio/leases/{lease_id}/occupants
//!        List current active occupants. Query: ?include_former=true for history.
//!        -> 200 { active: [ActiveOccupant], former: [FormerOccupant] }
//!
//! POST   /api/folio/leases/{lease_id}/occupants
//!        Register a household member. Body: RegisterOccupantHttpInput
//!        -> 201 ActiveOccupant
//!
//! PATCH  /api/folio/leases/{lease_id}/occupants/{entry_id}
//!        Correct occupant details (name typo, add doc). Body: UpdateOccupantHttpInput
//!        -> 200 ActiveOccupant
//!
//! POST   /api/folio/leases/{lease_id}/occupants/{entry_id}/depart
//!        Record departure (soft delete). Body: DepartOccupantInput
//!        -> 200 FormerOccupant
//!
//! --- Landlord routes ---
//!
//! GET    /api/folio/units/{unit_id}/occupants
//!        All current occupants across active leases for this unit.
//!        -> 200 [{ lease_id, occupants, vehicles }]
//! ```

use axum::{
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Router,
};
use chrono::NaiveDate;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::services::pm::household::{
    ActiveOccupant, AdultRelationship, DepartureReason, FormerOccupant,
    HouseholdService, MinorRelationship,
    RegisterAdultInput, RegisterMinorInput, RegisterOccupantInput,
    RegisterVehicleInput, UpdateOccupantInput, UpdateVehicleInput, VehicleRecord,
};
use crate::services::pm::household::{CountryCode, LicensePlate, ModelYear};

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes() -> Router<DatabaseConnection> {
    Router::new()
        // Vehicles
        .route(
            "/api/folio/leases/{lease_id}/vehicles",
            get(list_vehicles).post(register_vehicle),
        )
        .route(
            "/api/folio/leases/{lease_id}/vehicles/{entry_id}",
            patch(update_vehicle).delete(remove_vehicle),
        )
        // Occupants
        .route(
            "/api/folio/leases/{lease_id}/occupants",
            get(list_occupants).post(register_occupant),
        )
        .route(
            "/api/folio/leases/{lease_id}/occupants/{entry_id}",
            patch(update_occupant),
        )
        .route(
            "/api/folio/leases/{lease_id}/occupants/{entry_id}/depart",
            post(depart_occupant),
        )
        // Landlord unit view
        .route("/api/folio/units/{unit_id}/occupants", get(list_unit_occupants))
}

// ═══════════════════════════════════════════════════════════════════════════════
// HTTP input types — deserialized from JSON, validated into typed service inputs
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Deserialize)]
pub struct RegisterVehicleHttpInput {
    pub make: String,
    pub model: String,
    pub year: i32,
    pub color: String,
    pub license_plate: String,
    pub state: String,
    pub country: String,
    pub parking_spot: Option<String>,
    pub registration_expiry: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVehicleHttpInput {
    pub make: Option<String>,
    pub model: Option<String>,
    pub year: Option<i32>,
    pub color: Option<String>,
    pub license_plate: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub parking_spot: Option<String>,
    pub registration_expiry: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RegisterOccupantHttpInput {
    Adult {
        full_name: String,
        relationship: AdultRelationship,
        profile_id: Option<Uuid>,
        id_document_type: Option<String>,
        id_document_number: Option<String>,
        notes: Option<String>,
    },
    Minor {
        full_name: String,
        relationship: MinorRelationship,
        date_of_birth: NaiveDate,
        notes: Option<String>,
    },
}

#[derive(Debug, Deserialize)]
pub struct UpdateOccupantHttpInput {
    pub full_name: Option<String>,
    pub id_document_type: Option<String>,
    pub id_document_number: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DepartOccupantHttpInput {
    pub reason: DepartureReason,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListOccupantsQuery {
    pub include_former: Option<bool>,
}

// ── Serializable combined occupant response ───────────────────────────────────

#[derive(Debug, Serialize)]
pub struct OccupantsResponse {
    pub active: Vec<ActiveOccupant>,
    pub former: Vec<FormerOccupant>,
}

// ── Unit occupant response ────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct UnitOccupantsResponse {
    pub lease_id: Uuid,
    pub occupants: Vec<ActiveOccupant>,
    pub vehicles: Vec<VehicleRecord>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Handlers
// ═══════════════════════════════════════════════════════════════════════════════

async fn list_vehicles(
    Extension(db): Extension<DatabaseConnection>,
    Extension(_user_id): Extension<Uuid>,
    Extension(tenant_id): Extension<Uuid>,
    Path(lease_id): Path<Uuid>,
) -> impl IntoResponse {
    match HouseholdService::list_vehicles(&db, tenant_id, lease_id).await {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(e) => {
            tracing::error!("list_vehicles: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn register_vehicle(
    Extension(db): Extension<DatabaseConnection>,
    Extension(user_id): Extension<Uuid>,
    Extension(tenant_id): Extension<Uuid>,
    Path(lease_id): Path<Uuid>,
    Json(body): Json<RegisterVehicleHttpInput>,
) -> impl IntoResponse {
    // Validate newtypes at the HTTP boundary — errors return 422
    let plate = match LicensePlate::new(&body.license_plate) {
        Ok(p) => p,
        Err(e) => return (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response(),
    };
    let year = match ModelYear::new(body.year) {
        Ok(y) => y,
        Err(e) => return (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response(),
    };
    let country = match CountryCode::new(&body.country) {
        Ok(c) => c,
        Err(e) => return (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response(),
    };

    let input = RegisterVehicleInput {
        make: body.make,
        model: body.model,
        year,
        color: body.color,
        license_plate: plate,
        state: body.state,
        country,
        parking_spot: body.parking_spot,
        registration_expiry: body.registration_expiry,
    };

    match HouseholdService::register_vehicle(&db, tenant_id, user_id, lease_id, input).await {
        Ok(v) => (StatusCode::CREATED, Json(v)).into_response(),
        Err(e) => {
            tracing::error!("register_vehicle: {e:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

async fn update_vehicle(
    Extension(db): Extension<DatabaseConnection>,
    Extension(user_id): Extension<Uuid>,
    Extension(tenant_id): Extension<Uuid>,
    Path((lease_id, entry_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateVehicleHttpInput>,
) -> impl IntoResponse {
    let license_plate = match body.license_plate.as_deref() {
        Some(p) => match LicensePlate::new(p) {
            Ok(lp) => Some(lp),
            Err(e) => return (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response(),
        },
        None => None,
    };
    let year = match body.year {
        Some(y) => match ModelYear::new(y) {
            Ok(my) => Some(my),
            Err(e) => return (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response(),
        },
        None => None,
    };
    let country = match body.country.as_deref() {
        Some(c) => match CountryCode::new(c) {
            Ok(cc) => Some(cc),
            Err(e) => return (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response(),
        },
        None => None,
    };

    let patch = UpdateVehicleInput {
        make: body.make,
        model: body.model,
        year,
        color: body.color,
        license_plate,
        state: body.state,
        country,
        parking_spot: body.parking_spot,
        registration_expiry: body.registration_expiry,
    };

    match HouseholdService::update_vehicle(&db, tenant_id, user_id, lease_id, entry_id, patch).await {
        Ok(v) => (StatusCode::OK, Json(v)).into_response(),
        Err(e) => {
            tracing::error!("update_vehicle: {e:#}");
            (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()
        }
    }
}

async fn remove_vehicle(
    Extension(db): Extension<DatabaseConnection>,
    Extension(user_id): Extension<Uuid>,
    Extension(tenant_id): Extension<Uuid>,
    Path((lease_id, entry_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    match HouseholdService::remove_vehicle(&db, tenant_id, user_id, lease_id, entry_id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("remove_vehicle: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn list_occupants(
    Extension(db): Extension<DatabaseConnection>,
    Extension(tenant_id): Extension<Uuid>,
    Path(lease_id): Path<Uuid>,
    Query(q): Query<ListOccupantsQuery>,
) -> impl IntoResponse {
    let include_former = q.include_former.unwrap_or(false);

    let active = match HouseholdService::list_active_occupants(&db, tenant_id, lease_id).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("list_active_occupants: {e:#}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let former = if include_former {
        match HouseholdService::list_former_occupants(&db, tenant_id, lease_id).await {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("list_former_occupants: {e:#}");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    } else {
        vec![]
    };

    (StatusCode::OK, Json(OccupantsResponse { active, former })).into_response()
}

async fn register_occupant(
    Extension(db): Extension<DatabaseConnection>,
    Extension(user_id): Extension<Uuid>,
    Extension(tenant_id): Extension<Uuid>,
    Path(lease_id): Path<Uuid>,
    Json(body): Json<RegisterOccupantHttpInput>,
) -> impl IntoResponse {
    let input = match body {
        RegisterOccupantHttpInput::Adult {
            full_name, relationship, profile_id, id_document_type, id_document_number, notes,
        } => RegisterOccupantInput::Adult(RegisterAdultInput {
            full_name, relationship, profile_id, id_document_type, id_document_number, notes,
        }),
        RegisterOccupantHttpInput::Minor {
            full_name, relationship, date_of_birth, notes,
        } => RegisterOccupantInput::Minor(RegisterMinorInput {
            full_name, relationship, date_of_birth, notes,
        }),
    };

    match HouseholdService::register_occupant(&db, tenant_id, user_id, lease_id, input).await {
        Ok(o) => (StatusCode::CREATED, Json(o)).into_response(),
        Err(e) => {
            tracing::error!("register_occupant: {e:#}");
            (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()
        }
    }
}

async fn update_occupant(
    Extension(db): Extension<DatabaseConnection>,
    Extension(user_id): Extension<Uuid>,
    Extension(tenant_id): Extension<Uuid>,
    Path((lease_id, entry_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateOccupantHttpInput>,
) -> impl IntoResponse {
    let patch = UpdateOccupantInput {
        full_name: body.full_name,
        id_document_type: body.id_document_type,
        id_document_number: body.id_document_number,
        notes: body.notes,
    };

    match HouseholdService::update_occupant(&db, tenant_id, user_id, lease_id, entry_id, patch).await {
        Ok(o) => (StatusCode::OK, Json(o)).into_response(),
        Err(e) => {
            tracing::error!("update_occupant: {e:#}");
            (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()
        }
    }
}

async fn depart_occupant(
    Extension(db): Extension<DatabaseConnection>,
    Extension(user_id): Extension<Uuid>,
    Extension(tenant_id): Extension<Uuid>,
    Path((lease_id, entry_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<DepartOccupantHttpInput>,
) -> impl IntoResponse {
    match HouseholdService::remove_occupant(
        &db, tenant_id, user_id, lease_id, entry_id, body.reason, body.notes,
    ).await {
        Ok(former) => (StatusCode::OK, Json(former)).into_response(),
        Err(e) => {
            tracing::error!("depart_occupant: {e:#}");
            (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()
        }
    }
}

async fn list_unit_occupants(
    Extension(db): Extension<DatabaseConnection>,
    Extension(tenant_id): Extension<Uuid>,
    Path(unit_id): Path<Uuid>,
) -> impl IntoResponse {
    match HouseholdService::list_unit_occupants(&db, tenant_id, unit_id).await {
        Ok(results) => {
            let response: Vec<UnitOccupantsResponse> = results
                .into_iter()
                .map(|(lease_id, occupants, vehicles)| UnitOccupantsResponse {
                    lease_id,
                    occupants,
                    vehicles,
                })
                .collect();
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!("list_unit_occupants: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
