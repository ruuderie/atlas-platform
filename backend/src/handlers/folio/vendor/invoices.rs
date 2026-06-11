//! Folio Vendor — Invoices handler.
//!
//! Vendor invoices are `atlas_ledger_entry` rows where
//! `billable_entity_type = 'service_provider'` and
//! `billable_entity_id` matches the vendor's `atlas_service_provider.id`.
//!
//! # Authorization
//! All routes use the `VendorOnly` extractor — declarative role gate.
//!
//! # Routes
//!
//! ```ignore
//! GET  /api/folio/vendor/invoices           ?status=pending|paid|all
//!      -> 200 [VendorInvoiceSummary]
//!
//! GET  /api/folio/vendor/invoices/{id}
//!      -> 200 VendorInvoiceDetail
//!
//! POST /api/folio/vendor/invoices
//!      Submit a new invoice against a completed work order.
//!      -> 201 { "id": uuid }
//! ```

use axum::{
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection,
    EntityTrait, QueryFilter, QueryOrder,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{atlas_ledger_entry, atlas_service_provider, user};
use crate::extractors::folio_role::VendorOnly;

// ── Route constructors ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/vendor/invoices",     get(list_invoices).post(submit_invoice))
        .route("/api/folio/vendor/invoices/{id}", get(get_invoice))
}

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ListInvoicesQuery {
    #[serde(default = "default_invoice_status")]
    pub status: String,
}
fn default_invoice_status() -> String { "pending".to_string() }

#[derive(Debug, Serialize)]
pub struct VendorInvoiceSummary {
    pub id:                 Uuid,
    pub gross_amount_cents: i64,
    pub currency:           String,
    pub status:             String,
    pub due_date:           Option<chrono::NaiveDate>,
    pub paid_at:            Option<chrono::DateTime<chrono::Utc>>,
    pub created_at:         chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct VendorInvoiceDetail {
    pub id:                 Uuid,
    pub gross_amount_cents: i64,
    pub fee_amount_cents:   i64,
    pub net_amount_cents:   i64,
    pub currency:           String,
    pub payment_rail:       Option<String>,
    pub external_tx_id:     Option<String>,
    pub status:             String,
    pub due_date:           Option<chrono::NaiveDate>,
    pub paid_at:            Option<chrono::DateTime<chrono::Utc>>,
    pub billable_entity_id: Uuid,
    pub created_at:         chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitInvoiceInput {
    pub work_order_id:      Uuid,
    pub gross_amount_cents: i64,
    pub currency:           String,
    pub payment_rail:       Option<String>,
    pub due_date:           Option<chrono::NaiveDate>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/folio/vendor/invoices
async fn list_invoices(
    _guard: VendorOnly,
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(params): Query<ListInvoicesQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, sp) = resolve_vendor_context(&db, current_user.id).await?;

    let mut query = atlas_ledger_entry::Entity::find()
        .filter(atlas_ledger_entry::Column::TenantId.eq(tenant_id))
        .filter(atlas_ledger_entry::Column::BillableEntityType.eq("service_provider"))
        .filter(atlas_ledger_entry::Column::BillableEntityId.eq(sp.id));

    match params.status.as_str() {
        "paid" => { query = query.filter(atlas_ledger_entry::Column::Status.eq("paid")); }
        "all"  => {}
        _      => { query = query.filter(atlas_ledger_entry::Column::Status.ne("paid")); }
    }

    let entries = query
        .order_by_desc(atlas_ledger_entry::Column::CreatedAt)
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let summaries: Vec<VendorInvoiceSummary> = entries
        .into_iter()
        .map(|e| VendorInvoiceSummary {
            id:                 e.id,
            gross_amount_cents: e.gross_amount_cents,
            currency:           e.currency,
            status:             e.status,
            due_date:           e.due_date,
            paid_at:            e.paid_at,
            created_at:         e.created_at,
        })
        .collect();

    Ok(Json(summaries))
}

/// GET /api/folio/vendor/invoices/{id}
async fn get_invoice(
    _guard: VendorOnly,
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, sp) = resolve_vendor_context(&db, current_user.id).await?;

    let entry = atlas_ledger_entry::Entity::find_by_id(id)
        .filter(atlas_ledger_entry::Column::TenantId.eq(tenant_id))
        .filter(atlas_ledger_entry::Column::BillableEntityType.eq("service_provider"))
        .filter(atlas_ledger_entry::Column::BillableEntityId.eq(sp.id))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(VendorInvoiceDetail {
        id:                 entry.id,
        gross_amount_cents: entry.gross_amount_cents,
        fee_amount_cents:   entry.fee_amount_cents,
        net_amount_cents:   entry.net_amount_cents,
        currency:           entry.currency,
        payment_rail:       entry.payment_rail,
        external_tx_id:     entry.external_tx_id,
        status:             entry.status,
        due_date:           entry.due_date,
        paid_at:            entry.paid_at,
        billable_entity_id: entry.billable_entity_id,
        created_at:         entry.created_at,
    }))
}

/// POST /api/folio/vendor/invoices
async fn submit_invoice(
    _guard: VendorOnly,
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<SubmitInvoiceInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let (tenant_id, sp) = resolve_vendor_context(&db, current_user.id).await?;

    // Platform takes 0% fee on vendor invoices — net = gross.
    // Billing admin may add adjustments via ledger splits in a later reconciliation pass.
    let new_entry = atlas_ledger_entry::ActiveModel {
        id:                    Set(Uuid::new_v4()),
        tenant_id:             Set(tenant_id),
        billable_entity_type:  Set("service_provider".to_string()),
        billable_entity_id:    Set(sp.id),
        payer_user_id:         Set(None),
        payer_email:           Set(None),
        gross_amount_cents:    Set(input.gross_amount_cents),
        fee_amount_cents:      Set(0),
        net_amount_cents:      Set(input.gross_amount_cents),
        currency:              Set(input.currency),
        payment_rail:          Set(input.payment_rail),
        external_tx_id:        Set(None),
        receipt_attachment_id: Set(None),
        status:                Set("pending".to_string()),
        due_date:              Set(input.due_date),
        paid_at:               Set(None),
        verified_by_user_id:   Set(None),
        verified_at:           Set(None),
        reconciled_at:         Set(None),
        reconciliation_note:   Set(None),
        created_at:            Set(chrono::Utc::now()),
    };

    let saved = new_entry.insert(&db).await.map_err(|e| {
        tracing::error!("vendor::submit_invoice failed: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, Json(serde_json::json!({ "id": saved.id }))))
}

// ── Shared helper ─────────────────────────────────────────────────────────────

async fn resolve_vendor_context(
    db:      &DatabaseConnection,
    user_id: Uuid,
) -> Result<(Uuid, atlas_service_provider::Model), StatusCode> {
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

    let tenant_id = profile.tenant_id;

    let sp = atlas_service_provider::Entity::find()
        .filter(atlas_service_provider::Column::TenantId.eq(tenant_id))
        .filter(atlas_service_provider::Column::UserId.eq(user_id))
        .filter(atlas_service_provider::Column::Status.ne("inactive"))
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)?;

    Ok((tenant_id, sp))
}
