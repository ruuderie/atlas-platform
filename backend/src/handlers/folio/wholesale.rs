//! Folio — Wholesale handler.
//!
//! # Routes
//!
//! ```ignore
//! GET  /api/folio/wholesale
//!      List all wholesale leads (atlas_opportunities) for the tenant.
//!      -> 200 [WholesaleSummary]
//!
//! POST /api/folio/wholesale/mao
//!      Stateless MAO calculation — no auth, no DB write.
//!      Body: { "arv_cents": i64, "repair_cents": i64, "wholesale_fee_cents": i64,
//!              "multiplier"?: f64, "currency"?: string }
//!      -> 200 MaoResult
//!
//! POST /api/folio/wholesale
//!      Create a wholesale opportunity (lead).
//!      Body: CreateWholesaleHttpInput
//!      -> 201 { "id": uuid }
//!
//! POST /api/folio/wholesale/:id/advance
//!      Advance a lead's Kanban stage.
//!      Body: { "stage": WholesaleStage }
//!      -> 204
//! ```

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
use crate::services::pm::wholesale::WholesaleService;
use crate::types::pm::{SellerMotivation, WholesaleStage};

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/wholesale", get(list_leads).post(create_lead))
        .route("/api/folio/wholesale/mao", post(compute_mao))
        .route("/api/folio/wholesale/{id}/advance", post(advance_stage))
}

// ── Request / response types ──────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct WholesaleSummary {
    pub id: Uuid,
    pub asset_id: Option<Uuid>,
    pub name: String,
    pub status: String,
    pub currency: String,
    pub deal_amount_cents: Option<i64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct MaoHttpInput {
    pub arv_cents: i64,
    pub repair_cents: i64,
    pub wholesale_fee_cents: i64,
    /// Optional multiplier override (0.65–0.75). Defaults to 0.70.
    pub multiplier: Option<String>,
    pub currency: Option<String>,
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

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/folio/wholesale
async fn list_leads(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let opportunities = crate::entities::atlas_opportunity::Entity::find()
        .filter(crate::entities::atlas_opportunity::Column::TenantId.eq(tenant_id))
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "list_wholesale_leads error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let summaries: Vec<WholesaleSummary> = opportunities
        .into_iter()
        .map(|o| WholesaleSummary {
            id: o.id,
            asset_id: o.asset_id,
            name: o.name,
            status: o.status,
            currency: o.currency,
            deal_amount_cents: o.deal_amount_cents,
            created_at: o.created_at,
        })
        .collect();

    Ok(axum::response::Json(summaries))
}

/// POST /api/folio/wholesale/mao
///
/// Stateless — no auth required. The UI can call this speculatively as the
/// user types ARV and repair estimates before committing the lead.
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

    axum::response::Json(result)
}

/// POST /api/folio/wholesale
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
        input.owner_user_id,
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

/// POST /api/folio/wholesale/:id/advance
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

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn resolve_tenant_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Uuid, StatusCode> {
    crate::extractors::tenant::resolve_tenant_id(db, user_id).await
}
