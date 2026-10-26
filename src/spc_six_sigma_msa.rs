// ## Map
// Role: stat/spc_six_sigma_msa — 六標準差製程能力 (Cp/Cpk)、截斷常態分佈 (Truncated Normal) 與車用 AIAG MSA 量測系統分析 (Gage R&R, P/T Ratio, ndc) 微積木。
// Inputs: 規格上下限 (USL, LSL)、製程均值與標準差、截斷區間、設備重複性 (EV)、作業員再現性 (AV)、零件變異 (PV)。
// Outputs: Cp/Cpk 指標、合格機率與 DPPM、截斷平均值與變異量、%GRR、P/T 比率、區分度分類數 (ndc)。
// Invariants: 零外部依賴、純 Rust 實現、除零剛性防護、無溢位、單一函式 <= 60 行。

/// @outputs: 1
/// @domain: usl [lsl..1.0e12]
/// @domain: std_dev [1.0e-12..1.0e9]
pub fn spc_cp_index(usl: f64, lsl: f64, std_dev: f64) -> Option<f64> {
    if usl <= lsl || std_dev <= 0.0 || !usl.is_finite() || !lsl.is_finite() || !std_dev.is_finite() {
        return None;
    }
    let denom = 6.0 * std_dev;
    if denom <= 1e-15 {
        return None;
    }
    Some((usl - lsl) / denom)
}

/// @outputs: 3
/// @domain: usl [lsl..1.0e12]
/// @domain: std_dev [1.0e-12..1.0e9]
pub fn spc_cpk_index(usl: f64, lsl: f64, mean: f64, std_dev: f64) -> Option<(f64, f64, f64)> {
    if usl <= lsl || std_dev <= 0.0 || !mean.is_finite() || !std_dev.is_finite() {
        return None;
    }
    let denom = 3.0 * std_dev;
    if denom <= 1e-15 {
        return None;
    }
    let cpu = (usl - mean) / denom;
    let cpl = (mean - lsl) / denom;
    let cpk = cpu.min(cpl);
    Some((cpk, cpu, cpl))
}

/// @outputs: 1
pub fn spc_normal_cdf_phi(z: f64) -> f64 {
    if !z.is_finite() {
        return if z < 0.0 { 0.0 } else { 1.0 };
    }
    // Abramowitz & Stegun 7.1.26 高精度誤差函數近似
    let sign = if z < 0.0 { -1.0 } else { 1.0 };
    let x = z.abs() / std::f64::consts::SQRT_2;
    let p = 0.3275911;
    let t = 1.0 / (1.0 + p * x);
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let poly = (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t;
    let erf_val = sign * (1.0 - poly * (-x * x).exp());
    0.5 * (1.0 + erf_val)
}

/// @outputs: 2
/// @domain: usl [lsl..1.0e12]
/// @domain: std_dev [1.0e-12..1.0e9]
pub fn spc_spec_in_boundary_probability(
    usl: f64,
    lsl: f64,
    mean: f64,
    std_dev: f64,
) -> Option<(f64, f64)> {
    if usl <= lsl || std_dev <= 0.0 || !mean.is_finite() || !std_dev.is_finite() {
        return None;
    }
    let z_low = (lsl - mean) / std_dev;
    let z_high = (usl - mean) / std_dev;
    let p_low = spc_normal_cdf_phi(z_low);
    let p_high = spc_normal_cdf_phi(z_high);
    let in_spec = (p_high - p_low).clamp(0.0, 1.0);
    let defect_prob = 1.0 - in_spec;
    let dppm = defect_prob * 1.0e6;
    Some((in_spec, dppm))
}

/// @outputs: 3
/// @domain: a [..b]
/// @domain: std_dev [1.0e-12..1.0e9]
pub fn spc_truncated_normal_stats(
    a: f64,
    b: f64,
    mean: f64,
    std_dev: f64,
) -> Option<(f64, f64, f64)> {
    if a >= b || std_dev <= 0.0 || !mean.is_finite() || !std_dev.is_finite() {
        return None;
    }
    let alpha = (a - mean) / std_dev;
    let beta = (b - mean) / std_dev;
    let inv_sqrt_2pi = 0.3989422804014327;
    let phi_alpha = inv_sqrt_2pi * (-0.5 * alpha * alpha).exp();
    let phi_beta = inv_sqrt_2pi * (-0.5 * beta * beta).exp();
    let cap_phi_a = spc_normal_cdf_phi(alpha);
    let cap_phi_b = spc_normal_cdf_phi(beta);
    let z = cap_phi_b - cap_phi_a;
    if z <= 1e-12 {
        return None;
    }
    let delta_phi = phi_alpha - phi_beta;
    let trunc_mean = mean + std_dev * (delta_phi / z);
    let term2 = (alpha * phi_alpha - beta * phi_beta) / z;
    let ratio = delta_phi / z;
    let var_factor = (1.0 + term2 - ratio * ratio).max(0.0);
    let trunc_std_dev = std_dev * var_factor.sqrt();
    let median_shift = trunc_mean - mean;
    Some((trunc_mean, trunc_std_dev, median_shift))
}

/// @outputs: 1
/// @domain: a [..b]
/// @domain: std_dev [1.0e-12..1.0e9]
pub fn spc_truncated_normal_pdf(
    x: f64,
    a: f64,
    b: f64,
    mean: f64,
    std_dev: f64,
) -> Option<f64> {
    if a >= b || std_dev <= 0.0 || x < a || x > b || !x.is_finite() || !mean.is_finite() {
        return None;
    }
    let alpha = (a - mean) / std_dev;
    let beta = (b - mean) / std_dev;
    let z = spc_normal_cdf_phi(beta) - spc_normal_cdf_phi(alpha);
    if z <= 1e-12 {
        return None;
    }
    let inv_sqrt_2pi = 0.3989422804014327;
    let z_x = (x - mean) / std_dev;
    let phi_x = inv_sqrt_2pi * (-0.5 * z_x * z_x).exp();
    Some(phi_x / (std_dev * z))
}

/// @outputs: 1
/// @domain: repeatability_ev [0.0..1.0e9]
/// @domain: reproducibility_av [0.0..1.0e9]
pub fn msa_gage_rr_ev_av(repeatability_ev: f64, reproducibility_av: f64) -> Option<f64> {
    if repeatability_ev < 0.0 || reproducibility_av < 0.0
        || !repeatability_ev.is_finite() || !reproducibility_av.is_finite() {
        return None;
    }
    Some((repeatability_ev * repeatability_ev + reproducibility_av * reproducibility_av).sqrt())
}

/// @outputs: 1
/// @domain: grr [0.0..1.0e9]
/// @domain: pv [0.0..1.0e9]
pub fn msa_gage_rr_percent_grr(grr: f64, pv: f64) -> Option<f64> {
    if grr < 0.0 || pv < 0.0 || !grr.is_finite() || !pv.is_finite() {
        return None;
    }
    let total_var = grr * grr + pv * pv;
    if total_var <= 1e-15 {
        return None;
    }
    let tv = total_var.sqrt();
    Some((grr / tv) * 100.0)
}

/// @outputs: 1
/// @domain: grr_sigma [0.0..1.0e9]
/// @domain: usl [lsl..1.0e12]
pub fn msa_precision_to_tolerance_ratio(grr_sigma: f64, usl: f64, lsl: f64) -> Option<f64> {
    if usl <= lsl || grr_sigma < 0.0 || !grr_sigma.is_finite() || !usl.is_finite() || !lsl.is_finite() {
        return None;
    }
    let tol = usl - lsl;
    if tol <= 1e-15 {
        return None;
    }
    // AIAG 標準 P/T Ratio = (6 * GRR) / (USL - LSL) * 100%
    Some(((6.0 * grr_sigma) / tol) * 100.0)
}

/// @outputs: 1
/// @domain: pv [0.0..1.0e9]
/// @domain: grr [1.0e-12..1.0e9]
pub fn msa_number_of_distinct_categories_ndc(pv: f64, grr: f64) -> Option<u32> {
    if pv < 0.0 || grr <= 0.0 || !pv.is_finite() || !grr.is_finite() {
        return None;
    }
    // AIAG 4th Edition: ndc = 1.41 * (PV / GRR)
    let ndc_float = 1.41421356237 * (pv / grr);
    Some(ndc_float.floor() as u32)
}
