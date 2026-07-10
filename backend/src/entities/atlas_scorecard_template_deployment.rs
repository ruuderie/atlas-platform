#![allow(dead_code, unused_imports)]
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// G-27 Phase 1b: which templates an app instance may list/use.
///
/// Unique on `(template_id, app_instance_id)`.
/// Contract: `docs/contracts/g27_scorecard_platform.md` §4 Deployment.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_scorecard_template_deployments")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub template_id: Uuid,
    pub app_instance_id: Uuid,
    pub tenant_id: Uuid,
    pub is_enabled: bool,
    /// MVP default: `manual`. Full trigger matrix is out of scope for Phase 1b.
    pub trigger_event: String,
    pub trigger_context_entity_type: Option<String>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::atlas_scorecard_template::Entity",
        from = "Column::TemplateId",
        to = "super::atlas_scorecard_template::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Template,
    #[sea_orm(
        belongs_to = "super::app_instance::Entity",
        from = "Column::AppInstanceId",
        to = "super::app_instance::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    AppInstance,
}

impl Related<super::atlas_scorecard_template::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Template.def()
    }
}

impl Related<super::app_instance::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AppInstance.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
