// ## Map
// Role: Integration test for astro_vimshottari_dasha packing / unpacking.
// Inputs: predefined golden vectors.
// Outputs: assertions that the contract behaves exactly as specified.

use rust_bricks::astro_vimshottari_dasha::*;

#[test]
fn golden_pack_and_unpack() {
    // Golden vector from the specification:
    // maha=3 (Sun), antar=7 (Jupiter), prat=1 (Ketu), sookshma=4 (Moon),
    // prana=5 (Mars), balance_permille=2500
    let packed = astro_vimshottari_dasha_pack(3, 7, 1, 4, 5, 2500);
    assert_eq!(packed, Some(2621784435));

    // Unpack must reproduce the original fields
    let unpacked = astro_vimshottari_dasha_unpack(2621784435);
    assert_eq!(unpacked, (3, 7, 1, 4, 5, 2500));
}

#[test]
fn reject_invalid_planet_codes() {
    // maha out of range (0)
    assert!(astro_vimshottari_dasha_pack(0, 1, 1, 1, 1, 0).is_none());

    // antar out of range (10)
    assert!(astro_vimshottari_dasha_pack(1, 10, 1, 1, 1, 0).is_none());

    // prat out of range (0)
    assert!(astro_vimshottari_dasha_pack(1, 1, 0, 1, 1, 0).is_none());

    // sookshma out of range (10)
    assert!(astro_vimshottari_dasha_pack(1, 1, 1, 10, 1, 0).is_none());

    // prana out of range (0)
    assert!(astro_vimshottari_dasha_pack(1, 1, 1, 1, 0, 0).is_none());
}

#[test]
fn reject_invalid_balance() {
    // balance_permille exceeds 12‑bit maximum (4095)
    assert!(astro_vimshottari_dasha_pack(1, 1, 1, 1, 1, 4096).is_none());
}
