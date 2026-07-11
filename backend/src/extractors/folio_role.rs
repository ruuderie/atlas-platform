//! Axum extractors for declarative Folio role enforcement.
//!
//! All extractors compose on top of `TenantContext` so tenant resolution
//! is performed exactly once per request regardless of how many extractors
//! a handler declares.
//!
//! # Available extractors
//!
//! | Extractor | Allowed roles | Carries |
//! |---|---|---|
//! | `RequireFolioRole` | any assigned Folio role | `FolioRole` |
//! | `VendorOnly` | Vendor | unit |
//! | `LandlordOnly` | Landlord | unit |
//! | `TenantOnly` | Tenant | unit |
//! | `LandlordOrVendor` | Landlord, Vendor | `FolioRole` |
//!
//! # Usage
//!
//! ```rust,ignore
//! use crate::extractors::folio_role::VendorOnly;
//!
//! async fn vendor_handler(
//!     _: VendorOnly,
//!     Extension(db): Extension<DatabaseConnection>,
//!     Extension(current_user): Extension<user::Model>,
//! ) -> impl IntoResponse { /* ... */ }
//! ```

use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};

use crate::extractors::tenant::TenantContext;
use crate::services::rbac::RbacService;
use crate::types::pm::FolioRole;

// ── Core extractor ────────────────────────────────────────────────────────────

/// Resolves the current user's FolioRole via G-32 RBAC.
/// Returns 403 if the user has no role assigned in the Folio app.
pub struct RequireFolioRole(pub FolioRole);

impl<S> FromRequestParts<S> for RequireFolioRole
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, StatusCode> {
        let ctx = TenantContext::from_request_parts(parts, state).await?;

        // Platform admins bypass app-level role assignment — treat as Landlord
        // (full access) rather than blocking them with 403.
        if ctx.is_platform_admin() {
            return Ok(RequireFolioRole(FolioRole::Landlord));
        }

        // Extract db from parts for the G-32 role lookup
        let db = parts
            .extensions
            .get::<sea_orm::DatabaseConnection>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
            .clone();

        let role_slug = RbacService::get_user_app_role(&db, ctx.user_id, ctx.tenant_id, "folio")
            .await
            .ok_or_else(|| {
                tracing::warn!(
                    user_id = %ctx.user_id, tenant_id = %ctx.tenant_id,
                    "RequireFolioRole: no folio role assigned"
                );
                StatusCode::FORBIDDEN
            })?;

        let role = FolioRole::try_from(role_slug.as_str()).map_err(|e| {
            tracing::warn!(
                user_id = %ctx.user_id, slug = %role_slug,
                "RequireFolioRole: unrecognised role slug: {e}"
            );
            StatusCode::FORBIDDEN
        })?;

        Ok(RequireFolioRole(role))
    }
}

// ── Role-specific guards ──────────────────────────────────────────────────────

/// Rejects any user whose Folio role is not `Vendor`. Returns 403.
pub struct VendorOnly;

impl<S> FromRequestParts<S> for VendorOnly
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, StatusCode> {
        let RequireFolioRole(role) = RequireFolioRole::from_request_parts(parts, state).await?;
        match role {
            FolioRole::Vendor => Ok(VendorOnly),
            _ => {
                tracing::warn!("VendorOnly: access denied (role={role})");
                Err(StatusCode::FORBIDDEN)
            }
        }
    }
}

/// Rejects any user whose Folio role is not `Landlord`. Returns 403.
pub struct LandlordOnly;

impl<S> FromRequestParts<S> for LandlordOnly
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, StatusCode> {
        let RequireFolioRole(role) = RequireFolioRole::from_request_parts(parts, state).await?;
        match role {
            FolioRole::Landlord => Ok(LandlordOnly),
            _ => {
                tracing::warn!("LandlordOnly: access denied (role={role})");
                Err(StatusCode::FORBIDDEN)
            }
        }
    }
}

/// Rejects any user whose Folio role is not `Tenant`. Returns 403.
pub struct FolioTenantOnly;

impl<S> FromRequestParts<S> for FolioTenantOnly
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, StatusCode> {
        let RequireFolioRole(role) = RequireFolioRole::from_request_parts(parts, state).await?;
        match role {
            FolioRole::Tenant => Ok(FolioTenantOnly),
            _ => {
                tracing::warn!("FolioTenantOnly: access denied (role={role})");
                Err(StatusCode::FORBIDDEN)
            }
        }
    }
}

/// Allows Landlord or Vendor; rejects Tenant. Carries the resolved role.
pub struct LandlordOrVendor(pub FolioRole);

impl<S> FromRequestParts<S> for LandlordOrVendor
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, StatusCode> {
        let RequireFolioRole(role) = RequireFolioRole::from_request_parts(parts, state).await?;
        match &role {
            FolioRole::Landlord | FolioRole::Vendor => Ok(LandlordOrVendor(role)),
            _ => {
                tracing::warn!("LandlordOrVendor: access denied (role={role})");
                Err(StatusCode::FORBIDDEN)
            }
        }
    }
}

/// Rejects any user whose Folio role is not `PropertyManager`. Returns 403.
///
/// Also validates that the tenant's Folio deployment config is in `pmc` mode —
/// a `PropertyManager` role assignment alone is not sufficient; the instance must
/// be configured as a PMC (`folio_mode = "pmc"` in `atlas_app_deployment_config`).
///
/// This replaces the legacy `pmc_enabled: true` JSON check with the typed
/// `folio_mode` column added by migration `m20260909_folio_instance_mode`.
pub struct PropertyManagerOnly;

impl<S> FromRequestParts<S> for PropertyManagerOnly
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, StatusCode> {
        use crate::entities::atlas_app_deployment_config::FolioMode;
        use crate::extractors::app_config::AppDeploymentConfig;

        let RequireFolioRole(role) = RequireFolioRole::from_request_parts(parts, state).await?;
        if role != FolioRole::PropertyManager {
            tracing::warn!("PropertyManagerOnly: access denied (role={role})");
            return Err(StatusCode::FORBIDDEN);
        }

        // Double-check: the instance must be in PMC mode.
        // Prevents a stale role assignment from granting PMC access on a standard deployment.
        let cfg = AppDeploymentConfig::from_request_parts(parts, state).await?;
        if cfg.folio_mode != FolioMode::Pmc {
            tracing::warn!(
                tenant_id = %cfg.tenant_id,
                folio_mode = %cfg.folio_mode,
                "PropertyManagerOnly: PMC role assigned but instance folio_mode != 'pmc'"
            );
            return Err(StatusCode::FORBIDDEN);
        }

        Ok(PropertyManagerOnly)
    }
}

/// Rejects any user whose Folio role is not `Broker` or `Agent`. Returns 403.
///
/// Also validates that the tenant's Folio deployment config is in `brokerage` mode —
/// both the role assignment and the instance mode must be correct.
///
/// Mirrors `PropertyManagerOnly` — same dual-gate pattern.
pub struct BrokerageOnly(pub FolioRole);

impl<S> FromRequestParts<S> for BrokerageOnly
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, StatusCode> {
        use crate::entities::atlas_app_deployment_config::FolioMode;
        use crate::extractors::app_config::AppDeploymentConfig;

        let RequireFolioRole(role) = RequireFolioRole::from_request_parts(parts, state).await?;
        match &role {
            FolioRole::Broker | FolioRole::Agent => {}
            _ => {
                tracing::warn!("BrokerageOnly: access denied (role={role})");
                return Err(StatusCode::FORBIDDEN);
            }
        }

        let cfg = AppDeploymentConfig::from_request_parts(parts, state).await?;
        if cfg.folio_mode != FolioMode::Brokerage {
            tracing::warn!(
                tenant_id = %cfg.tenant_id,
                folio_mode = %cfg.folio_mode,
                "BrokerageOnly: brokerage role assigned but instance folio_mode != 'brokerage'"
            );
            return Err(StatusCode::FORBIDDEN);
        }

        Ok(BrokerageOnly(role))
    }
}
