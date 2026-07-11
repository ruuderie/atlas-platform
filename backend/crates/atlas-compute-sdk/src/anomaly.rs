//! `anomaly` — Rolling z-score and anomaly detection for time-series data.
//!
//! Extracted from `ScorecardService::refresh_time_series_for_dimension` (Phase 2).
//! Computes a z-score against a trailing window of historical period values.

/// Compute the z-score of a current value against a trailing window of prior values.
///
/// Returns `None` if fewer than 3 prior values are available (insufficient history).
/// Returns `0.0` if the window standard deviation is near-zero (flat series).
///
/// # Formula
/// ```text
/// μ = mean(window)
/// σ = std_dev(window)     [population std — no Bessel correction, matches Phase 2]
/// z = (current - μ) / σ
/// ```
///
/// # Arguments
/// - `window`  — trailing period values (up to 6 most-recent, exclusive of current)
/// - `current` — the value for the current period
pub fn z_score(window: &[f64], current: f64) -> Option<f64> {
    if window.len() < 3 {
        return None; // insufficient trailing history
    }

    let n = window.len() as f64;
    let mean = window.iter().sum::<f64>() / n;
    let variance = window.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    let std = variance.sqrt();

    if std < 1e-9 {
        return Some(0.0); // flat series — no anomaly signal
    }

    Some((current - mean) / std)
}

/// Determine whether a z-score represents an anomaly.
///
/// Threshold: `|z| > 2.0` (strict greater-than, matching Phase 2 spec).
pub fn is_anomaly(z: f64) -> bool {
    z.abs() > 2.0
}

/// Return the anomaly direction string for a z-score.
///
/// - `z > 2.0`  → `"spike"` (unusually high for this period)
/// - `z < -2.0` → `"drop"`  (unusually low for this period)
/// - otherwise  → `None`    (not an anomaly)
pub fn anomaly_direction(z: f64) -> Option<&'static str> {
    if z > 2.0 {
        Some("spike")
    } else if z < -2.0 {
        Some("drop")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn z_score_none_for_fewer_than_three_periods() {
        assert_eq!(z_score(&[7.0, 8.0], 9.0), None);
        assert_eq!(z_score(&[7.0], 9.0), None);
        assert_eq!(z_score(&[], 9.0), None);
    }

    #[test]
    fn z_score_zero_when_all_values_equal_and_current_matches() {
        let window = vec![5.0; 6];
        assert_eq!(z_score(&window, 5.0), Some(0.0));
    }

    #[test]
    fn z_score_zero_when_flat_series_regardless_of_current() {
        // Flat window → std≈0 → z=0.0 (guarded division)
        let window = vec![5.0; 6];
        assert_eq!(z_score(&window, 9.5), Some(0.0));
    }

    #[test]
    fn z_score_positive_for_spike() {
        // Window: 5,5,5,5,5,5 — but we need variance, so use varied window
        let window = vec![4.0, 5.0, 5.0, 5.0, 5.0, 6.0]; // std ≈ 0.58
        let z = z_score(&window, 10.0).unwrap();
        assert!(z > 2.0, "spike should produce z > 2.0, got {z}");
    }

    #[test]
    fn z_score_negative_for_drop() {
        let window = vec![4.0, 5.0, 5.0, 5.0, 5.0, 6.0]; // std ≈ 0.58
        let z = z_score(&window, 0.0).unwrap();
        assert!(z < -2.0, "drop should produce z < -2.0, got {z}");
    }

    #[test]
    fn is_anomaly_true_above_threshold() {
        assert!(is_anomaly(2.1));
        assert!(is_anomaly(-2.1));
        assert!(is_anomaly(5.0));
    }

    #[test]
    fn is_anomaly_false_at_and_below_threshold() {
        assert!(!is_anomaly(2.0)); // strict >
        assert!(!is_anomaly(-2.0));
        assert!(!is_anomaly(1.5));
        assert!(!is_anomaly(0.0));
    }

    #[test]
    fn anomaly_direction_spike_when_positive() {
        assert_eq!(anomaly_direction(2.1), Some("spike"));
        assert_eq!(anomaly_direction(5.0), Some("spike"));
    }

    #[test]
    fn anomaly_direction_drop_when_negative() {
        assert_eq!(anomaly_direction(-2.1), Some("drop"));
        assert_eq!(anomaly_direction(-5.0), Some("drop"));
    }

    #[test]
    fn anomaly_direction_none_within_bounds() {
        assert_eq!(anomaly_direction(2.0), None); // boundary: not anomalous
        assert_eq!(anomaly_direction(-2.0), None);
        assert_eq!(anomaly_direction(0.0), None);
    }
}
