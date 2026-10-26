//! # Vision Morphology & FAST Corner Micro-Bricks (電腦視覺形態學與角點微積木)
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 3x3 二值膨脹/侵蝕、形態學梯度邊緣提取、FAST-9 圓環角點檢測、
//! 霍夫直線轉換累加索引、水平/垂直投影積分、IoU 包圍盒重疊度、NCC 樣板比對、RGB 轉 HSV 與色彩遮罩。

/// ## Map: 3x3 二值圖像形態學膨脹 (Binary Dilation: 鄰域有 255 則輸出 255)
/// @outputs: 1
pub fn morphology_binary_dilation_3x3(src: &[u8], width: usize, height: usize, out: &mut [u8]) -> bool {
    if width < 3 || height < 3 || src.len() < width * height || out.len() < width * height {
        return false;
    }
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            if y == 0 || y == height - 1 || x == 0 || x == width - 1 {
                out[idx] = src[idx];
                continue;
            }
            let mut hit = false;
            for dy in 0..3 {
                for dx in 0..3 {
                    let neighbor_idx = (y + dy - 1) * width + (x + dx - 1);
                    if src[neighbor_idx] == 255 {
                        hit = true;
                        break;
                    }
                }
                if hit { break; }
            }
            out[idx] = if hit { 255 } else { 0 };
        }
    }
    true
}

/// ## Map: 3x3 二值圖像形態學侵蝕 (Binary Erosion: 鄰域全為 255 才輸出 255)
/// @outputs: 1
pub fn morphology_binary_erosion_3x3(src: &[u8], width: usize, height: usize, out: &mut [u8]) -> bool {
    if width < 3 || height < 3 || src.len() < width * height || out.len() < width * height {
        return false;
    }
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            if y == 0 || y == height - 1 || x == 0 || x == width - 1 {
                out[idx] = src[idx];
                continue;
            }
            let mut all_set = true;
            for dy in 0..3 {
                for dx in 0..3 {
                    let neighbor_idx = (y + dy - 1) * width + (x + dx - 1);
                    if src[neighbor_idx] != 255 {
                        all_set = false;
                        break;
                    }
                }
                if !all_set { break; }
            }
            out[idx] = if all_set { 255 } else { 0 };
        }
    }
    true
}

/// ## Map: 形態學梯度邊緣提取 (Gradient = Dilation - Erosion)
/// @outputs: 1
pub fn morphology_gradient_edge_3x3(src: &[u8], width: usize, height: usize, out: &mut [u8]) -> bool {
    if width < 3 || height < 3 || src.len() < width * height || out.len() < width * height {
        return false;
    }
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            if y == 0 || y == height - 1 || x == 0 || x == width - 1 {
                out[idx] = 0;
                continue;
            }
            let mut min_val = 255u8;
            let mut max_val = 0u8;
            for dy in 0..3 {
                for dx in 0..3 {
                    let v = src[(y + dy - 1) * width + (x + dx - 1)];
                    if v < min_val { min_val = v; }
                    if v > max_val { max_val = v; }
                }
            }
            out[idx] = max_val.saturating_sub(min_val);
        }
    }
    true
}

/// ## Map: FAST-9 圓環特徵角點判定 (半徑 3 圓周 16 點中連續 9 點亮度超過門檻)
/// @outputs: 1
pub fn fast9_corner_detect_point(img: &[u8], width: usize, height: usize, cx: usize, cy: usize, threshold: u8) -> bool {
    if cx < 3 || cy < 3 || cx + 3 >= width || cy + 3 >= height || img.len() < width * height {
        return false;
    }
    let center = img[cy * width + cx] as i16;
    let t = threshold as i16;
    // 圓周 16 個點相對 (dx, dy)
    let circle_offsets: [(i32, i32); 16] = [
        (0, -3), (1, -3), (2, -2), (3, -1), (3, 0), (3, 1), (2, 2), (1, 3),
        (0, 3), (-1, 3), (-2, 2), (-3, 1), (-3, 0), (-3, -1), (-2, -2), (-1, -3),
    ];
    let mut flags = [0i8; 16]; // 1: brighter, -1: darker, 0: similar
    for (i, &(dx, dy)) in circle_offsets.iter().enumerate() {
        let px = (cx as i32 + dx) as usize;
        let py = (cy as i32 + dy) as usize;
        let val = img[py * width + px] as i16;
        if val > center + t {
            flags[i] = 1;
        } else if val < center - t {
            flags[i] = -1;
        }
    }
    // 檢查連續 9 個點同號 (環形長度 16 + 8 = 24)
    for start in 0..16 {
        let mut count_bright = 0;
        let mut count_dark = 0;
        for step in 0..9 {
            let f = flags[(start + step) % 16];
            if f == 1 { count_bright += 1; }
            if f == -1 { count_dark += 1; }
        }
        if count_bright == 9 || count_dark == 9 {
            return true;
        }
    }
    false
}

/// ## Map: 霍夫直線轉換累加器索引換算 (rho = x * cos(theta) + y * sin(theta))
/// @outputs: 1
pub fn vision_hough_line_accumulate_point(
    x: usize,
    y: usize,
    theta_deg: usize,
    max_rho: usize,
) -> Option<usize> {
    if theta_deg >= 180 || max_rho == 0 {
        return None;
    }
    // 零依賴正弦/餘弦定點查表 (0..180 度，放大 1000 倍)
    let rad = (theta_deg as f32) * 0.0174532925;
    let cos_t = fast_cos_approx(rad);
    let sin_t = fast_sin_approx(rad);
    let rho = (x as f32) * cos_t + (y as f32) * sin_t;
    let rho_idx = (rho + (max_rho as f32)) as isize;
    if rho_idx < 0 || (rho_idx as usize) >= max_rho * 2 {
        None
    } else {
        Some(rho_idx as usize)
    }
}

/// ## Map: 圖像水平/垂直投影積分直方圖
/// @outputs: 1
pub fn vision_projection_integral_profile(
    src: &[u8],
    width: usize,
    height: usize,
    vertical_proj: bool,
    out_profile: &mut [u32],
) -> bool {
    if width == 0 || height == 0 || src.len() < width * height {
        return false;
    }
    let target_len = if vertical_proj { width } else { height };
    if out_profile.len() < target_len {
        return false;
    }
    for item in out_profile.iter_mut().take(target_len) {
        *item = 0;
    }
    for y in 0..height {
        for x in 0..width {
            let val = src[y * width + x] as u32;
            if vertical_proj {
                out_profile[x] += val;
            } else {
                out_profile[y] += val;
            }
        }
    }
    true
}

/// ## Map: 目標檢測包圍盒 IoU (Intersection over Union: [x, y, w, h])
/// @outputs: 1
pub fn vision_bbox_intersection_over_union(bbox_a: [u32; 4], bbox_b: [u32; 4]) -> f32 {
    let ax1 = bbox_a[0];
    let ay1 = bbox_a[1];
    let ax2 = ax1.saturating_add(bbox_a[2]);
    let ay2 = ay1.saturating_add(bbox_a[3]);

    let bx1 = bbox_b[0];
    let by1 = bbox_b[1];
    let bx2 = bx1.saturating_add(bbox_b[2]);
    let by2 = by1.saturating_add(bbox_b[3]);

    let inter_x1 = ax1.max(bx1);
    let inter_y1 = ay1.max(by1);
    let inter_x2 = ax2.min(bx2);
    let inter_y2 = ay2.min(by2);

    let inter_w = inter_x2.saturating_sub(inter_x1) as f32;
    let inter_h = inter_y2.saturating_sub(inter_y1) as f32;
    let inter_area = inter_w * inter_h;

    let area_a = (bbox_a[2] * bbox_a[3]) as f32;
    let area_b = (bbox_b[2] * bbox_b[3]) as f32;
    let union_area = area_a + area_b - inter_area;

    if union_area <= 0.0 {
        0.0
    } else {
        inter_area / union_area
    }
}

/// ## Map: 標準化互相關 (NCC, Normalized Cross-Correlation) 樣板比對
/// @outputs: 1
pub fn vision_normalized_cross_correlation_step(patch_a: &[u8], patch_b: &[u8]) -> Option<f32> {
    if patch_a.is_empty() || patch_a.len() != patch_b.len() {
        return None;
    }
    let n = patch_a.len() as f32;
    let sum_a: f32 = patch_a.iter().map(|&v| v as f32).sum();
    let sum_b: f32 = patch_b.iter().map(|&v| v as f32).sum();
    let mean_a = sum_a / n;
    let mean_b = sum_b / n;

    let mut num = 0.0f32;
    let mut den_a = 0.0f32;
    let mut den_b = 0.0f32;
    for (&va, &vb) in patch_a.iter().zip(patch_b.iter()) {
        let da = (va as f32) - mean_a;
        let db = (vb as f32) - mean_b;
        num += da * db;
        den_a += da * da;
        den_b += db * db;
    }
    let den = (den_a * den_b).sqrt();
    if den < 1e-6 {
        None
    } else {
        Some(num / den)
    }
}

/// ## Map: RGB 轉 HSV 色彩空間 (H: 0..360, S: 0..255, V: 0..255)
/// @outputs: 1
pub fn vision_rgb_to_hsv_step(r: u8, g: u8, b: u8) -> (u16, u8, u8) {
    let rf = r as f32;
    let gf = g as f32;
    let bf = b as f32;
    let max = rf.max(gf.max(bf));
    let min = rf.min(gf.min(bf));
    let delta = max - min;
    let v = max as u8;
    if max <= 0.0 || delta <= 0.0 {
        return (0, 0, v);
    }
    let s = ((delta / max) * 255.0) as u8;
    let mut h = if rf >= max {
        ((gf - bf) / delta) * 60.0
    } else if gf >= max {
        (2.0 + (bf - rf) / delta) * 60.0
    } else {
        (4.0 + (rf - gf) / delta) * 60.0
    };
    if h < 0.0 {
        h += 360.0;
    }
    (h as u16, s, v)
}

/// ## Map: HSV 色彩範圍遮罩判定 (支援色相跨 0 度環形判斷)
/// @domain: h [0, 360]
/// @domain: h_min [0, 360]
/// @domain: h_max [0, 360]
/// @outputs: 1
pub fn vision_color_hsv_in_range(
    h: u16,
    s: u8,
    v: u8,
    h_min: u16,
    h_max: u16,
    s_min: u8,
    v_min: u8,
) -> bool {
    if h > 360 || h_min > 360 || h_max > 360 {
        return false;
    }
    if s < s_min || v < v_min {
        return false;
    }
    if h_min <= h_max {
        h >= h_min && h <= h_max
    } else {
        // 跨越 360/0 邊界（例如紅色: 350..10）
        h >= h_min || h <= h_max
    }
}

/// 內部五階正弦多項式近似
/// @outputs: 1
fn fast_sin_approx(rad: f32) -> f32 {
    let x = rad;
    let x3 = x * x * x;
    let x5 = x3 * x * x;
    x - x3 / 6.0 + x5 / 120.0
}

/// 內部四階餘弦多項式近似
/// @outputs: 1
fn fast_cos_approx(rad: f32) -> f32 {
    let x = rad;
    let x2 = x * x;
    let x4 = x2 * x2;
    1.0 - x2 / 2.0 + x4 / 24.0
}
