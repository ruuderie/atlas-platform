#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// G-32: Platform-generic role profile template.
/// `is_platform_default = true` rows have `tenant_id = NULL` and are
/// seeded by each `AtlasApp::provision()`. Tenant-scoped profiles
/// (custom overrides) have `tenant_id` set and `is_platform_default = false`.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_role_profiles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id:                  Uuid,
    pub tenant_id:           Option<Uuid>,
    pub app_slug:            String,
    pub role_slug:           String,
    pub display_name:        String,
    pub description:         Option<String>,
    pub is_platform_default: bool,
    pub metadata:            serde_json::Value,
    pub created_at:          DateTimeUtc,
    pub updated_at:          DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::atlas_user_app_roles::Entity")]
    UserAppRoles,
    #[sea_orm(has_many = "super::atlas_role_profile_permissions::Entity")]
    Permissions,
}

impl Related<super::atlas_user_app_roles::Entity> for Entity {
    fn to() -> RelationDef { Relation::UserAppRoles.def() }
}
impl Related<super::atlas_role_profile_permissions::Entity> for Entity {
    fn to() -> RelationDef { Relation::Permissions.def() }
}

impl ActiveModelBehavior for ActiveModel {}
