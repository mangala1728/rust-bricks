# 模組名稱: IIR 梳狀濾波器步進計算純函式 (IIR Comb Filter Step)

## 1. 目標定位
- 目標模組檔: `src/dsp_iir_comb_filter.rs`
- 目標測試檔: `tests/test_dsp_iir_comb_filter.rs`
- 功能簡述: 聲學迴音與混響 (Reverberation) 之 IIR 反饋與前饋梳狀濾波單步純函式，提供具穩定性守衛之即時音訊 DSP 積木。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Feedback and feedforward comb filter single step computation
// Inputs: x: f64, delayed: f64, gain: f64
// Outputs: Option<f64>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: x in [-1e6..=1e6], delayed in [-1e6..=1e6], gain in [-0.999..=0.999]
pub fn dsp_iir_comb_feedback_step(x: f64, delayed_y: f64, gain: f64) -> Option<f64>;

/// @outputs: 1
/// @domain: x in [-1e6..=1e6], delayed in [-1e6..=1e6], gain in [-1.0..=1.0]
pub fn dsp_comb_feedforward_step(x: f64, delayed_x: f64, gain: f64) -> Option<f64>;
```

## 3. 邊界與異常條件 (Domain Guards)
- `dsp_iir_comb_feedback_step`:
  - 若 `!x.is_finite() || !delayed_y.is_finite() || !gain.is_finite()` $\rightarrow$ `None`
  - 若 `gain < -0.999 || gain > 0.999` $\rightarrow$ `None` (確保反饋 IIR 極點在單位圓內，避免發散)
  - 計算：`y = x + gain * delayed_y`
  - 回傳 `Some(y)`
- `dsp_comb_feedforward_step`:
  - 若 `!x.is_finite() || !delayed_x.is_finite() || !gain.is_finite()` $\rightarrow$ `None`
  - 若 `gain < -1.0 || gain > 1.0` $\rightarrow$ `None`
  - 計算：`y = x + gain * delayed_x`
  - 回傳 `Some(y)`

## 4. 確定性黃金值測試要求
- 確定性數值：
  - `x = 1.0, delayed_y = 0.5, gain = 0.8` $\rightarrow$ `1.0 + 0.4 = 1.4`
  - `x = 0.5, delayed_x = 0.5, gain = -1.0` $\rightarrow$ `0.5 - 0.5 = 0.0` (完全相消)
- 增益 $\ge 1.0$ 反饋拒絕。

```golden
(dsp_iir_comb_feedback_step(1.0, 0.5, 0.8).unwrap() - 1.4).abs() < 1e-12
(dsp_comb_feedforward_step(0.5, 0.5, -1.0).unwrap() - 0.0).abs() < 1e-12
dsp_iir_comb_feedback_step(1.0, 0.5, 1.0).is_none()
dsp_iir_comb_feedback_step(1.0, 0.5, -1.0).is_none()
dsp_iir_comb_feedback_step(f64::NAN, 0.5, 0.5).is_none()
dsp_comb_feedforward_step(1.0, 0.5, 1.05).is_none()
```
