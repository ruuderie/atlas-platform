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
use crate::types::scorecard::{DeploymentTriggerEvent, ScoringMethod};

struct PmTemplateSpec {
    name: &'static str,
    entity_type: ScorecardEntityType,
    scope: TemplateScope,
    description: &'static str,
    scoring_method: ScoringMethod,
    default_trigger_event: DeploymentTriggerEvent,
}

const PM_TEMPLATES: &[PmTemplateSpec] = &[
    PmTemplateSpec {
        name: "STR Property Assessment",
        entity_type: ScorecardEntityType::StrProperty,
        scope: TemplateScope::Platform,
        description: "Rates a short-term rental property across cleanliness, amenities, location, communication, and STR compliance readiness.",
        scoring_method: ScoringMethod::WeightedMean,
        default_trigger_event: DeploymentTriggerEvent::PostCheckout,
    },
    PmTemplateSpec {
        name: "Rental Unit Quality",
        entity_type: ScorecardEntityType::RentalUnit,
        scope: TemplateScope::Platform,
        description: "Rates a long-term rental unit across condition, responsiveness, lease clarity, and condomínio transparency.",
        scoring_method: ScoringMethod::WeightedMean,
        default_trigger_event: DeploymentTriggerEvent::Manual,
    },
    PmTemplateSpec {
        name: "Contractor Performance",
        entity_type: ScorecardEntityType::Contractor,
        scope: TemplateScope::Platform,
        description: "Rates a maintenance vendor across timeliness, workmanship quality, communication, and professionalism.",
        scoring_method: ScoringMethod::WeightedMean,
        default_trigger_event: DeploymentTriggerEvent::CaseResolved,
    },
    PmTemplateSpec {
        name: "Lead Quality Assessment",
        entity_type: ScorecardEntityType::WholesaleLead,
        scope: TemplateScope::Tenant, // Private per operator — excluded from cross-tenant pool
        description: "Rates a wholesale acquisition lead across motivation strength, ARV confidence, repair estimate accuracy, and negotiation leverage.",
        scoring_method: ScoringMethod::WeightedMean,
        default_trigger_event: DeploymentTriggerEvent::Manual,
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

/// Upsert deployment rows for every PM template belonging to `tenant_id` onto
/// `app_instance_id`, with `is_enabled=true` and product-default `trigger_event`
/// (e.g. STR → `post_checkout`, Contractor → `case_resolved`, else `manual`).
///
/// Idempotent on unique `(template_id, app_instance_id)`. Re-running updates
/// `is_enabled=true` and refreshes `trigger_event` to the product default so
/// existing tenants pick up new trigger wiring on reprovision.
pub async fn deploy_templates_for_instance(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    app_instance_id: Uuid,
) -> Result<u32> {
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
    use crate::entities::atlas_scorecard_template as templates;
    use crate::entities::atlas_scorecard_template_deployment as deployments;

    let names: Vec<&str> = PM_TEMPLATES.iter().map(|s| s.name).collect();
    let seeded = templates::Entity::find()
        .filter(templates::Column::TenantId.eq(tenant_id))
        .filter(templates::Column::Name.is_in(names))
        .all(db)
        .await?;

    let mut touched = 0u32;
    for t in seeded {
        let trigger = PM_TEMPLATES
            .iter()
            .find(|s| s.name == t.name)
            .map(|s| s.default_trigger_event)
            .unwrap_or(DeploymentTriggerEvent::Manual);

        let existing = deployments::Entity::find()
            .filter(deployments::Column::TemplateId.eq(t.id))
            .filter(deployments::Column::AppInstanceId.eq(app_instance_id))
            .one(db)
            .await?;

        if let Some(row) = existing {
            let mut am: deployments::ActiveModel = row.into();
            am.is_enabled = Set(true);
            am.trigger_event = Set(trigger.to_string());
            am.update(db)
                .await
                .map_err(|e| anyhow::anyhow!(
                    "deploy_templates_for_instance: update failed for template {} on instance {}: {e}",
                    t.id, app_instance_id
                ))?;
            touched += 1;
            continue;
        }

        deployments::ActiveModel {
            id: Set(Uuid::new_v4()),
            template_id: Set(t.id),
            app_instance_id: Set(app_instance_id),
            tenant_id: Set(tenant_id),
            is_enabled: Set(true),
            trigger_event: Set(trigger.to_string()),
            trigger_context_entity_type: Set(None),
            created_at: Set(Utc::now()),
        }
        .insert(db)
        .await
        .map_err(|e| anyhow::anyhow!(
            "deploy_templates_for_instance: insert failed for template {} on instance {}: {e}",
            t.id, app_instance_id
        ))?;
        touched += 1;
    }

    tracing::info!(
        %tenant_id,
        %app_instance_id,
        touched,
        "scorecard_provisioner: deployments upserted for PM templates"
    );
    Ok(touched)
}

/// Seed PM templates and auto-deploy them onto the Folio app instance.
///
/// Looks up the tenant's `property_management` app_instance. If none exists yet,
/// templates are still seeded (deployments can be created later).
pub async fn seed_and_deploy_for_folio(
    db: &DatabaseConnection,
    tenant_id: Uuid,
) -> Result<()> {
    seed_pm_templates(db, tenant_id).await?;

    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
    let instance = crate::entities::app_instance::Entity::find()
        .filter(crate::entities::app_instance::Column::TenantId.eq(tenant_id))
        .filter(crate::entities::app_instance::Column::AppType.eq("property_management"))
        .order_by_asc(crate::entities::app_instance::Column::CreatedAt)
        .one(db)
        .await?;

    if let Some(inst) = instance {
        deploy_templates_for_instance(db, tenant_id, inst.id).await?;
    } else {
        tracing::warn!(
            %tenant_id,
            "scorecard_provisioner: no property_management app_instance — templates seeded without deployments"
        );
    }

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
