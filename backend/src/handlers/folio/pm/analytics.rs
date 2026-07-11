//! GET /api/folio/pm/analytics — cross-client aggregate analytics
//!
//! Returns KPI metrics across ALL client accounts in this PMC tenant.
//! No `ClientContext` here — this is the aggregate, unfiltered by client.

use axum::{Extension, Json, Router, http::StatusCode, response::IntoResponse, routing::get};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter};
use serde::Serialize;
use uuid::Uuid;

use crate::extractors::folio_role::PropertyManagerOnly;
use crate::extractors::tenant::TenantContext;

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new().route("/api/folio/pm/analytics", get(get_analytics))
}

#[derive(Serialize)]
pub struct PmAnalytics {
    pub tenant_id: Uuid,
    /// Total active leases across all client books
    pub total_active_leases: i64,
    /// Total portfolios managed
    pub total_portfolios: i64,
    /// Per-client metrics (one entry per client account)
    pub clients: Vec<ClientMetric>,
}

#[derive(Serialize)]
pub struct ClientMetric {
    pub account_id: Uuid,
    pub active_leases: i64,
    pub portfolio_count: i64,
}

/// Cross-client aggregate analytics for the PMC dashboard.
///
/// Production implementation: replace count queries with a materialized view
/// or a batch aggregation query. Stubbed here for scaffolding completeness.
async fn get_analytics(
    _guard: PropertyManagerOnly,
    ctx: TenantContext,
    Extension(db): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    // Aggregate totals: all leases in this tenant with a managed_account_id set
    // (PMC-mode rows). Leases without managed_account_id are treated as the PM's own.
    let total_active_leases = match crate::entities::atlas_contract::Entity::find()
        .filter(crate::entities::atlas_contract::Column::TenantId.eq(ctx.tenant_id))
        .filter(crate::entities::atlas_contract::Column::Status.eq("active"))
        .count(&db)
        .await
    {
        Ok(n) => n as i64,
        Err(e) => {
            tracing::error!(error = %e, "pm/analytics: error counting leases");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let total_portfolios = match crate::entities::atlas_portfolio::Entity::find()
        .filter(crate::entities::atlas_portfolio::Column::TenantId.eq(ctx.tenant_id))
        .count(&db)
        .await
    {
        Ok(n) => n as i64,
        Err(e) => {
            tracing::error!(error = %e, "pm/analytics: error counting portfolios");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Per-client breakdown — stubbed; production should use a single GROUP BY query
    let clients: Vec<ClientMetric> = vec![]; // TODO: GROUP BY managed_account_id

    Json(PmAnalytics {
        tenant_id: ctx.tenant_id,
        total_active_leases,
        total_portfolios,
        clients,
    })
    .into_response()
}
