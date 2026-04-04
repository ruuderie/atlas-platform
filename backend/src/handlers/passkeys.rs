use axum::{
    extract::{Extension, Json},
    http::StatusCode,
    routing::post,
    Router,
};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set};
use webauthn_rs::prelude::*;
use std::sync::Arc;
use uuid::Uuid;
use moka::future::Cache;
use crate::entities::{user, passkey};
use crate::auth::generate_jwt;

pub struct WebauthnStateRaw {
    pub webauthn: Arc<Webauthn>,
    pub reg_state: Cache<Uuid, PasskeyRegistration>,
    pub auth_state: Cache<Uuid, PasskeyAuthentication>,
}

pub type WebauthnState = Arc<WebauthnStateRaw>;

pub fn public_routes() -> Router<sea_orm::DatabaseConnection> {
    Router::new()
        .route("/api/passkeys/start-login", post(login_start))
        .route("/api/passkeys/finish-login", post(login_finish))
}

pub fn authenticated_routes() -> Router<sea_orm::DatabaseConnection> {
    Router::new()
        .route("/api/passkeys/start-register", post(register_start))
        .route("/api/passkeys/finish-register", post(register_finish))
}

pub async fn register_start(
    Extension(state): Extension<WebauthnState>,
    Extension(db): Extension<DatabaseConnection>,
    Extension(user): Extension<user::Model>,
) -> Result<Json<CreationChallengeResponse>, (StatusCode, String)> {
    
    let passkeys = passkey::Entity::find()
        .filter(passkey::Column::UserId.eq(user.id))
        .all(&db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        
    let exclude_credentials: Option<Vec<CredentialID>> = Some(
        passkeys.into_iter()
            .filter_map(|pk| CredentialID::try_from(pk.credential_id).ok())
            .collect()
    );

    let (ccr, reg_state) = state.webauthn.start_passkey_registration(
        user.id,
        user.email.as_str(),
        user.email.as_str(),
        exclude_credentials
    ).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to start registration: {:?}", e)))?;
    
    state.reg_state.insert(user.id, reg_state).await;

    Ok(Json(ccr))
}

pub async fn register_finish(
    Extension(state): Extension<WebauthnState>,
    Extension(db): Extension<DatabaseConnection>,
    Extension(user): Extension<user::Model>,
    Json(reg): Json<RegisterPublicKeyCredential>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    
    let reg_state = state.reg_state.get(&user.id).await
        .ok_or((StatusCode::BAD_REQUEST, "Registration challenge not found or expired".to_string()))?;
        
    let passkey_reg = state.webauthn.finish_passkey_registration(&reg, &reg_state)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Registration failed: {:?}", e)))?;
        
    // In webauthn-rs 0.5.4, passkey_reg is a Passkey object that we can store.
    
    let credential_id = passkey_reg.cred_id().clone();
    
    passkey::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user.id),
        credential_id: Set(credential_id.to_vec()),
        // Store whole passkey via serde json or extract pub key
        public_key: Set(serde_json::to_vec(&passkey_reg).unwrap()), // Hack for now to store the internal structure
        sign_count: Set(0),
        name: Set("My Passkey".to_string()),
        last_used_at: Set(None),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
    }.insert(&db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    state.reg_state.invalidate(&user.id).await;
    
    Ok(Json(serde_json::json!({"status": "ok"})))
}

#[derive(serde::Deserialize)]
pub struct LoginStartRequest {
    pub email: String,
}

pub async fn login_start(
    Extension(state): Extension<WebauthnState>,
    Extension(db): Extension<DatabaseConnection>,
    Json(req): Json<LoginStartRequest>,
) -> Result<Json<RequestChallengeResponse>, (StatusCode, String)> {
    
    let user = user::Entity::find()
        .filter(user::Column::Email.eq(&req.email))
        .one(&db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::BAD_REQUEST, "User not found".to_string()))?;

    let user_passkeys = passkey::Entity::find()
        .filter(passkey::Column::UserId.eq(user.id))
        .all(&db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let credentials: Vec<Passkey> = user_passkeys.into_iter().filter_map(|pk| {
        serde_json::from_slice(&pk.public_key).ok()
    }).collect();

    if credentials.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "No passkeys registered for this user".to_string()));
    }

    let (rcr, auth_state) = state.webauthn.start_passkey_authentication(&credentials)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    state.auth_state.insert(user.id, auth_state).await;

    Ok(Json(rcr))
}

#[derive(serde::Deserialize)]
pub struct LoginFinishRequest {
    pub email: String,
    pub response: PublicKeyCredential,
}

pub async fn login_finish(
    Extension(state): Extension<WebauthnState>,
    Extension(db): Extension<DatabaseConnection>,
    Json(req): Json<LoginFinishRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    
    let user = user::Entity::find()
        .filter(user::Column::Email.eq(&req.email))
        .one(&db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::BAD_REQUEST, "User not found".to_string()))?;

    let auth_state = state.auth_state.get(&user.id).await
        .ok_or((StatusCode::BAD_REQUEST, "Auth state not found".to_string()))?;

    let _auth_result = state.webauthn.finish_passkey_authentication(&req.response, &auth_state)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let token = generate_jwt(&user).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    state.auth_state.invalidate(&user.id).await;

    Ok(Json(serde_json::json!({
        "token": token,
        "user": user,
    })))
}
