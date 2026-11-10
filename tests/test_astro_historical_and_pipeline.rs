// SPDX-License-Identifier: LicenseRef-Project-License
//! ## Map
//! - Role: Integration test suite for Historical DST, Pre-natal Reverse Dasha, and Pure Rust 32B Domain Pipe
//! - Inputs:
//!   - Extreme historical birth scenario (1900-07-05 12:00)
//!   - Pre-natal events (parents marriage, conception, ancestor milestone)
//!   - Binary 32-byte wire packets
//! - Outputs:
//!   - Assertion validations for contract compliance, zero float math, and historical regulations

use rust_bricks::astro_historical_dst::*;
use rust_bricks::astro_pipeline::*;
use rust_bricks::astro_prenatal_dasha::*;

#[test]
/// @outputs: 1
fn test_historical_dst_1900_extreme_case() {
    // 1900-07-05: World had NO daylight saving time yet (invented in 1916).
    // If a user/agent claims DST was active (+60 min) in USA (840) or UK (826), it MUST be rejected as an anomaly.
    let check_usa_with_dst = astro_validate_historical_dst(840, 1900, 7, 5, 60);
    assert_eq!(check_usa_with_dst, Some(DST_RESULT_MISMATCH_DST_NOT_ALLOWED));

    // When correctly passed with 0 DST offset, it is validated as correct
    let check_usa_no_dst = astro_validate_historical_dst(840, 1900, 7, 5, 0);
    assert_eq!(check_usa_no_dst, Some(DST_RESULT_VALID_NO_DST));

    // Taiwan in 1955 summer (had statutory DST): claiming 60 min is valid
    let check_tw_dst = astro_validate_historical_dst(158, 1955, 7, 1, 60);
    assert_eq!(check_tw_dst, Some(DST_RESULT_VALID_DST_ACTIVE));

    // Taiwan in 1955 summer: claiming 0 min is flagged as missing DST
    let check_tw_miss = astro_validate_historical_dst(158, 1955, 7, 1, 0);
    assert_eq!(check_tw_miss, Some(DST_RESULT_MISMATCH_SHOULD_BE_DST));
}

#[test]
/// @outputs: 1
fn test_dst_clock_to_utc_sign_axiom() {
    // 1900-07-05 12:00:00 local time
    // Base UTC epoch timestamp for 1900-07-05 12:00:00 UTC is approx -2,192,961,600
    let clock_time = -2_192_961_600i64;

    // In New York (EST, -300 minutes = UTC-5), with NO DST (0 min):
    // UTC = 12:00 - (-5h) = 17:00 UTC (+5 hours = +18,000 sec)
    let utc_no_dst = astro_clock_to_utc_seconds(clock_time, -300, 0).unwrap();
    assert_eq!(utc_no_dst, clock_time + 18_000);

    // If DST WERE active (+60 min):
    // True Standard Time = Clock - 1h = 11:00 EST.
    // UTC = 11:00 EST + 5h = 16:00 UTC.
    // Notice: DST is SUBTRACTED from clock time!
    let utc_with_dst = astro_clock_to_utc_seconds(clock_time, -300, 60).unwrap();
    assert_eq!(utc_with_dst, clock_time + 18_000 - 3_600);
}

#[test]
/// @outputs: 1
fn test_pre_natal_vimshottari_reverse_step() {
    // Born in Sun Mahadasha (Sun=3, total 2191 days), with 1000 days balance remaining at birth.
    // Days spent in Sun before birth = 2191 - 1000 = 1191 days.

    // Case A: Event occurred 500 days before birth (e.g. mother pregnant / conception)
    // 500 <= 1191 -> Still in Sun Mahadasha!
    // Remaining days in Sun at conception = 1000 + 500 = 1500 days.
    let (conception_maha, rem_a) = astro_vimshottari_reverse_step(3, 1000, 500).unwrap();
    assert_eq!(conception_maha, 3);
    assert_eq!(rem_a, 1500);

    // Case B: Parents married 1500 days before birth.
    // 1500 > 1191 -> Crosses Sun into previous planet: Venus (2, total 7305 days)!
    // Days elapsed into Venus = 1500 - 1191 = 309 days before end of Venus.
    // Remaining in Venus = 7305 - 309 = 6996 days.
    let (marriage_maha, rem_b) = astro_vimshottari_reverse_step(3, 1000, 1500).unwrap();
    assert_eq!(marriage_maha, 2); // Venus
    assert_eq!(rem_b, 6996);
}

#[test]
/// @outputs: 1
fn test_btr_varga_subdivision_d9_and_d12() {
    // Moon at Taurus 15.000 deg (45,000 milli-deg)
    // Taurus is a Fixed Sign (index 1).
    // Sign pos: 15,000 milli-deg. In D9, 9 divisions of 3.333 deg (3,333 milli-deg).
    // Sub index = (15000 * 9) / 30000 = 4 (the 5th Navamsha).
    // For Fixed signs, D9 starts from 9th from Taurus: Capricorn (10).
    // Capricorn(10) + 4 = Taurus(2).
    let d9_sign = astro_btr_varga_subdivision(45_000, 9).unwrap();
    assert_eq!(d9_sign, 2); // Taurus

    // In D12 (Dwadasamsa, factor 12):
    // Sub index = (15000 * 12) / 30000 = 6 (the 7th division).
    // D12 always starts from the sign itself (Taurus = 2).
    // Taurus(2) + 6 = Scorpio(8).
    let d12_sign = astro_btr_varga_subdivision(45_000, 12).unwrap();
    assert_eq!(d12_sign, 8); // Scorpio
}

#[test]
/// @outputs: 1
fn test_pure_rust_vedic_packet_32_wire_format() {
    assert_eq!(std::mem::size_of::<VedicEventPacket32>(), 32);

    let pkt = VedicEventPacket32 {
        ts_us: 1_700_000_000_987_654,
        lat_milli_deg: 25_033,
        lon_milli_deg: 121_565,
        ayanamsha_arcsec: 85_871,
        dasha_packed: 0x0001_0703,
        transit_summary_hash: 0xABCD_1234,
        event_category_code: 101,
        flags: FLAG_PRE_NATAL_EVENT | FLAG_BTR_CALIBRATED,
    };

    let encoded = encode_vedic_event_packet_32(&pkt);
    assert_eq!(encoded.len(), 32);

    let decoded = decode_vedic_event_packet_32(&encoded).unwrap();
    assert_eq!(decoded, pkt);
    let decoded_flags = decoded.flags;
    assert_eq!(decoded_flags, FLAG_PRE_NATAL_EVENT | FLAG_BTR_CALIBRATED);
    assert_eq!(decoded_flags & FLAG_PRE_NATAL_EVENT, FLAG_PRE_NATAL_EVENT);
}
