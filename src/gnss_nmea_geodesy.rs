//! # GNSS NMEA & Geodesy Micro-Bricks (衛星導航 NMEA 與大地測量微積木)
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 NMEA 0183 異或校驗和、經緯度度分轉十進制度數、WGS84 經緯高 (LLH) 與地心地固直角坐標 (ECEF) 雙向變換、
//! ECEF 轉站心東-北-天 (ENU) 局域坐標、衛星仰角/方位角解算、幾何精度因子 (DOP)、
//! 大圓大地線前向方位角、偽距衛星鐘差校正、載波相位周跳檢測。

const WGS84_A: f64 = 6378137.0; // 長半軸 a (meters)
const WGS84_F: f64 = 1.0 / 298.257223563; // 扁率 f
const WGS84_B: f64 = WGS84_A * (1.0 - WGS84_F); // 短半軸 b
const WGS84_E2: f64 = 2.0 * WGS84_F - WGS84_F * WGS84_F; // 第一偏心率平方 e^2
const WGS84_EP2: f64 = (WGS84_A * WGS84_A - WGS84_B * WGS84_B) / (WGS84_B * WGS84_B); // 第二偏心率平方 e'^2
const SPEED_OF_LIGHT: f64 = 299792458.0; // 光速 c (m/s)
const PI: f64 = std::f64::consts::PI;

/// ## Parse: NMEA 0183 語句 XOR 異或校驗和計算
/// 計算 '$' 或 '!' 與 '*' 之間所有字元之逐位元 XOR
/// @outputs: 1
pub fn gnss_nmea_checksum(sentence: &[u8]) -> Option<u8> {
    if sentence.len() < 4 {
        return None;
    }
    let mut start_idx = None;
    let mut end_idx = None;
    for (i, &b) in sentence.iter().enumerate() {
        if (b == b'$' || b == b'!') && start_idx.is_none() {
            start_idx = Some(i + 1);
        } else if b == b'*' && start_idx.is_some() {
            end_idx = Some(i);
            break;
        }
    }
    let start = start_idx?;
    let end = end_idx?;
    if start > end { return None; }
    let mut checksum = 0u8;
    for &b in &sentence[start..end] {
        checksum ^= b;
    }
    Some(checksum)
}

/// ## Convert: NMEA 度分格式轉十進制度數 (DDMM.MMMM -> Decimal Degrees)
/// @outputs: 1
/// @domain: nmea_coord [0.0, 18000.0]
pub fn gnss_nmea_lat_lon_to_decimal_deg(nmea_coord: f64, direction: u8) -> Option<f64> {
    if !nmea_coord.is_finite() || nmea_coord < 0.0 || nmea_coord > 18000.0 {
        return None;
    }
    let degrees = (nmea_coord / 100.0).floor();
    let minutes = nmea_coord - (degrees * 100.0);
    if minutes >= 60.0 {
        return None;
    }
    let dec_deg = degrees + minutes / 60.0;
    match direction {
        b'N' | b'n' => if dec_deg <= 90.0 { Some(dec_deg) } else { None },
        b'S' | b's' => if dec_deg <= 90.0 { Some(-dec_deg) } else { None },
        b'E' | b'e' => if dec_deg <= 180.0 { Some(dec_deg) } else { None },
        b'W' | b'w' => if dec_deg <= 180.0 { Some(-dec_deg) } else { None },
        _ => None,
    }
}

/// ## Map: WGS84 大地坐標 (緯度, 經度, 高程) 轉 ECEF 地心地固直角坐標 (X, Y, Z)
/// @outputs: 1
/// @domain: lat_deg [-90.0, 90.0]
/// @domain: lon_deg [-180.0, 180.0]
/// @domain: alt_m [-10000.0, 1000000.0]
pub fn gnss_wgs84_llh_to_ecef(lat_deg: f64, lon_deg: f64, alt_m: f64) -> Option<[f64; 3]> {
    if !lat_deg.is_finite() || !lon_deg.is_finite() || !alt_m.is_finite() ||
       lat_deg < -90.0 || lat_deg > 90.0 || lon_deg < -180.0 || lon_deg > 180.0 {
        return None;
    }
    let phi = lat_deg * PI / 180.0;
    let lambda = lon_deg * PI / 180.0;
    let sin_phi = phi.sin();
    let cos_phi = phi.cos();
    let sin_lambda = lambda.sin();
    let cos_lambda = lambda.cos();

    let n = WGS84_A / (1.0 - WGS84_E2 * sin_phi * sin_phi).sqrt();
    let x = (n + alt_m) * cos_phi * cos_lambda;
    let y = (n + alt_m) * cos_phi * sin_lambda;
    let z = (n * (1.0 - WGS84_E2) + alt_m) * sin_phi;
    Some([x, y, z])
}

/// ## Solve: ECEF 地心地固直角坐標逆轉 WGS84 大地坐標 (Bowring 閉式解法)
/// @outputs: 1
/// @domain: x [-1e8, 1e8]
/// @domain: y [-1e8, 1e8]
/// @domain: z [-1e8, 1e8]
pub fn gnss_ecef_to_wgs84_llh(x: f64, y: f64, z: f64) -> Option<[f64; 3]> {
    if !x.is_finite() || !y.is_finite() || !z.is_finite() {
        return None;
    }
    let p = (x * x + y * y).sqrt();
    if p < 1e-4 {
        // 近南北極奇異點
        let lat = if z >= 0.0 { 90.0 } else { -90.0 };
        let alt = z.abs() - WGS84_B;
        return Some([lat, 0.0, alt]);
    }
    let theta = (z * WGS84_A).atan2(p * WGS84_B);
    let sin_theta = theta.sin();
    let cos_theta = theta.cos();

    let num = z + WGS84_EP2 * WGS84_B * sin_theta * sin_theta * sin_theta;
    let den = p - WGS84_E2 * WGS84_A * cos_theta * cos_theta * cos_theta;
    let phi = num.atan2(den);
    let lambda = y.atan2(x);

    let sin_phi = phi.sin();
    let n = WGS84_A / (1.0 - WGS84_E2 * sin_phi * sin_phi).sqrt();
    let alt = p / phi.cos() - n;

    Some([phi * 180.0 / PI, lambda * 180.0 / PI, alt])
}

/// ## Map: ECEF 相對基準向量轉站心東-北-天 (ENU) 局域坐標系
/// @outputs: 1
/// @domain: dx [-1e8, 1e8]
/// @domain: dy [-1e8, 1e8]
/// @domain: dz [-1e8, 1e8]
/// @domain: ref_lat_deg [-90.0, 90.0]
/// @domain: ref_lon_deg [-180.0, 180.0]
pub fn gnss_ecef_to_enu_step(dx: f64, dy: f64, dz: f64, ref_lat_deg: f64, ref_lon_deg: f64) -> [f64; 3] {
    if !dx.is_finite() || !dy.is_finite() || !dz.is_finite() ||
       !ref_lat_deg.is_finite() || !ref_lon_deg.is_finite() {
        return [0.0, 0.0, 0.0];
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
    [east, north, up]
}

/// ## Solve: 依站心 ENU 坐標向量解算衛星仰角與方位角 (Elevation & Azimuth in Degrees)
/// @outputs: 1
pub fn gnss_satellite_elevation_azimuth_deg(enu: [f64; 3]) -> Option<(f64, f64)> {
    let [e, n, u] = enu;
    if !e.is_finite() || !n.is_finite() || !u.is_finite() {
        return None;
    }
    let d_horiz = (e * e + n * n).sqrt();
    let dist = (e * e + n * n + u * u).sqrt();
    if dist < 1e-3 {
        return None;
    }
    let elev_rad = u.atan2(d_horiz);
    let azim_rad = e.atan2(n);
    let mut azim_deg = azim_rad * 180.0 / PI;
    if azim_deg < 0.0 {
        azim_deg += 360.0;
    }
    Some((elev_rad * 180.0 / PI, azim_deg))
}

/// ## Calc: 幾何精度衰減因子解算 (PDOP, HDOP, VDOP)
/// 輸入站心坐標協方差對角項 [q_ee, q_nn, q_uu, q_tt]
/// @outputs: 1
pub fn gnss_dilution_of_precision_dop(q_diag: [f64; 4]) -> Option<(f64, f64, f64)> {
    for &val in &q_diag {
        if !val.is_finite() || val < 0.0 {
            return None;
        }
    }
    let hdop = (q_diag[0] + q_diag[1]).sqrt();
    let vdop = q_diag[2].sqrt();
    let pdop = (q_diag[0] + q_diag[1] + q_diag[2]).sqrt();
    Some((pdop, hdop, vdop))
}

/// ## Calc: 兩點間大圓大地線前向方位角 (Initial Great-Circle Bearing in Degrees)
/// @outputs: 1
/// @domain: lat1_deg [-90.0, 90.0]
/// @domain: lon1_deg [-180.0, 180.0]
/// @domain: lat2_deg [-90.0, 90.0]
/// @domain: lon2_deg [-180.0, 180.0]
pub fn gnss_great_circle_bearing_deg(lat1_deg: f64, lon1_deg: f64, lat2_deg: f64, lon2_deg: f64) -> Option<f64> {
    if !lat1_deg.is_finite() || !lon1_deg.is_finite() || !lat2_deg.is_finite() || !lon2_deg.is_finite() ||
       lat1_deg < -90.0 || lat1_deg > 90.0 || lat2_deg < -90.0 || lat2_deg > 90.0 {
        return None;
    }
    let phi1 = lat1_deg * PI / 180.0;
    let phi2 = lat2_deg * PI / 180.0;
    let delta_lambda = (lon2_deg - lon1_deg) * PI / 180.0;

    let y = delta_lambda.sin() * phi2.cos();
    let x = phi1.cos() * phi2.sin() - phi1.sin() * phi2.cos() * delta_lambda.cos();
    let bearing_rad = y.atan2(x);
    let mut bearing_deg = bearing_rad * 180.0 / PI;
    if bearing_deg < 0.0 {
        bearing_deg += 360.0;
    }
    Some(bearing_deg)
}

/// ## Math: 偽距觀測量之衛星鐘差、電離層與對流層延遲校正
/// @outputs: 1
/// @domain: pseudorange_m [0.0, 1e8]
/// @domain: sat_clock_bias_s [-1.0, 1.0]
/// @domain: iono_delay_m [-100.0, 500.0]
/// @domain: tropo_delay_m [0.0, 100.0]
pub fn gnss_pseudorange_clock_bias_corr(
    pseudorange_m: f64,
    sat_clock_bias_s: f64,
    iono_delay_m: f64,
    tropo_delay_m: f64,
) -> f64 {
    if !pseudorange_m.is_finite() || !sat_clock_bias_s.is_finite() ||
       !iono_delay_m.is_finite() || !tropo_delay_m.is_finite() {
        return 0.0;
    }
    pseudorange_m + SPEED_OF_LIGHT * sat_clock_bias_s - iono_delay_m - tropo_delay_m
}

/// ## Filter: 高精度 RTK 載波相位周跳檢測 (Doppler Carrier Phase Cycle-Slip Detector)
/// @outputs: 1
/// @domain: dt_s [0.001, 10.0]
/// @domain: threshold_cycles [0.1, 100.0]
pub fn gnss_carrier_phase_cycle_slip_detector(
    phase_now: f64,
    phase_prev: f64,
    doppler_hz: f64,
    dt_s: f64,
    threshold_cycles: f64,
) -> bool {
    if !phase_now.is_finite() || !phase_prev.is_finite() || !doppler_hz.is_finite() ||
       !dt_s.is_finite() || dt_s <= 0.0 || !threshold_cycles.is_finite() || threshold_cycles <= 0.0 {
        return false;
    }
    let expected_diff = -doppler_hz * dt_s;
    let actual_diff = phase_now - phase_prev;
    (actual_diff - expected_diff).abs() > threshold_cycles
}
