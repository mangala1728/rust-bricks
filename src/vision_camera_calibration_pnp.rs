//! # 光學相機幾何、針孔畸變模型與外參位姿求解 (Camera Calibration, Lens Distortion & PnP Geometry)
//!
//! 提供針孔相機透視投影、Brown-Conrady 徑向與切向鏡頭畸變、
//! 反向去畸變迭代步進、對極幾何 (Epipolar) 基本矩陣約束與 Sampson 距離、
//! 本質矩陣 E = [t]x R、Hartley 坐標歸一化與重投影均方根誤差 (RMSE) 微積木。
//! 零外部依賴，符合 NASA JPL 規範與合約 09。

/// ## Map: 針孔相機 3D 空間點前向透視投影 (Pinhole Camera Projection)
/// 計算像素坐標 u = fx * (X / Z) + cx, v = fy * (Y / Z) + cy
/// @outputs: 1
/// @domain: p_cam[2] > 0.001 && fx > 0.0 && fy > 0.0
pub fn camera_pinhole_project_point(
    p_cam: [f32; 3],
    fx: f32,
    fy: f32,
    cx: f32,
    cy: f32,
) -> Option<[f32; 2]> {
    let z = p_cam[2];
    if !z.is_finite() || z <= 0.001 || !fx.is_finite() || !fy.is_finite() || fx <= 0.0 || fy <= 0.0 {
        return None;
    }
    if !cx.is_finite() || !cy.is_finite() || !p_cam[0].is_finite() || !p_cam[1].is_finite() {
        return None;
    }
    let inv_z = 1.0 / z;
    let u = fx * (p_cam[0] * inv_z) + cx;
    let v = fy * (p_cam[1] * inv_z) + cy;
    Some([u, v])
}

/// ## Calc: Brown-Conrady 徑向畸變多項式映射 (Radial Lens Distortion)
/// 計算 (x', y') = (x, y) * (1 + k1*r^2 + k2*r^4 + k3*r^6)
/// @outputs: 1
/// @domain: x_norm, y_norm 皆為有限實數
pub fn camera_radial_distortion_apply(
    x_norm: f32,
    y_norm: f32,
    k1: f32,
    k2: f32,
    k3: f32,
) -> [f32; 2] {
    if !x_norm.is_finite() || !y_norm.is_finite() || !k1.is_finite() || !k2.is_finite() || !k3.is_finite() {
        return [x_norm, y_norm];
    }
    let r2 = x_norm * x_norm + y_norm * y_norm;
    let r4 = r2 * r2;
    let r6 = r4 * r2;
    let radial = 1.0 + k1 * r2 + k2 * r4 + k3 * r6;
    [x_norm * radial, y_norm * radial]
}

/// ## Calc: Brown-Conrady 切向薄透鏡畸變映射 (Tangential Lens Distortion)
/// @outputs: 1
/// @domain: x_norm, y_norm 皆為有限實數
pub fn camera_tangential_distortion_apply(
    x_norm: f32,
    y_norm: f32,
    p1: f32,
    p2: f32,
) -> [f32; 2] {
    if !x_norm.is_finite() || !y_norm.is_finite() || !p1.is_finite() || !p2.is_finite() {
        return [0.0, 0.0];
    }
    let r2 = x_norm * x_norm + y_norm * y_norm;
    let dx = 2.0 * p1 * x_norm * y_norm + p2 * (r2 + 2.0 * x_norm * x_norm);
    let dy = p1 * (r2 + 2.0 * y_norm * y_norm) + 2.0 * p2 * x_norm * y_norm;
    [dx, dy]
}

/// ## Calc: 徑向畸變單步定點反向去畸變 (Radial Undistort Fixed-Point Step)
/// @outputs: 1
/// @domain: x_dist, y_dist 皆為有限實數
pub fn camera_radial_undistort_step(
    x_dist: f32,
    y_dist: f32,
    k1: f32,
    k2: f32,
) -> [f32; 2] {
    if !x_dist.is_finite() || !y_dist.is_finite() || !k1.is_finite() || !k2.is_finite() {
        return [x_dist, y_dist];
    }
    let mut x = x_dist;
    let mut y = y_dist;
    // 3 次定點迭代收斂
    for _ in 0..3 {
        let r2 = x * x + y * y;
        let scale = 1.0 / (1.0 + k1 * r2 + k2 * r2 * r2);
        x = x_dist * scale;
        y = y_dist * scale;
    }
    [x, y]
}

/// ## Calc: 基本矩陣對極線方程式計算 (Epipolar Line from Fundamental Matrix)
/// 計算 l = F * [x, y, 1]^T
/// @outputs: 1
/// @domain: pt 有限實數
pub fn epipolar_line_from_fundamental(
    f_mat: [[f32; 3]; 3],
    pt: [f32; 2],
) -> Option<[f32; 3]> {
    if !pt[0].is_finite() || !pt[1].is_finite() {
        return None;
    }
    let l_a = f_mat[0][0] * pt[0] + f_mat[0][1] * pt[1] + f_mat[0][2];
    let l_b = f_mat[1][0] * pt[0] + f_mat[1][1] * pt[1] + f_mat[1][2];
    let l_c = f_mat[2][0] * pt[0] + f_mat[2][1] * pt[1] + f_mat[2][2];
    let norm = (l_a * l_a + l_b * l_b).sqrt();
    if norm <= 1e-12 {
        None
    } else {
        Some([l_a / norm, l_b / norm, l_c / norm])
    }
}

/// ## Calc: 對極幾何 Sampson 一階幾何幾何誤差 (Sampson Epipolar Error)
/// @outputs: 1
/// @domain: p1, p2 有限實數
pub fn epipolar_sampson_distance_error(
    f_mat: [[f32; 3]; 3],
    p1: [f32; 2],
    p2: [f32; 2],
) -> Option<f32> {
    if !p1[0].is_finite() || !p1[1].is_finite() || !p2[0].is_finite() || !p2[1].is_finite() {
        return None;
    }
    // F * x1
    let fx1 = [
        f_mat[0][0] * p1[0] + f_mat[0][1] * p1[1] + f_mat[0][2],
        f_mat[1][0] * p1[0] + f_mat[1][1] * p1[1] + f_mat[1][2],
        f_mat[2][0] * p1[0] + f_mat[2][1] * p1[1] + f_mat[2][2],
    ];
    // x2^T * F * x1
    let num = p2[0] * fx1[0] + p2[1] * fx1[1] + fx1[2];

    // F^T * x2
    let ftx2_0 = f_mat[0][0] * p2[0] + f_mat[1][0] * p2[1] + f_mat[2][0];
    let ftx2_1 = f_mat[0][1] * p2[0] + f_mat[1][1] * p2[1] + f_mat[2][1];

    let denom = fx1[0] * fx1[0] + fx1[1] * fx1[1] + ftx2_0 * ftx2_0 + ftx2_1 * ftx2_1;
    if denom <= 1e-12 {
        Some(0.0)
    } else {
        Some((num * num) / denom)
    }
}

/// ## Step: 相機外參構造本質矩陣 (Essential Matrix from Rotation & Translation)
/// 計算 E = [t]x * R
/// @outputs: 1
/// @domain: 旋轉矩陣與平移向量輸入皆為有限實數
pub fn essential_matrix_from_rt(
    rot: [[f32; 3]; 3],
    t: [f32; 3],
) -> [[f32; 3]; 3] {
    // 反對稱矩陣 [t]x
    let tx = [
        [0.0, -t[2], t[1]],
        [t[2], 0.0, -t[0]],
        [-t[1], t[0], 0.0],
    ];
    let mut e = [[0.0f32; 3]; 3];
    for r in 0..3 {
        for c in 0..3 {
            e[r][c] = tx[r][0] * rot[0][c] + tx[r][1] * rot[1][c] + tx[r][2] * rot[2][c];
        }
    }
    e
}

/// ## Calc: Hartley 影像坐標等向歸一化尺度與質心 (Hartley Point Normalization)
/// 回傳 (質心 [cx, cy], 等向縮放因子 scale) 滿足平均離中心距離為 sqrt(2)
/// @outputs: 1
/// @domain: points.len() >= 4 && points.len() <= 2048
pub fn vision_dlt_point_normalize_scale(points: &[[f32; 2]]) -> Option<([f32; 2], f32)> {
    if points.len() < 4 || points.len() > 2048 {
        return None;
    }
    let n = points.len() as f32;
    let mut mean_x = 0.0f32;
    let mut mean_y = 0.0f32;
    for p in points {
        if !p[0].is_finite() || !p[1].is_finite() { return None; }
        mean_x += p[0];
        mean_y += p[1];
    }
    mean_x /= n;
    mean_y /= n;

    let mut mean_dist = 0.0f32;
    for p in points {
        let dx = p[0] - mean_x;
        let dy = p[1] - mean_y;
        mean_dist += (dx * dx + dy * dy).sqrt();
    }
    mean_dist /= n;
    if mean_dist <= 1e-8 {
        return None;
    }
    let scale = 1.41421356 / mean_dist;
    Some(([mean_x, mean_y], scale))
}

/// ## Calc: 相機重投影均方根誤差 (Reprojection RMSE in Pixels)
/// @outputs: 1
/// @domain: observed.len() == projected.len() && observed.len() > 0
pub fn camera_reprojection_rmse(
    observed: &[[f32; 2]],
    projected: &[[f32; 2]],
) -> Option<f32> {
    if observed.is_empty() || observed.len() != projected.len() || observed.len() > 4096 {
        return None;
    }
    let mut sum_sq = 0.0f32;
    for (o, p) in observed.iter().zip(projected.iter()) {
        let dx = o[0] - p[0];
        let dy = o[1] - p[1];
        if !dx.is_finite() || !dy.is_finite() {
            return None;
        }
        sum_sq += dx * dx + dy * dy;
    }
    let rmse = (sum_sq / observed.len() as f32).sqrt();
    Some(rmse)
}

/// ## Calc: P3P Grunert 三角餘弦定理邊長約制單步 (P3P Grunert Cosine Law Step)
/// 計算 dab^2 = ra^2 + rb^2 - 2 * ra * rb * cos_gamma
/// @outputs: 1
/// @domain: ra >= 0.0 && rb >= 0.0 && cos_gamma [-1.0..=1.0]
pub fn p3p_grunert_cosine_law_step(ra: f32, rb: f32, cos_gamma: f32) -> f32 {
    if !ra.is_finite() || !rb.is_finite() || !cos_gamma.is_finite() {
        return 0.0;
    }
    let dab_sq = ra * ra + rb * rb - 2.0 * ra * rb * cos_gamma.clamp(-1.0, 1.0);
    if dab_sq <= 0.0 { 0.0 } else { dab_sq.sqrt() }
}
