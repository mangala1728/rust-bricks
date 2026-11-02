// ## Map
// Role:   DSP primitive – comb‑filter building block
// Inputs: input: f64, buffer: &mut [f64], write_pos: &mut usize,
//         feedback_gain / blend: f64
// Outputs: 1 (filtered sample)
// Invariants:
//   - buffer.len() >= 1
//   - *write_pos < buffer.len()
//   - |feedback_gain| < 1.0  (feedback case)
//   - blend is finite        (feed‑forward case)
//   - all numeric arguments are finite

/// @outputs: 1
/// @domain: input is finite, feedback_gain ∈ (-1.0, 1.0), buffer_len >= 1,
///          write_pos < buffer_len
///
/// Feedback‑comb filter step.
///
/// Returns `None` if any pre‑condition is violated, otherwise the computed
/// output sample.
pub fn dsp_feedback_comb_step(
    input: f64,
    buffer: &mut [f64],
    write_pos: &mut usize,
    feedback_gain: f64,
) -> Option<f64> {
    // ---- contract guards ----------------------------------------------------
    if !input.is_finite()
        || !feedback_gain.is_finite()
        || feedback_gain <= -1.0
        || feedback_gain >= 1.0
        || buffer.is_empty()
    {
        return None;
    }

    let buf_len = buffer.len();
    if *write_pos >= buf_len {
        return None;
    }

    // ---- algorithm ---------------------------------------------------------
    let delayed = buffer[*write_pos];
    let output = input + feedback_gain * delayed;

    buffer[*write_pos] = output;
    *write_pos = (*write_pos + 1) % buf_len;

    Some(output)
}

/// @outputs: 1
/// @domain: input is finite, blend is finite, buffer_len >= 1,
///          write_pos < buffer_len
///
/// Feed‑forward comb filter step.
///
/// Returns `None` if any pre‑condition is violated, otherwise the computed
/// output sample.
pub fn dsp_feedforward_comb_step(
    input: f64,
    buffer: &mut [f64],
    write_pos: &mut usize,
    blend: f64,
) -> Option<f64> {
    // ---- contract guards ----------------------------------------------------
    if !input.is_finite() || !blend.is_finite() || buffer.is_empty() {
        return None;
    }

    let buf_len = buffer.len();
    if *write_pos >= buf_len {
        return None;
    }

    // ---- algorithm ---------------------------------------------------------
    let delayed = buffer[*write_pos];
    let output = input + blend * delayed;

    buffer[*write_pos] = input;
    *write_pos = (*write_pos + 1) % buf_len;

    Some(output)
}
