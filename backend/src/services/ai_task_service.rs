#![allow(unused_variables, dead_code)]
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set, Statement,
};
use serde_json::{Value, json};
use std::sync::atomic::{AtomicBool, Ordering};
use uuid::Uuid;

use crate::entities::atlas_ai_task::{
    self, ActiveModel as AiTaskActiveModel, Entity as AiTaskEntity,
};

/// In-memory pause flag for the AI task queue (process-local).
static AI_QUEUE_PAUSED: AtomicBool = AtomicBool::new(false);

/// Service layer for GENERIC-08: AtlasAiTask
/// Async LLM / AI job queue with priority, retries, and result storage.
pub struct AiTaskService;

impl AiTaskService {
    pub fn is_queue_paused() -> bool {
        AI_QUEUE_PAUSED.load(Ordering::Relaxed)
    }

    pub fn set_queue_paused(paused: bool) {
        AI_QUEUE_PAUSED.store(paused, Ordering::Relaxed);
        if paused {
            tracing::warn!("AI task queue PAUSED");
        } else {
            tracing::info!("AI task queue RESUMED");
        }
    }

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
            // Seed empty log_lines array in output_payload for progressive logging
            output_payload: Set(Some(json!({ "log_lines": [] }))),
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
        let task = AiTaskEntity::find()
            .filter(atlas_ai_task::Column::TenantId.eq(tenant_id))
            .filter(atlas_ai_task::Column::Id.eq(task_id))
            .one(db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("AI task {task_id} not found"))?;

        let mut active: AiTaskActiveModel = task.into();
        active.status = Set("running".to_string());
        active.started_at = Set(Some(Utc::now()));
        active
            .update(db)
            .await
            .map_err(|e| e.to_string())?;

        tracing::info!("AI task {} moved to running", task_id);
        Ok(())
    }

    /// Append a log line into `output_payload.log_lines` (JSON array).
    pub async fn append_log_line(
        db: &DatabaseConnection,
        task_id: Uuid,
        line: &str,
    ) -> Result<(), String> {
        let task = AiTaskEntity::find_by_id(task_id)
            .one(db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("AI task {task_id} not found"))?;

        let mut payload = task.output_payload.clone().unwrap_or_else(|| json!({}));
        if !payload.is_object() {
            payload = json!({ "result": payload });
        }
        let lines = payload
            .as_object_mut()
            .unwrap()
            .entry("log_lines")
            .or_insert_with(|| json!([]));
        if let Some(arr) = lines.as_array_mut() {
            arr.push(json!(line));
        }

        let mut active: AiTaskActiveModel = task.into();
        active.output_payload = Set(Some(payload));
        active
            .update(db)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Extract stored log lines from output_payload, if any.
    pub fn stored_log_lines(task: &atlas_ai_task::Model) -> Option<Vec<String>> {
        let payload = task.output_payload.as_ref()?;
        let lines = payload.get("log_lines")?.as_array()?;
        if lines.is_empty() {
            return None;
        }
        Some(
            lines
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
        )
    }

    pub async fn complete_task(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        task_id: Uuid,
        output_payload: Value,
    ) -> Result<(), String> {
        let task = AiTaskEntity::find()
            .filter(atlas_ai_task::Column::TenantId.eq(tenant_id))
            .filter(atlas_ai_task::Column::Id.eq(task_id))
            .one(db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("AI task {task_id} not found"))?;

        // Preserve existing log_lines when merging result payload
        let mut merged = output_payload;
        if let Some(existing) = &task.output_payload {
            if let Some(lines) = existing.get("log_lines") {
                if let Some(obj) = merged.as_object_mut() {
                    obj.insert("log_lines".to_string(), lines.clone());
                } else {
                    merged = json!({
                        "result": merged,
                        "log_lines": lines,
                    });
                }
            }
        }

        let mut active: AiTaskActiveModel = task.into();
        active.status = Set("success".to_string());
        active.output_payload = Set(Some(merged));
        active.completed_at = Set(Some(Utc::now()));
        active.error_message = Set(None);
        active
            .update(db)
            .await
            .map_err(|e| e.to_string())?;

        tracing::info!("AI task {} completed", task_id);
        Ok(())
    }

    pub async fn fail_task(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        task_id: Uuid,
        error: &str,
    ) -> Result<(), String> {
        let task = AiTaskEntity::find()
            .filter(atlas_ai_task::Column::TenantId.eq(tenant_id))
            .filter(atlas_ai_task::Column::Id.eq(task_id))
            .one(db)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("AI task {task_id} not found"))?;

        let mut active: AiTaskActiveModel = task.into();
        active.status = Set("failed".to_string());
        active.error_message = Set(Some(error.to_string()));
        active.completed_at = Set(Some(Utc::now()));
        active
            .update(db)
            .await
            .map_err(|e| e.to_string())?;

        let _ = Self::append_log_line(db, task_id, &format!("[ERROR] {error}")).await;
        tracing::warn!(task_id = %task_id, error, "AI task failed");
        Ok(())
    }

    /// Claim the next queued task with `FOR UPDATE SKIP LOCKED` when available.
    pub async fn dequeue_next(db: &DatabaseConnection) -> Result<Option<atlas_ai_task::Model>, String> {
        let stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            r#"
            UPDATE atlas_ai_tasks
            SET status = 'running', started_at = NOW()
            WHERE id = (
                SELECT id
                FROM atlas_ai_tasks
                WHERE status = 'queued'
                ORDER BY queued_at ASC
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            RETURNING *
            "#,
            vec![],
        );

        match atlas_ai_task::Entity::find().from_raw_sql(stmt).one(db).await {
            Ok(row) => Ok(row),
            Err(e) => {
                // Fallback: simple select + update (non-Postgres or SKIP LOCKED unavailable)
                tracing::debug!(
                    "SKIP LOCKED dequeue failed ({e}); falling back to simple select"
                );
                let task = AiTaskEntity::find()
                    .filter(atlas_ai_task::Column::Status.eq("queued"))
                    .order_by_asc(atlas_ai_task::Column::QueuedAt)
                    .limit(1)
                    .one(db)
                    .await
                    .map_err(|e| e.to_string())?;

                if let Some(task) = task {
                    let tenant_id = task.tenant_id;
                    let task_id = task.id;
                    Self::mark_in_progress(db, tenant_id, task_id).await?;
                    AiTaskEntity::find_by_id(task_id)
                        .one(db)
                        .await
                        .map_err(|e| e.to_string())
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// Background job entrypoint: dequeue and process up to a few queued tasks.
    pub async fn process_ai_tasks(db: &DatabaseConnection) -> Result<(), String> {
        if Self::is_queue_paused() {
            tracing::debug!("process_ai_tasks: queue paused — skipping");
            return Ok(());
        }

        // Process a small batch per tick
        for _ in 0..3 {
            if Self::is_queue_paused() {
                break;
            }
            let Some(task) = Self::dequeue_next(db).await? else {
                break;
            };
            if let Err(e) = Self::execute_task(db, &task).await {
                tracing::error!(task_id = %task.id, error = %e, "AI task execution failed");
                let _ = Self::fail_task(db, task.tenant_id, task.id, &e).await;
                if let Some(variant_id) = task.source_entity_id {
                    if task.task_type == "localize_product_page" {
                        let _ = crate::services::product_localization::ProductLocalizationService::mark_localization_failed(
                            db, variant_id, &e,
                        )
                        .await;
                    }
                }
            }
        }

        Ok(())
    }

    /// Dedicated 10s poller — complements the DataSync `process_ai_tasks` job so
    /// the queue advances even before tenant_background_jobs rows are seeded.
    pub async fn start_worker(db: DatabaseConnection) {
        tokio::spawn(async move {
            tracing::info!("Starting AiTaskService background worker (10s).");
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
            loop {
                interval.tick().await;
                if let Err(e) = Self::process_ai_tasks(&db).await {
                    tracing::error!("AiTaskService worker error: {e}");
                }
            }
        });
    }

    async fn execute_task(db: &DatabaseConnection, task: &atlas_ai_task::Model) -> Result<(), String> {
        let task_id = task.id;
        let tenant_id = task.tenant_id;

        Self::append_log_line(
            db,
            task_id,
            &format!(
                "[INFO] Execution started at {}",
                Utc::now().format("%H:%M:%S")
            ),
        )
        .await?;
        Self::append_log_line(
            db,
            task_id,
            &format!(
                "[INFO] Task Type: {}, Target Engine: {}",
                task.task_type,
                task.model.as_deref().unwrap_or("gpt-4o-mini")
            ),
        )
        .await?;

        match task.task_type.as_str() {
            "localize_product_page" => {
                Self::append_log_line(
                    db,
                    task_id,
                    "[INFO] Resolving product page variant context...",
                )
                .await?;
                Self::append_log_line(db, task_id, "[INFO] Running localization pipeline...")
                    .await?;

                let output = Self::build_localization_output(&task.input_payload)?;
                Self::append_log_line(
                    db,
                    task_id,
                    "[INFO] Committing output payload to state DB context.",
                )
                .await?;
                Self::append_log_line(db, task_id, "[SUCCESS] Task execution completed cleanly.")
                    .await?;

                Self::complete_task(db, tenant_id, task_id, output).await?;

                if let Err(e) = crate::services::product_localization::ProductLocalizationService::apply_localization_result(
                    db, task_id,
                )
                .await
                {
                    if let Some(variant_id) = task.source_entity_id {
                        let _ = crate::services::product_localization::ProductLocalizationService::mark_localization_failed(
                            db, variant_id, &e,
                        )
                        .await;
                    }
                    return Err(format!("apply localization failed: {e}"));
                }
            }
            other => {
                Self::append_log_line(
                    db,
                    task_id,
                    &format!(
                        "[WARNING] No specialized handler for task_type='{other}'; completing with passthrough."
                    ),
                )
                .await?;
                let output = json!({
                    "passthrough": true,
                    "input": task.input_payload,
                });
                Self::append_log_line(db, task_id, "[SUCCESS] Task execution completed cleanly.")
                    .await?;
                Self::complete_task(db, tenant_id, task_id, output).await?;
            }
        }

        Ok(())
    }

    /// Deterministic localization stub: copies source hero/blocks and annotates
    /// meta fields for the target locale/city. Real LLM wiring can replace this
    /// without changing the queue lifecycle.
    fn build_localization_output(input: &Value) -> Result<Value, String> {
        let locale = input
            .get("locale")
            .and_then(|v| v.as_str())
            .unwrap_or("en");
        let city = input
            .get("city")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let product = input
            .get("product_name")
            .and_then(|v| v.as_str())
            .unwrap_or("Product");

        let hero = input
            .get("current_hero_overrides")
            .filter(|v| !v.is_null() && v.as_object().map(|o| !o.is_empty()).unwrap_or(false))
            .or_else(|| input.get("source_hero"))
            .cloned()
            .unwrap_or_else(|| json!({}));

        let blocks = input
            .get("current_block_overrides")
            .filter(|v| !v.is_null() && v.as_object().map(|o| !o.is_empty()).unwrap_or(false))
            .or_else(|| input.get("source_blocks"))
            .cloned()
            .unwrap_or_else(|| json!({}));

        let meta_title = if city.is_empty() {
            format!("{product} — {locale}")
        } else {
            format!("{product} — {city}")
        };
        let meta_description = input
            .get("source_meta_description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                if city.is_empty() {
                    format!("{product} for the {locale} market.")
                } else {
                    format!("{product} for {city} ({locale}).")
                }
            });

        Ok(json!({
            "hero_overrides": hero,
            "block_overrides": blocks,
            "meta_title": meta_title.chars().take(60).collect::<String>(),
            "meta_description": meta_description.chars().take(160).collect::<String>(),
            "pipeline": "deterministic_stub",
        }))
    }
}
