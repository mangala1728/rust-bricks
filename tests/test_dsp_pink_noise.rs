// ## Map
// Role: integration test for 3-pole IIR pink noise generator filter
// Inputs: none (uses public API)
// Outputs: test results via assertions
// Invariants: 0 external dependencies, functions <= 60 lines

use rust_bricks::dsp_pink_noise::*;

#[test]
fn test_dsp_pink_filter_step_golden() {
    let mut s = [0.0, 0.0, 0.0];
    let out = dsp_pink_filter_step(1.0, &mut s).unwrap();
    assert!((out - 1.6330537).abs() < 1e-5);
    assert!((s[0] - 0.0990460).abs() < 1e-6);
    assert!((s[1] - 0.2965164).abs() < 1e-6);
    assert!((s[2] - 1.0526913).abs() < 1e-6);
}

#[test]
fn test_dsp_pink_filter_step_guards() {
    let mut s = [0.0, 0.0, 0.0];
    assert!(dsp_pink_filter_step(1.5, &mut s).is_none());
    assert!(dsp_pink_filter_step(-1.5, &mut s).is_none());
    assert!(dsp_pink_filter_step(f64::NAN, &mut s).is_none());

    let mut bad_s = [f64::NAN, 0.0, 0.0];
    assert!(dsp_pink_filter_step(0.5, &mut bad_s).is_none());
}

#[test]
fn test_dsp_pink_filter_slice() {
    let mut s = [0.0, 0.0, 0.0];
    let white = [1.0, 0.0, -1.0];
    let mut out = [0.0; 3];
    let written = dsp_pink_filter_slice(&white, &mut s, &mut out);
    assert_eq!(written, 3);
    assert!((out[0] - 1.6330537).abs() < 1e-5);
}
