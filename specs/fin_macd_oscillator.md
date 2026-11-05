# 模組名稱: MACD 指標單步狀態轉移與柱狀圖運算器 (MACD Single Step Oscillator)

## 1. 目標定位
- 目標模組檔: `src/fin_macd_oscillator.rs`
- 目標測試檔: `tests/test_fin_macd_oscillator.rs`
- 功能簡述: 金融交易異同移動平均線 (MACD) 遞迴單步狀態機。自快線 EMA、慢線 EMA 與訊號線 EMA 狀態中計算 MACD 差離值 (DIF)、訊號線 (DEM) 與能量柱狀圖 (Histogram / OSC)。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Single-step MACD line, signal line, and histogram recursive computation
// Inputs: price, prev_fast, prev_slow, prev_signal, a_fast, a_slow, a_signal: f64
// Outputs: Option<(f64, f64, f64, f64, f64)> (macd_line, signal_line, histogram, next_fast, next_slow)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 5
/// @domain: price > 0.0, 0.0 < a_slow < a_fast <= 1.0, 0.0 < a_signal <= 1.0
pub fn fin_macd_step(
    price: f64,
    prev_fast: f64,
    prev_slow: f64,
    prev_signal: f64,
    a_fast: f64,
    a_slow: f64,
    a_signal: f64,
) -> Option<(f64, f64, f64, f64, f64)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若任何參數非有限浮點數，立即回傳 `None`。
- 若 `price <= 0.0 || a_slow <= 0.0 || a_slow >= a_fast || a_fast > 1.0 || a_signal <= 0.0 || a_signal > 1.0`，立即回傳 `None`。
- 計算：
  - `next_fast = a_fast * price + (1.0 - a_fast) * prev_fast`
  - `next_slow = a_slow * price + (1.0 - a_slow) * prev_slow`
  - `macd_line = next_fast - next_slow`
  - `signal_line = a_signal * macd_line + (1.0 - a_signal) * prev_signal`
  - `histogram = macd_line - signal_line`
- 回傳 `Some((macd_line, signal_line, histogram, next_fast, next_slow))`。

## 4. 確定性黃金值測試要求
- 確定性步進測試：
  - `price = 100.0, prev_fast = 90.0, prev_slow = 80.0, prev_signal = 5.0`
  - `a_fast = 0.5, a_slow = 0.2, a_signal = 0.5`
  - `next_fast = 0.5*100 + 0.5*90 = 95.0`
  - `next_slow = 0.2*100 + 0.8*80 = 20 + 64 = 84.0`
  - `macd_line = 95.0 - 84.0 = 11.0`
  - `signal_line = 0.5*11.0 + 0.5*5.0 = 5.5 + 2.5 = 8.0`
  - `histogram = 11.0 - 8.0 = 3.0`
- 慢線係數大於快線係數拒絕。

```golden
let res = fin_macd_step(100.0, 90.0, 80.0, 5.0, 0.5, 0.2, 0.5).unwrap();
(res.0 - 11.0).abs() < 1e-12
(res.1 - 8.0).abs() < 1e-12
(res.2 - 3.0).abs() < 1e-12
(res.3 - 95.0).abs() < 1e-12
(res.4 - 84.0).abs() < 1e-12
fin_macd_step(100.0, 90.0, 80.0, 5.0, 0.2, 0.5, 0.5).is_none()
fin_macd_step(-10.0, 90.0, 80.0, 5.0, 0.5, 0.2, 0.5).is_none()
```
