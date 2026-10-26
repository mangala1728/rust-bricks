//! # 空間資訊、地理幾何投影與衛星導航大地解算 (Geographic Information, Projection & Geodesy)
//!
//! 提供 WGS84 經緯高轉地心地固座標系 (ECEF)、本地東北天切平面 (ENU)、
//! Web Mercator (EPSG:3857) 正反投影、Slippy Map 瓦片網格坐標換算、
//! UTM 投影分帶、Vincenty 歸化緯度與恒向線 (Rhumb Line) 航向角純微積木。
//! 零外部依賴，符合 NASA JPL 規範與合約 09。

const WGS84_A: f64 = 6378137.0; // 長半軸 a (公尺)
const WGS84_F: f64 = 1.0 / 298.257223563; // 扁率 f
const WGS84_E_SQ: f64 = 2.0 * WGS84_F - WGS84_F * WGS84_F; // 第一偏心率平方 e^2
const PI: f64 = 3.141592653589793;

/// ## Map: WGS84 經緯度高度轉地心地固直角坐標 (WGS84 LLA to ECEF)
/// @outputs: 1
/// @domain: lat_deg [-90.0..=90.0] && lon_deg [-180.0..=180.0] && alt_m [-10_000.0..=100_000.0]
pub fn wgs84_lla_to_ecef(lat_deg: f64, lon_deg: f64, alt_m: f64) -> Option<[f64; 3]> {
    if !lat_deg.is_finite() || !lon_deg.is_finite() || !alt_m.is_finite() {
        return None;
    }
    if lat_deg < -90.0 || lat_deg > 90.0 || lon_deg < -180.0 || lon_deg > 180.0 || alt_m < -10_000.0 || alt_m > 100_000.0 {
        return None;
    }
    let phi = lat_deg * PI / 180.0;
    let lambda = lon_deg * PI / 180.0;
    let s_phi = phi.sin();
    let c_phi = phi.cos();
    let s_lam = lambda.sin();
    let c_lam = lambda.cos();

    let n = WGS84_A / (1.0 - WGS84_E_SQ * s_phi * s_phi).sqrt();
    let x = (n + alt_m) * c_phi * c_lam;
    let y = (n + alt_m) * c_phi * s_lam;
    let z = (n * (1.0 - WGS84_E_SQ) + alt_m) * s_phi;
    Some([x, y, z])
}

/// ## Calc: ECEF 位移向量轉站心切平面東北天坐標 (ECEF to Local ENU Step)
/// @outputs: 1
/// @domain: ref_lat_deg [-90.0..=90.0] && ref_lon_deg [-180.0..=180.0]
pub fn ecef_to_enu_step(
    dx: f64,
    dy: f64,
    dz: f64,
    ref_lat_deg: f64,
    ref_lon_deg: f64,
) -> Option<[f64; 3]> {
    if !dx.is_finite() || !dy.is_finite() || !dz.is_finite() || !ref_lat_deg.is_finite() || !ref_lon_deg.is_finite() {
        return None;
    }
    if ref_lat_deg < -90.0 || ref_lat_deg > 90.0 || ref_lon_deg < -180.0 || ref_lon_deg > 180.0 {
        return None;
    }
    let phi = ref_lat_deg * PI / 180.0;
    let lambda = ref_lon_deg * PI / 180.0;
    let s_phi = phi.sin();
    let c_phi = phi.cos();
    let s_lam = lambda.sin();
    let c_lam = lambda.cos();

    let east = -s_lam * dx + c_lam * dy;
    let north = -s_phi * c_lam * dx - s_phi * s_lam * dy + c_phi * dz;
    let up = c_phi * c_lam * dx + c_phi * s_lam * dy + s_phi * dz;
    Some([east, north, up])
}

/// ## Calc: WGS84 經緯度轉 Web 麥卡托投影坐標 (EPSG:3857 Forward Projection)
/// 回傳 [X, Y] (公尺)
/// @outputs: 1
/// @domain: lat_deg [-85.051129..=85.051129] && lon_deg [-180.0..=180.0]
pub fn web_mercator_forward_epsg3857(lat_deg: f64, lon_deg: f64) -> Option<[f64; 2]> {
    if !lat_deg.is_finite() || !lon_deg.is_finite() {
        return None;
    }
    if lat_deg < -85.051129 || lat_deg > 85.051129 || lon_deg < -180.0 || lon_deg > 180.0 {
        return None;
    }
    let x = WGS84_A * (lon_deg * PI / 180.0);
    let phi = lat_deg * PI / 180.0;
    let y = WGS84_A * (PI * 0.25 + phi * 0.5).tan().ln();
    Some([x, y])
}

/// ## Calc: Web 麥卡托投影坐標反轉 WGS84 經緯度 (EPSG:3857 Inverse Projection)
/// 回傳 [lat_deg, lon_deg]
/// @outputs: 1
/// @domain: x_m 與 y_m 位於半徑邊界內 [-20037508.34..=20037508.34]
pub fn web_mercator_inverse_epsg3857(x_m: f64, y_m: f64) -> Option<[f64; 2]> {
    let max_bound = WGS84_A * PI;
    if !x_m.is_finite() || !y_m.is_finite() {
        return None;
    }
    if x_m < -max_bound || x_m > max_bound || y_m < -max_bound || y_m > max_bound {
        return None;
    }
    let lon_deg = (x_m / WGS84_A) * 180.0 / PI;
    let lat_rad = 2.0 * (y_m / WGS84_A).exp().atan() - PI * 0.5;
    let lat_deg = lat_rad * 180.0 / PI;
    Some([lat_deg, lon_deg])
}

/// ## Calc: 經緯度轉 Slippy Map 瓦片座標 (Slippy Map Tile X/Y)
/// @outputs: 1
/// @domain: zoom <= 25 && lat_deg [-85.051129..=85.051129]
pub fn slippy_tile_xy_from_latlon(lat_deg: f64, lon_deg: f64, zoom: u8) -> Option<(u32, u32)> {
    if zoom > 25 || !lat_deg.is_finite() || !lon_deg.is_finite() {
        return None;
    }
    if lat_deg < -85.051129 || lat_deg > 85.051129 || lon_deg < -180.0 || lon_deg > 180.0 {
        return None;
    }
    let n = (1u32 << zoom) as f64;
    let x = ((lon_deg + 180.0) / 360.0 * n).floor() as u32;
    let lat_rad = lat_deg * PI / 180.0;
    let sec_lat = 1.0 / lat_rad.cos();
    let tan_lat = lat_rad.tan();
    let y = ((1.0 - (tan_lat + sec_lat).ln() / PI) * 0.5 * n).floor() as u32;
    Some((x, y))
}

/// ## Step: 經度計算 UTM 投影帶編號 (UTM Zone from Longitude)
/// @outputs: 1
/// @domain: lon_deg [-180.0..=180.0]
pub fn utm_zone_from_longitude(lon_deg: f64) -> Option<u8> {
    if !lon_deg.is_finite() || lon_deg < -180.0 || lon_deg > 180.0 {
        return None;
    }
    let zone = ((lon_deg + 180.0) / 6.0).floor() as i32 + 1;
    let clamped_zone = zone.clamp(1, 60) as u8;
    Some(clamped_zone)
}

/// ## Calc: UTM 投影帶中央經線解算 (UTM Central Meridian Longitude in Degrees)
/// @outputs: 1
/// @domain: zone [1..=60]
pub fn utm_central_meridian_lon(zone: u8) -> Option<f64> {
    if zone == 0 || zone > 60 {
        return None;
    }
    let lon = (zone as f64 - 1.0) * 6.0 - 180.0 + 3.0;
    Some(lon)
}

/// ## Calc: Vincenty 橢球歸化緯度轉換 (Reduced Latitude)
/// @outputs: 1
/// @domain: lat_rad [-PI/2..=PI/2]
pub fn vincenty_reduced_latitude_rad(lat_rad: f64) -> Option<f64> {
    if !lat_rad.is_finite() || lat_rad.abs() > (PI * 0.5 + 1e-6) {
        return None;
    }
    let tan_u = (1.0 - WGS84_F) * lat_rad.tan();
    Some(tan_u.atan())
}

/// ## Calc: 兩點間恒向線航向角 (Rhumb Line Bearing in Degrees)
/// @outputs: 1
/// @domain: lat [-85.0..=85.0] && lon [-180.0..=180.0]
pub fn rhumb_line_bearing_deg(
    lat1_deg: f64,
    lon1_deg: f64,
    lat2_deg: f64,
    lon2_deg: f64,
) -> Option<f64> {
    if !lat1_deg.is_finite() || !lon1_deg.is_finite() || !lat2_deg.is_finite() || !lon2_deg.is_finite() {
        return None;
    }
    let phi1 = lat1_deg * PI / 180.0;
    let phi2 = lat2_deg * PI / 180.0;
    let delta_phi = (PI * 0.25 + phi2 * 0.5).tan().ln() - (PI * 0.25 + phi1 * 0.5).tan().ln();
    let mut delta_lambda = (lon2_deg - lon1_deg) * PI / 180.0;
    if delta_lambda.abs() > PI {
        delta_lambda = if delta_lambda > 0.0 { delta_lambda - 2.0 * PI } else { delta_lambda + 2.0 * PI };
    }
    let bearing_rad = delta_lambda.atan2(delta_phi);
    let mut bearing_deg = bearing_rad * 180.0 / PI;
    if bearing_deg < 0.0 {
        bearing_deg += 360.0;
    }
    Some(bearing_deg)
}

/// ## Calc: 球面多邊形球面角超轉地表面積 (Spherical Polygon Excess Area)
/// @outputs: 1
/// @domain: excess_steradians >= 0.0 && excess_steradians <= 4.0 * PI
pub fn spherical_polygon_area_m2(excess_steradians: f64) -> Option<f64> {
    if !excess_steradians.is_finite() || excess_steradians < 0.0 || excess_steradians > (4.0 * PI + 1e-6) {
        return None;
    }
    let r = 6371008.8; // 等面積球體平均半徑
    Some(excess_steradians * r * r)
}
