//! # 剛體物理引擎接觸動力學與 GJK/EPA 碰撞演算法 (Rigid Body Dynamics & GJK/EPA Collision)
//!
//! 提供三維凸多面體 GJK (Gilbert-Johnson-Keerthi) 支撐映射、單純形搜尋方向、
//! EPA (Expanding Polytope Algorithm) 穿透法向量與深度、接觸點相對速度、
//! Baumgarte 穿透穩定化脈衝、庫侖摩擦錐夾止與剛體旋轉衝量微積木。
//! 零外部依賴，符合 NASA JPL 規範與合約 09。

/// ## Map: 幾何球體之 GJK 支撐映射點 (Support Mapping: Sphere)
/// @outputs: 1
/// @domain: radius [0.0..=1e6]
pub fn gjk_support_sphere(center: [f32; 3], radius: f32, direction: [f32; 3]) -> Option<[f32; 3]> {
    if !radius.is_finite() || radius < 0.0 {
        return None;
    }
    let d_sq = direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2];
    if !d_sq.is_finite() || d_sq <= 1e-12 {
        return Some(center);
    }
    let inv_d = radius / d_sq.sqrt();
    Some([
        center[0] + direction[0] * inv_d,
        center[1] + direction[1] * inv_d,
        center[2] + direction[2] * inv_d,
    ])
}

/// ## Calc: 幾何方塊 (AABB/OBB) 之 GJK 支撐映射點 (Support Mapping: Box)
/// @outputs: 1
/// @domain: half_extents[i] >= 0.0
pub fn gjk_support_box(half_extents: [f32; 3], direction: [f32; 3]) -> Option<[f32; 3]> {
    if half_extents[0] < 0.0 || half_extents[1] < 0.0 || half_extents[2] < 0.0 {
        return None;
    }
    if !half_extents[0].is_finite() || !half_extents[1].is_finite() || !half_extents[2].is_finite() {
        return None;
    }
    let sx = if direction[0] >= 0.0 { half_extents[0] } else { -half_extents[0] };
    let sy = if direction[1] >= 0.0 { half_extents[1] } else { -half_extents[1] };
    let sz = if direction[2] >= 0.0 { half_extents[2] } else { -half_extents[2] };
    Some([sx, sy, sz])
}

/// ## Step: 閔可夫斯基差集支撐點映射 (Minkowski Difference Support Point)
/// @outputs: 1
/// @domain: sup_a 與 sup_b 元素皆為有限實數
pub fn gjk_minkowski_support_point(sup_a: [f32; 3], sup_b: [f32; 3]) -> [f32; 3] {
    if !sup_a[0].is_finite() || !sup_b[0].is_finite() {
        return [0.0; 3];
    }
    [
        sup_a[0] - sup_b[0],
        sup_a[1] - sup_b[1],
        sup_a[2] - sup_b[2],
    ]
}

/// ## Calc: GJK 2點單純形線段步進 (Simplex Line Step)
/// 回傳 (新原點法向搜尋方向, 保留頂點數)
/// @outputs: 1
/// @domain: a, b 有限實數
pub fn gjk_simplex_line_step(a: [f32; 3], b: [f32; 3]) -> ([f32; 3], usize) {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ao = [-a[0], -a[1], -a[2]];
    let dot_ab_ao = ab[0] * ao[0] + ab[1] * ao[1] + ab[2] * ao[2];
    let dot_ab_ab = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];

    if dot_ab_ab <= 1e-12 {
        return (ao, 1);
    }
    // 方向垂直於 AB 且朝向原點: (AB x AO) x AB
    // 或投影法向量: d = AO - AB * (AB . AO / AB . AB)
    let proj = dot_ab_ao / dot_ab_ab;
    let perp = [ao[0] - ab[0] * proj, ao[1] - ab[1] * proj, ao[2] - ab[2] * proj];
    let p_sq = perp[0] * perp[0] + perp[1] * perp[1] + perp[2] * perp[2];
    if p_sq <= 1e-12 {
        ([0.0; 3], 2)
    } else {
        (perp, 2)
    }
}

/// ## Calc: GJK 3點單純形三角形法向搜尋方向 (Simplex Triangle Normal Direction)
/// @outputs: 1
/// @domain: a, b, c 有限實數
pub fn gjk_simplex_triangle_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> Option<[f32; 3]> {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let norm = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let n_sq = norm[0] * norm[0] + norm[1] * norm[1] + norm[2] * norm[2];
    if !n_sq.is_finite() || n_sq <= 1e-12 {
        return None;
    }
    // 原點向量 AO
    let ao = [-a[0], -a[1], -a[2]];
    let dot = norm[0] * ao[0] + norm[1] * ao[1] + norm[2] * ao[2];
    if dot >= 0.0 {
        Some(norm)
    } else {
        Some([-norm[0], -norm[1], -norm[2]])
    }
}

/// ## Calc: EPA 三角形面外法向量與原點距離 (EPA Facet Normal & Distance)
/// 回傳 (單位外法向量, 原點到多面體面之有向距離)
/// @outputs: 1
/// @domain: a, b, c 不共線且頂點有限
pub fn epa_triangle_normal_and_distance(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> Option<([f32; 3], f32)> {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let len = (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();
    if !len.is_finite() || len <= 1e-8 {
        return None;
    }
    let n = [cross[0] / len, cross[1] / len, cross[2] / len];
    let dist = n[0] * a[0] + n[1] * a[1] + n[2] * a[2];
    Some((n, dist))
}

/// ## Calc: 接觸點剛體相對表面速度 (Contact Point Relative Velocity)
/// @outputs: 1
/// @domain: 線速度與角速度輸入均為有限實數
pub fn physics_contact_relative_velocity(
    va: [f32; 3],
    wa: [f32; 3],
    ra: [f32; 3],
    vb: [f32; 3],
    wb: [f32; 3],
    rb: [f32; 3],
) -> [f32; 3] {
    // v_contact_A = va + wa x ra
    let vca = [
        va[0] + (wa[1] * ra[2] - wa[2] * ra[1]),
        va[1] + (wa[2] * ra[0] - wa[0] * ra[2]),
        va[2] + (wa[0] * ra[1] - wa[1] * ra[0]),
    ];
    // v_contact_B = vb + wb x rb
    let vcb = [
        vb[0] + (wb[1] * rb[2] - wb[2] * rb[1]),
        vb[1] + (wb[2] * rb[0] - wb[0] * rb[2]),
        vb[2] + (wb[0] * rb[1] - wb[1] * rb[0]),
    ];
    // v_rel = vca - vcb
    [vca[0] - vcb[0], vca[1] - vcb[1], vca[2] - vcb[2]]
}

/// ## Calc: 法向碰撞恢復衝量計算 (Collision Normal Impulse with Baumgarte)
/// @outputs: 1
/// @domain: inv_mass_sum > 0.0 && restitution >= 0.0 && restitution <= 1.0
pub fn physics_contact_normal_impulse(
    v_rel_normal: f32,
    inv_mass_sum: f32,
    restitution: f32,
    penetration: f32,
    baumgarte_bias: f32,
) -> f32 {
    if !inv_mass_sum.is_finite() || inv_mass_sum <= 1e-8 {
        return 0.0;
    }
    let rest = restitution.clamp(0.0, 1.0);
    let pen_bias = if penetration > 0.0 { penetration * baumgarte_bias } else { 0.0 };
    // 衝量 J = -( (1 + e)*v_rel + bias ) / inv_mass_sum
    let j = -((1.0 + rest) * v_rel_normal - pen_bias) / inv_mass_sum;
    if j < 0.0 { 0.0 } else { j }
}

/// ## Calc: 庫侖摩擦錐夾止切向衝量 (Coulomb Friction Tangent Impulse Clamping)
/// @outputs: 1
/// @domain: normal_impulse >= 0.0 && friction_coeff >= 0.0
pub fn physics_coulomb_friction_clamp(
    normal_impulse: f32,
    friction_coeff: f32,
    desired_tangent_impulse: f32,
) -> f32 {
    if normal_impulse <= 0.0 || friction_coeff <= 0.0 || !desired_tangent_impulse.is_finite() {
        return 0.0;
    }
    let max_friction = normal_impulse * friction_coeff;
    desired_tangent_impulse.clamp(-max_friction, max_friction)
}

/// ## Calc: 剛體衝量產生之角速度增量 (Rigid Body Angular Velocity Delta from Impulse)
/// 計算 Delta_omega = I^(-1) * (r x J)
/// @outputs: 1
/// @domain: inv_inertia_diag[i] >= 0.0
pub fn physics_rigid_body_angular_impulse(
    impulse: [f32; 3],
    r: [f32; 3],
    inv_inertia_diag: [f32; 3],
) -> [f32; 3] {
    if inv_inertia_diag[0] < 0.0 || inv_inertia_diag[1] < 0.0 || inv_inertia_diag[2] < 0.0 {
        return [0.0; 3];
    }
    // torque = r x impulse
    let tx = r[1] * impulse[2] - r[2] * impulse[1];
    let ty = r[2] * impulse[0] - r[0] * impulse[2];
    let tz = r[0] * impulse[1] - r[1] * impulse[0];

    [tx * inv_inertia_diag[0], ty * inv_inertia_diag[1], tz * inv_inertia_diag[2]]
}
