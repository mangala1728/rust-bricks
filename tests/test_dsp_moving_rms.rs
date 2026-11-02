use rust_bricks::dsp_moving_rms::*;

/// Helper for floating‑point equality within a tight tolerance.
fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
    (a - b).abs() <= eps
}

#[test]
fn test_rms_slice_constant_signal() {
    let signal = [3.0_f64, 3.0, 3.0, 3.0];
    let rms = dsp_rms_slice(&signal).expect("valid slice");
    assert!(approx_eq(rms, 3.0, 1e-12));
}

#[test]
fn test_rms_slice_sine_like_vector() {
    // Vector [0, 1, 0, -1] → RMS = sqrt((0²+1²+0²+(-1)²)/4) = sqrt(0.5)
    let signal = [0.0_f64, 1.0, 0.0, -1.0];
    let rms = dsp_rms_slice(&signal).expect("valid slice");
    let golden = 0.7071067811865475_f64; // 1/√2
    assert!(approx_eq(rms, golden, 1e-12));
}

#[test]
fn test_moving_rms_step_basic() {
    // sum_sq = 9 (window of four 3.0 values), replace 3.0 with 4.0
    let sum_sq = 9.0_f64;
    let x_new = 4.0_f64;
    let x_old = 3.0_f64;
    let window_len = 4_usize;

    let (new_sum_sq, rms) =
        dsp_moving_rms_step(x_new, x_old, sum_sq, window_len).expect("valid step");

    assert!(approx_eq(new_sum_sq, 16.0, 1e-12));
    assert!(approx_eq(rms, 2.0, 1e-12));
}

#[test]
fn test_moving_rms_step_invalid_params() {
    // window_len = 0 → None
    assert!(dsp_moving_rms_step(1.0, 0.0, 0.0, 0).is_none());

    // negative sum_sq → None
    assert!(dsp_moving_rms_step(1.0, 0.0, -1.0, 4).is_none());

    // non‑finite sample → None
    assert!(dsp_moving_rms_step(f64::NAN, 0.0, 0.0, 4).is_none());
    assert!(dsp_moving_rms_step(0.0, f64::INFINITY, 0.0, 4).is_none());
}
