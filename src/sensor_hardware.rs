// ## Map
// Role: core/sensor_hardware — 手機底層硬體微控制：霍爾感測磁力翻蓋偵測、3軸磁力計傾斜補償指南針、閃光燈 PWM 與 SOS 時序編碼微積木。
// Inputs: 磁場強度 (uT)、門檻值、3軸加速度分量 (ax, ay, az)、3軸磁場分量 (mx, my, mz)、時間戳記 (us/ms)。
// Outputs: 翻蓋開合布林狀態、傾斜補償航向角 (0..360 度)、PWM 高低電位、SOS 閃爍信號狀態。
// Invariants: 零外部依賴、無遞迴、零全域可變狀態、浮點數防 NaN/Inf 守衛、控制流巢狀深度 <= 2、純標準庫 100% 零依賴。

const RAD_TO_DEG: f32 = 180.0 / std::f32::consts::PI;

/// @outputs: 2
/// @domain: current_field_ut [-10000.0..10000.0]
/// @domain: close_th_ut [0.0..10000.0]
/// @domain: open_th_ut [0.0..10000.0]
pub fn hall_magnetic_cover_state(
    current_field_ut: f32,
    close_th_ut: f32,
    open_th_ut: f32,
    prev_state: bool,
) -> bool {
    if current_field_ut.is_nan() || close_th_ut.is_nan() || open_th_ut.is_nan() {
        return prev_state;
    }
    if close_th_ut < open_th_ut {
        return prev_state;
    }
    let mag = current_field_ut.abs();
    if mag >= close_th_ut {
        true
    } else if mag <= open_th_ut {
        false
    } else {
        prev_state
    }
}

/// @outputs: 2
pub fn compass_tilt_compensated_bearing(
    ax: f32,
    ay: f32,
    az: f32,
    mx: f32,
    my: f32,
    mz: f32,
) -> Option<f32> {
    if ax.is_nan() || ay.is_nan() || az.is_nan() || mx.is_nan() || my.is_nan() || mz.is_nan() {
        return None;
    }
    let norm_a = (ax * ax + ay * ay + az * az).sqrt();
    if norm_a <= 1e-4 {
        return None;
    }
    let ax_n = ax / norm_a;
    let ay_n = ay / norm_a;
    let az_n = az / norm_a;

    let pitch = (-ax_n).atan2((ay_n * ay_n + az_n * az_n).sqrt());
    let roll = ay_n.atan2(az_n);

    let cos_pitch = pitch.cos();
    let sin_pitch = pitch.sin();
    let cos_roll = roll.cos();
    let sin_roll = roll.sin();

    let xh = mx * cos_pitch + mz * sin_pitch;
    let yh = mx * sin_roll * sin_pitch + my * cos_roll - mz * sin_roll * cos_pitch;

    let heading_rad = (-yh).atan2(xh);
    let heading_deg = (heading_rad * RAD_TO_DEG + 360.0) % 360.0;
    Some(heading_deg)
}

/// @outputs: 2
/// @domain: frequency_hz [1..100000]
/// @domain: duty_percent [0..100]
pub fn flashlight_pwm_duty(frequency_hz: u32, duty_percent: u8, time_us: u64) -> bool {
    if frequency_hz == 0 || frequency_hz > 100_000 || duty_percent > 100 {
        return false;
    }
    if duty_percent == 0 {
        return false;
    }
    if duty_percent == 100 {
        return true;
    }
    let period_us = 1_000_000u64 / (frequency_hz as u64);
    if period_us == 0 {
        return false;
    }
    let phase_us = time_us % period_us;
    let high_us = (period_us * (duty_percent as u64)) / 100;
    phase_us < high_us
}

/// @outputs: 2
/// @domain: unit_ms [10..2000]
pub fn sos_morse_signal(time_ms: u32, unit_ms: u32) -> bool {
    if unit_ms < 10 || unit_ms > 2000 {
        return false;
    }
    // SOS Pattern in units (1=Dit, 3=Dah, 1=Element Gap, 3=Letter Gap, 7=Word Gap):
    // S: ON, gap, ON, gap, ON -> 1,1,1,1,1 (5 units)
    // Letter Gap: 3 units (total 8)
    // O: ON(3), gap, ON(3), gap, ON(3) -> 3,1,3,1,3 (11 units, total 19)
    // Letter Gap: 3 units (total 22)
    // S: ON, gap, ON, gap, ON -> 1,1,1,1,1 (5 units, total 27)
    // Word Gap: 7 units (total 34 units)
    let total_units = 34u32;
    let cycle_ms = total_units.saturating_mul(unit_ms);
    let pos_in_cycle = time_ms % cycle_ms;
    let unit_idx = pos_in_cycle / unit_ms;

    match unit_idx {
        0 | 2 | 4 => true,               // S: Dit, Dit, Dit
        8..=10 | 12..=14 | 16..=18 => true, // O: Dah, Dah, Dah
        22 | 24 | 26 => true,            // S: Dit, Dit, Dit
        _ => false,
    }
}
