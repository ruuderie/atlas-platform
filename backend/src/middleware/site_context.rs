// backend/src/middleware/site_context.rs
use axum::{
    extract::{Extension, Host},
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use sea_orm::{DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter};
use std::sync::Arc;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use tokio::sync::RwLock;
use crate::entities::directory;
use crate::config::{SiteConfig, ModuleFlags};
use serde_json::Value;

// Cache for site configurations to avoid frequent DB lookups
static SITE_CACHE: Lazy<Arc<RwLock<HashMap<String, SiteConfig>>>> = 
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

pub async fn site_context_middleware<B>(
    Extension(db): Extension<DatabaseConnection>,
    Host(hostname): Host,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let domain = hostname.split(':').next().unwrap_or(&hostname).to_string();
    
    tracing::info!("Processing site context for domain: {}", domain);
    tracing::debug!("Original hostname with potential port: {}", hostname);
    
    // Skip site context for admin routes and authentication routes
    if is_admin_route(req.uri().path()) || is_auth_route(req.uri().path()) {
        tracing::debug!("Skipping site context for admin/auth route: {}", req.uri().path());
        return Ok(next.run(req).await);
    }
    
    // Skip for localhost in development mode (optional)
    if domain == "localhost" && cfg!(debug_assertions) {
        tracing::debug!("Development mode: Skipping site context for localhost");
        return Ok(next.run(req).await);
    }
    
    // Try to get from cache first
    tracing::debug!("Attempting to retrieve site config from cache");
    let site_config = {
        let cache = SITE_CACHE.read().await;
        tracing::debug!("Cache read lock acquired. Cache size: {} entries", cache.len());
        cache.get(&domain).cloned()
    };
    
    let site_config = match site_config {
        Some(config) => {
            tracing::info!("Cache hit: Found site configuration for domain: {}", domain);
            tracing::debug!("Site name: {}, Directory ID: {}", config.name, config.directory_id);
            tracing::debug!("Enabled modules: {:?}", config.enabled_modules);
            config
        },
        None => {
            tracing::info!("Cache miss for domain: {}, fetching from database", domain);
            
            // Fetch from database
            tracing::debug!("Querying database for directory with domain or custom_domain = {}", domain);
            let directory = directory::Entity::find()
                .filter(
                    directory::Column::Domain.eq(&domain)
                    .or(directory::Column::CustomDomain.eq(&domain))
                )
                .one(&db)
                .await
                .map_err(|e| {
                    tracing::error!("Database error fetching directory: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
            
            let directory = match directory {
                Some(dir) => {
                    tracing::info!("Found directory in database: id={}, name={}", dir.id, dir.name);
                    dir
                },
                None => {
                    tracing::warn!("No directory found in database for domain: {}", domain);
                    return Err(StatusCode::NOT_FOUND);
                }
            };
            
            // Convert to SiteConfig
            tracing::debug!("Converting directory entity to SiteConfig");
            let config = SiteConfig {
                directory_id: directory.id,
                name: directory.name,
                domain: directory.domain,
                subdomain: directory.subdomain,
                custom_domain: directory.custom_domain,
                enabled_modules: ModuleFlags::from_bits_truncate(directory.enabled_modules),
                theme: directory.theme,
                custom_settings: directory.custom_settings
                    .unwrap_or_default()
                    .as_object()
                    .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default(),
                site_status: Some(directory.site_status),
            };
            
            tracing::debug!("Created SiteConfig with modules: {:?}", config.enabled_modules);
            
            // Update cache
            {
                tracing::debug!("Acquiring write lock to update cache");
                let mut cache = SITE_CACHE.write().await;
                tracing::debug!("Cache write lock acquired");
                cache.insert(domain.clone(), config.clone());
                tracing::info!("Updated cache with configuration for domain: {}", domain);
                tracing::debug!("Cache now contains {} entries", cache.len());
            }
            
            config
        }
    };
    
    // Add site config to request extensions
    tracing::debug!("Adding site config to request extensions for domain: {}", domain);
    let mut req = req;
    req.extensions_mut().insert(site_config);
    
    tracing::info!("Site context middleware completed for domain: {}", domain);
    Ok(next.run(req).await)
}

// Helper function to clear the cache (useful for testing or when configs are updated)
pub async fn clear_site_cache() {
    let mut cache = SITE_CACHE.write().await;
    cache.clear();
    tracing::info!("Site configuration cache cleared");
}

// Helper function to determine if a route is an admin route
fn is_admin_route(path: &str) -> bool {
    path.starts_with("/api/admin") || path.starts_with("/admin")
}

// Helper function to determine if a route is an authentication route
fn is_auth_route(path: &str) -> bool {
    matches!(path, "/login" | "/register" | "/validate-session" | "/refresh-token" | "/logout" |
                  "/api/login" | "/api/register" | "/api/validate-session" | "/api/refresh-token" | "/api/logout")
}