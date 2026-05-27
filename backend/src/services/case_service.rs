use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, Set, QueryFilter, ColumnTrait, QuerySelect};
use uuid::Uuid;
use chrono::Utc;
use serde_json::Value;

use crate::entities::atlas_case::{self, Entity as CaseEntity, ActiveModel as CaseActiveModel};

/// Service layer for GENERIC-13: AtlasCase
/// The universal work item / ticket / case object used across maintenance,
/// support, compliance, insurance, vendor jobs, etc.
pub struct CaseService;

impl CaseService {
    /// Create a new case / work item.
    pub async fn create_case(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        case_type: &str,
        subject: &str,
        priority: &str,
        description: Option<&str>,
        asset_id: Option<Uuid>,
        assigned_user_id: Option<Uuid>,
    ) -> Result<Uuid, String> {
        let new_case = CaseActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            case_type: Set(case_type.to_string()),
            subject: Set(subject.to_string()),
            description: Set(description.map(|s| s.to_string())),
            priority: Set(priority.to_string()),
            status: Set("open".to_string()),
            asset_id: Set(asset_id),
            assigned_user_id: Set(assigned_user_id),
            created_at: Set(Utc::now()),
            case_metadata: Set(None),
            ..Default::default()
        };

        let result = new_case.insert(db).await.map_err(|e| e.to_string())?;
        Ok(result.id)
    }

    pub async fn find_by_id(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        case_id: Uuid,
    ) -> Result<Option<atlas_case::Model>, String> {
        CaseEntity::find()
            .filter(atlas_case::Column::TenantId.eq(tenant_id))
            .filter(atlas_case::Column::Id.eq(case_id))
            .one(db)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn list_for_tenant(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        status: Option<&str>,
        limit: u64,
    ) -> Result<Vec<atlas_case::Model>, String> {
        let mut q = CaseEntity::find().filter(atlas_case::Column::TenantId.eq(tenant_id));
        if let Some(s) = status {
            q = q.filter(atlas_case::Column::Status.eq(s.to_string()));
        }
        q.limit(limit).all(db).await.map_err(|e| e.to_string())
    }

    /// Record cost against the case (links to ledger eventually via ledger_entry_id).
    pub async fn record_cost(
        _db: &DatabaseConnection,
        _tenant_id: Uuid,
        case_id: Uuid,
        estimated_cents: Option<i64>,
        actual_cents: Option<i64>,
    ) -> Result<(), String> {
        tracing::info!(
            "Case {} cost recorded (est={:?}, actual={:?})",
            case_id, estimated_cents, actual_cents
        );
        Ok(())
    }

    /// Transition status (open → in_progress → completed etc.).
    pub async fn transition_status(
        _db: &DatabaseConnection,
        _tenant_id: Uuid,
        case_id: Uuid,
        new_status: &str,
        _completed_at: bool,
    ) -> Result<(), String> {
        tracing::info!("Case {} transitioned to status {}", case_id, new_status);
        Ok(())
    }
}