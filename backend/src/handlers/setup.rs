use axum::{
    extract::{Json, State, Extension},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use sea_orm::{DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter, ActiveModelTrait, Set, PaginatorTrait, TransactionTrait};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
use chrono::Utc;

use crate::entities::user::{self, Entity as User};
use crate::entities::{passkey, user_account};
use crate::entities::magic_link_token;
use crate::auth::hash_password;
use crate::handlers::sessions::create_passwordless_session;
use crate::handlers::passkeys::WebauthnState;
use webauthn_rs::prelude::*;
use once_cell::sync::Lazy;
use moka::future::Cache;
use std::time::Duration;
use crate::handlers::communications::{send_email_handler, SendEmailPayload};

#[derive(Serialize)]
pub struct SetupStatusResponse {
    pub needs_setup: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SetupRequest {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub init_token: Option<String>,
}

#[derive(Clone)]
pub struct SetupSessionPayload {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub init_token: Option<String>,
}

static SETUP_CACHE: Lazy<Cache<Uuid, SetupSessionPayload>> = Lazy::new(|| {
    Cache::builder()
        .time_to_live(Duration::from_secs(600))
        .build()
});

pub fn public_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/setup/status", get(get_setup_status))
        .route("/setup/webauthn-start", post(webauthn_start))
        .route("/setup/initialize-finish", post(initialize_finish))
}

pub async fn get_setup_status(
    State(db): State<DatabaseConnection>,
) -> Result<Json<SetupStatusResponse>, StatusCode> {
    // Admin status is now a role in user_account, not a field on the user entity.
    let admin_count = user_account::Entity::find()
        .filter(user_account::Column::Role.eq(crate::entities::user_account::UserRole::PlatformSuperAdmin))
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

pub async fn webauthn_start(
    State(db): State<DatabaseConnection>,
    Extension(webauthn_state): Extension<WebauthnState>,
    Json(req): Json<SetupRequest>,
) -> Result<Json<(Uuid, CreationChallengeResponse)>, (StatusCode, Json<serde_json::Value>)> {
    // Enforce INIT_TOKEN
    if let Ok(expected_token) = std::env::var("ATLAS_INIT_TOKEN") {
        if !expected_token.trim().is_empty() {
            let provided = req.init_token.clone().unwrap_or_default();
            if provided.is_empty() || provided != expected_token {
                return Err((StatusCode::UNAUTHORIZED, Json(json!({ 
                    "message": "Invalid or missing initialization token." 
                }))));
            }
        }
    }

    // Check admin count via user_account role instead of the removed user.is_admin field.
    let admin_count = user_account::Entity::find()
        .filter(user_account::Column::Role.eq(crate::entities::user_account::UserRole::PlatformSuperAdmin))
        .count(&db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": e.to_string() }))))?;

    if admin_count > 0 {
        return Err((StatusCode::FORBIDDEN, Json(json!({ "message": "System is already initialized" }))));
    }

    let user_unique_id = Uuid::new_v4();

    // Use the platform's primary origin for the setup flow.
    // The setup endpoint is served from the admin origin which is seeded at startup.
    let platform_origin = std::env::var("ANCHOR_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    let webauthn = webauthn_state.registry.get_or_create(&platform_origin).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": format!("WebAuthn registry error: {e}") }))))?;

    let (ccr, reg_state) = webauthn.start_passkey_registration(
        user_unique_id,
        req.email.as_str(),
        req.email.as_str(),
        None
    ).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": format!("WebAuthn error: {:?}", e) }))))?;

    webauthn_state.reg_state.insert(user_unique_id, reg_state).await;
    
    SETUP_CACHE.insert(user_unique_id, SetupSessionPayload {
        email: req.email,
        first_name: req.first_name,
        last_name: req.last_name,
        init_token: req.init_token,
    }).await;

    Ok(Json((user_unique_id, ccr)))
}

#[derive(Deserialize)]
pub struct InitializeFinishRequest {
    pub session_id: Uuid,
    pub webauthn_response: RegisterPublicKeyCredential,
}

pub async fn initialize_finish(
    State(db): State<DatabaseConnection>,
    Extension(webauthn_state): Extension<WebauthnState>,
    Json(req): Json<InitializeFinishRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    
    let setup_payload = SETUP_CACHE.get(&req.session_id).await
        .ok_or((StatusCode::BAD_REQUEST, Json(json!({ "message": "Setup session expired or invalid" }))))?;

    let reg_state = webauthn_state.reg_state.get(&req.session_id).await
        .ok_or((StatusCode::BAD_REQUEST, Json(json!({ "message": "Registration challenge expired" }))))?;

    // Check admin count via user_account role.
    let admin_count = user_account::Entity::find()
        .filter(user_account::Column::Role.eq(crate::entities::user_account::UserRole::PlatformSuperAdmin))
        .count(&db)
        .await
        .unwrap_or(0);
    if admin_count > 0 {
        return Err((StatusCode::FORBIDDEN, Json(json!({ "message": "System already initialized" }))));
    }

    // Resolve the setup-flow Webauthn instance via the registry (same origin as webauthn_start).
    let platform_origin = std::env::var("ANCHOR_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    let webauthn = webauthn_state.registry.get_or_create(&platform_origin).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": format!("WebAuthn registry error: {e}") }))))?;

    let passkey_reg = webauthn.finish_passkey_registration(&req.webauthn_response, &reg_state)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(json!({ "message": format!("Registration failed: {:?}", e) }))))?;

    let random_pwd = hash_password(&Uuid::new_v4().to_string()).unwrap();

    // Start transaction
    let txn = db.begin().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": e.to_string() }))))?;
    
    // Create Admin user — is_admin removed from user entity; admin status tracked via user_account role.
    let new_user = user::ActiveModel {
        id: Set(req.session_id),
        username: Set(setup_payload.email.clone()),
        first_name: Set(setup_payload.first_name),
        last_name: Set(setup_payload.last_name),
        email: Set(setup_payload.email.clone()),
        password_hash: Set(random_pwd),
        phone: Set(String::new()),
        is_active: Set(true),
        last_login: Set(Some(Utc::now())),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };
    
    match new_user.insert(&txn).await {
        Ok(_) => {},
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": e.to_string() })))),
    }

    // Grant PlatformSuperAdmin role — is_admin was removed from the user entity.
    let platform_role = user_account::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(req.session_id),
        account_id: Set(Uuid::nil()), // Platform-level sentinel; no real account for super admin
        role: Set(user_account::UserRole::PlatformSuperAdmin),
        is_active: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };
    match platform_role.insert(&txn).await {
        Ok(_) => {},
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": e.to_string() })))),
    }

    // Create Passkey
    let credential_id = passkey_reg.cred_id().clone();
    let passkey_model = passkey::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(req.session_id),
        credential_id: Set(credential_id.to_vec()),
        public_key: Set(
            serde_json::to_vec(&passkey_reg)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": format!("Passkey serialisation failed: {e}") }))))?),
        sign_count: Set(0),
        name: Set("System Admin Passkey".to_string()),
        last_used_at: Set(None),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    match passkey_model.insert(&txn).await {
        Ok(_) => {},
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": e.to_string() })))),
    }

    txn.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": e.to_string() }))))?;

    webauthn_state.reg_state.invalidate(&req.session_id).await;
    SETUP_CACHE.invalidate(&req.session_id).await;
    
    tracing::info!("Created first system admin user and passkey: {}", req.session_id);

    // Provide welcome session
    let session_response = create_passwordless_session(&db, &setup_payload.email)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": "Failed to auto-authenticate after initialization" }))))?;

    // Auto-dispatch verification email to let them know it's fully active
    let frontend_url = std::env::var("ADMIN_URL").unwrap_or_else(|_| "https://uat.atlas.oply.co".to_string());
    let email_payload = SendEmailPayload {
        tenant_id: Uuid::nil(),
        to_email: setup_payload.email.clone(),
        subject: "Welcome to Atlas Platform!".to_string(),
        body_html: format!("<h2>Atlas Platform Initialized</h2><p>Your administrator profile has been successfully generated and bound to your WebAuthn passkey.</p><br><a href=\"{0}/login\">Access the Platform</a>", frontend_url),
        attachments: Vec::new(),
    };
    let _ = send_email_handler(State(db.clone()), Json(email_payload)).await;

    Ok((StatusCode::CREATED, Json(session_response)))
}
