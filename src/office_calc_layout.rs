// ## Map
// Role: office/office_calc_layout — 零依賴 Excel 核心財務/統計/檢索算子與 Word/PPT 排版版面幾何控制微積木。
// Inputs: 檢索鍵值與陣列、利率與期數、現金流切片、百分位數、排版磅數/英吋、文字寬度與投影片尺寸。
// Outputs: XLOOKUP 查表結果、PMT 每期攤還本息、NPV 淨現值、Twips/EMUs 單位、折行行數、等距分佈座標。
// Invariants: 零外部依賴、純 Rust 實現、除零剛性防護、折現率合法邊界、單一函式 <= 60 行。

/// @outputs: 2
/// @domain: lookup_array.len() [1..65536]
pub fn excel_xlookup_f64(lookup_val: f64, lookup_array: &[f64], return_array: &[f64], exact_match: bool) -> Option<f64> {
    if lookup_array.is_empty() || lookup_array.len() != return_array.len() || !lookup_val.is_finite() {
        return None;
    }
    if exact_match {
        for (i, &v) in lookup_array.iter().enumerate() {
            if (v - lookup_val).abs() < 1e-9 {
                return Some(return_array[i]);
            }
        }
        None
    } else {
        // 近似匹配：尋找小於等於 lookup_val 中最接近者
        let mut best_idx = None;
        let mut best_diff = f64::MAX;
        for (i, &v) in lookup_array.iter().enumerate() {
            if v <= lookup_val {
                let diff = lookup_val - v;
                if diff < best_diff {
                    best_diff = diff;
                    best_idx = Some(i);
                }
            }
        }
        best_idx.map(|idx| return_array[idx])
    }
}

/// @outputs: 2
/// @domain: rate_per_period [0.0..10.0]
/// @domain: num_periods [1..1200]
pub fn excel_pmt_loan(rate_per_period: f64, num_periods: u32, present_value: f64) -> Option<f64> {
    if num_periods == 0 || present_value < 0.0 || rate_per_period < 0.0 || !present_value.is_finite() || !rate_per_period.is_finite() {
        return None;
    }
    if rate_per_period == 0.0 {
        return Some(present_value / (num_periods as f64));
    }
    let factor = (1.0 + rate_per_period).powi(-(num_periods as i32));
    let denom = 1.0 - factor;
    if denom <= 1e-12 {
        return None;
    }
    let pmt = (rate_per_period * present_value) / denom;
    if pmt.is_finite() { Some(pmt) } else { None }
}

/// @outputs: 2
/// @domain: rate [-0.999..100.0]
/// @domain: cash_flows.len() [1..1000]
pub fn excel_npv_discounted(rate: f64, cash_flows: &[f64]) -> Option<f64> {
    if rate <= -1.0 || cash_flows.is_empty() || cash_flows.len() > 1000 || !rate.is_finite() {
        return None;
    }
    let mut npv = 0.0;
    let mut denom = 1.0;
    let factor = 1.0 + rate;
    for &cf in cash_flows {
        if !cf.is_finite() {
            return None;
        }
        denom *= factor;
        if denom == 0.0 || !denom.is_finite() {
            return None;
        }
        npv += cf / denom;
    }
    if npv.is_finite() { Some(npv) } else { None }
}

/// @outputs: 2
/// @domain: percentile [0.0..1.0]
pub fn excel_percentile_linear(sorted_vals: &[f64], percentile: f64) -> Option<f64> {
    let n = sorted_vals.len();
    if n == 0 || percentile < 0.0 || percentile > 1.0 || !percentile.is_finite() {
        return None;
    }
    if n == 1 {
        return Some(sorted_vals[0]);
    }
    let rank = (n - 1) as f64 * percentile;
    let lower_idx = rank.floor() as usize;
    let frac = rank - (lower_idx as f64);
    let v_low = sorted_vals[lower_idx];
    if lower_idx + 1 < n {
        let v_high = sorted_vals[lower_idx + 1];
        Some(v_low + frac * (v_high - v_low))
    } else {
        Some(v_low)
    }
}

/// @outputs: 1
/// @domain: pt [0.0..10000.0]
pub fn office_units_pt_to_twips(pt: f64) -> i64 {
    if pt <= 0.0 || !pt.is_finite() {
        return 0;
    }
    (pt * 20.0).round() as i64
}

/// @outputs: 1
/// @domain: inches [0.0..1000.0]
pub fn office_units_inches_to_emus(inches: f64) -> i64 {
    if inches <= 0.0 || !inches.is_finite() {
        return 0;
    }
    (inches * 914_400.0).round() as i64
}

/// @outputs: 1
/// @domain: max_line_width_px [1..100000]
pub fn word_greedy_wrap_line_count(word_widths_px: &[u32], max_line_width_px: u32, space_width_px: u32) -> u32 {
    if word_widths_px.is_empty() || max_line_width_px == 0 {
        return 0;
    }
    let mut lines = 1u32;
    let mut current_line_width = 0u32;

    for &w in word_widths_px {
        let word_cost = if current_line_width == 0 { w } else { space_width_px + w };
        if current_line_width + word_cost <= max_line_width_px {
            current_line_width += word_cost;
        } else {
            lines += 1;
            current_line_width = w.min(max_line_width_px);
        }
    }
    lines
}

/// @outputs: 2
/// @domain: total_span [0.0..10000.0]
pub fn ppt_distribute_elements_1d(element_sizes: &[f32], total_span: f32, out_positions: &mut [f32]) -> bool {
    let n = element_sizes.len();
    if n == 0 || out_positions.len() != n || total_span <= 0.0 || !total_span.is_finite() {
        return false;
    }
    let mut sum_sizes = 0.0f32;
    for &s in element_sizes {
        if s < 0.0 || !s.is_finite() {
            return false;
        }
        sum_sizes += s;
    }
    if sum_sizes > total_span {
        return false;
    }
    let remaining_space = total_span - sum_sizes;
    let gap = if n > 1 { remaining_space / ((n - 1) as f32) } else { remaining_space * 0.5 };

    let mut current_pos = if n == 1 { gap } else { 0.0 };
    for i in 0..n {
        out_positions[i] = current_pos;
        current_pos += element_sizes[i] + gap;
    }
    true
}
