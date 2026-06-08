//! Folio — Portfolio handler.
//!
//! # Routes
//!
//! ```ignore
//! GET  /api/folio/portfolios
//!      Returns all portfolios for the authenticated tenant.
//!      -> 200 [PortfolioSummary]
//!
//! POST /api/folio/portfolios
//!      Create a new portfolio.
//!      Body: { "name": string, "description"?: string }
//!      -> 201 { "id": uuid }
//!
//! GET  /api/folio/portfolios/:id/nav
//!      Compute NAV aggregate for a portfolio (USD/BRL/BTC breakdowns).
//!      -> 200 PortfolioNav
//! ```

use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::user;

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/portfolios", get(list_portfolios).post(create_portfolio))
        .route("/api/folio/portfolios/{id}/nav", get(get_portfolio_nav))
}

// ── Request / response types ──────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct PortfolioSummary {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub asset_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct CreatePortfolioInput {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreatePortfolioResponse {
    pub id: Uuid,
}

#[derive(Debug, Serialize)]
struct PortfolioNav {
    pub portfolio_id: Uuid,
    /// Net asset value in USD cents.
    pub nav_usd_cents: i64,
    /// Net asset value in BRL cents (None until Phase 3 ledger aggregation).
    pub nav_brl_cents: Option<i64>,
    /// Net asset value in BTC satoshis (None until Phase 3 ledger aggregation).
    pub nav_btc_satoshis: Option<i64>,
    /// Total units in the portfolio.
    pub unit_count: i32,
    /// Occupied units.
    pub occupied_count: i32,
    /// Vacant units.
    pub vacant_count: i32,
    /// Occupancy rate 0.0–1.0.
    pub occupancy_rate: f64,
    pub calculated_at: chrono::DateTime<chrono::Utc>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/folio/portfolios
async fn list_portfolios(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let portfolios = crate::entities::atlas_portfolio::Entity::find()
        .filter(crate::entities::atlas_portfolio::Column::TenantId.eq(tenant_id))
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "list_portfolios error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let summaries: Vec<PortfolioSummary> = portfolios
        .into_iter()
        .map(|p| PortfolioSummary {
            id: p.id,
            tenant_id: p.tenant_id,
            name: p.name,
            description: p.description,
            asset_count: 0, // Phase 2: join with atlas_assets count
            created_at: p.created_at,
        })
        .collect();

    Ok(axum::response::Json(summaries))
}

/// POST /api/folio/portfolios
async fn create_portfolio(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<CreatePortfolioInput>,
) -> Result<impl IntoResponse, StatusCode> {
    use sea_orm::{Set, ActiveModelTrait};
    use chrono::Utc;

    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let id = Uuid::new_v4();
    let now = Utc::now();

    let model = crate::entities::atlas_portfolio::ActiveModel {
        id: Set(id),
        tenant_id: Set(tenant_id),
        owner_user_id: Set(current_user.id),
        portfolio_type: Set("real_estate".to_string()),
        name: Set(input.name),
        description: Set(input.description),
        metadata: Set(None),
        created_at: Set(now),
    };
    model.insert(&db).await.map_err(|e| {
        tracing::error!(%tenant_id, "create_portfolio insert error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!(portfolio_id = %id, %tenant_id, "create_portfolio: created");
    Ok((StatusCode::CREATED, axum::response::Json(CreatePortfolioResponse { id })))
}

/// GET /api/folio/portfolios/:id/nav
async fn get_portfolio_nav(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(portfolio_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let nav = crate::services::pm::portfolio::PortfolioService::compute_nav(
        &db,
        tenant_id,
        portfolio_id,
    )
    .await
    .map_err(|e| {
        tracing::error!(%tenant_id, %portfolio_id, "get_portfolio_nav error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(axum::response::Json(PortfolioNav {
        portfolio_id,
        nav_usd_cents: nav.nav_usd_cents,
        nav_brl_cents: nav.nav_brl_cents,
        nav_btc_satoshis: nav.nav_btc_satoshis,
        unit_count: nav.unit_count,
        occupied_count: nav.occupied_count,
        vacant_count: nav.vacant_count,
        occupancy_rate: nav.occupancy_rate,
        calculated_at: nav.calculated_at,
    }))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn resolve_tenant_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Uuid, StatusCode> {
    use sea_orm::QueryFilter;

    let user_accounts = crate::entities::user_account::Entity::find()
        .filter(crate::entities::user_account::Column::UserId.eq(user_id))
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
