# 模組名稱: 太陽天頂角與高度角快速推算器 (Solar Zenith and Elevation Angle Calculator)

## 1. 目標定位
- 目標模組檔: `src/astro_solar_zenith_angle.rs`
- 目標測試檔: `tests/test_astro_solar_zenith_angle.rs`
- 功能簡述: 太陽能發電預測、農業光照與智慧戶外感測器之太陽天頂角 (Zenith Angle) 與仰角 (Elevation Angle) 天文幾何推算純函式。依據一年中第幾天 (Day of Year) 與世界協調時 (UTC) 快速解算。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Solar zenith and elevation angle computation from day-of-year and UTC time
// Inputs: day_of_year: u16, utc_hour: f64, lat_deg: f64, lon_deg: f64
// Outputs: Option<(f64, f64)> (zenith_deg, elevation_deg)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 2
/// @domain: day_of_year in [1..=366], utc_hour in [0.0..=24.0], lat in [-90.0..=90.0], lon in [-180.0..=180.0]
pub fn astro_solar_zenith_and_elevation(
    day_of_year: u16,
    utc_hour: f64,
    lat_deg: f64,
    lon_deg: f64,
) -> Option<(f64, f64)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若 `day_of_year < 1 || day_of_year > 366`，立即回傳 `None`。
- 若 `!utc_hour.is_finite() || utc_hour < 0.0 || utc_hour > 24.0`，立即回傳 `None`。
- 若 `lat_deg.abs() > 90.0 || lon_deg.abs() > 180.0`，立即回傳 `None`。
- 運算幾何：
  - 赤緯角 (Cooper 方程)：
    - `let b = 2.0 * std::f64::consts::PI * ((284 + day_of_year as i32) as f64) / 365.0;`
    - `let declination_rad = (23.45f64).to_radians() * b.sin();`
  - 地方太陽時與時角：
    - `let local_solar_time = utc_hour + lon_deg / 15.0;`
    - `let hour_angle_rad = ((local_solar_time - 12.0) * 15.0).to_radians();`
  - 天頂角餘弦：
    - `let lat_rad = lat_deg.to_radians();`
    - `let cos_zenith = lat_rad.sin() * declination_rad.sin() + lat_rad.cos() * declination_rad.cos() * hour_angle_rad.cos();`
    - `let cos_z_clamped = cos_zenith.clamp(-1.0, 1.0);`
    - `let zenith_deg = cos_z_clamped.acos().to_degrees();`
    - `let elevation_deg = 90.0 - zenith_deg;`
- 回傳 `Some((zenith_deg, elevation_deg))`。

## 4. 確定性黃金值測試要求
- 春分正午赤道檢驗：
  - 春分約第 81 天，正午 12:00 在經度 0、緯度 0：
  - 太陽幾近正頭頂直射：`zenith_deg < 5.0`，`elevation_deg > 85.0`。
- 仰角 + 天頂角恆等式：`zenith_deg + elevation_deg == 90.0`。
- 越界時間或日期拒絕。

```golden
let (z, el) = astro_solar_zenith_and_elevation(81, 12.0, 0.0, 0.0).unwrap();
(z + el - 90.0).abs() < 1e-10
z < 5.0
el > 85.0
astro_solar_zenith_and_elevation(0, 12.0, 0.0, 0.0).is_none()
astro_solar_zenith_and_elevation(81, 25.0, 0.0, 0.0).is_none()
astro_solar_zenith_and_elevation(81, 12.0, 95.0, 0.0).is_none()
```
