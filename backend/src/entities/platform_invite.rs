use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "platform_invite")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub email: String,
    pub role: String,
    pub tenant_name: String,
    pub invited_by: String,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
