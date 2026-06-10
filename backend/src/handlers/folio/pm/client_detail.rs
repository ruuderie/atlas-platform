//! GET /api/folio/pm/clients/:account_id — single client detail
//!
//! Returns the full portfolio snapshot for one client account.
//! The PM must pass `X-Folio-Client-Account` header matching the path param.

use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter};
use serde::Serialize;
use uuid::Uuid;

use crate::extractors::client_context::ClientContext;
use crate::extractors::folio_role::PropertyManagerOnly;
use crate::extractors::tenant::TenantContext;

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/pm/clients/:account_id", get(get_client_detail))
}

#[derive(Serialize)]
pub struct ClientDetail {
    pub account_id:    Uuid,
    pub display_name:  String,
    /// Portfolio rows where managed_account_id = account_id
    pub portfolio_ids: Vec<Uuid>,
    /// Active lease count
    pub active_leases: i64,
}

/// Return a client's portfolio snapshot.
///
/// `ClientContext` validates that the account belongs to the PM's tenant
/// and is the same UUID as in the path.
async fn get_client_detail(
    _guard: PropertyManagerOnly,
    client: ClientContext,
    ctx: TenantContext,
    Path(account_id): Path<Uuid>,
    Extension(db): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    // Ensure path param matches the resolved client (defense-in-depth)
    if client.account_id != account_id {
        return StatusCode::BAD_REQUEST.into_response();
    }

    // Fetch portfolios scoped to this client
    let portfolios = match crate::entities::atlas_portfolio::Entity::find()
        .filter(crate::entities::atlas_portfolio::Column::TenantId.eq(ctx.tenant_id))
        .filter(crate::entities::atlas_portfolio::Column::ManagedAccountId.eq(account_id))
        .all(&db)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!(error = %e, "pm/client_detail: DB error");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Active lease count — scoped to this client's managed_account_id
    let active_leases = match crate::entities::atlas_contract::Entity::find()
        .filter(crate::entities::atlas_contract::Column::TenantId.eq(ctx.tenant_id))
        .filter(crate::entities::atlas_contract::Column::ManagedAccountId.eq(account_id))
        .filter(crate::entities::atlas_contract::Column::Status.eq("active"))
        .count(&db)
        .await
    {
        Ok(n) => n as i64,
        Err(_) => 0,
    };

    Json(ClientDetail {
        account_id:    client.account_id,
        display_name:  client.display_name,
        portfolio_ids: portfolios.iter().map(|p| p.id).collect(),
        active_leases,
    })
    .into_response()
}
