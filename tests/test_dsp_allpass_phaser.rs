use rust_bricks::dsp_allpass_phaser::*;

#[test]
fn test_dsp_allpass_first_order_golden() {
    let mut st = 0.0;

    let res1 = dsp_allpass_first_order_step(1.0, &mut st, 0.5);
    assert!(res1.is_some());
    if let Some(y1) = res1 {
        assert!((y1 - (-0.5)).abs() < 1e-10);
    }
    assert!((st - 1.0).abs() < 1e-10);

    let res2 = dsp_allpass_first_order_step(0.0, &mut st, 0.5);
    assert!(res2.is_some());
    if let Some(y2) = res2 {
        assert!((y2 - 0.75).abs() < 1e-10);
    }

    let res3 = dsp_allpass_first_order_step(0.0, &mut st, 0.5);
    assert!(res3.is_some());
    if let Some(y3) = res3 {
        assert!((y3 - 0.375).abs() < 1e-10);
    }

    assert!(dsp_allpass_first_order_step(1.0, &mut st, 1.0).is_none());
    assert!(dsp_allpass_first_order_step(1.0, &mut st, -1.0).is_none());
    assert!(dsp_allpass_first_order_step(f64::NAN, &mut st, 0.5).is_none());
}

#[test]
fn test_dsp_allpass_boundary_conditions() {
    let mut st = 0.0;
    
    // Invalid gains
    assert!(dsp_allpass_first_order_step(0.5, &mut st, 1.5).is_none());
    assert!(dsp_allpass_first_order_step(0.5, &mut st, -1.5).is_none());
    assert!(dsp_allpass_first_order_step(0.5, &mut st, f64::NAN).is_none());
    assert!(dsp_allpass_first_order_step(0.5, &mut st, f64::INFINITY).is_none());

    // Invalid states
    let mut nan_st = f64::NAN;
    assert!(dsp_allpass_first_order_step(0.5, &mut nan_st, 0.2).is_none());
    let mut inf_st = f64::INFINITY;
    assert!(dsp_allpass_first_order_step(0.5, &mut inf_st, 0.2).is_none());

    // Invalid inputs
    assert!(dsp_allpass_first_order_step(f64::INFINITY, &mut st, 0.2).is_none());
    assert!(dsp_allpass_first_order_step(f64::NEG_INFINITY, &mut st, 0.2).is_none());
}
