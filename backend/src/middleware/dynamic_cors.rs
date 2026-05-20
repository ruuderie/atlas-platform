use std::sync::Arc;
use dashmap::DashMap;
use sea_orm::{DatabaseConnection, EntityTrait};
use url::Url;
use axum::http::{HeaderValue, Method, header};
use tower_http::cors::CorsLayer;
use crate::entities::app_domain;

#[derive(Clone)]
pub struct DynamicCorsRegistry {
    db: Option<DatabaseConnection>,
    is_dev: bool,
    // Store allowed hosts (e.g. "acme.com", "dev.buildwithruud.com") in lowercase
    allowed_hosts: Arc<DashMap<String, ()>>,
}

impl DynamicCorsRegistry {
    /// Create a new registry backed by the database.
    pub fn new(db: DatabaseConnection) -> Self {
        let is_dev = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "production".to_string()) == "development";
        Self {
            db: Some(db),
            is_dev,
            allowed_hosts: Arc::new(DashMap::new()),
        }
    }

    /// Create an in-memory-only registry for testing/unit-tests.
    pub fn new_in_memory() -> Self {
        Self {
            db: None,
            is_dev: false,
            allowed_hosts: Arc::new(DashMap::new()),
        }
    }

    /// Dynamically add a single host to the in-memory allowlist.
    pub fn add_host(&self, host: &str) {
        let normalized = host.trim().to_lowercase();
        if !normalized.is_empty() {
            tracing::info!("Dynamic CORS: Adding host '{}' to registry", normalized);
            self.allowed_hosts.insert(normalized, ());
        }
    }

    /// Check if the in-memory allowlist contains the host.
    pub fn contains_host(&self, host: &str) -> bool {
        let normalized = host.trim().to_lowercase();
        self.allowed_hosts.contains_key(&normalized)
    }

    /// Hydrate the in-memory registry with all domains from the database,
    /// plus first-party origins from environment variables.
    pub async fn hydrate(&self, first_party_origins: &[String]) {
        // 1. Add first-party origins
        for origin in first_party_origins {
            if let Ok(url) = Url::parse(origin) {
                if let Some(host) = url.host_str() {
                    self.add_host(host);
                }
            } else {
                // If it's just a raw host/domain, add it directly
                self.add_host(origin);
            }
        }

        // 2. Add ADDITIONAL_ALLOWED_ORIGINS
        if let Ok(additional) = std::env::var("ADDITIONAL_ALLOWED_ORIGINS") {
            for origin in additional.split(',') {
                let trimmed = origin.trim();
                if let Ok(url) = Url::parse(trimmed) {
                    if let Some(host) = url.host_str() {
                        self.add_host(host);
                    }
                } else {
                    self.add_host(trimmed);
                }
            }
        }

        // 3. Hydrate from app_domains table
        if let Some(ref db) = self.db {
            match app_domain::Entity::find().all(db).await {
                Ok(domains) => {
                    tracing::info!("Dynamic CORS: Hydrating with {} domains from DB", domains.len());
                    for domain in domains {
                        self.add_host(&domain.domain_name);
                    }
                }
                Err(e) => {
                    tracing::error!("Dynamic CORS: Failed to query app_domains for hydration: {}", e);
                }
            }
        }
    }

    /// Check if the incoming origin is allowed.
    pub fn is_origin_allowed(&self, origin_val: &HeaderValue) -> bool {
        if self.is_dev {
            return true;
        }

        let origin_str = match origin_val.to_str() {
            Ok(s) => s,
            Err(_) => return false,
        };

        // Parse host from origin
        let host = match Url::parse(origin_str) {
            Ok(url) => match url.host_str() {
                Some(h) => h.to_lowercase(),
                None => return false,
            },
            Err(_) => {
                // Fallback: simple string manipulation if it doesn't parse as absolute URL
                origin_str
                    .trim_start_matches("https://")
                    .trim_start_matches("http://")
                    .split(':')
                    .next()
                    .unwrap_or("")
                    .to_lowercase()
            }
        };

        let is_allowed = self.allowed_hosts.contains_key(&host);
        tracing::debug!(
            "Dynamic CORS: origin '{}' (parsed host '{}') allowed = {}",
            origin_str, host, is_allowed
        );
        is_allowed
    }
}

/// Construct a tower-http CorsLayer configured with dynamic origin validation.
pub fn dynamic_cors_layer(registry: Arc<DynamicCorsRegistry>) -> CorsLayer {
    let registry_clone = registry.clone();

    CorsLayer::new()
        .allow_origin(tower_http::cors::AllowOrigin::predicate(move |origin, _parts| {
            registry_clone.is_origin_allowed(origin)
        }))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([
            header::AUTHORIZATION,
            header::ACCEPT,
            header::CONTENT_TYPE,
            header::ORIGIN,
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
            header::ACCESS_CONTROL_ALLOW_HEADERS,
        ])
        .allow_credentials(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_registry() {
        let registry = DynamicCorsRegistry::new_in_memory();
        
        // Initial state is empty
        assert!(!registry.contains_host("acme.com"));
        assert!(!registry.is_origin_allowed(&HeaderValue::from_static("https://acme.com")));

        // Add host
        registry.add_host("acme.com");
        assert!(registry.contains_host("acme.com"));
        assert!(registry.is_origin_allowed(&HeaderValue::from_static("https://acme.com")));
        assert!(registry.is_origin_allowed(&HeaderValue::from_static("http://acme.com:8080")));
        
        // Case insensitivity
        assert!(registry.contains_host("ACME.COM"));
        assert!(registry.is_origin_allowed(&HeaderValue::from_static("https://ACME.com")));

        // Non-matching host
        assert!(!registry.contains_host("evil.com"));
        assert!(!registry.is_origin_allowed(&HeaderValue::from_static("https://evil.com")));
    }

    #[test]
    fn test_dev_mode() {
        let mut registry = DynamicCorsRegistry::new_in_memory();
        registry.is_dev = true;

        // Even without adding, everything is allowed in dev mode
        assert!(registry.is_origin_allowed(&HeaderValue::from_static("https://acme.com")));
        assert!(registry.is_origin_allowed(&HeaderValue::from_static("https://evil.com")));
    }
}
