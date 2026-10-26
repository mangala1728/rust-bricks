// ## Map
// Role: core/vision_feature — 手機相機邊緣影像特徵萃取：Otsu 自適應雙峰二值化、Sobel 3x3 邊緣梯度與積分圖 O(1) 區域和計算微積木。
// Inputs: 灰階影像切片、寬高、座標 (x, y)、積分圖切片、矩形區域邊界。
// Outputs: Otsu 最佳門檻值 (u8)、Sobel 梯度強度 (u16)、積分圖向量 (Vec<u32>)、區域像素總和 (u32)。
// Invariants: 零外部依賴、無遞迴、零全域可變狀態、寬高有界守衛（最大 8192x8192）、控制流巢狀深度 <= 2、純標準庫 100% 零依賴。

const MAX_IMAGE_DIM: usize = 8192;

/// @outputs: 1
pub fn otsu_threshold(gray: &[u8]) -> u8 {
    if gray.is_empty() {
        return 128;
    }
    let mut hist = [0u32; 256];
    for &val in gray {
        hist[val as usize] += 1;
    }
    let total_pixels = gray.len() as f64;
    let mut sum_total = 0.0f64;
    for (i, &count) in hist.iter().enumerate() {
        sum_total += (i as f64) * (count as f64);
    }

    let mut weight_bg = 0.0f64;
    let mut sum_bg = 0.0f64;
    let mut max_variance = 0.0f64;
    let mut best_thresh = 0u8;

    for (t, &count) in hist.iter().enumerate() {
        weight_bg += count as f64;
        if weight_bg == 0.0 {
            continue;
        }
        let weight_fg = total_pixels - weight_bg;
        if weight_fg <= 0.0 {
            break;
        }
        sum_bg += (t as f64) * (count as f64);
        let mean_bg = sum_bg / weight_bg;
        let mean_fg = (sum_total - sum_bg) / weight_fg;
        let mean_diff = mean_bg - mean_fg;
        let between_variance = weight_bg * weight_fg * mean_diff * mean_diff;
        if between_variance > max_variance {
            max_variance = between_variance;
            best_thresh = t as u8;
        }
    }
    best_thresh
}

/// @outputs: 1
/// @domain: width [3..8192]
/// @domain: height [3..8192]
/// @domain: x [1..8191]
/// @domain: y [1..8191]
pub fn sobel_gradient_3x3(
    gray: &[u8],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
) -> u16 {
    if width < 3 || width > MAX_IMAGE_DIM || height < 3 || height > MAX_IMAGE_DIM {
        return 0;
    }
    if x < 1 || x >= width - 1 || y < 1 || y >= height - 1 {
        return 0;
    }
    if gray.len() < width.saturating_mul(height) {
        return 0;
    }
    let r0 = (y - 1) * width;
    let r1 = y * width;
    let r2 = (y + 1) * width;

    let p00 = gray[r0 + x - 1] as i32;
    let p01 = gray[r0 + x] as i32;
    let p02 = gray[r0 + x + 1] as i32;
    let p10 = gray[r1 + x - 1] as i32;
    let p12 = gray[r1 + x + 1] as i32;
    let p20 = gray[r2 + x - 1] as i32;
    let p21 = gray[r2 + x] as i32;
    let p22 = gray[r2 + x + 1] as i32;

    let gx = (p02 + 2 * p12 + p22) - (p00 + 2 * p10 + p20);
    let gy = (p20 + 2 * p21 + p22) - (p00 + 2 * p01 + p02);
    let mag = gx.abs() + gy.abs();
    if mag > 65535 { 65535 } else { mag as u16 }
}

/// @outputs: 2
/// @domain: width [1..8192]
/// @domain: height [1..8192]
pub fn integral_image_compute(
    gray: &[u8],
    width: usize,
    height: usize,
) -> Result<Vec<u32>, &'static str> {
    if width == 0 || width > MAX_IMAGE_DIM || height == 0 || height > MAX_IMAGE_DIM {
        return Err("影像寬高超出有界範圍 [1..8192]");
    }
    if gray.len() < width.saturating_mul(height) {
        return Err("影像緩衝區長度不足");
    }
    let stride = width + 1;
    let total_cells = stride.saturating_mul(height + 1);
    let mut integral = vec![0u32; total_cells];

    for y in 0..height {
        let mut row_sum = 0u32;
        let in_row = y * width;
        let out_row = (y + 1) * stride;
        let prev_out_row = y * stride;
        for x in 0..width {
            row_sum += gray[in_row + x] as u32;
            integral[out_row + x + 1] = integral[prev_out_row + x + 1] + row_sum;
        }
    }
    Ok(integral)
}

/// @outputs: 1
/// @domain: stride [2..8193]
pub fn integral_box_sum(
    integral: &[u32],
    stride: usize,
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
) -> u32 {
    if stride < 2 || x0 >= x1 || y0 >= y1 {
        return 0;
    }
    let idx_br = y1 * stride + x1;
    let idx_bl = y1 * stride + x0;
    let idx_tr = y0 * stride + x1;
    let idx_tl = y0 * stride + x0;
    if idx_br >= integral.len() {
        return 0;
    }
    let br = integral[idx_br];
    let bl = integral[idx_bl];
    let tr = integral[idx_tr];
    let tl = integral[idx_tl];
    // br - bl - tr 合法地可為負，須以模 2^32 運算後再加 tl，不可 saturating 截斷 (INC-20261026-03)
    br.wrapping_sub(bl).wrapping_sub(tr).wrapping_add(tl)
}
