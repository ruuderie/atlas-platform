//! Folio — Leases handler.
//!
//! # Routes
//!
//! ```ignore
//! GET  /api/folio/leases
//!      List all active leases for the tenant.
//!      -> 200 [LeaseSummary]
//!
//! POST /api/folio/leases
//!      Create a new lease contract.
//!      Body: CreateLeaseHttpInput
//!      -> 201 { "id": uuid }
//!
//! GET  /api/folio/leases/:id
//!      Fetch a lease record.
//!      -> 200 atlas_contract::Model (raw JSON)
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
use crate::services::pm::lease::{CreateLeaseInput, LeaseService};
use crate::types::pm::{Currency, GuaranteeType};

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/leases", get(list_leases).post(create_lease))
        .route("/api/folio/leases/{id}", get(get_lease))
}

// ── Request / response types ──────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct LeaseSummary {
    pub id: Uuid,
    pub asset_id: Option<Uuid>,
    pub counterparty_user_id: Option<Uuid>,
    pub currency: String,
    pub status: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLeaseHttpInput {
    pub asset_id: Uuid,
    /// Required: the tenant (lessee) must already have an Atlas account.
    pub counterparty_user_id: Uuid,
    pub monthly_rent_cents: i64,
    /// ISO 4217 currency code e.g. "USD", "BRL".
    pub currency: String,
    /// One of: "security_deposit", "guarantor", "fiador", "seguro_fianca",
    ///          "titulo_capitalizacao", "none"
    pub guarantee_type: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: Option<chrono::NaiveDate>,
    pub auto_renew: bool,
}

#[derive(Debug, Serialize)]
struct CreateLeaseResponse {
    pub id: Uuid,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/folio/leases
async fn list_leases(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let leases = crate::entities::atlas_contract::Entity::find()
        .filter(crate::entities::atlas_contract::Column::TenantId.eq(tenant_id))
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "list_leases error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let summaries: Vec<LeaseSummary> = leases
        .into_iter()
        .map(|l| LeaseSummary {
            id: l.id,
            asset_id: l.asset_id,
            counterparty_user_id: l.counterparty_user_id,
            currency: l.currency,
            status: l.status,
            start_date: Some(l.start_date),
            end_date: l.end_date,
            created_at: l.created_at,
        })
        .collect();

    Ok(axum::response::Json(summaries))
}

/// POST /api/folio/leases
async fn create_lease(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<CreateLeaseHttpInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let guarantee_type = GuaranteeType::try_from(input.guarantee_type.clone()).map_err(|_| {
        tracing::warn!("create_lease: invalid guarantee_type '{}'", input.guarantee_type);
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    let currency = Currency::try_from(input.currency.clone()).map_err(|_| {
        tracing::warn!("create_lease: invalid currency '{}'", input.currency);
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    let id = LeaseService::create_lease(
        &db,
        tenant_id,
        CreateLeaseInput {
            asset_id: input.asset_id,
            counterparty_user_id: input.counterparty_user_id,
            monthly_rent_cents: input.monthly_rent_cents,
            currency,
            start_date: input.start_date,
            end_date: input.end_date,
            auto_renew: input.auto_renew,
            guarantee_type,
        },
    )
    .await
    .map_err(|e| {
        tracing::error!(%tenant_id, "create_lease error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, axum::response::Json(CreateLeaseResponse { id })))
}

/// GET /api/folio/leases/:id
async fn get_lease(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(lease_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let lease = crate::entities::atlas_contract::Entity::find_by_id(lease_id)
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, %lease_id, "get_lease DB error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Tenant isolation.
    if lease.tenant_id != tenant_id {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(axum::response::Json(LeaseSummary {
        id: lease.id,
        asset_id: lease.asset_id,
        counterparty_user_id: lease.counterparty_user_id,
        currency: lease.currency,
        status: lease.status,
        start_date: Some(lease.start_date),
        end_date: lease.end_date,
        created_at: lease.created_at,
    }))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

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
