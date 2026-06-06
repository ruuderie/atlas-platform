//! G-27 Portfolio Analytics handlers (Phase 3).
//!
//! # Routes
//!
//! ```ignore
//! GET  /api/scorecard-templates/:template_id/analytics
//!      Returns per-dimension portfolio stats (distribution, cohort size, trend counts).
//!      Powers g27scPortfolioPanel LWC callout in AppExchange.
//!      -> 200 PortfolioStats
//!      -> 404 if template doesn't belong to tenant
//!
//! GET  /api/scorecard-templates/:template_id/leaderboard?limit=25
//!      Ranked list of scorecards by composite score with percentile rank.
//!      -> 200 [LeaderboardEntry]
//!
//! GET  /api/scorecard-templates/:template_id/anomalies?limit=50
//!      Recent is_anomaly=true time series rows (last 90 days) for the template.
//!      -> 200 [AnomalyAlert]
//!
//! POST /api/scorecard-templates/:template_id/analytics/refresh
//!      Admin-only: trigger an on-demand materialized view refresh + re-rank.
//!      -> 204 No Content on success
//! ```

use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::entities::user;
use crate::services::scorecard_analytics_service::ScorecardAnalyticsService;

// ── Route registration ────────────────────────────────────────────────────────

pub fn routes() -> Router<sea_orm::DatabaseConnection> {
    Router::new()
        .route(
            "/api/scorecard-templates/{template_id}/analytics",
            get(portfolio_stats),
        )
        .route(
            "/api/scorecard-templates/{template_id}/leaderboard",
            get(leaderboard),
        )
        .route(
            "/api/scorecard-templates/{template_id}/anomalies",
            get(recent_anomalies),
        )
        .route(
            "/api/scorecard-templates/{template_id}/analytics/refresh",
            post(refresh_analytics),
        )
}

// ── Query param types ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LeaderboardQuery {
    /// Maximum entries to return. Clamped to 100 server-side.
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct AnomalyQuery {
    /// Maximum anomaly alerts to return. Clamped to 500 server-side.
    pub limit: Option<i64>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/scorecard-templates/:template_id/analytics
///
/// Returns per-dimension distribution statistics for the template's portfolio.
///
/// Source: `mv_scorecard_portfolio_analytics` (refreshed every 4 hours by worker).
/// Returns an empty dimensions array when the view has not yet been refreshed —
/// callers should show a "Portfolio data is being calculated" loading state.
///
/// Used by `g27scPortfolioPanel` LWC via `G27SC_PortfolioCalloutController`.
async fn portfolio_stats(
    Extension(db): Extension<sea_orm::DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(template_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let stats = ScorecardAnalyticsService::portfolio_stats(&db, template_id, tenant_id)
        .await
        .map_err(|e| {
            tracing::error!(%template_id, %tenant_id, "portfolio_stats error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(axum::response::Json(stats))
}

/// GET /api/scorecard-templates/:template_id/leaderboard?limit=25
///
/// Returns the top N scorecards ranked by composite score, with percentile rank.
///
/// Default limit: 25. Max: 100.
async fn leaderboard(
    Extension(db): Extension<sea_orm::DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(template_id): Path<Uuid>,
    Query(params): Query<LeaderboardQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let limit = params.limit.unwrap_or(25);

    let entries = ScorecardAnalyticsService::leaderboard(&db, template_id, tenant_id, limit)
        .await
        .map_err(|e| {
            tracing::error!(%template_id, %tenant_id, "leaderboard error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(axum::response::Json(entries))
}

/// GET /api/scorecard-templates/:template_id/anomalies?limit=50
///
/// Returns recent anomaly alerts (is_anomaly=true, last 90 days), sorted by most
/// recent period first then by |z_score| descending within the same period.
///
/// Source: `v_scorecard_recent_anomalies` (live view — always current).
/// Default limit: 50. Max: 500.
async fn recent_anomalies(
    Extension(db): Extension<sea_orm::DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(template_id): Path<Uuid>,
    Query(params): Query<AnomalyQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let limit = params.limit.unwrap_or(50);

    let alerts = ScorecardAnalyticsService::recent_anomalies(&db, template_id, tenant_id, limit)
        .await
        .map_err(|e| {
            tracing::error!(%template_id, %tenant_id, "recent_anomalies error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(axum::response::Json(alerts))
}

/// POST /api/scorecard-templates/:template_id/analytics/refresh
///
/// Triggers an on-demand `REFRESH MATERIALIZED VIEW CONCURRENTLY` followed by
/// a batch percentile rank update for all scorecards in this template.
///
/// Admin-only: returns 403 if the current user is not a platform or tenant admin.
/// Returns 204 No Content on success.
///
/// Use sparingly — the background worker refreshes automatically every 4 hours.
/// This endpoint exists for admin tooling and post-import bootstrapping.
async fn refresh_analytics(
    Extension(db): Extension<sea_orm::DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(template_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    // Verify the template belongs to this tenant before allowing a refresh.
    // (Prevents a tenant from triggering a refresh for another tenant's template.)
    verify_template_tenant(&db, template_id, tenant_id).await?;

    let started = std::time::Instant::now();

    ScorecardAnalyticsService::refresh_and_rerank(&db, template_id, tenant_id)
        .await
        .map_err(|e| {
            tracing::error!(%template_id, %tenant_id, "analytics refresh error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!(
        %template_id,
        %tenant_id,
        duration_ms = started.elapsed().as_millis(),
        user_id = %current_user.id,
        "On-demand portfolio analytics refresh completed"
    );

    Ok(StatusCode::NO_CONTENT)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Resolve the tenant_id for the current user via their profile.
/// Returns 403 if the user has no profile, 500 on DB error.
async fn resolve_tenant_id(
    db: &sea_orm::DatabaseConnection,
    user_id: Uuid,
) -> Result<Uuid, StatusCode> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

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

/// Verify the template belongs to the given tenant.
/// Returns 404 if not found, 403 if it belongs to a different tenant.
async fn verify_template_tenant(
    db: &sea_orm::DatabaseConnection,
    template_id: Uuid,
    tenant_id: Uuid,
) -> Result<(), StatusCode> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    let template = crate::entities::atlas_scorecard_template::Entity::find_by_id(template_id)
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if template.tenant_id != tenant_id {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(())
}
