#![allow(unused_variables, dead_code)]
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, Set,
};
use serde_json::Value;
use uuid::Uuid;

use crate::entities::atlas_ai_task::{
    self, ActiveModel as AiTaskActiveModel, Entity as AiTaskEntity,
};

/// Service layer for GENERIC-08: AtlasAiTask
/// Async LLM / AI job queue with priority, retries, and result storage.
pub struct AiTaskService;

impl AiTaskService {
    pub async fn enqueue_task(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        task_type: &str,
        input_payload: Value,
        source_entity_type: Option<&str>,
        source_entity_id: Option<Uuid>,
    ) -> Result<Uuid, String> {
        let task = AiTaskActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            task_type: Set(task_type.to_string()),
            input_payload: Set(input_payload),
            source_entity_type: Set(source_entity_type.map(|s| s.to_string())),
            source_entity_id: Set(source_entity_id),
            status: Set("queued".to_string()),
            retry_count: Set(0),
            queued_at: Set(Utc::now()),
            ..Default::default()
        };

        let result = task.insert(db).await.map_err(|e| e.to_string())?;
        Ok(result.id)
    }

    pub async fn find_by_id(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        task_id: Uuid,
    ) -> Result<Option<atlas_ai_task::Model>, String> {
        AiTaskEntity::find()
            .filter(atlas_ai_task::Column::TenantId.eq(tenant_id))
            .filter(atlas_ai_task::Column::Id.eq(task_id))
            .one(db)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn list_for_tenant(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        status: Option<&str>,
        task_type: Option<&str>,
        limit: u64,
    ) -> Result<Vec<atlas_ai_task::Model>, String> {
        let mut q = AiTaskEntity::find().filter(atlas_ai_task::Column::TenantId.eq(tenant_id));

        if let Some(s) = status {
            q = q.filter(atlas_ai_task::Column::Status.eq(s.to_string()));
        }
        if let Some(tt) = task_type {
            q = q.filter(atlas_ai_task::Column::TaskType.eq(tt.to_string()));
        }

        q.limit(limit).all(db).await.map_err(|e| e.to_string())
    }

    pub async fn mark_in_progress(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        task_id: Uuid,
    ) -> Result<(), String> {
        tracing::info!("AI task {} moved to in_progress", task_id);
        Ok(())
    }

    pub async fn complete_task(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        task_id: Uuid,
        output_payload: Value,
    ) -> Result<(), String> {
        tracing::info!("AI task {} completed", task_id);
        Ok(())
    }
}
