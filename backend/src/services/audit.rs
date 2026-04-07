use crate::entities::audit_log;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use serde_json::Value;
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;

pub struct AuditService;

impl AuditService {
    /// Asynchronously logs an action in a non-blocking context (Soft-fail auditing)
    pub fn log_action(
        db: DatabaseConnection,
        tenant_id: Option<Uuid>,
        actor_id: Option<Uuid>,
        action_type: String,
        entity_type: String,
        entity_id: Uuid,
        old_state: Option<Value>,
        new_state: Option<Value>,
        ip_address: Option<String>,
    ) {
        // Spawn a background task to avoid blocking the primary web request
        tokio::spawn(async move {
            let active_log = audit_log::ActiveModel {
                id: Set(Uuid::new_v4()),
                tenant_id: Set(tenant_id),
                actor_id: Set(actor_id),
                action_type: Set(action_type.clone()),
                entity_type: Set(entity_type.clone()),
                entity_id: Set(entity_id),
                old_state: Set(old_state),
                new_state: Set(new_state),
                ip_address: Set(ip_address),
                created_at: Set(Utc::now()),
            };

            match active_log.insert(&db).await {
                Ok(_) => {
                    tracing::debug!(
                        "Successfully recorded audit log for {} action on {:?}",
                         action_type, entity_id
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to insert audit log for {} action on {}: {:?}",
                         action_type, entity_id, e
                    );
                }
            }
        });
    }
}
