// ## Map
// Role: integration test for STFT Overlap-Add (OLA) spectral subtraction denoiser
// Inputs: none (uses public API)
// Outputs: test results via assertions
// Invariants: 0 external dependencies, functions <= 60 lines

use rust_bricks::dsp_stft_ola_denoiser::*;

#[test]
fn test_gain_single_golden_and_guards() {
    // Golden values
    assert_eq!(spectral_subtraction_gain_single(100.0, 10.0, 2.0, 0.05), 0.8);
    assert_eq!(spectral_subtraction_gain_single(10.0, 10.0, 2.0, 0.05), 0.05);
    assert_eq!(spectral_subtraction_gain_single(0.0, 10.0, 2.0, 0.05), 0.05);
    assert_eq!(spectral_subtraction_gain_single(100.0, 0.0, 2.0, 0.05), 1.0);

    // Out of domain guards -> fallback to 1.0
    assert_eq!(spectral_subtraction_gain_single(100.0, 10.0, 0.5, 0.05), 1.0);
    assert_eq!(spectral_subtraction_gain_single(100.0, 10.0, 11.0, 0.05), 1.0);
    assert_eq!(spectral_subtraction_gain_single(100.0, 10.0, 2.0, 0.0001), 1.0);
    assert_eq!(spectral_subtraction_gain_single(100.0, 10.0, 2.0, 0.6), 1.0);
    assert_eq!(spectral_subtraction_gain_single(f32::NAN, 10.0, 2.0, 0.05), 1.0);
}

#[test]
fn test_clean_signal_perfect_reconstruction() {
    let mut input = [0i16; 256];
    for i in 0..256 {
        input[i] = ((i as f32 * 0.1).sin() * 10000.0) as i16;
    }
    let mut overlap = [0.0f32; 256];
    let noise = [0.0f32; 256]; // Zero noise
    let mut output = [0i16; 256];

    let written = stft_spectral_denoise_frame(&input, &mut overlap, &noise, 2.0, 0.05, &mut output);
    assert_eq!(written, 256);

    // With zero noise, output must exactly match input
    for i in 0..256 {
        assert_eq!(output[i], input[i], "Sample {} mismatch on clean passthrough", i);
    }
    // Overlap buffer must remain zero
    for i in 0..256 {
        assert!(overlap[i].abs() < 0.1, "Overlap[{}] = {} should be near zero", i, overlap[i]);
    }
}

#[test]
fn test_noise_suppression_and_overlap_continuity() {
    let noise_val = 500i16;
    let input = [noise_val; 256];
    let mut overlap = [0.0f32; 256];
    let mut noise_profile = [0.0f32; 256];

    // Estimate noise profile from 256 noise samples
    let est = estimate_noise_power_profile(&input, &mut noise_profile);
    assert_eq!(est, 256);
    assert!(noise_profile[0] > 0.0);

    let mut output = [0i16; 256];
    let written = stft_spectral_denoise_frame(&input, &mut overlap, &noise_profile, 2.0, 0.05, &mut output);
    assert_eq!(written, 256);

    // Suppressed output should be significantly reduced compared to input
    let max_out = output.iter().map(|&x| x.abs()).max().unwrap_or(0);
    assert!(max_out < noise_val / 2, "Max output {} should be suppressed below {}", max_out, noise_val / 2);
}

#[test]
fn test_domain_guards_frame() {
    let input = [0i16; 256];
    let mut overlap = [0.0f32; 256];
    let noise = [0.0f32; 256];
    let mut output = [0i16; 256];

    // Invalid oversubtraction
    assert_eq!(stft_spectral_denoise_frame(&input, &mut overlap, &noise, 0.5, 0.05, &mut output), 0);
    assert_eq!(stft_spectral_denoise_frame(&input, &mut overlap, &noise, 15.0, 0.05, &mut output), 0);
    // Invalid spectral floor
    assert_eq!(stft_spectral_denoise_frame(&input, &mut overlap, &noise, 2.0, 0.0001, &mut output), 0);
    assert_eq!(stft_spectral_denoise_frame(&input, &mut overlap, &noise, 2.0, 0.9, &mut output), 0);
    // Non-finite
    assert_eq!(stft_spectral_denoise_frame(&input, &mut overlap, &noise, f32::NAN, 0.05, &mut output), 0);
}
