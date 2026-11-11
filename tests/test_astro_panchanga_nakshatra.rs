use rust_bricks::astro_panchanga_nakshatra::astro_panchanga_nakshatra;

#[test]
fn golden_values() {
    // 0 milli‑deg → 1st Nakshatra, 1st Pada, 0 bp
    assert_eq!(astro_panchanga_nakshatra(0), Some((1, 1, 0)));

    // 142 500 milli‑deg → 11th Nakshatra, 3rd Pada, 6 875 bp
    // 142 500 * 3 / 50 = 8 550 arc‑min
    // 8 550 / 800 = 10 → Nakshatra 11, rem = 550
    // Pada = 550 / 200 + 1 = 3
    // Progress = 550 * 10 000 / 800 = 6 875
    assert_eq!(astro_panchanga_nakshatra(142_500), Some((11, 3, 6_875)));

    // Exact 360 000 milli‑deg → last Nakshatra, 4th Pada, 9 999 bp
    assert_eq!(astro_panchanga_nakshatra(360_000), Some((27, 4, 9_999)));
}

#[test]
fn out_of_range() {
    // Anything greater than 360 000 must be rejected
    assert!(astro_panchanga_nakshatra(360_001).is_none());
    // Upper bound check (just for completeness)
    assert!(astro_panchanga_nakshatra(u32::MAX).is_none());
}
