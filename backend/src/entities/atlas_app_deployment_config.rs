//! Sea-ORM entity for `atlas_app_deployment_config` (G-33).
//!
//! One row per (tenant_id, app_slug). Missing row = mode "standard".
//!
//! # Folio instance operational mode (`folio_mode`)
//!
//! For `app_slug = "property_management"`, the `folio_mode` column declares the
//! mutually-exclusive operational identity of the instance:
//!
//! | mode       | Operator type           | Frontend namespaces  |
//! |---|---|---|
//! | standard   | Solo landlord/portfolio | /l/**                |
//! | pmc        | PMC operator            | /pmc/**              |
//! | brokerage  | Real estate brokerage   | /a/**, /b/**         |
//!
//! Portal enablement (tenant portal, vendor portal) is controlled separately via
//! `tenant_portal_enabled` and `vendor_portal_enabled` JSON config keys inside
//! the `config` column — they are optional flags within a mode, not modes themselves.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_app_deployment_config")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id:         Uuid,
    pub tenant_id:  Uuid,
    pub app_slug:   String,
    /// Platform-level deployment topology (standard vs internal operator).
    pub mode:       AppDeploymentMode,
    /// Arbitrary JSON config for this deployment (portal toggles, etc.).
    pub config:     Json,

    // ── Folio operational mode (m20260909) ───────────────────────────────────
    /// Mutually exclusive operational identity for Folio instances.
    /// Only meaningful when `app_slug = "property_management"`.
    /// Defaults to "standard" (solo landlord).
    /// Added by migration m20260909_folio_instance_mode.
    pub folio_mode: FolioMode,

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

impl Model {
    /// Returns true if this is a Folio instance in the given mode.
    pub fn is_folio_mode(&self, mode: &FolioMode) -> bool {
        self.app_slug == "property_management" && self.folio_mode == *mode
    }

    /// Returns the `tenant_portal_enabled` config flag (default: false).
    pub fn tenant_portal_enabled(&self) -> bool {
        self.config.get("tenant_portal_enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    /// Returns the `vendor_portal_enabled` config flag (default: false).
    pub fn vendor_portal_enabled(&self) -> bool {
        self.config.get("vendor_portal_enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }
}

/// Platform-level deployment topology (standard vs internal operator).
/// This is separate from `folio_mode` — it governs billing/operator topology,
/// not the application's operational identity.
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, strum_macros::Display)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(50))")]
pub enum AppDeploymentMode {
    #[sea_orm(string_value = "standard")]
    Standard,
    #[sea_orm(string_value = "internal_operator")]
    InternalOperator,
}

/// Mutually-exclusive operational identity for a Folio instance.
///
/// Stored in `atlas_app_deployment_config.folio_mode` (TEXT with DB CHECK constraint).
/// The CHECK constraint prevents any row from holding an unrecognised value.
/// Enforces that a single instance cannot simultaneously be PMC AND brokerage.
///
/// Added by migration `m20260909_folio_instance_mode`.
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, strum_macros::Display, Default)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
pub enum FolioMode {
    /// Solo landlord / portfolio operator (default). Frontend: /l/**
    #[default]
    #[sea_orm(string_value = "standard")]
    Standard,
    /// Property Management Company — manages multiple client landlord accounts.
    /// Unlocks /pmc/** and PropertyManagerOnly extractor.
    #[sea_orm(string_value = "pmc")]
    Pmc,
    /// Real estate brokerage — agent and broker portals, commission plans.
    /// Unlocks /a/** (agent) and /b/** (broker) and BrokerageOnly extractor.
    #[sea_orm(string_value = "brokerage")]
    Brokerage,
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
