# 模組名稱: 地心地固 ECEF 座標轉 WGS-84 大地經緯高演算法 (ECEF to WGS-84 Geodetic LLA Converter)

## 1. 目標定位
- 目標模組檔: `src/geo_ecef_to_lla.rs`
- 目標測試檔: `tests/test_geo_ecef_to_lla.rs`
- 功能簡述: GNSS 衛星導航、雷達觀測與大地測量之地心地固直角座標系 (ECEF: X, Y, Z 公尺) 至 WGS-84 大地座標 (緯度度數、經度度數、橢球高公尺) Bowring 封閉形式高精度解析解。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: ECEF cartesian to WGS-84 Geodetic Latitude, Longitude, Altitude (LLA)
// Inputs: x, y, z: f64 (meters)
// Outputs: Option<(f64, f64, f64)> (lat_deg, lon_deg, alt_m)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 3
/// @domain: x, y, z finite, x.abs() < 1e8, y.abs() < 1e8, z.abs() < 1e8, (x*x + y*y) > 1.0
pub fn geo_ecef_to_geodetic_lla(
    x: f64,
    y: f64,
    z: f64,
) -> Option<(f64, f64, f64)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若任何座標非有限數值或絕對值 $\ge 10^8$ 公尺，立即回傳 `None`。
- `let p_sq = x * x + y * y;`
  若 `p_sq < 1.0` (極端靠近地心極軸奇異點)，立即回傳 `None`。
- WGS-84 基準常數：
  - `a = 6378137.0f64;`
  - `f = 1.0 / 298.257223563;`
  - `b = a * (1.0 - f);`
  - `e_sq = 2.0 * f - f * f;`
  - `ep_sq = (a * a - b * b) / (b * b);`
- Bowring 封閉演算法：
  - `let p = p_sq.sqrt();`
  - `let theta = (z * a).atan2(p * b);`
  - `let sin_th = theta.sin(); let cos_th = theta.cos();`
  - `let phi = (z + ep_sq * b * sin_th.powi(3)).atan2(p - e_sq * a * cos_th.powi(3));`
  - `let lambda = y.atan2(x);`
  - `let sin_phi = phi.sin();`
  - `let n_rad = a / (1.0 - e_sq * sin_phi * sin_phi).sqrt();`
  - `let h = p / phi.cos() - n_rad;`
- 回傳 `Some((phi.to_degrees(), lambda.to_degrees(), h))`。

## 4. 確定性黃金值測試要求
- 赤道本初子午線上海平面高 0 公尺點：
  - `x = 6378137.0, y = 0.0, z = 0.0`
  - 大地座標精確為 `(0.0, 0.0, 0.0)`
- 北極附近海平面上高 100 公尺點：
  - `x = 100.0, y = 0.0, z = b + 100.0`
- 含 NaN 拒絕。

```golden
let (lat0, lon0, alt0) = geo_ecef_to_geodetic_lla(6378137.0, 0.0, 0.0).unwrap();
lat0.abs() < 1e-7
lon0.abs() < 1e-7
alt0.abs() < 1e-4
geo_ecef_to_geodetic_lla(0.0, 0.0, 0.0).is_none()
geo_ecef_to_geodetic_lla(f64::NAN, 0.0, 0.0).is_none()
```
