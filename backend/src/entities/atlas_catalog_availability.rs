#![allow(dead_code, unused_imports)]
use chrono::{DateTime, NaiveDate, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// GENERIC-26 (table 3 of 3): AtlasCatalogAvailability
///
/// A per-date slot in the availability grid for a catalog entry.
/// Each row represents one calendar day for one product (room type, service slot, etc.),
/// tracking total inventory capacity, how many are reserved, whether the date is manually
/// blocked, and any day-specific price override.
///
/// `available_count` is a STORED GENERATED ALWAYS column computed as
/// `total_inventory - reserved_count` — managed by PostgreSQL, never set by application code.
///
/// `CatalogService::reserve_slots()` increments `reserved_count`.
/// `CatalogService::release_slots()` decrements it.
/// Both use a `SELECT ... FOR UPDATE` row lock to prevent double-booking.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_catalog_availability")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub catalog_entry_id: Uuid,
    pub tenant_id: Uuid,

    /// The calendar date this slot represents.
    pub slot_date: NaiveDate,

    /// How many units of inventory exist for this slot (e.g. 3 rooms of this type).
    pub total_inventory: i32,

    /// How many units are already reserved (committed from atlas_reservations).
    pub reserved_count: i32,

    /// GENERATED ALWAYS AS (total_inventory - reserved_count) STORED.
    /// Read-only — never set by application code.
    pub available_count: i32,

    /// Manual operator block (cleaning day, owner hold, maintenance).
    pub is_blocked: bool,
    pub block_reason: Option<String>,

    /// Day-specific absolute price override. Takes precedence over rate rules.
    pub override_price_cents: Option<i64>,
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
