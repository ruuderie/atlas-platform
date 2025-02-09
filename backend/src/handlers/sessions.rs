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



pub async fn create_session(
    Extension(db): Extension<DatabaseConnection>,
    Json(credentials): Json<UserLogin>,
) -> Result<SessionResponse, StatusCode> {
    tracing::info!("Creating session for user: {}", credentials.email);

    let user_result = user::Entity::find()
        .filter(user::Column::Email.eq(credentials.email.clone()))
        .one(&db)
        .await;

    let (request_status, failure_reason, user_info) = match user_result {
        Ok(Some(user)) => {
            if verify_password(&credentials.password, &user.password_hash).map_err(|e| {
                tracing::error!("Error verifying password: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })? {
                // Password is correct, create session
                (RequestStatus::Success, None, Some(user))
            } else {
                (RequestStatus::Failure, Some("Invalid password".to_string()), None)
            }
        },
        Ok(None) => (RequestStatus::Failure, Some("User not found".to_string()), None),
        Err(e) => {
            tracing::error!("Database error when finding user: {:?}", e);
            (RequestStatus::Failure, Some("Internal server error".to_string()), None)
        }
    };

    // Log the login attempt
    if let Err(e) = log_request(
        axum::http::Method::POST,
        axum::http::Uri::from_static("/login"),
        if request_status == RequestStatus::Success { StatusCode::OK } else { StatusCode::UNAUTHORIZED },
        user_info.as_ref().map(|u| u.id),
        "User Agent", // You might want to pass this from the request
        "IP Address", // You might want to pass this from the request
        RequestType::Login,
        request_status.clone(),
        failure_reason.clone(),
        &db
    ).await {
        tracing::error!("Failed to log login attempt: {:?}", e);
    }

    if request_status == RequestStatus::Success {
        let user = user_info.unwrap();  // Safe to unwrap here

        // Generate bearer token
        let bearer_token = if user.is_admin {
            generate_jwt_admin(&user)
        } else {
            generate_jwt(&user)
        }.map_err(|e| {
            tracing::error!("Error generating bearer token: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // Generate refresh token
        let refresh_token = generate_jwt(&user).map_err(|e| {
            tracing::error!("Error generating refresh token: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let token_expiration = Utc::now() + Duration::hours(1);
        let refresh_token_expiration = Utc::now() + Duration::days(7);
        let refresh_token_clone = &refresh_token.as_str().to_string();
        // Create new session
        let new_session = session::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user.id),
            bearer_token: Set(bearer_token.clone()),
            refresh_token: Set(refresh_token),
            token_expiration: Set(token_expiration),
            refresh_token_expiration: Set(refresh_token_expiration),
            created_at: Set(Utc::now()),
            last_accessed_at: Set(Utc::now()),
            last_modified_date: Set(Utc::now()),
            is_admin: Set(user.is_admin),
            is_active: Set(true),
            integrity_hash: Set(String::new()), // Temporary placeholder
        };

        // Convert ActiveModel to Model to generate integrity hash
        let model: session::Model = session::Model {
            id: new_session.id.clone().unwrap(),
            user_id: new_session.user_id.clone().unwrap(),
            bearer_token: new_session.bearer_token.clone().unwrap(),
            refresh_token: new_session.refresh_token.clone().unwrap(),
            token_expiration: new_session.token_expiration.clone().unwrap(),
            refresh_token_expiration: new_session.refresh_token_expiration.clone().unwrap(),
            created_at: new_session.created_at.clone().unwrap(),
            last_accessed_at: new_session.last_accessed_at.clone().unwrap(),
            last_modified_date: new_session.last_modified_date.clone().unwrap(),
            is_admin: new_session.is_admin.clone().unwrap(),
            is_active: new_session.is_active.clone().unwrap(),
            integrity_hash: String::new(), 
        };
        let integrity_hash = model.generate_integrity_hash();

        // Update the integrity_hash in the ActiveModel
        let mut new_session = new_session;
        new_session.integrity_hash = Set(integrity_hash);

        match new_session.insert(&db).await {
            Ok(_) => {
                tracing::info!("Session created successfully for user: {}", user.id);
                Ok(SessionResponse { 
                    user: Some(UserInfo {
                        id: user.id,
                        email: user.email,
                        first_name: user.first_name,
                        last_name: user.last_name,
                        is_admin: user.is_admin,
                    }),
                    token: bearer_token, 
                    refresh_token: refresh_token_clone.to_string(),
                })
            },
            Err(e) => {
                tracing::error!("Error creating session: {:?}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn validate_session(
    Extension(db): Extension<DatabaseConnection>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<SessionResponse>, StatusCode> {
    let token = bearer.token().to_string();
    
    let session = match session::Entity::find()
        .filter(session::Column::BearerToken.eq(token.clone()))
        .one(&db)
        .await
    {
        Ok(Some(session)) => session,
        Ok(None) => {
            tracing::warn!("No session found for token");
            return Err(StatusCode::UNAUTHORIZED);
        },
        Err(e) => {
            tracing::error!("Database error when fetching session: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    if !session.is_active || !session.verify_integrity() {
        tracing::warn!("Session is inactive or failed integrity check");
        return Err(StatusCode::UNAUTHORIZED);
    }

    if session.token_expiration < Utc::now() {
        tracing::warn!("Session has expired");
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
    let mut updated_session: session::ActiveModel = session.into();
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
        token: token,
        refresh_token: String::new(), // We don't need to return a new refresh token for validation
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
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<SessionResponse>, StatusCode> {
    let refresh_token = bearer.token().to_string();
    
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
