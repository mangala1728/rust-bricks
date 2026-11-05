# 模組名稱: 二分法隱含波動率數值求解器 (Implied Volatility Bisection)

## 1. 目標定位
- 目標模組檔: `src/fin_implied_volatility.rs`
- 目標測試檔: `tests/test_fin_implied_volatility.rs`
- 功能簡述: 在金融衍生品期權造市與微觀結構分析中，透過單調二分搜尋法對 Black-Scholes 歐式買權定價公式反推市場隱含波動率（Implied Volatility, IV）。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Black-Scholes analytical call pricing and bisection implied volatility inversion
// Inputs: s, k, r, t, sigma, market_price, tol: f64, max_iter: u32
// Outputs: Option<f64>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: s > 0.0, k > 0.0, t > 0.0, sigma > 0.0, r is finite
pub fn fin_bs_call_price(s: f64, k: f64, r: f64, t: f64, sigma: f64) -> Option<f64>;

/// @outputs: 1
/// @domain: s > 0.0, k > 0.0, t > 0.0, tol > 0.0, market_price > 0.0, max_iter >= 1
pub fn fin_implied_volatility_bisection(
    market_price: f64,
    s: f64,
    k: f64,
    r: f64,
    t: f64,
    tol: f64,
    max_iter: u32,
) -> Option<f64>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若任何參數非有限數、`s <= 0.0 || k <= 0.0 || t <= 0.0 || market_price <= 0.0 || tol <= 0.0 || max_iter == 0`，立即回傳 `None`。
- 無套利下界檢查：買權市場價格必須滿足 $\text{market\_price} \ge \max(0.0, s - k \times e^{-r \times t})$ 且 $\text{market\_price} < s$；否則回傳 `None`。
- 二分區間：$\sigma_{\text{low}} = 0.0001, \sigma_{\text{high}} = 5.0$。

## 4. 確定性黃金值測試要求
- 正向定價基準：`s=100.0, k=100.0, r=0.05, t=1.0, sigma=0.20`
  理論 Black-Scholes 歐式買權價格 $\approx 10.45058$
- 逆向反推 IV：輸入 `market_price = 10.45058`，求得之隱含波動率必收斂至 `0.20`（容差 $< 10^{-3}$）。

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_fin_implied_volatility.rs）
```golden
let p = fin_bs_call_price(100.0, 100.0, 0.05, 1.0, 0.20).unwrap();
(p - 10.45058).abs() < 1e-4
let iv = fin_implied_volatility_bisection(10.45058, 100.0, 100.0, 0.05, 1.0, 1e-5, 50).unwrap();
(iv - 0.20).abs() < 1e-3
fin_implied_volatility_bisection(150.0, 100.0, 100.0, 0.05, 1.0, 1e-5, 50).is_none()
fin_implied_volatility_bisection(-1.0, 100.0, 100.0, 0.05, 1.0, 1e-5, 50).is_none()
```
