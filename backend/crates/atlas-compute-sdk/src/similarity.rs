//! `similarity` — Masked cosine similarity for Phase 2 `find_similar`.
//!
//! Extracted from `ScorecardService::find_similar` (Gap 3 fix, Phase 2).
//! Uses a parallel boolean mask to skip dimensions where either entity has no data.

/// Compute masked cosine similarity between two dimension vectors.
///
/// Only considers dimensions where **both** `mask_a[i]` and `mask_b[i]` are `true`.
/// If the overlap is less than 30% of the total dimension count, returns `None`
/// (insufficient shared data to produce a meaningful similarity score).
///
/// Returns `None` if either vector has zero magnitude in the shared dimensions.
///
/// # Arguments
/// - `vec_a`, `vec_b`   — normalised dimension value arrays (must be equal length)
/// - `mask_a`, `mask_b` — parallel boolean data-presence masks
///
/// # Panics
/// Does not panic; returns `None` on all degenerate inputs.
pub fn masked_cosine(
    vec_a: &[f64],
    vec_b: &[f64],
    mask_a: &[bool],
    mask_b: &[bool],
) -> Option<f64> {
    let n = vec_a.len();
    if n == 0 || vec_b.len() != n || mask_a.len() != n || mask_b.len() != n {
        return None;
    }

    let min_overlap = (n as f64 * 0.30).ceil() as usize;

    let mut dot = 0.0f64;
    let mut mag_a = 0.0f64;
    let mut mag_b = 0.0f64;
    let mut overlap = 0usize;

    for i in 0..n {
        if mask_a[i] && mask_b[i] {
            dot += vec_a[i] * vec_b[i];
            mag_a += vec_a[i] * vec_a[i];
            mag_b += vec_b[i] * vec_b[i];
            overlap += 1;
        }
    }

    if overlap < min_overlap {
        return None; // too little shared data
    }

    let denom = mag_a.sqrt() * mag_b.sqrt();
    if denom < 1e-12 {
        return None; // zero-magnitude vector
    }

    Some((dot / denom).clamp(-1.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_true(n: usize) -> Vec<bool> {
        vec![true; n]
    }

    #[test]
    fn identical_vectors_return_one() {
        let v = vec![1.0, 2.0, 3.0, 4.0];
        let m = all_true(4);
        let sim = masked_cosine(&v, &v, &m, &m).unwrap();
        assert!(
            (sim - 1.0).abs() < 1e-10,
            "identical vectors → 1.0, got {sim}"
        );
    }

    #[test]
    fn orthogonal_vectors_return_zero() {
        let a = vec![1.0, 0.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0, 0.0];
        let m = all_true(4);
        let sim = masked_cosine(&a, &b, &m, &m).unwrap();
        assert!(sim.abs() < 1e-10, "orthogonal vectors → 0.0, got {sim}");
    }

    #[test]
    fn returns_none_below_30pct_overlap() {
        // 10 dims, only 2 share a mask → 20% < 30% → None
        let n = 10;
        let v = vec![1.0; n];
        let mut m_a = vec![false; n];
        let mut m_b = vec![false; n];
        m_a[0] = true;
        m_b[0] = true;
        m_a[1] = true;
        m_b[1] = true;
        // overlap = 2 / 10 = 20% < 30%
        assert_eq!(masked_cosine(&v, &v, &m_a, &m_b), None);
    }

    #[test]
    fn returns_some_at_or_above_30pct_overlap() {
        // 10 dims, 3 shared → 30% → should succeed (ceil(10*0.3)=3)
        let n = 10;
        let v = vec![1.0; n];
        let mut m_a = vec![false; n];
        let mut m_b = vec![false; n];
        m_a[0] = true;
        m_b[0] = true;
        m_a[1] = true;
        m_b[1] = true;
        m_a[2] = true;
        m_b[2] = true;
        assert!(masked_cosine(&v, &v, &m_a, &m_b).is_some());
    }

    #[test]
    fn ignores_unmasked_dimensions() {
        // Dim 0–2: both masked with matching values
        // Dim 3: a=100.0 but mask_b[3]=false → should not affect result
        let a = vec![1.0, 1.0, 1.0, 100.0];
        let b = vec![1.0, 1.0, 1.0, 0.0];
        let m_a = vec![true, true, true, true];
        let m_b = vec![true, true, true, false];
        let sim = masked_cosine(&a, &b, &m_a, &m_b).unwrap();
        // Only dims 0–2 contribute: all matching → should be 1.0
        assert!(
            (sim - 1.0).abs() < 1e-10,
            "unmasked dim should be ignored, got {sim}"
        );
    }

    #[test]
    fn zero_magnitude_returns_none() {
        let z = vec![0.0; 4];
        let m = all_true(4);
        assert_eq!(masked_cosine(&z, &z, &m, &m), None);
    }

    #[test]
    fn empty_input_returns_none() {
        assert_eq!(masked_cosine(&[], &[], &[], &[]), None);
    }

    #[test]
    fn mismatched_lengths_returns_none() {
        let a = vec![1.0, 2.0];
        let b = vec![1.0];
        let m = vec![true, true];
        assert_eq!(masked_cosine(&a, &b, &m, &m), None);
    }
}
