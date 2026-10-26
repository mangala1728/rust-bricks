//! # Radar MIMO & Target Tracking Micro-Bricks (雷達 MIMO 虛擬陣列與目標追蹤微積木)
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 MIMO 虛擬天線陣列索引、TDMA 發射通道輪詢排程、Barker 相位編碼序列、
//! CDMA 脈衝壓縮匹配濾波、Capon/MVDR 自適應波束成形權重、GNN 歐氏距與馬氏距門限檢定、
//! 卡爾曼濾波創新更新步進、航跡生命週期狀態評分、雙 PRF 多普勒速度解模糊。

/// ## Map: MIMO 虛擬天線陣列等效孔徑索引換算
/// 虛擬陣元索引 k = tx_idx * num_rx + rx_idx
/// @outputs: 1
pub fn radar_mimo_virtual_array_index(tx_idx: usize, rx_idx: usize, num_rx: usize) -> usize {
    if num_rx == 0 {
        return 0;
    }
    tx_idx.saturating_mul(num_rx).saturating_add(rx_idx)
}

/// ## Schedule: TDMA MIMO 發射天線分時輪詢排程器
/// @outputs: 1
pub fn radar_mimo_tdma_tx_active_mask(cycle_idx: usize, num_tx: usize) -> usize {
    if num_tx == 0 {
        return 0;
    }
    cycle_idx % num_tx
}

/// ## Gen: CDMA/PMCW 雷達 Barker 雙相編碼 (Biphase) 晶片序列
/// 支援 Barker-7, Barker-11, Barker-13，輸出 +1.0 或 -1.0
/// @outputs: 1
pub fn radar_mimo_barker_code_biphase(code_len: u8, chip_idx: usize) -> Option<f32> {
    match code_len {
        7 => {
            // Barker 7: +1, +1, +1, -1, -1, +1, -1
            const B7: [f32; 7] = [1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0];
            B7.get(chip_idx).copied()
        }
        11 => {
            // Barker 11: +1, +1, +1, -1, -1, -1, +1, -1, -1, +1, -1
            const B11: [f32; 11] = [1.0, 1.0, 1.0, -1.0, -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0];
            B11.get(chip_idx).copied()
        }
        13 => {
            // Barker 13: +1, +1, +1, +1, +1, -1, -1, +1, +1, -1, +1, -1, +1
            const B13: [f32; 13] = [1.0, 1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0, -1.0, 1.0];
            B13.get(chip_idx).copied()
        }
        _ => None,
    }
}

/// ## Filter: CDMA PMCW 雷達脈衝壓縮解調匹配濾波步進
/// @outputs: 2
/// @domain: rx_i [-1e6, 1e6]
/// @domain: rx_q [-1e6, 1e6]
/// @domain: chip_biphase [-1.0, 1.0]
pub fn radar_mimo_phase_matched_filter_step(
    rx_i: f32,
    rx_q: f32,
    chip_biphase: f32,
) -> (f32, f32) {
    if !rx_i.is_finite() || !rx_q.is_finite() || !chip_biphase.is_finite() {
        return (0.0, 0.0);
    }
    (rx_i * chip_biphase, rx_q * chip_biphase)
}

/// ## Solve: Capon / MVDR 2x2 自適應波束成形非歸一化權重向量 (w = R^-1 * a)
/// @outputs: 1
pub fn radar_capon_beamforming_weight_num(
    cov_inv_2x2: [[f32; 2]; 2],
    steer_vec: [f32; 2],
) -> Option<[f32; 2]> {
    for row in &cov_inv_2x2 {
        for &val in row {
            if !val.is_finite() { return None; }
        }
    }
    if !steer_vec[0].is_finite() || !steer_vec[1].is_finite() {
        return None;
    }
    let w0 = cov_inv_2x2[0][0] * steer_vec[0] + cov_inv_2x2[0][1] * steer_vec[1];
    let w1 = cov_inv_2x2[1][0] * steer_vec[0] + cov_inv_2x2[1][1] * steer_vec[1];
    if w0.is_finite() && w1.is_finite() {
        Some([w0, w1])
    } else {
        None
    }
}

/// ## Distance: 全局最近鄰 (GNN) 2D 歐氏距離波門檢定
/// @outputs: 1
pub fn radar_gnn_euclidean_distance_2d(track_pos: [f32; 2], meas_pos: [f32; 2]) -> f32 {
    if !track_pos[0].is_finite() || !track_pos[1].is_finite() ||
       !meas_pos[0].is_finite() || !meas_pos[1].is_finite() {
        return f32::INFINITY;
    }
    let dx = track_pos[0] - meas_pos[0];
    let dy = track_pos[1] - meas_pos[1];
    (dx * dx + dy * dy).sqrt()
}

/// ## Distance: 雷達目標追蹤馬氏距離平方 (Mahalanobis Distance Squared: d^2 = y^T * S^-1 * y)
/// @outputs: 1
pub fn radar_mahalanobis_distance_sq_2d(residual: [f32; 2], s_inv: [[f32; 2]; 2]) -> Option<f32> {
    if !residual[0].is_finite() || !residual[1].is_finite() {
        return None;
    }
    for row in &s_inv {
        for &val in row {
            if !val.is_finite() { return None; }
        }
    }
    let temp0 = residual[0] * s_inv[0][0] + residual[1] * s_inv[1][0];
    let temp1 = residual[0] * s_inv[0][1] + residual[1] * s_inv[1][1];
    let dist_sq = temp0 * residual[0] + temp1 * residual[1];
    if dist_sq.is_finite() && dist_sq >= 0.0 {
        Some(dist_sq)
    } else {
        None
    }
}

/// ## Update: 1D 卡爾曼濾波量測創新更新步進 (Kalman Innovation & Gain Update)
/// 回傳 (更新後狀態, 更新後方差, 卡爾曼增益)
/// @outputs: 3
/// @domain: meas [-1e6, 1e6]
/// @domain: pred [-1e6, 1e6]
/// @domain: var_pred [0.0001, 1e8]
/// @domain: var_meas [0.0001, 1e8]
pub fn radar_kalman_filter_innovation_step(
    meas: f32,
    pred: f32,
    var_pred: f32,
    var_meas: f32,
) -> Option<(f32, f32, f32)> {
    if !meas.is_finite() || !pred.is_finite() || !var_pred.is_finite() || !var_meas.is_finite() ||
       var_pred <= 0.0 || var_meas <= 0.0 {
        return None;
    }
    let s = var_pred + var_meas;
    let k = var_pred / s;
    let innovation = meas - pred;
    let updated_state = pred + k * innovation;
    let updated_var = (1.0 - k) * var_pred;
    Some((updated_state, updated_var, k))
}

/// ## Score: 航跡 M-of-N 存活確認與信心衰減評分
/// 回傳 (是否應保留航跡, 航跡信心度 0.0~1.0)
/// @outputs: 2
pub fn radar_track_state_coast_decay_score(hits: u32, misses: u32, max_misses: u32) -> (bool, f32) {
    if max_misses == 0 || misses >= max_misses {
        return (false, 0.0);
    }
    let total = hits + misses;
    let hit_ratio = if total > 0 { (hits as f32) / (total as f32) } else { 0.0 };
    let coast_penalty = 1.0 - (misses as f32) / (max_misses as f32);
    let confidence = (hit_ratio * coast_penalty).clamp(0.0, 1.0);
    (true, confidence)
}

/// ## Solve: 雙 PRF 多普勒模糊速度解算 (Dual-PRF Velocity Disambiguation)
/// @outputs: 1
/// @domain: v_measured [-1e4, 1e4]
/// @domain: v_nyquist1 [0.1, 1e4]
/// @domain: v_nyquist2 [0.1, 1e4]
pub fn radar_doppler_velocity_disambiguation(
    v_meas1: f32,
    v_meas2: f32,
    v_nyquist1: f32,
    v_nyquist2: f32,
) -> Option<f32> {
    if !v_meas1.is_finite() || !v_meas2.is_finite() || !v_nyquist1.is_finite() || !v_nyquist2.is_finite() ||
       v_nyquist1 <= 0.0 || v_nyquist2 <= 0.0 {
        return None;
    }
    // 依據兩不同 PRF 測得之折疊速度與中國剩餘定理 (CRT) 求解真速度
    let mut best_v = None;
    let mut min_k_mag = i32::MAX;
    for k1 in -6i32..=6i32 {
        let cand1 = v_meas1 + (k1 as f32) * 2.0 * v_nyquist1;
        for k2 in -6i32..=6i32 {
            let cand2 = v_meas2 + (k2 as f32) * 2.0 * v_nyquist2;
            let diff = (cand1 - cand2).abs();
            if diff < 0.2 {
                let k_mag = k1.abs() + k2.abs();
                if k_mag < min_k_mag {
                    min_k_mag = k_mag;
                    best_v = Some(0.5 * (cand1 + cand2));
                }
            }
        }
    }
    best_v
}
