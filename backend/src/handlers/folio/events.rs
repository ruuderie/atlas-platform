//! # G21 Event HTTP handlers — Folio (Phase 6)
//!
//! # Route surface
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | POST   | /api/folio/events | Create event |
//! | GET    | /api/folio/events | List events (filterable) |
//! | GET    | /api/folio/events/{id} | Get single event |
//! | POST   | /api/folio/events/{id}/status | Transition event status |
//! | GET    | /api/folio/events/{id}/subject | Find events by subject entity |
//! | POST   | /api/folio/events/{id}/ticket-types | Create ticket type |
//! | GET    | /api/folio/events/{id}/ticket-types | List ticket types |
//! | POST   | /api/folio/events/{id}/register | Register an attendee |
//! | GET    | /api/folio/events/{id}/registrations | List registrations |
//! | POST   | /api/folio/events/check-in | QR token check-in (no event ID needed) |

use axum::{
    Router,
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    entities::user,
    services::pm::event::{
        CreateEventPayload, CreateTicketTypePayload, EventFilter, EventService, RegistrationPayload,
    },
    types::pm::{EventStatus, EventType, RegistrationStatus},
};

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/events", post(create_event).get(list_events))
        .route("/api/folio/events/{id}", get(get_event))
        .route("/api/folio/events/{id}/status", post(transition_status))
        .route(
            "/api/folio/events/{id}/ticket-types",
            post(create_ticket_type).get(list_ticket_types),
        )
        .route("/api/folio/events/{id}/register", post(register_attendee))
        .route(
            "/api/folio/events/{id}/registrations",
            get(list_registrations),
        )
        .route("/api/folio/events/check-in", post(check_in))
}

// ── Shared tenant resolution ──────────────────────────────────────────────────

async fn resolve_tenant_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Uuid, StatusCode> {
    crate::extractors::tenant::resolve_tenant_id(db, user_id).await
}

// ── Request types ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct CreateEventRequest {
    name: String,
    slug: Option<String>,
    event_type: String,
    is_virtual: Option<bool>,
    virtual_url: Option<String>,
    venue_name: Option<String>,
    venue_address: Option<String>,
    venue_asset_id: Option<Uuid>,
    max_capacity: Option<i32>,
    waitlist_enabled: Option<bool>,
    starts_at: chrono::DateTime<chrono::Utc>,
    ends_at: chrono::DateTime<chrono::Utc>,
    registration_opens_at: Option<chrono::DateTime<chrono::Utc>>,
    registration_closes_at: Option<chrono::DateTime<chrono::Utc>>,
    campaign_id: Option<Uuid>,
    subject_entity_type: Option<String>,
    subject_entity_id: Option<Uuid>,
    is_public: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ListEventsQuery {
    event_type: Option<String>,
    status: Option<String>,
    campaign_id: Option<Uuid>,
    subject_entity_type: Option<String>,
    subject_entity_id: Option<Uuid>,
    upcoming_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct TransitionStatusRequest {
    status: String,
}

#[derive(Debug, Deserialize)]
struct CreateTicketTypeRequest {
    name: String,
    price_cents: i64,
    currency: Option<String>,
    quantity_available: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct ListTicketTypesQuery {
    active_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct RegisterRequest {
    ticket_type_id: Uuid,
    attendee_email: String,
    attendee_name: Option<String>,
    quantity: Option<i32>,
    attribution_touchpoint_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
struct ListRegistrationsQuery {
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CheckInRequest {
    check_in_token: String,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn create_event(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(req): Json<CreateEventRequest>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };

    let event_type = match EventType::try_from(req.event_type.as_str()) {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response();
        }
    };

    let payload = CreateEventPayload {
        name: req.name,
        slug: req.slug,
        event_type,
        is_virtual: req.is_virtual.unwrap_or(false),
        virtual_url: req.virtual_url,
        venue_name: req.venue_name,
        venue_address: req.venue_address,
        venue_asset_id: req.venue_asset_id,
        max_capacity: req.max_capacity,
        waitlist_enabled: req.waitlist_enabled,
        starts_at: req.starts_at,
        ends_at: req.ends_at,
        registration_opens_at: req.registration_opens_at,
        registration_closes_at: req.registration_closes_at,
        campaign_id: req.campaign_id,
        subject_entity_type: req.subject_entity_type,
        subject_entity_id: req.subject_entity_id,
        is_public: req.is_public,
    };

    match EventService::create(&db, tenant_id, payload).await {
        Ok(ev) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "event": ev })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn list_events(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(q): Query<ListEventsQuery>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };

    let event_type = match q.event_type.as_deref().map(EventType::try_from) {
        Some(Err(e)) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response();
        }
        Some(Ok(t)) => Some(t),
        None => None,
    };
    let status = match q.status.as_deref().map(EventStatus::try_from) {
        Some(Err(e)) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response();
        }
        Some(Ok(s)) => Some(s),
        None => None,
    };

    let filter = EventFilter {
        event_type,
        status,
        campaign_id: q.campaign_id,
        subject_entity_type: q.subject_entity_type,
        subject_entity_id: q.subject_entity_id,
        upcoming_only: q.upcoming_only,
    };

    match EventService::list(&db, tenant_id, filter).await {
        Ok(events) => Json(serde_json::json!({ "events": events })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn get_event(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };
    match EventService::get(&db, tenant_id, id).await {
        Ok(ev) => Json(serde_json::json!({ "event": ev })).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn transition_status(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(req): Json<TransitionStatusRequest>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };
    let new_status = match EventStatus::try_from(req.status.as_str()) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response();
        }
    };
    match EventService::transition_status(&db, tenant_id, id, new_status).await {
        Ok(ev) => Json(serde_json::json!({ "event": ev })).into_response(),
        Err(e) if e.to_string().contains("Invalid event transition") => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn create_ticket_type(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateTicketTypeRequest>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };
    let payload = CreateTicketTypePayload {
        event_id: id,
        name: req.name,
        price_cents: req.price_cents,
        currency: req.currency,
        quantity_available: req.quantity_available,
    };
    match EventService::create_ticket_type(&db, tenant_id, payload).await {
        Ok(tt) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "ticket_type": tt })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn list_ticket_types(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Query(q): Query<ListTicketTypesQuery>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };
    match EventService::list_ticket_types(&db, tenant_id, id, q.active_only.unwrap_or(true)).await {
        Ok(types) => Json(serde_json::json!({ "ticket_types": types })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn register_attendee(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };
    let payload = RegistrationPayload {
        event_id: id,
        ticket_type_id: req.ticket_type_id,
        attendee_email: req.attendee_email,
        attendee_name: req.attendee_name,
        attendee_user_id: Some(current_user.id),
        quantity: req.quantity,
        attribution_touchpoint_id: req.attribution_touchpoint_id,
    };
    match EventService::register(&db, tenant_id, payload).await {
        Ok(reg) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "registration": reg })),
        )
            .into_response(),
        Err(e) if e.to_string().contains("capacity") || e.to_string().contains("status") => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn list_registrations(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Query(q): Query<ListRegistrationsQuery>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };
    let status_filter = match q.status.as_deref().map(RegistrationStatus::try_from) {
        Some(Err(e)) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response();
        }
        Some(Ok(s)) => Some(s),
        None => None,
    };
    match EventService::list_registrations(&db, tenant_id, id, status_filter).await {
        Ok(regs) => Json(serde_json::json!({ "registrations": regs })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// QR code check-in endpoint — token is the only required field.
/// No event ID in the path: the token uniquely identifies the registration.
async fn check_in(
    Extension(db): Extension<DatabaseConnection>,
    Json(req): Json<CheckInRequest>,
) -> impl IntoResponse {
    match EventService::check_in(&db, &req.check_in_token).await {
        Ok(reg) => Json(serde_json::json!({
            "registration": reg,
            "message": "Check-in successful"
        }))
        .into_response(),
        Err(e)
            if e.to_string().contains("Already checked in")
                || e.to_string().contains("cancelled")
                || e.to_string().contains("no-show")
                || e.to_string().contains("Payment not confirmed") =>
        {
            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
        Err(e) if e.to_string().contains("Invalid check-in token") => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Invalid token" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
