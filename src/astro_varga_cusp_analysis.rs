// SPDX-License-Identifier: LicenseRef-Project-License
//! ## Map
//! - Role: Ultra-high-order Varga (D300/D150) Cusp & Sandhi Sensitivity Analyzer
//! - Inputs:
//!   - Coordinates in milli-arcseconds (mas, 1 mas = 10^-3 arcsec = 1/3,600,000 deg)
//!   - Planetary distance AU fp4 & observer topocentric geometry
//! - Outputs:
//!   - D300 amplified cusp margin, Sandhi boundary risk classification
//!   - Topocentric parallax correction bounds in arcseconds
//! - Invariants:
//!   1. Strict 09 contract boundary & NASA JPL <= 60 lines per function.
//!   2. 100% pure integer arithmetic using milli-arcseconds (mas).
//!   3. Cusp error amplification theorem: Error in D_N is exactly N * physical_error.

pub const CUSP_ZONE_SAFE: u8 = 0;       // Margin >= 60,000 mas (>= 60" in D300)
pub const CUSP_ZONE_SENSITIVE: u8 = 1;  // 1,000 mas <= Margin < 60,000 mas (1" .. 60")
pub const CUSP_ZONE_CRITICAL: u8 = 2;   // Margin < 1,000 mas (< 1" in D300, physical < 3.3 mas)

/// 1 Rashi sign in milli-arcseconds: 30 deg * 3600 * 1000 = 108,000,000 mas.
pub const SIGN_MAS: u64 = 108_000_000;

/// Analyzes a planet's placement in D300 down to milli-arcseconds (mas) resolution.
/// Determines distance to the nearest divisional cusp and flags critical boundary risks.
/// @outputs: 1
/// @domain: sidereal_mas [0..=1296000000000]
pub fn astro_d300_cusp_analysis_mas(sidereal_mas: u64) -> Option<(u8, u64, u8)> {
    if sidereal_mas > 1_296_000_000_000 {
        return None;
    }

    let clamped = sidereal_mas % 1_296_000_000_000;
    let rashi_sign = (clamped / SIGN_MAS) as u8; // 0=Aries .. 11=Pisces
    let pos_in_sign = clamped % SIGN_MAS;

    // D300 step size in physical sky: 108,000,000 / 300 = 360,000 mas (= 360" = 6')
    let step_physical_mas = 360_000u64;
    let sub_idx = (pos_in_sign / step_physical_mas) as u16;
    let rem_physical_mas = pos_in_sign % step_physical_mas;

    // In D300 divisional space, this physical remainder is magnified 300x:
    let d300_pos_mas = rem_physical_mas.saturating_mul(300); // 0 .. 108,000,000 mas
    let dist_to_boundary = if d300_pos_mas > (SIGN_MAS / 2) {
        SIGN_MAS - d300_pos_mas
    } else {
        d300_pos_mas
    };

    let risk_code = if dist_to_boundary < 1_000 {
        CUSP_ZONE_CRITICAL // Less than 1" in D300 (physical margin < 3.33 milli-arcsec!)
    } else if dist_to_boundary < 60_000 {
        CUSP_ZONE_SENSITIVE // Less than 1' in D300 (physical margin < 200 milli-arcsec)
    } else {
        CUSP_ZONE_SAFE
    };

    let target_sign = ((rashi_sign as u16 + sub_idx) % 12) as u8 + 1;
    Some((target_sign, dist_to_boundary, risk_code))
}

/// Evaluates topocentric parallax error for Mercury/Moon relative to geocentric calculations.
/// Demonstrates why naive geocentric ephemeris shifts D300 by over 1 degree.
/// @outputs: 1
/// @domain: distance_au_fp4 [2000..=20000] (0.2000 AU .. 2.0000 AU)
pub fn astro_topocentric_parallax_max_arcsec(distance_au_fp4: u16) -> Option<u32> {
    if distance_au_fp4 < 2_000 || distance_au_fp4 > 20_000 {
        return None;
    }

    // Earth equatorial radius = 6378 km. 1 AU = 149,597,870 km.
    // Earth radius / 1 AU in arcsec = 6378 / 149597870 * 206264.8 = 8.794" arcsec.
    // Parallax at distance D (AU) = 8.794 / D = (87940 / distance_au_fp4) arcsec.
    let px_arcsec = (87_940u32 / (distance_au_fp4 as u32)) as u32;
    Some(px_arcsec)
}

/// Multiplies physical angular uncertainty by the Varga magnification factor.
/// @outputs: 1
/// @domain: physical_error_mas [0..=10000000]
/// @domain: varga_n [1..=300]
pub fn astro_varga_error_amplification(
    physical_error_mas: u32,
    varga_n: u16,
) -> Option<u64> {
    if !(1..=300).contains(&varga_n) {
        return None;
    }
    Some((physical_error_mas as u64).saturating_mul(varga_n as u64))
}
