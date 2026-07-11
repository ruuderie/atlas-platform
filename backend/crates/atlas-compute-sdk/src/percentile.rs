//! `percentile` — Percentile band classification.
//!
//! Extracted from `ScorecardService` / `ScorecardAnalyticsService` (Phases 2/3).

/// Map a percentile rank (0.0–100.0) to a categorical band string.
///
/// Thresholds (matching Phase 2 DB constraint and Phase 3 MV):
///
/// | Band             | Condition        |
/// |------------------|------------------|
/// | `"top_10"`       | rank >= 90.0     |
/// | `"top_quartile"` | rank >= 75.0     |
/// | `"median"`       | rank >= 50.0     |
/// | `"bottom_quartile"` | rank < 50.0  |
///
/// Returns `None` if `rank` is outside [0.0, 100.0].
pub fn band_from_rank(rank: f64) -> Option<&'static str> {
    if !(0.0..=100.0).contains(&rank) {
        return None;
    }
    if rank >= 90.0 {
        Some("top_10")
    } else if rank >= 75.0 {
        Some("top_quartile")
    } else if rank >= 50.0 {
        Some("median")
    } else {
        Some("bottom_quartile")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn band_top_10_at_90() {
        assert_eq!(band_from_rank(90.0), Some("top_10"));
        assert_eq!(band_from_rank(100.0), Some("top_10"));
        assert_eq!(band_from_rank(95.5), Some("top_10"));
    }

    #[test]
    fn band_top_quartile_at_75() {
        assert_eq!(band_from_rank(75.0), Some("top_quartile"));
        assert_eq!(band_from_rank(89.99), Some("top_quartile"));
    }

    #[test]
    fn band_median_at_50() {
        assert_eq!(band_from_rank(50.0), Some("median"));
        assert_eq!(band_from_rank(74.99), Some("median"));
    }

    #[test]
    fn band_bottom_quartile_below_50() {
        assert_eq!(band_from_rank(49.99), Some("bottom_quartile"));
        assert_eq!(band_from_rank(0.0), Some("bottom_quartile"));
    }

    #[test]
    fn band_boundaries_all_correct() {
        let cases = [
            (49.99, "bottom_quartile"),
            (50.0, "median"),
            (74.99, "median"),
            (75.0, "top_quartile"),
            (89.99, "top_quartile"),
            (90.0, "top_10"),
        ];
        for (rank, expected) in cases {
            assert_eq!(
                band_from_rank(rank),
                Some(expected),
                "band_from_rank({rank}) should be {expected}"
            );
        }
    }

    #[test]
    fn band_none_for_out_of_range() {
        assert_eq!(band_from_rank(-0.01), None);
        assert_eq!(band_from_rank(100.01), None);
        assert_eq!(band_from_rank(f64::NAN), None);
    }
}
