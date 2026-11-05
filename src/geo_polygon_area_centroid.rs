// ## Map
// Role: 2D polygon shoelace signed area and centroid coordinates
// Inputs: vertices: &[(f64, f64)]
// Outputs: Option<f64> or Option<(f64, f64)>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: vertices.len() [3..100000]
pub fn geo_polygon_signed_area(vertices: &[(f64, f64)]) -> Option<f64> {
    let n = vertices.len();
    if n < 3 || n > 100_000 {
        return None;
    }
    let mut sum = 0.0;
    for i in 0..n {
        let (x0, y0) = vertices[i];
        let (x1, y1) = vertices[(i + 1) % n];
        if !x0.is_finite() || !y0.is_finite() || !x1.is_finite() || !y1.is_finite() {
            return None;
        }
        sum += x0 * y1 - x1 * y0;
    }
    Some(0.5 * sum)
}

/// @outputs: 2
/// @domain: vertices.len() [3..100000]
pub fn geo_polygon_centroid(vertices: &[(f64, f64)]) -> Option<(f64, f64)> {
    let area = geo_polygon_signed_area(vertices)?;
    if area.abs() <= 1e-12 {
        return None;
    }
    let n = vertices.len();
    let mut cx_sum = 0.0;
    let mut cy_sum = 0.0;

    for i in 0..n {
        let (x0, y0) = vertices[i];
        let (x1, y1) = vertices[(i + 1) % n];
        let cross = x0 * y1 - x1 * y0;
        cx_sum += (x0 + x1) * cross;
        cy_sum += (y0 + y1) * cross;
    }
    let factor = 1.0 / (6.0 * area);
    Some((cx_sum * factor, cy_sum * factor))
}
