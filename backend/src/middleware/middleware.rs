use axum::{
    middleware::Next,
    response::Response,
    http::{StatusCode, Request, Method},
    Extension,
};
use crate::entities::{user, session, user_account, profile, directory};
use sea_orm::{EntityTrait, DatabaseConnection, QueryFilter, ColumnTrait, Set};
use uuid::Uuid;
use chrono::Utc;
use axum::http;
use axum::extract::State;
use crate::middleware::request_logger::RequestLogger;
use crate::models::request_log::RequestType;
use http::header;
use crate::models::request_log::RequestInfo;
use crate::middleware::rate_limiter::RateLimiter;

pub async fn auth_middleware<B>(
    db: DatabaseConnection,
    rate_limiter: RateLimiter,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode>
where
    B: axum::body::HttpBody + Send + 'static + std::fmt::Debug,
{
    // Allow OPTIONS requests to pass through without authentication
    if req.method() == Method::OPTIONS {
        return Ok(next.run(req).await);
    }
    println!("TEST LOG: from auth_middleware and req: {:?}", req);
    // Initialize the request logger
    let request_logger = RequestLogger::new(db.clone());

    // Extract request details
    let path = req.uri().path().to_owned();
    let method = req.method().clone();
    let uri = req.uri().clone();

    tracing::info!("Processing request: {} {}", method, path);

    // Extract user information from request headers
    let (user_id, user_agent, ip_address) = {
        let headers = req.headers();
        let user_agent = headers
            .get(header::USER_AGENT)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("Unknown")
            .to_string();
        let ip_address = headers
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("Unknown")
            .to_string();
        let user_id = req.extensions().get::<user::Model>().map(|user| user.id);
        (user_id, user_agent, ip_address)
    };
    println!("TEST LOG: from auth_middleware and user_id: {:?}", user_id);
    // Determine the request type (Login or API)
    let request_type = if path == "/login" { RequestType::Login } else { RequestType::API };
    println!("TEST LOG: from auth_middleware and request_type: {:?}", request_type);
    tracing::debug!("Request type: {:?}", request_type);
    tracing::info!("Path for request: {:?}", &path);

    // Handle public routes with rate limiting
    if is_public_route(&path) {
        tracing::debug!("Public route detected, applying rate limiting");
        match rate_limiter.check_rate_limit(&req).await {
            Ok(_) => {
                let (parts, body) = req.into_parts();
                let req_info = RequestInfo::from_parts(&parts);
                let mut req = Request::from_parts(parts, body);

                if let Err(e) = request_logger.log_request(&req).await {
                    println!("TEST LOG: from auth_middleware and error: {:?}", e);
                    tracing::error!("Failed to log request: {:?}", e);
                }
                println!("TEST LOG: from auth_middleware and req: {:?}", req);
                tracing::debug!("Rate limiting successful");
                let response = next.run(req).await;
                return Ok(response)
            },
            Err(status) => return Err(status),
        }
    }

    tracing::debug!("Authenticating request");
    // Extract bearer token from request
    let token = extract_token(&req);
    tracing::debug!("Token extracted: {}", token.is_some());

    // Validate session using the token
    let session = match validate_session(&db, token).await {
        Ok(session) => {
            println!("TEST LOG: from auth_middleware and session: {:?}", session);
            tracing::debug!("Session validated successfully");
            session
        },
        Err(status) => {
            println!("TEST LOG: from auth_middleware and error: {:?}", status);
            tracing::warn!("Session validation failed: {:?}", status);
            return Err(status);
        }
    };

    // Retrieve user associated with the session
    let user = match get_user(&db, &session).await {
        Ok(user) => {
            println!("TEST LOG: from auth_middleware and user: {:?}", user);
            tracing::debug!("User retrieved successfully: {:?}", user.id);
            user
        },
        Err(status) => {
            println!("TEST LOG: from auth_middleware and error: {:?}", status);
            tracing::warn!("Failed to retrieve user: {:?}", status);
            return Err(status);
        }
    };

    // Check admin access for admin routes
    if req.uri().path().starts_with("/api/admin") && !user.is_admin {
        println!("TEST LOG: from auth_middleware and user: {:?}", user);
        tracing::warn!("Non-admin user {:?} attempted to access admin route", user.id);
        return Err(StatusCode::FORBIDDEN);
    }

    // Update session's last accessed time
    if let Err(e) = update_session(&db, &session).await {
        println!("TEST LOG: from auth_middleware and error: {:?}", e);
        tracing::error!("Failed to update session: {:?}", e);
        return Err(e);
    }

    // Insert user and session into request extensions for downstream handlers
    tracing::debug!("Inserting user and session into request extensions");
    println!("TEST LOG: from auth_middleware and user: {:?}", user);
    req.extensions_mut().insert(user.clone());
    req.extensions_mut().insert(session.clone());

    // Retrieve and insert user's directory IDs into request extensions
    let directory_ids = match get_user_directory_ids(&db, &user).await {
        Ok(ids) => {
            println!("TEST LOG: from auth_middleware and ids: {:?}", ids);
            tracing::debug!("Retrieved {} directory IDs for user", ids.len());
            ids
        },
        Err(e) => {
            println!("TEST LOG: from auth_middleware and error: {:?}", e);
            tracing::error!("Failed to get user directory IDs: {:?}", e);
            return Err(e);
        }
    };
    req.extensions_mut().insert(directory_ids);

    // Execute the next middleware in the chain
    tracing::debug!("Executing next middleware");

    // Log the request
    println!("TEST LOG: from auth_middleware and req: {:?}", req);
    if let Err(e) = request_logger.log_request(&req).await {
        println!("TEST LOG: from auth_middleware and error: {:?}", e);
        tracing::error!("Failed to log request: {:?}", e);
    }
    let response = next.run(req).await;
    println!("TEST LOG: from auth_middleware and response: {:?}", response);
    let status_code = response.status();

    tracing::info!("Request completed: {} {} - Status: {}", method, path, status_code);
    Ok(response)
}

// Check if the given path is a public route
fn is_public_route(path: &str) -> bool {
    let public_routes = vec![
        "/login",
        "/register",
        "/validate-session",
        "/refresh-token",
        "/api/listings",
        "/api/listing/",
        "/api/health",
        // Adding these routes to handle both prefixed and non-prefixed versions
        "/api/login",
        "/api/register",
        "/api/validate-session"
    ];
    public_routes.iter().any(|route| path.starts_with(route))
}

// Extract bearer token from the request headers
fn extract_token<B>(req: &Request<B>) -> Option<String> {
    req.headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|auth_value| {
            if auth_value.starts_with("Bearer ") {
                Some(auth_value[7..].to_owned())
            } else if auth_value.starts_with("bearer ") {
                Some(auth_value[7..].to_owned())
            } else {
                None
            }
        })
}

// Validate the session using the provided token
// Updated validate_session function with better error handling
async fn validate_session(db: &DatabaseConnection, token: Option<String>) -> Result<session::Model, StatusCode> {
    println!("TEST LOG: from validate_session and token: {:?}", token);
    let token = token.ok_or(StatusCode::UNAUTHORIZED)?;
    
    if token.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let session = session::Entity::find()
        .filter(session::Column::BearerToken.eq(token))
        .filter(session::Column::IsActive.eq(true))
        .one(db)
        .await
        .map_err(|e| {
            tracing::error!("Database error in validate_session: {:?}", e);
            println!("TEST LOG: from validate_session and error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !session.verify_integrity() || session.token_expiration < Utc::now() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(session)
}

// Retrieve the user associated with the given session
async fn get_user(db: &DatabaseConnection, session: &session::Model) -> Result<user::Model, StatusCode> {
    user::Entity::find_by_id(session.user_id)
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)
}

// Update the session's last accessed time
async fn update_session(db: &DatabaseConnection, session: &session::Model) -> Result<(), StatusCode> {
    session::Entity::update(session::ActiveModel {
        id: Set(session.id),
        last_accessed_at: Set(Utc::now()),
        ..Default::default()
    })
    .exec(db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}

// Retrieve the directory IDs associated with the user
async fn get_user_directory_ids(db: &DatabaseConnection, user: &user::Model) -> Result<Vec<Uuid>, StatusCode> {
    // Fetch user accounts associated with the user
    let user_accounts: Vec<Uuid> = user_account::Entity::find()
        .filter(user_account::Column::UserId.eq(user.id))
        .all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|user_account| user_account.account_id)
        .collect();
    // Get profiles from account ids on user_account
    let profiles = profile::Entity::find()
        .filter(profile::Column::AccountId.is_in(user_accounts))
        .all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    // Get directories from profiles
    let directories = directory::Entity::find()
        .filter(directory::Column::Id.is_in(profiles.into_iter().map(|profile| profile.directory_id)))
        .all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    return Ok(directories.into_iter().map(|directory| directory.id).collect());
}
