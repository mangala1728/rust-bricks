# 模組名稱: 穿戴式人體突發跌倒撞擊檢測器 (Fall Detection Detector)

## 1. 目標定位
- 目標模組檔: `src/sensor_fall_detection.rs`
- 目標測試檔: `tests/test_sensor_fall_detection.rs`
- 功能簡述: 透過三軸加速度計之訊號向量模長（SVM），以失重（Free-fall）與劇烈撞擊（Impact Peak）雙階段狀態機辨識人體跌倒事故。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Accelerometer SVM freefall and impact fall detection state machine
// Inputs: ax: f64, ay: f64, az: f64, svm: f64, lower_threshold: f64, upper_threshold: f64, in_freefall: bool
// Outputs: Option<f64> or Option<(bool, bool)>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: ax, ay, az are finite
pub fn sensor_svm_magnitude(ax: f64, ay: f64, az: f64) -> Option<f64>;

/// @outputs: 2
/// @domain: svm >= 0.0, 0.0 <= lower_threshold < upper_threshold
pub fn sensor_fall_detect_impact(
    svm: f64,
    lower_threshold: f64,
    upper_threshold: f64,
    in_freefall: bool,
) -> Option<(bool, bool)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- `sensor_svm_magnitude`: 若任何一軸非有限數，回傳 `None`。回傳 `Some((ax*ax + ay*ay + az*az).sqrt())`。
- `sensor_fall_detect_impact`: 若 `!svm.is_finite() || svm < 0.0 || lower_threshold < 0.0 || lower_threshold >= upper_threshold || !upper_threshold.is_finite()`，立即回傳 `None`。
- 狀態轉移：
  - 若 `in_freefall` 為真且 `svm >= upper_threshold`：撞擊觸發！回傳 `Some((false, true))`。
  - 若 `svm <= lower_threshold`：進入失重狀態，回傳 `Some((true, false))`。
  - 否則若 `svm > lower_threshold && !in_freefall`：維持正常，回傳 `Some((false, false))`。
  - 若仍處於失重狀態但尚未撞擊（`svm < upper_threshold`）：回傳 `Some((in_freefall, false))`。

## 4. 確定性黃金值測試要求
- 向量 1 (重力 1g 基準): `ax = 0.0, ay = 0.0, az = 9.8` -> `svm = 9.8`
- 向量 2 (進入失重): `svm = 3.0, lower = 5.0, upper = 25.0, freefall = false` -> `(true, false)`
- 向量 3 (觸發撞擊): `svm = 28.0, lower = 5.0, upper = 25.0, freefall = true` -> `(false, true)`
- 向量 4 (正常姿態): `svm = 9.8, lower = 5.0, upper = 25.0, freefall = false` -> `(false, false)`

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_sensor_fall_detection.rs）
```golden
(sensor_svm_magnitude(0.0, 0.0, 9.8).unwrap() - 9.8).abs() < 1e-6
sensor_svm_magnitude(f64::NAN, 0.0, 9.8).is_none()
sensor_fall_detect_impact(3.0, 5.0, 25.0, false) == Some((true, false))
sensor_fall_detect_impact(28.0, 5.0, 25.0, true) == Some((false, true))
sensor_fall_detect_impact(9.8, 5.0, 25.0, false) == Some((false, false))
sensor_fall_detect_impact(-1.0, 5.0, 25.0, false).is_none()
sensor_fall_detect_impact(10.0, 25.0, 5.0, false).is_none()
```
