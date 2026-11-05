// ## Map
// Role: Accelerometer SVM freefall and impact fall detection state machine
// Inputs: ax: f64, ay: f64, az: f64, svm: f64, lower_threshold: f64, upper_threshold: f64, in_freefall: bool
// Outputs: Option<f64> or Option<(bool, bool)>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: ax, ay, az are finite
pub fn sensor_svm_magnitude(ax: f64, ay: f64, az: f64) -> Option<f64> {
    if !ax.is_finite() || !ay.is_finite() || !az.is_finite() {
        return None;
    }
    Some((ax * ax + ay * ay + az * az).sqrt())
}

/// @outputs: 2
/// @domain: svm >= 0.0, 0.0 <= lower_threshold < upper_threshold
pub fn sensor_fall_detect_impact(
    svm: f64,
    lower_threshold: f64,
    upper_threshold: f64,
    in_freefall: bool,
) -> Option<(bool, bool)> {
    if !svm.is_finite()
        || svm < 0.0
        || !lower_threshold.is_finite()
        || lower_threshold < 0.0
        || !upper_threshold.is_finite()
        || lower_threshold >= upper_threshold
    {
        return None;
    }

    if in_freefall && svm >= upper_threshold {
        Some((false, true))
    } else if svm <= lower_threshold {
        Some((true, false))
    } else {
        Some((in_freefall, false))
    }
}
