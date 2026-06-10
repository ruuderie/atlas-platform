//! GET /api/folio/pm/clients       — list all client accounts for the PMC tenant
//! POST /api/folio/pm/clients      — onboard a new client account
//!
//! Both routes require `PropertyManagerOnly` extractor (role + PMC mode check).

use axum::{
    Extension, Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set, ActiveModelTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::extractors::folio_role::PropertyManagerOnly;
use crate::extractors::tenant::TenantContext;

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/pm/clients", get(list_clients).post(create_client))
}

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ClientSummary {
    pub account_id:          Uuid,
    pub display_name:        String,
    pub contact_name:        Option<String>,
    pub contact_email:       Option<String>,
    pub property_count:      Option<i64>,
    pub unit_count:          Option<i64>,
    pub active_lease_count:  Option<i64>,
    pub occupancy_pct:       Option<f64>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// List all client accounts managed by this PMC tenant, with aggregate metrics.
///
/// Fetches accounts + per-client metrics in two queries:
///   1. SELECT * FROM account WHERE tenant_id = $1
///   2. CTE GROUP BY query for portfolio/asset/lease counts (see services::pm::aggregates)
///
/// Total: 2 DB round-trips regardless of client count (no N+1).
async fn list_clients(
    _guard: PropertyManagerOnly,
    ctx: TenantContext,
    Extension(db): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    // ── 1. Fetch all client accounts ─────────────────────────────────────────
    let accounts = match crate::entities::atlas_account::Entity::find()
        .filter(crate::entities::atlas_account::Column::TenantId.eq(ctx.tenant_id))
        .order_by_asc(crate::entities::atlas_account::Column::CreatedAt)
        .all(&db)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::error!(error = %e, "pm/clients: DB error listing accounts");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // ── 2. Fetch aggregate metrics for all clients in one CTE query ──────────
    let metrics_map: std::collections::HashMap<Uuid, crate::services::pm::aggregates::ClientMetrics> =
        match crate::services::pm::aggregates::fetch_client_metrics(&db, ctx.tenant_id).await {
            Ok(rows) => rows.into_iter().map(|m| (m.account_id, m)).collect(),
            Err(e) => {
                tracing::error!(error = %e, "pm/clients: aggregate metrics query failed");
                // Degrade gracefully — return accounts without metrics rather than 500
                std::collections::HashMap::new()
            }
        };

    // ── 3. Merge ─────────────────────────────────────────────────────────────
    let summaries: Vec<ClientSummary> = accounts
        .into_iter()
        .map(|a| {
            let m = metrics_map.get(&a.id);
            ClientSummary {
                account_id:         a.id,
                display_name:       a.name.clone(),
                contact_name:       None, // TODO: join atlas_contact
                contact_email:      None,
                property_count:     m.map(|x| x.property_count),
                unit_count:         m.map(|x| x.unit_count),
                active_lease_count: m.map(|x| x.active_lease_count),
                occupancy_pct:      m.map(|x| x.occupancy_pct),
            }
        })
        .collect();

    Json(summaries).into_response()
}

// ── Create client ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateClientRequest {
    pub display_name:  String,
    pub contact_name:  Option<String>,
    pub contact_email: Option<String>,
    /// Internal notes — visible to PM only, not to the client.
    pub internal_notes: Option<String>,
}

#[derive(Serialize)]
pub struct CreateClientResponse {
    pub account_id: Uuid,
}

/// Onboard a new client account under this PMC tenant.
///
/// Creates an `atlas_account` row scoped to the PM's tenant_id.
/// The client is not yet a user — invite flow is a separate step.
async fn create_client(
    _guard: PropertyManagerOnly,
    ctx: TenantContext,
    Extension(db): Extension<DatabaseConnection>,
    Json(body): Json<CreateClientRequest>,
) -> impl IntoResponse {
    use crate::entities::atlas_account;

    let new_account = atlas_account::ActiveModel {
        id:        Set(Uuid::new_v4()),
        tenant_id: Set(ctx.tenant_id),
        name:      Set(body.display_name),
        ..Default::default()
    };

    match new_account.insert(&db).await {
        Ok(account) => (
            StatusCode::CREATED,
            Json(CreateClientResponse { account_id: account.id }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "pm/clients: failed to create client account");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
