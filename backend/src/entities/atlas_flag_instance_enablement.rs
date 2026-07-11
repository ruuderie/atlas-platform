//! Sea-ORM entity for `atlas_flag_instance_enablements`.
//!
//! Per-app-instance grant/deny overrides for catalog feature flags.
//! `effect` is persisted as VARCHAR; convert with [`crate::types::flags::FlagEffect`]
//! at the service / API boundary.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_flag_instance_enablements")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// Matches `feature_flags.key` (text, not FK — catalog keys are stable).
    pub flag_key: String,
    pub app_instance_id: Uuid,
    /// Wire value: `"grant"` | `"deny"`. See [`crate::types::flags::FlagEffect`].
    pub effect: String,
    pub rollout_pct: i32,
    pub updated_by: Option<Uuid>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::app_instance::Entity",
        from = "Column::AppInstanceId",
        to = "super::app_instance::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    AppInstance,
}

impl Related<super::app_instance::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AppInstance.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
