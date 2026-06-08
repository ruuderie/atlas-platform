//! Axum extractors for declarative Folio role enforcement.
//!
//! # Problem solved
//!
//! Imperative role checks (`ensure_vendor_role(...)`) are easy to forget —
//! one new route without the call is silently unprotected.
//!
//! `VendorOnly`, `LandlordOnly`, `TenantOnly`, `LandlordOrVendor`, and
//! `RequireFolioRole` are Axum extractors that run the role check *before*
//! the handler body executes. If the check fails, Axum returns 403 before
//! the handler is ever called.
//!
//! # Usage
//!
//! ```rust,no_run
//! use crate::extractors::folio_role::VendorOnly;
//!
//! async fn my_handler(
//!     _guard: VendorOnly,                              // ← role gate
//!     Extension(db): Extension<DatabaseConnection>,
//!     Extension(current_user): Extension<user::Model>,
//! ) -> impl IntoResponse { /* ... */ }
//! ```
//!
//! For multi-role handlers use `RequireFolioRole` and match on the value:
//!
//! ```rust,no_run
//! use crate::extractors::folio_role::RequireFolioRole;
//!
//! async fn shared_handler(
//!     RequireFolioRole(role): RequireFolioRole,
//! ) -> impl IntoResponse {
//!     match role {
//!         FolioRole::Landlord => { /* ... */ }
//!         FolioRole::Vendor   => { /* ... */ }
//!         FolioRole::Tenant   => unreachable!(),
//!     }
//! }
//! ```

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    Extension,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::entities::{user, user_account};
use crate::services::rbac::RbacService;
use crate::types::pm::FolioRole;

// ── Core extractor ────────────────────────────────────────────────────────────

/// Resolves and exposes the current user's FolioRole without enforcing any
/// specific role. Use role-specific guards (`VendorOnly` etc.) when you know
/// exactly which role is required.
pub struct RequireFolioRole(pub FolioRole);

impl<S> FromRequestParts<S> for RequireFolioRole
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, StatusCode> {
        let Extension(db) =
            Extension::<DatabaseConnection>::from_request_parts(parts, state)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let Extension(current_user) =
            Extension::<user::Model>::from_request_parts(parts, state)
                .await
                .map_err(|_| StatusCode::UNAUTHORIZED)?;

        let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

        let role_slug = RbacService::get_user_app_role(&db, current_user.id, tenant_id, "folio")
            .await
            .ok_or(StatusCode::FORBIDDEN)?;

        let role = FolioRole::try_from(role_slug.as_str()).map_err(|_| StatusCode::FORBIDDEN)?;

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
                tracing::warn!("VendorOnly extractor: access denied (role={role})");
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
                tracing::warn!("LandlordOnly extractor: access denied (role={role})");
                Err(StatusCode::FORBIDDEN)
            }
        }
    }
}

/// Rejects any user whose Folio role is not `Tenant`. Returns 403.
pub struct TenantOnly;

impl<S> FromRequestParts<S> for TenantOnly
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, StatusCode> {
        let RequireFolioRole(role) = RequireFolioRole::from_request_parts(parts, state).await?;
        match role {
            FolioRole::Tenant => Ok(TenantOnly),
            _ => {
                tracing::warn!("TenantOnly extractor: access denied (role={role})");
                Err(StatusCode::FORBIDDEN)
            }
        }
    }
}

/// Allows Landlord or Vendor; rejects Tenant. Carries the resolved role.
/// Useful for shared work-order views accessible to both roles.
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
                tracing::warn!("LandlordOrVendor extractor: access denied (role={role})");
                Err(StatusCode::FORBIDDEN)
            }
        }
    }
}

// ── Shared helper ─────────────────────────────────────────────────────────────

async fn resolve_tenant_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Uuid, StatusCode> {
    let user_accounts = user_account::Entity::find()
        .filter(user_account::Column::UserId.eq(user_id))
        .all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let account_ids: Vec<Uuid> = user_accounts.into_iter().map(|ua| ua.account_id).collect();

    let profile = crate::entities::profile::Entity::find()
        .filter(crate::entities::profile::Column::AccountId.is_in(account_ids))
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)?;

    Ok(profile.tenant_id)
}
