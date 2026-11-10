// ## Map
// Role: Single-step MACD line, signal line, and histogram recursive computation
// Inputs: price, prev_fast, prev_slow, prev_signal, a_fast, a_slow, a_signal: f64
// Outputs: Option<(f64, f64, f64, f64, f64)> (macd_line, signal_line, histogram, next_fast, next_slow)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 5
/// @domain: price > 0.0, 0.0 < a_slow < a_fast <= 1.0, 0.0 < a_signal <= 1.0
pub fn fin_macd_step(
    price: f64,
    prev_fast: f64,
    prev_slow: f64,
    prev_signal: f64,
    a_fast: f64,
    a_slow: f64,
    a_signal: f64,
) -> Option<(f64, f64, f64, f64, f64)> {
    if !price.is_finite()
        || !prev_fast.is_finite()
        || !prev_slow.is_finite()
        || !prev_signal.is_finite()
        || !a_fast.is_finite()
        || !a_slow.is_finite()
        || !a_signal.is_finite()
        || price <= 0.0
        || a_slow <= 0.0
        || a_slow >= a_fast
        || a_fast > 1.0
        || a_signal <= 0.0
        || a_signal > 1.0
    {
        return None;
    }

    let next_fast = a_fast * price + (1.0 - a_fast) * prev_fast;
    let next_slow = a_slow * price + (1.0 - a_slow) * prev_slow;
    let macd_line = next_fast - next_slow;
    let signal_line = a_signal * macd_line + (1.0 - a_signal) * prev_signal;
    let histogram = macd_line - signal_line;

    if !next_fast.is_finite()
        || !next_slow.is_finite()
        || !macd_line.is_finite()
        || !signal_line.is_finite()
        || !histogram.is_finite()
    {
        return None;
    }

    Some((macd_line, signal_line, histogram, next_fast, next_slow))
}
