use rust_bricks::fin_bollinger_bands::*;

#[test]
fn test_golden_values() {
    let result = fin_compute_bollinger_bands(&[10.0, 20.0], 1.0);
    assert!(result.is_some());
    
    // @safety-proof: Option is checked with is_some assertion directly above
    let (mid, up, low, pb, bw) = result.unwrap();

    assert!((mid - 15.0).abs() < 1e-12);
    assert!((up - 20.0).abs() < 1e-12);
    assert!((low - 10.0).abs() < 1e-12);
    assert!((pb - 1.0).abs() < 1e-12);
    assert!((bw - (10.0 / 15.0)).abs() < 1e-12);

    assert!(fin_compute_bollinger_bands(&[10.0], 1.0).is_none());
    assert!(fin_compute_bollinger_bands(&[10.0, -5.0], 1.0).is_none());
}

#[test]
fn test_boundary_and_invalid_conditions() {
    // Array length boundary check (< 2 or > 256)
    assert!(fin_compute_bollinger_bands(&[], 1.0).is_none());
    assert!(fin_compute_bollinger_bands(&[10.0], 1.0).is_none());
    let too_large = [10.0; 257];
    assert!(fin_compute_bollinger_bands(&too_large, 1.0).is_none());

    // Non-positive or non-finite prices
    assert!(fin_compute_bollinger_bands(&[0.0, 10.0], 1.0).is_none());
    assert!(fin_compute_bollinger_bands(&[10.0, -1.0], 1.0).is_none());
    assert!(fin_compute_bollinger_bands(&[10.0, f64::NAN], 1.0).is_none());
    assert!(fin_compute_bollinger_bands(&[f64::INFINITY, 10.0], 1.0).is_none());

    // Invalid multiplier k
    assert!(fin_compute_bollinger_bands(&[10.0, 20.0], 0.0).is_none());
    assert!(fin_compute_bollinger_bands(&[10.0, 20.0], -1.0).is_none());
    assert!(fin_compute_bollinger_bands(&[10.0, 20.0], f64::NAN).is_none());
}

#[test]
fn test_zero_bandwidth_flat_prices() {
    let result = fin_compute_bollinger_bands(&[10.0, 10.0, 10.0], 2.0);
    assert!(result.is_some());
    
    // @safety-proof: Option is checked with is_some assertion directly above
    let (mid, up, low, pb, bw) = result.unwrap();

    assert!((mid - 10.0).abs() < 1e-12);
    assert!((up - 10.0).abs() < 1e-12);
    assert!((low - 10.0).abs() < 1e-12);
    assert!((pb - 0.5).abs() < 1e-12);
    assert!((bw - 0.0).abs() < 1e-12);
}
