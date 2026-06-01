#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use serde_json::Value;
use rust_decimal::Decimal;

/// G-27: atlas_scorecard_dimensions — individual traits with scale and benchmark tiers.
///
/// Each dimension defines how it is measured (`scale_type`) and what each score
/// level means in plain language (`benchmark_tiers`). The `global_reference_value`
/// is the "bar" that separates above-average from below-average.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_scorecard_dimensions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub template_id: Uuid,
    pub tenant_id: Uuid,
    /// Stable machine-readable identifier within a template. e.g. 'internet_speed'.
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    /// Contribution weight in weighted_mean scoring. Default 1.0.
    #[sea_orm(column_type = "Decimal(Some((5, 4)))")]
    pub weight: Decimal,
    /// 'rating' | 'absolute' | 'boolean' | 'poll_single' | 'poll_multi'
    pub scale_type: String,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    pub scale_min: Decimal,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    pub scale_max: Decimal,
    /// Unit label displayed to contributors: 'Mbps', 'USD/mo', '°C', 'hrs', '%'
    pub unit_label: Option<String>,
    /// JSONB array of tier objects defining what each score range means.
    /// rating/absolute: [{label, min_score, color}]
    /// boolean: [{label, min_pct, color}]
    #[sea_orm(column_type = "JsonBinary")]
    pub benchmark_tiers: Value,
    /// The global "bar" value for above/below-bar comparison.
    #[sea_orm(column_type = "Decimal(Some((10, 2)))", nullable)]
    pub global_reference_value: Option<Decimal>,
    pub global_reference_label: Option<String>,
    pub min_entries_to_show: i32,
    pub is_community_ratable: bool,
    pub is_active: bool,
    pub sort_order: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
