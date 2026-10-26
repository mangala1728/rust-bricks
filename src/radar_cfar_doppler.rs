// ## Map
// Role: radar/radar_cfar_doppler — 零依賴毫米波與都卜勒雷達訊號處理 (CA-CFAR 恒虛警率、都卜勒測速、微都卜勒呼吸胸腔位移、點雲分群與 MIMO 虛擬孔徑)。
// Inputs: 訊號功率切片、保護/參考窗長度、閾值因子、都卜勒頻率、載波頻率、相位差、TX/RX 天線位置。
// Outputs: 標記之目標峰值、速度 (m/s)、胸腔微動位移 (mm)、點雲聚類標籤、MIMO 虛擬天線陣列座標。
// Invariants: 零外部依賴、純 Rust 實現、除零剛性防護、有界陣列切片、單一函式 <= 60 行。

const SPEED_OF_LIGHT_M_S: f32 = 299_792_458.0;

/// @outputs: 1
/// @domain: guard_len [1..32]
/// @domain: train_len [1..64]
/// @domain: alpha [1.0..100.0]
pub fn radar_ca_cfar_detect_1d(
    signal: &[f32],
    guard_len: usize,
    train_len: usize,
    alpha: f32,
    detections: &mut [bool],
) -> usize {
    let n = signal.len();
    let margin = guard_len + train_len;
    if n < 2 * margin + 1 || detections.len() != n || alpha <= 0.0 || !alpha.is_finite() {
        return 0;
    }
    for d in detections.iter_mut() {
        *d = false;
    }
    let mut hits = 0;
    let num_train = (2 * train_len) as f32;

    for i in margin..(n - margin) {
        let cut_val = signal[i];
        if !cut_val.is_finite() {
            continue;
        }
        let mut sum_noise = 0.0f32;
        // 左側參考單元
        for j in (i - margin)..(i - guard_len) {
            sum_noise += signal[j];
        }
        // 右側參考單元
        for j in (i + guard_len + 1)..=(i + margin) {
            sum_noise += signal[j];
        }
        let noise_avg = sum_noise / num_train;
        let threshold = alpha * noise_avg;
        if cut_val > threshold {
            detections[i] = true;
            hits += 1;
        }
    }
    hits
}

/// @outputs: 2
/// @domain: carrier_freq_hz [1e6..300e9]
pub fn radar_doppler_velocity_mps(doppler_freq_hz: f32, carrier_freq_hz: f32) -> Option<f32> {
    if carrier_freq_hz <= 0.0 || !doppler_freq_hz.is_finite() || !carrier_freq_hz.is_finite() {
        return None;
    }
    // v = (c * f_d) / (2 * f_c)
    let vel = (SPEED_OF_LIGHT_M_S * doppler_freq_hz) / (2.0 * carrier_freq_hz);
    if vel.is_finite() { Some(vel) } else { None }
}

/// @outputs: 2
/// @domain: carrier_freq_ghz [1.0..300.0]
pub fn radar_micro_doppler_chest_displacement_mm(phase_diff_rad: f32, carrier_freq_ghz: f32) -> Option<f32> {
    if carrier_freq_ghz <= 0.0 || !phase_diff_rad.is_finite() || !carrier_freq_ghz.is_finite() {
        return None;
    }
    // 波長 lambda = c / f_c (單位: mm) -> 299.792458 / f_c(GHz)
    let lambda_mm = 299.792458 / carrier_freq_ghz;
    // 微動位移 Delta d = (lambda * Delta phi) / (4 * pi)
    let displacement = (lambda_mm * phase_diff_rad) / (4.0 * std::f32::consts::PI);
    if displacement.is_finite() { Some(displacement) } else { None }
}

/// @outputs: 1
/// @domain: max_cluster_dist [0.001..1000.0]
pub fn radar_cluster_points_1d(points: &[f32], max_cluster_dist: f32, cluster_ids: &mut [usize]) -> usize {
    let n = points.len();
    if n == 0 || cluster_ids.len() != n || max_cluster_dist <= 0.0 || !max_cluster_dist.is_finite() {
        return 0;
    }
    let mut current_cluster = 0;
    cluster_ids[0] = current_cluster;

    for i in 1..n {
        let diff = (points[i] - points[i - 1]).abs();
        if diff > max_cluster_dist || !diff.is_finite() {
            current_cluster += 1;
        }
        cluster_ids[i] = current_cluster;
    }
    current_cluster + 1
}

/// @outputs: 1
pub fn radar_mimo_virtual_array_positions(tx_pos: &[f32], rx_pos: &[f32], out_virtual_pos: &mut [f32]) -> usize {
    if tx_pos.is_empty() || rx_pos.is_empty() {
        return 0;
    }
    let total_elements = tx_pos.len() * rx_pos.len();
    let write_len = total_elements.min(out_virtual_pos.len());
    let mut idx = 0;

    for &tx in tx_pos {
        for &rx in rx_pos {
            if idx >= write_len {
                return idx;
            }
            out_virtual_pos[idx] = tx + rx;
            idx += 1;
        }
    }
    idx
}
