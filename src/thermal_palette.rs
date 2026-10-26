// ## Map
// Role: core/thermal_palette — 熱成像 (FLIR / MLX90640) 攝氏度陣列轉 Ironbow/Rainbow 偽彩色與統計分析微積木。
// Inputs: 攝氏溫度值、顯示溫標上下限 (min..max)、溫度浮點數矩陣。
// Outputs: 偽彩色 RGB24 像素 (r, g, b)、溫度矩陣統計元組 Option<(min, max, avg)>。
// Invariants: 零外部依賴、無遞迴、零全域可變狀態、浮點數防 NaN 守衛、零 Heap 分配、控制流巢狀深度 <= 2。

/// @outputs: 1
/// @domain: min_temp [-100.0..2000.0]
/// @domain: max_temp [-100.0..2000.0]
pub fn thermal_ironbow_rgb(temp: f32, min_temp: f32, max_temp: f32) -> (u8, u8, u8) {
    if min_temp >= max_temp || min_temp.is_nan() || max_temp.is_nan() || temp.is_nan() {
        return (0, 0, 0);
    }
    let range = max_temp - min_temp;
    let t = ((temp - min_temp) / range).clamp(0.0, 1.0);

    if t < 0.25 {
        let seg = t / 0.25;
        let r = (75.0 * seg) as u8;
        let b = (130.0 * seg) as u8;
        (r, 0, b)
    } else if t < 0.50 {
        let seg = (t - 0.25) / 0.25;
        let r = (75.0 + (220.0 - 75.0) * seg) as u8;
        let g = (20.0 * seg) as u8;
        let b = (130.0 + (60.0 - 130.0) * seg) as u8;
        (r, g, b)
    } else if t < 0.75 {
        let seg = (t - 0.50) / 0.25;
        let r = (220.0 + (255.0 - 220.0) * seg) as u8;
        let g = (20.0 + (140.0 - 20.0) * seg) as u8;
        let b = (60.0 * (1.0 - seg)) as u8;
        (r, g, b)
    } else {
        let seg = (t - 0.75) / 0.25;
        let g = (140.0 + (255.0 - 140.0) * seg) as u8;
        let b = (255.0 * seg) as u8;
        (255, g, b)
    }
}

/// @outputs: 1
/// @domain: min_temp [-100.0..2000.0]
/// @domain: max_temp [-100.0..2000.0]
pub fn thermal_rainbow_rgb(temp: f32, min_temp: f32, max_temp: f32) -> (u8, u8, u8) {
    if min_temp >= max_temp || min_temp.is_nan() || max_temp.is_nan() || temp.is_nan() {
        return (0, 0, 255);
    }
    let range = max_temp - min_temp;
    let t = ((temp - min_temp) / range).clamp(0.0, 1.0);
    // 藍(0,0,255) -> 青(0,255,255) -> 綠(0,255,0) -> 黃(255,255,0) -> 紅(255,0,0)
    let val = (t * 4.0).clamp(0.0, 4.0);
    let seg = val.fract();
    let idx = val.floor() as u8;
    match idx {
        0 => (0, (seg * 255.0) as u8, 255),
        1 => (0, 255, ((1.0 - seg) * 255.0) as u8),
        2 => ((seg * 255.0) as u8, 255, 0),
        3 => (255, ((1.0 - seg) * 255.0) as u8, 0),
        _ => (255, 0, 0),
    }
}

/// @outputs: 2
/// @domain: matrix_len [1..65536]
pub fn thermal_matrix_stats(matrix: &[f32]) -> Option<(f32, f32, f32)> {
    if matrix.is_empty() || matrix.len() > 65536 {
        return None;
    }
    let mut min = f32::INFINITY;
    let mut max = f32::NEG_INFINITY;
    let mut sum = 0.0f64;
    let mut valid_count = 0usize;

    for &val in matrix {
        if val.is_nan() {
            continue;
        }
        if val < min { min = val; }
        if val > max { max = val; }
        sum += val as f64;
        valid_count += 1;
    }
    if valid_count == 0 {
        return None;
    }
    let avg = (sum / (valid_count as f64)) as f32;
    Some((min, max, avg))
}
