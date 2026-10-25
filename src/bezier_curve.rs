// ## Map
// Role: core/bezier_curve — 網頁與簡報視覺圖表 Wasm 渲染微積木：三次貝茲曲線座標求值、切線速度與曲率幾何計算。
// Inputs: 控制點座標 (p0, p1, p2, p3)、內插參數 t [0.0..1.0]。
// Outputs: 座標值 (f32)、切線向量斜率 (f32)、曲率 kappa (f32)。
// Invariants: 零外部依賴、無遞迴、零全域可變狀態、浮點數防 NaN/Inf 守衛、控制流巢狀深度 <= 2、純標準庫 100% 零依賴。

/// @outputs: 1
/// @domain: t [0.0..1.0]
pub fn cubic_bezier_point(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    if t.is_nan() || p0.is_nan() || p1.is_nan() || p2.is_nan() || p3.is_nan() {
        return 0.0;
    }
    let u = if t < 0.0 { 0.0 } else if t > 1.0 { 1.0 } else { t };
    let inv_u = 1.0 - u;
    let inv_u2 = inv_u * inv_u;
    let inv_u3 = inv_u2 * inv_u;
    let u2 = u * u;
    let u3 = u2 * u;

    inv_u3 * p0 + 3.0 * inv_u2 * u * p1 + 3.0 * inv_u * u2 * p2 + u3 * p3
}

/// @outputs: 1
/// @domain: t [0.0..1.0]
pub fn cubic_bezier_tangent(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    if t.is_nan() || p0.is_nan() || p1.is_nan() || p2.is_nan() || p3.is_nan() {
        return 0.0;
    }
    let u = if t < 0.0 { 0.0 } else if t > 1.0 { 1.0 } else { t };
    let inv_u = 1.0 - u;

    3.0 * inv_u * inv_u * (p1 - p0) + 6.0 * inv_u * u * (p2 - p1) + 3.0 * u * u * (p3 - p2)
}

/// @outputs: 1
/// @domain: t [0.0..1.0]
pub fn cubic_bezier_curvature(x_pts: [f32; 4], y_pts: [f32; 4], t: f32) -> f32 {
    if t.is_nan() {
        return 0.0;
    }
    let u = if t < 0.0 { 0.0 } else if t > 1.0 { 1.0 } else { t };
    let dx = cubic_bezier_tangent(x_pts[0], x_pts[1], x_pts[2], x_pts[3], u);
    let dy = cubic_bezier_tangent(y_pts[0], y_pts[1], y_pts[2], y_pts[3], u);

    let inv_u = 1.0 - u;
    let ddx = 6.0 * inv_u * (x_pts[2] - 2.0 * x_pts[1] + x_pts[0])
        + 6.0 * u * (x_pts[3] - 2.0 * x_pts[2] + x_pts[1]);
    let ddy = 6.0 * inv_u * (y_pts[2] - 2.0 * y_pts[1] + y_pts[0])
        + 6.0 * u * (y_pts[3] - 2.0 * y_pts[2] + y_pts[1]);

    let num = (dx * ddy - dy * ddx).abs();
    let denom = (dx * dx + dy * dy).powf(1.5);
    if denom <= 1e-6 {
        0.0
    } else {
        num / denom
    }
}
