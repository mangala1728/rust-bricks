//! # Quant L2 Order Book & Market Microstructure Micro-Bricks (量化高頻訂單簿與微觀結構微積木)
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 L2 最優買賣價 (BBO) 中間價與基點價差、微觀訂單不平衡度 (OBI)、
//! 交易量加權微觀價格 (Stoikov Micro-Price)、多檔深度加權中間價、有效價差、
//! 實現價差 (Realized Spread)、Kyle's Lambda 價格衝擊係數、Amihud 非流動性比率、
//! VPIN 毒性訂單流指標步進、TWAP 時間加權平均價。

/// ## Map: L2 最優買賣報價 (BBO) 中間價與絕對/相對價差 (基點 bps)
/// @outputs: 3
/// @domain: best_bid [0.0001, 1e9]
/// @domain: best_ask [0.0001, 1e9]
pub fn quant_l2_bbo_mid_and_spread(best_bid: f64, best_ask: f64) -> Option<(f64, f64, f64)> {
    if !best_bid.is_finite() || !best_ask.is_finite() || best_bid <= 0.0 || best_ask <= 0.0 || best_bid > best_ask {
        return None;
    }
    let mid_price = (best_bid + best_ask) * 0.5;
    let abs_spread = best_ask - best_bid;
    let rel_spread_bps = (abs_spread / mid_price) * 10000.0;
    Some((mid_price, abs_spread, rel_spread_bps))
}

/// ## Calc: 盤口訂單微觀不平衡度 (Order Book Imbalance, OBI)
/// OBI = (Q_bid - Q_ask) / (Q_bid + Q_ask), 範圍 [-1.0, 1.0]
/// @outputs: 1
/// @domain: bid_qty [0.0, 1e12]
/// @domain: ask_qty [0.0, 1e12]
pub fn quant_l2_order_book_imbalance_obi(bid_qty: f64, ask_qty: f64) -> Option<f64> {
    if !bid_qty.is_finite() || !ask_qty.is_finite() || bid_qty < 0.0 || ask_qty < 0.0 {
        return None;
    }
    let total_qty = bid_qty + ask_qty;
    if total_qty < 1e-9 {
        return None;
    }
    let obi = (bid_qty - ask_qty) / total_qty;
    Some(obi.clamp(-1.0, 1.0))
}

/// ## Price: 交易量加權微觀價格 (Stoikov Micro-Price)
/// P_micro = (Q_bid * P_ask + Q_ask * P_bid) / (Q_bid + Q_ask)
/// @outputs: 1
/// @domain: best_bid [0.0001, 1e9]
/// @domain: best_ask [0.0001, 1e9]
/// @domain: bid_qty [0.0, 1e12]
/// @domain: ask_qty [0.0, 1e12]
pub fn quant_l2_weighted_micro_price(
    best_bid: f64,
    best_ask: f64,
    bid_qty: f64,
    ask_qty: f64,
) -> Option<f64> {
    if !best_bid.is_finite() || !best_ask.is_finite() || !bid_qty.is_finite() || !ask_qty.is_finite() ||
       best_bid <= 0.0 || best_ask <= 0.0 || best_bid > best_ask || bid_qty < 0.0 || ask_qty < 0.0 {
        return None;
    }
    let total_qty = bid_qty + ask_qty;
    if total_qty < 1e-9 {
        return None;
    }
    let micro_price = (bid_qty * best_ask + ask_qty * best_bid) / total_qty;
    Some(micro_price)
}

/// ## Price: 多檔深度指數衰減加權中間價 (Multi-Level Depth-Weighted Mid-Price)
/// @outputs: 1
pub fn quant_l2_multi_level_depth_weighted_mid(
    bids: &[(f64, f64)],
    asks: &[(f64, f64)],
    levels: usize,
) -> Option<f64> {
    if bids.is_empty() || asks.is_empty() || levels == 0 {
        return None;
    }
    let n_levels = levels.min(bids.len()).min(asks.len());
    let mut sum_bid_val = 0.0f64;
    let mut sum_bid_qty = 0.0f64;
    let mut sum_ask_val = 0.0f64;
    let mut sum_ask_qty = 0.0f64;

    for i in 0..n_levels {
        let (p_bid, q_bid) = bids[i];
        let (p_ask, q_ask) = asks[i];
        if !p_bid.is_finite() || !q_bid.is_finite() || !p_ask.is_finite() || !q_ask.is_finite() ||
           p_bid <= 0.0 || q_bid < 0.0 || p_ask <= 0.0 || q_ask < 0.0 {
            return None;
        }
        sum_bid_val += p_bid * q_bid;
        sum_bid_qty += q_bid;
        sum_ask_val += p_ask * q_ask;
        sum_ask_qty += q_ask;
    }
    if sum_bid_qty < 1e-9 || sum_ask_qty < 1e-9 {
        return None;
    }
    let avg_bid = sum_bid_val / sum_bid_qty;
    let avg_ask = sum_ask_val / sum_ask_qty;
    Some((avg_bid + avg_ask) * 0.5)
}

/// ## Calc: 有效價差 (Effective Spread = 2 * |P_trade - P_mid|)
/// @outputs: 1
/// @domain: trade_price [0.0001, 1e9]
/// @domain: mid_price [0.0001, 1e9]
pub fn quant_effective_spread(trade_price: f64, mid_price: f64) -> f64 {
    if !trade_price.is_finite() || !mid_price.is_finite() || trade_price <= 0.0 || mid_price <= 0.0 {
        return 0.0;
    }
    2.0 * (trade_price - mid_price).abs()
}

/// ## Calc: 實現價差 (Realized Spread = 2 * Direction * (P_trade - P_future_mid))
/// @outputs: 1
/// @domain: trade_price [0.0001, 1e9]
/// @domain: future_mid_price [0.0001, 1e9]
pub fn quant_realized_spread(
    trade_price: f64,
    future_mid_price: f64,
    is_buyer_initiated: bool,
) -> f64 {
    if !trade_price.is_finite() || !future_mid_price.is_finite() || trade_price <= 0.0 || future_mid_price <= 0.0 {
        return 0.0;
    }
    let direction = if is_buyer_initiated { 1.0 } else { -1.0 };
    2.0 * direction * (trade_price - future_mid_price)
}

/// ## Fit: Kyle's Lambda 價格衝擊係數 (Price Impact = delta_price / order_flow)
/// @outputs: 1
/// @domain: delta_price [-1e6, 1e6]
/// @domain: order_flow [-1e12, 1e12]
pub fn quant_kyle_lambda_price_impact(delta_price: f64, order_flow: f64) -> Option<f64> {
    if !delta_price.is_finite() || !order_flow.is_finite() || order_flow.abs() < 1e-9 {
        return None;
    }
    Some(delta_price / order_flow)
}

/// ## Measure: Amihud 非流動性比率 (Illiquidity = |Return| / Dollar_Volume)
/// @outputs: 1
/// @domain: abs_return [0.0, 100.0]
/// @domain: dollar_volume [0.0, 1e15]
pub fn quant_amihud_illiquidity_ratio(abs_return: f64, dollar_volume: f64) -> Option<f64> {
    if !abs_return.is_finite() || !dollar_volume.is_finite() || abs_return < 0.0 || dollar_volume <= 1e-6 {
        return None;
    }
    Some(abs_return / dollar_volume)
}

/// ## Step: VPIN 毒性訂單流步進 (Volume-Synchronized Probability of Toxicity Bucket Step)
/// @outputs: 1
/// @domain: buy_volume [0.0, 1e12]
/// @domain: sell_volume [0.0, 1e12]
/// @domain: bucket_size [1.0, 1e12]
pub fn quant_vpin_bucket_toxicity_step(
    buy_volume: f64,
    sell_volume: f64,
    bucket_size: f64,
) -> Option<f64> {
    if !buy_volume.is_finite() || !sell_volume.is_finite() || !bucket_size.is_finite() ||
       buy_volume < 0.0 || sell_volume < 0.0 || bucket_size <= 0.0 {
        return None;
    }
    let imbalance = (buy_volume - sell_volume).abs();
    Some(imbalance / bucket_size)
}

/// ## Calc: 時間加權平均價步進 (TWAP Step = Cumulative Price-Time / Elapsed Time)
/// @outputs: 1
/// @domain: cumulative_price_time [0.0, 1e15]
/// @domain: elapsed_time_s [0.001, 1e8]
pub fn quant_twap_step(cumulative_price_time: f64, elapsed_time_s: f64) -> Option<f64> {
    if !cumulative_price_time.is_finite() || !elapsed_time_s.is_finite() ||
       cumulative_price_time < 0.0 || elapsed_time_s <= 0.0 {
        return None;
    }
    Some(cumulative_price_time / elapsed_time_s)
}
