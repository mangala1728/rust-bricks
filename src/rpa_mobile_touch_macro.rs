//! # Mobile Touch & RPA Macro Micro-Bricks (手機自動化連點與螢幕觸控巨集微積木)
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 螢幕旋轉坐標映射 (0/90/180/270度)、多指雙指捏合縮放 (Pinch-to-Zoom) 軌跡生成、
//! 連點防偵測常態分佈抖動偏移 (Gaussian Jitter)、OCR/UI 節點包圍盒中心點計算、
//! 滑動慣性甩動 (Fling Velocity) 指數衰減、Android 常用 Keycode 映射校驗、
//! 無障礙節點邊界字串 (Bounds [x1,y1][x2,y2]) 解析、擬人彎曲手勢路徑內插、長按/拖曳遲滯判定。

/// ## Map: 螢幕旋轉姿態 (0/90/180/270度) 實體觸控坐標映射 (Screen Rotation Coordinate Transform)
/// 回傳旋轉後的 (mapped_x, mapped_y)
/// @outputs: 2
/// @domain: raw_x [0, 10000]
/// @domain: raw_y [0, 10000]
/// @domain: screen_width [1, 10000]
/// @domain: screen_height [1, 10000]
/// @domain: rotation_deg [0, 270]
pub fn rpa_screen_rotate_coords(
    raw_x: u32,
    raw_y: u32,
    screen_width: u32,
    screen_height: u32,
    rotation_deg: u16,
) -> (u32, u32) {
    if screen_width == 0 || screen_height == 0 {
        return (0, 0);
    }
    let x = raw_x.min(screen_width - 1);
    let y = raw_y.min(screen_height - 1);
    match rotation_deg {
        90 => (y, screen_width.saturating_sub(1).saturating_sub(x)),
        180 => (
            screen_width.saturating_sub(1).saturating_sub(x),
            screen_height.saturating_sub(1).saturating_sub(y),
        ),
        270 => (screen_height.saturating_sub(1).saturating_sub(y), x),
        _ => (x, y), // 0度 (Portrait)
    }
}

/// ## Gen: 雙指縮放 (Pinch-to-Zoom) 兩觸控點步進坐標生成
/// progress 0.0 為起點，1.0 為終點；span_factor > 1.0 為放大 (Zoom in)，< 1.0 為縮小 (Zoom out)
/// 回傳 ((finger1_x, finger1_y), (finger2_x, finger2_y))
/// @outputs: 4
/// @domain: progress [0.0, 1.0]
/// @domain: initial_distance [1.0, 5000.0]
/// @domain: span_factor [0.01, 10.0]
pub fn rpa_pinch_to_zoom_step(
    center_x: f32,
    center_y: f32,
    initial_distance: f32,
    span_factor: f32,
    progress: f32,
) -> ((f32, f32), (f32, f32)) {
    if !center_x.is_finite() || !center_y.is_finite() || !initial_distance.is_finite() ||
       !span_factor.is_finite() || !progress.is_finite() || initial_distance <= 0.0 {
        return ((0.0, 0.0), (0.0, 0.0));
    }
    let t = progress.clamp(0.0, 1.0);
    let current_dist = initial_distance * (1.0 + (span_factor - 1.0) * t);
    let half_span = current_dist * 0.5;

    let f1 = (center_x - half_span, center_y);
    let f2 = (center_x + half_span, center_y);
    (f1, f2)
}

/// ## Offset: 連點巨集擬人化高斯抖動 (Box-Muller Gaussian Jitter)
/// 輸入兩組 0.0..1.0 偽隨機均勻採樣，產生符合指定標準差之 (dx, dy) 偏移
/// @outputs: 2
/// @domain: u1 [0.00001, 1.0]
/// @domain: u2 [0.0, 1.0]
/// @domain: sigma_px [0.0, 100.0]
pub fn rpa_human_jitter_offset(u1: f32, u2: f32, sigma_px: f32) -> (f32, f32) {
    if !u1.is_finite() || !u2.is_finite() || !sigma_px.is_finite() || u1 <= 0.0 || sigma_px < 0.0 {
        return (0.0, 0.0);
    }
    let clamped_u1 = u1.clamp(1e-5, 1.0);
    let r = (-2.0 * clamped_u1.ln()).sqrt();
    let theta = 2.0 * std::f32::consts::PI * u2;
    let z0 = r * theta.cos();
    let z1 = r * theta.sin();
    (z0 * sigma_px, z1 * sigma_px)
}

/// ## Center: UI 元素 / OCR 辨識包圍盒 (Bounding Box) 點擊中心點解算
/// @outputs: 2
/// @domain: x_min [0, 10000]
/// @domain: y_min [0, 10000]
/// @domain: x_max [0, 10000]
/// @domain: y_max [0, 10000]
pub fn rpa_bbox_click_target_center(x_min: u32, y_min: u32, x_max: u32, y_max: u32) -> (u32, u32) {
    if x_min > x_max || y_min > y_max {
        return (x_min, y_min);
    }
    let cx = x_min + (x_max - x_min) / 2;
    let cy = y_min + (y_max - y_min) / 2;
    (cx, cy)
}

/// ## Decay: 觸控滾動滑動釋放後慣性甩動速度衰減 (Fling Velocity Exponential Decay)
/// 回傳在經過 dt_s 秒後的剩餘滑動速度 (px/s) 與位移增量 (px)
/// @outputs: 2
/// @domain: initial_vel [0.0, 50000.0]
/// @domain: friction_coeff [0.1, 50.0]
/// @domain: dt_s [0.001, 10.0]
pub fn rpa_fling_velocity_decay(
    initial_vel: f32,
    friction_coeff: f32,
    dt_s: f32,
) -> (f32, f32) {
    if !initial_vel.is_finite() || !friction_coeff.is_finite() || !dt_s.is_finite() ||
       initial_vel <= 0.0 || friction_coeff <= 0.0 || dt_s <= 0.0 {
        return (0.0, 0.0);
    }
    let decay_factor = (-friction_coeff * dt_s).exp();
    let v_t = initial_vel * decay_factor;
    // 位移積分: delta_s = (v0 / friction) * (1 - exp(-friction * dt))
    let delta_s = (initial_vel / friction_coeff) * (1.0 - decay_factor);
    (v_t, delta_s)
}

/// ## Key: Android 常見虛擬按鍵碼 (Keycode) 驗證與名稱對照
/// 0x03: HOME, 0x04: BACK, 0x18: VOLUME_UP, 0x19: VOLUME_DOWN, 0x1A: POWER, 0x42: ENTER
/// @outputs: 1
pub fn rpa_android_keycode_valid(keycode: u32) -> Option<&'static str> {
    match keycode {
        3 => Some("KEYCODE_HOME"),
        4 => Some("KEYCODE_BACK"),
        24 => Some("KEYCODE_VOLUME_UP"),
        25 => Some("KEYCODE_VOLUME_DOWN"),
        26 => Some("KEYCODE_POWER"),
        66 => Some("KEYCODE_ENTER"),
        82 => Some("KEYCODE_MENU"),
        187 => Some("KEYCODE_APP_SWITCH"),
        _ => None,
    }
}

/// ## Parse: Android 無障礙節點邊界字串 (Bounds "[x1,y1][x2,y2]") 解析
/// @outputs: 1
pub fn rpa_parse_accessibility_bounds_text(bounds_str: &[u8]) -> Option<[u32; 4]> {
    // 預期格式如 b"[100,200][300,400]"
    if bounds_str.len() < 9 || bounds_str[0] != b'[' {
        return None;
    }
    let mut nums = [0u32; 4];
    let mut num_idx = 0;
    let mut current_val = 0u32;
    let mut has_digit = false;

    for &b in bounds_str {
        if b.is_ascii_digit() {
            current_val = current_val.saturating_mul(10).saturating_add((b - b'0') as u32);
            has_digit = true;
        } else if b == b',' || b == b']' {
            if has_digit && num_idx < 4 {
                nums[num_idx] = current_val;
                num_idx += 1;
                current_val = 0;
                has_digit = false;
            }
        }
    }
    if num_idx == 4 && nums[0] <= nums[2] && nums[1] <= nums[3] {
        Some(nums)
    } else {
        None
    }
}

/// ## Path: 擬人手勢滑動微彎曲路徑內插 (Curved Swipe Waypoint)
/// 使用二階貝茲曲線帶有側向偏置弧度
/// @outputs: 2
/// @domain: progress [0.0, 1.0]
/// @domain: arch_height_px [-500.0, 500.0]
pub fn rpa_curved_swipe_waypoint(
    start: (f32, f32),
    end: (f32, f32),
    arch_height_px: f32,
    progress: f32,
) -> (f32, f32) {
    if !start.0.is_finite() || !start.1.is_finite() || !end.0.is_finite() || !end.1.is_finite() ||
       !arch_height_px.is_finite() || !progress.is_finite() {
        return start;
    }
    let t = progress.clamp(0.0, 1.0);
    let mid_x = (start.0 + end.0) * 0.5;
    let mid_y = (start.1 + end.1) * 0.5;

    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    let len = (dx * dx + dy * dy).sqrt();
    let (ctrl_x, ctrl_y) = if len > 1e-4 {
        // 法向量 (-dy, dx)
        let nx = -dy / len;
        let ny = dx / len;
        (mid_x + nx * arch_height_px, mid_y + ny * arch_height_px)
    } else {
        (mid_x, mid_y)
    };

    // 二階貝茲: (1-t)^2 * P0 + 2*(1-t)*t * P1 + t^2 * P2
    let omt = 1.0 - t;
    let px = omt * omt * start.0 + 2.0 * omt * t * ctrl_x + t * t * end.0;
    let py = omt * omt * start.1 + 2.0 * omt * t * ctrl_y + t * t * end.1;
    (px, py)
}

/// ## State: 觸控長按/拖曳遲滯判定狀態機 (Hold vs Drag Hysteresis)
/// 回傳 0: TAP (輕點), 1: LONG_PRESS (長按), 2: DRAG (拖曳移動)
/// @outputs: 1
/// @domain: move_dist_px [0.0, 10000.0]
/// @domain: duration_ms [0, 60000]
/// @domain: slop_px [1.0, 100.0]
/// @domain: long_press_timeout_ms [100, 5000]
pub fn rpa_touch_gesture_state(
    move_dist_px: f32,
    duration_ms: u32,
    slop_px: f32,
    long_press_timeout_ms: u32,
) -> u8 {
    if !move_dist_px.is_finite() || !slop_px.is_finite() || move_dist_px < 0.0 || slop_px <= 0.0 {
        return 0;
    }
    if move_dist_px > slop_px {
        2 // DRAG (移動距離超過防手震臨界值 Slop)
    } else if duration_ms >= long_press_timeout_ms {
        1 // LONG_PRESS (保持在 Slop 範圍內且時間達標)
    } else {
        0 // TAP
    }
}

/// ## Match: 螢幕像素 RGB 色彩容差快速比對 (Pixel Color Tolerance Match)
/// 各通道色彩差異均不大於 max_channel_diff 則判定吻合
/// @outputs: 1
/// @domain: max_channel_diff [0, 255]
pub fn rpa_pixel_color_tolerance_match(
    c1: [u8; 3],
    c2: [u8; 3],
    max_channel_diff: u8,
) -> bool {
    let dr = (c1[0] as i16 - c2[0] as i16).abs() as u8;
    let dg = (c1[1] as i16 - c2[1] as i16).abs() as u8;
    let db = (c1[2] as i16 - c2[2] as i16).abs() as u8;
    dr <= max_channel_diff && dg <= max_channel_diff && db <= max_channel_diff
}

