//! # Radar & LiDAR Point Cloud Clustering & Multi-Model Tracking Micro-Bricks
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 DBSCAN 點雲鄰域密度判定、3D 質心與平均都卜勒徑向速度解算、
//! JPDA (聯合機率資料關聯) 權重歸一化、IMM (交互多模型) 模式機率轉移更新、
//! CTRV (等轉向速度) 非線性車輛運動預測、人體步態微都卜勒特徵提取、
//! 生命徵象微弱呼吸與心跳頻率分離、航跡確認雙門檻遲滯狀態機。

/// ## Map: DBSCAN 點雲密度可達鄰域判定 (DBSCAN Density Reachable Step)
/// @outputs: 1
/// @domain: dist_sq [0.0, 1e8], eps_sq [1e-6, 1e8]
pub fn radar_dbscan_density_reachable(dist_sq: f32, eps_sq: f32) -> bool {
    if !dist_sq.is_finite() || !eps_sq.is_finite() || dist_sq < 0.0 || eps_sq <= 0.0 {
        return false;
    }
    dist_sq <= eps_sq
}

/// ## Calc: 點雲聚類 3D 質心與平均都卜勒速度估算 (Cluster Centroid & Velocity)
/// 回傳 (質心 [cx, cy, cz], 平均徑向速度 avg_v)
/// @outputs: 1
pub fn radar_cluster_centroid_and_radial_velocity(
    points: &[[f32; 3]],
    v_rad: &[f32],
    cluster_indices: &[usize],
) -> Option<([f32; 3], f32)> {
    if cluster_indices.is_empty() || points.len() != v_rad.len() {
        return None;
    }
    let mut sum_x = 0.0f32;
    let mut sum_y = 0.0f32;
    let mut sum_z = 0.0f32;
    let mut sum_v = 0.0f32;
    let count = cluster_indices.len() as f32;

    for &idx in cluster_indices {
        if idx >= points.len() { return None; }
        let p = points[idx];
        let v = v_rad[idx];
        if !p[0].is_finite() || !p[1].is_finite() || !p[2].is_finite() || !v.is_finite() {
            return None;
        }
        sum_x += p[0];
        sum_y += p[1];
        sum_z += p[2];
        sum_v += v;
    }
    Some(([sum_x / count, sum_y / count, sum_z / count], sum_v / count))
}

/// ## Step: JPDA (Joint Probabilistic Data Association) 關聯權重解算
/// 依據馬氏距離平方 (d_sq) 透過高斯似然與假警報密度進行歸一化權重分配
/// @outputs: 1
pub fn radar_jpda_association_weights(
    mahalanobis_dists_sq: &[f32],
    clutter_density: f32,
    out_weights: &mut [f32],
) -> bool {
    if mahalanobis_dists_sq.is_empty() || mahalanobis_dists_sq.len() != out_weights.len() || !clutter_density.is_finite() || clutter_density <= 0.0 {
        return false;
    }
    let mut sum_lik = clutter_density;
    for (i, &d_sq) in mahalanobis_dists_sq.iter().enumerate() {
        if !d_sq.is_finite() || d_sq < 0.0 { return false; }
        let lik = (-0.5 * d_sq).exp();
        out_weights[i] = lik;
        sum_lik += lik;
    }
    if sum_lik <= 0.0 || !sum_lik.is_finite() { return false; }
    for w in out_weights.iter_mut() {
        *w /= sum_lik;
    }
    true
}

/// ## Step: IMM (Interacting Multiple Model) 交互多模型模式機率馬可夫更新
/// mode_probs 為 3 種運動模型機率 [CV, CA, CT]，trans_mat 為 3x3 轉移矩陣，likelihoods 為觀測似然
/// @outputs: 1
pub fn radar_imm_mode_probabilities_step(
    mode_probs: &mut [f32; 3],
    trans_mat: &[[f32; 3]; 3],
    likelihoods: &[f32; 3],
) -> bool {
    let mut mixed = [0.0f32; 3];
    for j in 0..3 {
        let mut sum_trans = 0.0f32;
        for i in 0..3 {
            let p = mode_probs[i];
            let t = trans_mat[i][j];
            if !p.is_finite() || !t.is_finite() { return false; }
            sum_trans += p * t;
        }
        mixed[j] = sum_trans;
    }
    let mut sum_lik = 0.0f32;
    for j in 0..3 {
        let lik = likelihoods[j];
        if !lik.is_finite() || lik < 0.0 { return false; }
        mixed[j] *= lik;
        sum_lik += mixed[j];
    }
    if sum_lik <= 0.0 || !sum_lik.is_finite() { return false; }
    for j in 0..3 {
        mode_probs[j] = mixed[j] / sum_lik;
    }
    true
}

/// ## Step: CTRV (Constant Turn Rate and Velocity) 車輛運動模型狀態外推
/// state: [x, y, v, yaw, yaw_rate]，dt 為取樣時間 (秒)
/// @outputs: 1
/// @domain: dt [1e-4, 10.0]
pub fn radar_ctrv_motion_model_predict(state: [f32; 5], dt: f32) -> [f32; 5] {
    if !dt.is_finite() || dt <= 0.0 {
        return state;
    }
    let [px, py, v, yaw, yaw_rate] = state;
    if !px.is_finite() || !py.is_finite() || !v.is_finite() || !yaw.is_finite() || !yaw_rate.is_finite() {
        return state;
    }
    if yaw_rate.abs() < 1e-5 {
        // 直線等速運動極限逼近
        let next_px = px + v * yaw.cos() * dt;
        let next_py = py + v * yaw.sin() * dt;
        [next_px, next_py, v, yaw, yaw_rate]
    } else {
        // 圓弧旋轉非線性運動
        let next_px = px + (v / yaw_rate) * ((yaw + yaw_rate * dt).sin() - yaw.sin());
        let next_py = py + (v / yaw_rate) * (-((yaw + yaw_rate * dt).cos()) + yaw.cos());
        let next_yaw = yaw + yaw_rate * dt;
        [next_px, next_py, v, next_yaw, yaw_rate]
    }
}

/// ## Sep: 生醫毫米波微弱胸腔呼吸與心跳頻帶分離估算 (Vital Sign Separation)
/// phase_series 為相位時間序列，fs 為取樣率 (Hz)。回傳 (呼吸頻率 Hz, 心跳頻率 Hz)
/// @outputs: 1
/// @domain: fs [1.0, 1000.0]
pub fn radar_vital_signs_frequency_separation(
    phase_series: &[f32],
    fs: f32,
) -> (f32, f32) {
    if phase_series.len() < 16 || !fs.is_finite() || fs <= 0.0 {
        return (0.0, 0.0);
    }
    let n = phase_series.len();
    // 粗頻譜估算：呼吸頻段 0.1..0.5 Hz，心跳頻段 0.8..2.5 Hz
    let mut max_breath_pwr = 0.0f32;
    let mut best_breath_f = 0.25f32;
    let mut max_heart_pwr = 0.0f32;
    let mut best_heart_f = 1.2f32;

    for k in 1..(n / 2) {
        let f = (k as f32 * fs) / (n as f32);
        let mut re = 0.0f32;
        let mut im = 0.0f32;
        const TWO_PI: f32 = 6.2831853;
        for (i, &p) in phase_series.iter().enumerate() {
            let angle = -TWO_PI * (k as f32) * (i as f32) / (n as f32);
            re += p * angle.cos();
            im += p * angle.sin();
        }
        let pwr = re * re + im * im;
        if (0.1..=0.5).contains(&f) && pwr > max_breath_pwr {
            max_breath_pwr = pwr;
            best_breath_f = f;
        } else if (0.8..=2.5).contains(&f) && pwr > max_heart_pwr {
            max_heart_pwr = pwr;
            best_heart_f = f;
        }
    }
    (best_breath_f, best_heart_f)
}

/// ## Calc: Swerling I 型慢起伏雷達截面積 (RCS) 目標瞬時 SNR 衰落
/// 公式：snr_inst = -mean_snr * ln(u_rand)，其中 u_rand in (0, 1]
/// @outputs: 1
/// @domain: mean_snr [0.0, 1e6], u_rand [1e-7, 1.0]
pub fn radar_swerling1_snr_fluctuation(mean_snr: f32, u_rand: f32) -> f32 {
    if !mean_snr.is_finite() || !u_rand.is_finite() || mean_snr <= 0.0 || u_rand <= 0.0 || u_rand > 1.0 {
        return 0.0;
    }
    -mean_snr * u_rand.ln()
}

/// ## Step: 航跡存活狀態三態遲滯確認狀態機 (Track Confirmation Hysteresis)
/// state: 0=Tentative (暫態), 1=Confirmed (已確認), 2=Deleted (消亡)
/// @outputs: 1
/// @domain: current_state [0, 2], hit_count [0, 100], miss_count [0, 100]
pub fn radar_track_lifecycle_hysteresis_step(
    current_state: u8,
    hit_count: u8,
    miss_count: u8,
    n_hits_to_confirm: u8,
    n_miss_to_delete: u8,
) -> u8 {
    match current_state {
        0 => {
            if hit_count >= n_hits_to_confirm { 1 } // 晉升為 Confirmed
            else if miss_count >= n_miss_to_delete { 2 } // 判定為虛警雜波，刪除
            else { 0 }
        }
        1 => {
            if miss_count >= n_miss_to_delete { 2 } // 連續失跟，刪除
            else { 1 }
        }
        _ => 2,
    }
}
