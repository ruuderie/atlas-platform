//! # G26 Catalog HTTP handlers — Folio (Phase 6)
//!
//! Routes:
//!
//! | Method | Path                                    | Description                        |
//! |--------|-----------------------------------------|------------------------------------|
//! | POST   | /api/folio/catalog                      | Create catalog entry               |
//! | GET    | /api/folio/catalog                      | List entries (filterable)          |
//! | GET    | /api/folio/catalog/{id}                 | Get single entry                   |
//! | POST   | /api/folio/catalog/{id}/rate-rules      | Add rate rule                      |
//! | GET    | /api/folio/catalog/{id}/availability    | Check availability + effective prices |
//! | POST   | /api/folio/catalog/{id}/block           | Block a date range (operator hold) |

use axum::{
    Extension, Json, Router,
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use chrono::NaiveDate;
use sea_orm::{ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entities::user,
    services::pm::catalog::{
        CatalogFilter, CatalogService, CreateCatalogEntryPayload, CreateRateRulePayload,
    },
};

// ── Route constructor ─────────────────────────────────────────────────────────

/// Returns an Axum router containing all G26 catalog routes.
/// State-free — `.with_state(db)` is applied once in `FolioApp::authenticated_router()`.
pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/catalog", post(create_entry).get(list_entries))
        .route("/api/folio/catalog/{id}", get(get_entry))
        .route("/api/folio/catalog/{id}/rate-rules", post(create_rate_rule))
        .route(
            "/api/folio/catalog/{id}/availability",
            get(get_availability),
        )
        .route("/api/folio/catalog/{id}/block", post(block_dates))
}

// ── Shared tenant resolution ──────────────────────────────────────────────────

/// Resolve the tenant_id for a user. Same pattern as other Folio handlers.
async fn resolve_tenant_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Uuid, StatusCode> {
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

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ListEntriesQuery {
    entry_type: Option<String>,
    asset_id: Option<Uuid>,
    is_available: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct AvailabilityQuery {
    from_date: NaiveDate,
    to_date: NaiveDate,
    #[allow(dead_code)]
    channel: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BlockDatesInput {
    from_date: NaiveDate,
    to_date: NaiveDate,
    block_reason: Option<String>,
}

#[derive(Debug, Serialize)]
struct CatalogEntryResponse {
    id: Uuid,
    entry_type: String,
    name: String,
    description: Option<String>,
    asset_id: Option<Uuid>,
    base_price_cents: i64,
    currency: String,
    billing_interval: Option<String>,
    is_available: bool,
    min_quantity: i32,
    max_quantity: Option<i32>,
    catalog_metadata: serde_json::Value,
    sort_order: i32,
}

#[derive(Debug, Serialize)]
struct RateRuleResponse {
    id: Uuid,
    catalog_entry_id: Uuid,
    rule_name: Option<String>,
    applies_from: Option<NaiveDate>,
    applies_to: Option<NaiveDate>,
    day_of_week_mask: Option<i32>,
    /// Minimum duration in billing-interval units (nights/hours/days/etc.)
    min_duration: Option<i32>,
    channel: Option<String>,
    price_override_cents: Option<i64>,
    price_modifier_pct: Option<f64>,
    priority: i32,
    is_active: bool,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// POST /api/folio/catalog
async fn create_entry(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<CreateCatalogEntryPayload>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(status) => return status.into_response(),
    };

    match CatalogService::create_entry(&db, tenant_id, input).await {
        Ok(entry) => (
            StatusCode::CREATED,
            Json(CatalogEntryResponse {
                id: entry.id,
                entry_type: entry.entry_type,
                name: entry.name,
                description: entry.description,
                asset_id: entry.asset_id,
                base_price_cents: entry.base_price_cents,
                currency: entry.currency,
                billing_interval: entry.billing_interval,
                is_available: entry.is_available,
                min_quantity: entry.min_quantity,
                max_quantity: entry.max_quantity,
                catalog_metadata: entry.catalog_metadata,
                sort_order: entry.sort_order,
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(tenant_id = %tenant_id, "create_entry error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// GET /api/folio/catalog
async fn list_entries(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(q): Query<ListEntriesQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let filter = CatalogFilter {
        entry_type: q.entry_type,
        asset_id: q.asset_id,
        is_available: q.is_available,
    };

    let rows = CatalogService::list(&db, tenant_id, filter)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "list_entries error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let response: Vec<CatalogEntryResponse> = rows
        .into_iter()
        .map(|e| CatalogEntryResponse {
            id: e.id,
            entry_type: e.entry_type,
            name: e.name,
            description: e.description,
            asset_id: e.asset_id,
            base_price_cents: e.base_price_cents,
            currency: e.currency,
            billing_interval: e.billing_interval,
            is_available: e.is_available,
            min_quantity: e.min_quantity,
            max_quantity: e.max_quantity,
            catalog_metadata: e.catalog_metadata,
            sort_order: e.sort_order,
        })
        .collect();

    Ok(Json(response))
}

/// GET /api/folio/catalog/{id}
async fn get_entry(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(entry_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let entry = CatalogService::get(&db, tenant_id, entry_id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(CatalogEntryResponse {
        id: entry.id,
        entry_type: entry.entry_type,
        name: entry.name,
        description: entry.description,
        asset_id: entry.asset_id,
        base_price_cents: entry.base_price_cents,
        currency: entry.currency,
        billing_interval: entry.billing_interval,
        is_available: entry.is_available,
        min_quantity: entry.min_quantity,
        max_quantity: entry.max_quantity,
        catalog_metadata: entry.catalog_metadata,
        sort_order: entry.sort_order,
    }))
}

/// POST /api/folio/catalog/{id}/rate-rules
async fn create_rate_rule(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(entry_id): Path<Uuid>,
    Json(mut input): Json<CreateRateRulePayload>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(status) => return status.into_response(),
    };

    // Ensure catalog_entry_id on the payload matches the path param.
    input.catalog_entry_id = entry_id;

    match CatalogService::apply_rate_rule(&db, tenant_id, input).await {
        Ok(rule) => (
            StatusCode::CREATED,
            Json(RateRuleResponse {
                id: rule.id,
                catalog_entry_id: rule.catalog_entry_id,
                rule_name: rule.rule_name,
                applies_from: rule.applies_from,
                applies_to: rule.applies_to,
                day_of_week_mask: rule.day_of_week_mask,
                min_duration: rule.min_duration,
                channel: rule.channel,
                price_override_cents: rule.price_override_cents,
                price_modifier_pct: rule
                    .price_modifier_pct
                    .map(|d| f64::try_from(d).unwrap_or(0.0)),
                priority: rule.priority,
                is_active: rule.is_active,
            }),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(%tenant_id, %entry_id, "create_rate_rule error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// GET /api/folio/catalog/{id}/availability?from_date=&to_date=[&channel=]
async fn get_availability(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(entry_id): Path<Uuid>,
    Query(q): Query<AvailabilityQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    if q.to_date <= q.from_date {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    let summary =
        CatalogService::check_availability(&db, tenant_id, entry_id, q.from_date, q.to_date)
            .await
            .map_err(|e| {
                tracing::error!(%tenant_id, %entry_id, "get_availability error: {e:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

    Ok(Json(summary))
}

/// POST /api/folio/catalog/{id}/block
///
/// Operator-initiated block (cleaning day, owner hold, maintenance window).
/// Creates or updates availability rows setting `is_blocked = true`.
async fn block_dates(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(entry_id): Path<Uuid>,
    Json(input): Json<BlockDatesInput>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(status) => return status.into_response(),
    };

    // Verify the entry belongs to this tenant.
    if CatalogService::get(&db, tenant_id, entry_id).await.is_err() {
        return StatusCode::NOT_FOUND.into_response();
    }

    let reason = input
        .block_reason
        .as_deref()
        .unwrap_or("")
        .replace('\'', "''"); // basic SQL escape for single-quote in reason text
    let from_str = input.from_date.to_string();
    let to_str = input.to_date.to_string();

    let raw = format!(
        "INSERT INTO atlas_catalog_availability
            (id, catalog_entry_id, tenant_id, slot_date, total_inventory, reserved_count, is_blocked, block_reason)
         SELECT gen_random_uuid(), '{entry_id}', '{tenant_id}', d::date, 1, 0, true, '{reason}'
         FROM generate_series('{from_str}'::date, '{to_str}'::date - INTERVAL '1 day', INTERVAL '1 day') AS d
         ON CONFLICT (catalog_entry_id, slot_date)
         DO UPDATE SET is_blocked = true, block_reason = EXCLUDED.block_reason"
    );

    match db.execute_unprepared(&raw).await {
        Ok(_) => {
            tracing::info!(
                %tenant_id, %entry_id,
                from = %input.from_date, to = %input.to_date,
                "block_dates: dates blocked"
            );
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            tracing::error!(%tenant_id, %entry_id, "block_dates error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
