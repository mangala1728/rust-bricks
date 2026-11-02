use rust_bricks::sensor_ppg_spo2::*;

#[test]
fn test_ratio_of_ratios_golden_vectors() {
    // Vector that should yield R = 0.5
    let r_half = ppg_ratio_of_ratios(0.5, 1.0, 1.0, 1.0).expect("valid ratio");
    assert!((r_half - 0.5).abs() < 1e-12);

    // Vector that should yield R = 1.0
    let r_one = ppg_ratio_of_ratios(1.0, 1.0, 1.0, 1.0).expect("valid ratio");
    assert!((r_one - 1.0).abs() < 1e-12);
}

#[test]
fn test_spo2_estimation_golden_vectors() {
    // Golden R = 0.5 → SpO2 ≈ 98.757
    let spo2_half = ppg_estimate_spo2(0.5).expect("valid SpO2");
    assert!((spo2_half - 98.757).abs() < 1e-3);

    // Golden R = 1.0 → SpO2 ≈ 80.139
    let spo2_one = ppg_estimate_spo2(1.0).expect("valid SpO2");
    assert!((spo2_one - 80.139).abs() < 1e-3);
}

#[test]
fn test_spo2_clamping() {
    // R very small → polynomial may exceed 100%
    let high = ppg_estimate_spo2(0.1).expect("valid SpO2");
    assert!(high <= 100.0 && high >= 0.0);

    // R very large → polynomial may go below 0%
    let low = ppg_estimate_spo2(3.0).expect("valid SpO2");
    assert!(low <= 100.0 && low >= 0.0);
}

#[test]
fn test_domain_boundary_guards() {
    assert!(ppg_ratio_of_ratios(-0.1, 1.0, 1.0, 1.0).is_none());
    assert!(ppg_ratio_of_ratios(1.0e7, 1.0, 1.0, 1.0).is_none());
    assert!(ppg_ratio_of_ratios(0.5, 0.5, 1.0, 1.0).is_none()); // dc_red < 1.0
    assert!(ppg_ratio_of_ratios(0.5, 1.0, -0.1, 1.0).is_none());
    assert!(ppg_ratio_of_ratios(0.5, 1.0, 1.0, 0.5).is_none()); // dc_ir < 1.0

    assert!(ppg_estimate_spo2(0.09).is_none());
    assert!(ppg_estimate_spo2(3.01).is_none());
}
