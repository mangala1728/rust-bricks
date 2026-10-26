// ## Map
// Role: math/curve_fit_trend — 物理實驗與數值分析曲線擬合、局部加權平滑 (LOWESS)、核方法 (RBF/Matérn)、神經基底 (SIREN/SiLU/傅立葉映射) 與擬合優度評估微積木。
// Inputs: 實驗觀測數據點 (x, y)、平滑帶寬、核超參數、頻率權重、殘差與容差。
// Outputs: 線性回歸斜率與截距、R² 判定係數、Huber 穩健損失、LOWESS 平滑值、RBF/Matérn 核矩陣元素、SIREN/SiLU 活化值。
// Invariants: 零外部依賴、純 Rust 實現、除零剛性防護、有界陣列切片、單一函式 <= 60 行。

/// @outputs: 2
/// @domain: x.len() [2..65536]
pub fn curve_linear_regression_slope_intercept(x: &[f64], y: &[f64]) -> Option<(f64, f64, f64)> {
    let n = x.len();
    if n < 2 || y.len() != n {
        return None;
    }
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    for i in 0..n {
        if !x[i].is_finite() || !y[i].is_finite() {
            return None;
        }
        sum_x += x[i];
        sum_y += y[i];
    }
    let mean_x = sum_x / (n as f64);
    let mean_y = sum_y / (n as f64);

    let mut ss_xx = 0.0;
    let mut ss_yy = 0.0;
    let mut ss_xy = 0.0;
    for i in 0..n {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        ss_xx += dx * dx;
        ss_yy += dy * dy;
        ss_xy += dx * dy;
    }
    if ss_xx <= 1e-15 || ss_yy <= 1e-15 {
        return None;
    }
    let slope = ss_xy / ss_xx;
    let intercept = mean_y - slope * mean_x;
    let r = ss_xy / (ss_xx * ss_yy).sqrt();
    Some((slope, intercept, r))
}

/// @outputs: 2
/// @domain: y_actual.len() [2..65536]
pub fn curve_r_squared_score(y_actual: &[f64], y_predicted: &[f64]) -> Option<f64> {
    let n = y_actual.len();
    if n < 2 || y_predicted.len() != n {
        return None;
    }
    let mut sum_y = 0.0;
    for &y in y_actual {
        if !y.is_finite() { return None; }
        sum_y += y;
    }
    let mean_y = sum_y / (n as f64);

    let mut ss_res = 0.0;
    let mut ss_tot = 0.0;
    for i in 0..n {
        if !y_predicted[i].is_finite() { return None; }
        let diff_res = y_actual[i] - y_predicted[i];
        let diff_tot = y_actual[i] - mean_y;
        ss_res += diff_res * diff_res;
        ss_tot += diff_tot * diff_tot;
    }
    if ss_tot <= 1e-15 {
        return None;
    }
    let r2 = 1.0 - (ss_res / ss_tot);
    Some(r2)
}

/// @outputs: 1
/// @domain: delta [1e-6..1000.0]
pub fn curve_huber_loss_single(residual: f64, delta: f64) -> f64 {
    if delta <= 0.0 || !residual.is_finite() || !delta.is_finite() {
        return 0.0;
    }
    let abs_r = residual.abs();
    if abs_r <= delta {
        0.5 * abs_r * abs_r
    } else {
        delta * (abs_r - 0.5 * delta)
    }
}

/// @outputs: 1
pub fn curve_lowess_tricube_weight(dist_norm: f64) -> f64 {
    let u = dist_norm.abs();
    if u >= 1.0 || !u.is_finite() {
        return 0.0;
    }
    let term = 1.0 - u * u * u;
    term * term * term
}

/// @outputs: 2
/// @domain: bandwidth [1e-6..100000.0]
pub fn curve_lowess_smooth_point(
    x_eval: f64,
    x_data: &[f64],
    y_data: &[f64],
    bandwidth: f64,
) -> Option<f64> {
    let n = x_data.len();
    if n == 0 || y_data.len() != n || bandwidth <= 0.0 || !x_eval.is_finite() {
        return None;
    }
    let mut sum_w = 0.0;
    let mut sum_wd = 0.0;
    let mut sum_wdd = 0.0;
    let mut sum_wy = 0.0;
    let mut sum_wdy = 0.0;

    for i in 0..n {
        let d = x_data[i] - x_eval;
        let w = curve_lowess_tricube_weight(d / bandwidth);
        if w > 0.0 {
            let wd = w * d;
            sum_w += w;
            sum_wd += wd;
            sum_wdd += wd * d;
            sum_wy += w * y_data[i];
            sum_wdy += wd * y_data[i];
        }
    }
    if sum_w <= 1e-12 {
        return None;
    }
    let det = sum_w * sum_wdd - sum_wd * sum_wd;
    if det.abs() <= 1e-12 {
        return Some(sum_wy / sum_w);
    }
    let a = (sum_wy * sum_wdd - sum_wd * sum_wdy) / det;
    Some(a)
}

/// @outputs: 1
/// @domain: length_scale [1e-6..100000.0]
/// @domain: sigma [1e-6..100000.0]
pub fn curve_kernel_rbf(x1: f64, x2: f64, length_scale: f64, sigma: f64) -> f64 {
    if length_scale <= 0.0 || sigma <= 0.0 || !x1.is_finite() || !x2.is_finite() {
        return 0.0;
    }
    let diff = x1 - x2;
    let exponent = -(diff * diff) / (2.0 * length_scale * length_scale);
    sigma * sigma * exponent.exp()
}

/// @outputs: 1
/// @domain: length_scale [1e-6..100000.0]
/// @domain: sigma [1e-6..100000.0]
pub fn curve_kernel_matern32(x1: f64, x2: f64, length_scale: f64, sigma: f64) -> f64 {
    if length_scale <= 0.0 || sigma <= 0.0 || !x1.is_finite() || !x2.is_finite() {
        return 0.0;
    }
    let d = (x1 - x2).abs();
    let sqrt3 = 1.7320508075688772;
    let term = (sqrt3 * d) / length_scale;
    sigma * sigma * (1.0 + term) * (-term).exp()
}

/// @outputs: 2
pub fn curve_fourier_feature_proj(x: f32, freq_weights: &[f32], out_proj: &mut [f32]) -> bool {
    let k = freq_weights.len();
    if k == 0 || out_proj.len() < 2 * k || !x.is_finite() {
        return false;
    }
    let two_pi = 2.0 * std::f32::consts::PI;
    for i in 0..k {
        let angle = two_pi * freq_weights[i] * x;
        out_proj[2 * i] = angle.cos();
        out_proj[2 * i + 1] = angle.sin();
    }
    true
}

/// @outputs: 1
/// @domain: omega_0 [0.1..1000.0]
pub fn curve_siren_sine_activation(x: f32, weight: f32, bias: f32, omega_0: f32) -> f32 {
    if omega_0 <= 0.0 || !x.is_finite() || !weight.is_finite() || !bias.is_finite() {
        return 0.0;
    }
    let inner = omega_0 * (weight * x + bias);
    inner.sin()
}

/// @outputs: 1
pub fn curve_silu_activation(x: f32) -> f32 {
    if !x.is_finite() {
        return 0.0;
    }
    if x >= 0.0 {
        x / (1.0 + (-x).exp())
    } else {
        let exp_x = x.exp();
        x * exp_x / (1.0 + exp_x)
    }
}
