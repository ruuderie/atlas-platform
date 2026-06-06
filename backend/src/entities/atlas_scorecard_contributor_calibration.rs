#![allow(dead_code, unused_imports)]
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

/// G-27 Data Science Upgrade — Gap 2: Contributor Bias Calibration
///
/// Stores per-contributor bias correction coefficients computed by the weekly
/// background job `ScorecardService::calibrate_contributor_bias_all()`.
///
/// # Activation threshold
/// Calibration is ONLY applied in `compute_numeric_aggregate` when
/// `entry_count >= 100`. Below this threshold, the offset/scale factor
/// would be computed from too little data and could increase noise rather
/// than reduce it. The table is populated regardless; the service checks
/// `entry_count` before applying corrections.
///
/// # Primary key layout
/// The composite PK is `(contributor_user_id, template_id, dimension_id)`.
/// A NULL `dimension_id` represents a template-level calibration applied
/// uniformly across all dimensions. Dimension-level calibrations are applied
/// first; template-level is a fallback.
///
/// # Math
/// ```text
/// calibrated_score = (raw_score - bias_offset) / scale_factor.max(0.1)
/// bias_offset = contributor_mean - ensemble_mean (for that dimension/template)
/// scale_factor = contributor_std / ensemble_std
/// ```
///
/// A `scale_factor` of 1.0 (default) applies no scaling — only the offset.
/// A `scale_factor` of 2.0 means this contributor has twice the spread of the
/// ensemble — their scores are compressed by factor 2.
///
/// # Populated by
/// `ScorecardService::calibrate_contributor_bias(db, template_id, tenant_id)`
/// → called weekly by the background worker in `main.rs`.
///
/// Do NOT write to this table directly.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atlas_scorecard_contributor_calibration")]
pub struct Model {
    /// Surrogate UUID primary key.
    /// Natural uniqueness is (contributor_user_id, template_id, dimension_id)
    /// enforced by a UNIQUE INDEX in the migration.
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// The contributor whose scoring behavior is being calibrated.
    pub contributor_user_id: Uuid,
    /// Template scope of this calibration. FK → atlas_scorecard_templates.
    pub template_id: Uuid,
    /// Dimension scope. NULL = template-level calibration (applies to all dims).
    /// FK → atlas_scorecard_dimensions.
    pub dimension_id: Option<Uuid>,
    /// Additive bias correction: contributor_mean - ensemble_mean.
    ///
    /// Positive value: contributor tends to score higher than ensemble.
    /// Applied: calibrated_score = raw_score - bias_offset.
    /// Range: typically ±3.0 on a 1–10 scale.
    #[sea_orm(column_type = "Decimal(Some((6, 3)))")]
    pub bias_offset: Decimal,
    /// Multiplicative scale correction: contributor_std / ensemble_std.
    ///
    /// Applied after bias offset: calibrated_score /= scale_factor.
    /// 1.0 = no scaling. Clamped to minimum 0.1 to prevent division by near-zero.
    #[sea_orm(column_type = "Decimal(Some((6, 3)))")]
    pub scale_factor: Decimal,
    /// Number of entries used to compute these coefficients.
    ///
    /// Calibration is NOT applied if entry_count < 100. The service checks
    /// this field before applying corrections to avoid noise amplification.
    pub entry_count: i32,
    /// When this calibration was last computed by the background job.
    #[sea_orm(column_type = "TimestampWithTimeZone", nullable)]
    pub last_calibrated_at: Option<DateTime<Utc>>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
