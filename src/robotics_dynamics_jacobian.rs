//! # Robotics Dynamics & Jacobian Micro-Bricks (機器人動力學與雅可比矩陣微積木)
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 2 連桿平面雅可比末端線速度、Yoshikawa 可操縱度橢球指標、雅可比轉置力矩換算 (tau = J^T * F)、
//! 柯氏力與離心力矩、關節空間慣性矩陣 (M 矩陣)、人工勢場關節極限斥力、
//! 前向歐拉動力學狀態積分、計算力矩控制 (Computed Torque Control)、庫侖與黏性關節摩擦力、笛卡爾順應性阻抗彈簧力。

/// ## Map: 2 連桿平面機械臂末端線速度 (Jacobian Forward Velocity: [vx, vy]^T = J * [w1, w2]^T)
/// @outputs: 2
/// @domain: theta1_rad [-10.0, 10.0]
/// @domain: theta2_rad [-10.0, 10.0]
/// @domain: l1 [0.001, 10.0]
/// @domain: l2 [0.001, 10.0]
/// @domain: omega1 [-1000.0, 1000.0]
/// @domain: omega2 [-1000.0, 1000.0]
pub fn robotics_jacobian_2link_planar_vel(
    theta1_rad: f64,
    theta2_rad: f64,
    l1: f64,
    l2: f64,
    omega1: f64,
    omega2: f64,
) -> (f64, f64) {
    if !theta1_rad.is_finite() || !theta2_rad.is_finite() || !l1.is_finite() || !l2.is_finite() ||
       !omega1.is_finite() || !omega2.is_finite() || l1 <= 0.0 || l2 <= 0.0 {
        return (0.0, 0.0);
    }
    let s1 = theta1_rad.sin();
    let c1 = theta1_rad.cos();
    let s12 = (theta1_rad + theta2_rad).sin();
    let c12 = (theta1_rad + theta2_rad).cos();

    // J = [ -l1*s1 - l2*s12,  -l2*s12 ]
    //     [  l1*c1 + l2*c12,   l2*c12 ]
    let j11 = -l1 * s1 - l2 * s12;
    let j12 = -l2 * s12;
    let j21 = l1 * c1 + l2 * c12;
    let j22 = l2 * c12;

    let vx = j11 * omega1 + j12 * omega2;
    let vy = j21 * omega1 + j22 * omega2;
    (vx, vy)
}

/// ## Measure: Yoshikawa 可操縱度指標 (Manipulability Measure: w = sqrt(det(J * J^T)))
/// @outputs: 1
pub fn robotics_manipulability_index_2d(j11: f64, j12: f64, j21: f64, j22: f64) -> f64 {
    if !j11.is_finite() || !j12.is_finite() || !j21.is_finite() || !j22.is_finite() {
        return 0.0;
    }
    // 2x2 矩陣 det(J)
    let det_j = (j11 * j22 - j12 * j21).abs();
    det_j
}

/// ## Solve: 雅可比轉置關節靜態力矩映射 (tau = J^T * F)
/// @outputs: 2
/// @domain: fx [-1e6, 1e6]
/// @domain: fy [-1e6, 1e6]
pub fn robotics_jacobian_transpose_joint_torques_2d(
    j: [[f64; 2]; 2],
    fx: f64,
    fy: f64,
) -> (f64, f64) {
    if !fx.is_finite() || !fy.is_finite() {
        return (0.0, 0.0);
    }
    for row in &j {
        for &val in row {
            if !val.is_finite() { return (0.0, 0.0); }
        }
    }
    // J^T = [ j[0][0], j[1][0] ]
    //       [ j[0][1], j[1][1] ]
    let tau1 = j[0][0] * fx + j[1][0] * fy;
    let tau2 = j[0][1] * fx + j[1][1] * fy;
    (tau1, tau2)
}

/// ## Dyn: 2 連桿平面柯氏力與離心力關節力矩 (Coriolis & Centrifugal Torques)
/// @outputs: 2
/// @domain: theta2_rad [-10.0, 10.0]
/// @domain: omega1 [-1000.0, 1000.0]
/// @domain: omega2 [-1000.0, 1000.0]
/// @domain: m2 [0.001, 1000.0]
/// @domain: l1 [0.001, 10.0]
/// @domain: lc2 [0.001, 10.0]
pub fn robotics_coriolis_centrifugal_torque_2link(
    theta2_rad: f64,
    omega1: f64,
    omega2: f64,
    m2: f64,
    l1: f64,
    lc2: f64,
) -> (f64, f64) {
    if !theta2_rad.is_finite() || !omega1.is_finite() || !omega2.is_finite() ||
       !m2.is_finite() || !l1.is_finite() || !lc2.is_finite() || m2 <= 0.0 || l1 <= 0.0 || lc2 <= 0.0 {
        return (0.0, 0.0);
    }
    let h = -m2 * l1 * lc2 * theta2_rad.sin();
    let tau1 = h * (2.0 * omega1 * omega2 + omega2 * omega2);
    let tau2 = -h * omega1 * omega1;
    (tau1, tau2)
}

/// ## Dyn: 2 連桿對稱慣性矩陣 (Mass/Inertia Matrix: M(q))
/// @outputs: 1
/// @domain: theta2_rad [-10.0, 10.0]
/// @domain: m1 [0.001, 1000.0]
/// @domain: m2 [0.001, 1000.0]
pub fn robotics_inertia_matrix_2link(
    theta2_rad: f64,
    m1: f64,
    m2: f64,
    l1: f64,
    l2: f64,
    lc1: f64,
    lc2: f64,
    i1: f64,
    i2: f64,
) -> [[f64; 2]; 2] {
    if !theta2_rad.is_finite() || m1 <= 0.0 || m2 <= 0.0 || l1 <= 0.0 || l2 <= 0.0 {
        return [[0.0, 0.0], [0.0, 0.0]];
    }
    let c2 = theta2_rad.cos();
    let m11 = i1 + i2 + m1 * lc1 * lc1 + m2 * (l1 * l1 + lc2 * lc2 + 2.0 * l1 * lc2 * c2);
    let m12 = i2 + m2 * (lc2 * lc2 + l1 * lc2 * c2);
    let m22 = i2 + m2 * lc2 * lc2;
    [[m11, m12], [m12, m22]]
}

/// ## Repel: 人工勢場關節極限軟邊界排斥力矩 (Joint Limit Repulsive Potential)
/// @outputs: 1
/// @domain: eta [0.0, 1e6]
/// @domain: dist_threshold [0.001, 10.0]
pub fn robotics_joint_limit_repulsive_potential(
    q: f64,
    q_min: f64,
    q_max: f64,
    eta: f64,
    dist_threshold: f64,
) -> f64 {
    if !q.is_finite() || !q_min.is_finite() || !q_max.is_finite() || !eta.is_finite() || !dist_threshold.is_finite() ||
       q_min >= q_max || dist_threshold <= 0.0 || eta <= 0.0 {
        return 0.0;
    }
    let d_min = q - q_min;
    let d_max = q_max - q;
    if d_min < dist_threshold && d_min > 0.0 {
        eta * (1.0 / d_min - 1.0 / dist_threshold) * (1.0 / (d_min * d_min))
    } else if d_max < dist_threshold && d_max > 0.0 {
        -eta * (1.0 / d_max - 1.0 / dist_threshold) * (1.0 / (d_max * d_max))
    } else {
        0.0
    }
}

/// ## Step: 前向歐拉動力學狀態積分 (Forward Euler Dynamic State Step)
/// @outputs: 2
/// @domain: dt [0.0001, 1.0]
pub fn robotics_forward_euler_dynamics_step(
    q: f64,
    q_dot: f64,
    q_ddot: f64,
    dt: f64,
) -> (f64, f64) {
    if !q.is_finite() || !q_dot.is_finite() || !q_ddot.is_finite() || !dt.is_finite() || dt <= 0.0 {
        return (0.0, 0.0);
    }
    let next_q_dot = q_dot + q_ddot * dt;
    let next_q = q + q_dot * dt;
    (next_q, next_q_dot)
}

/// ## Control: 計算力矩控制虛擬加速度步進 (Computed Torque Control: a_cmd = q_ddot_des + Kp*e + Kd*e_dot)
/// @outputs: 1
/// @domain: kp [0.0, 1e6]
/// @domain: kd [0.0, 1e6]
pub fn robotics_computed_torque_control_step(
    q_des: f64,
    q_dot_des: f64,
    q_ddot_des: f64,
    q_act: f64,
    q_dot_act: f64,
    kp: f64,
    kd: f64,
) -> f64 {
    if !q_des.is_finite() || !q_dot_des.is_finite() || !q_ddot_des.is_finite() ||
       !q_act.is_finite() || !q_dot_act.is_finite() || !kp.is_finite() || !kd.is_finite() {
        return 0.0;
    }
    let err = q_des - q_act;
    let err_dot = q_dot_des - q_dot_act;
    q_ddot_des + kp * err + kd * err_dot
}

/// ## Friction: 關節庫侖摩擦與黏性摩擦力矩模型 (tau_f = Fc * sgn(q_dot) + Fv * q_dot)
/// @outputs: 1
/// @domain: f_coulomb [0.0, 1e6]
/// @domain: f_viscous [0.0, 1e6]
pub fn robotics_friction_coulomb_viscous_step(
    q_dot: f64,
    f_coulomb: f64,
    f_viscous: f64,
) -> f64 {
    if !q_dot.is_finite() || !f_coulomb.is_finite() || !f_viscous.is_finite() ||
       f_coulomb < 0.0 || f_viscous < 0.0 {
        return 0.0;
    }
    let sgn = if q_dot > 1e-4 {
        1.0
    } else if q_dot < -1e-4 {
        -1.0
    } else {
        q_dot / 1e-4 // 零速鄰域平滑飽和避免數值抖振
    };
    f_coulomb * sgn + f_viscous * q_dot
}

/// ## Compliance: 笛卡爾空間順應性阻抗虛擬彈簧力 (Cartesian Compliance Spring Force: F = K * (p_des - p_act))
/// @outputs: 1
pub fn robotics_cartesian_compliance_spring_force(
    p_des: [f64; 3],
    p_act: [f64; 3],
    stiffness_k: [f64; 3],
) -> [f64; 3] {
    for i in 0..3 {
        if !p_des[i].is_finite() || !p_act[i].is_finite() || !stiffness_k[i].is_finite() || stiffness_k[i] < 0.0 {
            return [0.0, 0.0, 0.0];
        }
    }
    [
        stiffness_k[0] * (p_des[0] - p_act[0]),
        stiffness_k[1] * (p_des[1] - p_act[1]),
        stiffness_k[2] * (p_des[2] - p_act[2]),
    ]
}
