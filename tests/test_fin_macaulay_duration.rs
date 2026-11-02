use rust_bricks::fin_macaulay_duration::*;

#[test]
fn test_bond_duration_golden_vector() {
    // 3‑period annuity bond: coupons 10, redemption 100 at t=3
    let cash_flows = [10.0_f64, 10.0_f64, 110.0_f64];
    let ytm = 0.05_f64; // 5 %

    let result = bond_duration(&cash_flows, ytm);
    assert!(result.is_some(), "Expected Some((mac, mod))");

    let (d_mac, d_mod) = result.unwrap();

    // Golden values (rounded to 4 decimal places)
    let mac_expected = 2.7525_f64;
    let mod_expected = 2.6214_f64;
    let eps = 1e-4_f64;

    assert!((d_mac - mac_expected).abs() < eps,
        "Macaulay duration mismatch: got {d_mac}, expected {mac_expected}");
    assert!((d_mod - mod_expected).abs() < eps,
        "Modified duration mismatch: got {d_mod}, expected {mod_expected}");
}

#[test]
fn test_domain_boundary_guards() {
    let cf = [10.0, 10.0, 110.0];
    assert!(bond_duration(&[], 0.05).is_none());
    assert!(bond_duration(&cf, -0.01).is_none());
    assert!(bond_duration(&cf, 1.01).is_none());
    assert!(bond_duration(&[0.0, 0.0], 0.05).is_none()); // zero price
}
