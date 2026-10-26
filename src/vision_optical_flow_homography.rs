//! # Vision Optical Flow & Homography Micro-Bricks (電腦視覺光流與透視單應性微積木)
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 Harris 結構張量步進、Lucas-Kanade 2x2 光流方程求解、3x3 單應矩陣射影與求逆、
//! 2D 仿射變換、次像素雙線性插值採樣、RANSAC 單應內點判定、SE(2) 剛體平面變換、
//! Graham Scan 外積轉向判定、2D 最小二乘直線擬合。

/// ## Map: Harris 局部梯度結構張量步進 (I_x^2, I_y^2, I_x*I_y)
/// @outputs: 3
/// @domain: ix [-32768, 32767]
/// @domain: iy [-32768, 32767]
pub fn vision_harris_structure_tensor_step(ix: i16, iy: i16) -> (i32, i32, i32) {
    let ix32 = ix as i32;
    let iy32 = iy as i32;
    (ix32 * ix32, iy32 * iy32, ix32 * iy32)
}

/// ## Solve: Lucas-Kanade 2x2 稀疏光流方程組求解 (Cramer's Rule)
/// [ sum_ix2  sum_ixiy ] [ u ] = - [ sum_ixt ]
/// [ sum_ixiy sum_iy2  ] [ v ]     [ sum_iyt ]
/// @outputs: 1
/// @domain: sum_ix2 [0.0, 1e12]
/// @domain: sum_iy2 [0.0, 1e12]
/// @domain: sum_ixiy [-1e12, 1e12]
/// @domain: sum_ixt [-1e12, 1e12]
/// @domain: sum_iyt [-1e12, 1e12]
pub fn vision_lucas_kanade_2x2_solve(
    sum_ix2: f32,
    sum_iy2: f32,
    sum_ixiy: f32,
    sum_ixt: f32,
    sum_iyt: f32,
) -> Option<(f32, f32)> {
    if !sum_ix2.is_finite() || !sum_iy2.is_finite() || !sum_ixiy.is_finite() ||
       !sum_ixt.is_finite() || !sum_iyt.is_finite() {
        return None;
    }
    let det = sum_ix2 * sum_iy2 - sum_ixiy * sum_ixiy;
    if det.abs() < 1e-7 {
        return None;
    }
    let inv_det = 1.0 / det;
    let u = (-sum_ixt * sum_iy2 + sum_iyt * sum_ixiy) * inv_det;
    let v = (-sum_ix2 * sum_iyt + sum_ixiy * sum_ixt) * inv_det;
    if u.is_finite() && v.is_finite() {
        Some((u, v))
    } else {
        None
    }
}

/// ## Map: 3x3 單應矩陣 (Homography) 齊次射影轉換
/// [x', y', w']^T = H * [x, y, 1]^T, (X, Y) = (x'/w', y'/w')
/// @outputs: 1
/// @domain: x [-1e6, 1e6]
/// @domain: y [-1e6, 1e6]
pub fn vision_homography_project_point_2d(h: &[f32; 9], x: f32, y: f32) -> Option<(f32, f32)> {
    if !x.is_finite() || !y.is_finite() {
        return None;
    }
    for &elem in h.iter() {
        if !elem.is_finite() { return None; }
    }
    let w_prime = h[6] * x + h[7] * y + h[8];
    if w_prime.abs() < 1e-8 || !w_prime.is_finite() {
        return None;
    }
    let inv_w = 1.0 / w_prime;
    let x_prime = (h[0] * x + h[1] * y + h[2]) * inv_w;
    let y_prime = (h[3] * x + h[4] * y + h[5]) * inv_w;
    if x_prime.is_finite() && y_prime.is_finite() {
        Some((x_prime, y_prime))
    } else {
        None
    }
}

/// ## Map: 2D 仿射變換前向映射 (Affine Forward Mapping)
/// [x', y']^T = [a0 a1; a3 a4] * [x, y]^T + [a2, a5]^T
/// @outputs: 2
/// @domain: x [-1e6, 1e6]
/// @domain: y [-1e6, 1e6]
pub fn vision_affine_transform_point_2d(a: &[f32; 6], x: f32, y: f32) -> (f32, f32) {
    if !x.is_finite() || !y.is_finite() {
        return (0.0, 0.0);
    }
    for &val in a.iter() {
        if !val.is_finite() { return (0.0, 0.0); }
    }
    let x_prime = a[0] * x + a[1] * y + a[2];
    let y_prime = a[3] * x + a[4] * y + a[5];
    (x_prime, y_prime)
}

/// ## Sample: 次像素雙線性插值採樣 (Bilinear Interpolation Sample)
/// @outputs: 1
/// @domain: x [0.0, 65535.0]
/// @domain: y [0.0, 65535.0]
pub fn vision_bilinear_sample_u8(img: &[u8], width: usize, height: usize, x: f32, y: f32) -> Option<u8> {
    if width < 2 || height < 2 || img.len() < width * height || !x.is_finite() || !y.is_finite() {
        return None;
    }
    let max_x = (width - 1) as f32;
    let max_y = (height - 1) as f32;
    if x < 0.0 || x > max_x || y < 0.0 || y > max_y {
        return None;
    }
    let x0 = x.floor() as usize;
    let y0 = y.floor() as usize;
    let x1 = (x0 + 1).min(width - 1);
    let y1 = (y0 + 1).min(height - 1);

    let fx = x - (x0 as f32);
    let fy = y - (y0 as f32);

    let p00 = img[y0 * width + x0] as f32;
    let p10 = img[y0 * width + x1] as f32;
    let p01 = img[y1 * width + x0] as f32;
    let p11 = img[y1 * width + x1] as f32;

    let interp = (1.0 - fx) * (1.0 - fy) * p00
               + fx * (1.0 - fy) * p10
               + (1.0 - fx) * fy * p01
               + fx * fy * p11;
    Some(interp.round().clamp(0.0, 255.0) as u8)
}

/// ## Solve: 3x3 矩陣求逆 (伴隨矩陣法 Adjugate Matrix Inverse)
/// @outputs: 1
pub fn vision_homography_3x3_inverse(h: &[f32; 9], out_inv: &mut [f32; 9]) -> bool {
    for &val in h.iter() {
        if !val.is_finite() { return false; }
    }
    let c00 = h[4] * h[8] - h[5] * h[7];
    let c01 = -(h[3] * h[8] - h[5] * h[6]);
    let c02 = h[3] * h[7] - h[4] * h[6];

    let det = h[0] * c00 + h[1] * c01 + h[2] * c02;
    if det.abs() < 1e-9 || !det.is_finite() {
        return false;
    }
    let inv_det = 1.0 / det;

    out_inv[0] = c00 * inv_det;
    out_inv[1] = (h[2] * h[7] - h[1] * h[8]) * inv_det;
    out_inv[2] = (h[1] * h[5] - h[2] * h[4]) * inv_det;

    out_inv[3] = c01 * inv_det;
    out_inv[4] = (h[0] * h[8] - h[2] * h[6]) * inv_det;
    out_inv[5] = (h[2] * h[3] - h[0] * h[5]) * inv_det;

    out_inv[6] = c02 * inv_det;
    out_inv[7] = (h[1] * h[6] - h[0] * h[7]) * inv_det;
    out_inv[8] = (h[0] * h[4] - h[1] * h[3]) * inv_det;
    true
}

/// ## Filter: RANSAC 單應性重投影誤差內點判定 (Inlier Check)
/// @outputs: 1
/// @domain: threshold [0.0, 1e6]
pub fn vision_ransac_homography_inlier_step(
    src_pt: [f32; 2],
    dst_pt: [f32; 2],
    h: &[f32; 9],
    threshold: f32,
) -> bool {
    if threshold < 0.0 || !threshold.is_finite() {
        return false;
    }
    if let Some((proj_x, proj_y)) = vision_homography_project_point_2d(h, src_pt[0], src_pt[1]) {
        let dx = proj_x - dst_pt[0];
        let dy = proj_y - dst_pt[1];
        let dist_sq = dx * dx + dy * dy;
        dist_sq <= threshold * threshold
    } else {
        false
    }
}

/// ## Map: SE(2) 平面剛體運動前向變換 (Rigid 2D Transform: Translation & Rotation)
/// @outputs: 2
/// @domain: tx [-1e6, 1e6]
/// @domain: ty [-1e6, 1e6]
/// @domain: theta_rad [-1000.0, 1000.0]
/// @domain: x [-1e6, 1e6]
/// @domain: y [-1e6, 1e6]
pub fn vision_se2_rigid_transform_point_2d(
    tx: f32,
    ty: f32,
    theta_rad: f32,
    x: f32,
    y: f32,
) -> (f32, f32) {
    if !tx.is_finite() || !ty.is_finite() || !theta_rad.is_finite() || !x.is_finite() || !y.is_finite() {
        return (0.0, 0.0);
    }
    let c = fast_cos_approx(theta_rad);
    let s = fast_sin_approx(theta_rad);
    let x_prime = c * x - s * y + tx;
    let y_prime = s * x + c * y + ty;
    (x_prime, y_prime)
}

/// ## Math: Graham Scan 凸包外積轉向判定 (Cross Product Orientation)
/// 傳回值 > 0 為逆時針轉折 (Left turn)，< 0 為順時針轉折 (Right turn)，= 0 為共線
/// @outputs: 1
pub fn vision_graham_scan_cross_product(p1: [f32; 2], p2: [f32; 2], p3: [f32; 2]) -> f32 {
    if !p1[0].is_finite() || !p1[1].is_finite() ||
       !p2[0].is_finite() || !p2[1].is_finite() ||
       !p3[0].is_finite() || !p3[1].is_finite() {
        return 0.0;
    }
    (p2[0] - p1[0]) * (p3[1] - p1[1]) - (p2[1] - p1[1]) * (p3[0] - p1[0])
}

/// ## Fit: 2D 點集最小二乘直線擬合 (y = mx + c)
/// @outputs: 1
pub fn vision_least_squares_line_2d(points: &[[f32; 2]]) -> Option<(f32, f32)> {
    if points.len() < 2 {
        return None;
    }
    let n = points.len() as f32;
    let mut sum_x = 0.0f32;
    let mut sum_y = 0.0f32;
    for pt in points {
        if !pt[0].is_finite() || !pt[1].is_finite() { return None; }
        sum_x += pt[0];
        sum_y += pt[1];
    }
    let mean_x = sum_x / n;
    let mean_y = sum_y / n;

    let mut ss_xx = 0.0f32;
    let mut ss_xy = 0.0f32;
    for pt in points {
        let dx = pt[0] - mean_x;
        let dy = pt[1] - mean_y;
        ss_xx += dx * dx;
        ss_xy += dx * dy;
    }
    if ss_xx < 1e-8 {
        return None;
    }
    let m = ss_xy / ss_xx;
    let c = mean_y - m * mean_x;
    Some((m, c))
}

/// 內部高階正弦多項式近似 (7 階麥克勞林展開帶週期折疊)
/// @outputs: 1
fn fast_sin_approx(rad: f32) -> f32 {
    let pi = 3.14159265;
    let two_pi = 6.2831853;
    let mut x = rad % two_pi;
    if x > pi { x -= two_pi; } else if x < -pi { x += two_pi; }
    let x2 = x * x;
    let x3 = x * x2;
    let x5 = x3 * x2;
    let x7 = x5 * x2;
    x - x3 / 6.0 + x5 / 120.0 - x7 / 5040.0
}

/// 內部高階餘弦多項式近似 (6 階麥克勞林展開帶週期折疊)
/// @outputs: 1
fn fast_cos_approx(rad: f32) -> f32 {
    let pi = 3.14159265;
    let two_pi = 6.2831853;
    let mut x = rad % two_pi;
    if x > pi { x -= two_pi; } else if x < -pi { x += two_pi; }
    let x2 = x * x;
    let x4 = x2 * x2;
    let x6 = x4 * x2;
    1.0 - x2 / 2.0 + x4 / 24.0 - x6 / 720.0
}
