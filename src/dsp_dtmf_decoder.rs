// ## Map
// Role: DTMF dual-tone multi-frequency detector and key mapper (ITU-T Q.23)
// Inputs: 8‑frequency power spectrum array, threshold: f64, low_idx: u8, high_idx: u8
// Outputs: Option<(u8, u8)> or Option<char>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: threshold >= 0.0 && threshold.is_finite()
///
/// Detect the dominant low‑frequency and high‑frequency indices from an 8‑element
/// power spectrum.  Returns `None` if the threshold guard fails, any power is
/// invalid, or the strongest tones in either band are below the threshold.
pub fn dtmf_detect_frequencies(powers: &[f64; 8], threshold: f64) -> Option<(u8, u8)> {
    // domain guard
    if threshold < 0.0 || !threshold.is_finite() {
        return None;
    }
    // guard each power element
    if powers.iter().any(|&p| p < 0.0 || !p.is_finite()) {
        return None;
    }

    // find max in low band (indices 0..4)
    let mut low_max = threshold;
    let mut low_idx: u8 = 0;
    for (i, &p) in powers.iter().take(4).enumerate() {
        if p > low_max {
            low_max = p;
            low_idx = i as u8;
        }
    }
    // find max in high band (indices 4..8)
    let mut high_max = threshold;
    let mut high_idx: u8 = 0;
    for (i, &p) in powers.iter().skip(4).enumerate() {
        if p > high_max {
            high_max = p;
            high_idx = i as u8;
        }
    }

    // if either band did not exceed the threshold → no valid tone
    if low_max <= threshold || high_max <= threshold {
        return None;
    }

    Some((low_idx, high_idx))
}

/// @outputs: 1
/// @domain: low_idx < 4 && high_idx < 4
///
/// Convert a pair of low‑ and high‑frequency indices into the corresponding DTMF
/// character according to the ITU‑T Q.23 mapping table.
pub fn dtmf_indices_to_char(low_idx: u8, high_idx: u8) -> Option<char> {
    // domain guard
    if low_idx >= 4 || high_idx >= 4 {
        return None;
    }

    // mapping table – rows: low frequencies, columns: high frequencies
    let ch = match (low_idx, high_idx) {
        (0, 0) => '1',
        (0, 1) => '2',
        (0, 2) => '3',
        (0, 3) => 'A',
        (1, 0) => '4',
        (1, 1) => '5',
        (1, 2) => '6',
        (1, 3) => 'B',
        (2, 0) => '7',
        (2, 1) => '8',
        (2, 2) => '9',
        (2, 3) => 'C',
        (3, 0) => '*',
        (3, 1) => '0',
        (3, 2) => '#',
        (3, 3) => 'D',
        // The guard above guarantees we never reach this arm,
        // but the compiler needs an exhaustive match.
        _ => unreachable!(),
    };
    Some(ch)
}
