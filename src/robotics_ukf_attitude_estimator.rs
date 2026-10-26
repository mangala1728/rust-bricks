//! # 機器人、航太飛行力學與姿態估算微積木 (Robotics & Flight Attitude Estimation)
//!
//! 提供 Unscented Transform (UKF) 權重解算、Cholesky 協方差 1D 步進、
//! 四元數誤差狀態乘法 (MEKF)、四元數機體/慣性座標向量旋轉、
//! 空氣動力學動壓與馬赫數、X型四軸飛行器推力力矩分配、
//! 傾斜補償磁羅盤偏航角、Madgwick 重力梯度步進、無刷馬達推力轉 PWM 脈寬。
//! 零外部依賴，符合 NASA JPL 規範與合約 09。

/// ## Map: Scaled Unscented Transform (UKF) 權重係數解算 (UKF Van der Merwe Weights)
/// 回傳 (Wm0, Wc0, Wi) 代表均值零階權重、協方差零階權重與其餘 2n 個 Sigma 點權重
/// @outputs: 1
/// @domain: n >= 1 && n <= 32 && alpha > 0.0 && alpha <= 1.0 && beta >= 0.0 && kappa >= 0.0
pub fn ukf_van_der_merwe_weights(
    n: usize,
    alpha: f64,
    beta: f64,
    kappa: f64,
) -> Option<(f64, f64, f64)> {
    if n == 0 || n > 32 || alpha <= 0.0 || alpha > 1.0 || beta < 0.0 || kappa < 0.0 {
        return None;
    }
    let nf = n as f64;
    let lambda = alpha * alpha * (nf + kappa) - nf;
    let denom = nf + lambda;
    if denom.abs() < 1e-12 {
        return None;
    }
    let wm0 = lambda / denom;
    let wc0 = wm0 + (1.0 - alpha * alpha + beta);
    let wi = 1.0 / (2.0 * denom);
    Some((wm0, wc0, wi))
}

/// ## Calc: 1D 協方差矩陣 Cholesky 根 Sigma 點擴展偏移 (UKF Sigma Point 1D Spread)
/// @outputs: 1
/// @domain: variance >= 0.0 && c_scale > 0.0
pub fn ukf_sigma_point_1d_spread(variance: f64, c_scale: f64) -> Option<f64> {
    if variance < 0.0 || c_scale <= 0.0 {
        return None;
    }
    let sigma = variance.sqrt();
    Some(c_scale * sigma)
}

/// ## Calc: 四元數小角度誤差狀態乘法步進 (MEKF Quaternion Error State Step)
/// q_nominal = [w, x, y, z], delta_theta = [d_roll, d_pitch, d_yaw]
/// 回傳更新後並正規化之四元數 [w, x, y, z]
/// @outputs: 1
/// @domain: delta_theta 角度範數 <= 1.0 rad
pub fn quaternion_error_state_step(
    q_nominal: [f64; 4],
    delta_theta: [f64; 3],
) -> Option<[f64; 4]> {
    let dth_sq = delta_theta[0] * delta_theta[0]
        + delta_theta[1] * delta_theta[1]
        + delta_theta[2] * delta_theta[2];
    if dth_sq > 1.0 {
        return None;
    }
    // 小角度旋轉四元數: dq = [1, 0.5*dth_x, 0.5*dth_y, 0.5*dth_z]
    let dq = [
        1.0,
        0.5 * delta_theta[0],
        0.5 * delta_theta[1],
        0.5 * delta_theta[2],
    ];
    let (w1, x1, y1, z1) = (q_nominal[0], q_nominal[1], q_nominal[2], q_nominal[3]);
    let (w2, x2, y2, z2) = (dq[0], dq[1], dq[2], dq[3]);

    let w = w1 * w2 - x1 * x2 - y1 * y2 - z1 * z2;
    let x = w1 * x2 + x1 * w2 + y1 * z2 - z1 * y2;
    let y = w1 * y2 - x1 * z2 + y1 * w2 + z1 * x2;
    let z = w1 * z2 + x1 * y2 - y1 * x2 + z1 * w2;

    let norm_sq = w * w + x * x + y * y + z * z;
    if norm_sq < 1e-12 {
        return None;
    }
    let inv_norm = 1.0 / norm_sq.sqrt();
    Some([w * inv_norm, x * inv_norm, y * inv_norm, z * inv_norm])
}

/// ## Calc: 四元數機體座標至慣性座標向量旋轉 (Quaternion Rotate Body to Inertial)
/// v_inertial = q * v_body * q*
/// @outputs: 1
/// @domain: q 必須為非零四元數
pub fn quaternion_rotate_vector_body_to_inertial(q: [f64; 4], v: [f64; 3]) -> [f64; 3] {
    let (w, qx, qy, qz) = (q[0], q[1], q[2], q[3]);
    // t = 2 * cross(q_vec, v)
    let tx = 2.0 * (qy * v[2] - qz * v[1]);
    let ty = 2.0 * (qz * v[0] - qx * v[2]);
    let tz = 2.0 * (qx * v[1] - qy * v[0]);

    // v' = v + w * t + cross(q_vec, t)
    let rx = v[0] + w * tx + (qy * tz - qz * ty);
    let ry = v[1] + w * ty + (qz * tx - qx * tz);
    let rz = v[2] + w * tz + (qx * ty - qy * tx);
    [rx, ry, rz]
}

/// ## Calc: 空氣動力學動壓計算 (Aerodynamic Dynamic Pressure Q)
/// q = 0.5 * rho * v^2 (單位: Pascal)
/// @outputs: 1
/// @domain: rho >= 0.0 && rho <= 2.0 && airspeed >= 0.0 && airspeed <= 2000.0
pub fn flight_dynamic_pressure(rho: f64, airspeed: f64) -> Option<f64> {
    if rho < 0.0 || rho > 2.0 || airspeed < 0.0 || airspeed > 2000.0 {
        return None;
    }
    Some(0.5 * rho * airspeed * airspeed)
}

/// ## Calc: 根據空速與氣溫解算飛行馬赫數 (Flight Mach Number from Airspeed)
/// a = sqrt(gamma * R * T), Mach = v / a, 其中 gamma=1.4, R=287.05 J/(kg*K)
/// @outputs: 1
/// @domain: airspeed >= 0.0 && temp_kelvin >= 150.0 && temp_kelvin <= 350.0
pub fn flight_mach_number(airspeed: f64, temp_kelvin: f64) -> Option<f64> {
    if airspeed < 0.0 || temp_kelvin < 150.0 || temp_kelvin > 350.0 {
        return None;
    }
    let speed_of_sound = (1.4 * 287.05 * temp_kelvin).sqrt();
    if speed_of_sound <= 0.0 {
        return None;
    }
    Some(airspeed / speed_of_sound)
}

/// ## Calc: X 型四軸飛行器推力力矩分配 (Quadrotor X4 Thrust-Torque Mixing)
/// 輸入: 總推力 total_thrust (N), 滾轉力矩 tau_roll (N*m), 俯仰力矩 tau_pitch (N*m), 偏航力矩 tau_yaw (N*m)
/// 回傳 4 個馬達推力 [f1, f2, f3, f4]，且推力低於 0 時夾止為 0.0
/// @outputs: 1
/// @domain: total_thrust >= 0.0
pub fn quadrotor_thrust_torque_allocation_x4(
    total_thrust: f64,
    tau_roll: f64,
    tau_pitch: f64,
    tau_yaw: f64,
) -> [f64; 4] {
    if total_thrust < 0.0 {
        return [0.0, 0.0, 0.0, 0.0];
    }
    let t_base = total_thrust * 0.25;
    // X型四軸配置推力混控:
    // 馬達 1 (Front-Right, CCW): +T - Roll + Pitch - Yaw
    // 馬達 2 (Rear-Left,   CCW): +T + Roll - Pitch - Yaw
    // 馬達 3 (Front-Left,  CW):  +T + Roll + Pitch + Yaw
    // 馬達 4 (Rear-Right,  CW):  +T - Roll - Pitch + Yaw
    let f1 = (t_base - 0.5 * tau_roll + 0.5 * tau_pitch - 0.25 * tau_yaw).max(0.0);
    let f2 = (t_base + 0.5 * tau_roll - 0.5 * tau_pitch - 0.25 * tau_yaw).max(0.0);
    let f3 = (t_base + 0.5 * tau_roll + 0.5 * tau_pitch + 0.25 * tau_yaw).max(0.0);
    let f4 = (t_base - 0.5 * tau_roll - 0.5 * tau_pitch + 0.25 * tau_yaw).max(0.0);
    [f1, f2, f3, f4]
}

/// ## Calc: 傾斜補償電子羅盤偏航角 (Tilt Compensated Compass Yaw)
/// 依據姿態角 roll, pitch 將三軸磁力計讀數投影至水平面，輸出航向角 yaw (弧度 -PI..PI)
/// @outputs: 1
/// @domain: roll, pitch, mag_xyz 為有限數值
pub fn magnetometer_tilt_compensated_yaw(
    roll_rad: f64,
    pitch_rad: f64,
    mag: [f64; 3],
) -> Option<f64> {
    if !roll_rad.is_finite() || !pitch_rad.is_finite() || !mag[0].is_finite() || !mag[1].is_finite() || !mag[2].is_finite() {
        return None;
    }
    let cr = roll_rad.cos();
    let sr = roll_rad.sin();
    let cp = pitch_rad.cos();
    let sp = pitch_rad.sin();

    // 水平面磁場分量投影
    let x_h = mag[0] * cp + mag[2] * sp;
    let y_h = mag[0] * sr * sp + mag[1] * cr - mag[2] * sr * cp;

    if x_h.abs() < 1e-12 && y_h.abs() < 1e-12 {
        return None;
    }
    Some((-y_h).atan2(x_h))
}

/// ## Calc: Madgwick 姿態濾波重力誤差梯度步進 (Madgwick Gravity Gradient Step)
/// 輸入當前四元數 q = [w, x, y, z] 與正規化加速度計 [ax, ay, az]
/// 回傳目標函數梯度 Jacobian 向量 [grad_w, grad_x, grad_y, grad_z]
/// @outputs: 1
/// @domain: q 範數 > 0 且 a 範數 > 0
pub fn madgwick_gravity_gradient_step(q: [f64; 4], a: [f64; 3]) -> Option<[f64; 4]> {
    let (w, x, y, z) = (q[0], q[1], q[2], q[3]);
    let norm_a_sq = a[0] * a[0] + a[1] * a[1] + a[2] * a[2];
    if norm_a_sq < 1e-12 {
        return None;
    }
    let inv_a = 1.0 / norm_a_sq.sqrt();
    let (ax, ay, az) = (a[0] * inv_a, a[1] * inv_a, a[2] * inv_a);

    // 誤差函數 f_g(q) = 2 * [x*z - w*y, w*x + y*z, 0.5 - x^2 - y^2] - a
    let f1 = 2.0 * (x * z - w * y) - ax;
    let f2 = 2.0 * (w * x + y * z) - ay;
    let f3 = 2.0 * (0.5 - x * x - y * y) - az;

    // 梯度 J_g^T * f_g
    let gw = -2.0 * y * f1 + 2.0 * x * f2;
    let gx = 2.0 * z * f1 + 2.0 * w * f2 - 4.0 * x * f3;
    let gy = -2.0 * w * f1 + 2.0 * z * f2 - 4.0 * y * f3;
    let gz = 2.0 * x * f1 + 2.0 * y * f2;

    Some([gw, gx, gy, gz])
}

/// ## Calc: 無刷馬達推力比例轉 ESC PWM 脈衝微秒 (Brushless Motor Thrust to PWM Us)
/// 推力比例 ratio in [0.0, 1.0]，利用空氣動力推力二次方關係 T = k * omega^2 反求速度比例
/// 回傳 1000..2000 微秒之標準遙控 PWM 脈寬
/// @outputs: 1
/// @domain: thrust_ratio >= 0.0 && thrust_ratio <= 1.0
pub fn motor_thrust_ratio_to_pwm_us(thrust_ratio: f64) -> u16 {
    if thrust_ratio <= 0.0 {
        return 1000;
    }
    if thrust_ratio >= 1.0 {
        return 2000;
    }
    // 轉速比例 omega_ratio = sqrt(T_ratio)
    let speed_ratio = thrust_ratio.sqrt();
    1000 + (speed_ratio * 1000.0).round() as u16
}
