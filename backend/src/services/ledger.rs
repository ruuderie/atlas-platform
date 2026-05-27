// Temporary skeleton for the new unified ledger service.
// This will eventually absorb logic from legacy services like lead_billing.rs.

use sea_orm::DatabaseConnection;
use uuid::Uuid;
use anyhow::Result;

/// Records a charge related to acquiring a lead.
/// This is an example of moving legacy "lead billing" logic onto the generic
/// atlas_ledger_entries system (GENERIC-03).
pub async fn record_lead_purchase(
    _db: &DatabaseConnection,
    tenant_id: Uuid,
    account_id: Uuid,      // Now an atlas_accounts.id
    lead_id: Uuid,
    amount_cents: i64,
) -> Result<Uuid> {
    // TODO: Implement using atlas_ledger_entries + optional splits
    // For now this is a stub so the shape of the new service is visible.

    tracing::info!(
        "TODO: Record lead purchase via unified ledger. tenant={}, account={}, lead={}, amount={}",
        tenant_id, account_id, lead_id, amount_cents
    );

    Ok(Uuid::new_v4()) // Placeholder
}
