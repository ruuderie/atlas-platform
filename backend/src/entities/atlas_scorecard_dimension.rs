#![allow(dead_code, unused_imports)]
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

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
    /// Service layer: convert with `crate::types::scorecard::ScaleType::try_from(scale_type.clone())`.
    pub scale_type: String,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    pub scale_min: Decimal,
    #[sea_orm(column_type = "Decimal(Some((10, 2)))")]
    pub scale_max: Decimal,
    /// Unit label displayed to contributors: 'Mbps', 'USD/mo', '°C', 'hrs', '%'
    pub unit_label: Option<String>,
    /// JSONB array of `BenchmarkTier` objects defining what each score range means.
    /// Typed in service code as `crate::types::scorecard::BenchmarkTiers`.
    /// rating/absolute (normal):  [{label, min_score, color}]
    /// rating/absolute (inverted): [{label, max_score, color}]
    /// boolean:                   [{label, min_pct, color}]
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
    /// When true: lower score = better outcome.
    ///
    /// Affects `ScorecardService::recompute_aggregates` in three places:
    ///   1. `dimension_vector` normalization: uses `(max - score)` instead of `(score - min)`
    ///   2. `vs_global_label`: negative delta from reference = "above" (not "below")
    ///   3. Benchmark tier resolution: matches `max_score` keys instead of `min_score`
    ///
    /// Examples: `timeline_slippage`, `competition_risk`, `ramp_to_close`, `air_pollution`
    pub is_inverted: bool,
    /// Bayesian prior weight for cold-start shrinkage.
    ///
    /// NULL = disabled (pure observed mean). Non-null requires `global_reference_value`.
    ///
    /// Applied in `compute_numeric_aggregate`:
    ///   shrunk_mean = (weight × global_reference_value + Σscores) / (weight + n)
    ///
    /// Denominated in equivalent prior observations (weight = 5 → prior = 5 real entries).
    /// Converges to observed mean as n >> weight.
    #[sea_orm(column_type = "Decimal(Some((6, 2)))", nullable)]
    pub bayesian_prior_weight: Option<rust_decimal::Decimal>,
    /// Tenant extension flag — controls cross-tenant benchmark inclusion.
    ///
    /// true  = landlord-added custom dimension; excluded from the cross-tenant benchmark
    ///         aggregation pool. Landlords can add arbitrary custom dimensions for their
    ///         own tracking without polluting the platform canonical benchmark.
    /// false = canonical platform dimension; included in the cross-tenant benchmark pool.
    ///
    /// Default: false (all existing dimensions are canonical — backward compatible).
    /// Added by migration `m20260801_pm_g27_template_scope`.
    pub is_tenant_extension: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
