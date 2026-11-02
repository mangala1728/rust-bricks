// ## Map Role: Integration Test for dsp_resample_linear
// Inputs: none (hard‑coded vectors)
// Outputs: test assertions
// Invariants: output must match golden vector within 1e‑4 tolerance

use rust_bricks::dsp_resample_linear::*;

/// Verify up‑sampling from 1 kHz → 2 kHz on a simple ramp.
#[test]
fn test_upsample_linear() {
    // Golden input & expected output
    let input = vec![0.0_f64, 1.0, 2.0, 3.0, 4.0];
    let expected = vec![
        0.0_f64, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.0,
    ];

    // Perform resampling
    let result = resample_linear_1d(&input, 1000.0, 2000.0)
        .expect("valid parameters should produce Some");

    // Length check
    assert_eq!(result.len(), expected.len(), "output length mismatch");

    // Element‑wise tolerance check
    for (got, want) in result.iter().zip(expected.iter()) {
        let diff = (got - want).abs();
        assert!(
            diff < 1e-4,
            "value mismatch: got {}, want {}, diff {}",
            got,
            want,
            diff
        );
    }
}

#[test]
fn test_domain_boundary_guards() {
    assert!(resample_linear_1d(&[], 1000.0, 2000.0).is_none());
    assert!(resample_linear_1d(&[1.0, 2.0], 0.5, 2000.0).is_none());
    assert!(resample_linear_1d(&[1.0, 2.0], 1000.0, 0.5).is_none());
    assert!(resample_linear_1d(&[1.0, 2.0], 1_000_001.0, 2000.0).is_none());
}
