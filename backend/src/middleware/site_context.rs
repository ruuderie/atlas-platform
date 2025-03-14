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
    
    tracing::debug!("Processing site context for domain: {}", domain);
    
    // Try to get from cache first
    let site_config = {
        let cache = SITE_CACHE.read().await;
        cache.get(&domain).cloned()
    };
    
    let site_config = match site_config {
        Some(config) => {
            tracing::debug!("Found site configuration in cache for domain: {}", domain);
            config
        },
        None => {
            tracing::debug!("Cache miss for domain: {}, fetching from database", domain);
            
            // Fetch from database
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
                Some(dir) => dir,
                None => {
                    tracing::warn!("No directory found for domain: {}", domain);
                    return Err(StatusCode::NOT_FOUND);
                }
            };
            
            // Convert to SiteConfig
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
            
            // Update cache
            {
                let mut cache = SITE_CACHE.write().await;
                cache.insert(config.domain.clone(), config.clone());
                tracing::debug!("Updated cache with configuration for domain: {}", domain);
            }
            
            config
        }
    };
    
    // Add site config to request extensions
    let mut req = req;
    req.extensions_mut().insert(site_config);
    
    Ok(next.run(req).await)
}

// Helper function to clear the cache (useful for testing or when configs are updated)
pub async fn clear_site_cache() {
    let mut cache = SITE_CACHE.write().await;
    cache.clear();
    tracing::info!("Site configuration cache cleared");
}