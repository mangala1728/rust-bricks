// 本檔由 cloud_farmer 依 specs/sensor_mahony_ahrs.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::sensor_mahony_ahrs::*;

#[test]
fn golden_vectors_from_spec() {
    let mut ie = (0.0_f64, 0.0_f64, 0.0_f64);
    let qm = mahony_ahrs_update_6dof(&[1.0, 0.0, 0.0, 0.0], (0.0, 0.0, 0.0), (0.0, 0.0, 1.0), &mut ie, 1.0, 0.1, 0.01).unwrap();
    assert!((qm[0] - 1.0).abs() < 1e-4, "spec golden vector #1");
    assert!(qm[1].abs() < 1e-4, "spec golden vector #2");
    assert!(qm[2].abs() < 1e-4, "spec golden vector #3");
    assert!(qm[3].abs() < 1e-4, "spec golden vector #4");
    assert!(ie.0.abs() < 1e-12 && ie.1.abs() < 1e-12 && ie.2.abs() < 1e-12, "spec golden vector #5");
    assert!(mahony_ahrs_update_6dof(&[1.0, 0.0, 0.0, 0.0], (0.0, 0.0, 0.0), (0.0, 0.0, 1.0), &mut (0.0, 0.0, 0.0), 1.0, 0.1, 0.0).is_none(), "spec golden vector #6");
    assert!(mahony_ahrs_update_6dof(&[1.0, 0.0, 0.0, 0.0], (0.0, 0.0, 0.0), (0.0, 0.0, 1.0), &mut (0.0, 0.0, 0.0), -1.0, 0.1, 0.01).is_none(), "spec golden vector #7");
    assert!(mahony_ahrs_update_6dof(&[1.0, 0.0, 0.0, 0.0], (0.0, 0.0, 0.0), (0.0, 0.0, 1.0), &mut (0.0, 0.0, 0.0), 1.0, -0.1, 0.01).is_none(), "spec golden vector #8");
}
