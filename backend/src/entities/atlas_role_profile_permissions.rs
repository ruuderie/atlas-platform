#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// G-32: Additive permission slug for a role profile.
/// Slugs are free-form strings owned by each app (e.g. `lease:read`, `billing:admin`).
/// The owning app defines and documents its own slug namespace.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_role_profile_permissions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub role_profile_id: Uuid,
    pub permission_slug: String,
    pub is_allowed: bool,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::atlas_role_profiles::Entity",
        from = "Column::RoleProfileId",
        to = "super::atlas_role_profiles::Column::Id",
        on_delete = "Cascade"
    )]
    RoleProfile,
}

impl Related<super::atlas_role_profiles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RoleProfile.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
