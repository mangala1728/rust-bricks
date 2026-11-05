# 模組名稱: WGS-84 橢球體 Vincenty 逆算大圓距離與初方位角 (Vincenty Inverse Geodesy)

## 1. 目標定位
- 目標模組檔: `src/geo_vincenty_inverse.rs`
- 目標測試檔: `tests/test_geo_vincenty_inverse.rs`
- 功能簡述: 在 WGS-84 參考橢球體上（a = 6378137.0m, f = 1/298.257223563），透過 Vincenty 逆算迭代法精確求解兩經緯度點間之測地線長度（公尺）與初始前進方位角（0..360度）。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Vincenty inverse geodesic calculation on WGS-84 ellipsoid
// Inputs: lat1, lon1, lat2, lon2: f64 in degrees
// Outputs: Option<(f64, f64)> (distance_m, initial_bearing_deg)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 2
/// @domain: lat1, lat2 in [-90.0..=90.0], lon1, lon2 in [-180.0..=180.0]
pub fn geo_vincenty_inverse(
    lat1: f64,
    lon1: f64,
    lat2: f64,
    lon2: f64,
) -> Option<(f64, f64)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若任何經緯度非有限數、緯度超出 `[-90.0, 90.0]`、或經度超出 `[-180.0, 180.0]`，立即回傳 `None`。
- 若兩點重合（經緯度差之絕對值均 `< 1e-12`），直接回傳 `Some((0.0, 0.0))`。
- 若對拓點（Antipodal points）導致迭代不收斂（超過 100 次），回傳 `None`。

## 4. 確定性黃金值測試要求
- 赤道經度 1 度弧長（0°N 0°E 至 0°N 1°E）：
  WGS-84 理論測地距離約 $111319.49079$ 公尺，方位角正東 $90.0^\circ$。
- 重合點：`(0.0, 0.0)` 至 `(0.0, 0.0)` 回傳 `(0.0, 0.0)`。

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_geo_vincenty_inverse.rs）
```golden
let (dist, bear) = geo_vincenty_inverse(0.0, 0.0, 0.0, 1.0).unwrap();
(dist - 111319.49).abs() < 1.0
(bear - 90.0).abs() < 1e-3
let (dist0, _) = geo_vincenty_inverse(25.0, 121.0, 25.0, 121.0).unwrap();
dist0.abs() < 1e-6
geo_vincenty_inverse(95.0, 0.0, 0.0, 0.0).is_none()
geo_vincenty_inverse(f64::NAN, 0.0, 0.0, 0.0).is_none()
```
