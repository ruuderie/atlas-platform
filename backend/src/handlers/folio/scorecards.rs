//! Folio — Scorecard handler bridge.
//!
//! Merges the three platform G-27 handler modules into the Folio route tree
//! without calling `.with_state()` — state is applied once at the FolioApp
//! boundary in `atlas_apps/folio.rs`.
//!
//! # Route surface mounted here
//!
//! From `scorecard_entries`:
//!   PATCH  /api/scorecard-entries/:entry_id/verify
//!   GET    /api/scorecard-templates/:template_id/display-rules
//!
//! From `scorecard_analytics`:
//!   GET    /api/scorecard-templates/:template_id/analytics
//!   GET    /api/scorecard-templates/:template_id/leaderboard
//!   GET    /api/scorecard-templates/:template_id/anomalies
//!   POST   /api/scorecard-templates/:template_id/analytics/refresh
//!
//! From `scorecard_display_rules` (admin-only):
//!   GET    /api/admin/scorecard-templates/:template_id/display-rules
//!   POST   /api/admin/scorecard-display-rules
//!   PATCH  /api/admin/scorecard-display-rules/:id
//!   DELETE /api/admin/scorecard-display-rules/:id

use axum::Router;
use sea_orm::DatabaseConnection;

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        // G-27 entry verification + display rule reads (tenant-facing)
        .merge(crate::handlers::scorecard_entries::routes())
        // G-27 portfolio analytics + leaderboard + anomaly feed (tenant-facing)
        .merge(crate::handlers::scorecard_analytics::routes())
        // G-27 display rule admin CRUD (configurator UI)
        .merge(crate::handlers::scorecard_display_rules::routes())
}
