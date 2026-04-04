// backend/src/middleware/site_context.rs
use axum::{
    extract::{Extension},
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::{Host};
use sea_orm::{DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter};
use std::sync::Arc;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use tokio::sync::RwLock;
use crate::entities::{tenant, app_domain, app_instance};
use crate::config::{SiteConfig, ModuleFlags};

// Cache for site configurations to avoid frequent DB lookups
static SITE_CACHE: Lazy<Arc<RwLock<HashMap<String, SiteConfig>>>> = 
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

// We'll hardcode the admin routing check as well
fn is_admin_route(path: &str) -> bool {
    path.starts_with("/api/admin")
}

fn is_auth_route(path: &str) -> bool {
    path.starts_with("/login") || path.starts_with("/register") || path.starts_with("/auth")
}

fn is_setup_route(path: &str) -> bool {
    path.starts_with("/setup")
}

pub async fn site_context_middleware(
    Extension(db): Extension<DatabaseConnection>,
    Host(hostname): Host,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let domain = hostname.split(':').next().unwrap_or(&hostname).to_string();
    
    // Skip site context for admin routes, authentication routes, and setup routes
    if is_admin_route(req.uri().path()) || is_auth_route(req.uri().path()) || is_setup_route(req.uri().path()) {
        return Ok(next.run(req).await);
    }
    
    // Try to get from cache first
    let site_config = {
        let cache = SITE_CACHE.read().await;
        cache.get(&domain).cloned()
    };
    
    let site_config = match site_config {
        Some(config) => config,
        None => {
            // Find AppDomain
            let app_domain = app_domain::Entity::find()
                .filter(app_domain::Column::DomainName.eq(&domain))
                .one(&db)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
            let app_domain = match app_domain {
                Some(d) => d,
                None => {
                    // Fallback to "localhost" if debugging
                    if domain == "localhost" && cfg!(debug_assertions) {
                        return Ok(next.run(req).await);
                    }
                    return Err(StatusCode::NOT_FOUND);
                }
            };

            // Find AppInstance
            let app_instance = app_instance::Entity::find_by_id(app_domain.app_instance_id)
                .one(&db)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .ok_or(StatusCode::NOT_FOUND)?;

            // Find Tenant
            let tenant = tenant::Entity::find_by_id(app_instance.tenant_id)
                .one(&db)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .ok_or(StatusCode::NOT_FOUND)?;
            
            let config = SiteConfig {
                tenant_id: tenant.id,
                name: tenant.name.clone(),
                domain: app_domain.domain_name,
                subdomain: None,
                custom_domain: None,
                enabled_modules: ModuleFlags::all(),
                theme: None,
                site_status: Some(tenant.site_status),
                custom_settings: HashMap::new(),
            };
            
            // Update cache
            {
                let mut cache = SITE_CACHE.write().await;
                cache.insert(domain.clone(), config.clone());
            }
            
            config
        }
    };
    
    // Add site config to request extensions
    req.extensions_mut().insert(site_config.clone());
    
    // Inject X-Tenant-Id header for downstream apps like Anchor
    if let Ok(header_val) = axum::http::HeaderValue::from_str(&site_config.tenant_id.to_string()) {
        req.headers_mut().insert("X-Tenant-Id", header_val);
    }
    
    Ok(next.run(req).await)
}

pub async fn clear_site_cache() {
    let mut cache = SITE_CACHE.write().await;
    cache.clear();
}
