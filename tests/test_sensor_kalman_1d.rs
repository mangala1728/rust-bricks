use rust_bricks::sensor_kalman_1d::*;

#[test]
fn golden_vector_validation() {
    // Initial state: x̂₀ = 0.0, P₀ = 1.0
    let mut state = (0.0_f64, 1.0_f64);
    let q = 0.1_f64;
    let r = 0.4_f64;

    // ---- Step 1 ------------------------------------------------------------
    let est1 = match kalman_1d_step(&mut state, 1.0, q, r) {
        Some(v) => v,
        None => {
            // @safety-proof: inputs are within the documented domain, None must not occur
            panic!("Step 1 returned None");
        }
    };
    assert!((est1 - 0.7333333333).abs() < 1e-4);
    assert!((state.0 - 0.7333333333).abs() < 1e-4);
    assert!((state.1 - 0.2933333333).abs() < 1e-4);

    // ---- Step 2 ------------------------------------------------------------
    let est2 = match kalman_1d_step(&mut state, 1.0, q, r) {
        Some(v) => v,
        None => {
            // @safety-proof: inputs are within the documented domain, None must not occur
            panic!("Step 2 returned None");
        }
    };
    assert!((est2 - 0.865546).abs() < 1e-4);
    assert!((state.0 - 0.865546).abs() < 1e-4);
    assert!((state.1 - 0.198319).abs() < 1e-4);
}

#[test]
fn test_domain_boundary_guards() {
    let mut state = (0.0_f64, 1.0_f64);
    assert!(kalman_1d_step(&mut state, 1.0, -0.1, 0.4).is_none());
    assert!(kalman_1d_step(&mut state, 1.0, 1001.0, 0.4).is_none());
    assert!(kalman_1d_step(&mut state, 1.0, 0.1, 0.0).is_none());
    assert!(kalman_1d_step(&mut state, f64::NAN, 0.1, 0.4).is_none());

    let mut bad_cov = (0.0_f64, -1.0_f64);
    assert!(kalman_1d_step(&mut bad_cov, 1.0, 0.1, 0.4).is_none());
}
