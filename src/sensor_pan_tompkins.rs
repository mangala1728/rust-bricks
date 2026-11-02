// ## Map
// Role: 5‑point derivative filter & moving‑window integration for Pan‑Tompkins QRS detection
// Inputs:  x_n (f64) – current sample, history[4] (f64) – previous 4 samples
//          derivative (f64) – output of derivative filter
//          ring_buf (slice of f64) – circular buffer for squared values
//          head_idx (usize) – write index into ring_buf
//          running_sum (f64) – sum of values currently stored in ring_buf
// Outputs: y (f64) – derivative output, mwi (f64) – moving‑window integrated value
// Invariants:
//   - All numeric arguments stay inside the declared domain.
//   - `history` always holds the four most recent raw samples in order
//     [x[n‑1], x[n‑2], x[n‑3], x[n‑4]].
//   - `ring_buf.len() > 0` and `head_idx < ring_buf.len()`.
//   - `running_sum` equals the sum of all elements currently stored in `ring_buf`.

/// @outputs: 1
/// @domain: x_n [-1.0e6..1.0e6]
///
/// Pan‑Tompkins 5‑point derivative (single step).
///
/// Implements the difference equation
///   y[n] = (2·x[n] + x[n‑1] – x[n‑3] – 2·x[n‑4]) / 8
///
/// `history` holds `[x[n‑1], x[n‑2], x[n‑3], x[n‑4]]`.  After the computation the
/// history is shifted right and the new sample `x_n` is stored at index 0.
///
/// Returns `None` if `x_n` is outside the allowed domain.
pub fn pan_tompkins_derivative_step(
    x_n: f64,
    history: &mut [f64; 4],
) -> Option<f64> {
    // @safety-proof: simple bounds check, no unsafe code
    if x_n < -1.0e6 || x_n > 1.0e6 {
        return None;
    }

    // compute derivative according to the formula
    let y = (2.0 * x_n
        + history[0]          // x[n‑1]
        - history[2]          // x[n‑3]
        - 2.0 * history[3])   // x[n‑4]
        / 8.0;

    // shift history: [x[n‑1], x[n‑2], x[n‑3], x[n‑4]] → [x[n], x[n‑1], x[n‑2], x[n‑3]]
    history[3] = history[2];
    history[2] = history[1];
    history[1] = history[0];
    history[0] = x_n;

    Some(y)
}

/// @outputs: 1
/// @domain: derivative [-1.0e6..1.0e6]
///
/// Square the derivative and update a moving‑window integration buffer.
///
/// `ring_buf` is a circular buffer that always contains the last `N` squared
/// derivative values, where `N = ring_buf.len()`.  `head_idx` points to the slot
/// that will be overwritten next.  `running_sum` holds the sum of the current
/// contents of `ring_buf`.  The function updates these structures in O(1) time
/// and returns the average value of the window.
///
/// Returns `None` if the derivative is out of domain or if `ring_buf` is empty.
pub fn pan_tompkins_integrate_step(
    derivative: f64,
    ring_buf: &mut [f64],
    head_idx: &mut usize,
    running_sum: &mut f64,
) -> Option<f64> {
    // @safety-proof: bounds check, no unsafe code
    if derivative < -1.0e6 || derivative > 1.0e6 {
        return None;
    }
    if ring_buf.is_empty() {
        return None;
    }
    // ensure head_idx is always a valid index
    if *head_idx >= ring_buf.len() {
        *head_idx = 0;
    }

    // square the derivative
    let s = derivative * derivative;

    // update running sum using the value that will be overwritten
    *running_sum = *running_sum - ring_buf[*head_idx] + s;
    ring_buf[*head_idx] = s;

    // advance circular index
    *head_idx = (*head_idx + 1) % ring_buf.len();

    // compute average
    let avg = *running_sum / (ring_buf.len() as f64);
    Some(avg)
}
