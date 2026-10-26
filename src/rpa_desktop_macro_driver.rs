//! # Desktop RPA & OS Virtual Input Macro Micro-Bricks (桌面自動化與作業系統虛擬輸入微積木)
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 Win32 / Linux X11 虛擬鍵碼 (Virtual Keycode) 映射、擬人貝茲曲線滑鼠軌跡步進、
//! 螢幕像素座標至 SendInput 65535 絕對空間歸一化、滑鼠點擊標誌封裝、
//! 鍵盤組合熱鍵修飾符掩碼、OCR 文字模糊容錯編輯距離 (Levenshtein)、
//! 螢幕像素色差比對與首個匹配點搜尋、擬人按鍵延遲抖動 (Timing Jitter)。

/// ## Map: 常用鍵名映射為標準 Win32 Virtual-Key Code (VK Code)
/// @outputs: 1
pub fn rpa_win32_vk_code_map(key_name: &str) -> Option<u16> {
    if key_name.is_empty() {
        return None;
    }
    match key_name.to_ascii_uppercase().as_str() {
        "ENTER" | "RETURN" => Some(0x0D),
        "ESC" | "ESCAPE" => Some(0x1B),
        "SPACE" => Some(0x20),
        "TAB" => Some(0x09),
        "BACKSPACE" | "BKSP" => Some(0x08),
        "SHIFT" => Some(0x10),
        "CTRL" | "CONTROL" => Some(0x11),
        "ALT" => Some(0x12),
        "WIN" | "SUPER" => Some(0x5B),
        "UP" => Some(0x26),
        "DOWN" => Some(0x28),
        "LEFT" => Some(0x25),
        "RIGHT" => Some(0x27),
        "F1" => Some(0x70),
        "F2" => Some(0x71),
        "F3" => Some(0x72),
        "F4" => Some(0x73),
        "F5" => Some(0x74),
        "F12" => Some(0x7B),
        s if s.len() == 1 => {
            let ch = s.as_bytes()[0];
            if (b'A'..=b'Z').contains(&ch) || (b'0'..=b'9').contains(&ch) {
                Some(ch as u16)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// ## Map: 常用鍵名映射為 Linux X11 Keysym 代碼
/// @outputs: 1
pub fn rpa_x11_keysym_map(key_name: &str) -> Option<u32> {
    if key_name.is_empty() {
        return None;
    }
    match key_name.to_ascii_uppercase().as_str() {
        "ENTER" | "RETURN" => Some(0xFF0D),
        "ESC" | "ESCAPE" => Some(0xFF1B),
        "SPACE" => Some(0x0020),
        "TAB" => Some(0xFF09),
        "BACKSPACE" => Some(0xFF08),
        "SHIFT" => Some(0xFFE1),
        "CTRL" | "CONTROL" => Some(0xFFE3),
        "ALT" => Some(0xFFE9),
        "UP" => Some(0xFF52),
        "DOWN" => Some(0xFF54),
        "LEFT" => Some(0xFF51),
        "RIGHT" => Some(0xFF53),
        _ => None,
    }
}

/// ## Step: 擬人二次貝茲曲線滑鼠軌跡點計算 (Human-like Quadratic Bezier Step)
/// 公式：B(t) = (1-t)^2 * p0 + 2(1-t)t * p1 + t^2 * p2
/// @outputs: 1
/// @domain: t [0.0, 1.0]
pub fn rpa_mouse_bezier_human_step(
    p0: (f64, f64),
    p1: (f64, f64),
    p2: (f64, f64),
    t: f64,
) -> (i32, i32) {
    if !t.is_finite() || t < 0.0 || t > 1.0 {
        return (p0.0.round() as i32, p0.1.round() as i32);
    }
    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;
    let ut2 = 2.0 * u * t;

    let x = uu * p0.0 + ut2 * p1.0 + tt * p2.0;
    let y = uu * p0.1 + ut2 * p1.1 + tt * p2.1;
    (x.round() as i32, y.round() as i32)
}

/// ## Convert: 螢幕像素座標轉 Windows SendInput 絕對坐標 (0..65535 空間歸一化)
/// 公式：norm_x = (x * 65535) / (width - 1), norm_y = (y * 65535) / (height - 1)
/// @outputs: 1
/// @domain: x [0, 100000], y [0, 100000], screen_width [1, 100000], screen_height [1, 100000]
pub fn rpa_screen_coords_to_sendinput_normalized(
    x: i32,
    y: i32,
    screen_width: i32,
    screen_height: i32,
) -> (u16, u16) {
    if screen_width <= 1 || screen_height <= 1 || x < 0 || y < 0 {
        return (0, 0);
    }
    let clamped_x = x.min(screen_width - 1) as u64;
    let clamped_y = y.min(screen_height - 1) as u64;

    let norm_x = ((clamped_x * 65535) / ((screen_width - 1) as u64)) as u16;
    let norm_y = ((clamped_y * 65535) / ((screen_height - 1) as u64)) as u16;
    (norm_x, norm_y)
}

/// ## Encode: 滑鼠按鍵事件標誌封裝 (MOUSEEVENTF Flags)
/// button: 0=左鍵, 1=右鍵, 2=中鍵；is_down: 按下或彈起
/// @outputs: 1
/// @domain: button [0, 2]
pub fn rpa_mouse_event_flags_encode(button: u8, is_down: bool) -> u32 {
    match button {
        0 => if is_down { 0x0002 } else { 0x0004 }, // MOUSEEVENTF_LEFTDOWN / UP
        1 => if is_down { 0x0008 } else { 0x0010 }, // MOUSEEVENTF_RIGHTDOWN / UP
        2 => if is_down { 0x0020 } else { 0x0040 }, // MOUSEEVENTF_MIDDLEDOWN / UP
        _ => 0,
    }
}

/// ## Encode: 組合熱鍵修飾鍵掩碼 (HotKey Modifier Mask)
/// 回傳 4-bit 掩碼：Bit 0=Shift, Bit 1=Ctrl, Bit 2=Alt, Bit 3=Win
/// @outputs: 1
pub fn rpa_keyboard_modifiers_mask(shift: bool, ctrl: bool, alt: bool, win: bool) -> u8 {
    let mut mask = 0u8;
    if shift { mask |= 0x01; }
    if ctrl { mask |= 0x02; }
    if alt { mask |= 0x04; }
    if win { mask |= 0x08; }
    mask
}

/// ## Calc: 螢幕 OCR 文字模糊容錯 Levenshtein 編輯距離 (Levenshtein Distance)
/// 零動態分配，以靜態定長一維切片完成動態規劃
/// @outputs: 1
pub fn rpa_ocr_text_levenshtein_distance(s1: &str, s2: &str) -> usize {
    if s1.is_empty() { return s2.len(); }
    if s2.is_empty() { return s1.len(); }

    let b1 = s1.as_bytes();
    let b2 = s2.as_bytes();
    let len2 = b2.len().min(128); // 限制長度防範爆炸
    let mut prev_row = [0usize; 129];
    let mut curr_row = [0usize; 129];

    for j in 0..=len2 { prev_row[j] = j; }

    for i in 1..=b1.len().min(128) {
        curr_row[0] = i;
        for j in 1..=len2 {
            let cost = if b1[i - 1] == b2[j - 1] { 0 } else { 1 };
            let del = prev_row[j] + 1;
            let ins = curr_row[j - 1] + 1;
            let sub = prev_row[j - 1] + cost;
            curr_row[j] = del.min(ins).min(sub);
        }
        prev_row[..=len2].copy_from_slice(&curr_row[..=len2]);
    }
    prev_row[len2]
}

/// ## Search: 螢幕像素流首個目標顏色搜尋 (Screen First Color Search)
/// 輸入 24-bit RGB 緩衝區 (長度 >= width * height * 3)，在給定色彩容差下回傳首個匹配之 (x, y)
/// @outputs: 1
/// @domain: tolerance [0, 255]
pub fn rpa_screen_find_first_color(
    pixels_rgb: &[u8],
    width: usize,
    height: usize,
    target_rgb: [u8; 3],
    tolerance: u8,
) -> Option<(usize, usize)> {
    if width == 0 || height == 0 || pixels_rgb.len() < width * height * 3 {
        return None;
    }
    let tol = tolerance as i32;

    for y in 0..height {
        let row_start = y * width * 3;
        for x in 0..width {
            let idx = row_start + x * 3;
            let dr = (pixels_rgb[idx] as i32 - target_rgb[0] as i32).abs();
            let dg = (pixels_rgb[idx + 1] as i32 - target_rgb[1] as i32).abs();
            let db = (pixels_rgb[idx + 2] as i32 - target_rgb[2] as i32).abs();
            if dr <= tol && dg <= tol && db <= tol {
                return Some((x, y));
            }
        }
    }
    None
}

/// ## Calc: 擬人按鍵延遲時間微抖動 (Timing Jitter Step)
/// 依據基準毫秒與隨機值在 [base * (1 - jitter%), base * (1 + jitter%)] 內擺盪
/// @outputs: 1
/// @domain: base_ms [1, 1000000], jitter_percent [0, 50]
pub fn rpa_key_delay_jitter_ms(base_ms: u64, jitter_percent: u8, random_factor: u32) -> u64 {
    if base_ms == 0 || jitter_percent == 0 {
        return base_ms;
    }
    let pct = jitter_percent.min(50) as u64;
    let delta_max = (base_ms * pct) / 100;
    if delta_max == 0 {
        return base_ms;
    }
    let range = 2 * delta_max + 1;
    let offset = (random_factor as u64) % range;
    (base_ms + offset).saturating_sub(delta_max)
}
