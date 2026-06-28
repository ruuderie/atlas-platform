#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// A/B test variant for a platform-admin landing page.
///
/// Each `app_page` row acts as the control / baseline. Variants store
/// independent `blocks_payload` / `hero_payload` overrides and their own
/// engagement counters. Traffic allocation (`traffic_pct`) across all active
/// variants for a given `page_id` should sum to 100.
///
/// Promotion flow: when the platform-admin promotes a winning variant, the
/// handler copies the variant's payloads back to the parent `app_page` and
/// deletes all variant rows for that page.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "app_page_variants")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub page_id: Uuid,
    pub name: String,
    /// Whole-number traffic percentage (0–100). Sum across all active variants
    /// for a page should equal 100 at all times (enforced by handler, not DB).
    pub traffic_pct: i32,
    pub is_control: bool,
    #[sea_orm(column_type = "JsonBinary")]
    pub blocks_payload: Value,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub hero_payload: Option<Value>,
    pub view_count: i32,
    pub lead_count: i32,
    pub is_active: bool,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::app_page::Entity",
        from = "Column::PageId",
        to = "super::app_page::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    AppPage,
}

impl Related<super::app_page::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AppPage.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
