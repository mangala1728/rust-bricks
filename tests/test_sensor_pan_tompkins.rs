use rust_bricks::sensor_pan_tompkins::*;

#[test]
fn derivative_step_golden_vector() {
    // golden input: [0,0,0,0,8] → expected output 2.0 on the 5th call
    let mut history = [0.0_f64; 4];
    let inputs = [0.0, 0.0, 0.0, 0.0, 8.0];
    let mut result = None;

    for &x in &inputs {
        result = pan_tompkins_derivative_step(x, &mut history);
    }

    // the last call should produce the golden value
    let y = result.expect("Derivative step returned None");
    assert!((y - 2.0).abs() < 1e-4, "got {}, expected 2.0", y);
}

#[test]
fn integrate_step_golden_vector() {
    // window size = 4, all entries start at 0
    let mut ring_buf = [0.0_f64; 4];
    let mut head_idx = 0usize;
    let mut running_sum = 0.0_f64;

    // first derivative = 2.0 → square = 4.0, average = 4/4 = 1.0
    let derivative = 2.0_f64;
    let avg = pan_tompkins_integrate_step(
        derivative,
        &mut ring_buf,
        &mut head_idx,
        &mut running_sum,
    )
    .expect("Integrate step returned None");

    assert!((avg - 1.0).abs() < 1e-4, "got {}, expected 1.0", avg);
}
