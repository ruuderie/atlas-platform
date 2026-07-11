#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// G-32: Binds a user to a role profile within an app+tenant context.
/// One active row per (user_id, tenant_id, app_slug) — enforced by unique constraint.
///
/// `client_account_id` (added m20260818): nullable. When set, this Landlord user's
/// role is scoped to a specific client account within a PMC tenant. NULL = org-level role.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_user_app_roles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub app_slug: String,
    pub role_profile_id: Uuid,
    pub granted_by: Option<Uuid>,
    pub granted_at: DateTimeUtc,
    pub expires_at: Option<DateTimeUtc>,
    pub is_active: bool,
    /// PMC client scope. NULL = org-level. UUID = scoped to this specific client account.
    pub client_account_id: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id",
        on_delete = "Cascade"
    )]
    User,
    #[sea_orm(
        belongs_to = "super::tenant::Entity",
        from = "Column::TenantId",
        to = "super::tenant::Column::Id",
        on_delete = "Cascade"
    )]
    Tenant,
    #[sea_orm(
        belongs_to = "super::atlas_role_profiles::Entity",
        from = "Column::RoleProfileId",
        to = "super::atlas_role_profiles::Column::Id"
    )]
    RoleProfile,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}
impl Related<super::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}
impl Related<super::atlas_role_profiles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RoleProfile.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
