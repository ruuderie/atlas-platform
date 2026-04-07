use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "telemetry_events")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub event_source: String,
    pub event_type: String,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub event_payload: Option<serde_json::Value>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub timestamp: DateTime<Utc>,
    pub processed: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
