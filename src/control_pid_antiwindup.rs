// ## Map
// Role: Discrete PID step controller with clamping anti-windup protection
// Inputs: setpoint, measured, integral, prev_error, dt, kp, ki, kd, out_min, out_max: f64
// Outputs: Option<(f64, f64)> (output, next_integral)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 2
/// @domain: dt > 0.0, kp >= 0.0, ki >= 0.0, kd >= 0.0, out_min < out_max
pub fn control_pid_step(
    setpoint: f64,
    measured: f64,
    integral: f64,
    prev_error: f64,
    dt: f64,
    kp: f64,
    ki: f64,
    kd: f64,
    out_min: f64,
    out_max: f64,
) -> Option<(f64, f64)> {
    // ---- domain guard -------------------------------------------------------
    if !setpoint.is_finite()
        || !measured.is_finite()
        || !integral.is_finite()
        || !prev_error.is_finite()
        || !dt.is_finite()
        || !kp.is_finite()
        || !ki.is_finite()
        || !kd.is_finite()
        || !out_min.is_finite()
        || !out_max.is_finite()
        || dt <= 0.0
        || kp < 0.0
        || ki < 0.0
        || kd < 0.0
        || out_min >= out_max
    {
        return None;
    }

    // ---- PID computation ----------------------------------------------------
    let error = setpoint - measured;
    let p_term = kp * error;
    let d_term = kd * (error - prev_error) / dt;
    let cand_integral = integral + error * dt;
    let raw_out = p_term + ki * cand_integral + d_term;
    let output = raw_out.clamp(out_min, out_max);

    // ---- Conditional integration anti‑windup --------------------------------
    let next_integral = if (raw_out > out_max && error > 0.0)
        || (raw_out < out_min && error < 0.0)
    {
        integral // freeze integration
    } else {
        cand_integral
    };

    Some((output, next_integral))
}
