# 模組名稱: 人體體溫指數平滑與發燒分級檢測器 (Skin Temperature EMA & Fever Triage)

## 1. 目標定位
- 目標模組檔: `src/sensor_skin_temp_ema.rs`
- 目標測試檔: `tests/test_sensor_skin_temp_ema.rs`
- 功能簡述: 在智慧穿戴裝置與臨床連續體溫貼片中，對高頻熱敏電阻/紅外線溫度讀數進行 EMA 動態平滑濾波，並依醫學標準進行正常、低燒與高燒三級分診預警。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Wearable skin temperature EMA filter and medical fever triage
// Inputs: raw_celsius, prev_celsius, alpha, temp_celsius, normal_high, fever_high: f64
// Outputs: Option<f64> or Option<u8>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: raw_celsius in [10.0..=50.0], prev_celsius in [10.0..=50.0], alpha in [0.0..=1.0]
pub fn sensor_skin_temp_smooth(
    raw_celsius: f64,
    prev_celsius: f64,
    alpha: f64,
) -> Option<f64>;

/// @outputs: 1
/// @domain: temp in [25.0..=45.0], normal_high in [36.0..=38.0], fever_high in [38.0..=42.0]
pub fn sensor_fever_triage(
    temp_celsius: f64,
    normal_high: f64,
    fever_high: f64,
) -> Option<u8>;
```

## 3. 邊界與異常條件 (Domain Guards)
- `sensor_skin_temp_smooth`: 若 `raw_celsius < 10.0 || raw_celsius > 50.0 || prev_celsius < 10.0 || prev_celsius > 50.0 || alpha < 0.0 || alpha > 1.0`，立即回傳 `None`。
  $$\text{temp} = \text{prev\_celsius} + \alpha \times (\text{raw\_celsius} - \text{prev\_celsius})$$
- `sensor_fever_triage`: 若 `temp_celsius < 25.0 || temp_celsius > 45.0 || normal_high < 36.0 || normal_high > 38.0 || fever_high < 38.0 || fever_high > 42.0 || normal_high >= fever_high`，立即回傳 `None`。
  - 分級：`temp < normal_high` -> `Some(0)` (正常)
  - `temp < fever_high` -> `Some(1)` (低燒)
  - 否則 -> `Some(2)` (高燒)

## 4. 確定性黃金值測試要求
- 平滑更新：`raw = 37.0, prev = 36.0, alpha = 0.2` -> `36.2`
- 發燒分級：基準門檻 `normal_high = 37.3, fever_high = 38.5`
  - `temp = 36.8` -> `0`
  - `temp = 37.5` -> `1`
  - `temp = 39.2` -> `2`

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_sensor_skin_temp_ema.rs）
```golden
(sensor_skin_temp_smooth(37.0, 36.0, 0.2).unwrap() - 36.2).abs() < 1e-10
sensor_skin_temp_smooth(55.0, 36.0, 0.2).is_none()
sensor_skin_temp_smooth(37.0, 36.0, 1.2).is_none()
sensor_fever_triage(36.8, 37.3, 38.5) == Some(0)
sensor_fever_triage(37.5, 37.3, 38.5) == Some(1)
sensor_fever_triage(39.2, 37.3, 38.5) == Some(2)
sensor_fever_triage(20.0, 37.3, 38.5).is_none()
sensor_fever_triage(37.0, 39.0, 38.0).is_none()
```
