#![allow(dead_code)]
use crate::auth::{generate_jwt, generate_jwt_admin, hash_token, verify_password};
use crate::entities::{session, user};
use crate::models::session::{SessionResponse, UserInfo};
use crate::models::user::UserLogin;
use axum::extract::State;
use axum::{
    extract::{Extension, Json, Path},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::Deserialize;
use std::time::Instant;
use uuid::Uuid;

/// True when cookies may be set without the `Secure` flag (local HTTP only).
/// Production / UAT / DEV server envs must keep `Secure`.
pub fn cookie_secure_attribute_required() -> bool {
    match std::env::var("ENVIRONMENT")
        .unwrap_or_else(|_| "production".to_string())
        .to_lowercase()
        .as_str()
    {
        "development" | "dev" | "local" => false,
        _ => true,
    }
}

fn secure_cookie_fragment() -> &'static str {
    if cookie_secure_attribute_required() {
        "; Secure"
    } else {
        ""
    }
}

/// Builds the `Set-Cookie` header value for the session token.
pub fn session_cookie_header(token: &str, max_age_secs: i64) -> String {
    format!(
        "session={}; HttpOnly{}; SameSite=Strict; Path=/; Max-Age={}",
        token,
        secure_cookie_fragment(),
        max_age_secs
    )
}

/// Clears the session cookie (used on logout / revoke).
pub fn clear_session_cookie_header() -> String {
    format!(
        "session=; HttpOnly{}; SameSite=Strict; Path=/; Max-Age=0",
        secure_cookie_fragment()
    )
}

/// Extracts the session token from either cookie or Authorization header.
pub fn extract_session_token(headers: &HeaderMap) -> Option<String> {
    if let Some(cookie_header) = headers.get(header::COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for part in cookie_str.split(';') {
                let part = part.trim();
                if let Some(token) = part.strip_prefix("session=") {
                    if !token.is_empty() {
                        return Some(token.to_string());
                    }
                }
            }
        }
    }
    if let Some(auth_header) = headers.get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if !token.is_empty() {
                    return Some(token.to_string());
                }
            }
        }
    }
    None
}

pub async fn create_session(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<UserLogin>,
) -> Result<SessionResponse, StatusCode> {
    create_user_session(&db, &payload.email, &payload.password).await
}

pub async fn create_user_session(
    db: &DatabaseConnection,
    email: &str,
    password: &str,
) -> Result<SessionResponse, StatusCode> {
    let start = Instant::now();
    let request_id = uuid::Uuid::new_v4();

    let user = user::Entity::find()
        .filter(user::Column::Email.eq(email))
        .one(db)
        .await
        .map_err(|e| {
            tracing::error!("Database error in session creation: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !verify_password(password, &user.password_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        tracing::warn!(
            event = "session.creation.failed",
            request_id = %request_id,
            email = %email,
            reason = "invalid_password",
            duration_ms = start.elapsed().as_millis()
        );
        return Err(StatusCode::UNAUTHORIZED);
    }

    let result = create_session_for_user(db, &user).await;

    if result.is_ok() {
        tracing::info!(
            event = "session.created",
            request_id = %request_id,
            user_id = %user.id,
            duration_ms = start.elapsed().as_millis(),
            status = "success"
        );
    }

    result
}

pub async fn create_passwordless_session(
    db: &DatabaseConnection,
    email: &str,
) -> Result<SessionResponse, StatusCode> {
    let start = Instant::now();
    let request_id = uuid::Uuid::new_v4();

    let user = user::Entity::find()
        .filter(user::Column::Email.eq(email))
        .one(db)
        .await
        .map_err(|e| {
            tracing::error!("Database error in passwordless session creation: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let result = create_session_for_user(db, &user).await;

    if result.is_ok() {
        tracing::info!(
            event = "session.created.passwordless",
            request_id = %request_id,
            user_id = %user.id,
            duration_ms = start.elapsed().as_millis(),
            status = "success"
        );
    }

    result
}

pub async fn create_session_for_user(
    db: &DatabaseConnection,
    user: &user::Model,
) -> Result<SessionResponse, StatusCode> {
    let start = Instant::now();
    let request_id = uuid::Uuid::new_v4();

    let is_platform_admin = crate::entities::user_account::Entity::find()
        .filter(crate::entities::user_account::Column::UserId.eq(user.id))
        .filter(
            crate::entities::user_account::Column::Role
                .eq(crate::entities::user_account::UserRole::PlatformSuperAdmin),
        )
        .one(db)
        .await
        .unwrap_or(None)
        .is_some();

    let bearer_token = if is_platform_admin {
        generate_jwt_admin(user)
    } else {
        generate_jwt(user)
    }
    .map_err(|e| {
        tracing::error!("Token generation failed: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let refresh_token = generate_jwt(user).map_err(|e| {
        tracing::error!("Refresh token generation failed: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Local HTTP stacks keep a longer access token so Folio SSR proxies don't
    // 401 every hour while the UI still looks signed-in from a cached /me.
    let token_expiration = if cookie_secure_attribute_required() {
        Utc::now() + Duration::hours(1)
    } else {
        Utc::now() + Duration::hours(24)
    };
    let refresh_token_expiration = Utc::now() + Duration::days(7);

    let mut new_session = session::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user.id),
        bearer_token: Set(bearer_token.clone()),
        refresh_token: Set(refresh_token.clone()),
        // Store SHA-256 hashes for secure DB lookup (Security #7).
        // Dual-write: plaintext columns kept for backward compat until a follow-up
        // migration drops them after full rollout of hash-based lookup.
        bearer_token_hash: Set(Some(hash_token(&bearer_token))),
        refresh_token_hash: Set(Some(hash_token(&refresh_token))),
        token_expiration: Set(token_expiration),
        refresh_token_expiration: Set(refresh_token_expiration),
        created_at: Set(Utc::now()),
        last_accessed_at: Set(Utc::now()),
        last_modified_date: Set(Utc::now()),
        is_admin: Set(is_platform_admin),
        is_active: Set(true),
        integrity_hash: Set(String::new()),
    };

    let integrity_hash = {
        let temp_model = session::Model {
            id: new_session.id.clone().unwrap(),
            user_id: user.id,
            bearer_token: bearer_token.clone(),
            refresh_token: refresh_token.clone(),
            bearer_token_hash: Some(hash_token(&bearer_token)),
            refresh_token_hash: Some(hash_token(&refresh_token)),
            token_expiration,
            refresh_token_expiration,
            created_at: Utc::now(),
            last_accessed_at: Utc::now(),
            last_modified_date: Utc::now(),
            is_admin: is_platform_admin,
            is_active: true,
            integrity_hash: String::new(),
        };
        temp_model.generate_integrity_hash()
    };

    new_session.integrity_hash = Set(integrity_hash);

    new_session.insert(db).await.map_err(|e| {
        tracing::error!("Session creation failed: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let app_permissions: Vec<crate::models::session::AppPermission> =
        crate::entities::user_app_permission::Entity::find()
            .filter(crate::entities::user_app_permission::Column::UserId.eq(user.id))
            .all(db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|p| crate::models::session::AppPermission {
                tenant_id: p.tenant_id,
                app_slug: p.app_slug,
                permissions: p.permissions,
            })
            .collect();

    tracing::info!(
        event = "session.created.full",
        request_id = %request_id,
        user_id = %user.id,
        is_platform_admin = is_platform_admin,
        app_permissions_count = app_permissions.len(),
        duration_ms = start.elapsed().as_millis(),
        status = "success"
    );

    Ok(SessionResponse {
        user: Some(UserInfo {
            id: user.id,
            email: user.email.clone(),
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            is_admin: is_platform_admin,
            app_permissions,
        }),
        token: bearer_token,
        refresh_token,
    })
}

pub async fn validate_session(
    Extension(db): Extension<DatabaseConnection>,
    headers: HeaderMap,
) -> Result<Json<SessionResponse>, StatusCode> {
    let start = Instant::now();
    let request_id = uuid::Uuid::new_v4();

    let token = extract_session_token(&headers).ok_or_else(|| {
        tracing::warn!("No session cookie or Authorization header found");
        StatusCode::UNAUTHORIZED
    })?;

    // Look up session by bearer_token_hash (secure) with fallback to plaintext
    // for sessions created before migration m20260515_000001.
    let token_hash = hash_token(&token);
    let session = match session::Entity::find()
        .filter(session::Column::BearerTokenHash.eq(&token_hash))
        .one(&db)
        .await
    {
        Ok(Some(session)) => session,
        // Fallback: pre-migration sessions have NULL hash — look up by plaintext.
        Ok(None) => match session::Entity::find()
            .filter(session::Column::BearerToken.eq(token.clone()))
            .filter(session::Column::BearerTokenHash.is_null())
            .one(&db)
            .await
        {
            Ok(Some(session)) => session,
            Ok(None) => {
                tracing::warn!("No session found for token");
                return Err(StatusCode::UNAUTHORIZED);
            }
            Err(e) => {
                tracing::error!("Database error when fetching session (fallback): {:?}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        },
        Err(e) => {
            tracing::error!("Database error when fetching session: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if !session.is_active || !session.verify_integrity() || session.token_expiration < Utc::now() {
        tracing::warn!("Session invalid or expired");
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut updated_session: session::ActiveModel = session.clone().into();
    updated_session.last_accessed_at = Set(Utc::now());
    updated_session
        .update(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = match user::Entity::find_by_id(session.user_id).one(&db).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            tracing::warn!("User not found for session during validation");
            return Err(StatusCode::UNAUTHORIZED);
        }
        Err(e) => {
            tracing::error!(
                "Database error when finding user in session validation: {:?}",
                e
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let app_permissions: Vec<crate::models::session::AppPermission> =
        crate::entities::user_app_permission::Entity::find()
            .filter(crate::entities::user_app_permission::Column::UserId.eq(user.id))
            .all(&db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|p| crate::models::session::AppPermission {
                tenant_id: p.tenant_id,
                app_slug: p.app_slug,
                permissions: p.permissions,
            })
            .collect();

    tracing::info!(
        event = "session.validated",
        request_id = %request_id,
        user_id = %session.user_id,
        duration_ms = start.elapsed().as_millis(),
        status = "success"
    );

    Ok(Json(SessionResponse {
        user: Some(UserInfo {
            id: user.id,
            email: user.email.clone(),
            first_name: user.first_name.clone(),
            last_name: user.last_name.clone(),
            is_admin: session.is_admin,
            app_permissions,
        }),
        token: session.bearer_token.clone(),
        refresh_token: session.refresh_token.clone(),
    }))
}

pub async fn revoke_session(
    Extension(db): Extension<DatabaseConnection>,
    headers: HeaderMap,
) -> Response {
    let start = Instant::now();
    let request_id = uuid::Uuid::new_v4();

    let Some(token) = extract_session_token(&headers) else {
        return StatusCode::NO_CONTENT.into_response();
    };

    if let Ok(Some(sess)) = session::Entity::find()
        .filter(session::Column::BearerToken.eq(&token))
        .one(&db)
        .await
    {
        let revoked_model = session::Model {
            is_active: false,
            integrity_hash: String::new(),
            ..sess.clone()
        };
        let new_hash = revoked_model.generate_integrity_hash();

        let mut active: session::ActiveModel = sess.clone().into();
        active.is_active = Set(false);
        active.integrity_hash = Set(new_hash);
        let _ = active.update(&db).await;

        tracing::info!(
            event = "session.revoked",
            request_id = %request_id,
            user_id = %sess.user_id,
            duration_ms = start.elapsed().as_millis(),
            status = "success"
        );
    }

    (
        StatusCode::NO_CONTENT,
        [(header::SET_COOKIE, clear_session_cookie_header())],
    )
        .into_response()
}

pub async fn delete_session(
    Extension(db): Extension<DatabaseConnection>,
    Extension(session): Extension<session::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    session::Entity::delete_by_id(session.id)
        .exec(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

pub async fn cleanup_expired_sessions(db: &DatabaseConnection) {
    let result = session::Entity::delete_many()
        .filter(session::Column::RefreshTokenExpiration.lt(Utc::now()))
        .exec(db)
        .await;

    match result {
        Ok(del) => tracing::info!("Cleaned up {} expired sessions", del.rows_affected),
        Err(e) => tracing::error!("Error cleaning up expired sessions: {:?}", e),
    }
}

pub async fn refresh_token(
    Extension(db): Extension<DatabaseConnection>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<SessionResponse>, StatusCode> {
    let start = Instant::now();
    let request_id = uuid::Uuid::new_v4();

    let refresh_token = payload.refresh_token;

    // Look up by refresh_token_hash with plaintext fallback for pre-migration sessions.
    let refresh_hash = hash_token(&refresh_token);
    let session = match session::Entity::find()
        .filter(session::Column::RefreshTokenHash.eq(&refresh_hash))
        .one(&db)
        .await
    {
        Ok(Some(session)) => session,
        Ok(None) => match session::Entity::find()
            .filter(session::Column::RefreshToken.eq(&refresh_token))
            .filter(session::Column::RefreshTokenHash.is_null())
            .one(&db)
            .await
        {
            Ok(Some(session)) => session,
            Ok(None) => {
                tracing::warn!("No session found for refresh token");
                return Err(StatusCode::UNAUTHORIZED);
            }
            Err(e) => {
                tracing::error!("Database error when fetching session (fallback): {:?}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        },
        Err(e) => {
            tracing::error!("Database error when fetching session: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if session.refresh_token_expiration < Utc::now() {
        tracing::warn!("Refresh token has expired");
        return Err(StatusCode::UNAUTHORIZED);
    }

    let user = match user::Entity::find_by_id(session.user_id).one(&db).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            tracing::error!("User not found for session");
            return Err(StatusCode::UNAUTHORIZED);
        }
        Err(e) => {
            tracing::error!("Database error when finding user: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let is_platform_admin = crate::entities::user_account::Entity::find()
        .filter(crate::entities::user_account::Column::UserId.eq(user.id))
        .filter(
            crate::entities::user_account::Column::Role
                .eq(crate::entities::user_account::UserRole::PlatformSuperAdmin),
        )
        .one(&db)
        .await
        .unwrap_or(None)
        .is_some();

    let new_bearer_token = if is_platform_admin {
        generate_jwt_admin(&user)
    } else {
        generate_jwt(&user)
    }
    .map_err(|e| {
        tracing::error!("Error generating new bearer token: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let new_refresh_token = generate_jwt(&user).map_err(|e| {
        tracing::error!("Error generating new refresh token: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let new_token_expiration = Utc::now() + Duration::hours(1);
    let new_refresh_token_expiration = Utc::now() + Duration::days(7);
    let new_bearer_hash = hash_token(&new_bearer_token);
    let new_refresh_hash = hash_token(&new_refresh_token);
    let mut updated_session: session::ActiveModel = session.clone().into();
    updated_session.bearer_token = Set(new_bearer_token.clone());
    updated_session.refresh_token = Set(new_refresh_token.clone());
    updated_session.bearer_token_hash = Set(Some(new_bearer_hash.clone()));
    updated_session.refresh_token_hash = Set(Some(new_refresh_hash.clone()));
    updated_session.token_expiration = Set(new_token_expiration);
    updated_session.refresh_token_expiration = Set(new_refresh_token_expiration);
    updated_session.last_accessed_at = Set(Utc::now());
    updated_session.last_modified_date = Set(Utc::now());
    updated_session.is_admin = Set(is_platform_admin);

    let refreshed_hash = {
        let temp = session::Model {
            id: session.id,
            user_id: user.id,
            bearer_token: new_bearer_token.clone(),
            refresh_token: new_refresh_token.clone(),
            bearer_token_hash: Some(new_bearer_hash.clone()),
            refresh_token_hash: Some(new_refresh_hash.clone()),
            token_expiration: new_token_expiration,
            refresh_token_expiration: new_refresh_token_expiration,
            created_at: session.created_at,
            last_accessed_at: Utc::now(),
            last_modified_date: Utc::now(),
            is_admin: is_platform_admin,
            is_active: session.is_active,
            integrity_hash: String::new(),
        };
        temp.generate_integrity_hash()
    };
    updated_session.integrity_hash = Set(refreshed_hash);

    match updated_session.update(&db).await {
        Ok(_) => {
            tracing::info!(
                event = "session.refreshed",
                request_id = %request_id,
                user_id = %user.id,
                duration_ms = start.elapsed().as_millis(),
                status = "success"
            );

            let app_permissions: Vec<crate::models::session::AppPermission> =
                crate::entities::user_app_permission::Entity::find()
                    .filter(crate::entities::user_app_permission::Column::UserId.eq(user.id))
                    .all(&db)
                    .await
                    .unwrap_or_default()
                    .into_iter()
                    .map(|p| crate::models::session::AppPermission {
                        tenant_id: p.tenant_id,
                        app_slug: p.app_slug,
                        permissions: p.permissions,
                    })
                    .collect();

            Ok(Json(SessionResponse {
                user: Some(UserInfo {
                    id: user.id,
                    email: user.email,
                    first_name: user.first_name,
                    last_name: user.last_name,
                    is_admin: is_platform_admin,
                    app_permissions,
                }),
                token: new_bearer_token,
                refresh_token: new_refresh_token,
            }))
        }
        Err(e) => {
            tracing::error!("Error updating session: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct ImpersonateExchangeRequest {
    pub code: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ExchangeResponse {
    pub success: bool,
}

pub async fn exchange_impersonate_code(
    State(_db): State<DatabaseConnection>,
    Json(payload): Json<ImpersonateExchangeRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Consume code (single-use, validated)
    let jwt = crate::handlers::admin::consume_impersonation_code(&payload.code)
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Set secure domain-isolated cookie (Max-Age 2 hours)
    let cookie_val = session_cookie_header(&jwt, 7200);

    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        cookie_val.parse().map_err(|e| {
            tracing::error!("Failed to parse cookie header: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?,
    );

    Ok((headers, Json(ExchangeResponse { success: true })))
}

// ── Session list & targeted revoke ───────────────────────────────────────────

#[derive(Debug, serde::Serialize)]
pub struct SessionSummary {
    pub id: Uuid,
    pub created_at: String,
    pub last_accessed_at: String,
    pub is_active: bool,
    /// true when this row is the caller's own current session
    pub is_current: bool,
}

/// `GET /api/me/sessions` — list all active sessions for the authenticated user.
pub async fn list_user_sessions(
    Extension(db): Extension<DatabaseConnection>,
    headers: HeaderMap,
) -> Response {
    let Some(token) = extract_session_token(&headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    // Resolve caller's session to get user_id and current session id
    let token_hash = hash_token(&token);
    let caller_session = match session::Entity::find()
        .filter(session::Column::BearerTokenHash.eq(&token_hash))
        .one(&db)
        .await
    {
        Ok(Some(s)) => s,
        _ => {
            // fallback to plaintext
            match session::Entity::find()
                .filter(session::Column::BearerToken.eq(&token))
                .one(&db)
                .await
            {
                Ok(Some(s)) => s,
                _ => return StatusCode::UNAUTHORIZED.into_response(),
            }
        }
    };

    let user_id = caller_session.user_id;
    let current_id = caller_session.id;

    let sessions = match session::Entity::find()
        .filter(session::Column::UserId.eq(user_id))
        .filter(session::Column::IsActive.eq(true))
        .all(&db)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("list_user_sessions: {e:#}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let summaries: Vec<SessionSummary> = sessions
        .into_iter()
        .map(|s| SessionSummary {
            id: s.id,
            created_at: s.created_at.to_rfc3339(),
            last_accessed_at: s.last_accessed_at.to_rfc3339(),
            is_active: s.is_active,
            is_current: s.id == current_id,
        })
        .collect();

    (StatusCode::OK, Json(summaries)).into_response()
}

/// `DELETE /api/me/sessions/{session_id}` — revoke a specific session (not the caller's own).
pub async fn revoke_other_session(
    Extension(db): Extension<DatabaseConnection>,
    headers: HeaderMap,
    Path(session_id): Path<Uuid>,
) -> Response {
    let Some(token) = extract_session_token(&headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    // Resolve caller's current session
    let token_hash = hash_token(&token);
    let caller_session = match session::Entity::find()
        .filter(session::Column::BearerTokenHash.eq(&token_hash))
        .one(&db)
        .await
    {
        Ok(Some(s)) => s,
        _ => {
            match session::Entity::find()
                .filter(session::Column::BearerToken.eq(&token))
                .one(&db)
                .await
            {
                Ok(Some(s)) => s,
                _ => return StatusCode::UNAUTHORIZED.into_response(),
            }
        }
    };

    // Must be a session belonging to the same user
    let target = match session::Entity::find_by_id(session_id).one(&db).await {
        Ok(Some(s)) => s,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!("revoke_other_session lookup: {e:#}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    if target.user_id != caller_session.user_id {
        return StatusCode::FORBIDDEN.into_response();
    }

    let revoked_model = session::Model {
        is_active: false,
        integrity_hash: String::new(),
        ..target.clone()
    };
    let new_hash = revoked_model.generate_integrity_hash();

    let mut active: session::ActiveModel = target.into();
    active.is_active = Set(false);
    active.integrity_hash = Set(new_hash);

    match active.update(&db).await {
        Ok(_) => {
            tracing::info!(
                event = "session.revoked_other",
                session_id = %session_id,
                by_user = %caller_session.user_id,
            );
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            tracing::error!("revoke_other_session update: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// `DELETE /api/me/sessions` — revoke ALL sessions except the caller's current one.
pub async fn revoke_all_other_sessions(
    Extension(db): Extension<DatabaseConnection>,
    headers: HeaderMap,
) -> Response {
    let Some(token) = extract_session_token(&headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let token_hash = hash_token(&token);
    let caller_session = match session::Entity::find()
        .filter(session::Column::BearerTokenHash.eq(&token_hash))
        .one(&db)
        .await
    {
        Ok(Some(s)) => s,
        _ => {
            match session::Entity::find()
                .filter(session::Column::BearerToken.eq(&token))
                .one(&db)
                .await
            {
                Ok(Some(s)) => s,
                _ => return StatusCode::UNAUTHORIZED.into_response(),
            }
        }
    };

    let user_id = caller_session.user_id;
    let current_id = caller_session.id;

    let others = match session::Entity::find()
        .filter(session::Column::UserId.eq(user_id))
        .filter(session::Column::IsActive.eq(true))
        .filter(session::Column::Id.ne(current_id))
        .all(&db)
        .await
    {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("revoke_all_other_sessions: {e:#}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let revoked = others.len();

    for sess in others {
        let revoked_model = session::Model {
            is_active: false,
            integrity_hash: String::new(),
            ..sess.clone()
        };
        let new_hash = revoked_model.generate_integrity_hash();
        let mut active: session::ActiveModel = sess.into();
        active.is_active = Set(false);
        active.integrity_hash = Set(new_hash);
        let _ = active.update(&db).await;
    }

    tracing::info!(
        event = "session.revoked_all_others",
        user_id = %user_id,
        count = revoked,
    );

    (
        StatusCode::OK,
        Json(serde_json::json!({ "revoked": revoked })),
    )
        .into_response()
}
