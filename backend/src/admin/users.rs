//! Admin — Platform User Invitations handler
//!
//! Manages invitations for new platform users.

use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Router,
};
use chrono::{Utc, Duration};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{platform_invite, user};

pub fn routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/admin/users/invites", get(list_invites))
        .route("/api/admin/users/invite", post(create_invite))
        .route("/api/admin/users/invites/{id}", delete(revoke_invite))
        .route("/api/admin/users/invites/{id}/resend", post(resend_invite))
}

// ── Models ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InviteResponse {
    pub id: Uuid,
    pub email: String,
    pub role: String,
    pub tenant: String,
    pub invited_by: String,
    pub sent: String,
    pub expires: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateInviteInput {
    pub email: String,
    pub role: String,
    pub tenant: String,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn list_invites(
    State(db): State<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    let list = platform_invite::Entity::find()
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list platform invites: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let response: Vec<InviteResponse> = list
        .into_iter()
        .map(|m| InviteResponse {
            id: m.id,
            email: m.email,
            role: m.role,
            tenant: m.tenant_name,
            invited_by: m.invited_by,
            sent: m.created_at.format("%b %d").to_string(),
            expires: m.expires_at.format("%b %d").to_string(),
        })
        .collect();

    Ok(Json(response))
}

pub async fn create_invite(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<CreateInviteInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let invited_by = format!("{} {}", current_user.first_name, current_user.last_name);
    let invited_by = invited_by.trim();
    let invited_by_str = if invited_by.is_empty() {
        current_user.email
    } else {
        invited_by.to_string()
    };

    let id = Uuid::new_v4();
    let created_at = Utc::now();
    let expires_at = created_at + Duration::days(7);

    let new_invite = platform_invite::ActiveModel {
        id: Set(id),
        email: Set(input.email.clone()),
        role: Set(input.role.clone()),
        tenant_name: Set(input.tenant.clone()),
        invited_by: Set(invited_by_str.clone()),
        created_at: Set(created_at),
        expires_at: Set(expires_at),
    };

    new_invite.insert(&db).await.map_err(|e| {
        tracing::error!("Failed to create platform invite: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let response = InviteResponse {
        id,
        email: input.email,
        role: input.role,
        tenant: input.tenant,
        invited_by: invited_by_str,
        sent: created_at.format("%b %d").to_string(),
        expires: expires_at.format("%b %d").to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn revoke_invite(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    platform_invite::Entity::delete_by_id(id)
        .exec(&db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to revoke platform invite: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::OK)
}

pub async fn resend_invite(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let invite = platform_invite::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut active: platform_invite::ActiveModel = invite.into();
    let now = Utc::now();
    active.created_at = Set(now);
    active.expires_at = Set(now + Duration::days(7));
    active.update(&db).await.map_err(|e| {
        tracing::error!("Failed to resend platform invite: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(StatusCode::OK)
}
