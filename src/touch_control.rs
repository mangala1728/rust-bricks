// ## Map
// Role: core/touch_control — 手機觸控螢幕微控制：觸控消抖、滑動速度向量平滑、壓力觸控分級與手勢狀態機微積木。
// Inputs: 觸控開關狀態、取樣間隔 dt、位移增量 (dx, dy)、壓力浮點數、經過時間 (ms)。
// Outputs: 消抖後布林值、(vx, vy, 速度純量)、壓力等級 (0/1/2)、(次態, 觸發事件)。
// Invariants: 零外部依賴、無遞迴、零全域可變狀態、浮點數防 NaN/Inf 守衛、控制流巢狀深度 <= 2、純標準庫 100% 零依賴。

/// @outputs: 2
/// @domain: threshold [1..1000]
pub fn touch_debounce_filter(
    last_state: bool,
    reading: bool,
    stable_count: &mut u32,
    threshold: u32,
) -> bool {
    if threshold == 0 || threshold > 1000 {
        return reading;
    }
    if reading != last_state {
        *stable_count += 1;
        if *stable_count >= threshold {
            *stable_count = 0;
            return reading;
        }
        last_state
    } else {
        *stable_count = 0;
        last_state
    }
}

/// @outputs: 2
/// @domain: dt [0.0001..10.0]
/// @domain: alpha [0.0..1.0]
pub fn touch_velocity_estimate(
    dx: f32,
    dy: f32,
    dt: f32,
    alpha: f32,
    prev_vx: f32,
    prev_vy: f32,
) -> (f32, f32, f32) {
    if dt < 0.0001 || dt > 10.0 || alpha < 0.0 || alpha > 1.0 {
        return (0.0, 0.0, 0.0);
    }
    if dx.is_nan() || dy.is_nan() || prev_vx.is_nan() || prev_vy.is_nan() {
        return (0.0, 0.0, 0.0);
    }
    let raw_vx = dx / dt;
    let raw_vy = dy / dt;
    let vx = alpha * raw_vx + (1.0 - alpha) * prev_vx;
    let vy = alpha * raw_vy + (1.0 - alpha) * prev_vy;
    let speed = (vx * vx + vy * vy).sqrt();
    (vx, vy, speed)
}

/// @outputs: 3
/// @domain: pressure [0.0..10.0]
/// @domain: light_th [0.01..10.0]
/// @domain: heavy_th [0.01..10.0]
pub fn pressure_touch_level(
    pressure: f32,
    light_th: f32,
    heavy_th: f32,
    prev_level: u8,
) -> u8 {
    if pressure.is_nan() || light_th.is_nan() || heavy_th.is_nan() || light_th >= heavy_th {
        return 0;
    }
    let hys = 0.05 * (heavy_th - light_th); // 遲滯容差防止邊界抖動
    match prev_level {
        2 => {
            if pressure < (heavy_th - hys) {
                if pressure < (light_th - hys) { 0 } else { 1 }
            } else {
                2
            }
        }
        1 => {
            if pressure >= (heavy_th + hys) {
                2
            } else if pressure < (light_th - hys) {
                0
            } else {
                1
            }
        }
        _ => {
            if pressure >= (heavy_th + hys) {
                2
            } else if pressure >= (light_th + hys) {
                1
            } else {
                0
            }
        }
    }
}

/// @outputs: 4
/// @domain: state [0..4]
/// @domain: elapsed_ms [0..3600000]
/// @domain: max_tap_ms [10..5000]
/// @domain: double_tap_window_ms [50..5000]
pub fn gesture_tap_state_machine(
    state: u8,
    is_down: bool,
    elapsed_ms: u32,
    max_tap_ms: u32,
    double_tap_window_ms: u32,
) -> (u8, u8) {
    if state > 4 || max_tap_ms < 10 || max_tap_ms > 5000 || double_tap_window_ms < 50 || double_tap_window_ms > 5000 {
        return (0, 0); // 0=None
    }
    // state: 0=IDLE, 1=DOWN, 2=WAIT_DOUBLE, 3=DOUBLE_DOWN
    // events: 0=NONE, 1=SINGLE_TAP, 2=DOUBLE_TAP, 3=LONG_PRESS
    match state {
        0 => if is_down { (1, 0) } else { (0, 0) },
        1 => {
            if !is_down {
                if elapsed_ms <= max_tap_ms { (2, 0) } else { (0, 0) }
            } else if elapsed_ms > max_tap_ms {
                (0, 3) // Long press fired
            } else {
                (1, 0)
            }
        }
        2 => {
            // 必須先檢查雙擊視窗是否逾時，避免逾時後之新按壓仍被誤判為雙擊 (INC-20261026)
            if elapsed_ms > double_tap_window_ms {
                if is_down {
                    (1, 1) // 逾時：結算前次單擊事件，並將本次新按壓轉入 DOWN 態
                } else {
                    (0, 1) // 逾時：結算前次單擊事件
                }
            } else if is_down {
                (3, 0) // 雙擊視窗內再次按下，轉入 DOUBLE_DOWN
            } else {
                (2, 0) // 視窗內持續等待
            }
        }
        3 => {
            if !is_down {
                (0, 2) // Double tap confirmed
            } else {
                (3, 0)
            }
        }
        _ => (0, 0),
    }
}
