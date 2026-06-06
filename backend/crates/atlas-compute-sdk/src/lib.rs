//! atlas-compute-sdk — G-27 Pure-Math Scoring Algorithms
//!
//! This crate contains all deterministic, I/O-free algorithms extracted from
//! `atlas_backend::services::scorecard_service`. It compiles to:
//!
//!   - **native rlib**: used in-process by `atlas_backend` (zero overhead)
//!   - **wasm32-wasip1 cdylib**: loaded by enterprise BYOC clients via Wasmtime
//!     inside their own AWS Lambda / GCF / Azure Function deployment.
//!
//! ## Modules
//!
//! | Module        | Algorithms                                                  |
//! |---------------|-------------------------------------------------------------|
//! | `types`       | `ComputeRequest`, `ComputeResponse`, `DimensionInput`       |
//! | `aggregate`   | `weighted_mean`, `bayesian_shrinkage`, `confidence_weight`  |
//! | `calibration` | `apply_calibration`, `compute_bias_scale`                   |
//! | `anomaly`     | `z_score`, `is_anomaly`, `anomaly_direction`                |
//! | `percentile`  | `band_from_rank`                                            |
//! | `similarity`  | `masked_cosine`                                             |
//!
//! ## BYOC Entry Point
//!
//! Enterprise clients compile this crate to `.wasm` and wrap it in a thin
//! Lambda handler. The handler deserialises a `ComputeRequest` from stdin,
//! calls `compute()`, and writes `ComputeResponse` to stdout. Atlas Platform
//! sends the request and reads the response — compute stays inside the
//! customer's VPC.
//!
//! ```rust,no_run
//! use atlas_compute_sdk::{compute, types::ComputeRequest};
//!
//! fn main() {
//!     let input   = std::io::stdin();
//!     let request: ComputeRequest = serde_json::from_reader(input).unwrap();
//!     let response = compute(request);
//!     println!("{}", serde_json::to_string(&response).unwrap());
//! }
//! ```

pub mod aggregate;
pub mod anomaly;
pub mod calibration;
pub mod percentile;
pub mod similarity;
pub mod types;

use types::{ComputeRequest, ComputeResponse};

/// Primary entry point for BYOC Lambda wrappers.
///
/// Accepts a `ComputeRequest` (JSON-deserialisable) and returns a
/// `ComputeResponse` (JSON-serialisable). Runs entirely in-process —
/// no I/O, no allocations beyond the input/output structs.
pub fn compute(req: ComputeRequest) -> ComputeResponse {
    use aggregate::{bayesian_shrinkage, confidence_weight, weighted_mean};
    use calibration::apply_calibration;

    let scores_with_weights: Vec<(f64, f64)> = req
        .entries
        .iter()
        .filter_map(|e| {
            let raw = e.score?;
            // Apply calibration (bias + scale) if provided
            let calibrated = apply_calibration(
                raw,
                e.bias_offset.unwrap_or(0.0),
                e.scale_factor.unwrap_or(1.0),
                req.scale_min,
                req.scale_max,
            );
            Some((calibrated, e.credibility_weight.unwrap_or(1.0)))
        })
        .collect();

    let raw_wm = weighted_mean(&scores_with_weights);

    let shrunk_wm = bayesian_shrinkage(
        raw_wm,
        scores_with_weights.iter().map(|(_, w)| *w).sum(),
        req.bayesian_prior_weight,
        req.global_reference_value,
    );

    let contributor_count = scores_with_weights.len() as f64;
    let cw = confidence_weight(contributor_count, req.saturation_threshold.unwrap_or(50.0));

    ComputeResponse {
        weighted_mean: shrunk_wm,
        confidence_weight: cw,
        contributor_count: scores_with_weights.len() as u32,
    }
}
