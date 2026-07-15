//! Folio Deal Ops handler — Wholesaling + Creative Finance.
//!
//! ```ignore
//! GET    /api/folio/deals?track=wholesale|creative_finance
//! POST   /api/folio/deals
//! POST   /api/folio/deals/mao
//! POST   /api/folio/deals/{id}/advance
//! POST   /api/folio/deals/{id}/structure
//! POST   /api/folio/deals/{id}/cya
//! POST   /api/folio/deals/{id}/title
//! POST   /api/folio/deals/{id}/convert
//! POST   /api/folio/deals/{id}/assign
//! POST   /api/folio/deals/{id}/install-lease-option
//! POST   /api/folio/deals/{id}/convert-to-cf
//! ```

use axum::{
    Router,
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::user;
use crate::services::pm::deal_ops::{
    CreateDealInput, DealOpsService, DealSummary, StructureOfferInput,
};
use crate::types::pm::DealTrack;

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/deals", get(list_deals).post(create_deal))
        .route("/api/folio/deals/mao", post(compute_mao))
        .route("/api/folio/deals/{id}/advance", post(advance_stage))
        .route("/api/folio/deals/{id}/structure", post(structure_offer))
        .route("/api/folio/deals/{id}/cya", post(set_cya))
        .route("/api/folio/deals/{id}/title", post(set_title))
        .route("/api/folio/deals/{id}/convert", post(convert_acquisition))
        .route("/api/folio/deals/{id}/assign", post(create_assignment))
        .route(
            "/api/folio/deals/{id}/install-lease-option",
            post(install_lease_option),
        )
        .route("/api/folio/deals/{id}/convert-to-cf", post(convert_to_cf))
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub track: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDealHttp {
    pub track: String,
    pub address: String,
    pub arv_cents: Option<i64>,
    pub repair_cents: Option<i64>,
    pub asking_cents: Option<i64>,
    pub loan_balance_cents: Option<i64>,
    pub piti_cents: Option<i64>,
    pub sqft: Option<i64>,
    pub vacant: Option<bool>,
    pub listed: Option<bool>,
    pub seller_motivation: Option<String>,
    pub owner_user_id: Option<Uuid>,
    pub as_buyer: Option<bool>,
    pub buyer_fit: Option<String>,
    pub max_cash_cents: Option<i64>,
}

#[derive(Debug, Serialize)]
struct IdResponse {
    pub id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct AdvanceHttp {
    pub stage: String,
}

#[derive(Debug, Deserialize)]
pub struct MaoHttp {
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
pub struct CyaHttp {
    pub signed: bool,
}

#[derive(Debug, Deserialize)]
pub struct TitleHttp {
    pub title_search_ordered: Option<bool>,
    pub title_clear: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ConvertHttp {
    pub counterparty_user_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
struct ConvertResponse {
    pub asset_id: Uuid,
    pub contract_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct AssignHttp {
    pub assignee_user_id: Option<Uuid>,
    pub assignment_fee_cents: i64,
    pub deposit_cents: i64,
    pub expires_days: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct InstallLoHttp {
    pub asset_id: Uuid,
    pub counterparty_user_id: Uuid,
    pub option_price_cents: i64,
    pub option_deposit_cents: i64,
    pub monthly_rent_cents: i64,
    pub dpap_extra_cents: Option<i64>,
    pub dpap_price_credit_cents: Option<i64>,
    pub term_end: Option<chrono::NaiveDate>,
}

async fn list_deals(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant(&db, current_user.id).await?;
    let track = q
        .track
        .map(DealTrack::try_from)
        .transpose()
        .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
    let deals = DealOpsService::list_deals(&db, tenant_id, track)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "list_deals: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(deals))
}

async fn create_deal(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(body): Json<CreateDealHttp>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant(&db, current_user.id).await?;
    let track = DealTrack::try_from(body.track.clone()).map_err(|_| {
        StatusCode::UNPROCESSABLE_ENTITY
    })?;
    let input = CreateDealInput {
        track,
        address: body.address,
        arv_cents: body.arv_cents,
        repair_cents: body.repair_cents,
        asking_cents: body.asking_cents,
        loan_balance_cents: body.loan_balance_cents,
        piti_cents: body.piti_cents,
        sqft: body.sqft,
        vacant: body.vacant,
        listed: body.listed,
        seller_motivation: body.seller_motivation,
        owner_user_id: body.owner_user_id.or(Some(current_user.id)),
        as_buyer: body.as_buyer,
        buyer_fit: body.buyer_fit,
        max_cash_cents: body.max_cash_cents,
    };
    let id = DealOpsService::create_deal(&db, tenant_id, input)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "create_deal: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok((StatusCode::CREATED, Json(IdResponse { id })))
}

async fn compute_mao(Json(input): Json<MaoHttp>) -> impl IntoResponse {
    use rust_decimal::Decimal;
    use std::str::FromStr;
    let multiplier = input
        .multiplier
        .as_deref()
        .and_then(|s| Decimal::from_str(s).ok());
    let r = DealOpsService::calculate_mao(
        input.arv_cents,
        input.repair_cents,
        input.wholesale_fee_cents,
        multiplier,
        input.currency,
    );
    Json(MaoHttpResult {
        arv_cents: r.arv_cents,
        repair_cents: r.repair_cents,
        wholesale_fee_cents: r.wholesale_fee_cents,
        multiplier: r.multiplier.to_string(),
        mao_cents: r.mao_cents,
        is_viable: r.is_viable,
        equity_cushion_pct: DealOpsService::equity_cushion_pct(
            r.arv_cents,
            r.mao_cents,
            r.repair_cents,
        ),
        currency: r.currency,
    })
}

async fn advance_stage(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(body): Json<AdvanceHttp>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant(&db, current_user.id).await?;
    DealOpsService::advance_stage(&db, tenant_id, id, &body.stage)
        .await
        .map_err(map_err_status)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn structure_offer(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(body): Json<StructureOfferInput>,
) -> Result<Json<DealSummary>, StatusCode> {
    let tenant_id = resolve_tenant(&db, current_user.id).await?;
    let summary = DealOpsService::structure_offer(&db, tenant_id, id, body)
        .await
        .map_err(map_err_status)?;
    Ok(Json(summary))
}

async fn set_cya(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(body): Json<CyaHttp>,
) -> Result<Json<DealSummary>, StatusCode> {
    let tenant_id = resolve_tenant(&db, current_user.id).await?;
    let summary = DealOpsService::set_cya_signed(&db, tenant_id, id, body.signed)
        .await
        .map_err(map_err_status)?;
    Ok(Json(summary))
}

async fn set_title(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(body): Json<TitleHttp>,
) -> Result<Json<DealSummary>, StatusCode> {
    let tenant_id = resolve_tenant(&db, current_user.id).await?;
    let summary = DealOpsService::set_title_flags(
        &db,
        tenant_id,
        id,
        body.title_search_ordered,
        body.title_clear,
    )
    .await
    .map_err(map_err_status)?;
    Ok(Json(summary))
}

async fn convert_acquisition(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(body): Json<ConvertHttp>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant(&db, current_user.id).await?;
    let (asset_id, contract_id) =
        DealOpsService::convert_acquisition(&db, tenant_id, id, body.counterparty_user_id)
            .await
            .map_err(map_err_status)?;
    Ok((
        StatusCode::CREATED,
        Json(ConvertResponse {
            asset_id,
            contract_id,
        }),
    ))
}

async fn create_assignment(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(body): Json<AssignHttp>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant(&db, current_user.id).await?;
    let contract_id = DealOpsService::create_assignment(
        &db,
        tenant_id,
        id,
        body.assignee_user_id,
        body.assignment_fee_cents,
        body.deposit_cents,
        body.expires_days.unwrap_or(14),
    )
    .await
    .map_err(map_err_status)?;
    Ok((StatusCode::CREATED, Json(IdResponse { id: contract_id })))
}

async fn install_lease_option(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(body): Json<InstallLoHttp>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant(&db, current_user.id).await?;
    let contract_id = DealOpsService::install_lease_option(
        &db,
        tenant_id,
        id,
        body.asset_id,
        body.counterparty_user_id,
        body.option_price_cents,
        body.option_deposit_cents,
        body.monthly_rent_cents,
        body.dpap_extra_cents,
        body.dpap_price_credit_cents,
        body.term_end,
    )
    .await
    .map_err(map_err_status)?;
    Ok((StatusCode::CREATED, Json(IdResponse { id: contract_id })))
}

async fn convert_to_cf(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<Json<DealSummary>, StatusCode> {
    let tenant_id = resolve_tenant(&db, current_user.id).await?;
    let summary = DealOpsService::convert_wholesale_to_cf(&db, tenant_id, id)
        .await
        .map_err(map_err_status)?;
    Ok(Json(summary))
}

fn map_err_status(e: anyhow::Error) -> StatusCode {
    let msg = e.to_string();
    if msg.contains("not found") {
        StatusCode::NOT_FOUND
    } else if msg.contains("terminal") || msg.contains("CYA") || msg.contains("Title must") {
        StatusCode::CONFLICT
    } else if msg.contains("unknown") || msg.contains("invalid") || msg.contains("requires") {
        StatusCode::UNPROCESSABLE_ENTITY
    } else {
        tracing::error!("deal_ops error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

async fn resolve_tenant(db: &DatabaseConnection, user_id: Uuid) -> Result<Uuid, StatusCode> {
    crate::extractors::tenant::resolve_tenant_id(db, user_id).await
}
