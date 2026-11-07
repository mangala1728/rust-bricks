// ## Map
// Role: Speech signal pre-emphasis high-frequency boost and de-emphasis recovery
// Inputs: sample: f64, prev_sample: f64, alpha: f64
// Outputs: Option<f64>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: x in [-1e6..=1e6], prev_x in [-1e6..=1e6], alpha in [0.0..=1.0]
pub fn dsp_preemphasis_step(x: f64, prev_x: f64, alpha: f64) -> Option<f64> {
    if !x.is_finite()
        || !prev_x.is_finite()
        || !alpha.is_finite()
        || x < -1e6
        || x > 1e6
        || prev_x < -1e6
        || prev_x > 1e6
        || alpha < 0.0
        || alpha > 1.0
    {
        return None;
    }

    let result = x - alpha * prev_x;
    if result.is_finite() {
        Some(result)
    } else {
        None
    }
}

/// @outputs: 1
/// @domain: y in [-1e6..=1e6], prev_x in [-1e6..=1e6], alpha in [0.0..=1.0]
pub fn dsp_deemphasis_step(y: f64, prev_x: f64, alpha: f64) -> Option<f64> {
    if !y.is_finite()
        || !prev_x.is_finite()
        || !alpha.is_finite()
        || y < -1e6
        || y > 1e6
        || prev_x < -1e6
        || prev_x > 1e6
        || alpha < 0.0
        || alpha > 1.0
    {
        return None;
    }

    let result = y + alpha * prev_x;
    if result.is_finite() {
        Some(result)
    } else {
        None
    }
}
