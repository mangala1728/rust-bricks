// ## Map
// Role: 3-pole IIR pink noise generator filter
// Inputs: white: f64, state: &mut [f64; 3]
// Outputs: Option<f64>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: white [-1.0..=1.0]
pub fn dsp_pink_filter_step(white: f64, state: &mut [f64; 3]) -> Option<f64> {
    if white < -1.0
        || white > 1.0
        || !white.is_finite()
        || !state[0].is_finite()
        || !state[1].is_finite()
        || !state[2].is_finite()
    {
        return None;
    }

    state[0] = 0.99765 * state[0] + white * 0.0990460;
    state[1] = 0.96300 * state[1] + white * 0.2965164;
    state[2] = 0.57000 * state[2] + white * 1.0526913;
    let out = state[0] + state[1] + state[2] + white * 0.1848;

    Some(out)
}

/// @outputs: 1
/// @domain: white_samples.len() [0..65536]
pub fn dsp_pink_filter_slice(
    white_samples: &[f64],
    state: &mut [f64; 3],
    output: &mut [f64],
) -> usize {
    if white_samples.len() > 65536 || output.len() < white_samples.len() {
        return 0;
    }
    for (i, &w) in white_samples.iter().enumerate() {
        match dsp_pink_filter_step(w, state) {
            Some(val) => output[i] = val,
            None => return i,
        }
    }
    white_samples.len()
}
