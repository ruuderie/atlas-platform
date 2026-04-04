use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use serde::de::DeserializeOwned;
use chrono::{Utc, Duration};
use bcrypt::{hash, verify, DEFAULT_COST};
use crate::entities::user;
use std::env;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,           // User ID
    pub is_admin: bool,        // Admin flag
    pub exp: usize,            // Expiration timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,   // JWT ID for tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impersonator_id: Option<String>, // Admin ID impersonating this session
}

pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    let operation_id = Uuid::new_v4();
    tracing::debug!("[{}] Hashing password (length: {})", operation_id, password.len());
    
    let start = std::time::Instant::now();
    let result = hash(password, DEFAULT_COST);
    let duration = start.elapsed();
    
    match &result {
        Ok(hash) => {
            tracing::debug!("[{}] Password hashed successfully in {:.2}ms (hash length: {})", 
                operation_id, duration.as_millis(), hash.len());
        },
        Err(e) => {
            tracing::error!("[{}] Password hashing failed: {:?} (took {:.2}ms)", 
                operation_id, e, duration.as_millis());
        }
    }
    
    result
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    let operation_id = Uuid::new_v4();
    tracing::debug!("[{}] Verifying password (password length: {}, hash length: {})", 
        operation_id, password.len(), hash.len());
    
    let start = std::time::Instant::now();
    let result = verify(password, hash);
    let duration = start.elapsed();
    
    match &result {
        Ok(is_valid) => {
            if *is_valid {
                tracing::info!("[{}] Password verification successful (took {:.2}ms)", 
                    operation_id, duration.as_millis());
            } else {
                tracing::warn!("[{}] Password verification failed - invalid password (took {:.2}ms)", 
                    operation_id, duration.as_millis());
            }
        },
        Err(e) => {
            tracing::error!("[{}] Password verification error: {:?} (took {:.2}ms)", 
                operation_id, e, duration.as_millis());
        }
    }
    
    result
}

pub fn generate_jwt(user: &user::Model) -> Result<String, jsonwebtoken::errors::Error> {
    let operation_id = Uuid::new_v4();
    tracing::debug!("[{}] Generating JWT for user: {} ({})", 
        operation_id, user.id, user.email);
    
    // Get JWT secret from environment or use default (for development only)
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
        tracing::warn!("[{}] JWT_SECRET not set in environment, using default (insecure)", operation_id);
        "your-secret-key".to_string()
    });
    
    let jwt_expiry_hours = env::var("JWT_EXPIRY_HOURS")
        .ok()
        .and_then(|val| val.parse::<i64>().ok())
        .unwrap_or(24);
        
    tracing::debug!("[{}] JWT will expire in {} hours", operation_id, jwt_expiry_hours);

    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(jwt_expiry_hours))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user.id.to_string(),
        is_admin: user.is_admin,
        exp: expiration as usize,
        jti: Some(Uuid::new_v4().to_string()),
        impersonator_id: None,
    };

    tracing::debug!("[{}] JWT claims: sub={}, is_admin={}, exp={}, jti={}", 
        operation_id, claims.sub, claims.is_admin, claims.exp, claims.jti.as_ref().unwrap_or(&"none".to_string()));

    let header = Header::default();
    let encoding_key = EncodingKey::from_secret(jwt_secret.as_ref());

    let start = std::time::Instant::now();
    let result = encode(&header, &claims, &encoding_key);
    let duration = start.elapsed();
    
    match &result {
        Ok(token) => {
            let token_preview = if token.len() > 20 {
                format!("{}...{}", &token[0..10], &token[token.len()-10..])
            } else {
                token.clone()
            };
            
            tracing::info!("[{}] JWT generated successfully for user {} (token: {}, took {:.2}ms)", 
                operation_id, user.id, token_preview, duration.as_millis());
        },
        Err(e) => {
            tracing::error!("[{}] JWT generation failed for user {}: {:?} (took {:.2}ms)", 
                operation_id, user.id, e, duration.as_millis());
        }
    }
    
    result
}

pub fn generate_jwt_admin(user: &user::Model) -> Result<String, jsonwebtoken::errors::Error> {
    let operation_id = Uuid::new_v4();
    
    if !user.is_admin {
        tracing::warn!("[{}] Attempted to generate admin JWT for non-admin user: {} ({})", 
            operation_id, user.id, user.email);
    }
    
    tracing::debug!("[{}] Generating admin JWT for user: {} ({})", 
        operation_id, user.id, user.email);
    
    // Get JWT secret from environment or use default (for development only)
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
        tracing::warn!("[{}] JWT_SECRET not set in environment, using default (insecure)", operation_id);
        "your-secret-key".to_string()
    });
    
    let jwt_expiry_hours = env::var("ADMIN_JWT_EXPIRY_HOURS")
        .ok()
        .and_then(|val| val.parse::<i64>().ok())
        .unwrap_or(12); // Shorter expiry for admin tokens
        
    tracing::debug!("[{}] Admin JWT will expire in {} hours", operation_id, jwt_expiry_hours);

    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(jwt_expiry_hours))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user.id.to_string(),
        exp: expiration as usize,
        is_admin: true, // Force admin flag to true
        jti: Some(Uuid::new_v4().to_string()),
        impersonator_id: None,
    };

    tracing::debug!("[{}] Admin JWT claims: sub={}, is_admin={}, exp={}, jti={}", 
        operation_id, claims.sub, claims.is_admin, claims.exp, claims.jti.as_ref().unwrap_or(&"none".to_string()));

    let header = Header::default();
    let encoding_key = EncodingKey::from_secret(jwt_secret.as_ref());

    let start = std::time::Instant::now();
    let result = encode(&header, &claims, &encoding_key);
    let duration = start.elapsed();
    
    match &result {
        Ok(token) => {
            let token_preview = if token.len() > 20 {
                format!("{}...{}", &token[0..10], &token[token.len()-10..])
            } else {
                token.clone()
            };
            
            tracing::info!("[{}] Admin JWT generated successfully for user {} (token: {}, took {:.2}ms)", 
                operation_id, user.id, token_preview, duration.as_millis());
        },
        Err(e) => {
            tracing::error!("[{}] Admin JWT generation failed for user {}: {:?} (took {:.2}ms)", 
                operation_id, user.id, e, duration.as_millis());
        }
    }
    
    result
}

pub fn generate_impersonation_jwt(target_user: &user::Model, admin_id: &Uuid) -> Result<String, jsonwebtoken::errors::Error> {
    let operation_id = Uuid::new_v4();
    tracing::info!("[{}] Generating IMPERSONATION JWT for user: {} ({}) by Admin: {}", 
        operation_id, target_user.id, target_user.email, admin_id);
    
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
        "your-secret-key".to_string()
    });
    
    let jwt_expiry_hours = 2; // Hardcode a short 2 hour span for impersonation scopes
        
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(jwt_expiry_hours))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: target_user.id.to_string(),
        is_admin: target_user.is_admin, // Carry over whatever bounds the target has naturally
        exp: expiration as usize,
        jti: Some(Uuid::new_v4().to_string()),
        impersonator_id: Some(admin_id.to_string()),
    };

    let header = Header::default();
    let encoding_key = EncodingKey::from_secret(jwt_secret.as_ref());
    encode(&header, &claims, &encoding_key)
}

pub fn validate_jwt<T: DeserializeOwned>(token: &str) -> Result<T, jsonwebtoken::errors::Error> {
    let operation_id = Uuid::new_v4();
    
    let token_preview = if token.len() > 20 {
        format!("{}...{}", &token[0..10], &token[token.len()-10..])
    } else {
        token.to_string()
    };
    
    tracing::debug!("[{}] Validating JWT: {}", operation_id, token_preview);
    
    // Get JWT secret from environment or use default (for development only)
    let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
        tracing::warn!("[{}] JWT_SECRET not set in environment, using default (insecure)", operation_id);
        "your-secret-key".to_string()
    });
    
    let decoding_key = DecodingKey::from_secret(jwt_secret.as_ref());
    let mut validation = Validation::default();
    
    // Configure validation options
    validation.validate_exp = true;
    validation.leeway = 60; // 1 minute leeway for clock skew
    
    tracing::debug!("[{}] JWT validation parameters: {:?}", operation_id, validation);

    let start = std::time::Instant::now();
    let result = decode::<T>(token, &decoding_key, &validation);
    let duration = start.elapsed();
    
    match &result {
        Ok(_token_data) => {
            tracing::info!("[{}] JWT validated successfully (type: {}, took {:.2}ms)", 
                operation_id, std::any::type_name::<T>(), duration.as_millis());
                
            // Additional logging for Claims type - only attempt if T is our Claims type
            if std::any::type_name::<T>().contains("Claims") {
                // Just log that we decoded claims without showing the content
                tracing::debug!("[{}] Decoded claims successfully", operation_id);
            }
        },
        Err(e) => {
            tracing::warn!("[{}] JWT validation failed: {:?} (took {:.2}ms)", 
                operation_id, e, duration.as_millis());
                
            match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                    tracing::info!("[{}] JWT expired", operation_id);
                },
                jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                    tracing::warn!("[{}] JWT has invalid signature - possible tampering attempt", operation_id);
                },
                _ => {
                    tracing::error!("[{}] JWT validation error: {:?}", operation_id, e);
                }
            }
        }
    }
    
    result.map(|token_data| token_data.claims)
}

// Helper function to extract JWT from Authorization header
pub fn extract_jwt_from_header(headers: &axum::http::HeaderMap) -> Option<String> {
    let operation_id = Uuid::new_v4();
    
    tracing::debug!("[{}] Extracting JWT from Authorization header", operation_id);
    
    let auth_header = match headers.get("Authorization") {
        Some(header) => {
            match header.to_str() {
                Ok(value) => {
                    tracing::debug!("[{}] Authorization header found", operation_id);
                    value
                },
                Err(e) => {
                    tracing::warn!("[{}] Failed to parse Authorization header: {:?}", operation_id, e);
                    return None;
                }
            }
        },
        None => {
            tracing::debug!("[{}] No Authorization header found", operation_id);
            return None;
        }
    };
    
    if !auth_header.starts_with("Bearer ") {
        tracing::debug!("[{}] Authorization header is not a Bearer token", operation_id);
        return None;
    }
    
    let token = auth_header[7..].trim().to_string();
    
    if token.is_empty() {
        tracing::debug!("[{}] Bearer token is empty", operation_id);
        return None;
    }
    
    let token_preview = if token.len() > 20 {
        format!("{}...{}", &token[0..10], &token[token.len()-10..])
    } else {
        token.clone()
    };
    
    tracing::debug!("[{}] JWT extracted successfully: {}", operation_id, token_preview);
    Some(token)
}