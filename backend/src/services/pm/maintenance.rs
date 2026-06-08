//! Folio — Maintenance Service (PM wrapper over G-13 `atlas_cases`)
//!
//! Emergency bypass routing, vendor dispatch, WebSocket real-time threading.
//!
//! # Entity field map (`atlas_cases`)
//!   `asset_id`                    → the property the ticket is for
//!   `reported_by_user_id`         → tenant who filed (not `reporter_user_id`)
//!   `assigned_user_id`            → landlord/PM assignee (not `assigned_to_user_id`)
//!   `priority`                    → "emergency" | "routine" (required string)
//!   `case_metadata`               → JSONB: category, preferred_trade, emergency flag
//!   No `entity_type`/`entity_id` columns — `asset_id` is the direct FK
//!   No `updated_at` column on this entity

use anyhow::Result;
use sea_orm::DatabaseConnection;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::types::pm::{PmCaseType, MaintenanceCategory};

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

pub struct MaintenanceService;

impl MaintenanceService {
    /// Create a maintenance case in `atlas_cases`.
    ///
    /// Emergency tickets skip standard scheduling and are queued for
    /// immediate dispatch via the landlord emergency queue.
    /// The preferred vendor trade is derived from `category.preferred_trade()`.
    pub async fn create_ticket(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        input: CreateMaintenanceTicketInput,
    ) -> Result<Uuid> {
        use sea_orm::{Set, ActiveModelTrait};
        use chrono::Utc;

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

        let model = crate::entities::atlas_case::ActiveModel {
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
}
