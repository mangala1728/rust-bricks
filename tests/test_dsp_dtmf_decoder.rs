// ## Map
// Role: DTMF dual-tone multi-frequency detector and key mapper (ITU-T Q.23)
// Inputs: 8‑frequency power spectrum array, threshold: f64, low_idx: u8, high_idx: u8
// Outputs: Option<(u8, u8)> or Option<char>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

use rust_bricks::dsp_dtmf_decoder::*;

/// Verify the index‑to‑character mapping against the golden table.
#[test]
fn test_indices_to_char_golden() {
    assert_eq!(dtmf_indices_to_char(0, 0), Some('1'));
    assert_eq!(dtmf_indices_to_char(3, 1), Some('0'));
    assert_eq!(dtmf_indices_to_char(3, 2), Some('#'));
    assert_eq!(dtmf_indices_to_char(1, 3), Some('B'));
    assert!(dtmf_indices_to_char(4, 0).is_none());
}

/// Detect frequencies when a clear dominant tone exists.
#[test]
fn test_detect_frequencies_success() {
    let p = [10.0, 1.0, 1.0, 1.0, 1.0, 12.0, 1.0, 1.0];
    assert_eq!(dtmf_detect_frequencies(&p, 5.0), Some((0, 1)));
}

/// Detect frequencies should fail when low‑band power is below threshold.
#[test]
fn test_detect_frequencies_low_below_threshold() {
    let p_low = [1.0, 1.0, 1.0, 1.0, 1.0, 12.0, 1.0, 1.0];
    assert!(dtmf_detect_frequencies(&p_low, 5.0).is_none());
}

/// Guard checks – invalid threshold or power values return `None`.
#[test]
fn test_detect_frequencies_invalid_inputs() {
    let bad_threshold = -1.0;
    let p = [0.0; 8];
    assert!(dtmf_detect_frequencies(&p, bad_threshold).is_none());

    let p_nan = [f64::NAN, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    assert!(dtmf_detect_frequencies(&p_nan, 0.0).is_none());
}
