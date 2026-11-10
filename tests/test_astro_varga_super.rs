// SPDX-License-Identifier: LicenseRef-Project-License
//! ## Map
//! - Role: Integration test for ultra-high-order Varga charts (D81, D108, D144) exact integer arithmetic
//! - Inputs: Arcsecond coordinates, high division numbers
//! - Outputs: Assertions verifying exact integer steps per Pada and deterministic varga signs

use rust_bricks::astro_varga_super::*;

#[test]
/// @outputs: 1
fn test_exact_integer_steps_per_nakshatra_pada() {
    // 1 Nakshatra Pada = 3°20' = 12,000" arcseconds.
    // D9 (Navamsha, 1/9 of sign): exactly 1 step per Pada
    assert_eq!(astro_pada_to_super_varga_steps(9), Some(1));

    // D81 (Navamsha of Navamsha, 9*9): exactly 9 steps per Pada!
    assert_eq!(astro_pada_to_super_varga_steps(81), Some(9));

    // D108 (Ashtottaramsa, 12*9): exactly 12 steps per Pada!
    assert_eq!(astro_pada_to_super_varga_steps(108), Some(12));

    // D144 (Dwadasamsa of Dwadasamsa, 12*12): exactly 16 steps per Pada!
    assert_eq!(astro_pada_to_super_varga_steps(144), Some(16));

    // D12: 12,000 * 12 = 144,000 % 108,000 != 0 (not evenly divisible, 1.333 steps)
    assert_eq!(astro_pada_to_super_varga_steps(12), None);
}

#[test]
/// @outputs: 1
fn test_d108_exact_thousand_arcsecond_steps() {
    // 1 sign = 108,000". D108 step size = 108,000 / 108 = 1,000" exact!
    // Test planet at Aries 0°00'00" (0")
    let res_0 = astro_super_varga_arcsec(0, 108).unwrap();
    assert_eq!(res_0, (1, 0, 0)); // Sign 1 (Aries), sub_idx 0, 0 permille

    // Test planet at Aries 0°16'40" (1,000" exact) -> enters 2nd sub-division of D108
    let res_1000 = astro_super_varga_arcsec(1_000, 108).unwrap();
    assert_eq!(res_1000, (2, 1, 0)); // Sign 2 (Taurus), sub_idx 1, 0 permille

    // Test planet at Aries 0°25'00" (1,500") -> in 2nd sub-division, halfway (500 permille)
    let res_1500 = astro_super_varga_arcsec(1_500, 108).unwrap();
    assert_eq!(res_1500, (2, 1, 500));
}

#[test]
/// @outputs: 1
fn test_d144_exact_seven_hundred_fifty_arcsecond_steps() {
    // 1 sign = 108,000". D144 step size = 108,000 / 144 = 750" exact (12' 30")!
    // Test planet at Taurus 0°12'30" (108,000 + 750 = 108,750")
    // Taurus is sign 1 (0-indexed). Sub_idx = 1. Target sign = (1 + 1) % 12 + 1 = 3 (Gemini).
    let res = astro_super_varga_arcsec(108_750, 144).unwrap();
    assert_eq!(res, (3, 1, 0));
}
