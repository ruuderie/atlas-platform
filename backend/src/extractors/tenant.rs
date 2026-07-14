//! Platform-generic `TenantContext` Axum extractor.
//!
//! Resolves the current user's tenant in a single DB round-trip and
//! makes `tenant_id`, `user_account`, and `user_account.role` available
//! to any handler via extraction — no more per-handler `resolve_tenant_id()`.
//!
//! # Platform-generic design
//!
//! `TenantContext` is app-agnostic. It does not know about Folio roles,
//! app slugs, or any app-specific concept. It answers one question:
//! "For the currently authenticated user, what is their tenant_id and
//! what is their platform-level UserRole (Owner/Admin/Member/PSA)?"
//!
//! All apps (Folio, future CRM, HR, etc.) use this extractor as the
//! foundation for their own role-gating logic.
//!
//! # Usage
//!
//! ```rust,ignore
//! use crate::extractors::tenant::TenantContext;
//!
//! async fn my_handler(
//!     ctx: TenantContext,
//!     Extension(db): Extension<DatabaseConnection>,
//! ) -> impl IntoResponse {
//!     // ctx.tenant_id  — the user's tenant UUID
//!     // ctx.user_role  — platform UserRole (Owner/Admin/Member/PSA)
//!     // ctx.account_id — the user's account UUID
//! }
//! ```
//!
//! # Performance
//!
//! One query joins `user_account` + `account` (tenant_id lives on account).
//! Profile is optional — onboarding / invite paths may create account +
//! user_account before a profile row exists. The result is extracted once
//! per request by Axum; all downstream extractors that also call
//! `TenantContext::from_request_parts` share the same resolution path
//! (Axum deduplicates via `Parts` extension insertion).

use axum::{
    Extension,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, DbBackend, EntityTrait, QueryFilter,
    Statement,
};
use uuid::Uuid;

use crate::entities::user_account::UserRole;
use crate::entities::{user, user_account};

/// Resolved tenant context for the current request.
///
/// Extracted from the authenticated user's `user_account` → `account` chain
/// (profile is preferred when present, but not required).
/// Available in handlers as `ctx: TenantContext`.
#[derive(Clone, Debug)]
pub struct TenantContext {
    /// The tenant this user belongs to.
    pub tenant_id: Uuid,
    /// The user's platform-level role (Owner/Admin/Member/PlatformSuperAdmin).
    /// This is the coarse-grained platform role, NOT an app-specific role.
    pub user_role: UserRole,
    /// The account this user_account record belongs to.
    pub account_id: Uuid,
    /// The authenticated user's ID (convenience — same as `user::Model.id`).
    pub user_id: Uuid,
}

impl TenantContext {
    /// Returns true if this user has a platform-level administrative role
    /// (Owner, Admin, or PlatformSuperAdmin). These roles bypass app-level
    /// permission checks by convention.
    pub fn is_platform_admin(&self) -> bool {
        matches!(
            self.user_role,
            UserRole::Owner | UserRole::Admin | UserRole::PlatformSuperAdmin
        )
    }
}

/// Resolve a real (non-platform) tenant_id for `user_id` via
/// `user_account` → `account.tenant_id`.
///
/// Prefer this over profile lookups: profile rows are not always created
/// during invite / partial onboarding, while account.tenant_id is.
pub async fn resolve_tenant_id(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<Uuid, StatusCode> {
    db.query_one(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"SELECT a.tenant_id
           FROM user_account ua
           JOIN account a ON ua.account_id = a.id
           WHERE ua.user_id = $1
             AND ua.is_active = true
             AND a.tenant_id <> $2
           ORDER BY ua.created_at ASC
           LIMIT 1"#,
        [user_id.into(), Uuid::nil().into()],
    ))
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .and_then(|r| r.try_get::<Uuid>("", "tenant_id").ok())
    .ok_or(StatusCode::FORBIDDEN)
}

impl<S> FromRequestParts<S> for TenantContext
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, StatusCode> {
        // If already resolved this request, return cached value
        if let Some(ctx) = parts.extensions.get::<TenantContext>() {
            return Ok(ctx.clone());
        }

        let Extension(db) = Extension::<DatabaseConnection>::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let Extension(current_user) = Extension::<user::Model>::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        let ua = user_account::Entity::find()
            .filter(user_account::Column::UserId.eq(current_user.id))
            .filter(user_account::Column::IsActive.eq(true))
            .one(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::FORBIDDEN)?;

        // Prefer profile.tenant_id when present; otherwise account.tenant_id
        // (matches GET /api/folio/me and onboarding submit).
        let tenant_id = match crate::entities::profile::Entity::find()
            .filter(crate::entities::profile::Column::AccountId.eq(ua.account_id))
            .one(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        {
            Some(profile) if profile.tenant_id != Uuid::nil() => profile.tenant_id,
            _ => {
                let account = crate::entities::account::Entity::find_by_id(ua.account_id)
                    .one(&db)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                    .ok_or(StatusCode::FORBIDDEN)?;
                if account.tenant_id == Uuid::nil() {
                    return Err(StatusCode::FORBIDDEN);
                }
                account.tenant_id
            }
        };

        let ctx = TenantContext {
            tenant_id,
            user_role: ua.role,
            account_id: ua.account_id,
            user_id: current_user.id,
        };

        // Cache in request extensions so subsequent extractors reuse it
        parts.extensions.insert(ctx.clone());

        Ok(ctx)
    }
}
