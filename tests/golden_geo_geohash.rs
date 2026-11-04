// 本檔由 cloud_farmer 依 specs/geo_geohash.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::geo_geohash::*;

#[test]
fn golden_vectors_from_spec() {
    assert!(geo_geohash_encode_bits(0.0, 0.0, 4) == Some(12), "spec golden vector #1");
    assert!(geo_geohash_decode_bounds(12, 4) == Some(((0.0, 45.0), (0.0, 90.0))), "spec golden vector #2");
    let h = geo_geohash_encode_bits(25.0330, 121.5654, 10).unwrap();
    let ((lat_min, lat_max), (lon_min, lon_max)) = geo_geohash_decode_bounds(h, 10).unwrap();
    assert!(lat_min <= 25.0330 && 25.0330 <= lat_max, "spec golden vector #3");
    assert!(lon_min <= 121.5654 && 121.5654 <= lon_max, "spec golden vector #4");
    assert!(geo_geohash_encode_bits(0.0, 0.0, 0).is_none(), "spec golden vector #5");
    assert!(geo_geohash_encode_bits(0.0, 0.0, 65).is_none(), "spec golden vector #6");
    assert!(geo_geohash_encode_bits(90.5, 0.0, 10).is_none(), "spec golden vector #7");
    assert!(geo_geohash_encode_bits(0.0, -180.5, 10).is_none(), "spec golden vector #8");
    assert!(geo_geohash_encode_bits(f64::NAN, 0.0, 10).is_none(), "spec golden vector #9");
    assert!(geo_geohash_decode_bounds(12, 0).is_none(), "spec golden vector #10");
    assert!(geo_geohash_decode_bounds(12, 65).is_none(), "spec golden vector #11");
}
