use rust_bricks::dsp_state_variable_filter::*;

#[test]
fn test_golden_vector() {
    let res = dsp_svf_step(1.0, 0.0, 0.0, 0.5, 1.0).unwrap();
    assert!((res.0 - 0.25).abs() < 1e-12);
    assert!((res.1 - 0.50).abs() < 1e-12);
    assert!((res.2 - 1.00).abs() < 1e-12);
    assert!((res.3 - 1.25).abs() < 1e-12);
    assert!((res.4 - 0.25).abs() < 1e-12);
    assert!((res.5 - 0.50).abs() < 1e-12);

    assert!(dsp_svf_step(1.0, 0.0, 0.0, 0.9, 1.0).is_none());
    assert!(dsp_svf_step(1.0, 0.0, 0.0, 0.5, 0.0).is_none());
}

#[test]
fn test_out_of_domain_guards() {
    // Check upper and lower bounds for input, s1, s2, f, q
    assert!(dsp_svf_step(1e6 + 1.0, 0.0, 0.0, 0.1, 1.0).is_none());
    assert!(dsp_svf_step(-1e6 - 1.0, 0.0, 0.0, 0.1, 1.0).is_none());

    assert!(dsp_svf_step(0.0, 1e6 + 1.0, 0.0, 0.1, 1.0).is_none());
    assert!(dsp_svf_step(0.0, -1e6 - 1.0, 0.0, 0.1, 1.0).is_none());

    assert!(dsp_svf_step(0.0, 0.0, 1e6 + 1.0, 0.1, 1.0).is_none());
    assert!(dsp_svf_step(0.0, 0.0, -1e6 - 1.0, 0.1, 1.0).is_none());

    assert!(dsp_svf_step(0.0, 0.0, 0.0, 0.00009, 1.0).is_none());
    assert!(dsp_svf_step(0.0, 0.0, 0.0, 0.8001, 1.0).is_none());

    assert!(dsp_svf_step(0.0, 0.0, 0.0, 0.1, 0.00009).is_none());
    assert!(dsp_svf_step(0.0, 0.0, 0.0, 0.1, 2.0001).is_none());

    // Non-finite value guards
    assert!(dsp_svf_step(f64::NAN, 0.0, 0.0, 0.1, 1.0).is_none());
    assert!(dsp_svf_step(0.0, f64::INFINITY, 0.0, 0.1, 1.0).is_none());
    assert!(dsp_svf_step(0.0, 0.0, f64::NEG_INFINITY, 0.1, 1.0).is_none());
    assert!(dsp_svf_step(0.0, 0.0, 0.0, f64::NAN, 1.0).is_none());
    assert!(dsp_svf_step(0.0, 0.0, 0.0, 0.1, f64::NAN).is_none());
}

#[test]
fn test_multi_step_simulation() {
    let mut s1 = 0.0;
    let mut s2 = 0.0;
    let f = 0.1;
    let q = 0.7071;
    let input = 1.0;

    for _ in 0..10 {
        let res = dsp_svf_step(input, s1, s2, f, q);
        assert!(res.is_some());
        let (lp, bp, hp, notch, next_s1, next_s2) = res.unwrap();
        
        assert!((next_s1 - lp).abs() < 1e-12);
        assert!((next_s2 - bp).abs() < 1e-12);
        assert!((notch - (hp + lp)).abs() < 1e-12);

        s1 = next_s1;
        s2 = next_s2;
    }
}
