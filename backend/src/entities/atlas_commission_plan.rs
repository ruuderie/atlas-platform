//! SeaORM entity: `atlas_commission_plans` (G25)

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "atlas_commission_plans")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub plan_type: String,
    pub is_active: bool,
    pub commission_basis: String,
    pub default_rate: Option<Decimal>,
    pub tiers: Option<Json>,
    pub cap_cents: Option<i64>,
    pub minimum_cents: Option<i64>,
    pub clawback_days: Option<i32>,
    pub applies_to_entity_type: Option<String>,
    pub applies_to_entity_id: Option<Uuid>,
    pub applies_to_ledger_type: Option<String>,
    pub recipient_type: Option<String>,
    pub effective_from: Date,
    pub effective_to: Option<Date>,
    pub created_by_user_id: Option<Uuid>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
