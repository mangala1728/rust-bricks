// ## Map
// Role: First-order canonical allpass filter step
// Inputs: input: f64, state: &mut f64, gain: f64
// Outputs: Option<f64>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: |gain| < 1.0, input is finite, *state is finite
pub fn dsp_allpass_first_order_step(
    input: f64,
    state: &mut f64,
    gain: f64,
) -> Option<f64> {
    if !input.is_finite() || !state.is_finite() || !gain.is_finite() || gain <= -1.0 || gain >= 1.0 {
        return None;
    }

    let v = input + gain * (*state);
    let y = -gain * v + (*state);
    *state = v;
    Some(y)
}
