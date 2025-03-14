use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub environment: String,
    pub admin_email: String,
    pub admin_password: String,
    // Add other configuration items
}

impl AppConfig {
    pub fn from_env() -> Self {
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| "your_jwt_secret".to_string()),
            environment,
            admin_email: env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@example.com".to_string()),
            admin_password: env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "password".to_string()),
            // Initialize other config values
        }
    }
    
    pub fn is_production(&self) -> bool {
        self.environment == "production"
    }
} 