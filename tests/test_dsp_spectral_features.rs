use rust_bricks::dsp_spectral_features::*;

#[test]
fn test_centroid_and_rolloff_golden() {
    // Golden vector (spec from the requirement)
    let sample_rate = 1000.0_f64;
    let mags = [1.0_f64, 2.0, 4.0, 2.0, 1.0]; // N = 5
    // Expected values
    let golden_centroid = 250.0_f64;
    let golden_rolloff = 375.0_f64;
    let rolloff_ratio = 0.85_f64;

    // Compute
    let centroid = spectral_centroid(&mags, sample_rate).expect("valid centroid");
    let rolloff = spectral_rolloff(&mags, sample_rate, rolloff_ratio).expect("valid rolloff");

    // Verify within tolerance
    assert!((centroid - golden_centroid).abs() < 1e-4, "centroid mismatch");
    assert!((rolloff - golden_rolloff).abs() < 1e-4, "rolloff mismatch");
}

#[test]
fn test_invalid_inputs() {
    // Sample rate out of domain
    assert!(spectral_centroid(&[1.0, 2.0], 0.5).is_none());
    // Not enough bins
    assert!(spectral_centroid(&[1.0], 1000.0).is_none());
    // Silent spectrum
    assert!(spectral_centroid(&[0.0, 0.0], 1000.0).is_none());

    // Rolloff: rolloff_ratio out of domain
    assert!(spectral_rolloff(&[1.0, 2.0], 1000.0, 0.4).is_none());
    // Not enough bins
    assert!(spectral_rolloff(&[1.0], 1000.0, 0.8).is_none());
    // Silent spectrum
    assert!(spectral_rolloff(&[0.0, 0.0], 1000.0, 0.8).is_none());
}
