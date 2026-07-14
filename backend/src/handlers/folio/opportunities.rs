//! # G15 Opportunity HTTP handlers — Folio
//!
//! Routes:
//!
//! | Method | Path                                        | Description                          |
//! |--------|---------------------------------------------|--------------------------------------|
//! | POST   | /api/folio/opportunities                    | Create opportunity                   |
//! | GET    | /api/folio/opportunities                    | List opportunities (filterable)      |
//! | GET    | /api/folio/opportunities/{id}               | Get single opportunity               |
//! | POST   | /api/folio/opportunities/{id}/stage         | Advance pipeline stage               |
//! | POST   | /api/folio/opportunities/{id}/forecast      | Update probability + close date      |
//! | POST   | /api/folio/opportunities/{id}/quote         | Attach a quote to this opportunity   |

use axum::{
    Extension, Json, Router,
    extract::{Path, Query},
    http::StatusCode,
    routing::{get, post},
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entities::user,
    services::pm::opportunity::{CreateOpportunityPayload, OpportunityFilter, OpportunityService},
    types::pm::{OpportunityStage, OpportunityType},
};

// ── Route constructor ─────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route(
            "/api/folio/opportunities",
            post(create_opportunity).get(list_opportunities),
        )
        .route("/api/folio/opportunities/{id}", get(get_opportunity))
        .route("/api/folio/opportunities/{id}/stage", post(advance_stage))
        .route(
            "/api/folio/opportunities/{id}/forecast",
            post(update_forecast),
        )
}

// ── Tenant resolution ─────────────────────────────────────────────────────────

async fn resolve_tenant_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Uuid, StatusCode> {
    crate::extractors::tenant::resolve_tenant_id(db, user_id).await
}

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ListOpportunitiesQuery {
    stage: Option<String>,
    opportunity_type: Option<String>,
    owner_user_id: Option<Uuid>,
    asset_id: Option<Uuid>,
    open_only: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct CreateOpportunityInput {
    name: String,
    opportunity_type: String,
    asset_id: Option<Uuid>,
    owner_user_id: Option<Uuid>,
    counterparty_user_id: Option<Uuid>,
    description: Option<String>,
    amount_cents: Option<i64>,
    currency: Option<String>,
    probability_pct: Option<i32>,
    close_date: Option<chrono::NaiveDate>,
    financial_inputs: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct AdvanceStageInput {
    stage: String,
    won_amount_cents: Option<i64>,
    lost_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UpdateForecastInput {
    probability_pct: Option<i32>,
    close_date: Option<chrono::NaiveDate>,
    amount_cents: Option<i64>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn create_opportunity(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(body): Json<CreateOpportunityInput>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let opportunity_type = OpportunityType::try_from(body.opportunity_type.as_str())
        .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;

    let opp = OpportunityService::create(
        &db,
        tenant_id,
        CreateOpportunityPayload {
            name: body.name,
            opportunity_type,
            asset_id: body.asset_id,
            owner_user_id: body.owner_user_id,
            counterparty_user_id: body.counterparty_user_id,
            description: body.description,
            amount_cents: body.amount_cents,
            currency: body.currency,
            probability_pct: body.probability_pct,
            close_date: body.close_date,
            financial_inputs: body.financial_inputs,
            created_by_user_id: Some(current_user.id),
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(opp)))
}

async fn list_opportunities(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(q): Query<ListOpportunitiesQuery>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let stage = q
        .stage
        .as_deref()
        .map(OpportunityStage::try_from)
        .transpose()
        .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;

    let opportunity_type = q
        .opportunity_type
        .as_deref()
        .map(OpportunityType::try_from)
        .transpose()
        .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;

    let opps = OpportunityService::list(
        &db,
        tenant_id,
        OpportunityFilter {
            stage,
            opportunity_type,
            owner_user_id: q.owner_user_id,
            asset_id: q.asset_id,
            open_only: q.open_only,
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(opps))
}

async fn get_opportunity(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let opp = OpportunityService::get(&db, tenant_id, id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(opp))
}

async fn advance_stage(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(body): Json<AdvanceStageInput>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let new_stage = OpportunityStage::try_from(body.stage.as_str())
        .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;

    let opp = OpportunityService::advance_stage(
        &db,
        tenant_id,
        id,
        new_stage,
        body.won_amount_cents,
        body.lost_reason,
    )
    .await
    .map_err(|e| {
        if e.to_string().contains("not found") {
            StatusCode::NOT_FOUND
        } else if e.to_string().contains("already closed") {
            StatusCode::UNPROCESSABLE_ENTITY
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    Ok(Json(opp))
}

async fn update_forecast(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateForecastInput>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let opp = OpportunityService::update_forecast(
        &db,
        tenant_id,
        id,
        body.probability_pct,
        body.close_date,
        body.amount_cents,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(opp))
}
