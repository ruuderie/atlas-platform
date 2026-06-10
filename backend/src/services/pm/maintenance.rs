//! Folio — Maintenance Service (PM wrapper over G-13 `atlas_cases`)
//!
//! Emergency bypass routing, vendor dispatch, WebSocket real-time threading.
//!
//! # Entity field map (`atlas_cases`)
//!   `asset_id`                    → the property/unit/appliance/system the case is for
//!   `reported_by_user_id`         → tenant who filed (not `reporter_user_id`)
//!   `assigned_user_id`            → landlord/PM assignee (not `assigned_to_user_id`)
//!   `assigned_service_provider_id`→ vendor pre-assigned for inspections
//!   `priority`                    → "emergency" | "routine" (required string)
//!   `scheduled_at`                → DateTime of scheduled inspection (NULL for reactive tickets)
//!   `completed_at`                → DateTime inspection was signed off
//!   `case_metadata`               → JSONB: category, preferred_trade, emergency flag, findings
//!   No `entity_type`/`entity_id` columns — `asset_id` is the direct FK
//!   No `updated_at` column on this entity

use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{atlas_asset, atlas_case};
use crate::types::pm::{MaintenanceCategory, PmCaseType};

// ── Reactive maintenance ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMaintenanceTicketInput {
    pub asset_id: Uuid,
    pub reported_by_user_id: Uuid,
    pub category: MaintenanceCategory,
    pub description: String,
    pub is_emergency: bool,
    /// R2 key for voice recording submitted via Maintenance Wizard Step 2.
    pub voice_note_r2_key: Option<String>,
}

// ── Scheduled inspections ─────────────────────────────────────────────────────

/// Input for scheduling a proactive inspection on any lifecycle-tracked asset
/// (appliance, building system, unit, or any atlas_assets row).
#[derive(Debug, Deserialize)]
pub struct ScheduleInspectionInput {
    /// The asset being inspected (appliance, building system, unit, etc.).
    pub asset_id: Uuid,
    /// Human-readable subject e.g. "Annual elevator inspection", "Boiler service".
    pub subject: String,
    pub notes: Option<String>,
    /// When the inspection is scheduled. Stored in `atlas_cases.scheduled_at`.
    pub scheduled_at: DateTime<Utc>,
    /// Vendor pre-assigned to perform the inspection.
    pub service_provider_id: Option<Uuid>,
    /// Landlord/PM user who will oversee / sign off.
    pub assigned_user_id: Option<Uuid>,
    pub estimated_cost_cents: Option<i64>,
}

/// Input for completing an inspection and rolling the asset lifecycle forward.
#[derive(Debug, Deserialize)]
pub struct CompleteInspectionInput {
    pub case_id: Uuid,
    /// Findings / technician notes recorded during the inspection.
    pub findings: String,
    /// Updated asset condition post-inspection.
    pub condition_after: Option<String>,
    /// Next inspection due date. Written to `atlas_assets.scheduled_service_date`.
    /// If None, clears the scheduled service date (no next schedule set).
    pub next_inspection_date: Option<NaiveDate>,
    /// Updated cert/warranty expiry (e.g. elevator cert renewed after inspection).
    /// R2 object keys for inspection report files (photos, PDFs, checklists).
    /// Each key is registered in `atlas_documents` as `PmDocumentType::InspectionReport`
    /// linked to the case, so owners and landlords can view them from the Vault.
    /// Files must already be uploaded to R2 before calling this endpoint.
    #[serde(default)]
    pub attachment_r2_keys: Vec<String>,
    pub updated_expiry_date: Option<NaiveDate>,
    pub actual_cost_cents: Option<i64>,
}

/// Inspection case summary.
#[derive(Debug, Serialize)]
pub struct InspectionDetail {
    pub id: Uuid,
    pub asset_id: Option<Uuid>,
    pub subject: String,
    pub status: String,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub service_provider_id: Option<Uuid>,
    pub assigned_user_id: Option<Uuid>,
    pub estimated_cost_cents: Option<i64>,
    pub actual_cost_cents: Option<i64>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

// ── MaintenanceService ────────────────────────────────────────────────────────

pub struct MaintenanceService;

impl MaintenanceService {
    /// Create a reactive maintenance case in `atlas_cases`.
    ///
    /// Emergency tickets skip standard scheduling and are queued for
    /// immediate dispatch via the landlord emergency queue.
    /// The preferred vendor trade is derived from `category.preferred_trade()`.
    pub async fn create_ticket(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        input: CreateMaintenanceTicketInput,
    ) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let priority = if input.is_emergency { "emergency" } else { "routine" };
        let preferred_trade = input.category.preferred_trade();

        let metadata = serde_json::json!({
            "category": input.category.to_string(),
            "preferred_trade": preferred_trade.to_string(),
            "is_emergency": input.is_emergency,
            "voice_note_r2_key": input.voice_note_r2_key,
        });

        let subject = if input.is_emergency {
            format!("{} — EMERGENCY", input.category)
        } else {
            format!("{} — Routine", input.category)
        };

        let model = atlas_case::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_id),
            case_type: Set(PmCaseType::Maintenance.to_string()),
            asset_id: Set(Some(input.asset_id)),
            reported_by_user_id: Set(Some(input.reported_by_user_id)),
            subject: Set(subject),
            description: Set(Some(input.description)),
            status: Set("open".to_string()),
            priority: Set(priority.to_string()),
            assigned_user_id: Set(None),
            case_metadata: Set(Some(metadata)),
            created_at: Set(now),
            ..Default::default()
        };
        model.insert(db).await?;

        if input.is_emergency {
            tracing::warn!(case_id = %id, %tenant_id, "MaintenanceService: EMERGENCY ticket created");
        } else {
            tracing::info!(case_id = %id, %tenant_id, "MaintenanceService: maintenance ticket created");
        }

        // Phase 4: if voice_note_r2_key.is_some(), enqueue transcribe_maintenance_audio OutboxJob
        Ok(id)
    }

    /// Schedule a proactive inspection on a lifecycle-tracked asset.
    ///
    /// Creates an `atlas_cases` row with `case_type = "scheduled_inspection"` and
    /// `status = "scheduled"`. Advances `atlas_assets.scheduled_service_date` to the
    /// inspection date so no duplicate lifecycle alert fires before it.
    pub async fn schedule_inspection(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        input: ScheduleInspectionInput,
    ) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let metadata = serde_json::json!({
            "inspection_notes": input.notes,
        });

        atlas_case::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_id),
            case_type: Set(PmCaseType::ScheduledInspection.to_string()),
            asset_id: Set(Some(input.asset_id)),
            assigned_service_provider_id: Set(input.service_provider_id),
            assigned_user_id: Set(input.assigned_user_id),
            subject: Set(input.subject),
            status: Set("scheduled".to_string()),
            priority: Set("routine".to_string()),
            scheduled_at: Set(Some(input.scheduled_at)),
            estimated_cost_cents: Set(input.estimated_cost_cents),
            case_metadata: Set(Some(metadata)),
            created_at: Set(now),
            ..Default::default()
        }
        .insert(db)
        .await?;

        // Advance scheduled_service_date to the inspection date so the lifecycle
        // alert doesn't keep firing for a due date that now has an appointment booked.
        let inspection_date = input.scheduled_at.date_naive();
        if let Ok(Some(asset)) = atlas_asset::Entity::find_by_id(input.asset_id)
            .filter(atlas_asset::Column::TenantId.eq(tenant_id))
            .one(db)
            .await
        {
            let mut active: atlas_asset::ActiveModel = asset.into();
            active.scheduled_service_date = Set(Some(inspection_date));
            let _ = active.update(db).await;
        }

        tracing::info!(case_id = %id, %tenant_id, asset_id = %input.asset_id,
            "MaintenanceService: inspection scheduled");
        Ok(id)
    }

    /// Complete an inspection: record findings and roll the asset lifecycle forward.
    ///
    /// 1. Sets `atlas_cases.status = "completed"` and `completed_at = now`
    /// 2. Writes findings into `case_metadata.findings`
    /// 3. Updates `atlas_assets.scheduled_service_date` → next inspection date
    /// 4. Updates `atlas_assets.condition` if provided
    /// 5. Updates `atlas_assets.expiry_date` if cert was renewed
    /// 6. Registers each `attachment_r2_key` as a `PmDocumentType::InspectionReport`
    ///    in `atlas_documents` linked to the case — visible in the Vault.
    pub async fn complete_inspection(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        input: CompleteInspectionInput,
    ) -> Result<()> {
        let now = Utc::now();

        let case = atlas_case::Entity::find_by_id(input.case_id)
            .filter(atlas_case::Column::TenantId.eq(tenant_id))
            .filter(atlas_case::Column::CaseType.eq(PmCaseType::ScheduledInspection.to_string()))
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("inspection case {} not found", input.case_id))?;

        if case.status == "completed" {
            anyhow::bail!("inspection {} is already completed", input.case_id);
        }

        let asset_id = case.asset_id;

        // Merge findings into case_metadata
        let mut meta = case.case_metadata.clone().unwrap_or(serde_json::json!({}));
        if let Some(obj) = meta.as_object_mut() {
            obj.insert("findings".to_string(), serde_json::Value::String(input.findings.clone()));
            if let Some(c) = &input.condition_after {
                obj.insert("condition_after".to_string(), serde_json::Value::String(c.clone()));
            }
        }

        let mut case_model: atlas_case::ActiveModel = case.into();
        case_model.status = Set("completed".to_string());
        case_model.completed_at = Set(Some(now));
        case_model.case_metadata = Set(Some(meta));
        if let Some(cost) = input.actual_cost_cents {
            case_model.actual_cost_cents = Set(Some(cost));
        }
        case_model.update(db).await?;

        // Roll asset lifecycle forward
        if let Some(aid) = asset_id {
            if let Ok(Some(asset)) = atlas_asset::Entity::find_by_id(aid)
                .filter(atlas_asset::Column::TenantId.eq(tenant_id))
                .one(db)
                .await
            {
                let mut asset_model: atlas_asset::ActiveModel = asset.into();
                asset_model.scheduled_service_date = Set(input.next_inspection_date);
                if let Some(c) = input.condition_after {
                    asset_model.condition = Set(Some(c));
                }
                if let Some(e) = input.updated_expiry_date {
                    asset_model.expiry_date = Set(Some(e));
                }
                asset_model.update(db).await?;
                tracing::info!(case_id = %input.case_id, asset_id = %aid, %tenant_id,
                    next_due = ?input.next_inspection_date,
                    "MaintenanceService: inspection completed, asset lifecycle updated");
            }
        }

        // Register inspection report attachments in the Vault
        for r2_key in &input.attachment_r2_keys {
            if let Err(e) = crate::services::pm::vault::VaultService::register_document(
                db,
                tenant_id,
                "atlas_cases",
                input.case_id,
                crate::services::pm::vault::PmDocumentType::InspectionReport,
                r2_key,
            ).await {
                // Log but don't fail the whole completion — the inspection is done
                // even if an individual attachment registration fails.
                tracing::warn!(case_id = %input.case_id, %r2_key,
                    "complete_inspection: vault registration failed for attachment: {e:#}");
            } else {
                tracing::info!(case_id = %input.case_id, %r2_key,
                    "complete_inspection: inspection report registered in vault");
            }
        }

        Ok(())
    }

    /// List all inspections (any status) for a given asset.
    pub async fn list_inspections_for_asset(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        asset_id: Uuid,
    ) -> Result<Vec<InspectionDetail>> {
        let rows = atlas_case::Entity::find()
            .filter(atlas_case::Column::TenantId.eq(tenant_id))
            .filter(atlas_case::Column::AssetId.eq(asset_id))
            .filter(atlas_case::Column::CaseType.eq(PmCaseType::ScheduledInspection.to_string()))
            .all(db)
            .await?;
        Ok(rows.into_iter().map(to_inspection_detail).collect())
    }

    /// List all upcoming inspections across the whole tenant (status = "scheduled").
    pub async fn list_upcoming_inspections(
        db: &DatabaseConnection,
        tenant_id: Uuid,
    ) -> Result<Vec<InspectionDetail>> {
        let rows = atlas_case::Entity::find()
            .filter(atlas_case::Column::TenantId.eq(tenant_id))
            .filter(atlas_case::Column::CaseType.eq(PmCaseType::ScheduledInspection.to_string()))
            .filter(atlas_case::Column::Status.eq("scheduled"))
            .all(db)
            .await?;
        Ok(rows.into_iter().map(to_inspection_detail).collect())
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn to_inspection_detail(c: atlas_case::Model) -> InspectionDetail {
    InspectionDetail {
        id: c.id,
        asset_id: c.asset_id,
        subject: c.subject,
        status: c.status,
        scheduled_at: c.scheduled_at,
        completed_at: c.completed_at,
        service_provider_id: c.assigned_service_provider_id,
        assigned_user_id: c.assigned_user_id,
        estimated_cost_cents: c.estimated_cost_cents,
        actual_cost_cents: c.actual_cost_cents,
        metadata: c.case_metadata,
        created_at: c.created_at,
    }
}
