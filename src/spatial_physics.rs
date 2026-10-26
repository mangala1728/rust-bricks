// ## Map
// Role: core/spatial_physics — 3D VR/AR 空間運算、四元數旋轉、射線拾取與彈簧物理微積木。
// Inputs: 四元數 (w, x, y, z)、3D 向量 (x, y, z)、射線起點與方向、球體中心與半徑、彈簧係數與步長。
// Outputs: 四元數哈密頓乘積、旋轉後 3D 向量、射線與球體最近碰撞距離 Option<f32>、彈簧阻尼積分狀態 (pos, vel)。
// Invariants: 零外部依賴、無遞迴、零全域可變狀態、浮點數防 NaN/Inf 守衛、控制流巢狀深度 <= 2。

/// @outputs: 1
/// @domain: q1_w [-1000.0..1000.0]
pub fn quaternion_multiply(
    q1: (f32, f32, f32, f32),
    q2: (f32, f32, f32, f32),
) -> (f32, f32, f32, f32) {
    if q1.0.is_nan() || q1.1.is_nan() || q1.2.is_nan() || q1.3.is_nan() {
        return (1.0, 0.0, 0.0, 0.0);
    }
    if q2.0.is_nan() || q2.1.is_nan() || q2.2.is_nan() || q2.3.is_nan() {
        return (1.0, 0.0, 0.0, 0.0);
    }
    let (w1, x1, y1, z1) = q1;
    let (w2, x2, y2, z2) = q2;
    let w = w1 * w2 - x1 * x2 - y1 * y2 - z1 * z2;
    let x = w1 * x2 + x1 * w2 + y1 * z2 - z1 * y2;
    let y = w1 * y2 - x1 * z2 + y1 * w2 + z1 * x2;
    let z = w1 * z2 + x1 * y2 - y1 * x2 + z1 * w2;
    (w, x, y, z)
}

/// @outputs: 1
/// @domain: q_w [-1000.0..1000.0]
pub fn quaternion_rotate_vector(
    q: (f32, f32, f32, f32),
    v: (f32, f32, f32),
) -> (f32, f32, f32) {
    if q.0.is_nan() || q.1.is_nan() || q.2.is_nan() || q.3.is_nan() {
        return v;
    }
    if v.0.is_nan() || v.1.is_nan() || v.2.is_nan() {
        return (0.0, 0.0, 0.0);
    }
    let (w, qx, qy, qz) = q;
    let (vx, vy, vz) = v;
    let dot_uv = qx * vx + qy * vy + qz * vz;
    let dot_uu = qx * qx + qy * qy + qz * qz;
    let cross_x = qy * vz - qz * vy;
    let cross_y = qz * vx - qx * vz;
    let cross_z = qx * vy - qy * vx;

    let term1_factor = 2.0 * dot_uv;
    let term2_factor = w * w - dot_uu;
    let term3_factor = 2.0 * w;

    let rx = term1_factor * qx + term2_factor * vx + term3_factor * cross_x;
    let ry = term1_factor * qy + term2_factor * vy + term3_factor * cross_y;
    let rz = term1_factor * qz + term2_factor * vz + term3_factor * cross_z;
    (rx, ry, rz)
}

/// @outputs: 2
/// @domain: radius [0.0..10000.0]
pub fn ray_sphere_intersect(
    origin: (f32, f32, f32),
    dir: (f32, f32, f32),
    sphere_center: (f32, f32, f32),
    radius: f32,
) -> Option<f32> {
    if radius <= 0.0 || radius > 10000.0 || radius.is_nan() {
        return None;
    }
    let dir_sq = dir.0 * dir.0 + dir.1 * dir.1 + dir.2 * dir.2;
    if dir_sq < 1e-8 || dir_sq.is_nan() {
        return None;
    }
    let inv_len = 1.0 / dir_sq.sqrt();
    let dx = dir.0 * inv_len;
    let dy = dir.1 * inv_len;
    let dz = dir.2 * inv_len;

    let oc_x = origin.0 - sphere_center.0;
    let oc_y = origin.1 - sphere_center.1;
    let oc_z = origin.2 - sphere_center.2;

    let b = oc_x * dx + oc_y * dy + oc_z * dz;
    let c = (oc_x * oc_x + oc_y * oc_y + oc_z * oc_z) - radius * radius;
    let disc = b * b - c;
    if disc < 0.0 {
        return None;
    }
    let disc_sqrt = disc.sqrt();
    let t0 = -b - disc_sqrt;
    let t1 = -b + disc_sqrt;
    if t0 > 0.0 {
        Some(t0)
    } else if t1 > 0.0 {
        Some(t1)
    } else {
        None
    }
}

/// @outputs: 1
/// @domain: dt [0.0..1.0]
/// @domain: k_spring [0.0..10000.0]
/// @domain: c_damping [0.0..1000.0]
pub fn spring_damper_step(
    pos: f32,
    vel: f32,
    target: f32,
    k_spring: f32,
    c_damping: f32,
    dt: f32,
) -> (f32, f32) {
    if dt <= 0.0 || dt > 1.0 || dt.is_nan() {
        return (pos, vel);
    }
    if k_spring < 0.0 || k_spring > 10000.0 || c_damping < 0.0 || c_damping > 1000.0 {
        return (pos, vel);
    }
    if pos.is_nan() || vel.is_nan() || target.is_nan() {
        return (0.0, 0.0);
    }
    let force = -k_spring * (pos - target) - c_damping * vel;
    let new_vel = (vel + force * dt).clamp(-1000.0, 1000.0);
    let new_pos = (pos + new_vel * dt).clamp(-10000.0, 10000.0);
    (new_pos, new_vel)
}
