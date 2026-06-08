//! SeaORM entity: `atlas_quote_line_items` (G24)

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "atlas_quote_line_items")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub quote_id: Uuid,
    pub line_item_type: String,
    pub catalog_entry_id: Option<Uuid>,
    pub description: String,
    pub quantity: i32,
    pub unit_price_cents: i64,
    pub discount_basis_points: i32,
    pub line_total_cents: i64,
    pub sort_order: i32,
    pub line_metadata: Option<Json>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
