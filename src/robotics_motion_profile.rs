// ## Map
// Role: robot/motion_profile — 機器人逆向/正向運動學 (2-Link IK/FK)、雅可比奇異點、S-curve 躍度、步進馬達微步正余弦驅動微積木。
// Inputs: 目標座標 (x, y)、連桿長度、關節角度、加載力矩、梯形/S曲線時序、導程與齒輪比。
// Outputs: 關節角 (rad)、末端座標、雅可比行列式值、即時位移、Q15 微步電流比、線性導程速度 (mm/s)。
// Invariants: 零外部依賴、純 Rust 實現、除零剛性防護、無溢位、單一函式 <= 60 行。

/// @outputs: 2
/// @domain: l1 [0.001..1000.0]
/// @domain: l2 [0.001..1000.0]
pub fn robotics_planar_2link_ik(
    target_x: f64,
    target_y: f64,
    l1: f64,
    l2: f64,
    elbow_up: bool,
) -> Option<(f64, f64)> {
    if l1 <= 0.0 || l2 <= 0.0 || !target_x.is_finite() || !target_y.is_finite() {
        return None;
    }
    let r_sq = target_x * target_x + target_y * target_y;
    let r = r_sq.sqrt();
    if r > (l1 + l2) || r < (l1 - l2).abs() || r <= 1e-12 {
        return None; // 工作空間之外或奇異原點
    }
    let cos_theta2 = (r_sq - l1 * l1 - l2 * l2) / (2.0 * l1 * l2);
    let cos_theta2_clamped = cos_theta2.clamp(-1.0, 1.0);
    let sin_theta2_mag = (1.0 - cos_theta2_clamped * cos_theta2_clamped).max(0.0).sqrt();
    let theta2 = if elbow_up { -sin_theta2_mag.atan2(cos_theta2_clamped) } else { sin_theta2_mag.atan2(cos_theta2_clamped) };

    let k1 = l1 + l2 * cos_theta2_clamped;
    let k2 = l2 * theta2.sin();
    let theta1 = target_y.atan2(target_x) - k2.atan2(k1);
    Some((theta1, theta2))
}

/// @outputs: 2
/// @domain: l1 [0.001..1000.0]
/// @domain: l2 [0.001..1000.0]
pub fn robotics_planar_2link_fk(
    theta1_rad: f64,
    theta2_rad: f64,
    l1: f64,
    l2: f64,
) -> Option<(f64, f64)> {
    if l1 <= 0.0 || l2 <= 0.0 || !theta1_rad.is_finite() || !theta2_rad.is_finite() {
        return None;
    }
    let x = l1 * theta1_rad.cos() + l2 * (theta1_rad + theta2_rad).cos();
    let y = l1 * theta1_rad.sin() + l2 * (theta1_rad + theta2_rad).sin();
    Some((x, y))
}

/// @outputs: 1
/// @domain: l1 [0.001..1000.0]
/// @domain: l2 [0.001..1000.0]
pub fn robotics_planar_2link_jacobian_det(theta2_rad: f64, l1: f64, l2: f64) -> f64 {
    if l1 <= 0.0 || l2 <= 0.0 || !theta2_rad.is_finite() {
        return 0.0;
    }
    // 2-link 雅可比矩陣行列式 det(J) = l1 * l2 * sin(theta2)
    l1 * l2 * theta2_rad.sin()
}

/// @outputs: 1
/// @domain: v_max [0.001..10000.0]
/// @domain: a_max [0.001..10000.0]
pub fn robotics_trapezoidal_profile_distance(
    time_s: f64,
    v_max: f64,
    a_max: f64,
    total_dist: f64,
) -> Option<f64> {
    if time_s < 0.0 || v_max <= 0.0 || a_max <= 0.0 || total_dist <= 0.0 || !time_s.is_finite() {
        return None;
    }
    let t_acc = v_max / a_max;
    let d_acc = 0.5 * a_max * t_acc * t_acc;
    if 2.0 * d_acc >= total_dist {
        // 三角形運動輪廓
        let t_peak = (total_dist / a_max).sqrt();
        if time_s <= t_peak {
            Some(0.5 * a_max * time_s * time_s)
        } else if time_s <= 2.0 * t_peak {
            let dt = time_s - t_peak;
            let v_peak = a_max * t_peak;
            Some(0.5 * total_dist + v_peak * dt - 0.5 * a_max * dt * dt)
        } else {
            Some(total_dist)
        }
    } else {
        // 標準梯形運動輪廓
        let d_cruise = total_dist - 2.0 * d_acc;
        let t_cruise = d_cruise / v_max;
        let t_total = 2.0 * t_acc + t_cruise;
        if time_s <= t_acc {
            Some(0.5 * a_max * time_s * time_s)
        } else if time_s <= (t_acc + t_cruise) {
            Some(d_acc + v_max * (time_s - t_acc))
        } else if time_s <= t_total {
            let dt = time_s - (t_acc + t_cruise);
            Some(d_acc + d_cruise + v_max * dt - 0.5 * a_max * dt * dt)
        } else {
            Some(total_dist)
        }
    }
}

/// @outputs: 1
/// @domain: jerk [0.001..1.0e6]
pub fn robotics_scurve_jerk_velocity(time_s: f64, jerk: f64) -> Option<f64> {
    if time_s < 0.0 || jerk <= 0.0 || !time_s.is_finite() || !jerk.is_finite() {
        return None;
    }
    // S曲線 第一階段恆定加加速度：v(t) = 0.5 * j * t^2
    Some(0.5 * jerk * time_s * time_s)
}

/// @outputs: 2
/// @domain: microsteps_per_step [1..256]
pub fn robotics_stepper_microstep_sin_cos(
    step_index: u16,
    microsteps_per_step: u16,
) -> Option<(i16, i16)> {
    if microsteps_per_step == 0 {
        return None;
    }
    let total_steps = 4 * (microsteps_per_step as usize);
    let idx = (step_index as usize) % total_steps;
    let angle_rad = 2.0 * std::f64::consts::PI * (idx as f64) / (total_steps as f64);
    let sin_val = (angle_rad.sin() * 32767.0).round().clamp(-32768.0, 32767.0) as i16;
    let cos_val = (angle_rad.cos() * 32767.0).round().clamp(-32768.0, 32767.0) as i16;
    Some((sin_val, cos_val))
}

/// @outputs: 2
pub fn robotics_joint_soft_limit_clamp(
    angle_rad: f64,
    min_rad: f64,
    max_rad: f64,
    margin_rad: f64,
) -> (f64, bool) {
    if min_rad >= max_rad || margin_rad < 0.0 || !angle_rad.is_finite() {
        return (angle_rad, true);
    }
    let clamped = angle_rad.clamp(min_rad, max_rad);
    let in_warning_zone = (clamped - min_rad) < margin_rad || (max_rad - clamped) < margin_rad;
    (clamped, in_warning_zone)
}

/// @outputs: 1
/// @domain: pitch_mm [0.001..1000.0]
/// @domain: gear_ratio [0.001..1000.0]
pub fn robotics_lead_screw_linear_distance(
    revolutions: f64,
    pitch_mm: f64,
    gear_ratio: f64,
) -> Option<f64> {
    if pitch_mm <= 0.0 || gear_ratio <= 0.0 || !revolutions.is_finite() {
        return None;
    }
    Some((revolutions * pitch_mm) / gear_ratio)
}

/// @outputs: 1
/// @domain: arm_length_m [0.001..100.0]
/// @domain: payload_kg [0.0..10000.0]
pub fn robotics_arm_payload_torque(
    arm_length_m: f64,
    payload_kg: f64,
    gravity_accel: f64,
    joint_accel_rad_s2: f64,
) -> Option<f64> {
    if arm_length_m <= 0.0 || payload_kg < 0.0 || !gravity_accel.is_finite() || !joint_accel_rad_s2.is_finite() {
        return None;
    }
    // 力矩 = 重力力矩 (水平最差情況) + 轉動慣量力矩 (I = m * L^2)
    let static_torque = payload_kg * gravity_accel * arm_length_m;
    let inertia = payload_kg * arm_length_m * arm_length_m;
    let dynamic_torque = inertia * joint_accel_rad_s2;
    Some(static_torque + dynamic_torque)
}

/// @outputs: 1
/// @domain: lead_mm [0.001..1000.0]
pub fn robotics_scara_z_axis_feed_rate(lead_mm: f64, motor_rpm: f64) -> Option<f64> {
    if lead_mm <= 0.0 || motor_rpm < 0.0 || !motor_rpm.is_finite() {
        return None;
    }
    // 速度 (mm/s) = (RPM / 60) * 導程
    Some((motor_rpm / 60.0) * lead_mm)
}
