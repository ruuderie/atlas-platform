//! Entity: atlas_lp_events — landing page funnel analytics events

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_lp_events")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub app_page_id: Uuid,
    pub event_type: String, // "view" | "lead_submitted" | "cta_click"
    pub session_id: String,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_campaign: Option<String>,
    pub utm_content: Option<String>,
    pub utm_term: Option<String>,
    pub viewport: Option<String>,
    pub referrer: Option<String>,
    pub country_code: Option<String>,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::app_page::Entity",
        from = "Column::AppPageId",
        to = "super::app_page::Column::Id",
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
