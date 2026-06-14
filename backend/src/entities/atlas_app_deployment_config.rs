//! Sea-ORM entity for `atlas_app_deployment_config` (G-33).
//!
//! One row per (tenant_id, app_slug). Missing row = mode "standard".

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_app_deployment_config")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id:         Uuid,
    pub tenant_id:  Uuid,
    pub app_slug:   String,
    /// Deployment mode (standard vs internal operator).
    pub mode:       AppDeploymentMode,
    /// Arbitrary JSON config for this mode.
    pub config:     Json,

    // ── Public config (m20260902) ─────────────────────────────────────────────
    /// Short handle for shared-platform URLs, e.g. "oakwood" → oakwood.folio.app
    pub public_slug:     Option<String>,
    /// Full FQDN for white-label deployments, e.g. "listings.oakwoodpm.com"
    pub custom_domain:   Option<String>,
    /// Instance operational lifecycle status
    pub instance_status: AppInstanceStatus,

    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

/// Platform-level deployment topology (standard vs internal operator).
/// App-specific configurations (such as Folio's PMC mode or broker mode)
/// are stored inside the JSON `config` payload, not as deployment modes.
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, strum_macros::Display)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(50))")]
pub enum AppDeploymentMode {
    #[sea_orm(string_value = "standard")]
    Standard,
    #[sea_orm(string_value = "internal_operator")]
    InternalOperator,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, strum_macros::Display)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
pub enum AppInstanceStatus {
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "suspended")]
    Suspended,
    #[sea_orm(string_value = "archived")]
    Archived,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
