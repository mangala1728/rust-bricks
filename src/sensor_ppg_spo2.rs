// ## Map
// Role: PPG雙波長血氧飽和度估算器
// Inputs: ac_red, dc_red, ac_ir, dc_ir, r
// Outputs: R (Ratio of Ratios), SpO2 (%)
// Invariants:
//   - dc_red > 0, dc_ir > 0
//   - ac_ir > 1e-12 for a valid ratio
//   - R ∈ [0.1, 3.0] for the calibration polynomial
//   - SpO2 is clamped to [0.0, 100.0]

/// Compute the Ratio of Ratios (R) from dual‑wavelength AC/DC components.
///
/// @outputs: 1
/// @domain: ac_red [0.0..1.0e6]
/// @domain: dc_red [1.0..1.0e8]
/// @domain: ac_ir [0.0..1.0e6]
/// @domain: dc_ir [1.0..1.0e8]
pub fn ppg_ratio_of_ratios(
    ac_red: f64,
    dc_red: f64,
    ac_ir: f64,
    dc_ir: f64,
) -> Option<f64> {
    // Guard against out‑of‑range inputs
    if ac_red < 0.0 || ac_red > 1.0e6 { return None; }
    if dc_red < 1.0 || dc_red > 1.0e8 { return None; }
    if ac_ir < 0.0 || ac_ir > 1.0e6 { return None; }
    if dc_ir < 1.0 || dc_ir > 1.0e8 { return None; }

    // Additional safety guards required by the contract
    if dc_red <= 0.0 || dc_ir <= 0.0 { return None; }
    if ac_ir <= 1e-12 { return None; }

    // Compute the ratios (depth ≤ 2, no unwrap)
    let red_ratio = ac_red / dc_red;
    let ir_ratio = ac_ir / dc_ir;

    // Guard against division by a near‑zero IR ratio
    if ir_ratio <= 1e-12 { return None; }

    // Ratio of Ratios
    Some(red_ratio / ir_ratio)
}

/// Estimate SpO₂ from the Ratio of Ratios using the standard empirical polynomial.
///
/// @outputs: 1
/// @domain: r [0.1..3.0]
pub fn ppg_estimate_spo2(r: f64) -> Option<f64> {
    // Guard against out‑of‑range R
    if r < 0.1 || r > 3.0 { return None; }

    // Polynomial: SpO2 = -45.060·R² + 30.354·R + 94.845
    let spo2 = -45.060 * r * r + 30.354 * r + 94.845;

    // Clamp to the physically meaningful interval [0.0, 100.0]
    let clamped = if spo2 < 0.0 {
        0.0
    } else if spo2 > 100.0 {
        100.0
    } else {
        spo2
    };

    Some(clamped)
}
