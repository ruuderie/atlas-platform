#![allow(dead_code, unused_imports)]
use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Reusable UTM parameter set for the platform-admin Landing Page Builder.
///
/// Presets are scoped by `app_id` so each platform product maintains its
/// own campaign tracking templates independently. The URL Builder in the
/// platform-admin combines a preset with a page slug and a domain to
/// generate a fully-tagged acquisition URL.
///
/// `click_count` is denormalized and incremented asynchronously by the
/// link-click telemetry webhook when that integration is active.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "app_utm_presets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// Platform product this preset belongs to ("folio", "ruuderie", "network").
    pub app_id: String,
    pub name: String,
    pub utm_source: String,
    pub utm_medium: String,
    pub utm_campaign: String,
    pub utm_content: Option<String>,
    pub utm_term: Option<String>,
    pub click_count: i32,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
