// ## Map
// Role: core/geo_collision — 手遊虛擬按鍵/點擊命中判定、AABB 碰撞盒與 2D 線段相交幾何微積木。
// Inputs: 2D 點座標、軸對齊邊界盒 (AABB)、線段端點、凸/凹多邊形頂點序列。
// Outputs: 碰撞重疊布林值、相交座標點 Option<(f32, f32)>、點在多邊形內外布林值。
// Invariants: 零外部依賴、無遞迴、零全域可變狀態、浮點數防 NaN/Inf 守衛、頂點數量上限 <= 100、控制流巢狀深度 <= 2。

/// @outputs: 2
/// @domain: min_a_x [-10000.0..10000.0]
pub fn bounding_box_overlap(
    min_a: (f32, f32),
    max_a: (f32, f32),
    min_b: (f32, f32),
    max_b: (f32, f32),
) -> bool {
    if min_a.0 > max_a.0 || min_a.1 > max_a.1 || min_b.0 > max_b.0 || min_b.1 > max_b.1 {
        return false;
    }
    if min_a.0.is_nan() || max_a.0.is_nan() || min_b.0.is_nan() || max_b.0.is_nan() {
        return false;
    }
    if min_a.1.is_nan() || max_a.1.is_nan() || min_b.1.is_nan() || max_b.1.is_nan() {
        return false;
    }
    !(max_a.0 < min_b.0 || min_a.0 > max_b.0 || max_a.1 < min_b.1 || min_a.1 > max_b.1)
}

/// @outputs: 2
/// @domain: p1_x [-10000.0..10000.0]
pub fn segment_intersection_2d(
    p1: (f32, f32),
    p2: (f32, f32),
    p3: (f32, f32),
    p4: (f32, f32),
) -> Option<(f32, f32)> {
    if p1.0.is_nan() || p1.1.is_nan() || p2.0.is_nan() || p2.1.is_nan() {
        return None;
    }
    if p3.0.is_nan() || p3.1.is_nan() || p4.0.is_nan() || p4.1.is_nan() {
        return None;
    }
    let denom = (p2.0 - p1.0) * (p4.1 - p3.1) - (p2.1 - p1.1) * (p4.0 - p3.0);
    if denom.abs() < 1e-7 {
        return None;
    }
    let t_num = (p3.0 - p1.0) * (p4.1 - p3.1) - (p3.1 - p1.1) * (p4.0 - p3.0);
    let u_num = (p3.0 - p1.0) * (p2.1 - p1.1) - (p3.1 - p1.1) * (p2.0 - p1.0);
    let t = t_num / denom;
    let u = u_num / denom;
    if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
        Some((p1.0 + t * (p2.0 - p1.0), p1.1 + t * (p2.1 - p1.1)))
    } else {
        None
    }
}

/// @outputs: 2
/// @domain: px [-10000.0..10000.0]
/// @domain: py [-10000.0..10000.0]
pub fn point_in_polygon_raycast(px: f32, py: f32, vertices: &[(f32, f32)]) -> bool {
    if vertices.len() < 3 || vertices.len() > 100 {
        return false;
    }
    if px < -10000.0 || px > 10000.0 || py < -10000.0 || py > 10000.0 || px.is_nan() || py.is_nan() {
        return false;
    }
    let mut inside = false;
    let n = vertices.len();
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = vertices[i];
        let (xj, yj) = vertices[j];
        if xi.is_nan() || yi.is_nan() || xj.is_nan() || yj.is_nan() {
            return false;
        }
        let edge_crosses = (yi > py) != (yj > py);
        if edge_crosses {
            let intersect_x = (xj - xi) * (py - yi) / (yj - yi) + xi;
            if px < intersect_x {
                inside = !inside;
            }
        }
        j = i;
    }
    inside
}
