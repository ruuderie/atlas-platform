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
use chrono::Utc;

use crate::entities::{user, account, profile, user_account};
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
        .route("/api/auth/register", post(register))
        .route("/api/auth/verify-email", get(verify_email))
        .route("/api/auth/login", post(login))
        .route("/api/auth/webauthn/register", post(webauthn_register_start))
        .route("/api/auth/webauthn/authenticate", post(webauthn_auth_start))
}

pub fn authenticated_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/me", get(get_me))
}

pub async fn register(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<UserRegistration>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Check if user already exists
    let existing_user = user::Entity::find()
        .filter(user::Column::Email.eq(&payload.email))
        .one(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if existing_user.is_some() {
        return Err((StatusCode::CONFLICT, "Email already in use".into()));
    }

    let hashed_password = hash_password(&payload.password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Create User (Initially inactive until email verified, but for now we'll set active to simplify flow)
    let new_user = user::ActiveModel {
        id: Set(Uuid::new_v4()),
        username: Set(payload.username.clone()),
        first_name: Set(payload.first_name),
        last_name: Set(payload.last_name),
        email: Set(payload.email.clone()),
        phone: Set(payload.phone),
        password_hash: Set(hashed_password),
        is_admin: Set(false),
        is_active: Set(true), // TODO: Set false and require email verification
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        last_login: Set(None),
    };

    let inserted_user = new_user.insert(&db).await.map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    // Do NOT automatically create an Account here.
    // The specification clarifies: "Registration creates a user only. Users create an Account post-registration."

    Ok((StatusCode::CREATED, Json(json!({"message": "User registered", "user_id": inserted_user.id}))))
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
