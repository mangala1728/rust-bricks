// SPDX-License-Identifier: LicenseRef-Project-License
//! ## Map
//! - Role: Historical Daylight Saving Time (DST) validation & Clock-to-UTC signed conversion
//! - Inputs:
//!   - clock_epoch_sec: i64 (supports pre-1970 and negative timestamps down to year 0001)
//!   - tz_offset_minutes: i16 (-840..840)
//!   - dst_offset_minutes: i16 (-120..120)
//!   - country_iso_num: u16 (1..999)
//!   - calendar date: year: i32, month: u8, day: u8
//! - Outputs:
//!   - Option<i64>: Exact UTC epoch seconds (negative for pre-1970)
//!   - Option<u8>: DST validation code (0=ValidNoDst, 1=ValidDst, 2=MismatchOmission, 3=MismatchAnomalous)
//! - Invariants:
//!   1. Strict 09 contract boundary & NASA JPL <= 60 lines per function.
//!   2. Pure integer math only, zero float rounding.
//!   3. Clock-to-UTC axiom: UTC = Clock - (TZ * 60) - (DST * 60). DST is always subtracted.
//!   4. Pre-1916 world hard barrier: No sovereign DST existed before 1916.

pub const DST_RESULT_VALID_NO_DST: u8 = 0;
pub const DST_RESULT_VALID_DST_ACTIVE: u8 = 1;
pub const DST_RESULT_MISMATCH_SHOULD_BE_DST: u8 = 2;
pub const DST_RESULT_MISMATCH_DST_NOT_ALLOWED: u8 = 3;

/// Converts a local wall-clock timestamp to absolute UTC epoch seconds.
/// Axiom: Clock time is advanced during DST, so DST offset is SUBTRACTED.
/// Standard Time = Clock - DST; UTC = Standard Time - TimeZone.
/// @outputs: 1
/// @domain: clock_epoch_sec [-62135596800..=253402300799]
/// @domain: tz_offset_minutes [-840..=840]
/// @domain: dst_offset_minutes [-120..=120]
pub fn astro_clock_to_utc_seconds(
    clock_epoch_sec: i64,
    tz_offset_minutes: i16,
    dst_offset_minutes: i16,
) -> Option<i64> {
    if clock_epoch_sec < -62_135_596_800 || clock_epoch_sec > 253_402_300_799 {
        return None;
    }
    if tz_offset_minutes < -840 || tz_offset_minutes > 840 {
        return None;
    }
    if dst_offset_minutes < -120 || dst_offset_minutes > 120 {
        return None;
    }

    let tz_sec = (tz_offset_minutes as i64).saturating_mul(60);
    let dst_sec = (dst_offset_minutes as i64).saturating_mul(60);

    // Clock - DST gives standard time, then - TZ gives UTC
    let std_sec = clock_epoch_sec.checked_sub(dst_sec)?;
    std_sec.checked_sub(tz_sec)
}

/// Helper to check if a month and day fall within an inclusive summer range.
fn is_in_summer_window(month: u8, day: u8, start_m: u8, start_d: u8, end_m: u8, end_d: u8) -> bool {
    let cur = (month as u16) * 100 + (day as u16);
    let start = (start_m as u16) * 100 + (start_d as u16);
    let end = (end_m as u16) * 100 + (end_d as u16);
    cur >= start && cur <= end
}

/// Evaluates whether a specific nation had active statutory DST on a given calendar date.
fn query_statutory_dst(country: u16, year: i32, month: u8, day: u8) -> bool {
    if year < 1916 {
        return false;
    }
    if country == 158 {
        // Taiwan (158): Observed DST in 1945-1961, 1972, 1974, 1975, 1979
        let valid_year = (1945..=1961).contains(&year)
            || year == 1972
            || year == 1974
            || year == 1975
            || year == 1979;
        return valid_year && is_in_summer_window(month, day, 5, 1, 9, 30);
    }
    if country == 156 {
        // China (156): Observed DST only from 1986 to 1991 (mid-April to mid-September)
        return (1986..=1991).contains(&year) && is_in_summer_window(month, day, 4, 15, 9, 15);
    }
    if country == 840 {
        // USA (840): 1918-1919, 1942-1945 (War Time), 1966+ (Uniform Time Act)
        if year == 1918 || year == 1919 || (1942..=1945).contains(&year) {
            return true;
        }
        if year >= 1966 {
            let start_m = if year >= 2007 { 3 } else { 4 };
            let end_m = if year >= 2007 { 11 } else { 10 };
            return is_in_summer_window(month, day, start_m, 1, end_m, 31);
        }
    }
    false
}

/// Validates claimed DST offset against historical statutory regulations.
/// Prevents anachronisms (e.g. claiming DST in 1900 before DST existed).
/// @outputs: 1
/// @domain: country_iso_num [1..=999]
/// @domain: year [1800..=2100]
/// @domain: month [1..=12]
/// @domain: day [1..=31]
/// @domain: claimed_dst_minutes [-120..=120]
pub fn astro_validate_historical_dst(
    country_iso_num: u16,
    year: i32,
    month: u8,
    day: u8,
    claimed_dst_minutes: i16,
) -> Option<u8> {
    if !(1..=999).contains(&country_iso_num) || !(1800..=2100).contains(&year) {
        return None;
    }
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    if !(-120..=120).contains(&claimed_dst_minutes) {
        return None;
    }

    let statutory_active = query_statutory_dst(country_iso_num, year, month, day);
    let has_claimed = claimed_dst_minutes > 0;

    if statutory_active && has_claimed {
        Some(DST_RESULT_VALID_DST_ACTIVE)
    } else if !statutory_active && !has_claimed {
        Some(DST_RESULT_VALID_NO_DST)
    } else if statutory_active && !has_claimed {
        Some(DST_RESULT_MISMATCH_SHOULD_BE_DST)
    } else {
        Some(DST_RESULT_MISMATCH_DST_NOT_ALLOWED)
    }
}
