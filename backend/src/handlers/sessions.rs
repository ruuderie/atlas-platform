use axum::{
    extract::{Extension, Json, TypedHeader},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    headers::{Authorization, authorization::Bearer},
};
use axum::http::{Method, Uri};
use sea_orm::{DatabaseConnection, ColumnTrait, EntityTrait, Set, ActiveModelTrait, QueryFilter};
use uuid::Uuid;
use chrono::{Utc, Duration};
use crate::entities::{directory, session, user};
use crate::auth::{generate_jwt, hash_password, verify_password, generate_jwt_admin};
use crate::handlers::users::LoginResponse;
use crate::models::session::{UserInfo, SessionResponse};
use crate::models::user::UserLogin;
use crate::models::request_log::{RequestStatus, RequestType};
use crate::handlers::request_logs::log_request;
use axum::extract::State;
use serde::Deserialize;


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

    // Generate tokens
    let bearer_token = if user.is_admin {
        generate_jwt_admin(&user)
    } else {
        generate_jwt(&user)
    }.map_err(|e| {
        println!("TEST LOG: from create_user_session and token generation failed: {:?}", e);
        tracing::error!("Token generation failed: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let refresh_token = generate_jwt(&user).map_err(|e| {
        println!("TEST LOG: from create_user_session and refresh token generation failed: {:?}", e);
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
        is_admin: Set(user.is_admin),
        is_active: Set(true),
        integrity_hash: Set(String::new()),
    };

    // Generate integrity hash
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
            is_admin: user.is_admin,
            is_active: true,
            integrity_hash: String::new(),
        };
        temp_model.generate_integrity_hash()
    };

    new_session.integrity_hash = Set(integrity_hash);

    // Insert session
    new_session.insert(db).await.map_err(|e| {
        println!("TEST LOG: from create_user_session and session creation failed: {:?}", e);
        tracing::error!("Session creation failed: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(SessionResponse {
        user: Some(UserInfo {
            id: user.id,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            is_admin: user.is_admin,
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
    
    // Extract token from Authorization header
    let token = match headers.get("Authorization") {
        Some(auth_header) => {
            match auth_header.to_str() {
                Ok(auth_str) => {
                    if auth_str.starts_with("Bearer ") {
                        let token = auth_str[7..].to_string();
                        tracing::info!("Token extracted: {}", token.chars().take(10).collect::<String>() + "...");
                        token
                    } else {
                        tracing::warn!("Authorization header doesn't start with 'Bearer '");
                        return Err(StatusCode::UNAUTHORIZED);
                    }
                },
                Err(e) => {
                    tracing::error!("Failed to parse Authorization header: {:?}", e);
                    return Err(StatusCode::UNAUTHORIZED);
                }
            }
        },
        None => {
            tracing::warn!("No Authorization header found");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };
    
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
    updated_session.update(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SessionResponse { 
        user: Some(UserInfo {
            id: user.id,
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            is_admin: user.is_admin,
        }),
        token: session.bearer_token.clone(),
        refresh_token: session.refresh_token.clone(),
    }))
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

    // Generate new bearer token
    let new_bearer_token = if user.is_admin {
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

    // Update the session with the new tokens
    let new_token_expiration = Utc::now() + Duration::hours(1);
    let new_refresh_token_expiration = Utc::now() + Duration::days(7);
    let mut updated_session: session::ActiveModel = session.into();
    updated_session.bearer_token = Set(new_bearer_token.clone());
    updated_session.refresh_token = Set(new_refresh_token.clone());
    updated_session.token_expiration = Set(new_token_expiration);
    updated_session.refresh_token_expiration = Set(new_refresh_token_expiration);
    updated_session.last_accessed_at = Set(Utc::now());

    match updated_session.update(&db).await {
        Ok(_) => {
            tracing::info!("Session refreshed successfully for user: {}", user.id);
            Ok(Json(SessionResponse { 
                user: Some(UserInfo {
                    id: user.id,
                    email: user.email,
                    first_name: user.first_name,
                    last_name: user.last_name,
                    is_admin: user.is_admin,
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
