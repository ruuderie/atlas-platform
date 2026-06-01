use anyhow::Result;
use uuid::Uuid;
use tracing::{info, error};
use sea_orm::DatabaseConnection;

use crate::services::ledger;

/// DEPRECATED: Legacy lead billing facade.
/// 
/// This now delegates to the unified `ledger::record_lead_purchase` (GENERIC-03).
/// All new code should call ledger directly or via a higher-level acquisition service.
/// The old `lead_charge` table is no longer written to after unification cutover.
///
/// The single caller in handlers/leads.rs will be updated in the next vertical slice
/// of the legacy handler migration.
#[deprecated(since = "2026-06-01", note = "Use services::ledger::record_lead_purchase instead")]
pub async fn charge_for_lead(
    db: &DatabaseConnection,
    tenant_id: Uuid,           // NEW: required for unified path
    account_id: Uuid,          // atlas_accounts.id (or legacy during transition)
    lead_id: Uuid,
    _stripe_customer_id: Option<String>, // kept for compat; real charging happens via payment rails in G-03
) -> Result<()> {
    // Amount is currently hardcoded at $50 CPL in the old flow.
    // In real usage this would come from tenant settings, lead source config, or a pricing engine.
    const DEFAULT_LEAD_CHARGE_CENTS: i64 = 5000;

    info!(
        "[DEPRECATED lead_billing] Forwarding lead charge to unified ledger. tenant={}, account={}, lead={}",
        tenant_id, account_id, lead_id
    );

    match ledger::record_lead_purchase(db, tenant_id, account_id, lead_id, DEFAULT_LEAD_CHARGE_CENTS, Some("stripe")).await {
        Ok(entry_id) => {
            info!("Lead purchase recorded as ledger entry {}", entry_id);
            Ok(())
        }
        Err(e) => {
            error!("Failed to record lead purchase in unified ledger: {:?}", e);
            // Do not fail the lead ingestion path for billing issues (best effort, same as before)
            Ok(())
        }
    }
}
