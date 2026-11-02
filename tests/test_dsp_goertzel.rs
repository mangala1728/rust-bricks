use rust_bricks::dsp_goertzel::*;

#[test]
fn golden_vector_1000hz_at_8k_sampling() {
    // Parameters from the spec
    let sample_rate = 8000.0_f64;
    let target_freq = 1000.0_f64;

    // 1️⃣ Compute coefficient and verify
    let coeff = goertzel_coeff(target_freq, sample_rate)
        .expect("valid coefficient should be produced");
    assert!((coeff - 1.41421356_f64).abs() < 1e-4, "coeff mismatch");

    // 2️⃣ Build the 8‑sample sine wave
    let n_samples = 8usize;
    let mut samples = Vec::with_capacity(n_samples);
    for n in 0..n_samples {
        let phase = 2.0 * std::f64::consts::PI * target_freq * n as f64 / sample_rate;
        samples.push(phase.sin());
    }

    // 3️⃣ Run detection
    let (power, magnitude) = goertzel_detect(&samples, coeff)
        .expect("detection should succeed");

    // 4️⃣ Verify against golden values
    assert!((power - 16.0_f64).abs() < 1e-4, "power mismatch");
    assert!((magnitude - 4.0_f64).abs() < 1e-4, "magnitude mismatch");
}

#[test]
fn test_domain_boundary_guards() {
    assert!(goertzel_coeff(0.5, 8000.0).is_none());
    assert!(goertzel_coeff(5000.0, 8000.0).is_none()); // >= fs / 2
    assert!(goertzel_coeff(1000.0, 1.0).is_none());

    assert!(goertzel_detect(&[], 1.414).is_none());
    assert!(goertzel_detect(&[1.0, 2.0], 2.5).is_none());
}
