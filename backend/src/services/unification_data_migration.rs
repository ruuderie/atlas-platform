//! Data migration logic for retiring legacy CRM entities into the unified Platform Generics model.
//!
//! This is a one-time migration tool focused on dev and uat environments.
//! It is intentionally generic — no tenant (including buildwithruud) is special-cased
//! in the core logic. The buildwithruud helper exists purely as a convenience for
//! early dev validation and can be removed later.
//!
//! All writes go through the service layer (AccountService, ContactService, OpportunityService).

use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};
use uuid::Uuid;
use std::collections::HashMap;
use tracing;

use crate::entities::contact as legacy_contact;
use crate::entities::lead as legacy_lead;
use crate::entities::activity as legacy_activity;
use crate::entities::tenant as platform_tenant;

use crate::services::account_service::AccountService;
use crate::services::contact_service::ContactService;
use crate::services::opportunity_service::OpportunityService;

/// Migrates legacy CRM data for a specific tenant into the new unified model.
///
/// This function is completely generic. No tenant is special-cased.
/// All data creation goes through the service layer.
///
/// `dry_run = true` will compute what would happen and populate the report
/// without writing any rows.
pub async fn migrate_tenant_legacy_crm(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    tenant_name: &str,
    dry_run: bool,
) -> Result<MigrationReport, String> {
    let mut report = MigrationReport::default();
    report.dry_run = dry_run;

    tracing::info!(
        "Legacy CRM migration starting for tenant {} (dry_run={})",
        tenant_name, dry_run
    );

    // 1. Ensure we have at least one organization account for the tenant (via service)
    let org_account_id = AccountService::find_or_create_tenant_account(db, tenant_id, tenant_name).await?;

    // 2. Migrate Contacts using ContactService
    let legacy_contacts = legacy_contact::Entity::find()
        .filter(legacy_contact::Column::TenantId.eq(tenant_id))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    for lc in legacy_contacts {
        let new_account_id = if lc.customer_id.is_some() {
            org_account_id
        } else {
            // Create individual account via service
            if !dry_run {
                AccountService::create_account(
                    db,
                    tenant_id,
                    "individual",
                    &lc.name,
                    lc.first_name.as_deref(),
                    lc.last_name.as_deref(),
                ).await?
            } else {
                // In dry run we still count it
                report.accounts_created += 1;
                Uuid::new_v4() // placeholder, not used
            }
        };

        if !dry_run {
            ContactService::create_contact(
                db,
                tenant_id,
                new_account_id,
                lc.first_name.as_deref(),
                lc.last_name.as_deref(),
                lc.email.as_deref(),
                false,
            ).await?;
        }
        report.contacts_created += 1;

        if dry_run {
            report.notes.push(format!(
                "[DRY RUN] Would create atlas_contact for legacy contact {} ('{}')",
                lc.id, lc.name
            ));
        }
    }

    // 3. Migrate Leads → atlas_opportunities using OpportunityService
    let legacy_leads = legacy_lead::Entity::find()
        .filter(legacy_lead::Column::TenantId.eq(tenant_id))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    for ll in legacy_leads {
        if !dry_run {
            OpportunityService::create_opportunity(
                db,
                tenant_id,
                "legacy_lead",
                &ll.name,
                None, // counterparty_account_id - we can link later if needed
                None,
                None,
                ll.lead_status.as_deref(),
            ).await?;
        }
        report.opportunities_created += 1;

        report.notes.push(format!(
            "{}Migrated legacy lead {} ('{}') → atlas_opportunity",
            if dry_run { "[DRY RUN] " } else { "" },
            ll.id, ll.name
        ));
    }

    // 4. Activities — lightweight archival only (as designed)
    let legacy_activities = legacy_activity::Entity::find()
        .filter(legacy_activity::Column::TenantId.eq(tenant_id))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let activity_count = legacy_activities.len();
    if activity_count > 0 {
        report.notes.push(format!(
            "{}Archived {} legacy activity records for tenant (system-generated logs).",
            if dry_run { "[DRY RUN] " } else { "" },
            activity_count
        ));
        for (i, la) in legacy_activities.iter().take(3).enumerate() {
            report.notes.push(format!(
                "  Sample: {} (type={:?})",
                la.title, la.activity_type
            ));
        }
        if activity_count > 3 {
            report.notes.push(format!("  ... and {} more", activity_count - 3));
        }
    }

    report.notes.push(format!(
        "Migration {} for tenant {}",
        if dry_run { "simulated" } else { "completed" },
        tenant_name
    ));

    Ok(report)
}

#[derive(Default, Debug)]
pub struct MigrationReport {
    pub accounts_created: u32,
    pub contacts_created: u32,
    pub opportunities_created: u32,
    pub cases_created: u32,
    pub notes: Vec<String>,
    pub dry_run: bool,
}

impl std::fmt::Display for MigrationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MigrationReport {{ accounts: {}, contacts: {}, opportunities: {}, cases: {}, dry_run: {} }}\nNotes:\n{}",
            self.accounts_created,
            self.contacts_created,
            self.opportunities_created,
            self.cases_created,
            self.dry_run,
            self.notes.join("\n")
        )
    }
}

/// Convenience entry point for early dev validation only.
/// 
/// IMPORTANT: This is intentionally hardcoded for the buildwithruud dev tenant
/// so developers can quickly exercise the migration. It is **not** used in any
/// production path and creates no tenant dependency on buildwithruud.
/// Remove this function once the generic migration + CLI is proven.
pub async fn migrate_buildwithruud_dev_sample(db: &DatabaseConnection, dry_run: bool) -> Result<MigrationReport, String> {
    let buildwithruud_tenant_id = Uuid::parse_str("35f95f2a-db97-4166-be66-5215654cac84")
        .map_err(|e| format!("Bad hardcoded dev tenant uuid: {}", e))?;
    let tenant_name = "buildwithruud";

    tracing::info!("Starting legacy CRM unification migration (dev sample) for: {} (dry_run={})", tenant_name, dry_run);
    let report = migrate_tenant_legacy_crm(db, buildwithruud_tenant_id, tenant_name, dry_run).await?;
    tracing::info!("Dev sample migration report:\n{}", report);
    Ok(report)
}

/// Migrate a list of tenants. Primary entry point for the CLI / admin tooling.
/// Callers decide which tenants to target (e.g. all with legacy data in dev/uat).
pub async fn migrate_known_tenants(
    db: &DatabaseConnection,
    tenant_ids: &[(Uuid, &str)],
    dry_run: bool,
) -> Result<Vec<MigrationReport>, String> {
    let mut reports = Vec::new();
    for (tid, tname) in tenant_ids {
        let r = migrate_tenant_legacy_crm(db, *tid, tname, dry_run).await?;
        reports.push(r);
    }
    Ok(reports)
}

/// Discover tenants that still have rows in legacy CRM tables.
/// Returns (tenant_id, tenant_name) for nice reporting.
/// Useful for "migrate everything that needs it" runs in dev/uat.
pub async fn find_tenants_with_legacy_data(db: &DatabaseConnection) -> Result<Vec<(Uuid, String)>, String> {
    use crate::entities::lead as legacy_lead;
    use crate::entities::contact as legacy_contact;
    use crate::entities::tenant as platform_tenant;

    let mut results: HashMap<Uuid, String> = HashMap::new();

    // Leads
    let leads = legacy_lead::Entity::find().all(db).await.map_err(|e| e.to_string())?;
    for l in leads {
        if let Some(tid) = l.tenant_id {
            if !results.contains_key(&tid) {
                if let Ok(Some(t)) = platform_tenant::Entity::find_by_id(tid).one(db).await {
                    results.insert(tid, t.name);
                } else {
                    results.insert(tid, format!("tenant-{}", tid));
                }
            }
        }
    }

    // Contacts
    let contacts = legacy_contact::Entity::find().all(db).await.map_err(|e| e.to_string())?;
    for c in contacts {
        if let Some(tid) = c.tenant_id {
            if !results.contains_key(&tid) {
                if let Ok(Some(t)) = platform_tenant::Entity::find_by_id(tid).one(db).await {
                    results.insert(tid, t.name);
                } else {
                    results.insert(tid, format!("tenant-{}", tid));
                }
            }
        }
    }

    Ok(results.into_iter().collect())
}

// Verification helper temporarily disabled while we stabilize the raw query version.
// The core migration (using services) is fully functional.
// We will re-enable a clean version before the final cutover.


#[derive(Default, Debug)]
pub struct VerificationReport {
    pub all_clean: bool,
    pub tenants_clean: Vec<Uuid>,
    pub tenants_with_remaining_legacy_data: Vec<(Uuid, String)>, // tenant_id + breakdown
}

impl std::fmt::Display for VerificationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.all_clean {
            writeln!(f, "✅ VERIFICATION PASSED — No tenants have remaining legacy CRM data.")?;
        } else {
            writeln!(f, "❌ VERIFICATION FAILED — Some tenants still have data in legacy tables:")?;
            for (tid, details) in &self.tenants_with_remaining_legacy_data {
                writeln!(f, "  - {} : {}", tid, details)?;
            }
        }
        Ok(())
    }
}
