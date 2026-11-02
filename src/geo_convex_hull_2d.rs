// ## Map
// Role: 2D geometry primitive – convex‑hull construction.
// Inputs: Vec<(f64,f64)> – a set of planar points.
// Outputs: Option<Vec<(f64,f64)>> – vertices of the convex hull in CCW order, or None.
// Invariants: all coordinates lie inside the closed interval [MIN..MAX] (no NaN).

/// Numerical limits used for all runtime guards.
const MIN: f64 = -1.0e9;
const MAX: f64 =  1.0e9;

/// @outputs: 1
/// @domain: p0.x [MIN..MAX]
/// @domain: p0.y [MIN..MAX]
/// @domain: p1.x [MIN..MAX]
/// @domain: p1.y [MIN..MAX]
/// @domain: p2.x [MIN..MAX]
/// @domain: p2.y [MIN..MAX]
///
/// Returns the signed 2‑D cross product of vectors **p0→p1** and **p0→p2**.
/// Positive → left turn, negative → right turn, zero → collinear.
pub fn cross_product_2d(p0: (f64, f64), p1: (f64, f64), p2: (f64, f64)) -> f64 {
    // runtime guard – any out‑of‑range coordinate yields a neutral result (0.0)
    if p0.0 < MIN || p0.0 > MAX ||
       p0.1 < MIN || p0.1 > MAX ||
       p1.0 < MIN || p1.0 > MAX ||
       p1.1 < MIN || p1.1 > MAX ||
       p2.0 < MIN || p2.0 > MAX ||
       p2.1 < MIN || p2.1 > MAX {
        return 0.0;
    }

    let ux = p1.0 - p0.0;
    let uy = p1.1 - p0.1;
    let vx = p2.0 - p0.0;
    let vy = p2.1 - p0.1;
    ux * vy - uy * vx
}

/// @outputs: 1
/// @domain: points[i].x [MIN..MAX] (checked for every point)
/// @domain: points[i].y [MIN..MAX] (checked for every point)
///
/// Computes the convex hull of a set of 2‑D points using Andrew’s monotone chain.
/// Returns `None` for an empty input, otherwise a vector of hull vertices in
/// counter‑clockwise order (first vertex is not repeated at the end).
pub fn convex_hull_2d(mut points: Vec<(f64, f64)>) -> Option<Vec<(f64, f64)>> {
    // Guard: reject empty input early.
    if points.is_empty() {
        return None;
    }

    // Guard: reject any point outside the allowed domain.
    for &(x, y) in &points {
        if x < MIN || x > MAX || y < MIN || y > MAX {
            return None;
        }
    }

    // Trivial cases – 1 or 2 distinct points are already a hull.
    if points.len() < 3 {
        return Some(points);
    }

    // Sort by x, then by y (total ordering, no unwrap needed).
    points.sort_by(|a, b| {
        a.0.total_cmp(&b.0)
            .then_with(|| a.1.total_cmp(&b.1))
    });

    // Lower hull.
    let mut lower: Vec<(f64, f64)> = Vec::new();
    for p in &points {
        while lower.len() >= 2 {
            let k = lower.len();
            let cross = cross_product_2d(lower[k - 2], lower[k - 1], *p);
            if cross > 0.0 { break; }
            lower.pop();
        }
        lower.push(*p);
    }

    // Upper hull.
    let mut upper: Vec<(f64, f64)> = Vec::new();
    for p in points.iter().rev() {
        while upper.len() >= 2 {
            let k = upper.len();
            let cross = cross_product_2d(upper[k - 2], upper[k - 1], *p);
            if cross > 0.0 { break; }
            upper.pop();
        }
        upper.push(*p);
    }

    // Concatenate lower and upper, removing duplicate end points.
    lower.pop();
    upper.pop();
    lower.extend(upper);
    Some(lower)
}
