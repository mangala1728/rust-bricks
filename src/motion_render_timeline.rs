// ## Map
// Role: motion/motion_render_timeline — 零依賴 Remotion 影音幀時序、CSS Cubic-Bezier 曲線求值、SVG 描邊動畫與 FFMPEG 剪輯微積木。
// Inputs: 視訊影格編號 (Frame)、影格率分母分子 (FPS)、CSS 貝茲控制點 (x1, y1, x2, y2)、進度百分比 (0.0..1.0)、毫秒時間戳。
// Outputs: 毫秒時間、SMPTE 時間碼字串 (HH:MM:SS:FF)、Cubic-Bezier 緩動值 (y)、SVG dashoffset、FFMPEG 剪輯時間字串。
// Invariants: 零外部依賴、純 Rust 實現、除零剛性防護、牛頓法有限迭代收斂、單一函式 <= 60 行。

/// @outputs: 1
/// @domain: fps_den [1..1000]
pub fn remotion_frame_to_ms(frame: u64, fps_num: u32, fps_den: u32) -> u64 {
    if fps_num == 0 || fps_den == 0 || fps_num > 1_000_000 || fps_den > 1000 {
        return 0;
    }
    // ms = (frame * 1000 * fps_den) / fps_num
    let num = (frame as u128) * 1000 * (fps_den as u128);
    let den = fps_num as u128;
    (num / den) as u64
}

/// @outputs: 1
/// @domain: fps_den [1..1000]
pub fn remotion_ms_to_frame(ms: u64, fps_num: u32, fps_den: u32) -> u64 {
    if fps_num == 0 || fps_den == 0 || fps_num > 1_000_000 || fps_den > 1000 {
        return 0;
    }
    // frame = (ms * fps_num) / (1000 * fps_den)
    let num = (ms as u128) * (fps_num as u128);
    let den = 1000u128 * (fps_den as u128);
    (num / den) as u64
}

/// @outputs: 1
/// @domain: out_buf.len() [11..11]
pub fn smpte_timecode_format(frame: u64, fps_int: u32, out_buf: &mut [u8; 11]) -> bool {
    if fps_int == 0 || fps_int > 240 {
        return false;
    }
    let fps = fps_int as u64;
    let ff = frame % fps;
    let total_secs = frame / fps;
    let ss = total_secs % 60;
    let total_mins = total_secs / 60;
    let mm = total_mins % 60;
    let hh = (total_mins / 60) % 100;

    out_buf[0] = b'0' + ((hh / 10) as u8);
    out_buf[1] = b'0' + ((hh % 10) as u8);
    out_buf[2] = b':';
    out_buf[3] = b'0' + ((mm / 10) as u8);
    out_buf[4] = b'0' + ((mm % 10) as u8);
    out_buf[5] = b':';
    out_buf[6] = b'0' + ((ss / 10) as u8);
    out_buf[7] = b'0' + ((ss % 10) as u8);
    out_buf[8] = b':';
    out_buf[9] = b'0' + ((ff / 10) as u8);
    out_buf[10] = b'0' + ((ff % 10) as u8);
    true
}

/// @outputs: 1
/// @domain: x [0.0..1.0]
pub fn css_cubic_bezier_solve(x1: f64, y1: f64, x2: f64, y2: f64, x: f64) -> f64 {
    if x <= 0.0 || !x.is_finite() {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    // B_x(t) = 3*(1-t)^2*t*x1 + 3*(1-t)*t^2*x2 + t^3
    // 牛頓法求解 B_x(t) = x
    let mut t = x;
    for _ in 0..8 {
        let one_minus_t = 1.0 - t;
        let bx = 3.0 * one_minus_t * one_minus_t * t * x1 + 3.0 * one_minus_t * t * t * x2 + t * t * t;
        let diff = bx - x;
        if diff.abs() < 1e-5 {
            break;
        }
        // 導數: dB_x/dt
        let dbx = 3.0 * one_minus_t * one_minus_t * x1 + 6.0 * one_minus_t * t * (x2 - x1) + 3.0 * t * t * (1.0 - x2);
        if dbx.abs() < 1e-6 {
            break;
        }
        t -= diff / dbx;
        t = t.clamp(0.0, 1.0);
    }
    // 求解得到 t，代入 B_y(t)
    let one_minus_t = 1.0 - t;
    let by = 3.0 * one_minus_t * one_minus_t * t * y1 + 3.0 * one_minus_t * t * t * y2 + t * t * t;
    by.clamp(0.0, 1.0)
}

/// @outputs: 1
/// @domain: progress [0.0..1.0]
pub fn svg_stroke_dashoffset(total_length: f32, progress: f32) -> f32 {
    if total_length <= 0.0 || !total_length.is_finite() || !progress.is_finite() {
        return 0.0;
    }
    let p = progress.clamp(0.0, 1.0);
    total_length * (1.0 - p)
}

/// @outputs: 2
/// @domain: out_buf.len() [12..16]
pub fn ffmpeg_time_seek_str(total_ms: u64, out_buf: &mut [u8; 12]) -> bool {
    let mmm = (total_ms % 1000) as u32;
    let total_secs = total_ms / 1000;
    let ss = (total_secs % 60) as u32;
    let total_mins = total_secs / 60;
    let mm = (total_mins % 60) as u32;
    let hh = ((total_mins / 60) % 100) as u32;

    out_buf[0] = b'0' + ((hh / 10) as u8);
    out_buf[1] = b'0' + ((hh % 10) as u8);
    out_buf[2] = b':';
    out_buf[3] = b'0' + ((mm / 10) as u8);
    out_buf[4] = b'0' + ((mm % 10) as u8);
    out_buf[5] = b':';
    out_buf[6] = b'0' + ((ss / 10) as u8);
    out_buf[7] = b'0' + ((ss % 10) as u8);
    out_buf[8] = b'.';
    out_buf[9] = b'0' + ((mmm / 100) as u8);
    out_buf[10] = b'0' + (((mmm / 10) % 10) as u8);
    out_buf[11] = b'0' + ((mmm % 10) as u8);
    true
}
