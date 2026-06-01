#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;
use rust_decimal::Decimal;

/// G-27: atlas_scorecards — a template applied to one specific entity instance.
///
/// Polymorphic via `subject_entity_type` + `subject_entity_id`. Any entity in the
/// platform can have a scorecard. The UNIQUE constraint on
/// (template_id, subject_entity_type, subject_entity_id) ensures one scorecard
/// per entity per template.
///
/// `composite_score` and `dimension_vector` are recomputed by the background job
/// `recompute_scorecard_aggregates` (5-min interval). Never write these directly.
///
/// `dimension_vector` is stored as JSONB (Vec<f64>) because SeaORM has no native
/// DECIMAL[] type. The Combinator similarity search reads and computes in Rust.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_scorecards")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub template_id: Uuid,
    /// Type discriminator for the rated entity.
    /// 'atlas_asset' | 'listing' | 'atlas_catalog_entry' | 'atlas_account' |
    /// 'atlas_service_provider' | 'profile' | 'atlas_opportunity' |
    /// 'atlas_lead' | 'atlas_contact' | 'atlas_portfolio'
    pub subject_entity_type: String,
    pub subject_entity_id: Uuid,
    /// Computed weighted composite (None until min_entries_to_publish met).
    /// Recomputed by background job — do NOT write directly.
    #[sea_orm(column_type = "Decimal(Some((5, 2)))", nullable)]
    pub composite_score: Option<Decimal>,
    /// 'insufficient'(<5) | 'low'(<10) | 'medium'(<50) | 'high'(<200) | 'very_high'
    pub confidence_level: String,
    pub total_contributors: i32,
    pub total_sessions: i32,
    pub total_entries: i32,
    /// Ordered Vec<f64> of weighted normalized scores per dimension (by sort_order).
    /// Normalized: (score - scale_min) / (scale_max - scale_min) * weight
    /// Zero-filled for dimensions with no entries.
    /// Used by The Combinator for Euclidean distance similarity search.
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub dimension_vector: Option<Value>,
    #[sea_orm(column_type = "TimestampWithTimeZone", nullable)]
    pub last_computed_at: Option<DateTime<Utc>>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
