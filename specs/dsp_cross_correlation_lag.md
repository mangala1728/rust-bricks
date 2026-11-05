# 模組名稱: 離散訊號互相關最大延遲 TDOA 估算器 (Cross-Correlation Peak Lag / TDOA Estimator)

## 1. 目標定位
- 目標模組檔: `src/dsp_cross_correlation_lag.rs`
- 目標測試檔: `tests/test_dsp_cross_correlation_lag.rs`
- 功能簡述: 聲源定位 (DOA)、聲波到達時間差 (TDOA) 與雷達回波匹配中，計算兩組離散取樣序列在指定搜尋窗內之互相關峰值與最佳時延 (Peak Lag)。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Cross-correlation peak lag and TDOA time-delay estimator
// Inputs: x: &[f64], y: &[f64], max_lag: usize
// Outputs: Option<(i32, f64)> (best_lag, peak_correlation)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 2
/// @domain: x.len() >= 4, y.len() >= 4, max_lag >= 1 && max_lag < min(x.len(), y.len())
pub fn dsp_find_peak_cross_correlation_lag(
    x: &[f64],
    y: &[f64],
    max_lag: usize,
) -> Option<(i32, f64)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若 `x.len() < 4 || y.len() < 4`，立即回傳 `None`。
- 若 `max_lag == 0 || max_lag >= x.len() || max_lag >= y.len()`，立即回傳 `None`。
- 檢查序列內容是否全為有限浮點數，若含 `NaN` 立即回傳 `None`。
- 計算：
  - 在時延 $\tau \in [-(max\_lag \text{ as } i32) ..= (max\_lag \text{ as } i32)]$ 範圍內搜尋：
  - 對重疊區間 $n$ 累加乘積 $x[n] \times y[n + \tau]$。
  - 當 $n \ge 0$ 且 $n < x.len()$ 且 $n + \tau \ge 0$ 且 $n + \tau < y.len()$ 時累加。
  - 找出使相關值最大之最佳 $\tau$ 與最大相關值。
  - 回傳 `Some((best_lag, max_val))`。

## 4. 確定性黃金值測試要求
- 構造訊號：`x = [0.0, 1.0, 2.0, 3.0, 0.0]`, `y` 為延遲 1 格的 `[0.0, 0.0, 1.0, 2.0, 3.0]`
  - 當 `lag = 1` 時，$y[n+1]$ 與 $x[n]$ 對齊，相關值最大。
  - 輸出 `best_lag = 1`。

```golden
let x = [0.0, 1.0, 2.0, 3.0, 0.0];
let y = [0.0, 0.0, 1.0, 2.0, 3.0];
let (lag, corr) = dsp_find_peak_cross_correlation_lag(&x, &y, 2).unwrap();
lag == 1
corr > 10.0
dsp_find_peak_cross_correlation_lag(&x, &y, 0).is_none()
dsp_find_peak_cross_correlation_lag(&[1.0, 2.0], &y, 1).is_none()
```
