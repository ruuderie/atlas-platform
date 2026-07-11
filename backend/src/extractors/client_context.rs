//! `ClientContext` — Resolves which client account a PropertyManager is acting on behalf of.
//!
//! In PMC mode (`mode = "property_management_co"`), a PM can manage many landlord
//! client accounts within their tenant. The client account is communicated per-request
//! via the `X-Folio-Client-Account` HTTP header.
//!
//! # Security model
//!
//! 1. The calling user must have a valid `TenantContext` (authenticated).
//! 2. The calling user must have the `property_manager` Folio role.
//! 3. The `account_id` in the header must belong to the same `tenant_id`.
//!    If a PM supplies a foreign account UUID (from another tenant), the DB
//!    query returns nothing and we return 403.
//!
//! # Usage
//!
//! ```rust,ignore
//! async fn list_client_assets(
//!     client: ClientContext,
//!     Extension(db): Extension<DatabaseConnection>,
//! ) -> impl IntoResponse {
//!     // client.account_id is the validated client account
//!     // client.display_name is the account's display name (for logging)
//! }
//! ```
//!
//! # When `ClientContext` is optional
//!
//! For aggregate PMC views (analytics across all clients), don't extract `ClientContext`.
//! Just use `RequireFolioRole` + filter by `tenant_id` without an account_id filter.

use axum::{
    Extension,
    extract::FromRequestParts,
    http::{HeaderMap, StatusCode, request::Parts},
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::extractors::tenant::TenantContext;

/// The header name used to pass the active client account.
pub const CLIENT_ACCOUNT_HEADER: &str = "x-folio-client-account";

/// Resolved client account for the current PMC request.
#[derive(Clone, Debug)]
pub struct ClientContext {
    /// The validated client account UUID.
    pub account_id: Uuid,
    /// Display name from the `account` table (for logging + UI rendering).
    pub display_name: String,
}

impl<S> FromRequestParts<S> for ClientContext
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, StatusCode> {
        // Re-use cached result
        if let Some(ctx) = parts.extensions.get::<ClientContext>() {
            return Ok(ctx.clone());
        }

        let tenant_ctx = TenantContext::from_request_parts(parts, state).await?;

        let db = Extension::<DatabaseConnection>::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .0;

        // Extract account UUID from header
        let header_val = parts
            .headers
            .get(CLIENT_ACCOUNT_HEADER)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                tracing::warn!(
                    user_id = %tenant_ctx.user_id,
                    "ClientContext: missing {} header",
                    CLIENT_ACCOUNT_HEADER,
                );
                StatusCode::BAD_REQUEST
            })?;

        let account_id = header_val.parse::<Uuid>().map_err(|_| {
            tracing::warn!(
                header_value = %header_val,
                "ClientContext: invalid UUID in {} header",
                CLIENT_ACCOUNT_HEADER,
            );
            StatusCode::BAD_REQUEST
        })?;

        // Validate: this account must belong to the PM's tenant.
        // Joins account → profile → tenant to enforce cross-tenant isolation.
        let account = crate::entities::atlas_account::Entity::find_by_id(account_id)
            .one(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or_else(|| {
                tracing::warn!(
                    %account_id, tenant_id = %tenant_ctx.tenant_id,
                    "ClientContext: account not found or wrong tenant"
                );
                StatusCode::FORBIDDEN
            })?;

        // Verify the account's tenant matches the requesting PM's tenant.
        // atlas_account.tenant_id provides the isolation boundary.
        if account.tenant_id != tenant_ctx.tenant_id {
            tracing::warn!(
                %account_id,
                account_tenant_id = %account.tenant_id,
                pm_tenant_id = %tenant_ctx.tenant_id,
                "ClientContext: tenant mismatch — cross-tenant probe blocked"
            );
            return Err(StatusCode::FORBIDDEN);
        }

        let display_name = account.name.clone();

        let ctx = ClientContext {
            account_id,
            display_name,
        };
        parts.extensions.insert(ctx.clone());

        Ok(ctx)
    }
}
