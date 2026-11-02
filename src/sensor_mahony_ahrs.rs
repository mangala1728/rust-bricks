// ## Map
// Role: Mahony 6‑DOF AHRS core – pure‑Rust, zero external dependencies.
// Inputs:  q (unit quaternion), gyro_rps (rad/s), accel_g (g), integral_err (bias accumulator), kp, ki, dt.
// Outputs:  gravity error (3), updated quaternion (4) wrapped in Option.
// Invariants:  q is normalized, dt ∈ [0.0001, 1.0], kp, ki ∈ [0.0, 100.0], integral_err components stay in [‑1.0, 1.0].

/// @outputs: 3
pub fn mahony_compute_gravity_error(
    q: &[f64; 4],
    accel_g: (f64, f64, f64),
) -> (f64, f64, f64) {
    // Normalise accelerometer measurement
    let (ax, ay, az) = accel_g;
    let norm = (ax * ax + ay * ay + az * az).sqrt();
    if norm < 1e-6 {
        return (0.0, 0.0, 0.0);
    }
    let (ax, ay, az) = (ax / norm, ay / norm, az / norm);

    // Estimated direction of gravity (from quaternion)
    let q0 = q[0];
    let q1 = q[1];
    let q2 = q[2];
    let q3 = q[3];
    let vx = 2.0 * (q1 * q3 - q0 * q2);
    let vy = 2.0 * (q0 * q1 + q2 * q3);
    let vz = q0 * q0 - q1 * q1 - q2 * q2 + q3 * q3;

    // Cross‑product between measured and estimated gravity vectors
    let ex = ay * vz - az * vy;
    let ey = az * vx - ax * vz;
    let ez = ax * vy - ay * vx;
    (ex, ey, ez)
}

/// @outputs: 4
/// @domain: dt [0.0001..1.0]
/// @domain: kp [0.0..100.0]
/// @domain: ki [0.0..100.0]
pub fn mahony_ahrs_update_6dof(
    q: &[f64; 4],
    gyro_rps: (f64, f64, f64),
    accel_g: (f64, f64, f64),
    integral_err: &mut (f64, f64, f64),
    kp: f64,
    ki: f64,
    dt: f64,
) -> Option<[f64; 4]> {
    // Guard: parameter domains
    if dt < 0.0001 || dt > 1.0 || kp < 0.0 || kp > 100.0 || ki < 0.0 || ki > 100.0 {
        return None;
    }

    // 1️⃣ Compute gravity error
    let (ex, ey, ez) = mahony_compute_gravity_error(q, accel_g);

    // 2️⃣ Integral error accumulation with clamping
    integral_err.0 = (integral_err.0 + ex * ki * dt).clamp(-1.0, 1.0);
    integral_err.1 = (integral_err.1 + ey * ki * dt).clamp(-1.0, 1.0);
    integral_err.2 = (integral_err.2 + ez * ki * dt).clamp(-1.0, 1.0);

    // 3️⃣ Apply proportional‑integral correction to gyro
    let (gx, gy, gz) = gyro_rps;
    let wx = gx + kp * ex + integral_err.0;
    let wy = gy + kp * ey + integral_err.1;
    let wz = gz + kp * ez + integral_err.2;

    // 4️⃣ Quaternion derivative (q̇ = ½ * q ⊗ ω)
    let q0 = q[0];
    let q1 = q[1];
    let q2 = q[2];
    let q3 = q[3];
    let dq0 = -0.5 * (q1 * wx + q2 * wy + q3 * wz);
    let dq1 =  0.5 * (q0 * wx + q2 * wz - q3 * wy);
    let dq2 =  0.5 * (q0 * wy - q1 * wz + q3 * wx);
    let dq3 =  0.5 * (q0 * wz + q1 * wy - q2 * wx);

    // 5️⃣ Integrate and renormalise
    let mut nq0 = q0 + dq0 * dt;
    let mut nq1 = q1 + dq1 * dt;
    let mut nq2 = q2 + dq2 * dt;
    let mut nq3 = q3 + dq3 * dt;
    let norm = (nq0 * nq0 + nq1 * nq1 + nq2 * nq2 + nq3 * nq3).sqrt();
    nq0 /= norm;
    nq1 /= norm;
    nq2 /= norm;
    nq3 /= norm;

    Some([nq0, nq1, nq2, nq3])
}
