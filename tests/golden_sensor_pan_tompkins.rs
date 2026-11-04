// 本檔由 cloud_farmer 依 specs/sensor_pan_tompkins.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::sensor_pan_tompkins::*;

#[test]
fn golden_vectors_from_spec() {
    let mut pt_hist = [0.0_f64; 4];
    assert!((pan_tompkins_derivative_step(8.0, &mut pt_hist).unwrap() - 2.0).abs() < 1e-4, "spec golden vector #1");
    assert!(pt_hist == [8.0, 0.0, 0.0, 0.0], "spec golden vector #2");
    let mut pt_ring = [0.0_f64; 4];
    let mut pt_head = 0_usize;
    let mut pt_sum = 0.0_f64;
    assert!((pan_tompkins_integrate_step(2.0, &mut pt_ring, &mut pt_head, &mut pt_sum).unwrap() - 1.0).abs() < 1e-4, "spec golden vector #3");
    assert!((pt_sum - 4.0).abs() < 1e-12 && (pt_ring[0] - 4.0).abs() < 1e-12 && pt_head == 1, "spec golden vector #4");
}
