# 模組名稱: 協調轉彎轉彎率與轉向半徑計算器 (Coordinated Turn Rate and Radius Calculator)

## 1. 目標定位
- 目標模組檔: `src/nav_turn_rate_radius.rs`
- 目標測試檔: `tests/test_nav_turn_rate_radius.rs`
- 功能簡述: 航電飛行力學、車載自動駕駛與無人機導引律之協調轉彎 (Coordinated Turn) 物理純函式。依據地速 (m/s) 與滾轉/傾角 (Bank Angle 度數)，計算轉彎角速度 (度/秒) 與轉向圓弧曲率半徑 (公尺)。符合 GEMINI 鐵律第 1 與第 10 條。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Aircraft and vehicle coordinated turn rate and turn radius calculator
// Inputs: speed_mps: f64, bank_angle_deg: f64
// Outputs: Option<(f64, f64)> (turn_rate_deg_per_sec, turn_radius_meters)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 2
/// @domain: speed_mps in [1.0..=1000.0], bank_angle_deg in [-80.0..=80.0] with abs >= 0.1
pub fn nav_coordinated_turn_metrics(
    speed_mps: f64,
    bank_angle_deg: f64,
) -> Option<(f64, f64)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若任何參數非有限浮點數，立即回傳 `None`。
- 若 `speed_mps < 1.0 || speed_mps > 1000.0`，立即回傳 `None`。
- 若 `bank_angle_deg.abs() < 0.1 || bank_angle_deg.abs() > 80.0`，立即回傳 `None` (避免直行時半徑發散或傾角過大失速)。
- 物理方程式：
  - 標準重力加速度：`let g = 9.80665f64;`
  - 滾轉弧度：`let phi = bank_angle_deg.to_radians();`
  - `let tan_phi = phi.tan();`
  - 轉彎半徑 (公尺)：`let radius = speed_mps * speed_mps / (g * tan_phi.abs());`
  - 轉向角速度 (rad/s)：`let rate_rad = g * tan_phi / speed_mps;`
  - 轉彎率 (度/秒)：`let rate_deg = rate_rad.to_degrees();`
- 回傳 `Some((rate_deg, radius))`。

## 4. 確定性黃金值測試要求
- 航空標準轉彎 (Standard Rate Turn, 3度/秒)：
  - 地速 $v = 50.0$ m/s, $\tan\phi = (3^\circ \times \pi / 180) \times 50 / 9.80665 \approx 0.26694$ ($\phi \approx 14.94^\circ$)
  - `speed = 50.0, bank = 15.0`
  - `tan(15 deg) = 0.26794919`
  - `rate = 9.80665 * 0.26794919 / 50.0 = 0.0525547 rad/s = 3.011度/秒`
  - `radius = 2500 / (9.80665 * 0.26794919) = 951.4公尺`
- 傾角 0 或超速拒絕。

```golden
let (rate, radius) = nav_coordinated_turn_metrics(50.0, 15.0).unwrap();
(rate - 3.0112).abs() < 1e-2
(radius - 951.4).abs() < 1.0
nav_coordinated_turn_metrics(50.0, 0.0).is_none()
nav_coordinated_turn_metrics(0.5, 15.0).is_none()
nav_coordinated_turn_metrics(50.0, 85.0).is_none()
```
