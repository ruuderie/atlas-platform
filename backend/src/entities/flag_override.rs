use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "flag_overrides")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub flag_id: Uuid,
    pub tenant_id: Uuid,
    pub override_type: String, // "grant" or "deny"
    pub rollout_pct: i32,
    pub reason: String,
    pub jira: Option<String>,
    pub changed_by: String,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::feature_flag::Entity",
        from = "Column::FlagId",
        to = "super::feature_flag::Column::Id"
    )]
    FeatureFlag,
}

impl Related<super::feature_flag::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FeatureFlag.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
