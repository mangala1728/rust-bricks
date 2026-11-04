// 本檔由 cloud_farmer 依 specs/sensor_kalman_1d.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::sensor_kalman_1d::*;

#[test]
fn golden_vectors_from_spec() {
    let mut kf = (0.0_f64, 1.0_f64);
    assert!((kalman_1d_step(&mut kf, 1.0, 0.1, 0.4).unwrap() - 0.733333).abs() < 1e-4, "spec golden vector #1");
    assert!((kf.0 - 0.733333).abs() < 1e-4, "spec golden vector #2");
    assert!((kf.1 - 0.293333).abs() < 1e-4, "spec golden vector #3");
    assert!((kalman_1d_step(&mut kf, 1.0, 0.1, 0.4).unwrap() - 0.865546).abs() < 1e-4, "spec golden vector #4");
    assert!((kf.0 - 0.865546).abs() < 1e-4, "spec golden vector #5");
    assert!((kf.1 - 0.198319).abs() < 1e-4, "spec golden vector #6");
    assert!(kalman_1d_step(&mut (0.0, 1.0), 1.0, -0.1, 0.4).is_none(), "spec golden vector #7");
    assert!(kalman_1d_step(&mut (0.0, 1.0), 1.0, 0.1, 1.0e-10).is_none(), "spec golden vector #8");
    assert!(kalman_1d_step(&mut (0.0, -1.0), 1.0, 0.1, 0.4).is_none(), "spec golden vector #9");
    assert!(kalman_1d_step(&mut (0.0, 1.0), f64::NAN, 0.1, 0.4).is_none(), "spec golden vector #10");
}
