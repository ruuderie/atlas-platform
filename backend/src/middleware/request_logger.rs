use axum::{
    http::{Method, Request, StatusCode, Uri},
    response::Response,
};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use crate::handlers::request_logs::log_request;
use crate::models::request_log::{RequestType, RequestStatus};
use uuid::Uuid;

#[derive(Clone)]
pub struct RequestLogger {
    db: Arc<DatabaseConnection>,
}

impl RequestLogger {
    pub fn new(db: DatabaseConnection) -> Self {
        RequestLogger {
            db: Arc::new(db),
        }
    }

    pub async fn log_request<B>(
        &self,
        req: &Request<B>,
    ) -> Result<(), StatusCode> {
        let method = req.method().clone();
        let path = req.uri().path().to_string();
        tracing::debug!("Request logging started - Method: {:?}, Path: {:?}", method, path);
    
        // Skip logging for validate-session to reduce noise
        if path == "/validate-session" {
            tracing::debug!("Skipping request logging for /validate-session endpoint");
            return Ok(());
        }
    
        // Generate a unique ID for this request for correlation
        let request_id = Uuid::new_v4();
        let uri = req.uri().clone();
        let headers = req.headers().clone();
        
        // Extract user information if available
        let user_id = req.extensions().get::<crate::entities::user::Model>().map(|user| {
            tracing::debug!("Request associated with authenticated user ID: {}", user.id);
            user.id
        });
    
        // Extract client information
        let ip_address = headers
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("Unknown")
            .to_string();
            
        let user_agent = headers
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("Unknown")
            .to_string();
    
        // Determine request type
        let request_type = if path == "/login" {
            tracing::debug!("Login request detected");
            RequestType::Login
        } else {
            RequestType::API
        };
    
        // Log detailed request information
        tracing::info!(
            "Request received: ID: {}, Method: {}, Path: {}, User ID: {:?}, IP: {}, User-Agent: {}, Type: {:?}",
            request_id, method, path, user_id, ip_address, 
            if user_agent.len() > 30 { &user_agent[0..30] } else { &user_agent }, // Truncate long user agents
            request_type
        );
    
        // For login requests, add extra debugging
        if path == "/login" {
            tracing::debug!(
                "Processing login request - Headers present: {:?}",
                headers.keys().map(|k| k.as_str()).collect::<Vec<_>>()
            );
            
            // Check for CORS-related headers
            if let Some(origin) = headers.get("origin").and_then(|h| h.to_str().ok()) {
                tracing::debug!("Login request origin: {}", origin);
            }
            
            if method == Method::OPTIONS {
                tracing::debug!("Received OPTIONS preflight request for login endpoint");
            }
        }
    
        // Log the request to the database
        match log_request(
            method, 
            uri, 
            StatusCode::OK, 
            user_id, 
            &user_agent, 
            &ip_address, 
            request_type, 
            RequestStatus::Success, 
            None, 
            &self.db
        ).await {
            Ok(_) => tracing::debug!("Successfully logged request to database"),
            Err(e) => {
                tracing::error!("Failed to log request to database: {}", e);
                eprintln!("Failed to log request: {}", e);
            }
        }
    
        Ok(())
    }

    pub async fn log_response(
        &self,
        response: &Response,
        method: Method,
        uri: Uri,
        user_id: Option<Uuid>,
        user_agent: &str,
        ip_address: &str,
        request_type: RequestType,
    ) -> Result<(), StatusCode> {
        let status = response.status();
        let path = uri.path();
        
        tracing::debug!(
            "Logging response - Path: {}, Method: {}, Status: {}", 
            path, method, status
        );
        
        let request_status = if status.is_success() {
            RequestStatus::Success
        } else {
            tracing::warn!(
                "Request failed - Path: {}, Method: {}, Status: {}", 
                path, method, status
            );
            RequestStatus::Failure
        };
        
        let failure_reason = if status.is_client_error() || status.is_server_error() {
            let reason = status.canonical_reason().unwrap_or("Unknown error").to_string();
            tracing::warn!("Request error: {}", reason);
            Some(reason)
        } else {
            None
        };

        // For login endpoint, add extra logging
        if path == "/login" {
            tracing::info!(
                "Login request completed - Status: {}, Success: {}", 
                status, status.is_success()
            );
            
            if let Some(reason) = &failure_reason {
                tracing::warn!("Login failed: {}", reason);
            }
        }

        if let Err(e) = log_request(
            method,
            uri,
            status,
            user_id,
            user_agent,
            ip_address,
            request_type,
            request_status,
            failure_reason,
            &self.db
        ).await {
            tracing::error!("Failed to log response: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }

        Ok(())
    }
}
