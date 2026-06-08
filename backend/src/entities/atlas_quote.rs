//! SeaORM entity: `atlas_quotes` (G24)

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "atlas_quotes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub subject_entity_type: Option<String>,
    pub subject_entity_id: Option<Uuid>,
    pub recipient_user_id: Option<Uuid>,
    pub recipient_email: Option<String>,
    pub recipient_name: Option<String>,
    pub campaign_id: Option<Uuid>,
    pub catalog_entry_id: Option<Uuid>,
    pub quote_number: Option<String>,
    pub title: String,
    pub notes: Option<String>,
    pub status: String,
    pub subtotal_cents: i64,
    pub discount_cents: i64,
    pub tax_cents: i64,
    pub total_cents: i64,
    pub currency: String,
    pub valid_from: Option<DateTimeWithTimeZone>,
    pub valid_until: Option<DateTimeWithTimeZone>,
    pub accepted_at: Option<DateTimeWithTimeZone>,
    pub rejected_at: Option<DateTimeWithTimeZone>,
    pub converted_reservation_id: Option<Uuid>,
    pub revision_number: i32,
    pub superseded_by_id: Option<Uuid>,
    pub quote_metadata: Option<Json>,
    pub created_by_user_id: Option<Uuid>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
