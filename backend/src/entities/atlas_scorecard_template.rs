#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
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
    /// Cold-start display strategy when entry_count < min_entries_to_publish.
    ///
    /// Values (see `crate::types::scorecard::ColdStartStrategy`):
    /// - 'suppress': hide score entirely (current default)
    /// - 'prior': show global_reference_value as Bayesian prior with 'Estimated' label
    /// - 'category': maps to 'prior' until category-pool averaging is implemented
    pub cold_start_strategy: String,
    /// Number of distinct contributors at which confidence_weight = 1.0 (fully saturated).
    ///
    /// confidence_weight = MIN(contributor_count / threshold, 1.0)
    /// Applied per-dimension in composite weighting. Default: 50.
    pub cold_start_saturation_threshold: i32,
    /// Template-wide Bayesian prior weight fallback (Decision 4 — hierarchical lookup).
    ///
    /// Applied when a dimension has `bayesian_prior_weight = NULL`.
    /// Lookup order:
    ///   1. `atlas_scorecard_dimensions.bayesian_prior_weight` (dimension override)
    ///   2. This field (template default)
    ///   3. NULL → no shrinkage (current behavior, backward compatible)
    ///
    /// Recommended starting value: 5.0 (prior = 5 equivalent observations).
    /// NULL = disabled (no template-level default, shrinkage disabled unless dim-specific).
    #[sea_orm(column_type = "Decimal(Some((5, 2)))", nullable)]
    pub default_bayesian_prior_weight: Option<Decimal>,
    /// Per-template contributor calibration activation threshold (Decision 3).
    ///
    /// Calibration (bias_offset, scale_factor) is applied in `compute_numeric_aggregate`
    /// only when a contributor's `entry_count` for this template is >= this value.
    /// Below the threshold, raw scores are used unmodified.
    ///
    /// Different templates have different data velocities:
    /// - High-volume marketplace template: may hit 100 entries/week → calibrate early
    /// - Enterprise deal health template: may take 18 months → need lower threshold
    /// Default: 100 (backward compatible — calibration not yet activated anywhere).
    pub calibration_minimum_entries: i32,
    pub created_by_user_id: Option<Uuid>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
