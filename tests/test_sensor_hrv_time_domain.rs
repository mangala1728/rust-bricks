use rust_bricks::sensor_hrv_time_domain::*;

#[test]
fn test_hrv_time_domain_golden_vector() {
    let rr_intervals = [800.0, 810.0, 890.0, 820.0, 900.0];
    let result = hrv_time_domain_metrics(&rr_intervals);

    assert!(result.is_some());
    if let Some((mean_rr, sdnn, rmssd, pnn50)) = result {
        assert!((mean_rr - 844.0).abs() < 1e-4);
        assert!((sdnn - 42.2374).abs() < 1e-4);
        assert!((rmssd - 66.7083).abs() < 1e-4);
        assert!((pnn50 - 75.0).abs() < 1e-4);
    }
}

#[test]
fn test_hrv_time_domain_insufficient_length() {
    let short_data = [800.0];
    assert!(hrv_time_domain_metrics(&short_data).is_none());
}

#[test]
fn test_hrv_time_domain_out_of_bounds() {
    let low_bound = [199.9, 800.0];
    assert!(hrv_time_domain_metrics(&low_bound).is_none());

    let high_bound = [800.0, 3000.1];
    assert!(hrv_time_domain_metrics(&high_bound).is_none());

    let nan_data = [800.0, f64::NAN];
    assert!(hrv_time_domain_metrics(&nan_data).is_none());
}
