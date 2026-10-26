// ## Map
// Role: eda/pcb_cad_fem — 零依賴 KiCad / OpenEMS / FreeCAD 之微帶線特徵阻抗、FDTD CFL 穩定性極限與 Gerber 幾何運算微積木。
// Inputs: 走線寬度與介質厚度、相對介電常數、FDTD 3D 網格步長 (dx, dy, dz)、Gerber 座標字串。
// Outputs: 50Ω 特徵阻抗 Z0、有效介電常數、FDTD 最大時間步長 dt (s)、微米奈米座標、多邊形面積與重心。
// Invariants: 零外部依賴、純 Rust 實現、除零剛性防護、單一函式 <= 60 行。

const SPEED_OF_LIGHT_M_S: f64 = 299_792_458.0;

/// @outputs: 2
/// @domain: width_mm [0.001..100.0]
/// @domain: height_mm [0.001..100.0]
/// @domain: er [1.0..100.0]
pub fn microstrip_impedance_hammerstad(width_mm: f64, height_mm: f64, er: f64) -> Option<(f64, f64)> {
    if width_mm <= 0.0 || height_mm <= 0.0 || er < 1.0 || !width_mm.is_finite() || !height_mm.is_finite() || !er.is_finite() {
        return None;
    }
    let u = width_mm / height_mm;
    let eps_eff = (er + 1.0) / 2.0 + ((er - 1.0) / 2.0) * (1.0 + 12.0 / u).powf(-0.5);
    let sqrt_eps = eps_eff.sqrt();
    if sqrt_eps <= 0.0 {
        return None;
    }

    let z0 = if u <= 1.0 {
        let term = (8.0 / u) + 0.25 * u;
        (60.0 / sqrt_eps) * term.ln()
    } else {
        let term = u + 1.393 + 0.667 * (u + 1.444).ln();
        (120.0 * std::f64::consts::PI) / (sqrt_eps * term)
    };
    if z0.is_finite() && eps_eff.is_finite() {
        Some((z0, eps_eff))
    } else {
        None
    }
}

/// @outputs: 2
/// @domain: dx_m [1e-9..1.0]
/// @domain: dy_m [1e-9..1.0]
/// @domain: dz_m [1e-9..1.0]
pub fn openems_fdtd_cfl_max_timestep_sec(dx_m: f64, dy_m: f64, dz_m: f64) -> Option<f64> {
    if dx_m <= 0.0 || dy_m <= 0.0 || dz_m <= 0.0 || !dx_m.is_finite() || !dy_m.is_finite() || !dz_m.is_finite() {
        return None;
    }
    let inv_sum = (1.0 / (dx_m * dx_m)) + (1.0 / (dy_m * dy_m)) + (1.0 / (dz_m * dz_m));
    let sqrt_inv = inv_sum.sqrt();
    if sqrt_inv <= 0.0 {
        return None;
    }
    let dt_max = 1.0 / (SPEED_OF_LIGHT_M_S * sqrt_inv);
    if dt_max.is_finite() { Some(dt_max) } else { None }
}

/// @outputs: 2
/// @domain: frac_digits [1..8]
pub fn gerber_coord_to_microns(coord_str: &[u8], frac_digits: usize) -> Option<i64> {
    if coord_str.is_empty() || frac_digits == 0 || frac_digits > 8 || coord_str.len() > 32 {
        return None;
    }
    let mut i = 0;
    let mut sign = 1i64;
    if coord_str[0] == b'-' {
        sign = -1;
        i += 1;
    } else if coord_str[0] == b'+' {
        i += 1;
    }
    let mut raw_val = 0i64;
    while i < coord_str.len() {
        let b = coord_str[i];
        if !b.is_ascii_digit() {
            return None;
        }
        raw_val = raw_val.saturating_mul(10).saturating_add((b - b'0') as i64);
        i += 1;
    }
    // 假設輸入單位為英吋 (Gerber 2.4 / 2.5 / 2.6 標準英吋座標格式)
    // 1 inch = 25.4 mm = 25,400 微米 (um)
    let unit_scale = 25400i64;
    let mut divisor = 1i64;
    for _ in 0..frac_digits {
        divisor = divisor.saturating_mul(10);
    }
    if divisor == 0 {
        return None;
    }
    let microns = (raw_val * unit_scale * sign) / divisor;
    Some(microns)
}

/// @outputs: 2
/// @domain: width_mm [0.01..50.0]
/// @domain: height_mm [0.01..50.0]
pub fn pcb_rf_mitered_bend_fraction(width_mm: f64, height_mm: f64) -> Option<f64> {
    if width_mm <= 0.0 || height_mm <= 0.0 || !width_mm.is_finite() || !height_mm.is_finite() {
        return None;
    }
    let u = width_mm / height_mm;
    let m = 0.52 + 0.65 * (-1.35 * u).exp();
    if m >= 0.5 && m <= 1.0 {
        Some(m)
    } else {
        Some(m.clamp(0.5, 0.9))
    }
}

/// @outputs: 2
/// @domain: vertices.len() [3..1000]
pub fn polygon_shoelace_area_2d(vertices: &[(f64, f64)]) -> Option<f64> {
    let n = vertices.len();
    if n < 3 || n > 1000 {
        return None;
    }
    let mut double_area = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        let (xi, yi) = vertices[i];
        let (xj, yj) = vertices[j];
        if !xi.is_finite() || !yi.is_finite() || !xj.is_finite() || !yj.is_finite() {
            return None;
        }
        double_area += xi * yj - xj * yi;
    }
    let area = (double_area * 0.5).abs();
    if area.is_finite() { Some(area) } else { None }
}
