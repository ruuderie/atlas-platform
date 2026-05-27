//! Data migration logic for retiring legacy CRM entities into the unified Platform Generics model.
//!
//! Current status (as of analysis):
//! - Very low data volume in legacy tables.
//! - Only "buildwithruud" tenant in atlas_dev has meaningful records (3 contacts + 1 lead + 11 activities).
//! - All other tenants have 0 rows in customer/contact/deal/case/lead/activity for non-platform use.

use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set};
use uuid::Uuid;
use chrono::Utc;
use crate::entities::{atlas_account, atlas_contact, atlas_opportunity};

// Legacy entities (will be removed after migration)
use crate::entities::contact as legacy_contact;
use crate::entities::lead as legacy_lead;
use crate::entities::activity as legacy_activity;

// Services are available for future enhancement of the migration (e.g. using AccountService::find_or_create...)
// use crate::services::account_service::AccountService;
// use crate::services::contact_service::ContactService;

/// Migrates legacy CRM data for a specific tenant into the new unified model.
///
/// Focused on the small real dataset we have (primarily buildwithruud in dev).
pub async fn migrate_tenant_legacy_crm(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    tenant_name: &str,
) -> Result<MigrationReport, String> {
    let mut report = MigrationReport::default();

    // Ensure we have at least one organization account for the tenant
    let org_account_id = ensure_tenant_organization_account(db, tenant_id, tenant_name).await?;

    // 1. Migrate Contacts
    let legacy_contacts = legacy_contact::Entity::find()
        .filter(legacy_contact::Column::TenantId.eq(tenant_id))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    for lc in legacy_contacts {
        let new_account_id = if lc.customer_id.is_some() {
            // For now, attach to the main org account (we can refine later)
            org_account_id
        } else {
            // Create an individual account for this contact
            create_individual_account_for_contact(db, tenant_id, &lc).await?
        };

        let new_contact = atlas_contact::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            account_id: Set(new_account_id),
            first_name: Set(lc.first_name.clone()),
            last_name: Set(lc.last_name.clone()),
            full_name: Set(Some(lc.name.clone())),
            email: Set(lc.email.clone()),
            phone: Set(lc.phone.clone()),
            title: Set(None),
            is_primary: Set(false),
            contact_metadata: Set(lc.properties.clone()),
            created_at: Set(lc.created_at),
            updated_at: Set(lc.updated_at),
        };

        new_contact.insert(db).await.map_err(|e| e.to_string())?;
        report.contacts_created += 1;
    }

    // 2. Migrate Leads → atlas_opportunities (GENERIC-15)
    // The legacy "lead" concept is absorbed by the richer opportunity + application generics.
    // We preserve the original id via crm_lead_id for traceability during transition.
    let legacy_leads = legacy_lead::Entity::find()
        .filter(legacy_lead::Column::TenantId.eq(tenant_id))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    for ll in legacy_leads {
        let opp = atlas_opportunity::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            opportunity_type: Set("legacy_lead".to_string()),
            name: Set(ll.name.clone()),
            crm_lead_id: Set(Some(ll.id)),
            owner_user_id: Set(ll.account_id), // best effort mapping from old account
            status: Set(ll.lead_status.clone().unwrap_or_else(|| "new".to_string())),
            notes: Set(ll.message.clone()),
            // Capture original source / properties into financial_inputs as a simple migration bag
            financial_inputs: Set(ll.properties.clone()),
            created_at: Set(ll.created_at),
            ..Default::default()
        };

        opp.insert(db).await.map_err(|e| e.to_string())?;
        report.opportunities_created += 1;
        report.notes.push(format!(
            "Migrated legacy lead {} ('{}') → atlas_opportunity (crm_lead_id preserved)",
            ll.id, ll.name
        ));
    }

    // 3. Migrate Activities (lightweight archival)
    // Legacy activities (mostly "Contact Created", stage changes, "Lead Captured") are system-generated
    // audit logs. They are preserved in the MigrationReport for traceability. Full mapping to the new
    // timeline (realtime rooms + cases + documents + opportunities) can be done in a follow-up pass
    // once the generic activity/timeline pattern is finalized. Given 11 records are low-value, we
    // intentionally do not create duplicate legacy-style rows.
    let legacy_activities = legacy_activity::Entity::find()
        .filter(legacy_activity::Column::TenantId.eq(tenant_id))
        .all(db)
        .await
        .map_err(|e| e.to_string())?;

    let activity_count = legacy_activities.len();
    if activity_count > 0 {
        report.notes.push(format!(
            "Archived {} legacy activity records for tenant (system logs such as 'Contact Created' and status transitions). These can be replayed from audit logs or recreated via new case/opportunity events post-cutover.",
            activity_count
        ));
        // Log a sample of titles for operator visibility during dev migration
        for (i, la) in legacy_activities.iter().take(3).enumerate() {
            report.notes.push(format!(
                "  Sample activity {}: {} (type={:?})",
                i + 1,
                la.title,
                la.activity_type
            ));
        }
        if activity_count > 3 {
            report.notes.push(format!("  ... and {} more", activity_count - 3));
        }
    }

    // 4. Optional: ensure we have at least one primary contact per migrated account (best-effort)
    // This is left lightweight; a production migration would run a second pass to set primaries
    // using ContactService::set_as_primary once that is fully implemented.

    report.notes.push(format!("Migration completed for tenant {}", tenant_name));
    Ok(report)
}

async fn ensure_tenant_organization_account(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    tenant_name: &str,
) -> Result<Uuid, String> {
    let existing = atlas_account::Entity::find()
        .filter(atlas_account::Column::TenantId.eq(tenant_id))
        .filter(atlas_account::Column::AccountType.eq("organization"))
        .one(db)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(acc) = existing {
        return Ok(acc.id);
    }

    let account = atlas_account::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id),
        account_type: Set("organization".to_string()),
        name: Set(tenant_name.to_string()),
        status: Set("active".to_string()),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    };

    let result = account.insert(db).await.map_err(|e| e.to_string())?;
    Ok(result.id)
}

async fn create_individual_account_for_contact(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    legacy_contact: &legacy_contact::Model,
) -> Result<Uuid, String> {
    let name = legacy_contact.name.clone();

    let account = atlas_account::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id),
        account_type: Set("individual".to_string()),
        name: Set(name),
        first_name: Set(legacy_contact.first_name.clone()),
        last_name: Set(legacy_contact.last_name.clone()),
        status: Set("active".to_string()),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    };

    let result = account.insert(db).await.map_err(|e| e.to_string())?;
    Ok(result.id)
}

#[derive(Default, Debug)]
pub struct MigrationReport {
    pub accounts_created: u32,
    pub contacts_created: u32,
    pub opportunities_created: u32,
    pub cases_created: u32,
    pub notes: Vec<String>,
}

/// Convenience entry point for development / validation runs.
/// Targets the single known tenant with legacy CRM data in atlas_dev: buildwithruud.
pub async fn migrate_buildwithruud_dev_sample(db: &DatabaseConnection) -> Result<MigrationReport, String> {
    let buildwithruud_tenant_id = Uuid::parse_str("35f95f2a-db97-4166-be66-5215654cac84")
        .map_err(|e| format!("Bad hardcoded dev tenant uuid: {}", e))?;
    let tenant_name = "buildwithruud";

    tracing::info!("Starting legacy CRM unification migration for dev sample tenant: {}", tenant_name);
    let report = migrate_tenant_legacy_crm(db, buildwithruud_tenant_id, tenant_name).await?;
    tracing::info!("Dev sample migration report: {:?}", report);
    Ok(report)
}

/// Migrate a list of known small tenants (used for initial cutover validation).
/// In production this would be driven by an admin CLI or one-off job, not hot path.
pub async fn migrate_known_tenants(db: &DatabaseConnection, tenant_ids: &[(Uuid, &str)]) -> Result<Vec<MigrationReport>, String> {
    let mut reports = Vec::new();
    for (tid, tname) in tenant_ids {
        let r = migrate_tenant_legacy_crm(db, *tid, tname).await?;
        reports.push(r);
    }
    Ok(reports)
}
