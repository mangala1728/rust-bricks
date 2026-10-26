//! # 自動控制系統、系統辨識與卡爾曼平滑器 (Control Theory, System Identification & RTS Smoother)
//!
//! 提供 Rauch-Tung-Striebel (RTS) 反向卡爾曼軌跡平滑、LQR 最佳增益與 Riccati 代數步進、
//! 狀態空間 2D 離散時間步進、遞迴最小平方法 (RLS) 遺忘因子參數辨識、
//! Tustin (雙線性) 連續轉離散濾波器係數、抗積分飽和 Back-Calculation、
//! 超前落後補償器相位、滑動模態 (SMC) 飽和連續化與殘差白雜訊檢驗微積木。
//! 零外部依賴，符合 NASA JPL 規範與合約 09。

/// ## Map: RTS 反向卡爾曼平滑器增益計算 (RTS Backward Smoother Gain)
/// @outputs: 1
/// @domain: cov_prior >= 0.0 && cov_pred > 0.0
pub fn rts_smoother_gain_1d(cov_prior: f32, f_trans: f32, cov_pred: f32) -> Option<f32> {
    if !cov_prior.is_finite() || !f_trans.is_finite() || !cov_pred.is_finite() {
        return None;
    }
    if cov_prior < 0.0 || cov_pred <= 1e-12 {
        return None;
    }
    let c_k = (cov_prior * f_trans) / cov_pred;
    if c_k.is_finite() { Some(c_k) } else { None }
}

/// ## Calc: RTS 平滑狀態單步反向更新 (RTS Smoothed State Step)
/// @outputs: 1
/// @domain: 所有輸入皆為有限實數
pub fn rts_smoother_state_step_1d(
    state_filt: f32,
    c_k: f32,
    state_smooth_next: f32,
    state_pred_next: f32,
) -> f32 {
    if !state_filt.is_finite() || !c_k.is_finite() || !state_smooth_next.is_finite() || !state_pred_next.is_finite() {
        return state_filt;
    }
    state_filt + c_k * (state_smooth_next - state_pred_next)
}

/// ## Calc: 離散時間 LQR 最佳回授增益與 Riccati 單步迭代 (LQR Gain & Riccati Step)
/// 回傳 (更新後 Riccati 方差 P_next, 最佳控制增益 K)
/// @outputs: 1
/// @domain: q >= 0.0 && r > 0.0 && p_curr >= 0.0
pub fn lqr_gain_step_1d(a: f32, b: f32, q: f32, r: f32, p_curr: f32) -> Option<(f32, f32)> {
    if !a.is_finite() || !b.is_finite() || !q.is_finite() || !r.is_finite() || !p_curr.is_finite() {
        return None;
    }
    if q < 0.0 || r <= 1e-12 || p_curr < 0.0 {
        return None;
    }
    let denom = r + b * b * p_curr;
    if denom <= 1e-12 {
        return None;
    }
    let k = (b * p_curr * a) / denom;
    let p_next = q + a * a * p_curr - a * b * p_curr * k;
    Some((p_next, k))
}

/// ## Step: 2維狀態空間 (State-Space) 離散時間轉移步進 (LTI State-Space Step)
/// 回傳 (下一狀態 [x0_next, x1_next], 輸出 y)
/// @outputs: 1
/// @domain: 矩陣係數與狀態皆為有限實數
pub fn state_space_step_2d(
    a: [[f32; 2]; 2],
    b: [f32; 2],
    c: [f32; 2],
    d: f32,
    x: [f32; 2],
    u: f32,
) -> ([f32; 2], f32) {
    if !u.is_finite() || !x[0].is_finite() || !x[1].is_finite() {
        return (x, 0.0);
    }
    let x0_next = a[0][0] * x[0] + a[0][1] * x[1] + b[0] * u;
    let x1_next = a[1][0] * x[0] + a[1][1] * x[1] + b[1] * u;
    let y = c[0] * x[0] + c[1] * x[1] + d * u;
    ([x0_next, x1_next], y)
}

/// ## Calc: 遞迴最小平方法 (RLS) 帶遺忘因子參數辨識 (Recursive Least Squares with Forgetting)
/// 回傳 (更新後參數估計值 theta_new, 更新後方差 P_new)
/// @outputs: 1
/// @domain: lambda >= 0.5 && lambda <= 1.0 && p_cov > 0.0
pub fn rls_parameter_step_1d(
    theta: f32,
    p_cov: f32,
    phi: f32,
    y: f32,
    lambda: f32,
) -> Option<(f32, f32)> {
    if !theta.is_finite() || !p_cov.is_finite() || !phi.is_finite() || !y.is_finite() || !lambda.is_finite() {
        return None;
    }
    if lambda < 0.5 || lambda > 1.0 || p_cov <= 1e-12 {
        return None;
    }
    let denom = lambda + phi * p_cov * phi;
    if denom <= 1e-12 {
        return None;
    }
    let gain_k = (p_cov * phi) / denom;
    let error = y - phi * theta;
    let theta_new = theta + gain_k * error;
    let p_new = (1.0 / lambda) * (p_cov - gain_k * phi * p_cov);
    Some((theta_new, p_new.max(0.0)))
}

/// ## Calc: 一階連續傳遞函數雙線性 (Tustin) 離散化 (Bilinear Transform First-Order)
/// 將 H(s) = 1 / (tau*s + 1) 轉換為 H(z) = (b0 + b0*z^(-1)) / (1 + a1*z^(-1))
/// 回傳 (係數 b0, 係數 a1)
/// @outputs: 1
/// @domain: tau > 0.0 && ts > 0.0
pub fn tustin_bilinear_first_order(tau: f32, ts: f32) -> Option<(f32, f32)> {
    if !tau.is_finite() || !ts.is_finite() || tau <= 1e-6 || ts <= 1e-6 {
        return None;
    }
    let denom = 2.0 * tau + ts;
    if denom <= 1e-12 {
        return None;
    }
    let b0 = ts / denom;
    let a1 = (ts - 2.0 * tau) / denom;
    Some((b0, a1))
}

/// ## Calc: PID 積分抗飽和反向計算單步 (PID Anti-Windup Back-Calculation Step)
/// @outputs: 1
/// @domain: tt > 0.0 && dt > 0.0
pub fn pid_anti_windup_back_calc_step(
    u_unsat: f32,
    u_sat: f32,
    tt: f32,
    integral: f32,
    dt: f32,
) -> f32 {
    if !u_unsat.is_finite() || !u_sat.is_finite() || !tt.is_finite() || !integral.is_finite() || !dt.is_finite() {
        return integral;
    }
    if tt <= 1e-6 || dt <= 0.0 {
        return integral;
    }
    let diff = u_sat - u_unsat;
    integral + (dt / tt) * diff
}

/// ## Calc: 超前補償器最大相位超前角 (Lead Compensator Max Phase Lead in Radians)
/// 計算 phi_m = arcsin((1 - alpha) / (1 + alpha))
/// @outputs: 1
/// @domain: alpha > 0.0 && alpha < 1.0
pub fn lead_lag_max_phase_lead_rad(alpha: f32) -> Option<f32> {
    if !alpha.is_finite() || alpha <= 0.0 || alpha >= 1.0 {
        return None;
    }
    let sin_phi = (1.0 - alpha) / (1.0 + alpha);
    Some(sin_phi.asin())
}

/// ## Calc: 滑動模態控制 (SMC) 邊界層連續飽和函數 (SMC Boundary Layer Saturation)
/// 用於消除開關抖振 (Chattering Free)
/// @outputs: 1
/// @domain: boundary_layer > 0.0 && k_gain >= 0.0
pub fn sliding_mode_saturation_control(s_surface: f32, boundary_layer: f32, k_gain: f32) -> f32 {
    if !s_surface.is_finite() || !boundary_layer.is_finite() || !k_gain.is_finite() {
        return 0.0;
    }
    if boundary_layer <= 1e-6 || k_gain < 0.0 {
        return 0.0;
    }
    let sat = (s_surface / boundary_layer).clamp(-1.0, 1.0);
    -k_gain * sat
}

/// ## Calc: 系統辨識殘差一階自相關 Q 檢定統計量 (Ljung-Box Q-Stat Lag 1)
/// 驗證殘差是否為白雜訊 (White Noise)
/// @outputs: 1
/// @domain: residuals.len() >= 4 && residuals.len() <= 4096
pub fn ljung_box_q_stat_lag1(residuals: &[f32]) -> Option<f32> {
    if residuals.len() < 4 || residuals.len() > 4096 {
        return None;
    }
    let n = residuals.len() as f32;
    let mut sum_sq = 0.0f32;
    let mut sum_lag = 0.0f32;
    for i in 0..residuals.len() {
        let r = residuals[i];
        if !r.is_finite() { return None; }
        sum_sq += r * r;
        if i > 0 {
            sum_lag += r * residuals[i - 1];
        }
    }
    if sum_sq <= 1e-12 {
        return Some(0.0);
    }
    let rho1 = sum_lag / sum_sq;
    let q = n * (rho1 * rho1);
    Some(q)
}
