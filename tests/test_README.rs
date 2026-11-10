use rust_bricks::readme::*;

/// Golden test vector generated from the specification:
///   milli_deg = 180_000  →  arcmin = 10_800
#[test]
fn test_milli_deg_to_arcmin_golden() {
    let input = 180_000u32;          // 180.000°
    let expected = Some(10_800u32); // (180_000 * 3) / 50 = 10_800
    assert_eq!(milli_deg_to_arcmin(input), expected);
}

/// Verify that out‑of‑range inputs are rejected safely.
#[test]
fn test_milli_deg_to_arcmin_out_of_range() {
    // Below the lower bound (cannot happen with u32, but we test the upper bound)
    let too_big = 360_001u32;
    assert_eq!(milli_deg_to_arcmin(too_big), None);
}
