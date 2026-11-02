use rust_bricks::dsp_dc_blocker::{dsp_dc_blocker_slice, dsp_dc_blocker_step};

#[test]
fn test_step_basic_gold_vector() {
    let r = 0.9_f64;
    // n = 0
    let (y0, x0, y0_state) = dsp_dc_blocker_step(10.0, 0.0, 0.0, r).expect("valid step");
    assert!((y0 - 10.0).abs() < 1e-12);
    // n = 1
    let (y1, x1, y1_state) = dsp_dc_blocker_step(10.0, x0, y0_state, r).expect("valid step");
    assert!((y1 - 9.0).abs() < 1e-12);
    // n = 2
    let (y2, _, _) = dsp_dc_blocker_step(10.0, x1, y1_state, r).expect("valid step");
    assert!((y2 - 8.1).abs() < 1e-12);
}

#[test]
fn test_step_invalid_parameters() {
    // r_coeff out of range
    assert!(dsp_dc_blocker_step(1.0, 0.0, 0.0, -0.1).is_none());
    assert!(dsp_dc_blocker_step(1.0, 0.0, 0.0, 1.0).is_none());
    // non‑finite input
    assert!(dsp_dc_blocker_step(f64::NAN, 0.0, 0.0, 0.5).is_none());
    assert!(dsp_dc_blocker_step(1.0, f64::INFINITY, 0.0, 0.5).is_none());
}

#[test]
fn test_slice_basic_gold_vector() {
    let r = 0.9_f64;
    let mut data = [10.0_f64, 10.0, 10.0];
    let ok = dsp_dc_blocker_slice(&mut data, r);
    assert!(ok, "slice processing should succeed");

    let expected = [10.0_f64, 9.0, 8.1];
    for (out, &exp) in data.iter().zip(expected.iter()) {
        assert!((out - exp).abs() < 1e-12);
    }
}

#[test]
fn test_slice_invalid_cases() {
    // Empty slice
    let mut empty: [f64; 0] = [];
    assert!(!dsp_dc_blocker_slice(&mut empty, 0.5));

    // Bad coefficient
    let mut data = [1.0_f64, 2.0];
    assert!(!dsp_dc_blocker_slice(&mut data, -0.01));
    assert!(!dsp_dc_blocker_slice(&mut data, 1.0));

    // Non‑finite sample
    let mut bad = [0.0_f64, f64::NAN];
    assert!(!dsp_dc_blocker_slice(&mut bad, 0.5));
}
