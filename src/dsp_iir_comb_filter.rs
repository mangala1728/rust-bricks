// ## Map
// Role: Feedback and feedforward comb filter single step computation
// Inputs: x: f64, delayed: f64, gain: f64
// Outputs: Option<f64>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: x in [-1e6..=1e6], delayed_y in [-1e6..=1e6], gain in [-0.999..=0.999]
pub fn dsp_iir_comb_feedback_step(x: f64, delayed_y: f64, gain: f64) -> Option<f64> {
    if !x.is_finite()
        || !delayed_y.is_finite()
        || !gain.is_finite()
        || x < -1e6
        || x > 1e6
        || delayed_y < -1e6
        || delayed_y > 1e6
        || gain < -0.999
        || gain > 0.999
    {
        return None;
    }

    Some(x + gain * delayed_y)
}

/// @outputs: 1
/// @domain: x in [-1e6..=1e6], delayed_x in [-1e6..=1e6], gain in [-1.0..=1.0]
pub fn dsp_comb_feedforward_step(x: f64, delayed_x: f64, gain: f64) -> Option<f64> {
    if !x.is_finite()
        || !delayed_x.is_finite()
        || !gain.is_finite()
        || x < -1e6
        || x > 1e6
        || delayed_x < -1e6
        || delayed_x > 1e6
        || gain < -1.0
        || gain > 1.0
    {
        return None;
    }

    Some(x + gain * delayed_x)
}
