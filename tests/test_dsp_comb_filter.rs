// ## Map
// Role:   Integration test for dsp_comb_filter module
// Inputs: predefined impulse sequence, delay line, gains
// Outputs: verified output vector
// Invariants: deterministic processing, no external randomness

use rust_bricks::dsp_comb_filter::{dsp_feedback_comb_step, dsp_feedforward_comb_step};

#[test]
fn feedback_comb_pulse_response() {
    // delay length D = 2, feedback gain g = 0.5
    let mut buffer = [0.0_f64; 2];
    let mut write_pos = 0usize;
    let gain = 0.5_f64;

    // input impulse sequence
    let input = [1.0_f64, 0.0, 0.0, 0.0, 0.0];
    let mut output = Vec::with_capacity(input.len());

    for &sample in &input {
        // unwrap is safe because we know the contract is satisfied
        let y = dsp_feedback_comb_step(sample, &mut buffer, &mut write_pos, gain)
            .expect("valid parameters should never return None");
        output.push(y);
    }

    // golden vector from the specification
    let golden = [1.0_f64, 0.0, 0.5, 0.0, 0.25];
    assert_eq!(output.as_slice(), &golden);
}

#[test]
fn feedforward_comb_basic() {
    // delay length D = 3, blend = 0.3
    let mut buffer = [0.0_f64; 3];
    let mut write_pos = 0usize;
    let blend = 0.3_f64;

    // simple ramp input
    let input = [1.0_f64, 2.0, 3.0, 4.0];
    let mut output = Vec::with_capacity(input.len());

    for &sample in &input {
        let y = dsp_feedforward_comb_step(sample, &mut buffer, &mut write_pos, blend)
            .expect("valid parameters should never return None");
        output.push(y);
    }

    // manual calculation:
    // n=0: 1.0 + 0.3*0 = 1.0   (buffer[0]=1.0)
    // n=1: 2.0 + 0.3*0 = 2.0   (buffer[1]=2.0)
    // n=2: 3.0 + 0.3*0 = 3.0   (buffer[2]=3.0)
    // n=3: 4.0 + 0.3*1.0 = 4.3 (buffer[0]=4.0)
    let golden = [1.0_f64, 2.0, 3.0, 4.3];
    assert_eq!(output.as_slice(), &golden);
}
