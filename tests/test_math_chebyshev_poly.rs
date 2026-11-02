use rust_bricks::math_chebyshev_poly::*;

#[test]
fn test_chebyshev_t_golden_vectors() {
    // Golden values for x = 0.5
    let x = 0.5_f64;
    let expected = [
        (0_u32, 1.0),
        (1, 0.5),
        (2, -0.5),
        (3, -1.0),
        (4, -0.5),
    ];
    for &(n, val) in &expected {
        let got = math_chebyshev_t(n, x).expect("valid input");
        assert!(
            (got - val).abs() < 1e-12,
            "T_{}({}) = {}, expected {}",
            n,
            x,
            got,
            val
        );
    }
}

#[test]
fn test_chebyshev_t_out_of_domain() {
    // x outside [-1, 1] must yield None.
    assert!(math_chebyshev_t(3, 1.5).is_none());
    assert!(math_chebyshev_t(3, f64::NAN).is_none());
}

#[test]
fn test_clenshaw_golden_vector() {
    // Coefficients c = [1, 2, 3], x = 0.5
    // Expected: 1·T₀ + 2·T₁ + 3·T₂ = 0.5
    let coeffs = [1.0_f64, 2.0, 3.0];
    let x = 0.5_f64;
    let got = math_chebyshev_clenshaw(&coeffs, x).expect("valid input");
    let expected = 0.5_f64;
    assert!(
        (got - expected).abs() < 1e-12,
        "Clenshaw result = {}, expected {}",
        got,
        expected
    );
}

#[test]
fn test_clenshaw_edge_cases() {
    // Single coefficient.
    let coeffs = [42.0_f64];
    assert_eq!(math_chebyshev_clenshaw(&coeffs, 0.0).unwrap(), 42.0);

    // Invalid domain.
    assert!(math_chebyshev_clenshaw(&[1.0, 2.0], 2.0).is_none());
    // Non‑finite coefficient.
    assert!(math_chebyshev_clenshaw(&[1.0, f64::INFINITY], 0.0).is_none());
}
