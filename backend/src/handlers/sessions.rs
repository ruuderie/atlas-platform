use axum::{
    extract::{Extension, Json},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use sea_orm::{DatabaseConnection, ColumnTrait, EntityTrait, Set, ActiveModelTrait, QueryFilter};
use uuid::Uuid;
use chrono::{Utc, Duration};
use crate::entities::{session, user};
use crate::auth::{generate_jwt, verify_password, generate_jwt_admin};
use crate::models::session::{UserInfo, SessionResponse};
use crate::models::user::UserLogin;
use axum::extract::State;
use serde::Deserialize;

/// Builds the `Set-Cookie` header value for the session token.
/// HttpOnly; Secure; SameSite=Strict — never readable by JavaScript.
/// Call this on every auth success endpoint (passkey login, magic link verify).
pub fn session_cookie_header(token: &str, max_age_secs: i64) -> String {
    format!(
        "session={}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age={}",
        token, max_age_secs
    )
}

/// Clears the session cookie (used on logout / revoke).
pub fn clear_session_cookie_header() -> String {
    "session=; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=0".to_string()
}

/// Extracts the session token from either:
///   1. `Authorization: Bearer <token>` header (platform-admin legacy, transitional)
///   2. `session` HttpOnly cookie (all apps post-migration)
/// Cookie takes priority when both are present.
pub fn extract_session_token(headers: &HeaderMap) -> Option<String> {
    // 1. Cookie (preferred — JS-inaccessible)
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
    // 2. Bearer header (transitional fallback)
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
    password: &str
) -> Result<SessionResponse, StatusCode> {
    let user = user::Entity::find()
        .filter(user::Column::Email.eq(email))
        .one(db)
        .await
        .map_err(|e| {
            tracing::error!("Database error in session creation: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !verify_password(password, &user.password_hash).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        return Err(StatusCode::UNAUTHORIZED);
    }

    create_session_for_user(db, &user).await
}

pub async fn create_passwordless_session(
    db: &DatabaseConnection,
    email: &str,
) -> Result<SessionResponse, StatusCode> {
    let user = user::Entity::find()
        .filter(user::Column::Email.eq(email))
        .one(db)
        .await
        .map_err(|e| {
            tracing::error!("Database error in passwordless session creation: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    create_session_for_user(db, &user).await
}

pub async fn create_session_for_user(
    db: &DatabaseConnection,
    user: &user::Model,
) -> Result<SessionResponse, StatusCode> {
    // Determine admin status via user_account role (is_admin was removed from the user entity).
    // A 'PlatformSuperAdmin' role in user_account grants elevated JWT claims.
    let is_platform_admin = crate::entities::user_account::Entity::find()
        .filter(crate::entities::user_account::Column::UserId.eq(user.id))
        .filter(crate::entities::user_account::Column::Role.eq(
            crate::entities::user_account::UserRole::PlatformSuperAdmin
        ))
        .one(db)
        .await
        .unwrap_or(None)
        .is_some();

    // Generate tokens
    let bearer_token = if is_platform_admin {
        generate_jwt_admin(user)
    } else {
        generate_jwt(user)
    }.map_err(|e| {
        tracing::error!("Token generation failed: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let refresh_token = generate_jwt(user).map_err(|e| {
        tracing::error!("Refresh token generation failed: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Create session model
    let token_expiration = Utc::now() + Duration::hours(1);
    let refresh_token_expiration = Utc::now() + Duration::days(7);
    
    let mut new_session = session::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user.id),
        bearer_token: Set(bearer_token.clone()),
        refresh_token: Set(refresh_token.clone()),
        token_expiration: Set(token_expiration),
        refresh_token_expiration: Set(refresh_token_expiration),
        created_at: Set(Utc::now()),
        last_accessed_at: Set(Utc::now()),
        last_modified_date: Set(Utc::now()),
        is_admin: Set(is_platform_admin),
        is_active: Set(true),
        integrity_hash: Set(String::new()),
    };

    // Generate integrity hash — must match the fields covered by generate_integrity_hash().
    // is_active = true because this is a freshly created, active session.
    let integrity_hash = {
        let temp_model = session::Model {
            id: new_session.id.clone().unwrap(),
            user_id: user.id,
            bearer_token: bearer_token.clone(),
            refresh_token: refresh_token.clone(),
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

    // Insert session
    new_session.insert(db).await.map_err(|e| {
        tracing::error!("Session creation failed: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Issue 4 fix: fetch app-level permissions so the login response is immediately
    // usable by client-side feature gates (e.g. ListingPoster). The middleware
    // re-injects these per-request, but on first login the client only has the
    // JSON response to go on.
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
    tracing::info!("Validating session");

    let token = extract_session_token(&headers).ok_or_else(|| {
        tracing::warn!("No session cookie or Authorization header found");
        StatusCode::UNAUTHORIZED
    })?;
    
    // Query for session with this token
    tracing::info!("Querying database for session with token");
    let session = match session::Entity::find()
        .filter(session::Column::BearerToken.eq(token.clone()))
        .one(&db)
        .await
    {
        Ok(Some(session)) => {
            tracing::info!("Session found: {}", session.id);
            session
        },
        Ok(None) => {
            tracing::warn!("No session found for token");
            return Err(StatusCode::UNAUTHORIZED);
        },
        Err(e) => {
            tracing::error!("Database error when fetching session: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Check session validity
    if !session.is_active {
        tracing::warn!("Session is inactive");
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    if !session.verify_integrity() {
        tracing::warn!("Session failed integrity check");
        return Err(StatusCode::UNAUTHORIZED);
    }

    if session.token_expiration < Utc::now() {
        tracing::warn!("Session has expired at {}, current time is {}", 
                      session.token_expiration, Utc::now());
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Get user information
    let user = match user::Entity::find_by_id(session.user_id)
        .one(&db)
        .await {
            Ok(Some(user)) => user,
            Ok(None) => {
                tracing::error!("User not found for session");
                return Err(StatusCode::UNAUTHORIZED);
            },
            Err(e) => {
                tracing::error!("Database error when finding user: {:?}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

    // Update last_accessed_at
    let mut updated_session: session::ActiveModel = session.clone().into();
    updated_session.last_accessed_at = Set(Utc::now());
    // Note: integrity_hash is intentionally NOT recomputed here.
    // generate_integrity_hash() covers: id, user_id, bearer_token, token_expiration, is_admin.
    // last_accessed_at is excluded from the hash by design so that this frequent
    // housekeeping write does not trigger a hash update on every request.

    updated_session.update(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Note: app_permissions are NOT re-fetched here. The auth middleware already injects
    // the user's permissions into Axum request extensions on every authenticated API request,
    // which is the canonical source for RBAC checks in downstream handlers.
    // Returning an empty vec keeps the response shape stable for consumers that read `user`.
    Ok(Json(SessionResponse {
        user: Some(UserInfo {
            id: user.id,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            is_admin: session.is_admin,
            app_permissions: Vec::new(),
        }),
        token: session.bearer_token.clone(),
        refresh_token: session.refresh_token.clone(),
    }))
}

/// Revokes the current session and clears the HttpOnly cookie.
/// Called by all apps on logout.
pub async fn revoke_session(
    Extension(db): Extension<DatabaseConnection>,
    headers: HeaderMap,
) -> Response {
    let Some(token) = extract_session_token(&headers) else {
        return StatusCode::NO_CONTENT.into_response();
    };

    // Deactivate session in DB and regenerate integrity hash to reflect the revocation.
    // is_active is now part of the hash, so setting it to false without updating the hash
    // would cause any subsequent verify_integrity() call to fail — but more importantly,
    // recomputing here means an attacker who flips is_active back to true in the DB
    // will also need to forge the hash, which they cannot do without the secret.
    if let Ok(Some(sess)) = session::Entity::find()
        .filter(session::Column::BearerToken.eq(&token))
        .one(&db)
        .await
    {
        // Build the revoked model to compute the new hash.
        let revoked_model = session::Model {
            is_active: false,
            integrity_hash: String::new(), // placeholder for hash computation
            ..sess.clone()
        };
        let new_hash = revoked_model.generate_integrity_hash();

        let mut active: session::ActiveModel = sess.into();
        active.is_active = Set(false);
        active.integrity_hash = Set(new_hash);
        let _ = active.update(&db).await;
    }

    // Clear the cookie regardless
    (StatusCode::NO_CONTENT,
     [(header::SET_COOKIE, clear_session_cookie_header())])
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
    let refresh_token = payload.refresh_token;
    tracing::info!("Refreshing token with refresh_token: {}", 
                  refresh_token.chars().take(10).collect::<String>() + "...");
    
    // Find the session with the given refresh token
    let session = match session::Entity::find()
        .filter(session::Column::RefreshToken.eq(refresh_token))
        .one(&db)
        .await {
            Ok(Some(session)) => session,
            Ok(None) => {
                tracing::warn!("No session found for refresh token");
                return Err(StatusCode::UNAUTHORIZED);
            },
            Err(e) => {
                tracing::error!("Database error when fetching session: {:?}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

    // Check if the refresh token is still valid
    if session.refresh_token_expiration < Utc::now() {
        tracing::warn!("Refresh token has expired");
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Find the user associated with this session
    let user = match user::Entity::find_by_id(session.user_id)
        .one(&db)
        .await {
            Ok(Some(user)) => user,
            Ok(None) => {
                tracing::error!("User not found for session");
                return Err(StatusCode::UNAUTHORIZED);
            },
            Err(e) => {
                tracing::error!("Database error when finding user: {:?}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

    // Generate new bearer token — re-check admin status since roles may have changed.
    let is_platform_admin = crate::entities::user_account::Entity::find()
        .filter(crate::entities::user_account::Column::UserId.eq(user.id))
        .filter(crate::entities::user_account::Column::Role.eq(
            crate::entities::user_account::UserRole::PlatformSuperAdmin
        ))
        .one(&db)
        .await
        .unwrap_or(None)
        .is_some();

    let new_bearer_token = if is_platform_admin {
        generate_jwt_admin(&user)
    } else {
        generate_jwt(&user)
    }.map_err(|e| {
        tracing::error!("Error generating new bearer token: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Generate new refresh token
    let new_refresh_token = generate_jwt(&user).map_err(|e| {
        tracing::error!("Error generating new refresh token: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Update the session with the new tokens and refreshed admin status.
    let new_token_expiration = Utc::now() + Duration::hours(1);
    let new_refresh_token_expiration = Utc::now() + Duration::days(7);
    let mut updated_session: session::ActiveModel = session.clone().into();
    updated_session.bearer_token = Set(new_bearer_token.clone());
    updated_session.refresh_token = Set(new_refresh_token.clone());
    updated_session.token_expiration = Set(new_token_expiration);
    updated_session.refresh_token_expiration = Set(new_refresh_token_expiration);
    updated_session.last_accessed_at = Set(Utc::now());
    updated_session.last_modified_date = Set(Utc::now());
    // Issue 5 fix: persist the re-evaluated admin status so the session row
    // stays consistent with the current user_account.role. Without this write,
    // a revoked admin who refreshes will keep is_admin=true in the DB, causing
    // the integrity hash to diverge on the next verify_integrity() call.
    updated_session.is_admin = Set(is_platform_admin);

    // Recompute integrity hash over ALL the new field values before writing.
    let refreshed_hash = {
        let temp = session::Model {
            id: session.id,
            user_id: user.id,
            bearer_token: new_bearer_token.clone(),
            refresh_token: new_refresh_token.clone(),
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
            tracing::info!("Session refreshed successfully for user: {}", user.id);
            Ok(Json(SessionResponse { 
                user: Some(UserInfo {
                    id: user.id,
                    email: user.email,
                    first_name: user.first_name,
                    last_name: user.last_name,
                    is_admin: is_platform_admin,
                    app_permissions: Vec::new(),
                }), token: new_bearer_token, refresh_token: new_refresh_token }))
        },
        Err(e) => {
            tracing::error!("Error updating session: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Add this struct for the refresh token request
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}
