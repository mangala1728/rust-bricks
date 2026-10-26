//! # CNC G-Code & CAM Toolpath Micro-Bricks (CNC 工具路徑與 G-Code 運動控制微積木)
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 G01 直線插補座標步進、G02/G03 圓弧圓心與徑向角解算、順/逆時針角跨度、
//! 3軸進給率速度分解 (mm/min -> mm/s)、G41/G42 刀具半徑左右補償法向量偏移、
//! 主軸轉速與切削線速度換算、G83 啄鑽循環深度步進、每齒進給量轉進給率、梯形速度輪廓即時速度規劃。

const PI: f64 = std::f64::consts::PI;

/// ## Map: G01 3D 直線插補座標步進
/// @outputs: 1
/// @domain: progress [0.0, 1.0]
pub fn cnc_g01_linear_interpolation_step(start: [f64; 3], end: [f64; 3], progress: f64) -> [f64; 3] {
    if !progress.is_finite() {
        return start;
    }
    for &v in start.iter().chain(end.iter()) {
        if !v.is_finite() { return start; }
    }
    let t = progress.clamp(0.0, 1.0);
    [
        start[0] + (end[0] - start[0]) * t,
        start[1] + (end[1] - start[1]) * t,
        start[2] + (end[2] - start[2]) * t,
    ]
}

/// ## Center: G02/G03 增量式 I/J 偏移量換算圓弧圓心坐標
/// C = (start_x + I, start_y + J)
/// @outputs: 1
/// @domain: i [-10000.0, 10000.0]
/// @domain: j [-10000.0, 10000.0]
pub fn cnc_g02_g03_arc_center_from_ij(start_xy: [f64; 2], i: f64, j: f64) -> [f64; 2] {
    if !start_xy[0].is_finite() || !start_xy[1].is_finite() || !i.is_finite() || !j.is_finite() {
        return [0.0, 0.0];
    }
    [start_xy[0] + i, start_xy[1] + j]
}

/// ## Solve: 依起點、終點與圓心解算圓弧半徑與起訖角度 (弧度 [-PI, PI])
/// 若起點半徑與終點半徑誤差 > 0.05 mm 則判定為無效圓弧 (G-code 幾何畸形)
/// 回傳 (半徑, 起始角, 結束角)
/// @outputs: 1
pub fn cnc_arc_radius_and_angles(
    start_xy: [f64; 2],
    end_xy: [f64; 2],
    center_xy: [f64; 2],
) -> Option<(f64, f64, f64)> {
    let dx_s = start_xy[0] - center_xy[0];
    let dy_s = start_xy[1] - center_xy[1];
    let dx_e = end_xy[0] - center_xy[0];
    let dy_e = end_xy[1] - center_xy[1];
    if !dx_s.is_finite() || !dy_s.is_finite() || !dx_e.is_finite() || !dy_e.is_finite() {
        return None;
    }
    let r_start = (dx_s * dx_s + dy_s * dy_s).sqrt();
    let r_end = (dx_e * dx_e + dy_e * dy_e).sqrt();
    if (r_start - r_end).abs() > 0.05 || r_start < 1e-4 {
        return None;
    }
    let angle_start = dy_s.atan2(dx_s);
    let angle_end = dy_e.atan2(dx_e);
    Some((0.5 * (r_start + r_end), angle_start, angle_end))
}

/// ## Span: G02 順時針 (CW) 或 G03 逆時針 (CCW) 有向圓弧跨越角 (弧度 [0, 2*PI])
/// @outputs: 1
/// @domain: start_angle [-10.0, 10.0]
/// @domain: end_angle [-10.0, 10.0]
pub fn cnc_arc_angular_span(start_angle: f64, end_angle: f64, is_cw: bool) -> f64 {
    if !start_angle.is_finite() || !end_angle.is_finite() {
        return 0.0;
    }
    let two_pi = 2.0 * PI;
    if is_cw {
        // 順時針：角度遞減
        let mut diff = start_angle - end_angle;
        while diff < 0.0 { diff += two_pi; }
        while diff >= two_pi { diff -= two_pi; }
        if diff == 0.0 { two_pi } else { diff }
    } else {
        // 逆時針：角度遞增
        let mut diff = end_angle - start_angle;
        while diff < 0.0 { diff += two_pi; }
        while diff >= two_pi { diff -= two_pi; }
        if diff == 0.0 { two_pi } else { diff }
    }
}

/// ## Decompose: CNC 總進給率 (F, mm/min) 分解為各軸速度向量 (mm/sec)
/// @outputs: 1
/// @domain: dx [-10000.0, 10000.0]
/// @domain: dy [-10000.0, 10000.0]
/// @domain: dz [-10000.0, 10000.0]
/// @domain: feedrate_mm_min [0.001, 100000.0]
pub fn cnc_feedrate_to_axis_velocities(
    dx: f64,
    dy: f64,
    dz: f64,
    feedrate_mm_min: f64,
) -> Option<[f64; 3]> {
    if !dx.is_finite() || !dy.is_finite() || !dz.is_finite() ||
       !feedrate_mm_min.is_finite() || feedrate_mm_min <= 0.0 {
        return None;
    }
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    if dist < 1e-6 {
        return Some([0.0, 0.0, 0.0]);
    }
    let v_mag = feedrate_mm_min / 60.0; // mm/s
    let scale = v_mag / dist;
    Some([dx * scale, dy * scale, dz * scale])
}

/// ## Offset: G41 (左側) / G42 (右側) 刀具半徑補償線段法向量外擴
/// 回傳 (偏移後起點, 偏移後終點)
/// @outputs: 1
/// @domain: tool_radius_mm [0.001, 100.0]
pub fn cnc_cutter_radius_compensation_2d(
    p0: [f64; 2],
    p1: [f64; 2],
    tool_radius_mm: f64,
    is_left_g41: bool,
) -> Option<([f64; 2], [f64; 2])> {
    if !p0[0].is_finite() || !p0[1].is_finite() || !p1[0].is_finite() || !p1[1].is_finite() ||
       !tool_radius_mm.is_finite() || tool_radius_mm <= 0.0 {
        return None;
    }
    let dx = p1[0] - p0[0];
    let dy = p1[1] - p0[1];
    let len = (dx * dx + dy * dy).sqrt();
    if len < 1e-6 {
        return None;
    }
    let inv_len = 1.0 / len;
    let ux = dx * inv_len;
    let uy = dy * inv_len;

    // G41: 向量向左旋轉 90 度 (-uy, ux); G42: 向右旋轉 (uy, -ux)
    let (nx, ny) = if is_left_g41 { (-uy, ux) } else { (uy, -ux) };

    let off_p0 = [p0[0] + nx * tool_radius_mm, p0[1] + ny * tool_radius_mm];
    let off_p1 = [p1[0] + nx * tool_radius_mm, p1[1] + ny * tool_radius_mm];
    Some((off_p0, off_p1))
}

/// ## Calc: 切削線速度 (Vc, m/min) 與刀具直徑換算主軸轉速 (RPM)
/// N = (1000 * Vc) / (PI * D)
/// @outputs: 1
/// @domain: diameter_mm [0.01, 1000.0]
/// @domain: cutting_speed_m_min [0.1, 10000.0]
pub fn cnc_spindle_speed_surface_speed(diameter_mm: f64, cutting_speed_m_min: f64) -> Option<f64> {
    if !diameter_mm.is_finite() || !cutting_speed_m_min.is_finite() ||
       diameter_mm <= 0.0 || cutting_speed_m_min <= 0.0 {
        return None;
    }
    let rpm = (1000.0 * cutting_speed_m_min) / (PI * diameter_mm);
    Some(rpm)
}

/// ## Cycle: G83 啄鑽深孔加工循環步進深度與完工判斷
/// 回傳 (當前啄鑽深度, 是否已鑽透總深度)
/// @outputs: 2
/// @domain: total_depth_mm [0.01, 1000.0]
/// @domain: peck_depth_mm [0.01, 1000.0]
pub fn cnc_drilling_peck_cycles_step(
    total_depth_mm: f64,
    peck_depth_mm: f64,
    current_step: usize,
) -> (f64, bool) {
    if !total_depth_mm.is_finite() || !peck_depth_mm.is_finite() ||
       total_depth_mm <= 0.0 || peck_depth_mm <= 0.0 {
        return (0.0, true);
    }
    let target = ((current_step + 1) as f64) * peck_depth_mm;
    if target >= total_depth_mm {
        (total_depth_mm, true)
    } else {
        (target, false)
    }
}

/// ## Calc: 每齒進給量換算總進給率 (F = RPM * Z * Fz, mm/min)
/// @outputs: 1
/// @domain: spindle_rpm [1.0, 100000.0]
/// @domain: fz_mm_tooth [0.0001, 10.0]
pub fn cnc_feed_per_tooth_to_feedrate(spindle_rpm: f64, num_flutes: usize, fz_mm_tooth: f64) -> f64 {
    if !spindle_rpm.is_finite() || !fz_mm_tooth.is_finite() ||
       spindle_rpm <= 0.0 || num_flutes == 0 || fz_mm_tooth <= 0.0 {
        return 0.0;
    }
    spindle_rpm * (num_flutes as f64) * fz_mm_tooth
}

/// ## Profile: 梯形速度輪廓即時進給速率規劃步進 (加速、巡航、減速)
/// @outputs: 1
/// @domain: current_dist [0.0, 100000.0]
/// @domain: total_dist [0.001, 100000.0]
/// @domain: v_max [0.001, 10000.0]
/// @domain: a_max [0.001, 100000.0]
pub fn cnc_trapezoidal_velocity_profile_step(
    current_dist: f64,
    total_dist: f64,
    v_max: f64,
    a_max: f64,
) -> f64 {
    if !current_dist.is_finite() || !total_dist.is_finite() || !v_max.is_finite() || !a_max.is_finite() ||
       total_dist <= 0.0 || v_max <= 0.0 || a_max <= 0.0 || current_dist < 0.0 {
        return 0.0;
    }
    let s = current_dist.min(total_dist);
    // 加速段速度: v^2 = 2 * a * s
    let v_acc = (2.0 * a_max * s).sqrt();
    // 減速段速度: v^2 = 2 * a * (total_dist - s)
    let rem_dist = (total_dist - s).max(0.0);
    let v_dec = (2.0 * a_max * rem_dist).sqrt();

    v_acc.min(v_dec).min(v_max)
}
