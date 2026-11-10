// ## Map
// Role: Cloud Farmer (Coder)
// Inputs:  milli_deg: u32  – angle in millidegrees, domain [0..360_000]
// Outputs: arcmin: u32    – angle in arc‑minutes, domain [0..21_600]
// Invariants: 0 ≤ milli_deg ≤ 360_000  ⇒  0 ≤ arcmin ≤ 21_600
//
// The implementation is a deterministic, zero‑allocation conversion
// that respects the universal scale defined in the specification.

/// @outputs: 1
/// @domain: milli_deg [0..360_000]
///
/// Convert an angle expressed in **millidegrees** (1 milli‑deg = 10⁻³ deg)
/// to **arc‑minutes** (1 arc‑min = 1/60 deg).
///
/// The exact relationship is:
///
