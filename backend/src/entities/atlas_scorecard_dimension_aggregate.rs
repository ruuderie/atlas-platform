#![allow(dead_code, unused_imports)]
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// G-27: atlas_scorecard_dimension_aggregates — rolled-up community scores per dimension.
///
/// PRIMARY KEY: (scorecard_id, dimension_id) — one row per dimension per scorecard.
/// Recomputed atomically by `ScorecardService::recompute_aggregates`.
/// Never write to this table directly — always go through the service.
///
/// `display_value` is the pre-formatted human-readable label:
///   - "Fast: 16 Mbps" (absolute with unit)
///   - "7.3/10" (rating without unit)
///   - "83% say clean" (boolean)
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_scorecard_dimension_aggregates")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub scorecard_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub dimension_id: Uuid,
    /// For rating / absolute dimensions.
    #[sea_orm(column_type = "Decimal(Some((5, 2)))", nullable)]
    pub mean_score: Option<Decimal>,
    /// Credibility-weighted mean. This is the canonical score shown to end users.
    #[sea_orm(column_type = "Decimal(Some((5, 2)))", nullable)]
    pub weighted_mean_score: Option<Decimal>,
    /// For boolean dimensions: percentage of true responses (0-100).
    #[sea_orm(column_type = "Decimal(Some((5, 2)))", nullable)]
    pub percent_true: Option<Decimal>,
    /// Resolved benchmark tier label from benchmark_tiers JSONB.
    pub benchmark_label: Option<String>,
    /// Hex color for the resolved benchmark tier.
    pub benchmark_color: Option<String>,
    /// Pre-formatted display string: "Fast: 16 Mbps", "$1,183/mo", "83% say clean"
    pub display_value: Option<String>,
    #[sea_orm(column_type = "Decimal(Some((5, 2)))", nullable)]
    pub std_deviation: Option<Decimal>,
    /// 'strong_consensus' | 'consensus' | 'mixed' | 'disputed'
    pub consensus_level: Option<String>,
    #[sea_orm(column_type = "Decimal(Some((5, 2)))", nullable)]
    pub min_score: Option<Decimal>,
    #[sea_orm(column_type = "Decimal(Some((5, 2)))", nullable)]
    pub max_score: Option<Decimal>,
    pub contributor_count: i32,
    pub session_count: i32,
    /// Delta from global_reference_value (weighted_mean - reference).
    #[sea_orm(column_type = "Decimal(Some((8, 2)))", nullable)]
    pub vs_global_delta: Option<Decimal>,
    /// 'above' | 'at' | 'below'
    pub vs_global_label: Option<String>,
    /// Percentile rank within the tenant pool for this (template, dimension).
    ///
    /// Range: 0.0–100.0. Computed by `ScorecardService::compute_percentile_ranks()`
    /// after the aggregate is written. NULL until computed (requires >= 2 scorecards).
    /// Interpretation: rank=85 means this entity outperforms 85% of the tenant pool.
    #[sea_orm(column_type = "Decimal(Some((5, 2)))", nullable)]
    pub percentile_rank: Option<Decimal>,
    /// Number of entities in the comparison cohort.
    /// Shown as "Top X% of N entities" in UI.
    pub percentile_cohort_size: Option<i32>,
    /// Categorical band derived from percentile_rank.
    ///
    /// 'top_10'       : rank >= 90
    /// 'top_quartile' : rank >= 75
    /// 'median'       : rank >= 50
    /// 'bottom_quartile': rank < 50
    pub percentile_band: Option<String>,
    #[sea_orm(column_type = "TimestampWithTimeZone", nullable)]
    pub last_computed_at: Option<DateTime<Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
