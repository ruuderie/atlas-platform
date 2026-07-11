#![allow(dead_code, unused_imports)]
use chrono::{DateTime, NaiveDate, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// GENERIC-26 (table 1 of 3): AtlasCatalogEntry
///
/// A saleable product definition: what can be sold, at what base price,
/// and over what billing interval. The middle layer between G10 (what you *own*)
/// and G24 (what you *quote*).
///
/// Covers: room types (hotel/STR), service slots, package tiers,
/// subscription tiers, insurance coverage options, equipment units, add-ons.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_catalog_entries")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,

    /// Discriminates the product category. Values: room_type, service_slot,
    /// package_tier, subscription_tier, coverage_option, add_on, equipment_unit.
    pub entry_type: String,

    pub name: String,
    pub description: Option<String>,

    /// Optional link to the underlying G10 asset (e.g. room type → room unit).
    pub asset_id: Option<Uuid>,

    /// Base price in the smallest currency unit (cents).
    pub base_price_cents: i64,

    /// ISO 4217 currency code.
    pub currency: String,

    /// NULL = one-time purchase. Otherwise: 'nightly', 'monthly', 'annually'.
    pub billing_interval: Option<String>,

    pub is_available: bool,
    pub min_quantity: i32,
    pub max_quantity: Option<i32>,

    /// App-specific product attributes as JSONB.
    /// Examples:
    ///   room_type:          {bed_type, max_occupancy, view_type, amenities[]}
    ///   subscription_tier:  {feature_flags[], max_uploads, ai_credits}
    ///   coverage_option:    {limit_cents, deductible_cents, coverage_type}
    pub catalog_metadata: Json,

    pub sort_order: i32,
    pub cover_image_attachment_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::atlas_catalog_rate_rule::Entity")]
    RateRules,
    #[sea_orm(has_many = "super::atlas_catalog_availability::Entity")]
    AvailabilitySlots,
}

impl Related<super::atlas_catalog_rate_rule::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RateRules.def()
    }
}

impl Related<super::atlas_catalog_availability::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AvailabilitySlots.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
