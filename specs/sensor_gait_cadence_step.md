# 模組名稱: 步態分析步頻與左右對稱性評估器 (Gait Analysis Cadence and Symmetry Ratio)

## 1. 目標定位
- 目標模組檔: `src/sensor_gait_cadence_step.rs`
- 目標測試檔: `tests/test_sensor_gait_cadence_step.rs`
- 功能簡述: 智慧手環與足底壓力感測器步態分析演算法。從步態著地時間戳記序列中計算平均步頻 (Cadence, steps/min) 與左右腳跨步時間對稱性比率 (Symmetry Ratio)。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Gait cadence and left-right step symmetry evaluation
// Inputs: step_timestamps_sec: &[f64]
// Outputs: Option<(f64, f64)> (cadence_spm, symmetry_ratio)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 2
/// @domain: step_timestamps_sec.len() in [4..=64], strictly increasing, time diffs in [0.2..=2.0]
pub fn sensor_gait_cadence_and_symmetry(
    step_timestamps_sec: &[f64],
) -> Option<(f64, f64)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若 `step_timestamps_sec.len() < 4 || step_timestamps_sec.len() > 64`，立即回傳 `None`。
- 檢查遞增與步間隔合理性：
  - 每個步間隔 $dt_i = t_i - t_{i-1}$ 必須滿足 $0.2 \le dt_i \le 2.0$。若有任何違反，立即回傳 `None`。
- 計算步頻：
  - 總時間差：`total_t = step_timestamps_sec.last() - step_timestamps_sec[0]`
  - 總步數跨度：`n_steps = step_timestamps_sec.len() - 1`
  - `cadence = 60.0 * (n_steps as f64) / total_t`
- 計算左右步態對稱比率：
  - 假設奇偶步交替（左/右）：
  - 奇數步間隔平均值 $T_{odd}$，偶數步間隔平均值 $T_{even}$。
  - 對稱比率：`sym = (T_odd / T_even).min(T_even / T_odd)`（完美對稱時為 1.0）。
- 回傳 `Some((cadence, sym))`。

## 4. 確定性黃金值測試要求
- 完美均勻步態：每 0.5 秒一步（每秒 2 步，步頻 120 SPM，對稱性 1.0）
  - `timestamps = [0.0, 0.5, 1.0, 1.5, 2.0]`
  - `cadence = 60 * 4 / 2.0 = 120.0`
  - `sym = 1.0`
- 非遞增時間戳或序列過短拒絕。

```golden
let ts = [0.0, 0.5, 1.0, 1.5, 2.0];
let (cadence, sym) = sensor_gait_cadence_and_symmetry(&ts).unwrap();
(cadence - 120.0).abs() < 1e-12
(sym - 1.0).abs() < 1e-12
sensor_gait_cadence_and_symmetry(&[0.0, 0.5, 1.0]).is_none()
sensor_gait_cadence_and_symmetry(&[0.0, 0.5, 0.4, 1.0]).is_none()
```
