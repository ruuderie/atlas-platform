use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "global_search_index")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub tenant_id: Option<Uuid>,
    // SeaORM doesn't natively map tsvector to a standard Rust struct out of the box nicely for inserts
    // without custom types, but `String` usually works for reads/writes if it's implicitly castable,
    // or we skip treating it as a standard model field if it's pure SQL managed.
    // For now, we will represent it as an Option<String> and treat it manually in updates.
    #[sea_orm(column_type = "custom(\"tsvector\")", nullable)]
    pub searchable_text: Option<String>,
    #[sea_orm(column_type = "JsonBinary")]
    pub metadata: Value,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
