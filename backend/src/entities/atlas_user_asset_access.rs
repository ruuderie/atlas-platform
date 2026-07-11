#![allow(dead_code, unused_imports)]
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Per-asset permission grant.
///
/// Implements the finest level of access scoping in Folio:
/// "this user is allowed to see/act on this specific property."
///
/// # Semantics
///
/// Additive / open-by-default:
/// - A user with **0** rows here is NOT asset-restricted (full account access for their role).
/// - A user with **N** rows here can ONLY access those N assets.
///
/// This is checked at query time by the service layer whenever the user's role is
/// asset-scoped (Cohost, asset-restricted delegate, etc.).
///
/// # Use cases
///
/// | Role       | What asset_id points to                    |
/// |------------|--------------------------------------------|
/// | Cohost     | An `atlas_assets` row for an STR property  |
/// | Delegate   | Any `atlas_assets` row in the landlord's portfolio |
/// | Vendor     | Assets where their work orders are filed   |
///
/// Tenants are NOT stored here — use `atlas_leases.tenant_user_id` instead.
///
/// # Migration
/// Created in `m20261005_atlas_user_asset_access`.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_user_asset_access")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub asset_id: Uuid,
    pub role_profile_id: Uuid,
    pub granted_by: Option<Uuid>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub granted_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone", nullable)]
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
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
impl Related<super::atlas_role_profiles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RoleProfile.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
