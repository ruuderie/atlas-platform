//! Platform-generic `AppDeploymentConfig` Axum extractor.
//!
//! Resolves the current tenant's deployment configuration for a given app.
//! Any app on the Atlas platform can use this to implement mode-based behavior
//! without touching the platform's core code.
//!
//! # Design
//!
//! This extractor is *not* parameterized by app slug at the type level — instead,
//! callers pass the app slug at construction time (via the `for_app` method) or
//! the handler extracts it directly by reading the `X-Atlas-App-Slug` from the
//! route context. The simplest pattern is calling `AppDeploymentConfig::resolve()`
//! inside a handler after extracting `TenantContext`.
//!
//! # Usage in a handler (recommended)
//!
//! ```rust,ignore
//! use crate::extractors::app_config::AppDeploymentConfig;
//! use crate::entities::atlas_app_deployment_config::FolioMode;
//!
//! async fn my_pm_handler(
//!     ctx: TenantContext,
//!     cfg: AppDeploymentConfig,
//!     Extension(db): Extension<DatabaseConnection>,
//! ) -> impl IntoResponse {
//!     if cfg.folio_mode != FolioMode::Pmc {
//!         return StatusCode::FORBIDDEN.into_response();
//!     }
//!     // ... PMC logic
//! }
//! ```
//!
//! # Caching
//!
//! Result is inserted into request extensions. If `TenantContext` already ran
//! (it always does for authenticated routes), the tenant_id is reused —
//! only one DB query runs per request regardless of how many handlers
//! extract `AppDeploymentConfig`.
//!
//! # Fallback
//!
//! If no row exists for (tenant_id, app_slug), returns `mode = "standard"`,
//! `folio_mode = "standard"`, and `config = {}`. Fully backward-compatible.

use axum::{
    Extension,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde_json::Value;
use uuid::Uuid;

use crate::entities::atlas_app_deployment_config::{self, AppDeploymentMode, FolioMode};
use crate::extractors::tenant::TenantContext;

// Cache key — one per app slug per request
#[derive(Clone, Debug)]
struct CachedAppConfig {
    app_slug: String,
    mode: AppDeploymentMode,
    folio_mode: FolioMode,
    config: Value,
}

/// Resolved deployment configuration for the current tenant + app.
#[derive(Clone, Debug)]
pub struct AppDeploymentConfig {
    pub tenant_id: Uuid,
    pub app_slug: String,
    /// Platform-level deployment topology.
    pub mode: AppDeploymentMode,
    /// Folio operational identity (standard | pmc | brokerage).
    /// Only meaningful when `app_slug = "property_management"`.
    /// Always `Standard` for other apps.
    pub folio_mode: FolioMode,
    /// Arbitrary JSON config for this deployment. `{}` if not configured.
    pub config: Value,
}

impl AppDeploymentConfig {
    /// Reads a string value from the config JSON.
    pub fn config_str(&self, key: &str) -> Option<&str> {
        self.config.get(key)?.as_str()
    }

    /// Reads a u64 value from the config JSON.
    pub fn config_u64(&self, key: &str) -> Option<u64> {
        self.config.get(key)?.as_u64()
    }

    /// Returns `tenant_portal_enabled` config flag (default: false).
    pub fn tenant_portal_enabled(&self) -> bool {
        self.config
            .get("tenant_portal_enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    /// Returns `vendor_portal_enabled` config flag (default: false).
    pub fn vendor_portal_enabled(&self) -> bool {
        self.config
            .get("vendor_portal_enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }
}

/// Extracts `AppDeploymentConfig` for the `"folio"` app slug (default).
///
/// For other apps, implement a newtype wrapper or call the internal resolver
/// directly. The extractor defaulting to "folio" is a convenience for
/// the most common case on this platform right now.
impl<S> FromRequestParts<S> for AppDeploymentConfig
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, StatusCode> {
        // Re-use cached result if the same request already resolved it.
        if let Some(cached) = parts.extensions.get::<CachedAppConfig>().cloned() {
            let ctx = TenantContext::from_request_parts(parts, state).await?;
            return Ok(AppDeploymentConfig {
                tenant_id: ctx.tenant_id,
                app_slug: cached.app_slug,
                mode: cached.mode,
                folio_mode: cached.folio_mode,
                config: cached.config,
            });
        }

        let ctx = TenantContext::from_request_parts(parts, state).await?;

        let db = Extension::<DatabaseConnection>::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .0;

        let app_slug = parts
            .extensions
            .get::<&'static str>()
            .copied()
            .unwrap_or("folio")
            .to_string();

        let row = atlas_app_deployment_config::Entity::find()
            .filter(atlas_app_deployment_config::Column::TenantId.eq(ctx.tenant_id))
            .filter(atlas_app_deployment_config::Column::AppSlug.eq(&app_slug))
            .one(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let (mode, folio_mode, config) = match row {
            Some(r) => (r.mode, r.folio_mode, r.config),
            // Missing row = standard mode, standard folio_mode, empty config (backward compatible)
            None => (
                AppDeploymentMode::Standard,
                FolioMode::Standard,
                Value::Object(Default::default()),
            ),
        };

        let cached = CachedAppConfig {
            app_slug: app_slug.clone(),
            mode: mode.clone(),
            folio_mode: folio_mode.clone(),
            config: config.clone(),
        };
        parts.extensions.insert(cached);

        Ok(AppDeploymentConfig {
            tenant_id: ctx.tenant_id,
            app_slug,
            mode,
            folio_mode,
            config,
        })
    }
}
