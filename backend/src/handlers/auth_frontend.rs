use axum::{
    extract::{State, Query, Extension, Json},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;
use chrono::{Utc, Duration};
use rand::{distributions::Alphanumeric, Rng};

use crate::entities::{user, account, user_account, magic_link_token};
use crate::auth::verify_password;
use crate::handlers::sessions::create_user_session;

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
        .route("/api/auth/flow/{email}", get(get_auth_flow))
        .route("/api/auth/magic-link/request", post(request_magic_link))
        .route("/api/auth/magic-link/verify", post(verify_magic_link))
        .route("/api/auth/webauthn/register", post(webauthn_register_start))
        .route("/api/auth/webauthn/authenticate", post(webauthn_auth_start))
}

pub fn authenticated_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/me", get(get_me))
}

#[derive(serde::Serialize)]
pub struct AuthFlowResponse {
    pub has_passkey: bool,
}

pub async fn get_auth_flow(
    State(db): State<DatabaseConnection>,
    axum::extract::Path(email): axum::extract::Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_model = user::Entity::find()
        .filter(user::Column::Email.eq(&email))
        .one(&db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database Error".to_string()))?;

    let has_passkey = if let Some(user_mod) = user_model {
        let passkeys = crate::entities::passkey::Entity::find()
            .filter(crate::entities::passkey::Column::UserId.eq(user_mod.id))
            .all(&db)
            .await
            .unwrap_or_default();
        passkeys.len() > 0
    } else {
        false
    };

    Ok((StatusCode::OK, Json(AuthFlowResponse { has_passkey })))
}

pub async fn verify_email(
    State(_db): State<DatabaseConnection>,
    Query(_query): Query<VerifyEmailQuery>,
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
        
        let frontend_url = std::env::var("ADMIN_URL").unwrap_or_else(|_| "https://uat.atlas.oply.co".to_string());
        let setup_link = format!("{}/verify-token/{}", frontend_url, token_str);
        
        let email_payload = crate::handlers::communications::SendEmailPayload {
            tenant_id: Uuid::nil(),
            to_email: user_mod.email.clone(),
            subject: "Your Atlas Platform Setup Token".to_string(),
            body_html: format!("<h2>Atlas Platform Access</h2><p>Click the link below to securely log in to your account and configure your device passkey:</p><br><a href=\"{0}\">{0}</a>", setup_link),
        };

        if let Err((status, msg)) = crate::handlers::communications::send_email_handler(State(db.clone()), Json(email_payload)).await {
            tracing::error!("Failed to dispatch setup token email natively: {}", msg);
        } else {
            tracing::info!("Successfully dispatched Setup Token routing to {}", user_mod.email);
        }
    }

    Ok((StatusCode::OK, Json(json!({"message": "If the email exists, a magic link has been sent."}))))
}

#[derive(Deserialize)]
pub struct VerifyMagicLinkPayload {
    pub token: String,
    pub tenant_id: Option<Uuid>,
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
        .map_err(|_e| (StatusCode::INTERNAL_SERVER_ERROR, "Database Error".to_string()))?;

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

    // Validate tenant access if tenant_id is provided
    if let Some(target_tenant_id) = payload.tenant_id {
        let user_accounts = user_account::Entity::find()
            .filter(user_account::Column::UserId.eq(user_mod.id))
            .all(&db)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database Error".to_string()))?;
            
        let mut has_access = false;
        for ua in user_accounts {
            let acc = account::Entity::find_by_id(ua.account_id)
                .one(&db)
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database Error".to_string()))?;
            if let Some(acc) = acc {
                if acc.tenant_id == target_tenant_id {
                    has_access = true;
                    break;
                }
            }
        }
        
        if !has_access {
            return Err((StatusCode::UNAUTHORIZED, "User does not have access to this tenant".to_string()));
        }
    }

    // Mark as used
    // Invalidate all existing passkeys for this user because they used a setup token
    let _ = crate::entities::passkey::Entity::delete_many()
        .filter(crate::entities::passkey::Column::UserId.eq(user_mod.id))
        .exec(&db)
        .await;

    let mut ml_active: magic_link_token::ActiveModel = magic_link.into();
    ml_active.is_used = Set(true);
    let _ = ml_active.update(&db).await;

    // We can use `create_session_for_user`
    let session_response = crate::handlers::sessions::create_session_for_user(&db, &user_mod)
        .await
        .map_err(|e| (e, "Failed to create session".to_string()))?;
    
    Ok((StatusCode::OK, Json(session_response)))
}

