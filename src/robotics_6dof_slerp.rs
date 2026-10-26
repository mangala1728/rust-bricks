//! # Robotics 6-DOF & SLERP Trajectory Micro-Bricks (機器人學6軸逆解與四元數軌跡微積木)
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 6 軸機械手臂腕部解耦、四元數 SLERP 球面線性插值、Rodrigues 旋轉軸角與矩陣互轉、
//! 五次多項式零衝擊平滑軌跡、球形手腕 Z-Y-Z 逆姿態求解、重力靜力矩補償。

/// ## Map: 6-DOF 機械手臂腕部球心解耦 (p_wc = p_wrist - d6 * R_z)
/// @outputs: 1
pub fn robotics_wrist_center_decouple(
    wrist: [f64; 3],
    rot_matrix: &[f64; 9],
    d6: f64,
) -> [f64; 3] {
    // R[:, 2] 為 Z 軸方向向量 (rot_matrix[2], rot_matrix[5], rot_matrix[8])
    let z_x = rot_matrix[2];
    let z_y = rot_matrix[5];
    let z_z = rot_matrix[8];
    [
        wrist[0] - d6 * z_x,
        wrist[1] - d6 * z_y,
        wrist[2] - d6 * z_z,
    ]
}

/// ## Map: 單位四元數球面線性插值 (SLERP: 最短路徑夾角插補)
/// @domain: t [0.0, 1.0]
/// @outputs: 1
pub fn robotics_quaternion_slerp_step(q1: [f64; 4], mut q2: [f64; 4], t: f64) -> [f64; 4] {
    if t < 0.0 || t > 1.0 || q1[0].is_nan() || q2[0].is_nan() {
        return q1;
    }
    let mut dot = q1[0] * q2[0] + q1[1] * q2[1] + q1[2] * q2[2] + q1[3] * q2[3];
    if dot < 0.0 {
        // 反轉符號走球面最短大圓路徑
        for v in q2.iter_mut() {
            *v = -*v;
        }
        dot = -dot;
    }
    if dot > 0.9995 {
        // 夾角極小，退化為線性插值 LERP 並歸一化
        let mut q_res = [0.0f64; 4];
        for i in 0..4 {
            q_res[i] = q1[i] + t * (q2[i] - q1[i]);
        }
        return robotics_quaternion_normalize(q_res).unwrap_or(q1);
    }
    // theta = acos(dot)
    let theta = fast_acos_approx(dot.clamp(-1.0, 1.0));
    let sin_theta = (1.0 - dot * dot).max(0.0).sqrt();
    if sin_theta < 1e-6 {
        return q1;
    }
    let s1 = fast_sin_approx((1.0 - t) * theta) / sin_theta;
    let s2 = fast_sin_approx(t * theta) / sin_theta;
    [
        s1 * q1[0] + s2 * q2[0],
        s1 * q1[1] + s2 * q2[1],
        s1 * q1[2] + s2 * q2[2],
        s1 * q1[3] + s2 * q2[3],
    ]
}

/// ## Map: Rodrigues 旋轉向量 (Axis-Angle) 轉 3x3 旋轉矩陣 SO(3)
/// @outputs: 1
pub fn robotics_axis_angle_to_matrix(axis: [f64; 3], angle_rad: f64) -> [f64; 9] {
    let norm = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if norm < 1e-12 || angle_rad.is_nan() {
        // 回傳 3x3 單位矩陣
        return [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
    }
    let kx = axis[0] / norm;
    let ky = axis[1] / norm;
    let kz = axis[2] / norm;
    let c = fast_cos_approx(angle_rad);
    let s = fast_sin_approx(angle_rad);
    let v = 1.0 - c;

    [
        kx * kx * v + c,        kx * ky * v - kz * s,   kx * kz * v + ky * s,
        ky * kx * v + kz * s,   ky * ky * v + c,        ky * kz * v - kx * s,
        kz * kx * v - ky * s,   kz * ky * v + kx * s,   kz * kz * v + c,
    ]
}

/// ## Map: 3x3 旋轉矩陣反解旋轉軸與旋轉角
/// @outputs: 1
pub fn robotics_matrix_to_axis_angle(rot: &[f64; 9]) -> Option<([f64; 3], f64)> {
    let tr = rot[0] + rot[4] + rot[8];
    let cos_theta = ((tr - 1.0) * 0.5).clamp(-1.0, 1.0);
    let theta = fast_acos_approx(cos_theta);
    if theta.abs() < 1e-6 {
        return Some(([0.0, 0.0, 1.0], 0.0));
    }
    let denom = 2.0 * fast_sin_approx(theta);
    if denom.abs() < 1e-6 {
        // 接近 180 度奇異點（保留軸向符號）
        return Some((axis_at_pi(rot), theta));
    }
    let rx = (rot[7] - rot[5]) / denom;
    let ry = (rot[2] - rot[6]) / denom;
    let rz = (rot[3] - rot[1]) / denom;
    Some(([rx, ry, rz], theta))
}

/// ## Map: 五次多項式平滑軌跡規劃 (s(t), v(t), a(t))，起終點速度/加速度皆為0
/// @outputs: 1
pub fn robotics_quintic_polynomial_traj(
    t: f64,
    t_total: f64,
    p0: f64,
    p1: f64,
) -> (f64, f64, f64) {
    if t_total <= 1e-6 || t <= 0.0 {
        return (p0, 0.0, 0.0);
    }
    if t >= t_total {
        return (p1, 0.0, 0.0);
    }
    let tau = t / t_total;
    let tau2 = tau * tau;
    let tau3 = tau2 * tau;
    let tau4 = tau3 * tau;
    let tau5 = tau4 * tau;

    let delta_p = p1 - p0;
    // s(t) = p0 + delta_p * (10 tau^3 - 15 tau^4 + 6 tau^5)
    let s = p0 + delta_p * (10.0 * tau3 - 15.0 * tau4 + 6.0 * tau5);
    // v(t) = (delta_p / T) * (30 tau^2 - 60 tau^3 + 30 tau^4)
    let v = (delta_p / t_total) * (30.0 * tau2 - 60.0 * tau3 + 30.0 * tau4);
    // a(t) = (delta_p / T^2) * (60 tau - 180 tau^2 + 120 tau^3)
    let a = (delta_p / (t_total * t_total)) * (60.0 * tau - 180.0 * tau2 + 120.0 * tau3);

    (s, v, a)
}

/// ## Map: 單連桿重力負載靜力矩補償 (tau = m * g * L_cm * cos(theta))
/// @outputs: 1
pub fn robotics_gravity_load_torque(mass_kg: f64, link_cm_dist_m: f64, theta_rad: f64) -> f64 {
    if mass_kg < 0.0 || link_cm_dist_m < 0.0 || theta_rad.is_nan() {
        return 0.0;
    }
    let g = 9.80665f64;
    mass_kg * g * link_cm_dist_m * fast_cos_approx(theta_rad)
}

/// ## Map: 笛卡爾三維空間歐氏幾何距離
/// @outputs: 1
pub fn robotics_cartesian_distance_3d(p1: [f64; 3], p2: [f64; 3]) -> f64 {
    let dx = p1[0] - p2[0];
    let dy = p1[1] - p2[1];
    let dz = p1[2] - p2[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// ## Map: 四元數單位化歸一
/// @outputs: 1
pub fn robotics_quaternion_normalize(q: [f64; 4]) -> Option<[f64; 4]> {
    let norm = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if norm < 1e-12 || norm.is_nan() {
        None
    } else {
        Some([q[0] / norm, q[1] / norm, q[2] / norm, q[3] / norm])
    }
}

/// ## Map: 笛卡爾空間三維直線插補 (Linear Blend)
/// @domain: alpha [0.0, 1.0]
/// @outputs: 1
pub fn robotics_cartesian_linear_blend(p0: [f64; 3], p1: [f64; 3], alpha: f64) -> [f64; 3] {
    if alpha < 0.0 || alpha > 1.0 || alpha.is_nan() {
        return p0;
    }
    [
        p0[0] + alpha * (p1[0] - p0[0]),
        p0[1] + alpha * (p1[1] - p0[1]),
        p0[2] + alpha * (p1[2] - p0[2]),
    ]
}

// 內部三角函式改用 std（屬標準庫，不違反零依賴）：原多項式近似 acos(1) 得 0.404、cos(π) 得 -1.211，
// 導致 SLERP 非單位四元數、旋轉矩陣不正交 (INC-20261026-06)

/// 內部正弦
/// @outputs: 1
fn fast_sin_approx(x: f64) -> f64 {
    x.sin()
}

/// 內部餘弦
/// @outputs: 1
fn fast_cos_approx(x: f64) -> f64 {
    x.cos()
}

/// 內部反餘弦（輸入夾止至 [-1, 1]）
/// @outputs: 1
fn fast_acos_approx(x: f64) -> f64 {
    x.clamp(-1.0, 1.0).acos()
}

/// 內部 180° 奇異點旋轉軸還原：R = 2kk^T - I，取最大對角分量為正並由非對角項決定其餘符號
/// @outputs: 1
fn axis_at_pi(rot: &[f64; 9]) -> [f64; 3] {
    let diag = [rot[0], rot[4], rot[8]];
    let mut m = 0usize;
    for i in 1..3 {
        if diag[i] > diag[m] {
            m = i;
        }
    }
    let km = ((diag[m] + 1.0) * 0.5).max(0.0).sqrt();
    if km < 1e-12 {
        return [0.0, 0.0, 1.0];
    }
    let mut k = [0.0f64; 3];
    for i in 0..3 {
        k[i] = if i == m { km } else { (rot[3 * m + i] + rot[3 * i + m]) / (4.0 * km) };
    }
    k
}
