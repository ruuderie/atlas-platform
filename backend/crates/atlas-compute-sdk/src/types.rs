//! `types` — JSON-serialisable request/response types for the BYOC API.
//!
//! These are the types that cross the host/WASM boundary. All fields use
//! standard JSON-compatible primitives so any Lambda runtime can serialise them.

use serde::{Deserialize, Serialize};

/// One entry from a contributor for a single dimension.
///
/// The host prepopulates `bias_offset` and `scale_factor` from
/// `atlas_scorecard_contributor_calibration` so the WASM module never touches
/// the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryInput {
    /// Raw numeric score submitted by the contributor. `None` for poll/boolean entries.
    pub score: Option<f64>,
    /// Credibility weight from `context` JSONB (duration_days / 30, clamped 0.5–3.0).
    /// Defaults to 1.0 if absent.
    pub credibility_weight: Option<f64>,
    /// Phase 4: per-contributor, per-dimension bias correction (subtracted from raw score).
    /// 0.0 = no correction (identity).
    pub bias_offset: Option<f64>,
    /// Phase 4: per-contributor scale correction (applied after bias).
    /// 1.0 = no correction (identity).
    pub scale_factor: Option<f64>,
}

/// Full compute request for one dimension aggregate.
///
/// Atlas Platform constructs this from the DB and sends it to the BYOC Lambda.
/// The Lambda passes it to `atlas_compute_sdk::compute()` and returns the response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeRequest {
    /// All verified entries for this (scorecard, dimension) pair.
    pub entries: Vec<EntryInput>,

    /// Dimension scale bounds — used to clamp calibrated scores.
    pub scale_min: f64,
    pub scale_max: f64,

    /// Phase 1: Bayesian prior weight (James-Stein shrinkage strength).
    /// `None` = no shrinkage.
    pub bayesian_prior_weight: Option<f64>,
    /// Phase 1: Global reference value used as the shrinkage prior mean.
    pub global_reference_value: Option<f64>,

    /// Phase 1: Saturation threshold for confidence weighting.
    /// Defaults to 50.0 if absent.
    pub saturation_threshold: Option<f64>,
}

/// Aggregate result returned by the BYOC Lambda.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeResponse {
    /// Credibility-weighted, Bayesian-shrunk, calibrated mean.
    /// `None` when all entries have no numeric score.
    pub weighted_mean: Option<f64>,
    /// Confidence weight in [0.0, 1.0] — how saturated this dimension's data is.
    pub confidence_weight: f64,
    /// Number of entries that contributed a numeric score.
    pub contributor_count: u32,
}
