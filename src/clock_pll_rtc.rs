// ## Map
// Role: core/clock_pll_rtc — 晶振 PLL 鎖相迴路分頻求解、RTC 32.768kHz 晶振溫漂補償與 NTP 網路時鐘對齊微積木。
// Inputs: 晶振輸入頻率、目標系統時脈、當前環境溫度、NTP 網路四元時戳 (t0..t3)。
// Outputs: PLL 最優配置元組 (M, N, P)、晶振 PPM 偏差與暫存器補償值、UNIX 秒數與網路時鐘偏移 (Offset, Delay)。
// Invariants: 零外部依賴、零 Heap 分配、無遞迴、零全域狀態、控制流巢狀深度 <= 2、RFC 5905 NTP 標竿演算法。

const NTP_TO_UNIX_EPOCH_DIFF: i64 = 2_208_988_800;

/// @outputs: 2
/// @domain: fin_hz [1000000..50000000]
/// @domain: fout_target_hz [1000000..500000000]
pub fn pll_calculate_prescalers(fin_hz: u32, fout_target_hz: u32) -> Option<(u32, u32, u32)> {
    if fin_hz < 1_000_000 || fout_target_hz < 1_000_000 || fout_target_hz > 500_000_000 {
        return None;
    }
    // 尋求 PLLM 使 fin / M 落在 1..2 MHz (常用 1MHz 或 2MHz 基準)
    let m = fin_hz / 1_000_000;
    if m < 2 || m > 63 {
        return None;
    }
    let ref_clock = fin_hz / m;

    let p_candidates = [2u32, 4u32, 6u32, 8u32];
    let mut best_tuple = None;
    let mut min_diff = u32::MAX;

    for &p in &p_candidates {
        let vco_target = fout_target_hz.saturating_mul(p);
        let n = (vco_target + ref_clock / 2) / ref_clock;
        if n >= 50 && n <= 432 {
            let actual_vco = ref_clock * n;
            let actual_fout = actual_vco / p;
            let diff = if actual_fout > fout_target_hz {
                actual_fout - fout_target_hz
            } else {
                fout_target_hz - actual_fout
            };
            if diff < min_diff {
                min_diff = diff;
                best_tuple = Some((m, n, p));
            }
        }
    }
    best_tuple
}

/// @outputs: 1
/// @domain: temp_celsius [-40.0..85.0]
/// @domain: turnover_temp [10.0..40.0]
/// @domain: parabolic_coeff [0.01..0.06]
pub fn rtc_32768_ppm_compensate(temp_celsius: f32, turnover_temp: f32, parabolic_coeff: f32) -> f32 {
    if temp_celsius.is_nan() || turnover_temp.is_nan() || parabolic_coeff.is_nan() {
        return 0.0;
    }
    let t = temp_celsius.clamp(-40.0, 85.0);
    let delta_t = t - turnover_temp;
    // 拋物線偏差公式: ppm = -k * (T - T0)^2
    -parabolic_coeff * (delta_t * delta_t)
}

/// @outputs: 1
/// @domain: ppm_error [-200.0..200.0]
/// @domain: ppm_per_step [0.1..10.0]
pub fn rtc_calibration_register_val(ppm_error: f32, ppm_per_step: f32) -> (bool, u8) {
    if ppm_error.is_nan() || ppm_per_step.is_nan() || ppm_per_step <= 0.0 {
        return (false, 0);
    }
    let is_negative = ppm_error < 0.0;
    let abs_ppm = ppm_error.abs();
    let steps = (abs_ppm / ppm_per_step).round();
    let reg_val = steps.clamp(0.0, 127.0) as u8;
    (is_negative, reg_val)
}

/// @outputs: 1
/// @domain: ntp_seconds [2208988800..4294967295]
pub fn ntp_timestamp_to_unix_secs(ntp_seconds: u32) -> i64 {
    let s = ntp_seconds as i64;
    if s < NTP_TO_UNIX_EPOCH_DIFF {
        return 0;
    }
    s - NTP_TO_UNIX_EPOCH_DIFF
}

/// @outputs: 1
pub fn ntp_clock_offset_and_delay(t0_ms: i64, t1_ms: i64, t2_ms: i64, t3_ms: i64) -> (i64, i64) {
    // RFC 5905 NTP:
    // Delay delta = (t3 - t0) - (t2 - t1)
    // Offset theta = ((t1 - t0) + (t2 - t3)) / 2
    let round_trip = (t3_ms.saturating_sub(t0_ms)).saturating_sub(t2_ms.saturating_sub(t1_ms));
    let offset = ((t1_ms.saturating_sub(t0_ms)) + (t2_ms.saturating_sub(t3_ms))) / 2;
    (offset, round_trip.max(0))
}
