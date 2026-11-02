// ## Map
// Role:   DSP – Mel‑scale utilities (pure Rust, std only)
// Inputs: hz  [0.0 .. 100000.0]   (f64)
//         mel [0.0 .. 10000.0]    (f64)
//         num_filters [2 .. 128] (usize)
//         min_hz [0.0 .. 10000.0] (f64)
//         max_hz [100.0 .. 100000.0] (f64)
// Outputs: mel (f64) , hz (f64) , Vec<f64> (centers)
// Invariants:
//   • All public functions return `Option` – `None` on out‑of‑range or non‑finite input.
//   • No external crates, no `unsafe`, no recursion, and control‑flow nesting ≤ 2.

/// @outputs: 1
/// @domain: hz [0.0..100000.0]
pub fn hz_to_mel(hz: f64) -> Option<f64> {
    // guard: domain & finiteness
    if !hz.is_finite() || hz < 0.0 || hz > 100_000.0 {
        return None;
    }
    // O'Shaughnessy standard conversion
    let mel = 2595.0 * (1.0 + hz / 700.0).log10();
    Some(mel)
}

/// @outputs: 1
/// @domain: mel [0.0..10000.0]
pub fn mel_to_hz(mel: f64) -> Option<f64> {
    // guard: domain & finiteness
    if !mel.is_finite() || mel < 0.0 || mel > 10_000.0 {
        return None;
    }
    // O'Shaughnessy standard conversion
    let hz = 700.0 * (10_f64.powf(mel / 2595.0) - 1.0);
    Some(hz)
}

/// @outputs: 1
/// @domain: num_filters [2..128]
/// @domain: min_hz [0.0..10000.0]
/// @domain: max_hz [100.0..100000.0]
pub fn mel_filterbank_centers_hz(
    num_filters: usize,
    min_hz: f64,
    max_hz: f64,
) -> Option<Vec<f64>> {
    // guard: domain checks
    if num_filters < 2 || num_filters > 128 {
        return None;
    }
    if !min_hz.is_finite() || min_hz < 0.0 || min_hz > 10_000.0 {
        return None;
    }
    if !max_hz.is_finite() || max_hz < 100.0 || max_hz > 100_000.0 {
        return None;
    }
    if min_hz >= max_hz {
        return None;
    }

    // Convert boundaries to Mel scale
    let min_mel = match hz_to_mel(min_hz) {
        Some(v) => v,
        None => return None,
    };
    let max_mel = match hz_to_mel(max_hz) {
        Some(v) => v,
        None => return None,
    };

    // Step size in Mel domain
    let steps = (num_filters + 1) as f64;
    let delta = (max_mel - min_mel) / steps;

    // Allocate result vector (num_filters + 2 points)
    let mut centers = Vec::with_capacity(num_filters + 2);
    for i in 0..=num_filters + 1 {
        let mel = min_mel + i as f64 * delta;
        // Convert back to Hz; any failure aborts the whole operation
        match mel_to_hz(mel) {
            Some(hz) => centers.push(hz),
            None => return None,
        }
    }
    Some(centers)
}
