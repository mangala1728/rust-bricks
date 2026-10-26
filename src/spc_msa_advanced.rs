//! # Advanced SPC & Automotive MSA Micro-Bricks (進階統計製程管制與車規MSA微積木)
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 長期性能 Pp/Ppk、樣本偏度/超額峰度、Box-Cox 常態轉換、Shewhart X-bar/R 管制界限、
//! Durbin-Watson 自相關檢定、Weibull 可靠度、Anderson-Darling 常態性檢定、卡方累積概率近似與 %PV 變異佔比。

/// ## Map: 長期製程性能指標 (Pp 與 Ppk: 採用總體變異 s_overall)
/// @outputs: 1
pub fn spc_pp_ppk_performance(mean: f64, overall_std: f64, usl: f64, lsl: f64) -> Option<(f64, f64)> {
    if overall_std <= 1e-12 || usl <= lsl || mean.is_nan() || overall_std.is_nan() {
        return None;
    }
    let pp = (usl - lsl) / (6.0 * overall_std);
    let cpu = (usl - mean) / (3.0 * overall_std);
    let cpl = (mean - lsl) / (3.0 * overall_std);
    let ppk = cpu.min(cpl);
    Some((pp, ppk))
}

/// ## Map: 樣本偏度 (Skewness) 與超額峰度 (Excess Kurtosis)
/// @outputs: 1
pub fn spc_skewness_kurtosis(samples: &[f64]) -> Option<(f64, f64)> {
    let n = samples.len() as f64;
    if samples.len() < 4 {
        return None;
    }
    let mean = samples.iter().sum::<f64>() / n;
    let mut m2 = 0.0f64;
    let mut m3 = 0.0f64;
    let mut m4 = 0.0f64;
    for &x in samples {
        if x.is_nan() { return None; }
        let d = x - mean;
        let d2 = d * d;
        m2 += d2;
        m3 += d2 * d;
        m4 += d2 * d2;
    }
    let s2 = m2 / (n - 1.0);
    let s = s2.sqrt();
    if s <= 1e-12 {
        return None;
    }
    // 樣本無偏偏度近似
    let skew = (n * m3) / ((n - 1.0) * (n - 2.0) * s * s2);
    // 樣本無偏超額峰度近似 (正態分佈 excess kurtosis = 0)
    let term1 = (n * (n + 1.0) * m4) / ((n - 1.0) * (n - 2.0) * (n - 3.0) * s2 * s2);
    let term2 = (3.0 * (n - 1.0) * (n - 1.0)) / ((n - 2.0) * (n - 3.0));
    let kurt = term1 - term2;
    Some((skew, kurt))
}

/// ## Map: Box-Cox 冪變換步進 (x > 0)
/// @domain: x [0.000001, 1000000000.0]
/// @outputs: 1
pub fn spc_box_cox_transform_step(x: f64, lambda: f64) -> Option<f64> {
    if x <= 0.0 || x.is_nan() || lambda.is_nan() {
        return None;
    }
    if lambda.abs() < 1e-6 {
        Some(x.ln())
    } else {
        Some((x.powf(lambda) - 1.0) / lambda)
    }
}

/// ## Map: Shewhart X-bar 平均值管制圖界限 (UCL, CL, LCL)
/// @outputs: 1
pub fn spc_shewhart_xbar_limits(grand_mean: f64, avg_range: f64, a2_factor: f64) -> (f64, f64, f64) {
    if grand_mean.is_nan() || avg_range < 0.0 || a2_factor < 0.0 {
        return (0.0, 0.0, 0.0);
    }
    let half_width = a2_factor * avg_range;
    let ucl = grand_mean + half_width;
    let lcl = grand_mean - half_width;
    (ucl, grand_mean, lcl)
}

/// ## Map: Shewhart R 全距管制圖界限 (UCL, CL, LCL)
/// @outputs: 1
pub fn spc_shewhart_r_limits(avg_range: f64, d3_factor: f64, d4_factor: f64) -> (f64, f64, f64) {
    if avg_range < 0.0 || d3_factor < 0.0 || d4_factor < d3_factor {
        return (0.0, 0.0, 0.0);
    }
    let ucl = d4_factor * avg_range;
    let lcl = d3_factor * avg_range;
    (ucl, avg_range, lcl)
}

/// ## Map: Durbin-Watson 自相關檢定統計量 (d: 2 表示無自相關)
/// @outputs: 1
pub fn spc_durbin_watson_autocorr(residuals: &[f64]) -> Option<f64> {
    if residuals.len() < 2 {
        return None;
    }
    let mut sum_diff_sq = 0.0f64;
    let mut sum_sq = 0.0f64;
    for i in 0..residuals.len() {
        let e = residuals[i];
        if e.is_nan() { return None; }
        sum_sq += e * e;
        if i > 0 {
            let diff = e - residuals[i - 1];
            sum_diff_sq += diff * diff;
        }
    }
    if sum_sq <= 1e-12 {
        None
    } else {
        Some(sum_diff_sq / sum_sq)
    }
}

/// ## Map: 韋伯 (Weibull) 可靠度函數 R(t) = exp(-(t/eta)^beta)
/// @domain: t [0.0, 1000000000.0]
/// @domain: eta_scale [0.000001, 1000000000.0]
/// @domain: beta_shape [0.01, 100.0]
/// @outputs: 1
pub fn spc_weibull_reliability_step(t: f64, eta_scale: f64, beta_shape: f64) -> Option<f64> {
    if t < 0.0 || eta_scale <= 0.0 || beta_shape <= 0.0 || t.is_nan() {
        return None;
    }
    if t == 0.0 {
        return Some(1.0); // R(0) = 1 (INC-20261026-07)
    }
    let ratio = t / eta_scale;
    let power = ratio.powf(beta_shape);
    Some((-power).exp().clamp(0.0, 1.0))
}

/// ## Map: Anderson-Darling A² 常態性統計量計算
/// @outputs: 1
pub fn spc_anderson_darling_a2_step(norm_cdf_vals_sorted: &[f64]) -> Option<f64> {
    let n = norm_cdf_vals_sorted.len() as f64;
    if norm_cdf_vals_sorted.is_empty() {
        return None;
    }
    let mut s = 0.0f64;
    for (idx, &f) in norm_cdf_vals_sorted.iter().enumerate() {
        let i = (idx + 1) as f64;
        let rev_f = norm_cdf_vals_sorted[norm_cdf_vals_sorted.len() - idx - 1];
        let p1 = f.clamp(1e-12, 1.0 - 1e-12);
        let p2 = (1.0 - rev_f).clamp(1e-12, 1.0 - 1e-12);
        let term = (2.0 * i - 1.0) * (p1.ln() + p2.ln());
        s += term;
    }
    let a2 = -n - s / n;
    Some(a2.max(0.0))
}

/// ## Map: 卡方累積概率近似 (Wilson-Hilferty 正態轉換近似)
/// @outputs: 1
pub fn spc_chi_square_cdf_approx(chi2: f64, df: u32) -> Option<f64> {
    if chi2 < 0.0 || df == 0 || chi2.is_nan() {
        return None;
    }
    let k = df as f64;
    // Wilson-Hilferty 轉換: Z = ((chi2 / k)^(1/3) - (1 - 2/(9k))) / sqrt(2/(9k))
    let term1 = (chi2 / k).powf(1.0 / 3.0);
    let term2 = 1.0 - 2.0 / (9.0 * k);
    let denom = (2.0 / (9.0 * k)).sqrt();
    let z = (term1 - term2) / denom;
    // 標準正態 CDF 近似
    Some(standard_normal_cdf(z))
}

/// ## Map: AIAG MSA 零件間變異佔比 (%PV = 100 * PV / TV)
/// @outputs: 1
pub fn spc_msa_part_to_part_variation(pv_std: f64, total_variation_std: f64) -> Option<f64> {
    if pv_std < 0.0 || total_variation_std <= 1e-12 || pv_std > total_variation_std {
        return None;
    }
    Some((pv_std / total_variation_std) * 100.0)
}

// exp(-8) 得 55.6，導致 Weibull / Box-Cox / 卡方 CDF 錯誤（CDF 可為負） (INC-20261026-07)



/// 內部標準常態 CDF 近似
/// @outputs: 1
fn standard_normal_cdf(z: f64) -> f64 {
    let x = z / 1.4142135623730951;
    0.5 * (1.0 + fast_erf_f64(x))
}

/// 內部誤差函數近似
/// @outputs: 1
fn fast_erf_f64(x: f64) -> f64 {
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let abs_x = x.abs();
    let p = 0.3275911;
    let t = 1.0 / (1.0 + p * abs_x);
    let poly = t * (0.254829592 + t * (-0.284496736 + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429))));
    sign * (1.0 - poly * (-abs_x * abs_x).exp())
}
