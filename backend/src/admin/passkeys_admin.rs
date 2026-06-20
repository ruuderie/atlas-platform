//! Admin — Passkeys management handler
//!
//! Allows super-admins to list all registered passkeys and revoke any of them.
//! Uses `GET /api/admin/passkeys` and `DELETE /api/admin/passkeys/{id}`.

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get},
    Json, Router,
};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{passkey, user};

pub fn routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/admin/passkeys", get(list_all_passkeys))
        .route("/api/admin/passkeys/{id}", delete(revoke_passkey_admin))
}

#[derive(Debug, Deserialize)]
pub struct PasskeyAdminQuery {
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct AdminPasskeyRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_email: String,
    pub name: String,
    pub sign_count: i32,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_all_passkeys(
    State(db): State<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
    Query(params): Query<PasskeyAdminQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut query = passkey::Entity::find()
        .order_by_desc(passkey::Column::CreatedAt);

    if let Some(uid) = params.user_id {
        query = query.filter(passkey::Column::UserId.eq(uid));
    }

    let passkeys = query
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Batch-fetch users to resolve emails
    let user_ids: Vec<Uuid> = passkeys.iter().map(|pk| pk.user_id).collect();
    let users = user::Entity::find()
        .filter(user::Column::Id.is_in(user_ids))
        .all(&db)
        .await
        .unwrap_or_default();
    let user_map: std::collections::HashMap<Uuid, String> =
        users.into_iter().map(|u| (u.id, u.email)).collect();

    let rows: Vec<AdminPasskeyRow> = passkeys
        .into_iter()
        .map(|pk| AdminPasskeyRow {
            id: pk.id,
            user_id: pk.user_id,
            user_email: user_map
                .get(&pk.user_id)
                .cloned()
                .unwrap_or_else(|| "unknown@example.com".to_string()),
            name: pk.name,
            sign_count: pk.sign_count,
            last_used_at: pk.last_used_at,
            created_at: pk.created_at,
        })
        .collect();

    Ok(Json(rows))
}

pub async fn revoke_passkey_admin(
    State(db): State<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let exists = passkey::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if exists.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    passkey::Entity::delete_by_id(id)
        .exec(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}
