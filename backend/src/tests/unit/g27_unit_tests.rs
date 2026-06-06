//! G-27 Data Science — Phase 5 unit tests.
//!
//! Pure math tests: no DB, no async, no I/O.
//! All functions under test live in `atlas_compute_sdk` and are also called
//! (via `atlas-compute-sdk` workspace dependency) by the service layer.
//!
//! These tests double as the specification for what BYOC Lambda wrappers
//! must implement to be compatible with Atlas Platform's expected outputs.

#[cfg(test)]
mod aggregate_tests {
    use atlas_compute_sdk::aggregate::{bayesian_shrinkage, confidence_weight, weighted_mean};

    #[test]
    fn weighted_mean_empty_returns_none() {
        assert_eq!(weighted_mean(&[]), None);
    }

    #[test]
    fn weighted_mean_equal_weights_is_arithmetic_mean() {
        let r = weighted_mean(&[(4.0, 1.0), (6.0, 1.0)]).unwrap();
        assert!((r - 5.0).abs() < 1e-10, "got {r}");
    }

    #[test]
    fn weighted_mean_higher_weight_pulls_toward_heavy_score() {
        // (9×3 + 3×1) / (3+1) = 30/4 = 7.5
        let r = weighted_mean(&[(9.0, 3.0), (3.0, 1.0)]).unwrap();
        assert!((r - 7.5).abs() < 1e-10, "got {r}");
    }

    #[test]
    fn bayesian_shrinkage_blends_toward_prior() {
        // prior_weight=5, ref=5, raw_mean=9, weight_total=2
        // shrunk = (25 + 18) / 7 ≈ 6.14
        let r = bayesian_shrinkage(Some(9.0), 2.0, Some(5.0), Some(5.0)).unwrap();
        assert!(r > 5.0 && r < 9.0, "expected 5 < shrunk < 9, got {r}");
    }

    #[test]
    fn bayesian_shrinkage_identity_when_no_prior_weight() {
        assert_eq!(bayesian_shrinkage(Some(7.0), 3.0, None, Some(5.0)), Some(7.0));
    }

    #[test]
    fn bayesian_shrinkage_identity_when_no_global_ref() {
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
}

#[cfg(test)]
mod calibration_tests {
    use atlas_compute_sdk::calibration::{
        apply_calibration, compute_bias_offset, compute_scale_factor,
    };

    #[test]
    fn calibration_identity_when_bias_zero_scale_one() {
        assert!((apply_calibration(7.0, 0.0, 1.0, 0.0, 10.0) - 7.0).abs() < 1e-10);
    }

    #[test]
    fn calibration_shifts_down_when_positive_bias() {
        assert!((apply_calibration(7.0, 2.0, 1.0, 0.0, 10.0) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn calibration_shifts_up_when_negative_bias() {
        assert!((apply_calibration(7.0, -1.0, 1.0, 0.0, 10.0) - 8.0).abs() < 1e-10);
    }

    #[test]
    fn calibration_scales_score() {
        assert!((apply_calibration(8.0, 0.0, 0.5, 0.0, 10.0) - 4.0).abs() < 1e-10);
    }

    #[test]
    fn calibration_clamps_to_scale_min() {
        assert!((apply_calibration(1.0, 5.0, 1.0, 0.0, 10.0)).abs() < 1e-10);
    }

    #[test]
    fn calibration_clamps_to_scale_max() {
        assert!((apply_calibration(9.0, -5.0, 1.0, 0.0, 10.0) - 10.0).abs() < 1e-10);
    }

    #[test]
    fn bias_positive_when_contributor_scores_high() {
        assert!((compute_bias_offset(8.0, 5.0) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn bias_zero_when_aligned_with_ensemble() {
        assert!(compute_bias_offset(5.0, 5.0).abs() < 1e-10);
    }

    #[test]
    fn scale_factor_one_when_stds_equal() {
        assert!((compute_scale_factor(2.0, 2.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn scale_factor_one_when_ensemble_std_near_zero() {
        assert!((compute_scale_factor(1.0, 0.001) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn scale_factor_clamped_below_01() {
        assert!((compute_scale_factor(0.05, 2.0) - 0.1).abs() < 1e-10);
    }

    #[test]
    fn scale_factor_clamped_above_30() {
        assert!((compute_scale_factor(10.0, 0.1) - 3.0).abs() < 1e-10);
    }
}

#[cfg(test)]
mod anomaly_tests {
    use atlas_compute_sdk::anomaly::{anomaly_direction, is_anomaly, z_score};

    #[test]
    fn z_score_none_for_fewer_than_three_periods() {
        assert_eq!(z_score(&[7.0, 8.0], 9.0), None);
        assert_eq!(z_score(&[], 9.0), None);
    }

    #[test]
    fn z_score_zero_for_flat_series() {
        assert_eq!(z_score(&[5.0; 6], 5.0), Some(0.0));
    }

    #[test]
    fn z_score_positive_for_spike() {
        let window = vec![4.0, 5.0, 5.0, 5.0, 5.0, 6.0];
        let z = z_score(&window, 10.0).unwrap();
        assert!(z > 2.0, "got {z}");
    }

    #[test]
    fn z_score_negative_for_drop() {
        let window = vec![4.0, 5.0, 5.0, 5.0, 5.0, 6.0];
        let z = z_score(&window, 0.0).unwrap();
        assert!(z < -2.0, "got {z}");
    }

    #[test]
    fn is_anomaly_true_above_threshold() {
        assert!(is_anomaly(2.1));
        assert!(is_anomaly(-2.1));
    }

    #[test]
    fn is_anomaly_false_at_exact_threshold() {
        assert!(!is_anomaly(2.0));   // strict >
        assert!(!is_anomaly(-2.0));
    }

    #[test]
    fn anomaly_direction_spike_for_positive_z() {
        assert_eq!(anomaly_direction(2.1), Some("spike"));
    }

    #[test]
    fn anomaly_direction_drop_for_negative_z() {
        assert_eq!(anomaly_direction(-2.1), Some("drop"));
    }

    #[test]
    fn anomaly_direction_none_within_bounds() {
        assert_eq!(anomaly_direction(2.0),  None);
        assert_eq!(anomaly_direction(-2.0), None);
        assert_eq!(anomaly_direction(0.0),  None);
    }
}

#[cfg(test)]
mod percentile_tests {
    use atlas_compute_sdk::percentile::band_from_rank;

    #[test]
    fn band_top_10_at_90() {
        assert_eq!(band_from_rank(90.0), Some("top_10"));
        assert_eq!(band_from_rank(100.0), Some("top_10"));
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
        assert_eq!(band_from_rank(0.0),   Some("bottom_quartile"));
    }

    #[test]
    fn band_boundaries_all_correct() {
        let cases = [
            (49.99, "bottom_quartile"),
            (50.0,  "median"),
            (74.99, "median"),
            (75.0,  "top_quartile"),
            (89.99, "top_quartile"),
            (90.0,  "top_10"),
        ];
        for (rank, expected) in cases {
            assert_eq!(band_from_rank(rank), Some(expected), "rank={rank}");
        }
    }

    #[test]
    fn band_none_for_out_of_range() {
        assert_eq!(band_from_rank(-0.01), None);
        assert_eq!(band_from_rank(100.01), None);
    }
}

#[cfg(test)]
mod similarity_tests {
    use atlas_compute_sdk::similarity::masked_cosine;

    fn all_true(n: usize) -> Vec<bool> { vec![true; n] }

    #[test]
    fn identical_vectors_return_one() {
        let v = vec![1.0, 2.0, 3.0, 4.0];
        let m = all_true(4);
        let s = masked_cosine(&v, &v, &m, &m).unwrap();
        assert!((s - 1.0).abs() < 1e-10, "got {s}");
    }

    #[test]
    fn orthogonal_vectors_return_zero() {
        let a = vec![1.0, 0.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0, 0.0];
        let m = all_true(4);
        let s = masked_cosine(&a, &b, &m, &m).unwrap();
        assert!(s.abs() < 1e-10, "got {s}");
    }

    #[test]
    fn returns_none_below_30pct_overlap() {
        let n = 10;
        let v = vec![1.0; n];
        let mut ma = vec![false; n];
        let mut mb = vec![false; n];
        ma[0] = true; mb[0] = true; // only 2/10 = 20% < 30%
        ma[1] = true; mb[1] = true;
        assert_eq!(masked_cosine(&v, &v, &ma, &mb), None);
    }

    #[test]
    fn returns_some_at_30pct_overlap() {
        let n = 10;
        let v = vec![1.0; n];
        let mut ma = vec![false; n];
        let mut mb = vec![false; n];
        ma[0] = true; mb[0] = true;
        ma[1] = true; mb[1] = true;
        ma[2] = true; mb[2] = true; // 3/10 = 30% = ceil(10*0.3)
        assert!(masked_cosine(&v, &v, &ma, &mb).is_some());
    }

    #[test]
    fn ignores_unmasked_dimensions() {
        let a = vec![1.0, 1.0, 1.0, 100.0];
        let b = vec![1.0, 1.0, 1.0,   0.0];
        let ma = vec![true, true, true, true ];
        let mb = vec![true, true, true, false]; // dim 3 masked out in b
        let s = masked_cosine(&a, &b, &ma, &mb).unwrap();
        assert!((s - 1.0).abs() < 1e-10, "unmasked dim should be ignored, got {s}");
    }

    #[test]
    fn zero_magnitude_returns_none() {
        let z = vec![0.0; 4];
        let m = all_true(4);
        assert_eq!(masked_cosine(&z, &z, &m, &m), None);
    }
}

#[cfg(test)]
mod compute_roundtrip_tests {
    use atlas_compute_sdk::{compute, types::{ComputeRequest, EntryInput}};

    fn make_request(scores: &[f64]) -> ComputeRequest {
        ComputeRequest {
            entries: scores.iter().map(|&s| EntryInput {
                score:             Some(s),
                credibility_weight: Some(1.0),
                bias_offset:        None,
                scale_factor:       None,
            }).collect(),
            scale_min:             0.0,
            scale_max:             10.0,
            bayesian_prior_weight: None,
            global_reference_value: None,
            saturation_threshold:  Some(50.0),
        }
    }

    #[test]
    fn compute_request_json_roundtrips() {
        let req = make_request(&[7.0, 8.0, 9.0]);
        let json = serde_json::to_string(&req).expect("serialise");
        let back: ComputeRequest = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(back.entries.len(), 3);
    }

    #[test]
    fn compute_response_json_roundtrips() {
        let resp = compute(make_request(&[6.0, 8.0]));
        let json = serde_json::to_string(&resp).expect("serialise");
        let back: atlas_compute_sdk::types::ComputeResponse =
            serde_json::from_str(&json).expect("deserialise");
        assert_eq!(back.contributor_count, 2);
    }

    #[test]
    fn compute_weighted_mean_matches_manual_calculation() {
        // Two entries, equal weights, no calibration, no shrinkage
        // expected mean = (6 + 8) / 2 = 7.0
        let resp = compute(make_request(&[6.0, 8.0]));
        let wm = resp.weighted_mean.expect("must produce a weighted mean");
        assert!((wm - 7.0).abs() < 1e-10, "expected 7.0, got {wm}");
    }

    #[test]
    fn compute_with_calibration_shifts_mean() {
        // bias = +2.0 on all entries: (6-2)=4, (8-2)=6 → mean = 5.0
        let req = ComputeRequest {
            entries: vec![
                EntryInput { score: Some(6.0), credibility_weight: Some(1.0), bias_offset: Some(2.0), scale_factor: Some(1.0) },
                EntryInput { score: Some(8.0), credibility_weight: Some(1.0), bias_offset: Some(2.0), scale_factor: Some(1.0) },
            ],
            scale_min: 0.0,
            scale_max: 10.0,
            bayesian_prior_weight:  None,
            global_reference_value: None,
            saturation_threshold:   Some(50.0),
        };
        let resp = compute(req);
        let wm = resp.weighted_mean.unwrap();
        assert!((wm - 5.0).abs() < 1e-10, "expected 5.0 after bias correction, got {wm}");
    }

    #[test]
    fn compute_with_bayesian_shrinkage_pulls_toward_prior() {
        // prior_weight=10, global_ref=5.0, 2 entries of 9.0
        // raw_mean=9.0, weight_total=2.0
        // shrunk = (10×5 + 2×9) / (10+2) = (50+18)/12 = 68/12 ≈ 5.67
        let req = ComputeRequest {
            entries: vec![
                EntryInput { score: Some(9.0), credibility_weight: Some(1.0), bias_offset: None, scale_factor: None },
                EntryInput { score: Some(9.0), credibility_weight: Some(1.0), bias_offset: None, scale_factor: None },
            ],
            scale_min: 0.0,
            scale_max: 10.0,
            bayesian_prior_weight:  Some(10.0),
            global_reference_value: Some(5.0),
            saturation_threshold:   Some(50.0),
        };
        let resp = compute(req);
        let wm = resp.weighted_mean.unwrap();
        assert!(wm < 9.0, "shrinkage must pull below raw mean 9.0, got {wm}");
        assert!(wm > 5.0, "shrinkage must stay above prior 5.0, got {wm}");
        assert!((wm - (68.0 / 12.0)).abs() < 1e-10, "expected ≈5.667, got {wm}");
    }
}
