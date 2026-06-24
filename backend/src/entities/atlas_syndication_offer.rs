//! Sea-ORM entity for `atlas_syndication_offer`.
//!
//! Platform admin-controlled catalog of available syndication connections.
//! This is Layer A of the two-layer syndication model.
//!
//! Platform admin creates offers that define:
//! - Which NI a Folio/app instance can connect to
//! - Whether the link is mandatory for certain billing tiers (monetization)
//! - Whether operators can self-service activate/deactivate it
//! - What listing types flow through the connection
//!
//! Created by migration `m20260912_atlas_syndication_offer`.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_syndication_offer")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// The destination Network Instance deployment config.
    pub ni_config_id: Uuid,

    /// Human-readable name shown in platform-admin and operator UI.
    pub display_name: String,

    /// Brief description shown to operators in the self-service UI.
    pub description: Option<String>,

    /// JSONB array of listing type slugs that flow through this offer.
    /// Valid values: "ltr", "str", "for_sale", "vendor_profile", "tenant_profile"
    pub syndication_types: Value,

    /// How the NI is presented to the operator.
    pub link_type: SyndicationLinkType,

    /// JSONB array of billing tier slugs for which this offer is mandatory.
    /// Operators on these tiers cannot opt out.
    /// Example: `["free", "starter"]`
    pub is_mandatory_for_tiers: Value,

    /// If true, operators can self-service activate/deactivate this offer.
    /// If false, only platform admin can create the active link.
    pub self_service_allowed: bool,

    /// Filter: which `folio_mode` this offer applies to. NULL = any mode.
    pub applies_to_folio_mode: Option<String>,

    /// Filter: which `app_slug` this offer applies to. NULL = any app.
    pub applies_to_app_slug: Option<String>,

    /// Offer lifecycle status.
    pub status: SyndicationOfferStatus,

    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

impl Model {
    /// Returns the syndication type slugs for this offer.
    pub fn types(&self) -> Vec<String> {
        self.syndication_types
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default()
    }

    /// Returns the billing tier slugs for which this offer is mandatory.
    pub fn mandatory_tiers(&self) -> Vec<String> {
        self.is_mandatory_for_tiers
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default()
    }

    /// Returns true if this offer is mandatory for the given billing tier slug.
    pub fn is_mandatory_for(&self, tier_slug: &str) -> bool {
        self.mandatory_tiers().iter().any(|t| t == tier_slug)
    }
}

/// How a linked NI is presented / functions for the operator.
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, strum_macros::Display)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(30))")]
pub enum SyndicationLinkType {
    /// Operator gets their own branded website showing only their inventory (1:1 coupling).
    #[sea_orm(string_value = "branded_portal")]
    BrandedPortal,
    /// Operator syndicates listings into a shared platform directory (many:1 coupling).
    #[sea_orm(string_value = "marketplace_syndication")]
    MarketplaceSyndication,
}

/// Lifecycle status of a syndication offer.
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, strum_macros::Display)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
pub enum SyndicationOfferStatus {
    #[sea_orm(string_value = "active")]
    Active,
    /// Retired offers: existing links remain but no new activations.
    #[sea_orm(string_value = "retired")]
    Retired,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::atlas_app_deployment_config::Entity",
        from = "Column::NiConfigId",
        to = "super::atlas_app_deployment_config::Column::Id"
    )]
    NiConfig,
    #[sea_orm(has_many = "super::atlas_app_instance_syndication::Entity")]
    ActiveLinks,
}

impl Related<super::atlas_app_deployment_config::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::NiConfig.def()
    }
}

impl Related<super::atlas_app_instance_syndication::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ActiveLinks.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
