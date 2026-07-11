//! G-27 scorecard trigger wiring — domain event → rating session.
//!
//! Opens rating sessions when an app-instance deployment's `trigger_event`
//! matches a domain lifecycle event (e.g. STR `post_checkout`).
//!
//! See: `docs/architecture/g27/g27_app_instance_runtime.md`

use anyhow::Result;
use chrono::Utc;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::entities::atlas_scorecard_template_deployment as deployments;
use crate::services::scorecard_service::ScorecardService;
use crate::types::scorecard::{DeploymentTriggerEvent, ScorecardEntityType, SessionType};

/// Result of opening sessions for a single trigger event.
#[derive(Debug, Clone)]
pub struct TriggerSessionOpened {
    pub deployment_id: Uuid,
    pub template_id: Uuid,
    pub scorecard_id: Uuid,
    pub session_id: Uuid,
}

/// Open rating sessions for all enabled deployments matching `trigger_event`
/// on `app_instance_id`.
///
/// For each matching deployment:
/// 1. `get_or_create` scorecard for `(template, subject_type, subject_id)`
/// 2. `open_session` with `session_type` + `context_entity_*`
///
/// Failures on individual deployments are logged and skipped (best-effort) so
/// the domain event (e.g. check-out) is never blocked by scorecard errors.
pub async fn open_sessions_for_trigger(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    app_instance_id: Uuid,
    trigger_event: DeploymentTriggerEvent,
    subject_entity_type: ScorecardEntityType,
    subject_entity_id: Uuid,
    rater_user_id: Uuid,
    session_type: SessionType,
    context_entity_type: Option<ScorecardEntityType>,
    context_entity_id: Option<Uuid>,
    session_label: Option<&str>,
) -> Result<Vec<TriggerSessionOpened>> {
    let trigger = trigger_event.to_string();
    let deps = deployments::Entity::find()
        .filter(deployments::Column::TenantId.eq(tenant_id))
        .filter(deployments::Column::AppInstanceId.eq(app_instance_id))
        .filter(deployments::Column::IsEnabled.eq(true))
        .filter(deployments::Column::TriggerEvent.eq(trigger.clone()))
        .all(db)
        .await?;

    let mut opened = Vec::with_capacity(deps.len());
    let subject_type = subject_entity_type.to_string();
    let context_type = context_entity_type.map(|t| t.to_string());
    let session_type_str = session_type.to_string();

    for dep in deps {
        let scorecard_id = match ScorecardService::get_or_create(
            db,
            tenant_id,
            dep.template_id,
            &subject_type,
            subject_entity_id,
        )
        .await
        {
            Ok(id) => id,
            Err(e) => {
                tracing::warn!(
                    %tenant_id,
                    template_id = %dep.template_id,
                    "scorecard_triggers: get_or_create failed (non-fatal): {e:#}"
                );
                continue;
            }
        };

        let session_id = match ScorecardService::open_session(
            db,
            scorecard_id,
            rater_user_id,
            tenant_id,
            Utc::now(),
            &session_type_str,
            context_type.as_deref(),
            context_entity_id,
            session_label,
            Some(app_instance_id),
        )
        .await
        {
            Ok(id) => id,
            Err(e) => {
                tracing::warn!(
                    %tenant_id,
                    %scorecard_id,
                    "scorecard_triggers: open_session failed (non-fatal): {e:#}"
                );
                continue;
            }
        };

        tracing::info!(
            %tenant_id,
            %app_instance_id,
            trigger_event = %trigger_event,
            template_id = %dep.template_id,
            %scorecard_id,
            %session_id,
            "scorecard_triggers: session opened"
        );

        // Best-effort: enqueue nudge evaluation for the rater (WS / NudgePrompt).
        if let Err(e) = enqueue_scorecard_nudge(
            db,
            tenant_id,
            dep.template_id,
            &subject_type,
            subject_entity_id,
            &trigger,
            rater_user_id,
            session_id,
            scorecard_id,
        )
        .await
        {
            tracing::warn!(
                %tenant_id,
                %session_id,
                "scorecard_triggers: nudge enqueue failed (non-fatal): {e:#}"
            );
        }

        opened.push(TriggerSessionOpened {
            deployment_id: dep.id,
            template_id: dep.template_id,
            scorecard_id,
            session_id,
        });
    }

    Ok(opened)
}

async fn enqueue_scorecard_nudge(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    template_id: Uuid,
    subject_entity_type: &str,
    subject_entity_id: Uuid,
    activity_type: &str,
    rater_user_id: Uuid,
    session_id: Uuid,
    scorecard_id: Uuid,
) -> Result<()> {
    use crate::entities::outbox_job;
    use crate::types::outbox::OutboxJobType;
    use chrono::Utc;
    use sea_orm::{ActiveModelTrait, Set};
    use serde_json::json;

    let job = outbox_job::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id),
        job_type: Set(OutboxJobType::EvaluateScorecardNudge.to_string()),
        payload: Set(json!({
            "template_id": template_id,
            "subject_entity_type": subject_entity_type,
            "subject_entity_id": subject_entity_id,
            "activity_type": activity_type,
            "rater_user_id": rater_user_id,
            "session_id": session_id,
            "scorecard_id": scorecard_id,
        })),
        status: Set("pending".to_owned()),
        attempts: Set(0),
        error_message: Set(None),
        locked_by: Set(None),
        locked_at: Set(None),
        run_at: Set(Utc::now()),
        created_at: Set(Utc::now()),
    };
    job.insert(db).await?;
    Ok(())
}

/// STR check-out → `post_checkout` sessions for the reserved asset.
///
/// Subject: `atlas_asset` / `reserved_asset_id` (matches AssetService scorecard provisioning).
/// Context: `atlas_reservation` / reservation id.
/// Session type: `stay`.
pub async fn on_str_checkout(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    app_instance_id: Uuid,
    reservation_id: Uuid,
    asset_id: Uuid,
    rater_user_id: Uuid,
) -> Result<Vec<TriggerSessionOpened>> {
    open_sessions_for_trigger(
        db,
        tenant_id,
        app_instance_id,
        DeploymentTriggerEvent::PostCheckout,
        ScorecardEntityType::AtlasAsset,
        asset_id,
        rater_user_id,
        SessionType::Stay,
        Some(ScorecardEntityType::AtlasReservation),
        Some(reservation_id),
        Some("Post-checkout stay rating"),
    )
    .await
}

/// Maintenance work-order complete → `case_resolved` sessions for the contractor.
///
/// Subject: `atlas_service_provider` / `assigned_service_provider_id`
///   (matches VendorService::onboard scorecard provisioning).
/// Context: `atlas_case` / case id.
/// Session type: `job`.
/// Rater: landlord/PM (`assigned_user_id`), not the vendor.
pub async fn on_case_resolved(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    app_instance_id: Uuid,
    case_id: Uuid,
    service_provider_id: Uuid,
    rater_user_id: Uuid,
) -> Result<Vec<TriggerSessionOpened>> {
    open_sessions_for_trigger(
        db,
        tenant_id,
        app_instance_id,
        DeploymentTriggerEvent::CaseResolved,
        ScorecardEntityType::AtlasServiceProvider,
        service_provider_id,
        rater_user_id,
        SessionType::Job,
        Some(ScorecardEntityType::AtlasCase),
        Some(case_id),
        Some("Post-job contractor rating"),
    )
    .await
}
