use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

/// GENERIC-08: AtlasAiTask
///
/// Queue for asynchronous AI/LLM work. All expensive model calls should go through here.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_ai_tasks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub task_type: String,
    pub model: Option<String>,
    pub input_payload: Value,
    pub output_payload: Option<Value>,
    pub source_entity_type: Option<String>,
    pub source_entity_id: Option<Uuid>,
    pub callback_entity_type: Option<String>,
    pub callback_entity_id: Option<Uuid>,
    pub callback_field: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub retry_count: i32,
    pub queued_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub estimated_cost_micro_usd: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
