//! `calibration` — Per-contributor bias correction.
//!
//! Extracted from `ScorecardService::compute_numeric_aggregate` (Phase 4 application)
//! and `ScorecardService::calibrate_contributor_bias` (Phase 4 computation).

/// Apply a (bias_offset, scale_factor) calibration to a raw score.
///
/// Formula: `calibrated = clamp((raw - bias) × scale, scale_min, scale_max)`
///
/// Identity: `bias=0.0, scale=1.0` → `calibrated == raw` (clamped).
///
/// # Arguments
/// - `raw`       — the raw numeric score as submitted by the contributor
/// - `bias`      — additive correction (positive = contributor scores high vs. ensemble)
/// - `scale`     — multiplicative correction (applied after bias subtraction)
/// - `scale_min` — dimension minimum (lower clamp bound)
/// - `scale_max` — dimension maximum (upper clamp bound)
pub fn apply_calibration(raw: f64, bias: f64, scale: f64, scale_min: f64, scale_max: f64) -> f64 {
    ((raw - bias) * scale).clamp(scale_min, scale_max)
}

/// Compute the bias offset for a contributor on a dimension.
///
/// `bias_offset = contributor_mean − ensemble_mean`
///
/// A positive value means the contributor rates higher than the ensemble.
/// Subtracting this in `apply_calibration` normalises their scores downward.
pub fn compute_bias_offset(contributor_mean: f64, ensemble_mean: f64) -> f64 {
    contributor_mean - ensemble_mean
}

/// Compute the scale factor for a contributor on a dimension.
///
/// `scale_factor = σ_contributor / σ_ensemble`
///
/// Clamped to [0.1, 3.0] to prevent extreme corrections from sparse data.
/// Returns 1.0 (identity) if either standard deviation is near-zero (< 0.01).
///
/// # Arguments
/// - `contributor_std` — standard deviation of the contributor's scores for this dimension
/// - `ensemble_std`    — standard deviation of all contributors' scores for this dimension
pub fn compute_scale_factor(contributor_std: f64, ensemble_std: f64) -> f64 {
    if ensemble_std > 0.01 && contributor_std > 0.01 {
        (contributor_std / ensemble_std).clamp(0.1, 3.0)
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── apply_calibration ────────────────────────────────────────────────────

    #[test]
    fn calibration_identity_when_bias_zero_scale_one() {
        assert!((apply_calibration(7.0, 0.0, 1.0, 0.0, 10.0) - 7.0).abs() < 1e-10);
    }

    #[test]
    fn calibration_shifts_down_when_positive_bias() {
        // bias=+2.0 → (7.0 - 2.0) × 1.0 = 5.0
        assert!((apply_calibration(7.0, 2.0, 1.0, 0.0, 10.0) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn calibration_shifts_up_when_negative_bias() {
        // bias=-1.0 → (7.0 - (-1.0)) × 1.0 = 8.0
        assert!((apply_calibration(7.0, -1.0, 1.0, 0.0, 10.0) - 8.0).abs() < 1e-10);
    }

    #[test]
    fn calibration_scales_score() {
        // bias=0, scale=0.5 → 8.0 × 0.5 = 4.0
        assert!((apply_calibration(8.0, 0.0, 0.5, 0.0, 10.0) - 4.0).abs() < 1e-10);
    }

    #[test]
    fn calibration_clamps_to_scale_min() {
        // (1.0 - 5.0) × 1.0 = -4.0 → clamped to 0.0
        assert!((apply_calibration(1.0, 5.0, 1.0, 0.0, 10.0) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn calibration_clamps_to_scale_max() {
        // (9.0 - (-5.0)) × 1.0 = 14.0 → clamped to 10.0
        assert!((apply_calibration(9.0, -5.0, 1.0, 0.0, 10.0) - 10.0).abs() < 1e-10);
    }

    // ── compute_bias_offset ──────────────────────────────────────────────────

    #[test]
    fn bias_positive_when_contributor_scores_high() {
        assert!((compute_bias_offset(8.0, 5.0) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn bias_zero_when_aligned_with_ensemble() {
        assert!((compute_bias_offset(5.0, 5.0)).abs() < 1e-10);
    }

    #[test]
    fn bias_negative_when_contributor_scores_low() {
        assert!((compute_bias_offset(3.0, 5.0) - (-2.0)).abs() < 1e-10);
    }

    // ── compute_scale_factor ─────────────────────────────────────────────────

    #[test]
    fn scale_factor_one_when_stds_equal() {
        assert!((compute_scale_factor(2.0, 2.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn scale_factor_one_when_ensemble_std_near_zero() {
        assert!((compute_scale_factor(1.0, 0.001) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn scale_factor_one_when_contributor_std_near_zero() {
        assert!((compute_scale_factor(0.001, 1.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn scale_factor_clamped_below_01() {
        // contributor_std=0.05, ensemble_std=2.0 → 0.025 → clamped to 0.1
        assert!((compute_scale_factor(0.05, 2.0) - 0.1).abs() < 1e-10);
    }

    #[test]
    fn scale_factor_clamped_above_30() {
        // contributor_std=10.0, ensemble_std=0.1 → 100 → clamped to 3.0
        assert!((compute_scale_factor(10.0, 0.1) - 3.0).abs() < 1e-10);
    }
}
