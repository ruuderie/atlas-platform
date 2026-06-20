use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "flag_audit_log")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub flag_id: Uuid,
    pub user_id: String,
    pub action: String,
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
