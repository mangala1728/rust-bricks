use rust_bricks::sensor_mahony_ahrs::*;

#[test]
fn test_static_level_facing_up() {
    // Golden initial conditions
    let q0 = [1.0_f64, 0.0, 0.0, 0.0];
    let gyro = (0.0_f64, 0.0, 0.0);
    let accel = (0.0_f64, 0.0, 1.0);
    let mut integral = (0.0_f64, 0.0, 0.0);
    let kp = 0.0_f64;
    let ki = 0.0_f64;
    let dt = 0.01_f64;

    // Expected: unchanged quaternion
    let golden = [1.0_f64, 0.0, 0.0, 0.0];

    let result = mahony_ahrs_update_6dof(&q0, gyro, accel, &mut integral, kp, ki, dt)
        .expect("valid parameters should yield Some");

    for (r, g) in result.iter().zip(golden.iter()) {
        assert!((r - g).abs() < 1e-4, "result {:?} deviates from golden {:?}", result, golden);
    }
}

#[test]
fn test_invalid_dt_returns_none() {
    let q = [1.0_f64, 0.0, 0.0, 0.0];
    let gyro = (0.0_f64, 0.0, 0.0);
    let accel = (0.0_f64, 0.0, 1.0);
    let mut integral = (0.0_f64, 0.0, 0.0);
    let kp = 1.0_f64;
    let ki = 0.0_f64;
    let dt = 0.0_f64; // below allowed range

    let result = mahony_ahrs_update_6dof(&q, gyro, accel, &mut integral, kp, ki, dt);
    assert!(result.is_none(), "dt out of range must yield None");
}

#[test]
fn test_domain_guards_all_params() {
    let q = [1.0_f64, 0.0, 0.0, 0.0];
    let gyro = (0.0_f64, 0.0, 0.0);
    let accel = (0.0_f64, 0.0, 1.0);
    let mut integral = (0.0_f64, 0.0, 0.0);

    assert!(mahony_ahrs_update_6dof(&q, gyro, accel, &mut integral, -0.1, 0.0, 0.01).is_none());
    assert!(mahony_ahrs_update_6dof(&q, gyro, accel, &mut integral, 100.1, 0.0, 0.01).is_none());
    assert!(mahony_ahrs_update_6dof(&q, gyro, accel, &mut integral, 1.0, -0.1, 0.01).is_none());
    assert!(mahony_ahrs_update_6dof(&q, gyro, accel, &mut integral, 1.0, 100.1, 0.01).is_none());
    assert!(mahony_ahrs_update_6dof(&q, gyro, accel, &mut integral, 1.0, 0.0, 1.1).is_none());

    // Degenerate zero accel
    let err = mahony_compute_gravity_error(&q, (0.0, 0.0, 0.0));
    assert_eq!(err, (0.0, 0.0, 0.0));
}
