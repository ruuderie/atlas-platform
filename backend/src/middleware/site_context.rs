
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
use crate::config::site_config::{SiteConfig, ModuleFlags};

// Cache for site configurations to avoid frequent DB lookups
static SITE_CACHE: Lazy<Arc<RwLock<HashMap<String, SiteConfig>>>> = 
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

pub async fn site_context_middleware<B>(
    Extension(db): Extension<DatabaseConnection>,
    Host(hostname): Host,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // Check cache first
    let domain = hostname.split(':').next().unwrap_or(&hostname).to_string();
    
    // Try to get from cache
    let site_config = {
        let cache = SITE_CACHE.read().await;
        cache.get(&domain).cloned()
    };
    
    let site_config = match site_config {
        Some(config) => config,
        None => {
            // Not in cache, fetch from database
            let directory = directory::Entity::find()
                .filter(directory::Column::Domain.eq(&domain))
                .one(&db)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            let directory = match directory {
                Some(dir) => dir,
                None => return Err(StatusCode::NOT_FOUND),
            };
            
            // Convert to SiteConfig
            let enabled_modules_value = directory.additional_info
                .get("enabled_modules")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;
            
            let enabled_modules = ModuleFlags::from_bits_truncate(enabled_modules_value);
            
            let theme = directory.additional_info
                .get("theme")
                .and_then(|v| v.as_str())
                .unwrap_or("default")
                .to_string();
            
            let custom_settings = directory.additional_info
                .get("custom_settings")
                .and_then(|v| v.as_object().cloned())
                .unwrap_or_default();
            
            let config = SiteConfig {
                directory_id: directory.id,
                name: directory.name,
                domain,
                enabled_modules,
                theme,
                custom_settings: custom_settings.into_iter().collect(),
            };
            
            // Update cache
            {
                let mut cache = SITE_CACHE.write().await;
                cache.insert(config.domain.clone(), config.clone());
            }
            
            config
        }
    };
    
    // Add site config to request extensions
    let mut req = req;
    req.extensions_mut().insert(site_config);
    
    // Continue with the request
    Ok(next.run(req).await)
}