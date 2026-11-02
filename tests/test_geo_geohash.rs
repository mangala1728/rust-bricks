use rust_bricks::geo_geohash::*;

#[test]
fn encode_origin_4bits() {
    // Golden vector 1: (0.0, 0.0) → 0b1100 = 12
    let result = geo_geohash_encode_bits(0.0, 0.0, 4);
    assert_eq!(result, Some(0b1100));
}

#[test]
fn encode_decode_taipei_10bits() {
    // Golden vector 2: Taipei (25.0330, 121.5654) with 10‑bit precision
    let lat = 25.0330_f64;
    let lon = 121.5654_f64;
    let bits = 10_u8;

    let hash = geo_geohash_encode_bits(lat, lon, bits).expect("encoding must succeed");
    let bounds = geo_geohash_decode_bounds(hash, bits).expect("decoding must succeed");

    // The original point must lie inside the returned bounding box
    let ((lat_min, lat_max), (lon_min, lon_max)) = bounds;
    assert!(lat >= lat_min && lat <= lat_max, "lat out of bounds");
    assert!(lon >= lon_min && lon <= lon_max, "lon out of bounds");
}
