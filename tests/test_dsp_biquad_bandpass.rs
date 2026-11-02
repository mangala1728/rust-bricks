// tests/dsp_biquad_bandpass.rs
use rust_bricks::dsp_biquad_bandpass::*;

const TOL: f64 = 1e-4;

/// Helper to compare two floating‑point numbers within tolerance.
fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < TOL
}

#[test]
fn golden_coefficients() {
    // Golden vector (sample_rate = 1000 Hz, f₀ = 250 Hz, Q = 1)
    let coeffs = biquad_bandpass_coefficients(1000.0, 250.0, 1.0)
        .expect("valid parameters should yield coefficients");

    let golden = [
        0.3333333333_f64,
        0.0,
        -0.3333333333_f64,
        0.0,
        0.3333333333_f64,
    ];

    for (calc, &ref_golden) in coeffs.iter().zip(golden.iter()) {
        assert!(
            approx_eq(*calc, ref_golden),
            "coeff mismatch: got {calc:.10}, expected {ref_golden:.10}"
        );
    }
}

#[test]
fn step_function_basic() {
    // Use the same golden coefficients
    let coeffs = biquad_bandpass_coefficients(1000.0, 250.0, 1.0)
        .expect("valid parameters should yield coefficients");

    // Initialise state to zero
    let mut state = [0.0_f64; 2];

    // Apply a single impulse (input = 1.0)
    let y = biquad_bandpass_step(1.0, &mut state, &coeffs)
        .expect("valid input should produce output");

    // For the first sample, y = b0 * x (since state[0] == 0)
    let expected_y = coeffs[0]; // b0 / a0 from the golden vector
    assert!(
        approx_eq(y, expected_y),
        "first output mismatch: got {y:.10}, expected {expected_y:.10}"
    );

    // Verify state update consistency with manual calculation
    let manual_state0 = coeffs[1] * 1.0 - coeffs[3] * y + 0.0;
    let manual_state1 = coeffs[2] * 1.0 - coeffs[4] * y;
    assert!(
        approx_eq(state[0], manual_state0),
        "state[0] mismatch: got {state0:.10}, expected {manual_state0:.10}",
        state0 = state[0],
        manual_state0 = manual_state0
    );
    assert!(
        approx_eq(state[1], manual_state1),
        "state[1] mismatch: got {state1:.10}, expected {manual_state1:.10}",
        state1 = state[1],
        manual_state1 = manual_state1
    );
}
