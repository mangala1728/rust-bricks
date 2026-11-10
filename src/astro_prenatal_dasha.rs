// SPDX-License-Identifier: LicenseRef-Project-License
//! ## Map
//! - Role: Pre-natal Dasha reverse-progression & Varga (D9/D12) divisional rectification
//! - Inputs:
//!   - birth_maha: u8 (1..9), birth_balance_days: u32 (0..7305), elapsed_pre_natal_days: u32 (0..43830)
//!   - sidereal_lon_milli_deg: u32 (0..360000), varga_factor: u8 (1..60)
//! - Outputs:
//!   - Option<(u8, u32)>: (pre_natal_maha [1..9], remaining_days_in_maha)
//!   - Option<u8>: Varga sign index [1..12] (1=Aries .. 12=Pisces)
//! - Invariants:
//!   1. Strict 09 contract boundary & NASA JPL <= 60 lines per function.
//!   2. Bounded while loop (max 9 iterations) for reverse dasha stepping.
//!   3. Pure integer arithmetic, zero floating-point calculations.

/// Vimshottari period durations in days (Sum = 43,830 days = 120 Julian years of 365.25d).
/// Index 0 is unused; 1=Ketu, 2=Venus, 3=Sun, 4=Moon, 5=Mars, 6=Rahu, 7=Jupiter, 8=Saturn, 9=Mercury.
const VIMSHOTTARI_DAYS: [u32; 10] = [
    0, 2_557, 7_305, 2_191, 3_652, 2_557, 6_575, 5_844, 6_940, 6_209,
];

/// Steps backward in time before birth to find the active Vimshottari Mahadasha at a pre-natal event.
/// @outputs: 1
/// @domain: birth_maha [1..=9]
/// @domain: birth_balance_days [0..=7305]
/// @domain: elapsed_pre_natal_days [0..=43830]
pub fn astro_vimshottari_reverse_step(
    birth_maha: u8,
    birth_balance_days: u32,
    elapsed_pre_natal_days: u32,
) -> Option<(u8, u32)> {
    if !(1..=9).contains(&birth_maha)
        || birth_balance_days > 7_305
        || elapsed_pre_natal_days > 43_830
    {
        return None;
    }

    let full_duration = VIMSHOTTARI_DAYS[birth_maha as usize];
    let spent_before_birth = full_duration.saturating_sub(birth_balance_days);

    if elapsed_pre_natal_days <= spent_before_birth {
        let remaining_at_event = birth_balance_days.saturating_add(elapsed_pre_natal_days);
        return Some((birth_maha, remaining_at_event));
    }

    let mut remaining_pre_natal = elapsed_pre_natal_days - spent_before_birth;
    let mut current_maha = birth_maha;
    let mut guard_count = 0u8;

    while guard_count < 10 {
        current_maha = if current_maha == 1 { 9 } else { current_maha - 1 };
        let prev_duration = VIMSHOTTARI_DAYS[current_maha as usize];
        if remaining_pre_natal <= prev_duration {
            let rem_in_period = prev_duration - remaining_pre_natal;
            return Some((current_maha, rem_in_period));
        }
        remaining_pre_natal -= prev_duration;
        guard_count += 1;
    }

    None
}

/// Computes the divisional chart (Varga) sign placement (1=Aries .. 12=Pisces).
/// Supports D1 (Rashi), D9 (Navamsha for marriage/Dharma), D12 (Dwadasamsa for parents/grandparents).
/// @outputs: 1
/// @domain: sidereal_lon_milli_deg [0..=360000]
/// @domain: varga_factor [1..=60]
pub fn astro_btr_varga_subdivision(
    sidereal_lon_milli_deg: u32,
    varga_factor: u8,
) -> Option<u8> {
    if sidereal_lon_milli_deg > 360_000 || !(1..=60).contains(&varga_factor) {
        return None;
    }

    let clamped_lon = sidereal_lon_milli_deg % 360_000;
    let rashi_sign = (clamped_lon / 30_000) as u8; // 0=Aries .. 11=Pisces
    let pos_in_sign = clamped_lon % 30_000;
    let sub_idx = ((pos_in_sign as u64 * varga_factor as u64) / 30_000) as u8;

    let target_sign_0 = if varga_factor == 9 {
        // D9 Navamsha: Movable (0,3,6,9) starts at sign; Fixed (1,4,7,10) starts at 9th; Dual starts at 5th
        let sign_mod4 = rashi_sign % 4;
        let start_offset = match sign_mod4 {
            0 => rashi_sign,
            1 => (rashi_sign + 8) % 12,
            2 => (rashi_sign + 4) % 12,
            _ => (rashi_sign + 8) % 12,
        };
        (start_offset + sub_idx) % 12
    } else if varga_factor == 12 {
        // D12 Dwadasamsa: Always starts from the sign itself
        (rashi_sign + sub_idx) % 12
    } else {
        // General linear varga projection (D1, D60 cyclic)
        (rashi_sign + sub_idx) % 12
    };

    Some(target_sign_0 + 1)
}
