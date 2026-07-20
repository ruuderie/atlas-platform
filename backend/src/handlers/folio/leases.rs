//! Folio — Leases handler.
//!
//! # Routes
//!
//! ```ignore
//! GET  /api/folio/leases
//!      List all leases for the tenant.
//!      -> 200 [LeaseSummary]
//!
//! POST /api/folio/leases
//!      Create a new lease contract.
//!      Body: CreateLeaseHttpInput
//!      -> 201 { "id": uuid }
//!
//! GET  /api/folio/leases/:id
//!      Full lease detail — all entity fields including financial terms.
//!      -> 200 LeaseDetail
//!
//! GET  /api/folio/leases/:id/invoices
//!      Ledger entries (G-03) for this lease contract.
//!      billable_entity_type = "atlas_contract", billable_entity_id = lease_id
//!      -> 200 [LeaseInvoiceSummary]
//! ```

use axum::{
    Router,
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::user;
use crate::services::pm::lease::{
    ActivateLeaseInput, CounterpartyKind, CreateHistoricalLeaseInput, CreateLeaseInput,
    CreateOccupancyInput, LeaseService, OfflinePerson,
};
use crate::types::pm::{Currency, GuaranteeType};

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/leases", get(list_leases).post(create_lease))
        .route(
            "/api/folio/leases/historical",
            post(create_historical_lease),
        )
        .route("/api/folio/leases/occupancy", post(create_occupancy))
        .route("/api/folio/leases/{id}", get(get_lease))
        .route(
            "/api/folio/leases/{id}/activate",
            post(activate_lease),
        )
        .route("/api/folio/leases/{id}/invoices", get(list_lease_invoices))
}

// ── Request / response types ──────────────────────────────────────────────────

/// Sparse list row — used by the lease list page.
#[derive(Debug, Serialize)]
struct LeaseSummary {
    pub id: Uuid,
    pub asset_id: Option<Uuid>,
    pub counterparty_user_id: Option<Uuid>,
    /// Offline / draft display name from terms_metadata when no Atlas user.
    pub counterparty_label: Option<String>,
    pub currency: String,
    pub status: String,
    pub monthly_rent_cents: Option<i64>,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Full detail response — all fields the operator needs on the detail page.
#[derive(Debug, Serialize)]
pub struct LeaseDetail {
    pub id: Uuid,
    pub asset_id: Option<Uuid>,
    pub counterparty_user_id: Option<Uuid>,
    pub contract_type: String,
    /// Monthly (or interval) rent in cents.
    pub recurring_amount_cents: Option<i64>,
    pub currency: String,
    pub billing_interval: String,
    pub status: String,
    pub guarantee_type: Option<String>,
    pub auto_renew: bool,
    pub start_date: chrono::NaiveDate,
    pub end_date: Option<chrono::NaiveDate>,
    pub signed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub terminated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub termination_reason: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Single ledger entry for this lease, from G-03 `atlas_ledger_entries`.
#[derive(Debug, Serialize)]
pub struct LeaseInvoiceSummary {
    pub id: Uuid,
    pub gross_amount_cents: i64,
    pub currency: String,
    pub status: String,
    pub payment_rail: Option<String>,
    pub due_date: Option<chrono::NaiveDate>,
    pub paid_at: Option<chrono::DateTime<chrono::Utc>>,
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

#[derive(Debug, Deserialize)]
pub struct CreateOccupancyHttpInput {
    pub asset_id: Uuid,
    pub offline_name: String,
    pub offline_phone: Option<String>,
    pub offline_email: Option<String>,
    pub offline_notes: Option<String>,
    pub start_date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct ActivateLeaseHttpInput {
    pub monthly_rent_cents: i64,
    pub currency: String,
    pub guarantee_type: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: Option<chrono::NaiveDate>,
    pub auto_renew: bool,
    pub counterparty_user_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CreateHistoricalLeaseHttpInput {
    pub asset_id: Uuid,
    /// `atlas_user` | `offline_person`
    pub counterparty_kind: String,
    pub counterparty_user_id: Option<Uuid>,
    pub offline_name: Option<String>,
    pub offline_phone: Option<String>,
    pub offline_email: Option<String>,
    pub offline_notes: Option<String>,
    pub monthly_rent_cents: i64,
    pub currency: String,
    pub guarantee_type: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: Option<chrono::NaiveDate>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// POST /api/folio/leases/occupancy — draft occupancy (offline person, no rent yet).
async fn create_occupancy(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<CreateOccupancyHttpInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let id = LeaseService::create_occupancy_draft(
        &db,
        tenant_id,
        CreateOccupancyInput {
            asset_id: input.asset_id,
            offline_person: OfflinePerson {
                name: input.offline_name,
                phone: input.offline_phone,
                email: input.offline_email,
                notes: input.offline_notes,
            },
            start_date: input.start_date,
        },
    )
    .await
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("required") || msg.contains("already has") {
            tracing::warn!(%tenant_id, "create_occupancy validation: {msg}");
            StatusCode::UNPROCESSABLE_ENTITY
        } else {
            tracing::error!(%tenant_id, "create_occupancy: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;
    Ok((
        StatusCode::CREATED,
        axum::response::Json(CreateLeaseResponse { id }),
    ))
}

/// POST /api/folio/leases/{id}/activate — draft → active with commercial terms.
async fn activate_lease(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(lease_id): Path<Uuid>,
    Json(input): Json<ActivateLeaseHttpInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let guarantee_type = GuaranteeType::try_from(input.guarantee_type.clone())
        .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
    let currency = Currency::try_from(input.currency.clone())
        .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;

    LeaseService::activate_lease(
        &db,
        tenant_id,
        lease_id,
        ActivateLeaseInput {
            monthly_rent_cents: input.monthly_rent_cents,
            currency,
            guarantee_type,
            start_date: input.start_date,
            end_date: input.end_date,
            auto_renew: input.auto_renew,
            counterparty_user_id: input.counterparty_user_id,
        },
    )
    .await
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("not found") {
            StatusCode::NOT_FOUND
        } else if msg.contains("only draft") || msg.contains("required") {
            StatusCode::UNPROCESSABLE_ENTITY
        } else {
            tracing::error!(%tenant_id, %lease_id, "activate_lease: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/folio/leases/historical — backfill lease (offline tenant OK).
async fn create_historical_lease(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<CreateHistoricalLeaseHttpInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let kind = CounterpartyKind::parse(&input.counterparty_kind).ok_or_else(|| {
        tracing::warn!(
            %tenant_id,
            kind = %input.counterparty_kind,
            "create_historical_lease: invalid counterparty_kind"
        );
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    let guarantee_type = GuaranteeType::try_from(input.guarantee_type.clone()).map_err(|_| {
        StatusCode::UNPROCESSABLE_ENTITY
    })?;
    let currency = Currency::try_from(input.currency.clone()).map_err(|_| {
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    let offline_person = match kind {
        CounterpartyKind::OfflinePerson => Some(OfflinePerson {
            name: input.offline_name.unwrap_or_default(),
            phone: input.offline_phone,
            email: input.offline_email,
            notes: input.offline_notes,
        }),
        CounterpartyKind::AtlasUser => None,
    };

    let id = LeaseService::create_historical_lease(
        &db,
        tenant_id,
        CreateHistoricalLeaseInput {
            asset_id: input.asset_id,
            counterparty_kind: kind,
            counterparty_user_id: input.counterparty_user_id,
            offline_person,
            monthly_rent_cents: input.monthly_rent_cents,
            currency,
            start_date: input.start_date,
            end_date: input.end_date,
            guarantee_type,
        },
    )
    .await
    .map_err(|e| {
        let msg = e.to_string();
        if msg.contains("required") {
            tracing::warn!(%tenant_id, "create_historical_lease validation: {msg}");
            StatusCode::UNPROCESSABLE_ENTITY
        } else {
            tracing::error!(%tenant_id, "create_historical_lease: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    Ok((
        StatusCode::CREATED,
        axum::response::Json(CreateLeaseResponse { id }),
    ))
}

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
        .map(|l| {
            let counterparty_label = l
                .terms_metadata
                .as_ref()
                .and_then(|m| m.get("offline_person"))
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                .map(|s| s.to_string());
            LeaseSummary {
                id: l.id,
                asset_id: l.asset_id,
                counterparty_user_id: l.counterparty_user_id,
                counterparty_label,
                currency: l.currency,
                status: l.status,
                monthly_rent_cents: l.recurring_amount_cents,
                start_date: Some(l.start_date),
                end_date: l.end_date,
                created_at: l.created_at,
            }
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
        tracing::warn!(
            "create_lease: invalid guarantee_type '{}'",
            input.guarantee_type
        );
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

    Ok((
        StatusCode::CREATED,
        axum::response::Json(CreateLeaseResponse { id }),
    ))
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

    Ok(axum::response::Json(LeaseDetail {
        id: lease.id,
        asset_id: lease.asset_id,
        counterparty_user_id: lease.counterparty_user_id,
        contract_type: lease.contract_type,
        recurring_amount_cents: lease.recurring_amount_cents,
        currency: lease.currency,
        billing_interval: lease.billing_interval,
        status: lease.status,
        guarantee_type: lease
            .terms_metadata
            .as_ref()
            .and_then(|m| m.get("guarantee_type"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        auto_renew: lease.auto_renew,
        start_date: lease.start_date,
        end_date: lease.end_date,
        signed_at: lease.signed_at,
        terminated_at: lease.terminated_at,
        termination_reason: lease.termination_reason,
        created_at: lease.created_at,
    }))
}

/// GET /api/folio/leases/:id/invoices
///
/// Returns ledger entries from G-03 `atlas_ledger_entries` where
/// `billable_entity_type = "atlas_contract"` and `billable_entity_id = lease_id`.
async fn list_lease_invoices(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(lease_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    // Authorise: confirm the lease belongs to this tenant before exposing invoices.
    let lease = crate::entities::atlas_contract::Entity::find_by_id(lease_id)
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!(%lease_id, "list_lease_invoices: lease lookup error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    if lease.tenant_id != tenant_id {
        return Err(StatusCode::NOT_FOUND);
    }

    let entries = crate::entities::atlas_ledger_entry::Entity::find()
        .filter(crate::entities::atlas_ledger_entry::Column::TenantId.eq(tenant_id))
        .filter(
            crate::entities::atlas_ledger_entry::Column::BillableEntityType.eq("atlas_contract"),
        )
        .filter(crate::entities::atlas_ledger_entry::Column::BillableEntityId.eq(lease_id))
        .order_by_desc(crate::entities::atlas_ledger_entry::Column::CreatedAt)
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, %lease_id, "list_lease_invoices error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let invoices: Vec<LeaseInvoiceSummary> = entries
        .into_iter()
        .map(|e| LeaseInvoiceSummary {
            id: e.id,
            gross_amount_cents: e.gross_amount_cents,
            currency: e.currency,
            status: e.status,
            payment_rail: e.payment_rail,
            due_date: e.due_date,
            paid_at: e.paid_at,
            created_at: e.created_at,
        })
        .collect();

    Ok(axum::response::Json(invoices))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn resolve_tenant_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Uuid, StatusCode> {
    crate::extractors::tenant::resolve_tenant_id(db, user_id).await
}
