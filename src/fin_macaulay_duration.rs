// ## Map
// Role: Calculator
// Inputs: cash_flows: &[f64], ytm: f64
// Outputs: (Macaulay Duration, Modified Duration)
// Invariants: cash_flows non‑empty, ytm ∈ [0.0 .. 1.0], bond price > 1e-12

/// @outputs: 2
/// @domain: ytm [0.0..1.0]
///
/// 計算債券的麥考利久期與修正久期。
/// 若輸入不符合規範或價格過小，回傳 `None`。
pub fn bond_duration(cash_flows: &[f64], ytm: f64) -> Option<(f64, f64)> {
    // domain guard for ytm
    if ytm < 0.0 || ytm > 1.0 {
        return None;
    }
    // cash flow guard
    if cash_flows.is_empty() {
        return None;
    }

    let mut price = 0.0_f64;
    let mut weighted = 0.0_f64;
    let one_plus_y = 1.0 + ytm;

    // iterate over periods t = 1 .. N
    for (idx, &cf) in cash_flows.iter().enumerate() {
        let t = (idx + 1) as i32;
        // discount factor = (1 + ytm)^t
        let df = one_plus_y.powi(t);
        // present value of cash flow
        let pv = cf / df;
        price += pv;
        weighted += (t as f64) * pv;
    }

    // price sanity check
    if price <= 1e-12 {
        return None;
    }

    let mac_duration = weighted / price;
    let mod_duration = mac_duration / one_plus_y;

    Some((mac_duration, mod_duration))
}
