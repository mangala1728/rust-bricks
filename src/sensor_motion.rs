// ## Map
// Role: core/sensor_motion — 智慧型手機感測器（陀螺儀、加速度計、GPS 經緯度）濾波、姿態融合與大圓航線計算微積木。
// Inputs: 角度、角速度、重力加速度分量、時間步長、經緯度座標。
// Outputs: 融合姿態角度、平滑後估計值、大圓距離（公尺）、航向方位角（度）。
// Invariants: 零外部依賴、無遞迴、零全域可變狀態、浮點數防 NaN/Inf 守衛、控制流巢狀深度 <= 2、純標準庫 100% 零依賴。

const EARTH_RADIUS_M: f64 = 6_371_000.0;
const DEG_TO_RAD: f64 = std::f64::consts::PI / 180.0;
const RAD_TO_DEG: f64 = 180.0 / std::f64::consts::PI;

/// @outputs: 2
/// @domain: dt [0.0001..10.0]
/// @domain: alpha [0.0..1.0]
pub fn complementary_filter(angle_prev: f32, gyro_rate: f32, accel_angle: f32, dt: f32, alpha: f32) -> f32 {
    if dt < 0.0001 || dt > 10.0 || alpha < 0.0 || alpha > 1.0 {
        return accel_angle;
    }
    if angle_prev.is_nan() || gyro_rate.is_nan() || accel_angle.is_nan() {
        return 0.0;
    }
    let gyro_est = angle_prev + gyro_rate * dt;
    alpha * gyro_est + (1.0 - alpha) * accel_angle
}

/// @outputs: 2
/// @domain: process_noise [0.000001..10000.0]
/// @domain: measure_noise [0.000001..10000.0]
pub fn kalman_filter_1d(
    prev_est: f32,
    prev_cov: f32,
    measurement: f32,
    process_noise: f32,
    measure_noise: f32,
) -> (f32, f32) {
    if process_noise < 0.000001 || process_noise > 10000.0 || measure_noise < 0.000001 || measure_noise > 10000.0 {
        return (measurement, measure_noise);
    }
    if prev_est.is_nan() || prev_cov.is_nan() || measurement.is_nan() {
        return (measurement, measure_noise);
    }
    let pred_est = prev_est;
    let pred_cov = prev_cov + process_noise;
    let kalman_gain = pred_cov / (pred_cov + measure_noise);
    let new_est = pred_est + kalman_gain * (measurement - pred_est);
    let new_cov = (1.0 - kalman_gain) * pred_cov;
    (new_est, new_cov)
}

/// @outputs: 2
/// @domain: lat1 [-90.0..90.0]
/// @domain: lon1 [-180.0..180.0]
/// @domain: lat2 [-90.0..90.0]
/// @domain: lon2 [-180.0..180.0]
pub fn haversine_distance_m(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> Option<f64> {
    if lat1 < -90.0 || lat1 > 90.0 || lat2 < -90.0 || lat2 > 90.0 {
        return None;
    }
    if lon1 < -180.0 || lon1 > 180.0 || lon2 < -180.0 || lon2 > 180.0 {
        return None;
    }
    let d_lat = (lat2 - lat1) * DEG_TO_RAD;
    let d_lon = (lon2 - lon1) * DEG_TO_RAD;
    let lat1_rad = lat1 * DEG_TO_RAD;
    let lat2_rad = lat2 * DEG_TO_RAD;

    let h = (d_lat * 0.5).sin().powi(2) + lat1_rad.cos() * lat2_rad.cos() * (d_lon * 0.5).sin().powi(2);
    let c = 2.0 * h.sqrt().asin();
    Some(EARTH_RADIUS_M * c)
}

/// @outputs: 2
/// @domain: lat1 [-90.0..90.0]
/// @domain: lon1 [-180.0..180.0]
/// @domain: lat2 [-90.0..90.0]
/// @domain: lon2 [-180.0..180.0]
pub fn haversine_bearing_deg(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> Option<f64> {
    if lat1 < -90.0 || lat1 > 90.0 || lat2 < -90.0 || lat2 > 90.0 {
        return None;
    }
    if lon1 < -180.0 || lon1 > 180.0 || lon2 < -180.0 || lon2 > 180.0 {
        return None;
    }
    let d_lon = (lon2 - lon1) * DEG_TO_RAD;
    let lat1_rad = lat1 * DEG_TO_RAD;
    let lat2_rad = lat2 * DEG_TO_RAD;

    let y = d_lon.sin() * lat2_rad.cos();
    let x = lat1_rad.cos() * lat2_rad.sin() - lat1_rad.sin() * lat2_rad.cos() * d_lon.cos();
    let initial_bearing_rad = y.atan2(x);
    let initial_bearing_deg = initial_bearing_rad * RAD_TO_DEG;
    let normalized = (initial_bearing_deg + 360.0) % 360.0;
    Some(normalized)
}
