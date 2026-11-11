//! Integration tests for the `astro_ayanamsha_converter` module.

use rust_bricks::astro_ayanamsha_converter::*;

/// Verify that a zero ayanamsha leaves the longitude unchanged.
#[test]
fn zero_ayanamsha_identity() {
    assert_eq!(astro_tropical_to_sidereal(150_000, 0), Some(150_000));
    assert_eq!(astro_sidereal_to_tropical(150_000, 0), Some(150_000));
}

/// Test conversion using the classic Lahiri ayanamsha (85 871″ → 23 853 milli‑deg).
#[test]
fn lahiri_example() {
    // Tropical 30.000° → Sidereal ≈ 6.147°
    assert_eq!(astro_tropical_to_sidereal(30_000, 85_871), Some(6_147));
    // Reverse conversion restores the original tropical longitude.
    assert_eq!(astro_sidereal_to_tropical(6_147, 85_871), Some(30_000));
}

/// Ensure out‑of‑range tropical input is rejected.
#[test]
fn tropical_out_of_range() {
    assert!(astro_tropical_to_sidereal(360_001, 85_871).is_none());
}

/// Ensure out‑of‑range ayanamsha input is rejected.
#[test]
fn ayanamsha_out_of_range() {
    assert!(astro_tropical_to_sidereal(30_000, 1_296_001).is_none());
}
