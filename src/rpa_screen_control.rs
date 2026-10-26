// ## Map
// Role: rpa/rpa_screen_control — 零依賴土法自動化操作 (按鍵精靈 / RPA / 螢幕取色找色)、擬人滑鼠軌跡與 GPIB/RS485 儀控協調微積木。
// Inputs: RGB 像素顏色、容差閾值、螢幕影像緩衝區、起點/終點/控制點座標、時間戳、GPIB 信號線狀態、波特率。
// Outputs: 顏色感知距離、顏色匹配結果、找到之 (x, y) 座標、擬人貝茲座標、雙擊判定、GPIB 狀態機碼、RS485 字符微秒時間。
// Invariants: 零外部依賴、純 Rust 實現、除零剛性防護、有界座標搜尋、單一函式 <= 60 行。

/// @outputs: 1
pub fn rpa_color_diff_rgb(c1: (u8, u8, u8), c2: (u8, u8, u8)) -> u32 {
    let r_mean = ((c1.0 as i32) + (c2.0 as i32)) / 2;
    let dr = (c1.0 as i32) - (c2.0 as i32);
    let dg = (c1.1 as i32) - (c2.1 as i32);
    let db = (c1.2 as i32) - (c2.2 as i32);

    let weight_r = 2 + (r_mean >> 8);
    let weight_g = 4;
    let weight_b = 2 + ((255 - r_mean) >> 8);

    let diff_sq = (weight_r * dr * dr) + (weight_g * dg * dg) + (weight_b * db * db);
    (diff_sq as f64).sqrt().round() as u32
}

/// @outputs: 1
/// @domain: tolerance [0..255]
pub fn rpa_color_match_tolerance(c1: (u8, u8, u8), c2: (u8, u8, u8), tolerance: u8) -> bool {
    let diff_r = (c1.0 as i16 - c2.0 as i16).abs() as u8;
    let diff_g = (c1.1 as i16 - c2.1 as i16).abs() as u8;
    let diff_b = (c1.2 as i16 - c2.2 as i16).abs() as u8;
    diff_r <= tolerance && diff_g <= tolerance && diff_b <= tolerance
}

/// @outputs: 2
/// @domain: width [1..8192]
/// @domain: height [1..8192]
pub fn rpa_screen_find_color_patch(
    pixels: &[u8],
    width: usize,
    height: usize,
    target_color: (u8, u8, u8),
    tolerance: u8,
) -> Option<(usize, usize)> {
    if width == 0 || height == 0 || pixels.len() < width * height * 3 || width > 8192 || height > 8192 {
        return None;
    }
    let bpp = if pixels.len() >= width * height * 4 { 4 } else { 3 };
    for y in 0..height {
        let row_offset = y * width * bpp;
        for x in 0..width {
            let px_offset = row_offset + x * bpp;
            let current_color = (pixels[px_offset], pixels[px_offset + 1], pixels[px_offset + 2]);
            if rpa_color_match_tolerance(current_color, target_color, tolerance) {
                return Some((x, y));
            }
        }
    }
    None
}

/// @outputs: 1
/// @domain: t [0.0..1.0]
pub fn rpa_mouse_human_trajectory_point(
    start: (f32, f32),
    end: (f32, f32),
    ctrl: (f32, f32),
    t: f32,
) -> (f32, f32) {
    if !t.is_finite() || !start.0.is_finite() || !end.0.is_finite() {
        return (0.0, 0.0);
    }
    let clamped_t = t.clamp(0.0, 1.0);
    let u = 1.0 - clamped_t;
    // 二次貝茲曲線模擬人手微彎曲移動路徑
    let x = u * u * start.0 + 2.0 * u * clamped_t * ctrl.0 + clamped_t * clamped_t * end.0;
    let y = u * u * start.1 + 2.0 * u * clamped_t * ctrl.1 + clamped_t * clamped_t * end.1;
    (x, y)
}

/// @outputs: 1
/// @domain: double_click_threshold_ms [50..2000]
pub fn rpa_click_double_click_detect(prev_click_ms: u64, now_ms: u64, double_click_threshold_ms: u64) -> bool {
    if now_ms < prev_click_ms || double_click_threshold_ms < 50 || double_click_threshold_ms > 2000 {
        return false;
    }
    let elapsed = now_ms - prev_click_ms;
    elapsed > 10 && elapsed <= double_click_threshold_ms
}

/// @outputs: 1
pub fn gpib_ieee488_handshake_step(dav: bool, nrfd: bool, ndac: bool) -> u8 {
    // IEEE 488.1 3 線交握狀態解碼:
    // dav: true=發送端數據有效, nrfd: true=所有接收端準備好, ndac: true=所有接收端已接收
    if !dav && !nrfd && !ndac {
        0 // 閒置狀態 (Idle / Initial)
    } else if !dav && nrfd && !ndac {
        1 // 接收者就緒，等待發送者掛上數據 (All Listeners Ready)
    } else if dav && !nrfd && !ndac {
        2 // 數據傳輸中，接收者正在讀取 (Data Valid, Reading)
    } else if dav && !nrfd && ndac {
        3 // 數據已被所有接收端鎖存接收 (Data Accepted)
    } else {
        4 // 交握週期結束復位中 (Cycle Complete / Transition)
    }
}

/// @outputs: 1
/// @domain: baud_rate [300..10000000]
/// @domain: data_bits [5..9]
pub fn rs485_calc_char_tx_time_us(baud_rate: u32, data_bits: u8, stop_bits: u8, has_parity: bool) -> u32 {
    if baud_rate == 0 || data_bits < 5 || data_bits > 9 || stop_bits == 0 || stop_bits > 2 {
        return 0;
    }
    let total_bits = 1 + (data_bits as u32) + (stop_bits as u32) + (if has_parity { 1 } else { 0 });
    let total_us = ((total_bits as u64) * 1_000_000) / (baud_rate as u64);
    total_us as u32
}
