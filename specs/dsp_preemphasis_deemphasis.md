# 模組名稱: 語音訊號預加重與去加重濾波器 (Speech Pre-emphasis and De-emphasis)

## 1. 目標定位
- 目標模組檔: `src/dsp_preemphasis_deemphasis.rs`
- 目標測試檔: `tests/test_dsp_preemphasis_deemphasis.rs`
- 功能簡述: 語音訊號處理與電信編解碼標準預加重 ($y[n] = x[n] - \alpha x[n-1]$) 與去加重 ($x[n] = y[n] + \alpha x[n-1]$) 單步純函式，具備雙向精確重構不變量。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Speech signal pre-emphasis high-frequency boost and de-emphasis recovery
// Inputs: sample: f64, prev_sample: f64, alpha: f64
// Outputs: Option<f64>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: x in [-1e6..=1e6], prev_x in [-1e6..=1e6], alpha in [0.0..=1.0]
pub fn dsp_preemphasis_step(x: f64, prev_x: f64, alpha: f64) -> Option<f64>;

/// @outputs: 1
/// @domain: y in [-1e6..=1e6], prev_x in [-1e6..=1e6], alpha in [0.0..=1.0]
pub fn dsp_deemphasis_step(y: f64, prev_x: f64, alpha: f64) -> Option<f64>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若任何參數非有限數值，立即回傳 `None`。
- 若 `alpha < 0.0 || alpha > 1.0`，立即回傳 `None`。
- `dsp_preemphasis_step`: 計算 `x - alpha * prev_x`。
- `dsp_deemphasis_step`: 計算 `y + alpha * prev_x`。

## 4. 確定性黃金值測試要求
- `x = 10.0, prev_x = 8.0, alpha = 0.95`
  - 預加重：`10.0 - 0.95 * 8.0 = 10.0 - 7.6 = 2.4`
  - 去加重：`2.4 + 0.95 * 8.0 = 10.0` (完全無損重構)
- 越界 `alpha` 拒絕。

```golden
(dsp_preemphasis_step(10.0, 8.0, 0.95).unwrap() - 2.4).abs() < 1e-12
(dsp_deemphasis_step(2.4, 8.0, 0.95).unwrap() - 10.0).abs() < 1e-12
dsp_preemphasis_step(10.0, 8.0, 1.1).is_none()
dsp_deemphasis_step(10.0, 8.0, -0.1).is_none()
```
