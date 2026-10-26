// ## Map
// Role: core/thermal_pid — 工業恆溫烙鐵/恆溫恆濕 PID 閉環控制、NTC 測溫、人眼知覺 LED 映射與電量監測微積木。
// Inputs: 目標溫度、當前測量值、PID 增益與時間步長、NTC 電阻值、線性亮度等級、電壓電流。
// Outputs: PID 控制輸出量與積分項、換算攝氏溫度 Option<f32>、對數感知 PWM Duty、有功電功率 (Watts)。
// Invariants: 零外部依賴、無遞迴、零全域可變狀態、抗積分飽和、浮點數防 NaN 守衛、控制流巢狀深度 <= 2。

const ABSOLUTE_ZERO_CELSIUS: f32 = -273.15;

/// @outputs: 1
/// @domain: dt [0.0001..100.0]
/// @domain: out_min [-10000.0..10000.0]
/// @domain: out_max [-10000.0..10000.0]
pub fn pid_controller_step(
    setpoint: f32,
    measured: f32,
    integral: f32,
    prev_error: f32,
    kp: f32,
    ki: f32,
    kd: f32,
    dt: f32,
    out_min: f32,
    out_max: f32,
) -> (f32, f32) {
    if dt < 0.0001 || dt > 100.0 || out_min > out_max {
        return (out_min, integral);
    }
    if setpoint.is_nan() || measured.is_nan() || integral.is_nan() || prev_error.is_nan() {
        return (out_min, 0.0);
    }
    let error = setpoint - measured;
    let new_integral = (integral + error * dt).clamp(out_min, out_max);
    let derivative = (error - prev_error) / dt;
    let raw_out = kp * error + ki * new_integral + kd * derivative;
    let clamped_out = raw_out.clamp(out_min, out_max);
    (clamped_out, new_integral)
}

/// @outputs: 2
/// @domain: r_ohms [0.1..10000000.0]
/// @domain: r0 [0.1..10000000.0]
/// @domain: beta [100.0..10000.0]
pub fn ntc_resistance_to_celsius(
    r_ohms: f32,
    r0: f32,
    beta: f32,
    t0_celsius: f32,
) -> Option<f32> {
    if r_ohms <= 0.1 || r0 <= 0.1 || beta <= 100.0 || beta > 10000.0 {
        return None;
    }
    if r_ohms.is_nan() || r0.is_nan() || beta.is_nan() || t0_celsius.is_nan() {
        return None;
    }
    let t0_kelvin = t0_celsius - ABSOLUTE_ZERO_CELSIUS;
    if t0_kelvin <= 0.0 {
        return None;
    }
    let ratio = r_ohms / r0;
    let ln_ratio = ratio.ln();
    let inv_t = (1.0 / t0_kelvin) + (ln_ratio / beta);
    if inv_t <= 0.0 {
        return None;
    }
    let t_celsius = (1.0 / inv_t) + ABSOLUTE_ZERO_CELSIUS;
    Some(t_celsius)
}

/// @outputs: 1
/// @domain: linear_level [0..65535]
/// @domain: max_level [1..65535]
pub fn led_perceptual_pwm_duty(linear_level: u16, max_level: u16) -> u16 {
    if max_level == 0 {
        return 0;
    }
    let level = linear_level.min(max_level);
    let norm = level as f32 / max_level as f32;
    let perceptual = norm * norm;
    (perceptual * max_level as f32).round() as u16
}

/// @outputs: 1
/// @domain: voltage_v [-10000.0..10000.0]
/// @domain: current_a [-1000.0..1000.0]
pub fn electrical_power_watts(voltage_v: f32, current_a: f32) -> f32 {
    if voltage_v.is_nan() || current_a.is_nan() {
        return 0.0;
    }
    if voltage_v < -10000.0 || voltage_v > 10000.0 || current_a < -1000.0 || current_a > 1000.0 {
        return 0.0;
    }
    (voltage_v * current_a).clamp(-10_000_000.0, 10_000_000.0)
}
