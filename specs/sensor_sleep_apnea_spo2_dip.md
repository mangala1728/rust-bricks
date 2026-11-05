# 模組名稱: 睡眠呼吸中止 ODI 4% 血氧驟降事件檢測器 (Sleep Apnea SpO2 Desaturation Dip Detector)

## 1. 目標定位
- 目標模組檔: `src/sensor_sleep_apnea_spo2_dip.rs`
- 目標測試檔: `tests/test_sensor_sleep_apnea_spo2_dip.rs`
- 功能簡晨: 睡眠監測與呼吸健康穿戴裝置之血氧飽和度 (SpO2) 驟降事件檢測演算法。依醫學標準判定血氧飽和度驟降指數 (Oxygen Desaturation Index, ODI 3%/4% 跌幅判定與恢復結算)。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Sleep apnea SpO2 oxygen desaturation event (ODI dip) detection
// Inputs: current_spo2, baseline_spo2, dip_threshold: f64, in_dip_event: bool
// Outputs: Option<(bool, f64)> (next_in_dip_event, dip_depth)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 2
/// @domain: current_spo2 in [50.0..=100.0], baseline_spo2 in [70.0..=100.0], dip_threshold in [2.0..=10.0]
pub fn sensor_detect_spo2_desaturation_dip(
    current_spo2: f64,
    baseline_spo2: f64,
    dip_threshold: f64,
    in_dip_event: bool,
) -> Option<(bool, f64)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若任何參數非有限浮點數，立即回傳 `None`。
- 若 `current_spo2 < 50.0 || current_spo2 > 100.0 || baseline_spo2 < 70.0 || baseline_spo2 > 100.0`，立即回傳 `None`。
- 若 `dip_threshold < 2.0 || dip_threshold > 10.0`，立即回傳 `None`。
- 計算：
  - 跌幅深度：`let dip_depth = (baseline_spo2 - current_spo2).max(0.0);`
  - 狀態轉移：
    - 若 `!in_dip_event && dip_depth >= dip_threshold` $\rightarrow$ 觸發事件 `next_state = true`
    - 若 `in_dip_event && dip_depth <= 1.0` (血氧已恢復至基準值 1% 以內) $\rightarrow$ 事件結束 `next_state = false`
    - 否則 $\rightarrow$ 保持原狀態 `next_state = in_dip_event`
- 回傳 `Some((next_state, dip_depth))`。

## 4. 確定性黃金值測試要求
- 跌幅觸發：`current = 92.0, baseline = 97.0 (跌5%), threshold = 4.0, in_dip = false`
  - `dip_depth = 5.0 >= 4.0` -> `next_state = true, dip = 5.0`
- 跌幅恢復：`current = 96.5, baseline = 97.0 (跌0.5%), threshold = 4.0, in_dip = true`
  - `dip_depth = 0.5 <= 1.0` -> `next_state = false, dip = 0.5`
- 越界 SpO2 拒絕。

```golden
let (s1, d1) = sensor_detect_spo2_desaturation_dip(92.0, 97.0, 4.0, false).unwrap();
s1 == true
(d1 - 5.0).abs() < 1e-12
let (s2, d2) = sensor_detect_spo2_desaturation_dip(96.5, 97.0, 4.0, true).unwrap();
s2 == false
(d2 - 0.5).abs() < 1e-12
sensor_detect_spo2_desaturation_dip(45.0, 97.0, 4.0, false).is_none()
sensor_detect_spo2_desaturation_dip(95.0, 97.0, 1.0, false).is_none()
```
