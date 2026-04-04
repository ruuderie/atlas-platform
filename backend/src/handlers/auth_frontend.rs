use axum::{
    extract::{State, Query, Extension, Json},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use chrono::{Utc, Duration};
use rand::{distributions::Alphanumeric, Rng};

use crate::entities::{user, account, profile, user_account, magic_link_token};
use crate::auth::{hash_password, verify_password};
use crate::handlers::sessions::create_user_session;
use crate::models::user::UserRegistration;

#[derive(Deserialize)]
pub struct VerifyEmailQuery {
    pub token: String,
}

#[derive(Deserialize)]
pub struct LoginCredentials {
    pub email: String,
    pub password: String,
}

pub fn public_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/auth/verify-email", get(verify_email))
        .route("/api/auth/login", post(login))
        .route("/api/auth/magic-link/request", post(request_magic_link))
        .route("/api/auth/magic-link/verify", post(verify_magic_link))
        .route("/api/auth/webauthn/register", post(webauthn_register_start))
        .route("/api/auth/webauthn/authenticate", post(webauthn_auth_start))
}

pub fn authenticated_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/me", get(get_me))
}



pub async fn verify_email(
    State(db): State<DatabaseConnection>,
    Query(query): Query<VerifyEmailQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Stub for email verification logic
    // We would look up the user by the verification token and set is_active = true
    Ok((StatusCode::OK, Json(json!({"message": "Email verified successfully"}))))
}

pub async fn login(
    State(db): State<DatabaseConnection>,
    Json(credentials): Json<LoginCredentials>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user = user::Entity::find()
        .filter(user::Column::Email.eq(&credentials.email))
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

    let is_valid = verify_password(&credentials.password, &user.password_hash)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !is_valid {
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
    }

    if !user.is_active {
        return Err((StatusCode::UNAUTHORIZED, "Account is inactive. Please verify your email.".to_string()));
    }

    let session_response = create_user_session(&db, &credentials.email, &credentials.password)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create session".to_string()))?;

    Ok(Json(session_response))
}

pub async fn get_me(
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    Ok(Json(json!({
        "id": current_user.id,
        "email": current_user.email,
        "first_name": current_user.first_name,
        "last_name": current_user.last_name,
        "username": current_user.username,
        "is_active": current_user.is_active,
        "is_admin": current_user.is_admin,
    })))
}

pub async fn webauthn_register_start() -> Result<impl IntoResponse, StatusCode> {
    // Stub for WebAuthn registration
    Ok((StatusCode::NOT_IMPLEMENTED, "WebAuthn not fully implemented yet"))
}

pub async fn webauthn_auth_start() -> Result<impl IntoResponse, StatusCode> {
    // Stub for WebAuthn authentication
    Ok((StatusCode::NOT_IMPLEMENTED, "WebAuthn not fully implemented yet"))
}

#[derive(Deserialize)]
pub struct RequestMagicLinkPayload {
    pub email: String,
}

pub async fn request_magic_link(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<RequestMagicLinkPayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let user_model = user::Entity::find()
        .filter(user::Column::Email.eq(&payload.email))
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("Error finding user: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if let Some(user_mod) = user_model {
        let token_str: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();
            
        let token_entity = magic_link_token::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user_mod.id),
            token: Set(token_str.clone()),
            expires_at: Set(Utc::now() + Duration::minutes(15)),
            is_used: Set(false),
            created_at: Set(Utc::now()),
        };
        
        token_entity.insert(&db).await.map_err(|e| {
            tracing::error!("Error saving magic link: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        tracing::info!("Mock Email logic: Send Token [{}] to User Email [{}]", token_str, user_mod.email);
        // Will hook into communications.rs later
    }

    Ok((StatusCode::OK, Json(json!({"message": "If the email exists, a magic link has been sent."}))))
}

#[derive(Deserialize)]
pub struct VerifyMagicLinkPayload {
    pub token: String,
}

pub async fn verify_magic_link(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<VerifyMagicLinkPayload>,
) -> Result<(StatusCode, Json<crate::models::session::SessionResponse>), (StatusCode, String)> {
    let magic_link_opt = magic_link_token::Entity::find()
        .filter(magic_link_token::Column::Token.eq(&payload.token))
        .filter(magic_link_token::Column::IsUsed.eq(false))
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, "Database Error".to_string()))?;

    let magic_link = match magic_link_opt {
        Some(m) => m,
        None => return Err((StatusCode::UNAUTHORIZED, "Invalid or expired token".to_string())),
    };

    if magic_link.expires_at < Utc::now() {
        return Err((StatusCode::UNAUTHORIZED, "Token has expired".to_string()));
    }

    let user_mod = user::Entity::find_by_id(magic_link.user_id)
        .one(&db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database Error".to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "User not found".to_string()))?;

    // Mark as used
    let mut ml_active: magic_link_token::ActiveModel = magic_link.into();
    ml_active.is_used = Set(true);
    let _ = ml_active.update(&db).await;

    // We can use `create_session_for_user`
    let session_response = crate::handlers::sessions::create_session_for_user(&db, &user_mod)
        .await
        .map_err(|e| (e, "Failed to create session".to_string()))?;
    
    Ok((StatusCode::OK, Json(session_response)))
}
