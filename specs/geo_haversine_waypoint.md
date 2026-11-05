# 模組名稱: Haversine 大圓球面距離與初始方位角計算器 (Haversine Great-Circle Distance and Initial Bearing)

## 1. 目標定位
- 目標模組檔: `src/geo_haversine_waypoint.rs`
- 目標測試檔: `tests/test_geo_haversine_waypoint.rs`
- 功能簡述: 航空、航海與無人機航路點導航之標準 Haversine 球面大圓距離 (公尺) 與起始航向方位角 (Initial Bearing, 0..360 度) 計算純函式。符合 GEMINI 鐵律第 1 與第 10 條。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Haversine spherical distance and initial forward azimuth bearing
// Inputs: lat1_deg, lon1_deg, lat2_deg, lon2_deg: f64
// Outputs: Option<(f64, f64)> (distance_meters, initial_bearing_degrees)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 2
/// @domain: lat in [-90.0..=90.0], lon in [-180.0..=180.0]
pub fn geo_haversine_distance_bearing(
    lat1_deg: f64,
    lon1_deg: f64,
    lat2_deg: f64,
    lon2_deg: f64,
) -> Option<(f64, f64)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若任何參數非有限浮點數，立即回傳 `None`。
- 若 `lat1_deg.abs() > 90.0 || lat2_deg.abs() > 90.0 || lon1_deg.abs() > 180.0 || lon2_deg.abs() > 180.0`，立即回傳 `None`。
- 角度轉弧度：
  - `let r_earth = 6371000.0f64;`
  - `let phi1 = lat1_deg.to_radians(); let phi2 = lat2_deg.to_radians();`
  - `let d_phi = (lat2_deg - lat1_deg).to_radians();`
  - `let d_lambda = (lon2_deg - lon1_deg).to_radians();`
- 距離計算：
  - `let a = (d_phi / 2.0).sin().powi(2) + phi1.cos() * phi2.cos() * (d_lambda / 2.0).sin().powi(2);`
  - `let c = 2.0 * a.sqrt().atan2((1.0 - a).max(0.0).sqrt());`
  - `let distance = r_earth * c;`
- 初始方位角計算：
  - `let y = d_lambda.sin() * phi2.cos();`
  - `let x = phi1.cos() * phi2.sin() - phi1.sin() * phi2.cos() * d_lambda.cos();`
  - `let mut bearing = y.atan2(x).to_degrees();`
  - `bearing = (bearing % 360.0 + 360.0) % 360.0;`
- 回傳 `Some((distance, bearing))`。

## 4. 確定性黃金值測試要求
- 同一點測試：`lat1=0, lon1=0, lat2=0, lon2=0` $\rightarrow$ 距離精確為 `0.0`。
- 赤道上經度相差 1 度：
  - 距離應為 $2\pi R / 360 \approx 111194.9$ 公尺。
  - 方位角向東應為 $90.0$ 度。

```golden
let (d0, _) = geo_haversine_distance_bearing(0.0, 0.0, 0.0, 0.0).unwrap();
d0.abs() < 1e-6
let (d1, b1) = geo_haversine_distance_bearing(0.0, 0.0, 0.0, 1.0).unwrap();
(d1 - 111194.9266).abs() < 1.0
(b1 - 90.0).abs() < 1e-6
geo_haversine_distance_bearing(95.0, 0.0, 0.0, 0.0).is_none()
geo_haversine_distance_bearing(0.0, -190.0, 0.0, 0.0).is_none()
```
