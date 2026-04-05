use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use sea_orm::{DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter, ActiveModelTrait, Set, PaginatorTrait};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use chrono::Utc;

use crate::entities::user::{self, Entity as User};
use crate::auth::hash_password;
use crate::handlers::sessions::create_user_session;

#[derive(Serialize)]
pub struct SetupStatusResponse {
    pub needs_setup: bool,
}

#[derive(Deserialize, Debug)]
pub struct SetupRequest {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub init_token: Option<String>,
}

pub fn public_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/setup/status", get(get_setup_status))
        .route("/setup/initialize", post(initialize_system))
        .route("/setup/purge_admin", post(purge_admin))
}

pub async fn purge_admin(
    State(db): State<DatabaseConnection>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if std::env::var("ENVIRONMENT").unwrap_or_default() == "production" {
        tracing::error!("Attempted to purge admin in production!");
        return Err((StatusCode::FORBIDDEN, Json(json!({ "message": "Cannot purge admin in production" }))));
    }

    user::Entity::delete_many()
        .filter(user::Column::IsAdmin.eq(true))
        .exec(&db)
        .await
        .map_err(|e| {
            let msg = format!("Database error: {:?}", e);
            tracing::error!("{}", msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": msg })))
        })?;

    tracing::info!("Admin users fully purged by dev command");
    Ok((StatusCode::OK, Json(json!({ "message": "Admin purged" }))))
}

pub async fn get_setup_status(
    State(db): State<DatabaseConnection>,
) -> Result<Json<SetupStatusResponse>, StatusCode> {
    // Check if any admin user exists
    let admin_count = User::find()
        .filter(user::Column::IsAdmin.eq(true))
        .count(&db)
        .await
        .map_err(|e| {
            tracing::error!("Database error when checking admin count: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(SetupStatusResponse {
        needs_setup: admin_count == 0,
    }))
}

pub async fn initialize_system(
    State(db): State<DatabaseConnection>,
    Json(req): Json<SetupRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // 0. Enforce INIT_TOKEN if the server requires it
    if let Ok(expected_token) = std::env::var("ATLAS_INIT_TOKEN") {
        if expected_token.trim().is_empty() == false {
            let provided = req.init_token.unwrap_or_default();
            if provided != expected_token {
                tracing::warn!("Unauthorized initialization attempt: token mismatch.");
                return Err((StatusCode::UNAUTHORIZED, Json(json!({ "message": "Invalid initialization token" }))));
            }
        }
    }

    // 1. Double check that NO admin exists yet
    let admin_count = User::find()
        .filter(user::Column::IsAdmin.eq(true))
        .count(&db)
        .await
        .map_err(|e| {
            let msg = format!("Database error: {:?}", e);
            tracing::error!("{}", msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": msg })))
        })?;

    if admin_count > 0 {
        return Err((StatusCode::FORBIDDEN, Json(json!({ "message": "System is already initialized" }))));
    }

    // 2. Hash the password
    let hashed_password = hash_password(&req.password)
        .map_err(|e| {
            let msg = format!("Error hashing password: {:?}", e);
            tracing::error!("{}", msg);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": msg })))
        })?;

    // 3. Create the admin user
    let new_user = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        username: Set(req.email.clone()), // Use email as username
        first_name: Set(req.first_name),
        last_name: Set(req.last_name),
        email: Set(req.email.clone()),
        password_hash: Set(hashed_password),
        phone: Set(String::new()),
        is_admin: Set(true),
        is_active: Set(true),
        last_login: Set(Some(Utc::now())),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let inserted_user = new_user.insert(&db).await.map_err(|e| {
        let msg = format!("Database error when creating user: {:?}", e);
        tracing::error!("{}", msg);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": msg })))
    })?;

    tracing::info!("Created first system admin user: {}", inserted_user.id);

    // 4. Create and return session
    let session_response = create_user_session(&db, &req.email, &req.password)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": "Failed to auto-authenticate after initialization" }))))?;

    Ok((StatusCode::CREATED, Json(session_response)))
}
