# 模組名稱: 技術分析布林帶動態通道與百分比 B 指標 (Bollinger Bands & Percent B Indicator)

## 1. 目標定位
- 目標模組檔: `src/fin_bollinger_bands.rs`
- 目標測試檔: `tests/test_fin_bollinger_bands.rs`
- 功能簡述: 金融量化交易之經典布林通道 (Bollinger Bands) 純函式。輸入價格滑動視窗，無配置計算中軌 (SMA)、上軌、下軌、指標 %B 與帶寬 (Bandwidth)。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Bollinger Bands channel computation with %B and Bandwidth indicators
// Inputs: prices: &[f64], k: f64
// Outputs: Option<(f64, f64, f64, f64, f64)> (middle, upper, lower, percent_b, bandwidth)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 5
/// @domain: prices.len() in [2..=256], k > 0.0, prices > 0.0
pub fn fin_compute_bollinger_bands(
    prices: &[f64],
    k: f64,
) -> Option<(f64, f64, f64, f64, f64)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若 `prices.len() < 2 || prices.len() > 256 || k <= 0.0`，立即回傳 `None`。
- 檢查價格皆為正數與有限值，若任何 `p <= 0.0` 或含 `NaN` 立即回傳 `None`。
- 計算：
  - `let n = prices.len() as f64;`
  - `let middle = sum(prices) / n;`
  - 樣本標準差（或總體標準差）：`let variance = sum((p - middle)^2) / n;`
  - `let std_dev = variance.sqrt();`
  - 上軌：`let upper = middle + k * std_dev;`
  - 下軌：`let lower = middle - k * std_dev;`
  - 最新價格：`let latest = *prices.last().unwrap();`
  - 帶寬比率：`let bandwidth = (upper - lower) / middle;`
  - %B 指標：若 `(upper - lower) < 1e-12` 則 `percent_b = 0.5`，否則 `percent_b = (latest - lower) / (upper - lower)`。
- 回傳 `Some((middle, upper, lower, percent_b, bandwidth))`。

## 4. 確定性黃金值測試要求
- 確定性序列：
  - `prices = [10.0, 20.0], k = 1.0`
  - `middle = 15.0, variance = (25 + 25) / 2 = 25.0 -> std_dev = 5.0`
  - `upper = 20.0, lower = 10.0`
  - `latest = 20.0` -> `percent_b = (20 - 10) / 10 = 1.0`
  - `bandwidth = 10.0 / 15.0 = 0.6666666666666666`
- 負價格或序列過短拒絕。

```golden
let (mid, up, low, pb, bw) = fin_compute_bollinger_bands(&[10.0, 20.0], 1.0).unwrap();
(mid - 15.0).abs() < 1e-12
(up - 20.0).abs() < 1e-12
(low - 10.0).abs() < 1e-12
(pb - 1.0).abs() < 1e-12
(bw - (10.0 / 15.0)).abs() < 1e-12
fin_compute_bollinger_bands(&[10.0], 1.0).is_none()
fin_compute_bollinger_bands(&[10.0, -5.0], 1.0).is_none()
```
