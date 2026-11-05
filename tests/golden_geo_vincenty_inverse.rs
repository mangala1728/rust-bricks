// 本檔由 cloud_farmer 依 specs/geo_vincenty_inverse.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::geo_vincenty_inverse::*;

#[test]
fn golden_vectors_from_spec() {
    let (dist, bear) = geo_vincenty_inverse(0.0, 0.0, 0.0, 1.0).unwrap();
    assert!((dist - 111319.49).abs() < 1.0, "spec golden vector #1");
    assert!((bear - 90.0).abs() < 1e-3, "spec golden vector #2");
    let (dist0, _) = geo_vincenty_inverse(25.0, 121.0, 25.0, 121.0).unwrap();
    assert!(dist0.abs() < 1e-6, "spec golden vector #3");
    assert!(geo_vincenty_inverse(95.0, 0.0, 0.0, 0.0).is_none(), "spec golden vector #4");
    assert!(geo_vincenty_inverse(f64::NAN, 0.0, 0.0, 0.0).is_none(), "spec golden vector #5");
}
