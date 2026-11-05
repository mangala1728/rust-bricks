// ## Map
// Role: Wearable skin temperature EMA filter and medical fever triage
// Inputs: raw_celsius, prev_celsius, alpha, temp_celsius, normal_high, fever_high: f64
// Outputs: Option<f64> or Option<u8>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: raw_celsius in [10.0..=50.0], prev_celsius in [10.0..=50.0], alpha in [0.0..=1.0]
pub fn sensor_skin_temp_smooth(
    raw_celsius: f64,
    prev_celsius: f64,
    alpha: f64,
) -> Option<f64> {
    if !raw_celsius.is_finite()
        || raw_celsius < 10.0
        || raw_celsius > 50.0
        || !prev_celsius.is_finite()
        || prev_celsius < 10.0
        || prev_celsius > 50.0
        || !alpha.is_finite()
        || alpha < 0.0
        || alpha > 1.0
    {
        return None;
    }
    Some(prev_celsius + alpha * (raw_celsius - prev_celsius))
}

/// @outputs: 1
/// @domain: temp in [25.0..=45.0], normal_high in [36.0..=38.0], fever_high in [38.0..=42.0]
pub fn sensor_fever_triage(
    temp_celsius: f64,
    normal_high: f64,
    fever_high: f64,
) -> Option<u8> {
    if !temp_celsius.is_finite()
        || temp_celsius < 25.0
        || temp_celsius > 45.0
        || !normal_high.is_finite()
        || normal_high < 36.0
        || normal_high > 38.0
        || !fever_high.is_finite()
        || fever_high < 38.0
        || fever_high > 42.0
        || normal_high >= fever_high
    {
        return None;
    }

    if temp_celsius < normal_high {
        Some(0)
    } else if temp_celsius < fever_high {
        Some(1)
    } else {
        Some(2)
    }
}
