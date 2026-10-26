//! # Numerical Optimization & Gradient Solver Micro-Bricks (數值最佳化與梯度求解微積木)
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋一維黃金分割線搜尋步進、Wolfe 充分下降與曲率條件檢定、
//! 非線性共軛梯度法 (Polak-Ribière CG) 權重步進、BFGS 逆黑塞矩陣秩二更新步進、
//! 2D 高斯-牛頓 (Gauss-Newton) 非線性最小二乘步進、Levenberg-Marquardt (LM) 阻尼係數自適應調整、
//! Nelder-Mead 單純形法反射與擴張點運算、餘弦退火學習率預熱排程 (Cosine Warmup)。

/// ## Map: 黃金分割線搜尋區間縮減步進 (Golden-Section Search Interval Step)
/// 比率 phi = 0.618033988749895。回傳縮減後之區間 [new_a, new_b]。
/// @outputs: 1
/// @domain: a [-1e12, 1e12], b [-1e12, 1e12], f_x1 [-1e12, 1e12], f_x2 [-1e12, 1e12]
pub fn opt_golden_section_search_step(
    a: f64,
    b: f64,
    f_x1: f64,
    f_x2: f64,
) -> (f64, f64) {
    if !a.is_finite() || !b.is_finite() || !f_x1.is_finite() || !f_x2.is_finite() || a >= b {
        return (a, b);
    }
    const INV_PHI: f64 = 0.618033988749895; // (sqrt(5) - 1) / 2
    let x1 = b - INV_PHI * (b - a);
    let x2 = a + INV_PHI * (b - a);

    if f_x1 < f_x2 {
        (a, x2) // 最小值在左側區間 [a, x2]
    } else {
        (x1, b) // 最小值在右側區間 [x1, b]
    }
}

/// ## Verify: Armijo 充分下降條件檢定 (Wolfe Condition 1 / Sufficient Decrease)
/// 條件：f(x + alpha * p) <= f(x) + c1 * alpha * (grad_f^T * p)
/// @outputs: 1
/// @domain: f_0 [-1e12, 1e12], f_alpha [-1e12, 1e12], dir_deriv [-1e12, 1e12], alpha [0.0, 1e6], c1 [0.0, 1.0]
pub fn opt_wolfe_condition_armijo(
    f_0: f64,
    f_alpha: f64,
    dir_deriv: f64,
    alpha: f64,
    c1: f64,
) -> bool {
    if !f_0.is_finite() || !f_alpha.is_finite() || !dir_deriv.is_finite() || !alpha.is_finite() || !c1.is_finite() {
        return false;
    }
    if alpha <= 0.0 || c1 <= 0.0 || c1 >= 1.0 || dir_deriv >= 0.0 {
        return false; // 下降方向必須滿足 dir_deriv < 0
    }
    f_alpha <= f_0 + c1 * alpha * dir_deriv
}

/// ## Verify: Wolfe 曲率條件檢定 (Wolfe Condition 2 / Curvature Condition)
/// 弱條件：grad(x + alpha * p)^T * p >= c2 * (grad_f^T * p)
/// @outputs: 1
/// @domain: dir_deriv_0 [-1e12, 1e12], dir_deriv_alpha [-1e12, 1e12], c2 [0.0, 1.0]
pub fn opt_wolfe_condition_curvature(
    dir_deriv_0: f64,
    dir_deriv_alpha: f64,
    c2: f64,
) -> bool {
    if !dir_deriv_0.is_finite() || !dir_deriv_alpha.is_finite() || !c2.is_finite() {
        return false;
    }
    if c2 <= 0.0 || c2 >= 1.0 || dir_deriv_0 >= 0.0 {
        return false;
    }
    dir_deriv_alpha >= c2 * dir_deriv_0
}

/// ## Step: Polak-Ribière 非線性共軛梯度法 Beta 係數步進 (PR-CG Beta Step)
/// 內建自動正交重置防護：beta = max(0, (g_k^T * (g_k - g_prev)) / ||g_prev||^2)
/// @outputs: 1
pub fn opt_conjugate_gradient_polak_ribiere(g_k: &[f64], g_prev: &[f64]) -> f64 {
    if g_k.is_empty() || g_k.len() != g_prev.len() {
        return 0.0;
    }
    let mut dot_diff = 0.0;
    let mut norm_prev_sq = 0.0;

    for i in 0..g_k.len() {
        let gk = g_k[i];
        let gp = g_prev[i];
        if !gk.is_finite() || !gp.is_finite() {
            return 0.0;
        }
        dot_diff += gk * (gk - gp);
        norm_prev_sq += gp * gp;
    }
    if norm_prev_sq < 1e-15 {
        return 0.0;
    }
    let beta = dot_diff / norm_prev_sq;
    if beta > 0.0 { beta } else { 0.0 }
}

/// ## Step: BFGS 擬牛頓法逆黑塞矩陣秩二更新單步 (BFGS Inverse Hessian Rank-2 Update)
/// H_{k+1} = (I - rho * s * y^T) * H * (I - rho * y * s^T) + rho * s * s^T
/// 其中 s = x_{k+1} - x_k, y = g_{k+1} - g_k, rho = 1 / (y^T * s)
/// H 為 n*n 行優先 (row-major) 陣列，長度必須為 n*n。
/// @outputs: 1
pub fn opt_bfgs_hessian_inv_update_step(
    h: &mut [f64],
    s: &[f64],
    y: &[f64],
    n: usize,
) -> bool {
    if n == 0 || h.len() != n * n || s.len() != n || y.len() != n {
        return false;
    }
    let mut ys = 0.0;
    for i in 0..n {
        if !s[i].is_finite() || !y[i].is_finite() { return false; }
        ys += y[i] * s[i];
    }
    if ys < 1e-15 { return false; } // 保證正定性曲率條件 y^T * s > 0
    let rho = 1.0 / ys;

    let mut hy = vec![0.0; n];
    for r in 0..n {
        let mut sum = 0.0;
        for c in 0..n { sum += h[r * n + c] * y[c]; }
        hy[r] = sum;
    }
    let mut y_hy = 0.0;
    for i in 0..n { y_hy += y[i] * hy[i]; }

    for r in 0..n {
        for c in 0..n {
            let term1 = rho * (s[r] * hy[c] + hy[r] * s[c]);
            let term2 = rho * (1.0 + rho * y_hy) * (s[r] * s[c]);
            h[r * n + c] = h[r * n + c] - term1 + term2;
        }
    }
    true
}

/// ## Solve: 2D 高斯-牛頓法非線性最小二乘步進解算 (Gauss-Newton 2D Step)
/// 求解 (J^T * J) * delta = -J^T * r，J 為 m*2 雅可比矩陣，r 為 m 維殘差向量。
/// @outputs: 1
pub fn opt_gauss_newton_step_2d(j: &[[f64; 2]], r: &[f64]) -> Option<[f64; 2]> {
    if j.is_empty() || j.len() != r.len() {
        return None;
    }
    let mut jtj = [0.0f64; 4]; // [00, 01, 10, 11]
    let mut neg_jtr = [0.0f64; 2];

    for i in 0..j.len() {
        let j0 = j[i][0];
        let j1 = j[i][1];
        let ri = r[i];
        if !j0.is_finite() || !j1.is_finite() || !ri.is_finite() { return None; }
        jtj[0] += j0 * j0;
        jtj[1] += j0 * j1;
        jtj[3] += j1 * j1;
        neg_jtr[0] -= j0 * ri;
        neg_jtr[1] -= j1 * ri;
    }
    jtj[2] = jtj[1]; // 對稱

    let det = jtj[0] * jtj[3] - jtj[1] * jtj[2];
    if det.abs() < 1e-12 {
        return None; // 奇異矩陣無法求逆
    }
    let inv_det = 1.0 / det;
    let delta0 = inv_det * (jtj[3] * neg_jtr[0] - jtj[1] * neg_jtr[1]);
    let delta1 = inv_det * (-jtj[2] * neg_jtr[0] + jtj[0] * neg_jtr[1]);
    Some([delta0, delta1])
}

/// ## Step: Levenberg-Marquardt (LM) 阻尼係數自適應更新步進
/// 依據增益比率 rho = (實際下降量 / 線性預測下降量) 縮放 lambda
/// @outputs: 1
/// @domain: lambda [1e-12, 1e12], rho [-1e6, 1e6]
pub fn opt_levenberg_marquardt_damping(lambda: f64, rho: f64) -> f64 {
    if !lambda.is_finite() || !rho.is_finite() || lambda <= 0.0 {
        return 1.0;
    }
    if rho > 0.75 {
        (lambda * 0.3333333333333333).max(1e-7) // 下降顯著，逼近牛頓法
    } else if rho < 0.25 {
        (lambda * 2.0).min(1e7) // 預測不佳，逼近梯度下降法
    } else {
        lambda // 預測合理，維持阻尼係數
    }
}

/// ## Step: Nelder-Mead 單純形法反射點運算 (Simplex Reflection Point)
/// 公式：x_r = centroid + alpha * (centroid - worst)
/// @outputs: 1
/// @domain: alpha [0.1, 10.0]
pub fn opt_nelder_mead_reflection_point(
    centroid: &[f64],
    worst: &[f64],
    alpha: f64,
    out: &mut [f64],
) -> bool {
    if centroid.is_empty() || centroid.len() != worst.len() || centroid.len() != out.len() {
        return false;
    }
    if !alpha.is_finite() || alpha <= 0.0 {
        return false;
    }
    for i in 0..centroid.len() {
        let c = centroid[i];
        let w = worst[i];
        if !c.is_finite() || !w.is_finite() { return false; }
        out[i] = c + alpha * (c - w);
    }
    true
}

/// ## Step: Nelder-Mead 單純形法擴張點運算 (Simplex Expansion Point)
/// 公式：x_e = centroid + gamma * (reflected - centroid)
/// @outputs: 1
/// @domain: gamma [1.0, 10.0]
pub fn opt_nelder_mead_expansion_point(
    centroid: &[f64],
    reflected: &[f64],
    gamma: f64,
    out: &mut [f64],
) -> bool {
    if centroid.is_empty() || centroid.len() != reflected.len() || centroid.len() != out.len() {
        return false;
    }
    if !gamma.is_finite() || gamma <= 1.0 {
        return false;
    }
    for i in 0..centroid.len() {
        let c = centroid[i];
        let r = reflected[i];
        if !c.is_finite() || !r.is_finite() { return false; }
        out[i] = c + gamma * (r - c);
    }
    true
}

/// ## Step: 餘弦退火學習率預熱排程步進 (Cosine Decay with Warmup)
/// @outputs: 1
/// @domain: base_lr [0.0, 100.0], min_lr [0.0, 100.0]
pub fn opt_learning_rate_warmup_cosine(
    step: usize,
    warmup_steps: usize,
    max_steps: usize,
    base_lr: f64,
    min_lr: f64,
) -> f64 {
    if !base_lr.is_finite() || !min_lr.is_finite() || base_lr <= 0.0 || min_lr < 0.0 || max_steps == 0 {
        return 0.0;
    }
    if step < warmup_steps {
        // 線性預熱階段
        let progress = (step as f64) / (warmup_steps.max(1) as f64);
        return min_lr + progress * (base_lr - min_lr);
    }
    if step >= max_steps {
        return min_lr;
    }
    // 餘弦退火階段
    let decay_steps = (max_steps - warmup_steps) as f64;
    let current_decay = (step - warmup_steps) as f64;
    let progress = current_decay / decay_steps;
    const PI: f64 = 3.141592653589793;
    let cosine_decay = 0.5 * (1.0 + (progress * PI).cos());
    min_lr + (base_lr - min_lr) * cosine_decay
}
