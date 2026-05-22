use axum::{
    extract::{Extension, Json},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use axum_extra::extract::cookie::CookieJar;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set, PaginatorTrait};
use webauthn_rs::prelude::*;
use std::sync::Arc;
use uuid::Uuid;
use moka::future::Cache;
use std::time::Instant;
use crate::entities::{user, passkey, webauthn_challenge};
use crate::auth::generate_jwt;
use crate::handlers::sessions::session_cookie_header;
use crate::webauthn_registry::WebauthnRegistry;
use crate::metrics;


pub struct WebauthnStateRaw {
    pub registry: Arc<WebauthnRegistry>,
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
        .route("/api/passkeys/has-passkey", get(has_passkey))
}

pub async fn has_passkey(
    Extension(db): Extension<DatabaseConnection>,
    Extension(user): Extension<user::Model>,
) -> Json<serde_json::Value> {
    let start = Instant::now();
    let request_id = uuid::Uuid::new_v4();

    let count = passkey::Entity::find()
        .filter(passkey::Column::UserId.eq(user.id))
        .count(&db)
        .await
        .unwrap_or(0);

    tracing::info!(
        event = "passkey.has_passkey.checked",
        request_id = %request_id,
        user_id = %user.id,
        duration_ms = start.elapsed().as_millis(),
        has_passkey = count > 0
    );

    Json(serde_json::json!({ "has_passkey": count > 0 }))
}

pub async fn register_start(
    Extension(state): Extension<WebauthnState>,
    Extension(db): Extension<DatabaseConnection>,
    Extension(user): Extension<user::Model>,
    headers: axum::http::HeaderMap,
) -> Result<Json<CreationChallengeResponse>, (StatusCode, String)> {
    let start = Instant::now();
    let request_id = uuid::Uuid::new_v4();

    let app_instance_id = headers
        .get("x-app-instance-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let origin = headers.get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .ok_or((StatusCode::BAD_REQUEST, "Missing or invalid Origin header".to_string()))?;

    let webauthn = state.registry.get_or_create(origin).await
        .map_err(|e: String| (StatusCode::BAD_REQUEST, e))?;

    let passkeys = passkey::Entity::find()
        .filter(passkey::Column::UserId.eq(user.id))
        .all(&db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        
    let exclude_credentials: Option<Vec<CredentialID>> = Some(
        passkeys.into_iter()
            .filter_map(|pk| CredentialID::try_from(pk.credential_id).ok())
            .collect()
    );

    let (mut ccr, reg_state) = webauthn.start_passkey_registration(
        user.id,
        user.email.as_str(),
        user.email.as_str(),
        exclude_credentials
    ).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to start registration: {:?}", e)))?;

    match ccr.public_key.authenticator_selection {
        Some(ref mut sel) => {
            sel.require_resident_key = true;
            sel.resident_key = serde_json::from_value(serde_json::json!("required"))
                .map_err(|e| (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to set resident_key on existing authenticator_selection: {e}"),
                ))?;
        }
        None => {
            ccr.public_key.authenticator_selection = Some(
                serde_json::from_value(serde_json::json!({
                    "requireResidentKey": true,
                    "residentKey": "required"
                }))
                .map_err(|e| (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to build AuthenticatorSelectionCriteria: {e}"),
                ))?
            );
        }
    }
    prune_expired_challenges(&db).await;
    save_challenge(&db, user.id, &reg_state, "registration").await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    metrics::PASSKEY_REGISTRATION_STARTED
        .with_label_values(&["unknown", &app_instance_id])
        .inc();

    tracing::info!(
        event = "passkey.registration.started",
        request_id = %request_id,
        user_id = %user.id,
        app_instance_id = %app_instance_id,
        duration_ms = start.elapsed().as_millis(),
        status = "success"
    );

    Ok(Json(ccr))
}

pub async fn register_finish(
    Extension(state): Extension<WebauthnState>,
    Extension(db): Extension<DatabaseConnection>,
    Extension(user): Extension<user::Model>,
    headers: axum::http::HeaderMap,
    Json(reg): Json<RegisterPublicKeyCredential>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let start = Instant::now();
    let request_id = uuid::Uuid::new_v4();

    let app_instance_id = headers
        .get("x-app-instance-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let origin = headers.get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .ok_or((StatusCode::BAD_REQUEST, "Missing or invalid Origin header".to_string()))?;

    let webauthn = state.registry.get_or_create(origin).await
        .map_err(|e: String| (StatusCode::BAD_REQUEST, e))?;

    let reg_state: PasskeyRegistration = get_challenge(&db, user.id, "registration").await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Registration challenge not found or expired: {e}")))?;
        
    let passkey_reg = webauthn.finish_passkey_registration(&reg, &reg_state)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Registration failed: {:?}", e)))?;
        
    let credential_id = passkey_reg.cred_id().clone();
    
    passkey::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user.id),
        credential_id: Set(credential_id.to_vec()),
        public_key: Set(
            serde_json::to_vec(&passkey_reg)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to serialise passkey: {e}")))?
        ),
        sign_count: Set(0),
        name: Set("My Passkey".to_string()),
        last_used_at: Set(None),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
    }.insert(&db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let _ = delete_challenge(&db, user.id).await;

    metrics::PASSKEY_REGISTRATION_SUCCESS
        .with_label_values(&["unknown", &app_instance_id])
        .inc();

    tracing::info!(
        event = "passkey.registration.success",
        request_id = %request_id,
        user_id = %user.id,
        app_instance_id = %app_instance_id,
        duration_ms = start.elapsed().as_millis(),
        status = "success"
    );

    Ok(Json(serde_json::json!({"status": "ok"})))
}

#[derive(serde::Deserialize)]
pub struct LoginStartRequest {
    pub email: String,
}

pub async fn login_start(
    Extension(state): Extension<WebauthnState>,
    Extension(db): Extension<DatabaseConnection>,
    headers: axum::http::HeaderMap,
    Json(req): Json<LoginStartRequest>,
) -> Result<Response, (StatusCode, String)> {
    let start = Instant::now();
    let request_id = uuid::Uuid::new_v4();

    let app_instance_id = headers
        .get("x-app-instance-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let origin = headers.get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .ok_or((StatusCode::BAD_REQUEST, "Missing or invalid Origin header".to_string()))?;

    let webauthn = state.registry.get_or_create(origin).await
        .map_err(|e: String| (StatusCode::BAD_REQUEST, e))?;

    let credentials = if req.email.trim().is_empty() {
        vec![]
    } else {
        let user = user::Entity::find()
            .filter(user::Column::Email.eq(req.email.trim()))
            .one(&db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .ok_or((StatusCode::BAD_REQUEST, "User not found".to_string()))?;

        let user_passkeys = passkey::Entity::find()
            .filter(passkey::Column::UserId.eq(user.id))
            .all(&db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let creds: Vec<Passkey> = user_passkeys.into_iter().filter_map(|pk| {
            serde_json::from_slice(&pk.public_key).ok()
        }).collect();

        if creds.is_empty() {
            return Err((StatusCode::BAD_REQUEST, "No passkeys registered for this user".to_string()));
        }
        creds
    };

    let (rcr, auth_state) = webauthn.start_passkey_authentication(&credentials)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let session_id = Uuid::new_v4();
    prune_expired_challenges(&db).await;
    save_challenge(&db, session_id, &auth_state, "authentication").await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let cookie = format!(
        "passkey_session={}; Path=/api/passkeys; HttpOnly; Secure; Max-Age=300; SameSite=Strict",
        session_id
    );
    
    tracing::info!(
        event = "passkey.auth.started",
        request_id = %request_id,
        app_instance_id = %app_instance_id,
        duration_ms = start.elapsed().as_millis(),
        status = "success"
    );

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(header::SET_COOKIE, cookie.parse().unwrap());
    headers.insert(
        axum::http::HeaderName::from_static("x-passkey-session"),
        session_id.to_string().parse().unwrap()
    );

    Ok((
        StatusCode::OK,
        headers,
        Json(rcr)
    ).into_response())
}

#[derive(serde::Deserialize)]
pub struct LoginFinishRequest {
    pub email: String,
    pub response: PublicKeyCredential,
}

pub async fn login_finish(
    Extension(state): Extension<WebauthnState>,
    Extension(db): Extension<DatabaseConnection>,
    jar: CookieJar,
    headers: axum::http::HeaderMap,
    Json(req): Json<LoginFinishRequest>,
) -> Result<Response, (StatusCode, String)> {
    let start = Instant::now();
    let request_id = uuid::Uuid::new_v4();

    let app_instance_id = headers
        .get("x-app-instance-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let origin = headers.get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .ok_or((StatusCode::BAD_REQUEST, "Missing or invalid Origin header".to_string()))?;

    let webauthn = state.registry.get_or_create(origin).await
        .map_err(|e: String| (StatusCode::BAD_REQUEST, e))?;

    let session_id = headers
        .get("x-passkey-session")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .or_else(|| {
            jar.get("passkey_session")
                .and_then(|c| Uuid::parse_str(c.value()).ok())
        })
        .ok_or((StatusCode::BAD_REQUEST, "Passkey session missing or expired".to_string()))?;

    let auth_state: PasskeyAuthentication = get_challenge(&db, session_id, "authentication").await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Auth state not found: {e}")))?;

    let _ = delete_challenge(&db, session_id).await;

    let auth_result = webauthn.finish_passkey_authentication(&req.response, &auth_state)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let cred_id = auth_result.cred_id();

    let pk_model = passkey::Entity::find()
        .filter(passkey::Column::CredentialId.eq(cred_id.to_vec()))
        .one(&db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::BAD_REQUEST, "Passkey not found in db".to_string()))?;

    let user = user::Entity::find_by_id(pk_model.user_id)
        .one(&db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::BAD_REQUEST, "User not found".to_string()))?;

    let session_response = crate::handlers::sessions::create_session_for_user(&db, &user)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create session".to_string()))?;

    let cookie = session_cookie_header(&session_response.token, 86_400);
    let clear_pk_session = "passkey_session=; Path=/api/passkeys; HttpOnly; Max-Age=0";
    
    metrics::PASSKEY_AUTH_SUCCESS
        .with_label_values(&["unknown", &app_instance_id])
        .inc();

    tracing::info!(
        event = "passkey.auth.success",
        request_id = %request_id,
        user_id = %user.id,
        app_instance_id = %app_instance_id,
        duration_ms = start.elapsed().as_millis(),
        status = "success"
    );

    Ok((
        StatusCode::OK,
        [
            (header::SET_COOKIE, cookie.clone()),
            (header::SET_COOKIE, clear_pk_session.to_string())
        ],
        Json(serde_json::json!(session_response)),
    ).into_response())
}

// ═══════════════════════════════════════════════════════════════════════════════
// DATABASE CHALLENGE PERSISTENCE (Enterprise Best Practices)
// ═══════════════════════════════════════════════════════════════════════════════

async fn save_challenge<T: serde::Serialize>(
    db: &DatabaseConnection,
    id: Uuid,
    challenge: &T,
    challenge_type: &str,
) -> Result<(), String> {
    use sea_orm::Set;

    let challenge_json = serde_json::to_value(challenge)
        .map_err(|e| format!("Failed to serialize challenge: {e}"))?;

    let existing = webauthn_challenge::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| format!("Database error on lookup: {e}"))?;

    if let Some(model) = existing {
        let mut active: webauthn_challenge::ActiveModel = model.into();
        active.challenge = Set(challenge_json);
        active.challenge_type = Set(challenge_type.to_string());
        active.expires_at = Set(chrono::Utc::now() + chrono::Duration::seconds(300));
        active.update(db)
            .await
            .map_err(|e| format!("Failed to update challenge: {e}"))?;
    } else {
        webauthn_challenge::ActiveModel {
            id: Set(id),
            challenge: Set(challenge_json),
            challenge_type: Set(challenge_type.to_string()),
            expires_at: Set(chrono::Utc::now() + chrono::Duration::seconds(300)),
            created_at: Set(chrono::Utc::now()),
        }
        .insert(db)
        .await
        .map_err(|e| format!("Failed to insert challenge: {e}"))?;
    }

    Ok(())
}

async fn get_challenge<T: serde::de::DeserializeOwned>(
    db: &DatabaseConnection,
    id: Uuid,
    expected_type: &str,
) -> Result<T, String> {
    let model = webauthn_challenge::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| format!("Database query failed: {e}"))?
        .ok_or_else(|| "Challenge not found".to_string())?;

    if model.challenge_type != expected_type {
        return Err("Challenge type mismatch".to_string());
    }

    if model.expires_at < chrono::Utc::now() {
        return Err("Challenge expired".to_string());
    }

    let challenge: T = serde_json::from_value(model.challenge)
        .map_err(|e| format!("Failed to deserialize challenge: {e}"))?;

    Ok(challenge)
}

async fn delete_challenge(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<(), String> {
    use sea_orm::EntityTrait;
    webauthn_challenge::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(|e| format!("Failed to delete challenge: {e}"))?;
    Ok(())
}

async fn prune_expired_challenges(db: &DatabaseConnection) {
    use sea_orm::EntityTrait;
    use sea_orm::QueryFilter;
    use sea_orm::ColumnTrait;

    let now = chrono::Utc::now();
    let _ = webauthn_challenge::Entity::delete_many()
        .filter(webauthn_challenge::Column::ExpiresAt.lt(now))
        .exec(db)
        .await;
}

