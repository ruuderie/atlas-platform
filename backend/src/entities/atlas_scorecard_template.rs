use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;
use rust_decimal::Decimal;

/// G-27: atlas_scorecard_templates — defines what traits exist for an entity type.
///
/// One template per (entity_type, tenant). A city template defines which dimensions
/// make sense to rate a city. A contractor template defines job quality dimensions.
/// The engine is identical; only the template differs.
///
/// See: docs/architecture/g27_scorecards_spec.md
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_scorecard_templates")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    /// Discriminator: 'city' | 'person' | 'restaurant' | 'product' | 'contractor' |
    /// 'airline' | 'property' | 'hotel' | 'agent' | 'carrier' | 'event' |
    /// 'atlas_lead' | 'atlas_opportunity' | 'atlas_account'
    pub entity_type: String,
    pub description: Option<String>,
    /// 'weighted_mean' | 'simple_mean' | 'percentile_rank'
    pub scoring_method: String,
    #[sea_orm(column_type = "Decimal(Some((6, 2)))")]
    pub default_scale_min: Decimal,
    #[sea_orm(column_type = "Decimal(Some((6, 2)))")]
    pub default_scale_max: Decimal,
    pub min_entries_to_publish: i32,
    pub is_published: bool,
    pub created_by_user_id: Option<Uuid>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
