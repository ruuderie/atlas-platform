use axum::{
    Json, Router,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{attachment, atlas_verification_request, tenant, user};
use crate::services::verification_service::VerificationService;
use crate::types::verification::{
    VerificationRequestType, VerificationStatus, VerificationSubjectType,
};

#[derive(Deserialize)]
pub struct VerificationQuery {
    pub tenant_id: Option<Uuid>,
    pub status: Option<String>,
}

#[derive(Serialize)]
pub struct AttachmentSummary {
    pub id: Uuid,
    pub title: Option<String>,
    pub url: String,
    pub mime_type: String,
}

#[derive(Serialize)]
pub struct VerificationRequestResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub entity_name: String,
    pub req_type: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub document_count: u32,
    pub rejection_reason: Option<String>,
    pub reviewer_notes: Option<String>,
    pub attachment: Option<AttachmentSummary>,
}

#[derive(Deserialize)]
pub struct RejectInput {
    pub reason: String,
}

#[derive(Deserialize)]
pub struct CreateVerificationInput {
    pub request_type: VerificationRequestType,
    pub subject_type: VerificationSubjectType,
    pub subject_id: Option<Uuid>,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct NotesInput {
    pub notes: String,
}

#[derive(Deserialize)]
pub struct RequestInfoInput {
    pub message: Option<String>,
}

#[derive(Serialize)]
pub struct CreateVerificationResponse {
    pub id: Uuid,
}

fn parse_status_filter(raw: &str) -> Result<Option<VerificationStatus>, (StatusCode, String)> {
    if raw == "all" {
        return Ok(None);
    }
    raw.parse::<VerificationStatus>()
        .map(Some)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))
}

async fn resolve_entity_name(
    db: &DatabaseConnection,
    subject_type: &str,
    subject_id: Uuid,
) -> String {
    let st = subject_type.to_ascii_lowercase();
    if st == "tenant" || st == "business" {
        if let Ok(Some(t)) = tenant::Entity::find_by_id(subject_id).one(db).await {
            return t.name;
        }
    }
    if st == "user" || st == "identity" {
        if let Ok(Some(u)) = user::Entity::find_by_id(subject_id).one(db).await {
            let name = format!("{} {}", u.first_name, u.last_name)
                .trim()
                .to_string();
            if !name.is_empty() {
                return name;
            }
            return u.email;
        }
    }
    // Fallback: tenant name for the request's subject when subject row is missing
    if let Ok(Some(t)) = tenant::Entity::find_by_id(subject_id).one(db).await {
        return t.name;
    }
    subject_id.to_string()
}

fn resolve_req_type(req: &atlas_verification_request::Model) -> String {
    if let Some(ref rt) = req.request_type {
        if !rt.is_empty() {
            return rt.clone();
        }
    }
    // Legacy rows stored category in subject_type
    req.subject_type.clone()
}

async fn to_response(
    db: &DatabaseConnection,
    req: atlas_verification_request::Model,
) -> VerificationRequestResponse {
    let entity_name = resolve_entity_name(db, &req.subject_type, req.subject_id).await;
    let document_count = if req.attachment_id.is_some() { 1 } else { 0 };
    let attachment = if let Some(aid) = req.attachment_id {
        match attachment::Entity::find_by_id(aid).one(db).await {
            Ok(Some(a)) => Some(AttachmentSummary {
                id: a.id,
                title: a.title.clone(),
                url: a.url.clone(),
                mime_type: a.mime_type.clone(),
            }),
            _ => Some(AttachmentSummary {
                id: aid,
                title: None,
                url: String::new(),
                mime_type: String::new(),
            }),
        }
    } else {
        None
    };

    VerificationRequestResponse {
        id: req.id,
        tenant_id: req.tenant_id,
        entity_name,
        req_type: resolve_req_type(&req),
        status: req.status,
        created_at: req.created_at,
        document_count,
        rejection_reason: req.rejection_reason,
        reviewer_notes: req.reviewer_notes,
        attachment,
    }
}

pub async fn list_verification_requests(
    State(db): State<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
    Query(params): Query<VerificationQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut query = atlas_verification_request::Entity::find()
        .order_by_desc(atlas_verification_request::Column::CreatedAt);

    if let Some(tenant_id) = params.tenant_id {
        query = query.filter(atlas_verification_request::Column::TenantId.eq(tenant_id));
    }

    if let Some(ref status) = params.status {
        if let Some(parsed) = parse_status_filter(status)? {
            query = query.filter(atlas_verification_request::Column::Status.eq(parsed.to_string()));
        }
    }

    let requests = query.all(&db).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let mut response_list = Vec::with_capacity(requests.len());
    for req in requests {
        response_list.push(to_response(&db, req).await);
    }

    Ok((StatusCode::OK, Json(response_list)))
}

pub async fn approve_request(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let updated = VerificationService::complete_verification_global(
        &db,
        id,
        VerificationStatus::Approved,
        None,
        current_user.id,
    )
    .await
    .map_err(|e| {
        let code = if e.contains("not found") {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };
        (code, e)
    })?;

    Ok((StatusCode::OK, Json(to_response(&db, updated).await)))
}

pub async fn reject_request(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<RejectInput>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if input.reason.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Rejection reason is required".to_string(),
        ));
    }

    let updated = VerificationService::complete_verification_global(
        &db,
        id,
        VerificationStatus::Rejected,
        Some(input.reason.trim()),
        current_user.id,
    )
    .await
    .map_err(|e| {
        let code = if e.contains("not found") {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };
        (code, e)
    })?;

    Ok((StatusCode::OK, Json(to_response(&db, updated).await)))
}

pub async fn add_notes(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<NotesInput>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let updated = VerificationService::add_reviewer_note(
        &db,
        id,
        &input.notes,
        current_user.id,
    )
    .await
    .map_err(|e| {
        let code = if e.contains("not found") {
            StatusCode::NOT_FOUND
        } else if e.contains("empty") {
            StatusCode::BAD_REQUEST
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };
        (code, e)
    })?;

    Ok((StatusCode::OK, Json(to_response(&db, updated).await)))
}

pub async fn request_info(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<RequestInfoInput>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let updated = VerificationService::request_more_info(
        &db,
        id,
        input.message.as_deref(),
        current_user.id,
    )
    .await
    .map_err(|e| {
        let code = if e.contains("not found") {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };
        (code, e)
    })?;

    Ok((StatusCode::OK, Json(to_response(&db, updated).await)))
}

/// POST /api/folio/verification-requests — tenant-scoped create.
pub async fn create_folio_verification_request(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<CreateVerificationInput>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let tenant_id = resolve_tenant_id(&db, current_user.id)
        .await
        .map_err(|c| (c, "Unable to resolve tenant".to_string()))?;

    let subject_id = match input.subject_id {
        Some(id) => id,
        None => match input.subject_type {
            VerificationSubjectType::User => current_user.id,
            VerificationSubjectType::Tenant | VerificationSubjectType::Asset => tenant_id,
        },
    };

    let id = VerificationService::create_verification_request(
        &db,
        tenant_id,
        input.request_type,
        input.subject_type,
        subject_id,
        current_user.id,
        VerificationStatus::Pending,
        input.notes.as_deref(),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok((
        StatusCode::CREATED,
        Json(CreateVerificationResponse { id }),
    ))
}

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

pub fn authenticated_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route(
            "/api/admin/verification-requests",
            get(list_verification_requests),
        )
        .route(
            "/api/admin/verification-requests/{id}/approve",
            post(approve_request),
        )
        .route(
            "/api/admin/verification-requests/{id}/reject",
            post(reject_request),
        )
        .route(
            "/api/admin/verification-requests/{id}/notes",
            post(add_notes),
        )
        .route(
            "/api/admin/verification-requests/{id}/request-info",
            post(request_info),
        )
}

/// Folio tenant routes (landlord+ shared create).
pub fn folio_routes() -> Router<DatabaseConnection> {
    Router::new().route(
        "/api/folio/verification-requests",
        post(create_folio_verification_request),
    )
}
