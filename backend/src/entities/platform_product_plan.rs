//! Sea-ORM entity for `platform_product_plans`.
//!
//! Product-scoped marketing pricing tiers shown on public landing pages.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "platform_product_plans")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub product_id: Uuid,
    pub slug: String,
    pub name: String,
    pub tagline: String,
    pub price_cents: i32,
    pub currency: String,
    pub billing_interval: ProductPlanBillingInterval,
    pub features: Value,
    pub cta_label: String,
    pub cta_href: Option<String>,
    pub is_featured: bool,
    pub sort_order: i32,
    pub is_active: bool,
    pub billing_plan_id: Option<Uuid>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

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
#[serde(rename_all = "snake_case")]
pub enum ProductPlanBillingInterval {
    #[sea_orm(string_value = "month")]
    Month,
    #[sea_orm(string_value = "year")]
    Year,
    #[sea_orm(string_value = "forever")]
    Forever,
    #[sea_orm(string_value = "custom")]
    Custom,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::platform_product::Entity",
        from = "Column::ProductId",
        to = "super::platform_product::Column::Id"
    )]
    PlatformProduct,
    #[sea_orm(
        belongs_to = "super::billing_plan::Entity",
        from = "Column::BillingPlanId",
        to = "super::billing_plan::Column::Id"
    )]
    BillingPlan,
}

impl Related<super::platform_product::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PlatformProduct.def()
    }
}

impl Related<super::billing_plan::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BillingPlan.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
