use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "outbox_job")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub attempts: i32,
    #[sea_orm(nullable)]
    pub error_message: Option<String>,
    #[sea_orm(nullable)]
    pub locked_by: Option<String>,
    #[sea_orm(column_type = "TimestampWithTimeZone", nullable)]
    pub locked_at: Option<DateTime<Utc>>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub run_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
