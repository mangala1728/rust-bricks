use rust_bricks::geo_haversine_destination::*;

#[test]
fn golden_vector_zero_distance() {
    let result = geo_destination_point(0.0, 0.0, 0.0, 0.0).expect("valid input");
    let (lat, lon) = result;
    assert!((lat - 0.0).abs() < 1e-12);
    assert!((lon - 0.0).abs() < 1e-12);
}

#[test]
fn golden_vector_one_degree_east() {
    // 1 degree of longitude at the equator ≈ 111 194.9266 m
    let distance = 6371000.0 * std::f64::consts::PI / 180.0;
    let result = geo_destination_point(0.0, 0.0, 90.0, distance).expect("valid input");
    let (lat, lon) = result;
    assert!((lat - 0.0).abs() < 1e-4, "lat diff: {}", (lat - 0.0).abs());
    assert!((lon - 1.0).abs() < 1e-3, "lon diff: {}", (lon - 1.0).abs());
}
