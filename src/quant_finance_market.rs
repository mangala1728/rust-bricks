// ## Map
// Role: fin/quant_market — 量化金融與股市分析 (技術面 EMA/RSI/布林通道、基本面 PE/PB/ROE、籌碼面 VWAP/淨流入、期貨基差與匯率調整報酬) 微積木。
// Inputs: 價格切片、成交量、EPS、BPS、淨利與權益、期貨與現貨價、存續天數 (DTE)、匯率。
// Outputs: EMA 數值、RSI 強弱指標、布林軌道三線、PE/PB/ROE/FCF Yield、VWAP、大單淨流入率、基差年化殖利率、匯率調整報酬率。
// Invariants: 零外部依賴、純 Rust 實現、除零剛性防護、無溢位、單一函式 <= 60 行。

/// @outputs: 1
/// @domain: period [1..1000]
pub fn quant_ema_step(prev_ema: f64, current_price: f64, period: usize) -> Option<f64> {
    if period == 0 || !prev_ema.is_finite() || !current_price.is_finite() {
        return None;
    }
    let alpha = 2.0 / ((period as f64) + 1.0);
    Some(alpha * current_price + (1.0 - alpha) * prev_ema)
}

/// @outputs: 1
/// @domain: avg_gain [0.0..1.0e9]
/// @domain: avg_loss [0.0..1.0e9]
pub fn quant_rsi_step(avg_gain: f64, avg_loss: f64) -> Option<f64> {
    if avg_gain < 0.0 || avg_loss < 0.0 || !avg_gain.is_finite() || !avg_loss.is_finite() {
        return None;
    }
    if avg_loss <= 1e-12 {
        return Some(100.0);
    }
    let rs = avg_gain / avg_loss;
    Some(100.0 - (100.0 / (1.0 + rs)))
}

/// @outputs: 3
/// @domain: k_mult [0.1..10.0]
pub fn quant_bollinger_bands(prices: &[f64], k_mult: f64) -> Option<(f64, f64, f64)> {
    let n = prices.len();
    if n < 2 || k_mult <= 0.0 || !k_mult.is_finite() {
        return None;
    }
    let mut sum = 0.0;
    for &p in prices {
        if !p.is_finite() { return None; }
        sum += p;
    }
    let sma = sum / (n as f64);
    let mut var_sum = 0.0;
    for &p in prices {
        let diff = p - sma;
        var_sum += diff * diff;
    }
    let std_dev = (var_sum / ((n - 1) as f64)).sqrt();
    let upper = sma + k_mult * std_dev;
    let lower = sma - k_mult * std_dev;
    Some((upper, sma, lower))
}

/// @outputs: 1
pub fn quant_macd_line(fast_ema: f64, slow_ema: f64) -> f64 {
    if !fast_ema.is_finite() || !slow_ema.is_finite() {
        return 0.0;
    }
    fast_ema - slow_ema
}

/// @outputs: 2
/// @domain: price [0.001..1.0e9]
pub fn quant_pe_pb_ratio(price: f64, eps: f64, bps: f64) -> Option<(f64, f64)> {
    if price <= 0.0 || eps.abs() <= 1e-6 || bps <= 1e-6 || !price.is_finite() || !eps.is_finite() || !bps.is_finite() {
        return None;
    }
    let pe = price / eps;
    let pb = price / bps;
    Some((pe, pb))
}

/// @outputs: 2
/// @domain: equity [1.0..1.0e15]
/// @domain: market_cap [1.0..1.0e15]
pub fn quant_roe_and_fcf_yield(
    net_income: f64,
    equity: f64,
    free_cash_flow: f64,
    market_cap: f64,
) -> Option<(f64, f64)> {
    if equity <= 0.0 || market_cap <= 0.0 || !net_income.is_finite() || !free_cash_flow.is_finite() {
        return None;
    }
    let roe_pct = (net_income / equity) * 100.0;
    let fcf_yield_pct = (free_cash_flow / market_cap) * 100.0;
    Some((roe_pct, fcf_yield_pct))
}

/// @outputs: 3
/// @domain: price [0.0001..1.0e9]
/// @domain: volume [0.0001..1.0e12]
pub fn quant_vwap_step(
    prev_cum_vp: f64,
    prev_cum_vol: f64,
    price: f64,
    volume: f64,
) -> Option<(f64, f64, f64)> {
    if price <= 0.0 || volume <= 0.0 || prev_cum_vp < 0.0 || prev_cum_vol < 0.0
        || !price.is_finite() || !volume.is_finite() {
        return None;
    }
    let cum_vp = prev_cum_vp + price * volume;
    let cum_vol = prev_cum_vol + volume;
    if cum_vol <= 1e-12 {
        return None;
    }
    let vwap = cum_vp / cum_vol;
    Some((cum_vp, cum_vol, vwap))
}

/// @outputs: 1
/// @domain: buy_volume [0.0..1.0e12]
/// @domain: sell_volume [0.0..1.0e12]
pub fn quant_order_flow_net_inflow_ratio(buy_volume: f64, sell_volume: f64) -> Option<f64> {
    if buy_volume < 0.0 || sell_volume < 0.0 || !buy_volume.is_finite() || !sell_volume.is_finite() {
        return None;
    }
    let total_vol = buy_volume + sell_volume;
    if total_vol <= 1e-12 {
        return None;
    }
    let net = buy_volume - sell_volume;
    Some((net / total_vol) * 100.0)
}

/// @outputs: 2
pub fn quant_futures_basis_spread(futures_price: f64, spot_price: f64) -> (f64, bool) {
    if !futures_price.is_finite() || !spot_price.is_finite() {
        return (0.0, false);
    }
    let basis = futures_price - spot_price;
    let is_contango = basis >= 0.0;
    (basis, is_contango)
}

/// @outputs: 1
/// @domain: spot_price [0.001..1.0e9]
/// @domain: dte_days [0.1..3650.0]
pub fn quant_futures_annualized_carry_yield(
    futures_price: f64,
    spot_price: f64,
    dte_days: f64,
) -> Option<f64> {
    if spot_price <= 0.0 || dte_days <= 0.0 || !futures_price.is_finite() || !spot_price.is_finite() || !dte_days.is_finite() {
        return None;
    }
    let basis = futures_price - spot_price;
    let raw_return = basis / spot_price;
    let annualized = raw_return * (365.0 / dte_days) * 100.0;
    Some(annualized)
}

/// @outputs: 1
/// @domain: fx_start [0.0001..1.0e6]
/// @domain: fx_end [0.0001..1.0e6]
pub fn quant_fx_adjusted_return(
    local_return: f64,
    fx_start: f64,
    fx_end: f64,
) -> Option<f64> {
    if fx_start <= 0.0 || fx_end <= 0.0 || !local_return.is_finite() || !fx_start.is_finite() || !fx_end.is_finite() {
        return None;
    }
    // (1 + R_local) * (FX_end / FX_start) - 1
    let gross_local = 1.0 + local_return;
    let fx_multiplier = fx_end / fx_start;
    Some(gross_local * fx_multiplier - 1.0)
}
