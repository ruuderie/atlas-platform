use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

/// GENERIC-13: AtlasCase
/// The universal work item / case / ticket object.
///
/// One of the highest-reuse generics in the platform.
/// Used for maintenance, insurance claims, housekeeping tasks, support tickets, compliance violations, etc.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_cases")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub case_type: String,
    pub reported_by_user_id: Option<Uuid>,
    pub asset_id: Option<Uuid>,                    // FK to atlas_assets
    pub contract_id: Option<Uuid>,                 // FK to atlas_contracts
    pub assigned_service_provider_id: Option<Uuid>,
    pub assigned_user_id: Option<Uuid>,
    pub priority: String,
    pub status: String,
    pub subject: String,
    pub description: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub estimated_cost_cents: Option<i64>,
    pub actual_cost_cents: Option<i64>,
    pub ledger_entry_id: Option<Uuid>,
    pub primary_attachment_id: Option<Uuid>,
    pub ws_room_id: Option<Uuid>,
    pub case_metadata: Option<Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
