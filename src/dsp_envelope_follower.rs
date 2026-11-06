// ## Map
// Role: Real-time audio envelope follower with attack/release coefficients
// Inputs: input_sample: f64, prev_env: f64, attack_coeff: f64, release_coeff: f64
// Outputs: Option<f64>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// Real-time dynamic audio envelope follower step calculator.
///
/// @outputs: 1
/// @domain: input_sample in [-1e6..=1e6], prev_env >= 0.0, attack_coeff in [0.0..=1.0], release_coeff in [0.0..=1.0]
pub fn dsp_envelope_follower_step(
    input_sample: f64,
    prev_env: f64,
    attack_coeff: f64,
    release_coeff: f64,
) -> Option<f64> {
    if !input_sample.is_finite()
        || !(-1e6..=1e6).contains(&input_sample)
        || !prev_env.is_finite()
        || prev_env < 0.0
        || !attack_coeff.is_finite()
        || !(0.0..=1.0).contains(&attack_coeff)
        || !release_coeff.is_finite()
        || !(0.0..=1.0).contains(&release_coeff)
    {
        return None;
    }

    let abs_x = input_sample.abs();
    let env = if abs_x > prev_env {
        attack_coeff * prev_env + (1.0 - attack_coeff) * abs_x
    } else {
        release_coeff * prev_env + (1.0 - release_coeff) * abs_x
    };

    Some(env)
}
