//! SeaORM entity: `atlas_commission_plan_splits` (G25)

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, serde::Serialize, serde::Deserialize)]
#[sea_orm(table_name = "atlas_commission_plan_splits")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub plan_id: Uuid,
    pub tenant_id: Uuid,
    pub recipient_type: String,
    pub recipient_account_id: Option<Uuid>,
    pub recipient_label: Option<String>,
    pub split_basis: String,
    pub split_rate: Decimal,
    pub cap_cents: Option<i64>,
    pub priority: i32,
    pub is_remainder: bool,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
