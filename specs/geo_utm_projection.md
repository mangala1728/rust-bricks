# 模組名稱: UTM 通用橫軸墨卡托投影分帶與中央經線判定器 (UTM Zone and Central Meridian Determinator)

## 1. 目標定位
- 目標模組檔: `src/geo_utm_projection.rs`
- 目標測試檔: `tests/test_geo_utm_projection.rs`
- 功能簡述: 地理資訊系統 (GIS) 與無人載具導航之 WGS-84 經緯度對應 UTM 投影分帶 (Zone 1..60)、半球標記 ('N'/'S') 與中央經線 (Central Meridian) 經度計算純函式。依 GEMINI 鐵律第 5 條使用歐幾里得除法。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: WGS-84 coordinates to UTM Zone, hemisphere, and central meridian calculator
// Inputs: lat_deg: f64, lon_deg: f64
// Outputs: Option<(u8, char, f64)> (zone, hemisphere, central_meridian_deg)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 3
/// @domain: lat_deg in [-80.0..=84.0], lon_deg in [-180.0..=180.0]
pub fn geo_determine_utm_zone(
    lat_deg: f64,
    lon_deg: f64,
) -> Option<(u8, char, f64)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若 `!lat_deg.is_finite() || !lon_deg.is_finite()`，立即回傳 `None`。
- 若 `lat_deg < -80.0 || lat_deg > 84.0 || lon_deg < -180.0 || lon_deg > 180.0`，立即回傳 `None` (UTM 投影南北緯標準涵蓋範圍)。
- 計算分帶：
  - 經度平移：若 `lon_deg == 180.0` 則視為 `179.9999`。
  - 分帶計算：`let zone = (((lon_deg + 180.0) / 6.0).floor() as i32) + 1;`
  - 若 `zone < 1 || zone > 60` 則 `None`，否則 `let zone_u8 = zone as u8;`。
  - 中央經線：`central_meridian = ((zone as f64) - 1.0) * 6.0 - 180.0 + 3.0;`
  - 半球字元：`let hemisphere = if lat_deg >= 0.0 { 'N' } else { 'S' };`
- 回傳 `Some((zone_u8, hemisphere, central_meridian))`。

## 4. 確定性黃金值測試要求
- 台灣本島 (東經 121.5, 北緯 25.0)：
  - `(121.5 + 180) / 6 = 301.5 / 6 = 50.25` -> Zone 51
  - 中央經線：`50 * 6 - 180 + 3 = 123.0`
  - 輸出 `(51, 'N', 123.0)`
- 英國格林威治 (經度 0.0, 北緯 51.5)：
  - `(0.0 + 180) / 6 = 30` -> Zone 31
  - 中央經線：`30 * 6 - 180 + 3 = 3.0`
  - 輸出 `(31, 'N', 3.0)`
- 極區越界 (緯度 85.0 或 -85.0) 拒絕。

```golden
let (z1, h1, cm1) = geo_determine_utm_zone(25.0, 121.5).unwrap();
z1 == 51
h1 == 'N'
(cm1 - 123.0).abs() < 1e-12
let (z2, h2, cm2) = geo_determine_utm_zone(51.5, 0.0).unwrap();
z2 == 31
h2 == 'N'
(cm2 - 3.0).abs() < 1e-12
geo_determine_utm_zone(85.0, 0.0).is_none()
geo_determine_utm_zone(-81.0, 0.0).is_none()
geo_determine_utm_zone(0.0, 185.0).is_none()
```
