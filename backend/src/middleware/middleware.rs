use axum::{
    middleware::Next,
    response::Response,
    extract::State,
    body::Body,
    http::{StatusCode, Request, Method},
    Extension,
};
use crate::entities::{user, session, user_account, profile, tenant};
use sea_orm::{EntityTrait, DatabaseConnection, QueryFilter, ColumnTrait, Set};
use uuid::Uuid;
use chrono::Utc;
use axum::http;
use crate::handlers::request_logs;
use crate::models::request_log::{RequestType, RequestStatus};
use http::header;
use crate::models::request_log::RequestInfo;
use crate::middleware::rate_limiter::RateLimiter;

pub async fn log_request_middleware<B>(
    State(db): State<DatabaseConnection>,
    request: Request<Body>,
    next: Next,
) -> Response {
    tracing::debug!("Logging request");
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    if path == "/validate-session" {
        tracing::debug!("Skipping request logging for /validate-session endpoint");
        return next.run(request).await;
    }
    let request_id = Uuid::new_v4();
    let uri = request.uri().clone();
    let headers = request.headers().clone();
    let user_id = request.extensions().get::<crate::entities::user::Model>().map(|user| {
        tracing::debug!("Request associated with authenticated user ID: {}", user.id);
        user.id
    });
    let ip_address = headers.get("x-forwarded-for").and_then(|h| h.to_str().ok()).unwrap_or("Unknown").to_string();
    let user_agent = headers.get("user-agent").and_then(|h| h.to_str().ok()).unwrap_or("Unknown").to_string();
    let request_type = if path == "/login" { RequestType::Login } else { RequestType::API };
    tracing::info!(
        "Request received: ID: {}, Method: {}, Path: {}, User ID: {:?}, IP: {}, User-Agent: {}, Type: {:?}",
        request_id, method, path, user_id, ip_address, 
        if user_agent.len() > 30 { &user_agent[0..30] } else { &user_agent },
        request_type
    );
    if path == "/login" {
        tracing::debug!("Processing login request - Headers present: {:?}", headers.keys().map(|k| k.as_str()).collect::<Vec<_>>());
        if let Some(origin) = headers.get("origin").and_then(|h| h.to_str().ok()) {
            tracing::debug!("Login request origin: {}", origin);
        }
        if method == Method::OPTIONS {
            tracing::debug!("Received OPTIONS preflight request for login endpoint");
        }
    }
    match request_logs::log_request(
        method, 
        uri, 
        StatusCode::OK, 
        user_id, 
        &user_agent, 
        &ip_address, 
        request_type, 
        RequestStatus::Success, 
        None, 
        &db
    ).await {
        Ok(_) => tracing::debug!("Successfully logged request to database"),
        Err(e) => tracing::error!("Failed to log request to database: {}", e),
    }
    next.run(request).await
}

pub async fn auth_middleware(
    Extension(db): Extension<DatabaseConnection>,
    Extension(rate_limiter): Extension<RateLimiter>,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let request_id = Uuid::new_v4();
    let path = req.uri().path().to_owned();
    let method = req.method().clone();
    let uri = req.uri().clone();
    
    tracing::info!("[{}] Request started: {} {}", request_id, method, path);
    
    // Log all headers for debugging
    tracing::debug!("[{}] Request headers:", request_id);
    for (name, value) in req.headers() {
        tracing::debug!("[{}]   {}: {:?}", request_id, name, value);
    }

    // Allow OPTIONS requests to pass through without authentication
    if req.method() == Method::OPTIONS {
        tracing::info!("[{}] OPTIONS request detected - bypassing authentication", request_id);
        let response = next.run(req).await;
        tracing::info!("[{}] OPTIONS request completed with status: {}", request_id, response.status());
        return Ok(response);
    }

    // Extract user information from request headers
    let (user_id, user_agent, ip_address) = {
        let headers = req.headers();
        tracing::debug!("[{}] Request headers: {:?}", request_id, headers.keys().map(|k| k.as_str()).collect::<Vec<_>>());
        
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
        
        tracing::debug!("[{}] Client info - IP: {}, User-Agent: {}, User ID: {:?}", 
            request_id, ip_address, 
            if user_agent.len() > 30 { &user_agent[0..30] } else { &user_agent },
            user_id);
            
        (user_id, user_agent, ip_address)
    };

    // Determine the request type (Login or API)
    let request_type = if path == "/login" { 
        tracing::info!("[{}] Login request detected", request_id);
        RequestType::Login 
    } else { 
        RequestType::API 
    };
    
    tracing::debug!("[{}] Request type: {:?}", request_id, request_type);

    // Handle public routes with rate limiting
    if is_public_route(&path) {
        tracing::info!("[{}] Public route detected: {}", request_id, path);
        
        // Apply rate limiting using extracted IP
        tracing::debug!("[{}] Applying rate limiting", request_id);
        match rate_limiter.check_rate_limit(&ip_address).await {
            Ok(_) => {
                tracing::debug!("[{}] Rate limit check passed", request_id);
                let (parts, body) = req.into_parts();
                let req_info = RequestInfo::from_parts(&parts);
                let mut req = Request::from_parts(parts, body);

                // Log the request
                match request_logs::log_request(
                    method.clone(),
                    uri.clone(),
                    StatusCode::OK,
                    user_id,
                    &user_agent,
                    &ip_address,
                    request_type,
                    RequestStatus::Success,
                    None,
                    &db
                ).await {
                    Ok(_) => tracing::debug!("[{}] Request logged successfully", request_id),
                    Err(e) => tracing::error!("[{}] Failed to log request: {:?}", request_id, e),
                }
                
                tracing::info!("[{}] Forwarding public route request to handler", request_id);
                let response = next.run(req).await;
                tracing::info!("[{}] Public route request completed with status: {}", request_id, response.status());
                return Ok(response)
            },
            Err(status) => {
                tracing::warn!("[{}] Rate limit exceeded, returning status: {}", request_id, status);
                return Err(status);
            },
        }
    }

    tracing::debug!("[{}] Protected route - authenticating request", request_id);
    
    // Extract bearer token from request
    let token = extract_token(&req);
    tracing::debug!("[{}] Bearer token extracted: {}", request_id, token.is_some());

    // Validate session using the token
    let session = match validate_session(&db, token).await {
        Ok(session) => {
            tracing::info!("[{}] Session validated successfully: {}", request_id, session.id);
            session
        },
        Err(status) => {
            tracing::warn!("[{}] Session validation failed with status: {}", request_id, status);
            return Err(status);
        }
    };

    // Retrieve user associated with the session
    let user = match get_user(&db, &session).await {
        Ok(user) => {
            tracing::info!("[{}] User retrieved successfully: {} ({})", request_id, user.id, user.email);
            user
        },
        Err(status) => {
            tracing::warn!("[{}] Failed to retrieve user with status: {}", request_id, status);
            return Err(status);
        }
    };

    // Check admin access for admin routes
    if req.uri().path().starts_with("/api/admin") {
        tracing::debug!("[{}] Admin route access attempt", request_id);
        if !user.is_admin {
            tracing::warn!("[{}] Non-admin user {} attempted to access admin route: {}", 
                request_id, user.id, path);
            return Err(StatusCode::FORBIDDEN);
        }
        tracing::info!("[{}] Admin access granted for user: {}", request_id, user.id);
    }

    // Update session's last accessed time
    tracing::debug!("[{}] Updating session last accessed time", request_id);
    if let Err(e) = update_session(&db, &session).await {
        tracing::error!("[{}] Failed to update session: {:?}", request_id, e);
        return Err(e);
    }

    // Insert user and session into request extensions for downstream handlers
    tracing::debug!("[{}] Inserting user and session into request extensions", request_id);
    req.extensions_mut().insert(user.clone());
    req.extensions_mut().insert(session.clone());

    // Retrieve and insert user's directory IDs into request extensions
    tracing::debug!("[{}] Retrieving user tenant IDs", request_id);
    let directory_ids = match get_user_tenant_ids(&db, &user).await {
        Ok(ids) => {
            tracing::debug!("[{}] Retrieved {} tenant IDs for user", request_id, ids.len());
            ids
        },
        Err(e) => {
            tracing::error!("[{}] Failed to get user directory IDs: {:?}", request_id, e);
            return Err(e);
        }
    };
    req.extensions_mut().insert(directory_ids);

    // Log the request
    tracing::debug!("[{}] Logging authenticated request", request_id);
    if let Err(e) = request_logs::log_request(
        method.clone(),
        uri.clone(),
        StatusCode::OK,
        Some(user.id),
        &user_agent,
        &ip_address,
        request_type,
        RequestStatus::Success,
        None,
        &db
    ).await {
        tracing::error!("[{}] Failed to log request: {:?}", request_id, e);
    }
    
    // Execute the next middleware in the chain
    tracing::info!("[{}] Forwarding authenticated request to handler", request_id);
    let response = next.run(req).await;
    let status_code = response.status();

    tracing::info!("[{}] Request completed: {} {} - Status: {}", 
        request_id, method, path, status_code);
        
    Ok(response)
}

// Check if the given path is a public route
fn is_public_route(path: &str) -> bool {
    let public_routes = vec![
        "/login",
        "/register",
        "/validate-session",
        "/refresh-token",
        "/api/health",
        // Adding these routes to handle both prefixed and non-prefixed versions
        "/api/login",
        "/api/register",
        "/api/validate-session"
    ];
    
    let is_public = public_routes.iter().any(|route| path.starts_with(route));
    if is_public {
        tracing::debug!("Path '{}' identified as public route", path);
    }
    is_public
}

// Extract bearer token from the request headers
fn extract_token<B>(req: &Request<B>) -> Option<String> {
    let token = req.headers()
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
        });
        
    if token.is_none() {
        tracing::debug!("No bearer token found in request");
    }
    
    token
}

// Validate the session using the provided token
async fn validate_session(db: &DatabaseConnection, token: Option<String>) -> Result<session::Model, StatusCode> {
    let token = match token {
        Some(t) if !t.is_empty() => {
            tracing::debug!("Validating token: {}", t.chars().take(8).collect::<String>() + "...");
            t
        },
        _ => {
            tracing::debug!("Missing or empty token");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };
    
    tracing::debug!("Querying database for session with token");
    let session = match session::Entity::find()
        .filter(session::Column::BearerToken.eq(&token))
        .filter(session::Column::IsActive.eq(true))
        .one(db)
        .await {
            Ok(Some(session)) => {
                tracing::debug!("Session found: {}", session.id);
                session
            },
            Ok(None) => {
                tracing::debug!("No active session found for token");
                return Err(StatusCode::UNAUTHORIZED);
            },
            Err(e) => {
                tracing::error!("Database error in validate_session: {:?}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

    // Verify session integrity and expiration
    if !session.verify_integrity() {
        tracing::warn!("Session integrity check failed for session: {}", session.id);
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    if session.token_expiration < Utc::now() {
        tracing::warn!("Session token expired for session: {}", session.id);
        return Err(StatusCode::UNAUTHORIZED);
    }

    tracing::debug!("Session validated successfully: {}", session.id);
    Ok(session)
}

// Retrieve the user associated with the given session
async fn get_user(db: &DatabaseConnection, session: &session::Model) -> Result<user::Model, StatusCode> {
    tracing::debug!("Retrieving user for session: {}", session.id);
    
    match user::Entity::find_by_id(session.user_id)
        .one(db)
        .await {
            Ok(Some(user)) => {
                tracing::debug!("User found: {} ({})", user.id, user.email);
                Ok(user)
            },
            Ok(None) => {
                tracing::warn!("No user found for session: {}", session.id);
                Err(StatusCode::UNAUTHORIZED)
            },
            Err(e) => {
                tracing::error!("Database error in get_user: {:?}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
}

// Update the session's last accessed time
async fn update_session(db: &DatabaseConnection, session: &session::Model) -> Result<(), StatusCode> {
    tracing::debug!("Updating last_accessed_at for session: {}", session.id);
    
    match session::Entity::update(session::ActiveModel {
        id: Set(session.id),
        last_accessed_at: Set(Utc::now()),
        ..Default::default()
    })
    .exec(db)
    .await {
        Ok(_) => {
            tracing::debug!("Session updated successfully");
            Ok(())
        },
        Err(e) => {
            tracing::error!("Failed to update session: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Retrieve the tenant IDs associated with the user
async fn get_user_tenant_ids(db: &DatabaseConnection, user: &user::Model) -> Result<Vec<Uuid>, StatusCode> {
    tracing::debug!("Retrieving tenant IDs for user: {}", user.id);
    
    // Fetch user accounts associated with the user
    let user_accounts = match user_account::Entity::find()
        .filter(user_account::Column::UserId.eq(user.id))
        .all(db)
        .await {
            Ok(accounts) => {
                tracing::debug!("Found {} user accounts", accounts.len());
                accounts
            },
            Err(e) => {
                tracing::error!("Failed to retrieve user accounts: {:?}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
        
    let account_ids: Vec<Uuid> = user_accounts.into_iter()
        .map(|user_account| user_account.account_id)
        .collect();
        
    if account_ids.is_empty() {
        tracing::debug!("User has no associated accounts");
        return Ok(Vec::new());
    }
    
    // Get profiles from account ids on user_account
    let profiles = match profile::Entity::find()
        .filter(profile::Column::AccountId.is_in(account_ids))
        .all(db)
        .await {
            Ok(profiles) => {
                tracing::debug!("Found {} profiles", profiles.len());
                profiles
            },
            Err(e) => {
                tracing::error!("Failed to retrieve profiles: {:?}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
        
    if profiles.is_empty() {
        tracing::debug!("No profiles found for user's accounts");
        return Ok(Vec::new());
    }
    
    let profile_tenant_ids: Vec<Uuid> = profiles.into_iter()
        .map(|profile| profile.tenant_id)
        .collect();
        
    // Get tenants from profiles
    let tenants = match tenant::Entity::find()
        .filter(tenant::Column::Id.is_in(profile_tenant_ids))
        .all(db)
        .await {
            Ok(tenants) => {
                tracing::debug!("Found {} tenants", tenants.len());
                tenants
            },
            Err(e) => {
                tracing::error!("Failed to retrieve tenants: {:?}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
        
    let tenant_ids: Vec<Uuid> = tenants.into_iter()
        .map(|tenant| tenant.id)
        .collect();
        
    tracing::debug!("Retrieved {} tenant IDs for user", tenant_ids.len());
    Ok(tenant_ids)
}
