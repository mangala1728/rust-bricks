// ## Map
// Role: cad/geom_model — KiCAD PCB DFM 製程校驗、IPC-2152 走線溫升、FreeCAD 幾何實體計算、OpenEMS 網格與 Mur 吸收邊界微積木。
// Inputs: 鑽孔徑、焊盤直徑、走線截面積 (mils²)、電流 (A)、圓角半徑、3D 包圍盒、網格頻率 (Hz) 與介電常數。
// Outputs: 焊盤環寬 (Annular Ring)、溫升 (°C)、弦長、體積、3D 包含判定、Mur 邊界係數、FDTD 網格步長 (m)。
// Invariants: 零外部依賴、純 Rust 實現、除零剛性防護、無溢位、單一函式 <= 60 行。

/// @outputs: 1
/// @domain: drill_mm [0.1..20.0]
/// @domain: pad_diameter_mm [0.2..30.0]
pub fn kicad_via_annular_ring_width_mm(drill_mm: f64, pad_diameter_mm: f64) -> Option<f64> {
    if drill_mm <= 0.0 || pad_diameter_mm <= drill_mm || !drill_mm.is_finite() || !pad_diameter_mm.is_finite() {
        return None;
    }
    Some((pad_diameter_mm - drill_mm) / 2.0)
}

/// @outputs: 1
/// @domain: min_clearance_mm [0.01..50.0]
pub fn kicad_trace_clearance_violation(
    p1: (f64, f64),
    p2: (f64, f64),
    min_clearance_mm: f64,
) -> bool {
    if min_clearance_mm <= 0.0 || !min_clearance_mm.is_finite()
        || !p1.0.is_finite() || !p1.1.is_finite() || !p2.0.is_finite() || !p2.1.is_finite() {
        return true; // 異常輸入預設報告違規防護
    }
    let dx = p1.0 - p2.0;
    let dy = p1.1 - p2.1;
    let dist_sq = dx * dx + dy * dy;
    let min_sq = min_clearance_mm * min_clearance_mm;
    dist_sq < min_sq
}

/// @outputs: 1
/// @domain: current_amps [0.01..50.0]
/// @domain: area_mils_sq [1.0..10000.0]
pub fn ipc2152_trace_temp_rise_celsius(
    current_amps: f64,
    area_mils_sq: f64,
    is_internal: bool,
) -> Option<f64> {
    if current_amps <= 0.0 || area_mils_sq <= 0.0 || !current_amps.is_finite() || !area_mils_sq.is_finite() {
        return None;
    }
    // IPC-2152 經典簡化擬合參數：I = k * (ΔT)^0.44 * A^0.725
    let k = if is_internal { 0.024 } else { 0.048 };
    let area_factor = area_mils_sq.powf(0.725);
    let denom = k * area_factor;
    if denom <= 1e-12 {
        return None;
    }
    let delta_t_ratio = current_amps / denom;
    Some(delta_t_ratio.powf(1.0 / 0.44))
}

/// @outputs: 1
/// @domain: radius_mm [0.1..10000.0]
/// @domain: angle_deg [0.1..360.0]
pub fn freecad_fillet_chord_length_mm(radius_mm: f64, angle_deg: f64) -> Option<f64> {
    if radius_mm <= 0.0 || angle_deg <= 0.0 || angle_deg > 360.0 || !radius_mm.is_finite() || !angle_deg.is_finite() {
        return None;
    }
    let pi = std::f64::consts::PI;
    let rad = (angle_deg * pi) / 180.0;
    Some(2.0 * radius_mm * (rad / 2.0).sin())
}

/// @outputs: 1
/// @domain: radius_mm [0.01..10000.0]
/// @domain: height_mm [0.01..10000.0]
pub fn freecad_extrude_cylinder_volume_mm3(radius_mm: f64, height_mm: f64) -> Option<f64> {
    if radius_mm <= 0.0 || height_mm <= 0.0 || !radius_mm.is_finite() || !height_mm.is_finite() {
        return None;
    }
    let pi = std::f64::consts::PI;
    Some(pi * radius_mm * radius_mm * height_mm)
}

/// @outputs: 1
pub fn freecad_bounding_box_3d_containment(
    min_c: (f64, f64, f64),
    max_c: (f64, f64, f64),
    p: (f64, f64, f64),
) -> bool {
    if !min_c.0.is_finite() || !min_c.1.is_finite() || !min_c.2.is_finite()
        || !max_c.0.is_finite() || !max_c.1.is_finite() || !max_c.2.is_finite()
        || !p.0.is_finite() || !p.1.is_finite() || !p.2.is_finite()
        || min_c.0 > max_c.0 || min_c.1 > max_c.1 || min_c.2 > max_c.2 {
        return false;
    }
    p.0 >= min_c.0 && p.0 <= max_c.0
        && p.1 >= min_c.1 && p.1 <= max_c.1
        && p.2 >= min_c.2 && p.2 <= max_c.2
}

/// @outputs: 1
/// @domain: c_m_s [1.0e6..3.0e8]
/// @domain: dt_s [1.0e-15..1.0e-6]
/// @domain: dx_m [1.0e-6..1.0]
pub fn openems_mur_absorbing_boundary_coeff(c_m_s: f64, dt_s: f64, dx_m: f64) -> Option<f64> {
    if c_m_s <= 0.0 || dt_s <= 0.0 || dx_m <= 0.0
        || !c_m_s.is_finite() || !dt_s.is_finite() || !dx_m.is_finite() {
        return None;
    }
    let c_dt = c_m_s * dt_s;
    let denom = c_dt + dx_m;
    if denom <= 1e-15 {
        return None;
    }
    Some((c_dt - dx_m) / denom)
}

/// @outputs: 1
/// @domain: max_freq_hz [1.0e6..100.0e9]
/// @domain: eps_r [1.0..100.0]
/// @domain: cells_per_wavelength [5..100]
pub fn openems_mesh_nyquist_cell_size_m(
    max_freq_hz: f64,
    eps_r: f64,
    cells_per_wavelength: u16,
) -> Option<f64> {
    if max_freq_hz <= 0.0 || eps_r < 1.0 || cells_per_wavelength < 2
        || !max_freq_hz.is_finite() || !eps_r.is_finite() {
        return None;
    }
    let c = 299_792_458.0;
    let lambda_min = c / (max_freq_hz * eps_r.sqrt());
    Some(lambda_min / (cells_per_wavelength as f64))
}

/// @outputs: 2
/// @domain: radius [0.01..10000.0]
pub fn svg_path_arc_bounding_box(
    center: (f64, f64),
    radius: f64,
) -> Option<((f64, f64), (f64, f64))> {
    if radius <= 0.0 || !radius.is_finite() || !center.0.is_finite() || !center.1.is_finite() {
        return None;
    }
    let min_pt = (center.0 - radius, center.1 - radius);
    let max_pt = (center.0 + radius, center.1 + radius);
    Some((min_pt, max_pt))
}

/// @outputs: 1
/// @domain: mass > 0.0 && stiffness > 0.0 && damping >= 0.0
pub fn remotion_spring_damping_ratio(mass: f64, stiffness: f64, damping: f64) -> Option<f64> {
    if mass <= 0.0 || stiffness <= 0.0 || damping < 0.0
        || !mass.is_finite() || !stiffness.is_finite() || !damping.is_finite() {
        return None;
    }
    // 阻尼比 zeta = c / (2 * sqrt(m * k))
    let critical_damping = 2.0 * (mass * stiffness).sqrt();
    if critical_damping <= 1e-12 {
        return None;
    }
    Some(damping / critical_damping)
}
