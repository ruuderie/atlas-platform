//! Sea-ORM entity for `atlas_app_instance_syndication`.
//!
//! Platform-generic active link between a source app instance (Folio, or any future Atlas app)
//! and a destination Network Instance. This is Layer B of the two-layer syndication model.
//!
//! # Creation paths
//!
//! 1. **Operator self-service**: operator activates an offer in their instance settings
//!    (requires `atlas_syndication_offer.self_service_allowed = true`).
//! 2. **Platform admin manual**: admin creates the link directly without an offer template.
//! 3. **Auto-provisioning**: at instance creation, mandatory offers for the tenant's billing
//!    tier are automatically activated with `is_mandatory = true`.
//!
//! # Bidirectional event contract (G-05 outbox pattern)
//!
//! **Outbound (source → NI)**: When a listing is published/updated in the source app,
//! an outbox event is fired. The syndication worker reads active links for that source
//! instance and dispatches the payload to each linked NI's integration endpoint.
//! All events logged in `atlas_integration_events`.
//!
//! **Inbound (NI → source)**: The NI posts events (inquiries, applications, signups)
//! to `inbound_webhook_url`. The source app verifies HMAC-SHA256 using
//! `inbound_webhook_secret` and routes to CRM / lease workflow.
//!
//! # Same-tenant vs cross-tenant
//!
//! Both go through this table. The link has properties (types, mandatory flag, webhook)
//! that are independent of ownership. Consistent with Shopify, Stripe Connect, and
//! Airbnb marketplace architecture patterns.
//!
//! Created by migration `m20260913_atlas_app_instance_syndication`.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_app_instance_syndication")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,

    /// FK to `atlas_app_deployment_config` — the source app instance.
    pub source_config_id: Uuid,

    /// FK to `atlas_app_deployment_config` — the destination NetworkInstance.
    pub ni_config_id: Uuid,

    /// FK to `atlas_syndication_offer` — the offer template governing this link.
    /// None = manually created by platform admin without an offer template.
    pub offer_id: Option<Uuid>,

    /// Effective syndication types for this link.
    /// Derived from the offer at creation; can be narrowed (not expanded) by operator.
    pub syndication_types: Value,

    /// How this NI is presented / functions for this operator.
    pub link_type: super::atlas_syndication_offer::SyndicationLinkType,

    /// If true, this link was auto-created due to a mandatory offer rule.
    /// Mandatory links cannot be revoked by the operator.
    pub is_mandatory: bool,

    /// Link lifecycle.
    pub status: SyndicationStatus,

    /// URL on the source app side that the NI posts inbound events to.
    /// None = unidirectional outbound only.
    pub inbound_webhook_url: Option<String>,

    /// HMAC-SHA256 secret for verifying inbound events from the NI.
    pub inbound_webhook_secret: Option<String>,

    /// The operator tenant that created this link.
    pub created_by_tenant_id: Uuid,

    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

impl Model {
    /// Returns the effective syndication type slugs for this link.
    pub fn active_types(&self) -> Vec<String> {
        self.syndication_types
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Returns true if this link syndicates the given listing type.
    pub fn includes_type(&self, type_slug: &str) -> bool {
        self.active_types().iter().any(|t| t == type_slug)
    }

    /// Returns true if inbound events from NI are configured.
    pub fn has_inbound_webhook(&self) -> bool {
        self.inbound_webhook_url.is_some()
    }

    /// Returns true if this is an active link (not paused or revoked).
    pub fn is_active(&self) -> bool {
        self.status == SyndicationStatus::Active
    }
}

/// Lifecycle status of an active syndication link.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    strum_macros::Display,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
pub enum SyndicationStatus {
    #[sea_orm(string_value = "active")]
    Active,
    /// Operator has paused syndication; link remains, events stop flowing.
    #[sea_orm(string_value = "paused")]
    Paused,
    /// Link has been revoked; a new link can be created if not mandatory.
    #[sea_orm(string_value = "revoked")]
    Revoked,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::atlas_app_deployment_config::Entity",
        from = "Column::SourceConfigId",
        to = "super::atlas_app_deployment_config::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    SourceConfig,
    #[sea_orm(
        belongs_to = "super::atlas_app_deployment_config::Entity",
        from = "Column::NiConfigId",
        to = "super::atlas_app_deployment_config::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    NiConfig,
    #[sea_orm(
        belongs_to = "super::atlas_syndication_offer::Entity",
        from = "Column::OfferId",
        to = "super::atlas_syndication_offer::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    Offer,
}

impl Related<super::atlas_syndication_offer::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Offer.def()
    }
}

// Note: We do NOT implement Related<atlas_app_deployment_config::Entity> here because
// this entity has TWO foreign keys to that same table (source_config_id and ni_config_id).
// Callers must specify which relation they want explicitly:
//   Entity::find().join(JoinType::InnerJoin, Relation::SourceConfig.def())
//   Entity::find().join(JoinType::InnerJoin, Relation::NiConfig.def())

impl ActiveModelBehavior for ActiveModel {}
