// ## Map
// Role: core/math_time — 基礎數值夾止、移動平均、天文儒略日與占星大限時間比例運算。
// Inputs: 數值、時間戳、年月日時、視窗切片。
// Outputs: 夾止數值、浮點均值、儒略日、大限天數比例。
// Invariants: 零全域可變狀態、無遞迴、除數防零守衛、有界迴圈（最大 100,000 點）。

/// @outputs: 3
/// @domain: val [-1000000000000..1000000000000]
/// @domain: min_val [-1000000000000..1000000000000]
/// @domain: max_val [-1000000000000..1000000000000]
pub fn clamp_i64(val: i64, min_val: i64, max_val: i64) -> i64 {
    if min_val > max_val {
        return val;
    }
    if val < -1_000_000_000_000 || val > 1_000_000_000_000 {
        return 0;
    }
    if val < min_val {
        min_val
    } else if val > max_val {
        max_val
    } else {
        val
    }
}

/// @outputs: 2
pub fn moving_average_f64(window: &[f64]) -> Option<f64> {
    if window.is_empty() || window.len() > 100_000 {
        return None;
    }
    let mut sum: f64 = 0.0;
    for &val in window {
        if val.is_nan() || val.is_infinite() {
            return None;
        }
        sum += val;
    }
    Some(sum / (window.len() as f64))
}

/// @outputs: 2
/// @domain: year [-4713..9999]
/// @domain: month [1..12]
/// @domain: day [1..31]
pub fn julian_day_from_ymd(year: i32, month: u32, day: u32) -> Option<f64> {
    if year < -4713 || year > 9999 || month < 1 || month > 12 || day < 1 || day > 31 {
        return None;
    }
    let mut y = year as i64;
    let mut m = month as i64;
    if m <= 2 {
        y -= 1;
        m += 12;
    }
    let a = y / 100;
    let b = 2 - a + (a / 4);
    let jd = (365.25 * ((y + 4716) as f64)).floor()
        + (30.6001 * ((m + 1) as f64)).floor()
        + (day as f64)
        + (b as f64)
        - 1524.5;
    Some(jd)
}

/// @outputs: 2
/// @domain: total_span_days [0.001..365000.0]
/// @domain: sub_period_years [0.001..200.0]
/// @domain: total_dasha_years [0.001..200.0]
pub fn dasha_proportion_days(total_span_days: f64, sub_period_years: f64, total_dasha_years: f64) -> Option<f64> {
    if total_span_days <= 0.0 || sub_period_years <= 0.0 || total_dasha_years <= 0.0 {
        return None;
    }
    if total_dasha_years.is_nan() || sub_period_years.is_nan() || total_span_days.is_nan() {
        return None;
    }
    let days = (total_span_days * sub_period_years) / total_dasha_years;
    Some(days)
}

/// @outputs: 1
/// @domain: state [0..18446744073709551615]
pub fn xorshift64_step(state: u64) -> u64 {
    let mut x = if state == 0 { 0x853c49e6748fea9b } else { state };
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}
