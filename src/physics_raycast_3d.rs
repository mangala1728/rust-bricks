// ## Map
// Role: core/physics_raycast_3d — 3D VR 虛擬實境與遊戲引擎之 Möller–Trumbore 射線三角形相交、向量鏡面反射與點面投影微積木。
// Inputs: 射線起點與方向、三角形頂點 (v0, v1, v2)、入射向量與法向量、平面參考點。
// Outputs: 射線-三角形交點座標 Option<(f32, f32, f32)>、反射向量、有號點面距離。
// Invariants: 零外部依賴、零 Heap 分配、無遞迴、零全域狀態、控制流巢狀深度 <= 2、平行射線防除以零除數守衛。

type Vec3 = (f32, f32, f32);

/// @outputs: 1
fn sub(a: Vec3, b: Vec3) -> Vec3 {
    (a.0 - b.0, a.1 - b.1, a.2 - b.2)
}

/// @outputs: 1
fn dot(a: Vec3, b: Vec3) -> f32 {
    a.0 * b.0 + a.1 * b.1 + a.2 * b.2
}

/// @outputs: 1
fn cross(a: Vec3, b: Vec3) -> Vec3 {
    (
        a.1 * b.2 - a.2 * b.1,
        a.2 * b.0 - a.0 * b.2,
        a.0 * b.1 - a.1 * b.0,
    )
}

/// @outputs: 2
pub fn ray_triangle_intersect_moller_trumbore(
    orig: Vec3,
    dir: Vec3,
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
) -> Option<Vec3> {
    if orig.0.is_nan() || dir.0.is_nan() {
        return None;
    }
    let edge1 = sub(v1, v0);
    let edge2 = sub(v2, v0);
    let pvec = cross(dir, edge2);
    let det = dot(edge1, pvec);

    if det.abs() < 1e-7 {
        return None; // 射線與三角形平行
    }
    let inv_det = 1.0 / det;
    let tvec = sub(orig, v0);
    let u = dot(tvec, pvec) * inv_det;
    if u < 0.0 || u > 1.0 {
        return None;
    }

    let qvec = cross(tvec, edge1);
    let v = dot(dir, qvec) * inv_det;
    if v < 0.0 || (u + v) > 1.0 {
        return None;
    }

    let t = dot(edge2, qvec) * inv_det;
    if t < 1e-6 {
        return None; // 交點在射線後方
    }

    let hit = (
        orig.0 + dir.0 * t,
        orig.1 + dir.1 * t,
        orig.2 + dir.2 * t,
    );
    Some(hit)
}

/// @outputs: 1
pub fn vector3_reflect(v: Vec3, normal: Vec3) -> Vec3 {
    if v.0.is_nan() || normal.0.is_nan() {
        return (0.0, 0.0, 0.0);
    }
    let d = dot(v, normal);
    (
        v.0 - 2.0 * d * normal.0,
        v.1 - 2.0 * d * normal.1,
        v.2 - 2.0 * d * normal.2,
    )
}

/// @outputs: 1
pub fn point_plane_distance_signed(pt: Vec3, plane_pt: Vec3, plane_norm: Vec3) -> f32 {
    if pt.0.is_nan() || plane_pt.0.is_nan() || plane_norm.0.is_nan() {
        return 0.0;
    }
    let diff = sub(pt, plane_pt);
    dot(diff, plane_norm)
}
