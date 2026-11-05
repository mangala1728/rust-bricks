# 模組名稱: 表面肌電圖 sEMG 線性包絡與肌肉激活度計算器 (Surface EMG Muscle Activation Percentage)

## 1. 目標定位
- 目標模組檔: `src/sensor_emg_muscle_activation.rs`
- 目標測試檔: `tests/test_sensor_emg_muscle_activation.rs`
- 功能簡述: 外骨骼機器人、假肢控制與復健穿戴設備之表面肌電圖 (sEMG) 訊號即時處理。全波整流、EMA 線性包絡濾波與最大自主收縮 (MVC) 肌肉活化百分比輸出。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Surface EMG linear envelope and %MVC muscle activation calculation
// Inputs: raw_emg, baseline, prev_env, alpha, mvc: f64
// Outputs: Option<(f64, f64)> (envelope, activation_pct)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 2
/// @domain: raw_emg in [-10.0..=10.0], baseline in [-2.0..=2.0], prev_env >= 0.0, alpha in [0.0..=1.0], mvc > 0.01
pub fn sensor_emg_activation_step(
    raw_emg: f64,
    baseline: f64,
    prev_env: f64,
    alpha: f64,
    mvc: f64,
) -> Option<(f64, f64)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若任何參數非有限數值，立即回傳 `None`。
- 若 `prev_env < 0.0 || alpha < 0.0 || alpha > 1.0 || mvc <= 0.01`，立即回傳 `None`。
- 計算：
  - 全波整流：`rectified = (raw_emg - baseline).abs()`
  - 線性包絡：`envelope = alpha * prev_env + (1.0 - alpha) * rectified`
  - 活化百分比：`pct = (envelope / mvc) * 100.0`
- 回傳 `Some((envelope, pct))`。

## 4. 確定性黃金值測試要求
- 確定性數值：
  - `raw = 1.5, baseline = 0.5 (整流後 1.0), prev_env = 0.0, alpha = 0.5, mvc = 2.0`
  - `envelope = 0.5 * 0.0 + 0.5 * 1.0 = 0.5`
  - `pct = (0.5 / 2.0) * 100.0 = 25.0%`
- 無效 MVC $\le 0.01$ 或越界 $\alpha$ 拒絕。

```golden
let (env, pct) = sensor_emg_activation_step(1.5, 0.5, 0.0, 0.5, 2.0).unwrap();
(env - 0.5).abs() < 1e-12
(pct - 25.0).abs() < 1e-12
sensor_emg_activation_step(1.5, 0.5, 0.0, 0.5, 0.0).is_none()
sensor_emg_activation_step(1.5, 0.5, -0.1, 0.5, 2.0).is_none()
sensor_emg_activation_step(1.5, 0.5, 0.0, 1.1, 2.0).is_none()
```
