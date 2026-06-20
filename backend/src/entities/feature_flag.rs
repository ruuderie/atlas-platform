use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "feature_flags")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub key: String,
    pub description: String,
    pub is_enabled: bool,
    pub has_global: bool,
    pub global_rollout_pct: i32,
    pub is_plan_gated: bool,
    pub plan_gate_tier: Option<String>,
    pub jira: Option<String>,
    pub owner: String,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::flag_override::Entity")]
    FlagOverrides,
    #[sea_orm(has_many = "super::flag_audit_log::Entity")]
    AuditLogs,
}

impl Related<super::flag_override::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::FlagOverrides.def()
    }
}

impl Related<super::flag_audit_log::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AuditLogs.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
