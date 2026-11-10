// SPDX-License-Identifier: LicenseRef-Project-License
//! ## Map
//! - Role: Integration test for D300 ultra-high-order cusp sensitivity & topocentric parallax
//! - Inputs: Real-world user scenario (Mercury at 29°59'59.05" in D300)
//! - Outputs: Assertions verifying critical boundary detection and 300x error amplification

use rust_bricks::astro_varga_cusp_analysis::*;

#[test]
/// @outputs: 1
fn test_user_d300_mercury_cusp_scenario() {
    // User scenario: Mercury enters D300 at 29°59'59.05"
    // Margin to 30° boundary in D300 is exactly 0.95" = 950 milli-arcseconds (mas).
    // In physical sky: physical step = 360,000 mas (6').
    // Physical position before cusp: 360,000 mas - (950 / 300) = 360,000 - 3.167 = 359,997 mas.
    // Let's test a coordinate that maps to this exact physical remainder in sign 0 (Aries):
    let physical_mas = 359_997u64;

    let (varga_sign, dist_to_boundary, risk_code) =
        astro_d300_cusp_analysis_mas(physical_mas).unwrap();

    // In D300, 359,997 * 300 = 107,999,100 mas.
    // Margin to boundary: 108,000,000 - 107,999,100 = 900 mas (< 1000 mas = 1")
    assert_eq!(varga_sign, 1); // Aries
    assert_eq!(dist_to_boundary, 900); // 900 mas (0.90")
    assert_eq!(risk_code, CUSP_ZONE_CRITICAL); // Critical Sandhi boundary
}

#[test]
/// @outputs: 1
fn test_topocentric_parallax_error_scale() {
    // Mercury at closest approach ~0.52 AU (5200 AU_fp4)
    // Parallax = 87940 / 5200 = 16" arcsec
    let px_arcsec = astro_topocentric_parallax_max_arcsec(5_200).unwrap();
    assert_eq!(px_arcsec, 16);

    // In D300, 16" physical parallax translates to: 16 * 300 = 4,800" = 1° 20' 00"!
    let d300_parallax_shift = astro_varga_error_amplification(px_arcsec * 1_000, 300).unwrap();
    assert_eq!(d300_parallax_shift, 4_800_000); // 4,800,000 mas = 4,800"
}

#[test]
/// @outputs: 1
fn test_error_amplification_300x() {
    // A tiny 0.1" (100 mas) physical ephemeris uncertainty in D300 becomes 30" (30,000 mas)
    let amplified = astro_varga_error_amplification(100, 300).unwrap();
    assert_eq!(amplified, 30_000);
}
