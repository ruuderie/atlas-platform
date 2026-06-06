//! `aggregate` — Credibility-weighted mean + Bayesian shrinkage + confidence weight.
//!
//! These are the core aggregation functions extracted from
//! `ScorecardService::compute_numeric_aggregate` and `recompute_aggregates`.
//! All functions are pure (no I/O, no panics for valid inputs).

/// Compute the credibility-weighted mean from a slice of `(score, weight)` pairs.
///
/// Returns `None` if the slice is empty or total weight is zero.
///
/// This mirrors the credibility-weighted sum in `compute_numeric_aggregate`:
/// ```text
/// weighted_mean = Σ(score_i × weight_i) / Σ(weight_i)
/// ```
pub fn weighted_mean(entries: &[(f64, f64)]) -> Option<f64> {
    let (score_sum, weight_total) = entries
        .iter()
        .fold((0.0f64, 0.0f64), |(ss, wt), (s, w)| (ss + s * w, wt + w));

    if weight_total > 0.0 {
        Some(score_sum / weight_total)
    } else {
        None
    }
}

/// Apply James-Stein Bayesian shrinkage to a raw weighted mean.
///
/// Pulls the observed mean toward a global reference prior:
/// ```text
/// shrunk = (prior_weight × global_ref + weight_total × raw_mean)
///          ──────────────────────────────────────────────────────
///                  prior_weight + weight_total
/// ```
///
/// Returns `raw_mean` unchanged if any of prior_weight, global_ref are absent,
/// or if prior_weight <= 0.
///
/// # Arguments
/// - `raw_mean`        — credibility-weighted mean (from `weighted_mean`)
/// - `weight_total`    — Σ(credibility weights) for all entries
/// - `prior_weight`    — shrinkage strength (e.g. 5.0 = "5 equivalent observations")
/// - `global_ref`      — the prior mean (global reference value for the dimension)
pub fn bayesian_shrinkage(
    raw_mean:     Option<f64>,
    weight_total: f64,
    prior_weight: Option<f64>,
    global_ref:   Option<f64>,
) -> Option<f64> {
    match (raw_mean, prior_weight, global_ref) {
        (Some(m), Some(pw), Some(gr)) if pw > 0.0 => {
            // Reconstruct the weighted sum from the mean
            let weighted_sum = m * weight_total;
            Some((pw * gr + weighted_sum) / (pw + weight_total))
        }
        _ => raw_mean,
    }
}

/// Compute the confidence weight for a dimension aggregate.
///
/// Represents how "saturated" the pool is relative to the template's
/// `cold_start_saturation_threshold`. Clamps to [0.0, 1.0].
///
/// ```text
/// confidence_weight = MIN(contributor_count / threshold, 1.0)
/// ```
///
/// - `contributor_count = 0, threshold = 50` → 0.0
/// - `contributor_count = 25, threshold = 50` → 0.5
/// - `contributor_count >= 50, threshold = 50` → 1.0
pub fn confidence_weight(contributor_count: f64, saturation_threshold: f64) -> f64 {
    if saturation_threshold <= 0.0 {
        return 1.0;
    }
    (contributor_count / saturation_threshold).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weighted_mean_empty_returns_none() {
        assert_eq!(weighted_mean(&[]), None);
    }

    #[test]
    fn weighted_mean_single_entry() {
        assert_eq!(weighted_mean(&[(7.0, 1.0)]), Some(7.0));
    }

    #[test]
    fn weighted_mean_equal_weights_is_arithmetic_mean() {
        let result = weighted_mean(&[(4.0, 1.0), (6.0, 1.0)]).unwrap();
        assert!((result - 5.0).abs() < 1e-10);
    }

    #[test]
    fn weighted_mean_higher_weight_pulls_mean() {
        // score=9 has weight 3, score=3 has weight 1 → mean = (27+3)/4 = 7.5
        let result = weighted_mean(&[(9.0, 3.0), (3.0, 1.0)]).unwrap();
        assert!((result - 7.5).abs() < 1e-10);
    }

    #[test]
    fn bayesian_shrinkage_blends_toward_prior() {
        // prior_weight=5, global_ref=5.0, raw_mean=9.0, weight_total=2.0
        // shrunk = (5×5 + 2×9) / (5+2) = (25+18)/7 = 43/7 ≈ 6.14
        let result = bayesian_shrinkage(Some(9.0), 2.0, Some(5.0), Some(5.0)).unwrap();
        assert!(result > 5.0, "shrunk must be > prior (prior is 5.0, result={result})");
        assert!(result < 9.0, "shrunk must be < raw mean (raw is 9.0, result={result})");
    }

    #[test]
    fn bayesian_shrinkage_identity_when_no_prior() {
        assert_eq!(bayesian_shrinkage(Some(7.0), 3.0, None, Some(5.0)), Some(7.0));
        assert_eq!(bayesian_shrinkage(Some(7.0), 3.0, Some(5.0), None), Some(7.0));
    }

    #[test]
    fn bayesian_shrinkage_none_when_no_raw_mean() {
        assert_eq!(bayesian_shrinkage(None, 0.0, Some(5.0), Some(5.0)), None);
    }

    #[test]
    fn confidence_weight_zero_when_no_contributors() {
        assert_eq!(confidence_weight(0.0, 50.0), 0.0);
    }

    #[test]
    fn confidence_weight_saturates_at_one() {
        assert_eq!(confidence_weight(50.0, 50.0), 1.0);
    }

    #[test]
    fn confidence_weight_clamped_above_one() {
        assert_eq!(confidence_weight(200.0, 50.0), 1.0);
    }

    #[test]
    fn confidence_weight_proportional_below_threshold() {
        assert!((confidence_weight(25.0, 50.0) - 0.5).abs() < 1e-10);
    }

    #[test]
    fn confidence_weight_zero_threshold_returns_one() {
        assert_eq!(confidence_weight(0.0, 0.0), 1.0);
    }
}
