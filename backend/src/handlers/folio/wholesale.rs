//! Folio — Wholesale handler (legacy paths; prefer `/api/folio/deals`).
//!
//! Kept for backward compatibility. List now filters `wholesale_lead` and
//! flattens `financial_inputs` for the Folio Kanban UI.

use axum::{
    Router,
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::user;
use crate::services::pm::deal_ops::DealOpsService;
use crate::services::pm::wholesale::WholesaleService;
use crate::types::pm::{PmOpportunityType, SellerMotivation, WholesaleStage};

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/wholesale", get(list_leads).post(create_lead))
        .route("/api/folio/wholesale/mao", post(compute_mao))
        .route("/api/folio/wholesale/{id}/advance", post(advance_stage))
}

#[derive(Debug, Serialize)]
struct WholesaleSummary {
    pub id: Uuid,
    pub asset_id: Option<Uuid>,
    pub name: String,
    /// Canonical stage string (also exposed as `stage` for UI).
    pub status: String,
    pub stage: String,
    pub property_address: String,
    pub arv_cents: Option<i64>,
    pub repair_cents: Option<i64>,
    pub offer_cents: Option<i64>,
    pub currency: String,
    pub deal_amount_cents: Option<i64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct MaoHttpInput {
    pub arv_cents: i64,
    pub repair_cents: i64,
    pub wholesale_fee_cents: i64,
    pub multiplier: Option<String>,
    pub currency: Option<String>,
}

#[derive(Debug, Serialize)]
struct MaoHttpResult {
    pub arv_cents: i64,
    pub repair_cents: i64,
    pub wholesale_fee_cents: i64,
    pub multiplier: String,
    pub mao_cents: i64,
    pub is_viable: bool,
    pub equity_cushion_pct: f64,
    pub currency: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateWholesaleHttpInput {
    pub address: String,
    pub arv_cents: i64,
    pub repair_cents: i64,
    pub seller_motivation: String,
    pub owner_user_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
struct CreateWholesaleResponse {
    pub id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct AdvanceStageInput {
    pub stage: String,
}

async fn list_leads(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let opportunities = crate::entities::atlas_opportunity::Entity::find()
        .filter(crate::entities::atlas_opportunity::Column::TenantId.eq(tenant_id))
        .filter(
            crate::entities::atlas_opportunity::Column::OpportunityType
                .eq(PmOpportunityType::WholesaleLead.to_string()),
        )
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "list_wholesale_leads error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let summaries: Vec<WholesaleSummary> = opportunities
        .into_iter()
        .map(|o| {
            let s = DealOpsService::summarize(&o);
            let stage = WholesaleStage::try_from(o.status.clone())
                .map(|st| st.canonical().to_string())
                .unwrap_or(o.status.clone());
            WholesaleSummary {
                id: s.id,
                asset_id: s.asset_id,
                name: s.name,
                status: stage.clone(),
                stage,
                property_address: s.property_address,
                arv_cents: s.arv_cents,
                repair_cents: s.repair_cents,
                offer_cents: s.offer_cents,
                currency: s.currency,
                deal_amount_cents: s.deal_amount_cents,
                created_at: s.created_at,
            }
        })
        .collect();

    Ok(axum::response::Json(summaries))
}

async fn compute_mao(Json(input): Json<MaoHttpInput>) -> impl IntoResponse {
    use rust_decimal::Decimal;
    use std::str::FromStr;

    let multiplier = input
        .multiplier
        .as_deref()
        .and_then(|s| Decimal::from_str(s).ok());

    let result = WholesaleService::calculate_mao(
        input.arv_cents,
        input.repair_cents,
        input.wholesale_fee_cents,
        multiplier,
        input.currency,
    );

    axum::response::Json(MaoHttpResult {
        arv_cents: result.arv_cents,
        repair_cents: result.repair_cents,
        wholesale_fee_cents: result.wholesale_fee_cents,
        multiplier: result.multiplier.to_string(),
        mao_cents: result.mao_cents,
        is_viable: result.is_viable,
        equity_cushion_pct: DealOpsService::equity_cushion_pct(
            result.arv_cents,
            result.mao_cents,
            result.repair_cents,
        ),
        currency: result.currency,
    })
}

async fn create_lead(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<CreateWholesaleHttpInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let motivation = SellerMotivation::try_from(input.seller_motivation.clone()).map_err(|_| {
        tracing::warn!(
            "create_lead: invalid seller_motivation '{}'",
            input.seller_motivation
        );
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    let id = WholesaleService::create_lead(
        &db,
        tenant_id,
        &input.address,
        input.arv_cents,
        input.repair_cents,
        motivation,
        input.owner_user_id.or(Some(current_user.id)),
    )
    .await
    .map_err(|e| {
        tracing::error!(%tenant_id, "create_wholesale_lead error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((
        StatusCode::CREATED,
        axum::response::Json(CreateWholesaleResponse { id }),
    ))
}

async fn advance_stage(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(opportunity_id): Path<Uuid>,
    Json(input): Json<AdvanceStageInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let stage = WholesaleStage::try_from(input.stage.clone()).map_err(|_| {
        tracing::warn!("advance_stage: invalid stage '{}'", input.stage);
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    WholesaleService::advance_stage(&db, tenant_id, opportunity_id, stage)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                StatusCode::NOT_FOUND
            } else if msg.contains("terminal") {
                StatusCode::CONFLICT
            } else {
                tracing::error!(%tenant_id, %opportunity_id, "advance_stage error: {e:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

async fn resolve_tenant_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Uuid, StatusCode> {
    crate::extractors::tenant::resolve_tenant_id(db, user_id).await
}
