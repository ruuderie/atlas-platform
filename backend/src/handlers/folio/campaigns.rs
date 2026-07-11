//! # G19 Campaign HTTP handlers — Folio (Phase 6)
//!
//! # Route surface
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | POST   | /api/folio/campaigns | Create campaign |
//! | GET    | /api/folio/campaigns | List campaigns (filterable) |
//! | GET    | /api/folio/campaigns/{id} | Get single campaign |
//! | POST   | /api/folio/campaigns/{id}/status | Transition status |
//! | GET    | /api/folio/campaigns/{id}/children | List direct child campaigns |
//! | GET    | /api/folio/campaigns/{id}/stats | Hierarchy roll-up stats |
//! | POST   | /api/folio/campaigns/{id}/steps | Add sequence step |
//! | GET    | /api/folio/campaigns/{id}/steps | List sequence steps |
//! | POST   | /api/folio/campaigns/{id}/enroll | Enroll a contact |
//! | GET    | /api/folio/campaigns/{id}/enrollments | List enrollments |
//! | POST   | /api/folio/campaigns/events | Record campaign event (webhook entrypoint) |

use axum::{
    Router,
    body::Body,
    extract::{Extension, Json, Path, Query},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entities::user,
    services::pm::campaign::{
        CampaignFilter, CampaignService, CreateCampaignPayload, CreateSequenceStepPayload,
        EnrollContactPayload, RecordEventPayload,
    },
    types::pm::{CampaignStatus, CampaignType, EnrollmentStatus},
};

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route(
            "/api/folio/campaigns",
            post(create_campaign).get(list_campaigns),
        )
        .route("/api/folio/campaigns/{id}", get(get_campaign))
        .route("/api/folio/campaigns/{id}/status", post(transition_status))
        .route("/api/folio/campaigns/{id}/children", get(list_children))
        .route("/api/folio/campaigns/{id}/stats", get(get_hierarchy_stats))
        .route(
            "/api/folio/campaigns/{id}/steps",
            post(add_sequence_step).get(list_steps),
        )
        .route("/api/folio/campaigns/{id}/enroll", post(enroll_contact))
        .route(
            "/api/folio/campaigns/{id}/enrollments",
            get(list_enrollments),
        )
        .route("/api/folio/campaigns/events", post(record_event))
        // Export campaign members as a direct-mail-ready CSV (leads + contacts)
        .route(
            "/api/folio/campaigns/{id}/enrollments/export",
            get(export_enrollments_csv),
        )
        // Bulk-enroll atlas_leads by lead ID
        .route(
            "/api/folio/campaigns/{id}/enroll-leads",
            post(enroll_leads_bulk),
        )
        // Bulk-enroll atlas_contacts by contact ID (address resolved from linked account)
        .route(
            "/api/folio/campaigns/{id}/enroll-contacts",
            post(enroll_contacts_bulk),
        )
}

// ── Shared tenant resolution ──────────────────────────────────────────────────

async fn resolve_tenant_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Uuid, StatusCode> {
    let user_accounts = crate::entities::user_account::Entity::find()
        .filter(crate::entities::user_account::Column::UserId.eq(user_id))
        .all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let account_ids: Vec<Uuid> = user_accounts.into_iter().map(|ua| ua.account_id).collect();

    let profile = crate::entities::profile::Entity::find()
        .filter(crate::entities::profile::Column::AccountId.is_in(account_ids))
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)?;

    Ok(profile.tenant_id)
}

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct CreateCampaignRequest {
    parent_campaign_id: Option<Uuid>,
    name: String,
    campaign_type: String,
    goal_type: Option<String>,
    goal_entity_type: Option<String>,
    target_conversion_count: Option<i32>,
    budget_cents: Option<i64>,
    currency: Option<String>,
    attribution_window_days: Option<i32>,
    integration_id: Option<Uuid>,
    external_campaign_id: Option<String>,
    subject_entity_type: Option<String>,
    subject_entity_id: Option<Uuid>,
    starts_at: Option<chrono::DateTime<chrono::Utc>>,
    ends_at: Option<chrono::DateTime<chrono::Utc>>,
    utm_source: Option<String>,
    utm_medium: Option<String>,
    utm_campaign: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ListCampaignsQuery {
    campaign_type: Option<String>,
    status: Option<String>,
    subject_entity_type: Option<String>,
    subject_entity_id: Option<Uuid>,
    roots_only: Option<bool>,
    parent_campaign_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
struct TransitionStatusRequest {
    status: String,
}

#[derive(Debug, Deserialize)]
struct AddStepRequest {
    step_number: i32,
    step_type: String,
    subject_template: Option<String>,
    body_template: Option<String>,
    wait_days: Option<i32>,
    wait_hours: Option<i32>,
    send_time_preference: Option<String>,
    condition_type: Option<String>,
    condition_value: Option<serde_json::Value>,
    on_true_step: Option<i32>,
    on_false_step: Option<i32>,
    ab_variants: Option<serde_json::Value>,
    exit_on_reply: Option<bool>,
    exit_on_conversion: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct EnrollRequest {
    contact_user_id: Option<Uuid>,
    contact_email: Option<String>,
    contact_name: Option<String>,
    contact_metadata: Option<serde_json::Value>,
    external_enrollment_id: Option<String>,
    next_step_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
struct ListEnrollmentsQuery {
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RecordEventRequest {
    enrollment_id: Uuid,
    event_type: String,
    channel: String,
    sequence_step_id: Option<Uuid>,
    link_clicked: Option<String>,
    ip_address: Option<String>,
    user_agent: Option<String>,
    metadata: Option<serde_json::Value>,
    conversion_entity_type: Option<String>,
    conversion_entity_id: Option<Uuid>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn create_campaign(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(req): Json<CreateCampaignRequest>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };

    let campaign_type = match CampaignType::try_from(req.campaign_type.as_str()) {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response();
        }
    };

    let goal_type = match req
        .goal_type
        .as_deref()
        .map(crate::types::pm::CampaignGoalType::try_from)
    {
        Some(Err(e)) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response();
        }
        Some(Ok(g)) => Some(g),
        None => None,
    };

    let payload = CreateCampaignPayload {
        parent_campaign_id: req.parent_campaign_id,
        name: req.name,
        campaign_type,
        goal_type,
        goal_entity_type: req.goal_entity_type,
        target_conversion_count: req.target_conversion_count,
        budget_cents: req.budget_cents,
        currency: req.currency,
        attribution_window_days: req.attribution_window_days,
        integration_id: req.integration_id,
        external_campaign_id: req.external_campaign_id,
        subject_entity_type: req.subject_entity_type,
        subject_entity_id: req.subject_entity_id,
        starts_at: req.starts_at,
        ends_at: req.ends_at,
        utm_source: req.utm_source,
        utm_medium: req.utm_medium,
        utm_campaign: req.utm_campaign,
        created_by_user_id: Some(current_user.id),
    };

    match CampaignService::create(&db, tenant_id, payload).await {
        Ok(c) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "campaign": c })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn list_campaigns(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(q): Query<ListCampaignsQuery>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };

    let campaign_type = match q.campaign_type.as_deref().map(CampaignType::try_from) {
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
    let status = match q.status.as_deref().map(CampaignStatus::try_from) {
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

    let filter = CampaignFilter {
        campaign_type,
        status,
        subject_entity_type: q.subject_entity_type,
        subject_entity_id: q.subject_entity_id,
        roots_only: q.roots_only,
        parent_campaign_id: q.parent_campaign_id,
    };

    match CampaignService::list(&db, tenant_id, filter).await {
        Ok(campaigns) => Json(serde_json::json!({ "campaigns": campaigns })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn get_campaign(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };
    match CampaignService::get(&db, tenant_id, id).await {
        Ok(c) => Json(serde_json::json!({ "campaign": c })).into_response(),
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
    let new_status = match CampaignStatus::try_from(req.status.as_str()) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response();
        }
    };
    match CampaignService::transition_status(&db, tenant_id, id, new_status).await {
        Ok(c) => Json(serde_json::json!({ "campaign": c })).into_response(),
        Err(e) if e.to_string().contains("Invalid transition") => (
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

async fn list_children(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };
    match CampaignService::find_children(&db, tenant_id, id).await {
        Ok(children) => Json(serde_json::json!({ "children": children })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn get_hierarchy_stats(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };
    match CampaignService::get_hierarchy_stats(&db, tenant_id, id).await {
        Ok(stats) => Json(serde_json::json!({ "stats": stats })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn add_sequence_step(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(req): Json<AddStepRequest>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };
    let step_type = match crate::types::pm::SequenceStepType::try_from(req.step_type.as_str()) {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response();
        }
    };
    let payload = CreateSequenceStepPayload {
        campaign_id: id,
        step_number: req.step_number,
        step_type,
        subject_template: req.subject_template,
        body_template: req.body_template,
        wait_days: req.wait_days,
        wait_hours: req.wait_hours,
        send_time_preference: req.send_time_preference,
        condition_type: req.condition_type,
        condition_value: req.condition_value,
        on_true_step: req.on_true_step,
        on_false_step: req.on_false_step,
        ab_variants: req.ab_variants,
        exit_on_reply: req.exit_on_reply,
        exit_on_conversion: req.exit_on_conversion,
    };
    match CampaignService::add_sequence_step(&db, tenant_id, payload).await {
        Ok(step) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "step": step })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn list_steps(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };
    match CampaignService::list_steps(&db, tenant_id, id).await {
        Ok(steps) => Json(serde_json::json!({ "steps": steps })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn enroll_contact(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(req): Json<EnrollRequest>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };
    let payload = EnrollContactPayload {
        campaign_id: id,
        contact_user_id: req.contact_user_id,
        contact_email: req.contact_email,
        contact_name: req.contact_name,
        contact_metadata: req.contact_metadata,
        external_enrollment_id: req.external_enrollment_id,
        next_step_at: req.next_step_at,
    };
    match CampaignService::enroll(&db, tenant_id, payload).await {
        Ok(enrollment) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "enrollment": enrollment })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn list_enrollments(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Query(q): Query<ListEnrollmentsQuery>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };
    let status_filter = match q.status.as_deref().map(EnrollmentStatus::try_from) {
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
    match CampaignService::list_enrollments(&db, tenant_id, id, status_filter).await {
        Ok(enrollments) => Json(serde_json::json!({ "enrollments": enrollments })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// Webhook-friendly event recording endpoint.
/// Does NOT require an authenticated user — callers pass the enrollment_id directly.
/// The tenant is resolved from the enrollment row itself.
async fn record_event(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(req): Json<RecordEventRequest>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };
    let event_type = match crate::types::pm::CampaignEventType::try_from(req.event_type.as_str()) {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response();
        }
    };
    let channel = match crate::types::pm::CampaignChannel::try_from(req.channel.as_str()) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response();
        }
    };
    let payload = RecordEventPayload {
        enrollment_id: req.enrollment_id,
        event_type,
        channel,
        sequence_step_id: req.sequence_step_id,
        link_clicked: req.link_clicked,
        ip_address: req.ip_address,
        user_agent: req.user_agent,
        metadata: req.metadata,
        conversion_entity_type: req.conversion_entity_type,
        conversion_entity_id: req.conversion_entity_id,
    };
    match CampaignService::record_event(&db, tenant_id, payload).await {
        Ok(event) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "event": event })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

// ── export_enrollments_csv ────────────────────────────────────────────────────
//
// GET /api/folio/campaigns/{id}/enrollments/export
//
// Returns a CSV suitable for uploading directly to a direct mail provider
// (PostGrid, Lob, USPS EDDM, Taradel, etc.).
//
// Columns: first_name, last_name, company, email, phone,
//          street_address, city, state, postal_code, country
//
// Supports members enrolled via enroll_leads_bulk OR enroll_contacts_bulk:
//   lead members    → contact_metadata.lead_id → atlas_leads
//   contact members → contact_metadata.contact_id → atlas_contacts + atlas_accounts
// Falls back to enrollment.contact_metadata values for any missing field.

#[derive(Deserialize)]
struct ExportQuery {
    /// Optional status filter. Defaults to all statuses.
    status: Option<String>,
}

async fn export_enrollments_csv(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Query(q): Query<ExportQuery>,
) -> Response {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(tid) => tid,
        Err(s) => return (s, "tenant resolution failed").into_response(),
    };

    let status_filter = match q.status.as_deref().map(EnrollmentStatus::try_from) {
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

    let enrollments =
        match CampaignService::list_enrollments(&db, tenant_id, id, status_filter).await {
            Ok(e) => e,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        };

    use std::collections::HashMap;

    // ── Collect which entity IDs appear in enrollment metadata ──────────────
    let mut lead_ids_in_meta: Vec<Uuid> = Vec::new();
    let mut contact_ids_in_meta: Vec<Uuid> = Vec::new();
    for e in &enrollments {
        if let Some(meta) = &e.contact_metadata {
            if let Some(id_val) = meta.get("lead_id").and_then(|v| v.as_str()) {
                if let Ok(uid) = id_val.parse::<Uuid>() {
                    lead_ids_in_meta.push(uid);
                }
            }
            if let Some(id_val) = meta.get("contact_id").and_then(|v| v.as_str()) {
                if let Ok(uid) = id_val.parse::<Uuid>() {
                    contact_ids_in_meta.push(uid);
                }
            }
        }
    }

    // ── Batch-load leads (tenant-scoped) ────────────────────────────────────
    let leads = if !lead_ids_in_meta.is_empty() {
        crate::entities::atlas_lead::Entity::find()
            .filter(crate::entities::atlas_lead::Column::TenantId.eq(tenant_id))
            .filter(crate::entities::atlas_lead::Column::Id.is_in(lead_ids_in_meta))
            .all(&db)
            .await
            .unwrap_or_default()
    } else {
        // Also load via email for backwards-compat with enrollments that predate metadata lead_id
        crate::entities::atlas_lead::Entity::find()
            .filter(crate::entities::atlas_lead::Column::TenantId.eq(tenant_id))
            .all(&db)
            .await
            .unwrap_or_default()
    };
    let lead_by_id: HashMap<Uuid, _> = leads.iter().map(|l| (l.id, l)).collect();
    let lead_by_email: HashMap<String, _> = leads
        .iter()
        .filter_map(|l| l.email.as_ref().map(|e| (e.to_lowercase(), l)))
        .collect();

    // ── Batch-load contacts ──────────────────────────────────────────────────
    let contacts = if !contact_ids_in_meta.is_empty() {
        crate::entities::atlas_contact::Entity::find()
            .filter(crate::entities::atlas_contact::Column::TenantId.eq(tenant_id))
            .filter(crate::entities::atlas_contact::Column::Id.is_in(contact_ids_in_meta))
            .all(&db)
            .await
            .unwrap_or_default()
    } else {
        vec![]
    };

    // ── Batch-load accounts for those contacts ───────────────────────────────
    let account_ids: Vec<Uuid> = contacts.iter().map(|c| c.account_id).collect();
    let accounts = if !account_ids.is_empty() {
        crate::entities::atlas_account::Entity::find()
            .filter(crate::entities::atlas_account::Column::Id.is_in(account_ids))
            .all(&db)
            .await
            .unwrap_or_default()
    } else {
        vec![]
    };
    let contact_by_id: HashMap<Uuid, _> = contacts.iter().map(|c| (c.id, c)).collect();
    let account_by_id: HashMap<Uuid, _> = accounts.iter().map(|a| (a.id, a)).collect();

    // ── Build CSV ────────────────────────────────────────────────────────────
    let mut csv = String::from(
        "first_name,last_name,company,email,phone,street_address,city,state,postal_code,country\r\n",
    );

    fn esc(s: &str) -> String {
        if s.contains(',') || s.contains('"') || s.contains('\n') {
            format!("\"{}\"", s.replace('"', "\"\""))
        } else {
            s.to_string()
        }
    }

    fn meta_str<'a>(meta: Option<&'a serde_json::Value>, key: &str) -> &'a str {
        meta.and_then(|m| m.get(key))
            .and_then(|v| v.as_str())
            .unwrap_or("")
    }

    for enrollment in &enrollments {
        let meta = enrollment.contact_metadata.as_ref();

        // ── Resolve which entity backs this enrollment ────────────────────
        // Priority: contact_metadata.lead_id > contact_metadata.contact_id
        //           > email fallback into lead table > raw metadata
        let lead_id = meta
            .and_then(|m| m.get("lead_id"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<Uuid>().ok());
        let contact_id = meta
            .and_then(|m| m.get("contact_id"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<Uuid>().ok());

        let lead = lead_id
            .and_then(|lid| lead_by_id.get(&lid).copied())
            .or_else(|| {
                enrollment
                    .contact_email
                    .as_deref()
                    .and_then(|e| lead_by_email.get(&e.to_lowercase()).copied())
            });

        let (contact, account) = if let Some(cid) = contact_id {
            let c = contact_by_id.get(&cid).copied();
            let a = c.and_then(|c| account_by_id.get(&c.account_id).copied());
            (c, a)
        } else {
            (None, None)
        };

        // ── first_name / last_name ───────────────────────────────────────
        let (first_name, last_name) = if let Some(l) = lead {
            (
                l.first_name.as_deref().unwrap_or(""),
                l.last_name.as_deref().unwrap_or(""),
            )
        } else if let Some(c) = contact {
            (
                c.first_name.as_deref().unwrap_or(""),
                c.last_name.as_deref().unwrap_or(""),
            )
        } else {
            // fall back: split enrollment.contact_name
            let full = enrollment.contact_name.as_deref().unwrap_or("");
            let mut parts = full.splitn(2, ' ');
            (parts.next().unwrap_or(""), parts.next().unwrap_or(""))
        };

        // ── company ──────────────────────────────────────────────────────
        let company = lead
            .and_then(|l| l.company.as_deref())
            .or_else(|| account.map(|a| a.name.as_str()))
            .or_else(|| meta_str(meta, "company").into())
            .unwrap_or("");

        // ── email ────────────────────────────────────────────────────────
        let email = lead
            .and_then(|l| l.email.as_deref())
            .or_else(|| contact.and_then(|c| c.email.as_deref()))
            .or(enrollment.contact_email.as_deref())
            .unwrap_or("");

        // ── phone ────────────────────────────────────────────────────────
        let phone = lead
            .and_then(|l| l.phone.as_deref())
            .or_else(|| contact.and_then(|c| c.phone.as_deref()))
            .or_else(|| account.and_then(|a| a.company_phone.as_deref()))
            .or_else(|| meta_str(meta, "phone").into())
            .unwrap_or("");

        // ── mailing address ──────────────────────────────────────────────
        let (street, city, state, zip, country) = if let Some(lead) = lead {
            let ma: Option<crate::types::shared::MailingAddress> =
                lead.mailing_address_typed().ok().flatten();
            (
                ma.as_ref()
                    .and_then(|a| a.street.as_deref())
                    .or(lead.street_address.as_deref())
                    .unwrap_or("")
                    .to_string(),
                ma.as_ref()
                    .and_then(|a| a.city.as_deref())
                    .or(lead.city.as_deref())
                    .unwrap_or("")
                    .to_string(),
                ma.as_ref()
                    .and_then(|a| a.state.as_deref())
                    .or(lead.state.as_deref())
                    .unwrap_or("")
                    .to_string(),
                ma.as_ref()
                    .and_then(|a| a.postal_code.as_deref())
                    .or(lead.postal_code.as_deref())
                    .unwrap_or("")
                    .to_string(),
                ma.as_ref()
                    .and_then(|a| a.country.as_deref())
                    .unwrap_or(&lead.country)
                    .to_string(),
            )
        } else if let Some(acct) = account {
            // Contact → account address
            (
                acct.street_address.as_deref().unwrap_or("").to_string(),
                acct.city.as_deref().unwrap_or("").to_string(),
                acct.state.as_deref().unwrap_or("").to_string(),
                acct.postal_code.as_deref().unwrap_or("").to_string(),
                acct.country.as_deref().unwrap_or("US").to_string(),
            )
        } else {
            // Fall back to enrollment.contact_metadata values
            (
                meta_str(meta, "street_address").to_string(),
                meta_str(meta, "city").to_string(),
                meta_str(meta, "state").to_string(),
                meta_str(meta, "zip").to_string(),
                {
                    let c = meta_str(meta, "country");
                    if c.is_empty() { "US" } else { c }
                }
                .to_string(),
            )
        };

        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{}\r\n",
            esc(first_name),
            esc(last_name),
            esc(company),
            esc(email),
            esc(phone),
            esc(&street),
            esc(&city),
            esc(&state),
            esc(&zip),
            esc(&country)
        ));
    }

    let filename = format!("campaign_{}_members.csv", id);
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/csv; charset=utf-8")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{filename}\""),
        )
        .body(Body::from(csv))
        .unwrap()
}

// ── enroll_leads_bulk ─────────────────────────────────────────────────────────
//
// POST /api/folio/campaigns/{id}/enroll-leads
// Body: { "lead_ids": ["uuid", ...] }
//
// Looks up each lead by ID (tenant-scoped), then enrolls them into the campaign.
// Returns a count of successfully enrolled + any errors.

#[derive(Deserialize)]
struct EnrollLeadsRequest {
    lead_ids: Vec<Uuid>,
}

async fn enroll_leads_bulk(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(campaign_id): Path<Uuid>,
    Json(req): Json<EnrollLeadsRequest>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };

    // Load all requested leads (tenant-scoped for safety)
    let leads = crate::entities::atlas_lead::Entity::find()
        .filter(crate::entities::atlas_lead::Column::TenantId.eq(tenant_id))
        .filter(crate::entities::atlas_lead::Column::Id.is_in(req.lead_ids.clone()))
        .all(&db)
        .await
        .unwrap_or_default();

    let mut enrolled = 0usize;
    let mut errors: Vec<String> = Vec::new();

    for lead in leads {
        let payload = EnrollContactPayload {
            campaign_id,
            contact_user_id: None,
            contact_email: lead.email.clone(),
            contact_name: Some(
                format!(
                    "{} {}",
                    lead.first_name.as_deref().unwrap_or(""),
                    lead.last_name.as_deref().unwrap_or(""),
                )
                .trim()
                .to_string(),
            )
            .filter(|s| !s.is_empty()),
            contact_metadata: Some(serde_json::json!({
                "lead_id": lead.id,
                "company": lead.company,
                "phone": lead.phone,
                "street_address": lead.street_address,
                "city": lead.city,
                "state": lead.state,
                "zip": lead.postal_code,
                "source": lead.source,
            })),
            external_enrollment_id: None,
            next_step_at: None,
        };
        match CampaignService::enroll(&db, tenant_id, payload).await {
            Ok(_) => enrolled += 1,
            Err(e) => errors.push(format!("lead {}: {}", lead.id, e)),
        }
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "enrolled": enrolled,
            "requested": req.lead_ids.len(),
            "errors": errors,
        })),
    )
        .into_response()
}

// ── enroll_contacts_bulk ──────────────────────────────────────────────────────
//
// POST /api/folio/campaigns/{id}/enroll-contacts
// Body: { "contact_ids": ["uuid", ...] }
//
// Resolves each contact's name/email from atlas_contacts and their mailing
// address from the linked atlas_account record. Stores all resolved data in
// enrollment.contact_metadata so the CSV export works without additional joins.

#[derive(Deserialize)]
struct EnrollContactsRequest {
    contact_ids: Vec<Uuid>,
}

async fn enroll_contacts_bulk(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(campaign_id): Path<Uuid>,
    Json(req): Json<EnrollContactsRequest>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };

    // ── Batch-load contacts (tenant-scoped) ──────────────────────────────────
    let contacts = crate::entities::atlas_contact::Entity::find()
        .filter(crate::entities::atlas_contact::Column::TenantId.eq(tenant_id))
        .filter(crate::entities::atlas_contact::Column::Id.is_in(req.contact_ids.clone()))
        .all(&db)
        .await
        .unwrap_or_default();

    // ── Batch-load linked accounts (one query, not N) ────────────────────────
    use std::collections::HashMap;
    let account_ids: Vec<Uuid> = contacts.iter().map(|c| c.account_id).collect();
    let accounts = if !account_ids.is_empty() {
        crate::entities::atlas_account::Entity::find()
            .filter(crate::entities::atlas_account::Column::Id.is_in(account_ids))
            .all(&db)
            .await
            .unwrap_or_default()
    } else {
        vec![]
    };
    let account_by_id: HashMap<Uuid, _> = accounts.into_iter().map(|a| (a.id, a)).collect();

    let mut enrolled = 0usize;
    let mut errors: Vec<String> = Vec::new();

    for contact in contacts {
        let acct = account_by_id.get(&contact.account_id);

        // Resolve display name — prefer full_name, fall back to first + last
        let contact_name = contact.full_name.clone().or_else(|| {
            let f = contact.first_name.as_deref().unwrap_or("");
            let l = contact.last_name.as_deref().unwrap_or("");
            let full = format!("{} {}", f, l).trim().to_string();
            if full.is_empty() { None } else { Some(full) }
        });

        let payload = EnrollContactPayload {
            campaign_id,
            contact_user_id: None,
            contact_email: contact.email.clone(),
            contact_name,
            contact_metadata: Some(serde_json::json!({
                // ID tags so the CSV export batch-loads without N+1
                "contact_id": contact.id,
                "account_id": contact.account_id,
                // Denormalized snapshot at enroll time
                "company":        acct.map(|a| a.name.as_str()).unwrap_or(""),
                "phone":          contact.phone.as_deref()
                                    .or_else(|| acct.and_then(|a| a.company_phone.as_deref()))
                                    .unwrap_or(""),
                "street_address": acct.and_then(|a| a.street_address.as_deref()).unwrap_or(""),
                "city":           acct.and_then(|a| a.city.as_deref()).unwrap_or(""),
                "state":          acct.and_then(|a| a.state.as_deref()).unwrap_or(""),
                "zip":            acct.and_then(|a| a.postal_code.as_deref()).unwrap_or(""),
                "country":        acct.and_then(|a| a.country.as_deref()).unwrap_or("US"),
            })),
            external_enrollment_id: None,
            next_step_at: None,
        };

        match CampaignService::enroll(&db, tenant_id, payload).await {
            Ok(_) => enrolled += 1,
            Err(e) => errors.push(format!("contact {}: {}", contact.id, e)),
        }
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "enrolled": enrolled,
            "requested": req.contact_ids.len(),
            "errors": errors,
        })),
    )
        .into_response()
}
