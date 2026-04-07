use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDate;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "platform_metrics_daily")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub date: NaiveDate,
    pub tenant_id: Uuid,
    pub metric_source: String,
    pub metric_key: String,
    #[sea_orm(column_type = "Float")]
    pub metric_value: f32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
