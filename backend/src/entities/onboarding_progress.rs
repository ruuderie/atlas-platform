#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Tracks explicit onboarding progress overrides (skips and custom completions) per
/// app instance. The primary source of truth for required steps is the real data
/// (domains, categories, settings) — this table records intentional user overrides only.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "onboarding_progress")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub app_instance_id: Uuid,
    /// Matches OnboardingStep.id e.g. "identity", "domain", "categories"
    pub step_id: String,
    /// Set when the user explicitly marks a custom step complete
    #[sea_orm(nullable)]
    pub completed_at: Option<DateTime<Utc>>,
    /// Set when the user clicks "Skip for now" on an optional step
    pub skipped: bool,
    /// Set when the user clicks "I'll do this later" to dismiss the full-page wizard
    #[sea_orm(nullable)]
    pub dismissed_at: Option<DateTime<Utc>>,
    /// Optional JSON payload captured at completion (e.g. which template was chosen)
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub metadata: Option<serde_json::Value>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::tenant::Entity",
        from = "Column::TenantId",
        to = "super::tenant::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Tenant,
    #[sea_orm(
        belongs_to = "super::app_instance::Entity",
        from = "Column::AppInstanceId",
        to = "super::app_instance::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    AppInstance,
}

impl Related<super::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}
impl Related<super::app_instance::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AppInstance.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
