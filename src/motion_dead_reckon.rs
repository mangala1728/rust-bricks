// ## Map
// Role: core/motion_dead_reckon — 手機慣性導航航位推算（Dead Reckoning）、零速更新（ZUPT）與震動馬達時序生成微積木。
// Inputs: 加速度計模長、陀螺儀角速度模長、時間步長 dt、當前速度與位移、震動模式與時序。
// Outputs: 靜止狀態布林值、更新後之一維速度與位移、馬達高低電位驅動布林值。
// Invariants: 零外部依賴、無遞迴、零全域可變狀態、浮點數防 NaN/Inf 守衛、數值邊界截斷、控制流巢狀深度 <= 2。

/// @outputs: 2
/// @domain: accel_mag [0.0..100.0]
/// @domain: gyro_mag [0.0..100.0]
/// @domain: accel_threshold [0.0..10.0]
/// @domain: gyro_threshold [0.0..10.0]
pub fn zupt_zero_velocity_detect(
    accel_mag: f32,
    gyro_mag: f32,
    accel_threshold: f32,
    gyro_threshold: f32,
) -> bool {
    if accel_mag < 0.0 || accel_mag > 100.0 || gyro_mag < 0.0 || gyro_mag > 100.0 {
        return false;
    }
    if accel_threshold < 0.0 || accel_threshold > 10.0 || gyro_threshold < 0.0 || gyro_threshold > 10.0 {
        return false;
    }
    if accel_mag.is_nan() || gyro_mag.is_nan() || accel_threshold.is_nan() || gyro_threshold.is_nan() {
        return false;
    }
    accel_mag <= accel_threshold && gyro_mag <= gyro_threshold
}

/// @outputs: 1
/// @domain: accel [-100.0..100.0]
/// @domain: dt [0.0..1.0]
/// @domain: current_vel [-100.0..100.0]
/// @domain: current_pos [-10000.0..10000.0]
pub fn dead_reckon_1d_step(
    accel: f32,
    dt: f32,
    is_stationary: bool,
    current_vel: f32,
    current_pos: f32,
) -> (f32, f32) {
    if accel < -100.0 || accel > 100.0 || dt < 0.0 || dt > 1.0 {
        return (0.0, current_pos);
    }
    if current_vel < -100.0 || current_vel > 100.0 || current_pos < -10000.0 || current_pos > 10000.0 {
        return (0.0, current_pos);
    }
    if accel.is_nan() || dt.is_nan() || current_vel.is_nan() || current_pos.is_nan() {
        return (0.0, current_pos);
    }
    if is_stationary {
        return (0.0, current_pos);
    }
    let new_vel = (current_vel + accel * dt).clamp(-100.0, 100.0);
    let avg_vel = (current_vel + new_vel) * 0.5;
    let new_pos = (current_pos + avg_vel * dt).clamp(-10000.0, 10000.0);
    (new_vel, new_pos)
}

/// @outputs: 2
/// @domain: pattern_id [0..3]
/// @domain: elapsed_ms [0..10000]
pub fn haptic_vibration_timing(pattern_id: u8, elapsed_ms: u32) -> bool {
    if pattern_id > 3 || elapsed_ms > 10000 {
        return false;
    }
    match pattern_id {
        0 => elapsed_ms <= 25,
        1 => (elapsed_ms <= 100) || (elapsed_ms >= 200 && elapsed_ms <= 300),
        2 => (elapsed_ms <= 40) || (elapsed_ms >= 90 && elapsed_ms <= 130),
        3 => elapsed_ms <= 60,
        _ => false,
    }
}
