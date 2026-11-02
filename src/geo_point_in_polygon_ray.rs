// ## Map
// Role:   point‑in‑polygon (ray‑casting)
// Inputs: point: (f64, f64), polygon: &[(f64, f64)]
// Outputs: Option<bool> – Some(true) if inside, Some(false) if outside, None on error
// Invariants:
//   - polygon must contain at least 3 vertices
//   - all coordinates must be finite numbers (no NaN / ±∞)
//   - algorithm follows the classic even‑odd rule

/// @outputs: 1
/// @domain: px [-1.0e308..1.0e308]
/// @domain: py [-1.0e308..1.0e308]
pub fn point_in_polygon_ray(
    point: (f64, f64),
    polygon: &[(f64, f64)],
) -> Option<bool> {
    // ---- domain guards -------------------------------------------------------
    let (px, py) = point;
    if !px.is_finite() || !py.is_finite() { return None; }

    // ---- basic validation ----------------------------------------------------
    if polygon.len() < 3 { return None; }

    // ---- ensure every vertex is a finite number -------------------------------
    for &(vx, vy) in polygon.iter() {
        if !vx.is_finite() || !vy.is_finite() { return None; }
    }

    // ---- ray‑casting core ----------------------------------------------------
    let mut inside = false;
    let n = polygon.len();

    for i in 0..n {
        let (xi, yi) = polygon[i];
        let (xj, yj) = polygon[(i + 1) % n];

        // does the horizontal ray at py cross the edge (xi,yi)-(xj,yj) ?
        let intersect_y = (yi > py) != (yj > py);
        if intersect_y {
            // compute x coordinate of intersection
            let x_inter = (xj - xi) * (py - yi) / (yj - yi) + xi;
            if px < x_inter {
                inside = !inside; // toggle parity
            }
        }
    }

    Some(inside)
}
