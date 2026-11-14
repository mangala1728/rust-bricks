use rust_bricks::math_brent_zero::*;

#[test]
fn test_secant_candidate_is_within_bounds() {
    // Golden test – secant path (c == a, thus fa == fc)
    let s = math_brent_candidate_step(0.0, 2.0, 0.0, -2.0, 2.0, -2.0).unwrap();
    assert!((s - 1.0).abs() < 1e-12);
}

#[test]
fn test_guard_divide_by_zero_returns_none() {
    // Golden test – (fa - fb) too small → guard triggers
    assert!(math_brent_candidate_step(0.0, 2.0, 1.0, 1.0, 1.0, 2.0).is_none());
}
