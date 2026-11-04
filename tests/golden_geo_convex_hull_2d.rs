// 本檔由 cloud_farmer 依 specs/geo_convex_hull_2d.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::geo_convex_hull_2d::*;

#[test]
fn golden_vectors_from_spec() {
    let h = convex_hull_2d(vec![(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0), (2.0, 2.0)]).unwrap();
    assert!(h.len() == 4, "spec golden vector #1");
    assert!(h.contains(&(0.0, 0.0)), "spec golden vector #2");
    assert!(h.contains(&(4.0, 0.0)), "spec golden vector #3");
    assert!(h.contains(&(4.0, 4.0)), "spec golden vector #4");
    assert!(h.contains(&(0.0, 4.0)), "spec golden vector #5");
    assert!(!h.contains(&(2.0, 2.0)), "spec golden vector #6");
    assert!((cross_product_2d((0.0, 0.0), (1.0, 0.0), (0.0, 1.0)) - 1.0).abs() < 1e-12, "spec golden vector #7");
    assert!((cross_product_2d((0.0, 0.0), (0.0, 1.0), (1.0, 0.0)) + 1.0).abs() < 1e-12, "spec golden vector #8");
    assert!(cross_product_2d((0.0, 0.0), (1.0, 1.0), (2.0, 2.0)).abs() < 1e-12, "spec golden vector #9");
}
