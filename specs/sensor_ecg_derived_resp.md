# 模組名稱: 心電信號導出呼吸率估算器 (ECG-Derived Respiration)

## 1. 目標定位
- 目標模組檔: `src/sensor_ecg_derived_resp.rs`
- 目標測試檔: `tests/test_sensor_ecg_derived_resp.rs`
- 功能簡述: 在單導程心電監護儀中，利用胸腔呼吸調變引發之 R 峰振幅起伏與呼吸間隔時間序列，估算每分鐘呼吸次數（Breaths Per Minute, BPM）。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: ECG R-peak baseline amplitude extraction and respiration rate estimation
// Inputs: ecg_samples: &[f64], peak_idx: usize, baseline: f64, intervals: &[f64]
// Outputs: Option<f64>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: peak_idx < ecg_samples.len(), baseline is finite
pub fn sensor_edr_r_peak_amplitude(
    ecg_samples: &[f64],
    peak_idx: usize,
    baseline: f64,
) -> Option<f64>;

/// @outputs: 1
/// @domain: intervals.len() >= 2, all elements in [1.0..=10.0]
pub fn sensor_respiration_rate_from_intervals(
    intervals: &[f64],
) -> Option<f64>;
```

## 3. 邊界與異常條件 (Domain Guards)
- `sensor_edr_r_peak_amplitude`: 若 `peak_idx >= ecg_samples.len() || !baseline.is_finite() || !ecg_samples[peak_idx].is_finite()`，立即回傳 `None`。
  $$\text{amp} = \text{ecg\_samples}[\text{peak\_idx}] - \text{baseline}$$
  回傳 `Some(amp)`。
- `sensor_respiration_rate_from_intervals`: 若 `intervals.len() < 2` 或任何間隔超出 `[1.0, 10.0]`（對應正常生理極限 6~60 BPM），回傳 `None`。
  $$\text{mean\_t} = \frac{1}{N} \sum_{i=0}^{N-1} \text{intervals}[i]$$
  $$\text{BPM} = \frac{60.0}{\text{mean\_t}}$$
  回傳 `Some(BPM)`。

## 4. 確定性黃金值測試要求
- 振幅計算：`ecg = [0.1, 1.5, 0.2]`, `peak_idx = 1`, `baseline = 0.2` -> `1.5 - 0.2 = 1.3`
- 呼吸率：`intervals = [4.0, 4.0, 4.0]` -> 平均 4.0 秒 -> `60.0 / 4.0 = 15.0` BPM
- 呼吸率：`intervals = [3.0, 3.0]` -> 平均 3.0 秒 -> `60.0 / 3.0 = 20.0` BPM

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_sensor_ecg_derived_resp.rs）
```golden
(sensor_edr_r_peak_amplitude(&[0.1, 1.5, 0.2], 1, 0.2).unwrap() - 1.3).abs() < 1e-10
sensor_edr_r_peak_amplitude(&[0.1, 1.5], 2, 0.2).is_none()
(sensor_respiration_rate_from_intervals(&[4.0, 4.0, 4.0]).unwrap() - 15.0).abs() < 1e-10
(sensor_respiration_rate_from_intervals(&[3.0, 3.0]).unwrap() - 20.0).abs() < 1e-10
sensor_respiration_rate_from_intervals(&[4.0]).is_none()
sensor_respiration_rate_from_intervals(&[0.5, 4.0]).is_none()
sensor_respiration_rate_from_intervals(&[15.0, 4.0]).is_none()
```
