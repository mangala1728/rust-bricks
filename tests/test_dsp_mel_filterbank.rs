// ## Map
// Role:   Integration test for `dsp_mel_filterbank`
// Inputs: None (uses golden vectors)
// Outputs: Test results (assertions)
// Invariants: Deterministic golden values, no `unwrap`/`expect` without safety proof.

use rust_bricks::dsp_mel_filterbank::*;

#[test]
fn test_hz_to_mel_and_back() {
    // Golden vector: 1000 Hz ↔ ≈ 999.985 Mel
    let hz = 1000.0_f64;
    let mel_opt = hz_to_mel(hz);
    assert!(mel_opt.is_some(), "hz_to_mel should succeed for 1000 Hz");
    let mel = mel_opt.unwrap(); // safe: we just asserted Some
    // Expected mel ≈ 999.985 (tolerance 0.02)
    assert!((mel - 999.985).abs() < 2e-2, "Mel conversion error: {}", mel);

    let hz_back_opt = mel_to_hz(mel);
    assert!(hz_back_opt.is_some(), "mel_to_hz should succeed for the computed mel");
    let hz_back = hz_back_opt.unwrap(); // safe after check
    // Tolerance 0.01 Hz
    assert!((hz_back - hz).abs() < 1e-2, "Round‑trip error: {}", hz_back);
}

#[test]
fn test_mel_filterbank_centers_hz() {
    // Golden vector: 2 filters, range 0 Hz – 4000 Hz
    let centers_opt = mel_filterbank_centers_hz(2, 0.0, 4000.0);
    assert!(centers_opt.is_some(), "centers calculation should succeed");
    let centers = centers_opt.unwrap(); // safe after check

    // Expected points (±0.01 tolerance)
    let expected = [0.0_f64, 620.5798_f64, 1791.3300_f64, 4000.0_f64];
    for (i, (&got, &exp)) in centers.iter().zip(expected.iter()).enumerate() {
        assert!(
            (got - exp).abs() < 1e-2,
            "center[{}] mismatch: got {:.6}, expected {:.6}",
            i,
            got,
            exp
        );
    }
}

#[test]
fn test_domain_boundary_guards() {
    assert!(hz_to_mel(-1.0).is_none());
    assert!(hz_to_mel(100_001.0).is_none());
    assert!(mel_to_hz(-1.0).is_none());
    assert!(mel_to_hz(10_001.0).is_none());

    assert!(mel_filterbank_centers_hz(1, 0.0, 4000.0).is_none());
    assert!(mel_filterbank_centers_hz(129, 0.0, 4000.0).is_none());
    assert!(mel_filterbank_centers_hz(10, 4000.0, 1000.0).is_none());
}
