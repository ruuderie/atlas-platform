//! Folio — PM Ledger Service (Phase 3)
//!
//! Thin PM-domain wrapper over `atlas_ledger_entries` (G-03).
//!
//! Provides the vocabulary the billing handler and background jobs need without
//! leaking ledger schema details into handler code.
//!
//! # Key operations
//!
//! - `create_pending()` — Opens a new ledger entry in `pending` status for a rent
//!   invoice. Called before the payment rail creates the provider invoice so the
//!   ledger_entry_id can be embedded in webhook metadata.
//!
//! - `mark_paid()` — Transitions a ledger entry from `pending`/`processing` to
//!   `paid`. Called by the webhook handler or the mempool poll background job.
//!
//! - `record_tx_id()` — Sets `external_tx_id` on a BTC on-chain ledger entry.
//!   Called by the tenant's POST /api/folio/billing/invoice/btc submission.
//!
//! - `list_btc_invoices()` — Returns BTC on-chain ledger entries for a tenant
//!   so the audit endpoint can surface their mempool status.
//!
//! # Status machine
//!
//! ```text
//! pending  →  processing  →  paid
//!          ↘              ↘  failed / refunded / waived
//!            (manual verify)
//! ```

use anyhow::{anyhow, Result};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection,
    EntityTrait, QueryFilter, QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::atlas_ledger_entry;
use crate::types::pm::Currency;

/// Returned by `list_btc_invoices()` for the BTC audit endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtcInvoiceAudit {
    pub ledger_entry_id: Uuid,
    pub gross_amount_cents: i64,
    pub currency: String,
    pub status: String,
    pub external_tx_id: Option<String>,
    pub due_date: Option<chrono::NaiveDate>,
    pub paid_at: Option<chrono::DateTime<Utc>>,
    pub created_at: chrono::DateTime<Utc>,
}

pub struct PmLedgerService;

impl PmLedgerService {
    // ── create_pending ────────────────────────────────────────────────────────

    /// Open a new ledger entry in `pending` status for a rent invoice.
    ///
    /// # Arguments
    /// - `billable_entity_type` — e.g. `"atlas_contract"` for a lease payment.
    /// - `billable_entity_id`   — The contract/lease ID this invoice is for.
    /// - `payer_user_id`        — The tenant paying rent. `None` for manual invoices.
    /// - `payment_rail`         — e.g. `"btc_onchain"`, `"stripe_connect"`, `"pix"`.
    pub async fn create_pending(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        billable_entity_type: &str,
        billable_entity_id: Uuid,
        payer_user_id: Option<Uuid>,
        gross_amount_cents: i64,
        currency: Currency,
        payment_rail: &str,
        due_date: Option<chrono::NaiveDate>,
    ) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        // Platform fee: Phase 3 uses 0 (no platform cut on PM rent payments).
        // Phase 4 will compute from `folio_platform_fee_bps` tenant setting.
        let fee_amount_cents: i64 = 0;
        let net_amount_cents = gross_amount_cents - fee_amount_cents;

        let model = atlas_ledger_entry::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_id),
            billable_entity_type: Set(billable_entity_type.to_owned()),
            billable_entity_id: Set(billable_entity_id),
            payer_user_id: Set(payer_user_id),
            payer_email: Set(None),
            gross_amount_cents: Set(gross_amount_cents),
            fee_amount_cents: Set(fee_amount_cents),
            net_amount_cents: Set(net_amount_cents),
            currency: Set(currency.to_string()),
            payment_rail: Set(Some(payment_rail.to_owned())),
            external_tx_id: Set(None),
            receipt_attachment_id: Set(None),
            status: Set("pending".to_owned()),
            due_date: Set(due_date),
            paid_at: Set(None),
            verified_by_user_id: Set(None),
            verified_at: Set(None),
            reconciled_at: Set(None),
            reconciliation_note: Set(None),
            created_at: Set(now),
        };

        model.insert(db).await.map_err(|e| {
            anyhow!(
                "PmLedgerService::create_pending failed for tenant {tenant_id}: {e}"
            )
        })?;

        tracing::info!(
            ledger_entry_id = %id, %tenant_id,
            billable_entity_type, %billable_entity_id,
            payment_rail, gross_amount_cents,
            currency = %currency,
            "PmLedgerService: pending ledger entry created"
        );

        Ok(id)
    }

    // ── mark_paid_for_tenant ──────────────────────────────────────────────────

    /// Transition a ledger entry to `paid` status with tenant ID guard.
    ///
    /// Used by handlers that have the tenant_id in context (background jobs,
    /// manual verify flows). Webhooks should use `mark_paid()` instead.
    pub async fn mark_paid_for_tenant(
        db: &DatabaseConnection,
        ledger_entry_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<()> {
        let entry = atlas_ledger_entry::Entity::find_by_id(ledger_entry_id)
            .filter(atlas_ledger_entry::Column::TenantId.eq(tenant_id))
            .one(db)
            .await?
            .ok_or_else(|| anyhow!(
                "ledger entry {ledger_entry_id} not found for tenant {tenant_id}"
            ))?;

        if entry.status == "paid" {
            tracing::debug!(
                ledger_entry_id = %ledger_entry_id,
                "PmLedgerService::mark_paid_for_tenant: already paid, skipping"
            );
            return Ok(());
        }

        let mut am: atlas_ledger_entry::ActiveModel = entry.into();
        am.status = Set("paid".to_owned());
        am.paid_at = Set(Some(Utc::now()));
        am.update(db).await.map_err(|e| {
            anyhow!("PmLedgerService::mark_paid_for_tenant failed for {ledger_entry_id}: {e}")
        })?;

        tracing::info!(
            ledger_entry_id = %ledger_entry_id, %tenant_id,
            "PmLedgerService: entry marked paid"
        );

        Ok(())
    }

    /// Transition a ledger entry to `paid` by ledger_entry_id only (no tenant check).
    ///
    /// Used by webhook handlers where the tenant_id is not available in the
    /// request context but is embedded in the provider metadata.
    /// Stamps `external_tx_id` with the provider's payment/charge ID.
    pub async fn mark_paid(
        db: &DatabaseConnection,
        ledger_entry_id: Uuid,
        provider_invoice_id: Option<String>,
    ) -> Result<()> {
        let entry = atlas_ledger_entry::Entity::find_by_id(ledger_entry_id)
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("ledger entry {ledger_entry_id} not found"))?;

        if entry.status == "paid" {
            tracing::debug!(%ledger_entry_id, "mark_paid: already paid, skipping");
            return Ok(());
        }

        let mut am: atlas_ledger_entry::ActiveModel = entry.into();
        am.status = Set("paid".to_owned());
        am.paid_at = Set(Some(Utc::now()));
        if let Some(pid) = provider_invoice_id {
            am.external_tx_id = Set(Some(pid));
        }
        am.update(db).await.map_err(|e| {
            anyhow!("mark_paid failed for {ledger_entry_id}: {e}")
        })?;
        tracing::info!(%ledger_entry_id, "ledger entry marked paid");
        Ok(())
    }

    /// Mark a ledger entry as `failed` (payment rejected by the provider).
    pub async fn mark_failed(
        db: &DatabaseConnection,
        ledger_entry_id: Uuid,
    ) -> Result<()> {
        let entry = atlas_ledger_entry::Entity::find_by_id(ledger_entry_id)
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("ledger entry {ledger_entry_id} not found"))?;

        if matches!(entry.status.as_str(), "paid" | "refunded" | "failed") {
            tracing::debug!(%ledger_entry_id, status = %entry.status, "mark_failed: already terminal, skipping");
            return Ok(());
        }

        let mut am: atlas_ledger_entry::ActiveModel = entry.into();
        am.status = Set("failed".to_owned());
        am.update(db).await.map_err(|e| anyhow!("mark_failed failed for {ledger_entry_id}: {e}"))?;
        tracing::warn!(%ledger_entry_id, "ledger entry marked failed");
        Ok(())
    }

    /// Mark a ledger entry as `refunded`.
    ///
    /// Idempotent — calling twice is a no-op if already refunded.
    pub async fn mark_refunded(
        db: &DatabaseConnection,
        ledger_entry_id: Uuid,
        reconciliation_note: Option<String>,
    ) -> Result<()> {
        let entry = atlas_ledger_entry::Entity::find_by_id(ledger_entry_id)
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("ledger entry {ledger_entry_id} not found"))?;

        if entry.status == "refunded" {
            tracing::debug!(%ledger_entry_id, "mark_refunded: already refunded, skipping");
            return Ok(());
        }

        let now = Utc::now();
        let mut am: atlas_ledger_entry::ActiveModel = entry.into();
        am.status = Set("refunded".to_owned());
        am.reconciled_at = Set(Some(now));
        if let Some(note) = reconciliation_note {
            am.reconciliation_note = Set(Some(note));
        }
        am.update(db).await.map_err(|e| anyhow!("mark_refunded failed for {ledger_entry_id}: {e}"))?;
        tracing::info!(%ledger_entry_id, "ledger entry marked refunded");
        Ok(())
    }

    // ── record_tx_id ──────────────────────────────────────────────────────────

    /// Set the Bitcoin txid on a ledger entry submitted by the tenant.
    ///
    /// Called by the tenant after broadcasting their BTC on-chain transaction.
    /// The mempool poll background job uses this txid to poll for confirmation.
    ///
    /// Validates that `txid` is exactly 64 lowercase hex characters.
    pub async fn record_tx_id(
        db: &DatabaseConnection,
        ledger_entry_id: Uuid,
        tenant_id: Uuid,
        txid: &str,
    ) -> Result<()> {
        // Validate txid format: 64 lowercase hex chars.
        if txid.len() != 64 || !txid.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(anyhow!(
                "invalid txid '{txid}': must be exactly 64 hex characters"
            ));
        }

        let entry = atlas_ledger_entry::Entity::find_by_id(ledger_entry_id)
            .filter(atlas_ledger_entry::Column::TenantId.eq(tenant_id))
            .filter(atlas_ledger_entry::Column::PaymentRail.eq("btc_onchain"))
            .one(db)
            .await?
            .ok_or_else(|| anyhow!(
                "BTC ledger entry {ledger_entry_id} not found for tenant {tenant_id}"
            ))?;

        let mut am: atlas_ledger_entry::ActiveModel = entry.into();
        am.external_tx_id = Set(Some(txid.to_owned()));
        am.status = Set("processing".to_owned()); // awaiting mempool confirmation
        am.update(db).await.map_err(|e| {
            anyhow!("PmLedgerService::record_tx_id failed for {ledger_entry_id}: {e}")
        })?;

        tracing::info!(
            ledger_entry_id = %ledger_entry_id, %tenant_id,
            txid,
            "PmLedgerService: BTC txid recorded, status → processing"
        );

        Ok(())
    }

    // ── list_btc_invoices ─────────────────────────────────────────────────────

    /// Return BTC on-chain ledger entries for a tenant (for the audit endpoint).
    pub async fn list_btc_invoices(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        limit: u64,
    ) -> Result<Vec<BtcInvoiceAudit>> {
        let entries = atlas_ledger_entry::Entity::find()
            .filter(atlas_ledger_entry::Column::TenantId.eq(tenant_id))
            .filter(atlas_ledger_entry::Column::PaymentRail.eq("btc_onchain"))
            .order_by_desc(atlas_ledger_entry::Column::CreatedAt)
            .limit(limit)
            .all(db)
            .await
            .map_err(|e| anyhow!("PmLedgerService::list_btc_invoices failed: {e}"))?;

        Ok(entries
            .into_iter()
            .map(|e| BtcInvoiceAudit {
                ledger_entry_id: e.id,
                gross_amount_cents: e.gross_amount_cents,
                currency: e.currency,
                status: e.status,
                external_tx_id: e.external_tx_id,
                due_date: e.due_date,
                paid_at: e.paid_at,
                created_at: e.created_at,
            })
            .collect())
    }
}
