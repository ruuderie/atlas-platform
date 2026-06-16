use axum::{
    extract::{Extension, Query, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set, ActiveModelTrait, PaginatorTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::entities::{atlas_verification_request, tenant, user};

#[derive(Deserialize)]
pub struct VerificationQuery {
    pub tenant_id: Option<Uuid>,
    pub status: Option<String>,
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
}

#[derive(Deserialize)]
pub struct RejectInput {
    pub reason: String,
}

pub async fn list_verification_requests(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(params): Query<VerificationQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Seed initial mock requests if table is empty
    let count = atlas_verification_request::Entity::find()
        .count(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if count == 0 {
        seed_mock_requests(&db).await.ok();
    }

    let mut query = atlas_verification_request::Entity::find()
        .order_by_desc(atlas_verification_request::Column::CreatedAt);

    if let Some(tenant_id) = params.tenant_id {
        query = query.filter(atlas_verification_request::Column::TenantId.eq(tenant_id));
    }

    if let Some(ref status) = params.status {
        if status != "all" {
            query = query.filter(atlas_verification_request::Column::Status.eq(status.clone()));
        }
    }

    let requests = query.all(&db).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e))
    })?;

    let mut response_list = Vec::new();
    for req in requests {
        // Find subject name (usually tenant name)
        let entity_name = if req.subject_type == "tenant" || req.subject_type == "Business" {
            if let Ok(Some(t)) = tenant::Entity::find_by_id(req.subject_id).one(&db).await {
                t.name
            } else {
                "Nexus Property Group".to_string()
            }
        } else {
            "João Carlos Silva".to_string()
        };

        response_list.push(VerificationRequestResponse {
            id: req.id,
            tenant_id: req.tenant_id,
            entity_name,
            req_type: req.subject_type,
            status: req.status,
            created_at: req.created_at,
            document_count: req.attachment_id.map(|_| 3).unwrap_or(2), // Mock document count for display
            rejection_reason: req.rejection_reason,
        });
    }

    Ok((StatusCode::OK, Json(response_list)))
}

pub async fn approve_request(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let request = atlas_verification_request::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Request not found".to_string()))?;

    let mut active: atlas_verification_request::ActiveModel = request.into();
    active.status = Set("approved".to_string());
    active.reviewed_by_user_id = Set(Some(current_user.id));
    active.reviewed_at = Set(Some(Utc::now()));

    let updated = active.update(&db).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update: {}", e))
    })?;

    Ok((StatusCode::OK, Json(updated)))
}

pub async fn reject_request(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<RejectInput>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let request = atlas_verification_request::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Request not found".to_string()))?;

    let mut active: atlas_verification_request::ActiveModel = request.into();
    active.status = Set("rejected".to_string());
    active.rejection_reason = Set(Some(input.reason));
    active.reviewed_by_user_id = Set(Some(current_user.id));
    active.reviewed_at = Set(Some(Utc::now()));

    let updated = active.update(&db).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update: {}", e))
    })?;

    Ok((StatusCode::OK, Json(updated)))
}

async fn seed_mock_requests(db: &DatabaseConnection) -> Result<(), String> {
    let tenants = tenant::Entity::find().all(db).await.map_err(|e| e.to_string())?;
    
    // Seed 3 request samples
    let sample_data = vec![
        ("Business", "Nexus Property Group"),
        ("Identity", "João Carlos Silva"),
        ("Document", "Ruud Logistics Corp"),
        ("Business", "Vizcaya STR Partners"),
        ("Identity", "Ana Carvalho"),
        ("Business", "Meridian Brokerage LLC"),
    ];

    for (idx, (req_type, _name)) in sample_data.into_iter().enumerate() {
        let tenant_id = tenants.get(idx % tenants.len()).map(|t| t.id).unwrap_or_else(Uuid::new_v4);
        let id = Uuid::new_v4();
        
        let vr = atlas_verification_request::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_id),
            subject_type: Set(req_type.to_string()),
            subject_id: Set(tenant_id), // Link subject to tenant itself
            requested_by_user_id: Set(Uuid::new_v4()),
            attachment_id: Set(Some(Uuid::new_v4())),
            status: Set(if idx == 5 { "approved".to_string() } else { "pending".to_string() }),
            created_at: Set(Utc::now() - chrono::Duration::days((idx + 1) as i64)),
            ..Default::default()
        };
        vr.insert(db).await.map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn authenticated_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/admin/verification-requests", get(list_verification_requests))
        .route("/api/admin/verification-requests/{id}/approve", post(approve_request))
        .route("/api/admin/verification-requests/{id}/reject", post(reject_request))
}
