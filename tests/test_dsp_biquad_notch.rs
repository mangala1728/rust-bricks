// ## Map
// Role: Integration test for IIR Biquad Notch Filter
// Inputs: none (uses public API)
// Outputs: test assertions
// Invariants: golden vectors verified against Audio EQ Cookbook standard

use rust_bricks::dsp_biquad_notch::*;

#[test]
fn coeffs_match_golden_vector() {
    // Golden parameters
    let sr = 1000.0_f64;
    let f0 = 250.0_f64;
    let q = 0.7071_f64;

    let coeffs = biquad_notch_coefficients(sr, f0, q).expect("valid parameters");

    // Expected values (rounded to 6 decimal places for readability)
    let golden = [
        0.585786_f64, // b0/a0
        0.0_f64,      // b1/a0
        0.585786_f64, // b2/a0
        0.0_f64,      // a1/a0
        0.171573_f64, // a2/a0
    ];

    for (calc, &exp) in coeffs.iter().zip(golden.iter()) {
        assert!(
            (calc - exp).abs() < 1e-4,
            "coeff mismatch: got {calc:.6}, expected {exp:.6}"
        );
    }
}

#[test]
fn step_produces_expected_output_and_state() {
    // Use the same golden coefficients as above
    let coeffs = biquad_notch_coefficients(1000.0, 250.0, 0.7071).unwrap();

    // Initialise zero state
    let mut state = [0.0_f64; 2];
    let x = 1.0_f64;

    // First step
    let y = biquad_notch_step(x, &mut state, &coeffs).expect("valid step");

    // Expected output: y = b0/a0 * x (since state[0] == 0)
    let expected_y = coeffs[0] * x;
    assert!(
        (y - expected_y).abs() < 1e-4,
        "output mismatch: got {y:.6}, expected {expected_y:.6}"
    );

    // Expected new state[0] = b1·x - a1·y + old_state[1] = 0
    assert!(
        state[0].abs() < 1e-12,
        "state[0] mismatch: got {:#?}", state[0]
    );

    // Expected new state[1] = b2·x - a2·y
    let expected_s1 = coeffs[2] * x - coeffs[4] * y;
    assert!(
        (state[1] - expected_s1).abs() < 1e-4,
        "state[1] mismatch: got {:#?}, expected {expected_s1:.6}",
        state[1]
    );
}
