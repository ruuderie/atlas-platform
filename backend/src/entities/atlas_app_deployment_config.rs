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
    /// Deployment mode — app-defined string.
    /// Folio: "standard" | "property_management_co"
    pub mode:       String,
    /// Arbitrary JSON config for this mode.
    pub config:     Json,

    // ── Public config (m20260902) ─────────────────────────────────────────────
    /// Short handle for shared-platform URLs, e.g. "oakwood" → oakwood.folio.app
    pub public_slug:     Option<String>,
    /// Full FQDN for white-label deployments, e.g. "listings.oakwoodpm.com"
    pub custom_domain:   Option<String>,
    /// "active" | "suspended" | "archived"
    pub instance_status: String,

    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
