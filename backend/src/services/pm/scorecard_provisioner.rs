//! Folio — PM Scorecard Provisioner
//!
//! Seeds the 4 canonical PM G-27 scorecard templates for a new Folio tenant.
//!
//! Templates seeded (idempotent — ON CONFLICT DO NOTHING):
//!   1. STR Property Assessment   — entity_type = str_property,  scope = platform
//!   2. Rental Unit Quality       — entity_type = rental_unit,   scope = platform
//!   3. Contractor Performance    — entity_type = contractor,     scope = platform
//!   4. Lead Quality Assessment   — entity_type = wholesale_lead, scope = tenant
//!
//! Called from `FolioApp::provision()`.

use anyhow::Result;
use sea_orm::{DatabaseConnection, ConnectionTrait, Statement};
use uuid::Uuid;
use chrono::Utc;

use crate::types::pm::{ScorecardEntityType, TemplateScope};
use crate::types::scorecard::ScoringMethod;

struct PmTemplateSpec {
    name: &'static str,
    entity_type: ScorecardEntityType,
    scope: TemplateScope,
    description: &'static str,
    scoring_method: ScoringMethod,
}

const PM_TEMPLATES: &[PmTemplateSpec] = &[
    PmTemplateSpec {
        name: "STR Property Assessment",
        entity_type: ScorecardEntityType::StrProperty,
        scope: TemplateScope::Platform,
        description: "Rates a short-term rental property across cleanliness, amenities, location, communication, and STR compliance readiness.",
        scoring_method: ScoringMethod::WeightedMean,
    },
    PmTemplateSpec {
        name: "Rental Unit Quality",
        entity_type: ScorecardEntityType::RentalUnit,
        scope: TemplateScope::Platform,
        description: "Rates a long-term rental unit across condition, responsiveness, lease clarity, and condomínio transparency.",
        scoring_method: ScoringMethod::WeightedMean,
    },
    PmTemplateSpec {
        name: "Contractor Performance",
        entity_type: ScorecardEntityType::Contractor,
        scope: TemplateScope::Platform,
        description: "Rates a maintenance vendor across timeliness, workmanship quality, communication, and professionalism.",
        scoring_method: ScoringMethod::WeightedMean,
    },
    PmTemplateSpec {
        name: "Lead Quality Assessment",
        entity_type: ScorecardEntityType::WholesaleLead,
        scope: TemplateScope::Tenant, // Private per operator — excluded from cross-tenant pool
        description: "Rates a wholesale acquisition lead across motivation strength, ARV confidence, repair estimate accuracy, and negotiation leverage.",
        scoring_method: ScoringMethod::WeightedMean,
    },
];

/// Seed all 4 canonical PM scorecard templates for a tenant.
///
/// Idempotent: uses ON CONFLICT DO NOTHING on the template name + tenant_id.
/// Safe to call multiple times — repeated calls are no-ops.
pub async fn seed_pm_templates(
    db: &DatabaseConnection,
    tenant_id: Uuid,
) -> Result<()> {
    let now = Utc::now();

    for spec in PM_TEMPLATES {
        let template_id = Uuid::new_v4();

        db.execute(Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            INSERT INTO atlas_scorecard_templates (
                id, tenant_id, name, entity_type, description,
                scoring_method, default_scale_min, default_scale_max,
                min_entries_to_publish, is_published,
                cold_start_strategy, cold_start_saturation_threshold,
                calibration_minimum_entries, template_scope,
                created_at, updated_at
            )
            SELECT
                $1, $2, $3, $4, $5,
                $6, 1.0, 10.0,
                5, false,
                'suppress', 50,
                100, $7,
                $8, $8
            WHERE NOT EXISTS (
                SELECT 1 FROM atlas_scorecard_templates
                WHERE tenant_id = $2 AND name = $3
            )
            "#,
            vec![
                template_id.into(),
                tenant_id.into(),
                spec.name.into(),
                spec.entity_type.to_string().into(),
                spec.description.into(),
                spec.scoring_method.to_string().into(),
                spec.scope.to_string().into(),
                now.into(),
            ],
        ))
        .await
        .map_err(|e| anyhow::anyhow!(
            "seed_pm_templates: failed to seed '{}' for tenant {}: {e}",
            spec.name, tenant_id
        ))?;

        tracing::debug!(
            %tenant_id,
            template_name = spec.name,
            entity_type = %spec.entity_type,
            scope = %spec.scope,
            "scorecard_provisioner: seeded PM template (or already exists)"
        );
    }

    tracing::info!(%tenant_id, "scorecard_provisioner: 4 PM templates seeded");
    Ok(())
}

/// Retrieve a provisioned PM template by name for a tenant.
///
/// Used by Phase 2 auto-provisioning hooks in AssetService/VendorService/WholesaleService.
pub async fn get_pm_template(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    template_name: &str,
) -> Result<crate::entities::atlas_scorecard_template::Model> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    crate::entities::atlas_scorecard_template::Entity::find()
        .filter(crate::entities::atlas_scorecard_template::Column::TenantId.eq(tenant_id))
        .filter(crate::entities::atlas_scorecard_template::Column::Name.eq(template_name))
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!(
            "PM template '{template_name}' not found for tenant {tenant_id}. \
             Was FolioApp::provision() called?"
        ))
}
