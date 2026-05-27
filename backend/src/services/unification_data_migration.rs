//! Data migration logic for retiring legacy CRM entities into the unified Platform Generics model.
//!
//! Current status (as of analysis):
//! - Very low data volume in legacy tables.
//! - Only "buildwithruud" tenant in atlas_dev has meaningful records (3 contacts + 1 lead + 11 activities).
//! - All other tenants have 0 rows in customer/contact/deal/case/lead/activity for non-platform use.

use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set};
use uuid::Uuid;
use chrono::Utc;
use crate::entities::{atlas_account, atlas_contact};

/// Migrates legacy CRM data for a specific tenant into the new unified model.
///
/// This is intentionally conservative and focused on the small amount of real data we have.
pub async fn migrate_tenant_legacy_crm(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    tenant_name: &str,
) -> Result<MigrationReport, String> {
    let mut report = MigrationReport::default();

    // For now we only have data in dev for "buildwithruud".
    // In a real run we would query the legacy tables (customer, contact, lead, activity, etc.).

    // Example: Create an Account for the tenant itself if it doesn't exist
    let existing_accounts = atlas_account::Entity::find()
        .filter(atlas_account::Column::TenantId.eq(tenant_id))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    if existing_accounts.is_empty() {
        let account = atlas_account::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            account_type: Set("organization".to_string()),
            name: Set(format!("{} (Migrated)", tenant_name)),
            status: Set("active".to_string()),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };
        account.insert(db).await.map_err(|e| e.to_string())?;
        report.accounts_created += 1;
    }

    // TODO: Implement actual migration of:
    // - Legacy contacts -> atlas_contacts + atlas_accounts (individual)
    // - Legacy leads -> atlas_applications or early atlas_opportunities
    // - Legacy activities -> atlas_cases or timeline events
    //
    // Because the data volume is tiny, this can be done with direct inserts + logging.

    report.notes.push(format!("Migration stub executed for tenant {}", tenant_name));
    Ok(report)
}

#[derive(Default, Debug)]
pub struct MigrationReport {
    pub accounts_created: u32,
    pub contacts_created: u32,
    pub opportunities_created: u32,
    pub cases_created: u32,
    pub notes: Vec<String>,
}
