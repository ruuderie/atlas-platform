#![allow(dead_code, unused_imports)]
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// GENERIC-26 (table 2 of 3): AtlasCatalogRateRule
///
/// A pricing override for a catalog entry, scoped by date range, day-of-week
/// bitmask, minimum stay, and/or channel. Multiple rules can apply to a single
/// entry — resolved in descending `priority` order (highest wins).
///
/// Revenue Manager uses this to push dynamic pricing without modifying the
/// base catalog entry. Direct Booking Engine reads these rules to compute
/// the effective nightly rate before creating a quote or reservation.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_catalog_rate_rules")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub catalog_entry_id: Uuid,
    pub tenant_id: Uuid,

    /// Human-readable name for the rule (e.g. "Weekend Premium", "Early-bird -10%").
    pub rule_name: Option<String>,

    /// Inclusive date range the rule applies to. NULL means open-ended.
    pub applies_from: Option<NaiveDate>,
    pub applies_to: Option<NaiveDate>,

    /// Day-of-week bitmask: 1=Mon, 2=Tue, 4=Wed, 8=Thu, 16=Fri, 32=Sat, 64=Sun.
    /// NULL = applies every day.
    pub day_of_week_mask: Option<i32>,

    /// Minimum duration in billing-interval units before this rule applies.
    /// The unit is determined by the parent `atlas_catalog_entry.billing_interval`:
    ///   Nightly → min consecutive nights | Hourly → min hours | Daily → min days
    ///   Weekly → min weeks | Monthly → min billing cycles | PerUnit → min quantity
    /// A NULL means the rule applies regardless of booking length.
    pub min_duration: Option<i32>,

    /// Booking channel scope: 'direct', 'ota', 'gds', 'corporate'. NULL = all.
    pub channel: Option<String>,

    /// Absolute price override in cents. Takes priority over `price_modifier_pct`
    /// if both are set.
    pub price_override_cents: Option<i64>,

    /// Percentage modifier relative to base price (e.g. 20.0 for +20%, -10.0 for -10%).
    #[sea_orm(column_type = "Decimal(Some((6, 2)))", nullable)]
    pub price_modifier_pct: Option<Decimal>,

    /// Higher priority rules are evaluated first. Resolves ties deterministically.
    pub priority: i32,

    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::atlas_catalog_entry::Entity",
        from = "Column::CatalogEntryId",
        to = "super::atlas_catalog_entry::Column::Id",
        on_delete = "Cascade"
    )]
    CatalogEntry,
}

impl Related<super::atlas_catalog_entry::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CatalogEntry.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
