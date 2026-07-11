// Unified Ledger Service (GENERIC-03 + unification target for legacy lead_billing).
// Records tenant-scoped financial events (lead acquisition charges, rent, payouts, etc.)
// using atlas_ledger_entries + atlas_ledger_splits. Pluggable rails via payment_credentials.

use anyhow::Result;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use uuid::Uuid;

use crate::entities::atlas_ledger_entry;

/// Records a charge related to acquiring a lead (or other billable CRM event).
/// This replaces the legacy lead_billing + lead_charge pattern.
///
/// During the transition period the billable_entity_type can be "lead" or "legacy_lead"
/// so that old and new flows can coexist until handlers are fully updated.
pub async fn record_lead_purchase(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    account_id: Uuid, // atlas_accounts.id (payer / billable party)
    lead_id: Uuid,
    amount_cents: i64,
    payment_rail: Option<&str>,
) -> Result<Uuid> {
    let now = Utc::now();
    let rail = payment_rail.unwrap_or("stripe").to_string();

    let ledger_entry = atlas_ledger_entry::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id),
        billable_entity_type: Set("lead".to_string()), // or "opportunity" / "acquisition" post-cutover
        billable_entity_id: Set(lead_id),
        payer_user_id: Set(None),
        payer_email: Set(None),
        gross_amount_cents: Set(amount_cents),
        fee_amount_cents: Set(0),
        net_amount_cents: Set(amount_cents),
        currency: Set("USD".to_string()),
        payment_rail: Set(Some(rail)),
        external_tx_id: Set(None),
        receipt_attachment_id: Set(None),
        status: Set("succeeded".to_string()),
        due_date: Set(None),
        paid_at: Set(Some(now)),
        verified_by_user_id: Set(None),
        verified_at: Set(None),
        reconciled_at: Set(None),
        reconciliation_note: Set(None),
        created_at: Set(now),
    };

    let inserted = ledger_entry.insert(db).await?;
    let entry_id = inserted.id;

    tracing::info!(
        "Recorded lead purchase via unified ledger: entry_id={}, tenant={}, account={}, lead={}, amount_cents={}, rail={}",
        entry_id,
        tenant_id,
        account_id,
        lead_id,
        amount_cents,
        payment_rail.unwrap_or("stripe")
    );

    // Future: create atlas_ledger_split rows here for MOR / platform fee / recipient splits
    // e.g. 70% to sales agent, 30% platform using the generic split model.

    Ok(entry_id)
}
