use rust_bricks::math_golden_section::*;

#[test]
fn test_golden_section_search_min_parabola() {
    let f = |x: f64| (x - 2.0) * (x - 2.0) + 3.0;
    let res = golden_section_search_min(0.0, 5.0, 1.0e-5, 100, f);

    assert!(res.is_some());
    if let Some((x_opt, y_opt)) = res {
        assert!((x_opt - 2.0).abs() < 1e-4);
        assert!((y_opt - 3.0).abs() < 1e-4);
    }
}

#[test]
fn test_golden_section_search_domain_guards() {
    let f = |x: f64| (x - 2.0) * (x - 2.0) + 3.0;

    // Invalid interval (a >= b)
    assert!(golden_section_search_min(5.0, 0.0, 1.0e-5, 100, f).is_none());
    assert!(golden_section_search_min(2.0, 2.0, 1.0e-5, 100, f).is_none());

    // Out-of-bound tolerance
    assert!(golden_section_search_min(0.0, 5.0, 1.0e-13, 100, f).is_none());
    assert!(golden_section_search_min(0.0, 5.0, 1.5, 100, f).is_none());

    // Out-of-bound max_iter
    assert!(golden_section_search_min(0.0, 5.0, 1.0e-5, 0, f).is_none());
    assert!(golden_section_search_min(0.0, 5.0, 1.0e-5, 1001, f).is_none());
}
