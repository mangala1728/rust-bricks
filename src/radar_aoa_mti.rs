//! # Radar AoA & MTI Micro-Bricks (雷達測角與動目標顯示雜波抑制微積木)
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 MTI 二脈衝/三脈衝延遲消除器、雙天線相位單脈衝測角、和差單脈衝比、
//! 雷達盲速計算、Range-Doppler 座標換算、CSR 雜波訊噪比、FMCW 最大無模糊距離/速度、都卜勒頻心。

/// ## Map: MTI 雙脈衝延遲消除器 (Two-Pulse Delay Line Canceler: y[n] = x[n] - x[n-1])
/// @outputs: 1
pub fn radar_mti_two_pulse_canceler_step(curr_sample: f32, prev_sample: f32) -> f32 {
    if curr_sample.is_nan() || prev_sample.is_nan() {
        return 0.0;
    }
    curr_sample - prev_sample
}

/// ## Map: MTI 三脈衝延遲消除器 (Three-Pulse Canceler: y[n] = x[n] - 2*x[n-1] + x[n-2])
/// @outputs: 1
pub fn radar_mti_three_pulse_canceler_step(curr_sample: f32, prev1_sample: f32, prev2_sample: f32) -> f32 {
    if curr_sample.is_nan() || prev1_sample.is_nan() || prev2_sample.is_nan() {
        return 0.0;
    }
    curr_sample - 2.0 * prev1_sample + prev2_sample
}

/// ## Map: 雙天線相位單脈衝測角 (AoA: theta = arcsin(delta_phi * lambda / (2 * pi * d)))
/// @domain: wavelength_m [0.0001, 10.0]
/// @domain: d_spacing_m [0.0001, 10.0]
/// @outputs: 1
pub fn radar_aoa_phase_monopulse(delta_phase_rad: f32, wavelength_m: f32, d_spacing_m: f32) -> Option<f32> {
    if wavelength_m < 0.0001 || wavelength_m > 10.0 || d_spacing_m < 0.0001 || d_spacing_m > 10.0 || delta_phase_rad.is_nan() {
        return None;
    }
    let pi = std::f32::consts::PI;
    let ratio = (delta_phase_rad * wavelength_m) / (2.0 * pi * d_spacing_m);
    if ratio < -1.0 || ratio > 1.0 {
        return None; // 超出視角極限
    }
    Some(ratio.asin())
}

/// ## Map: 和差單脈衝比值實部估計 (Real Part of Delta / Sigma)
/// @outputs: 1
pub fn radar_monopulse_ratio_slope(delta_i: f32, delta_q: f32, sigma_i: f32, sigma_q: f32) -> Option<f32> {
    let sigma_mag_sq = sigma_i * sigma_i + sigma_q * sigma_q;
    if sigma_mag_sq < 1e-12 || delta_i.is_nan() || delta_q.is_nan() {
        return None;
    }
    // Re((Delta_I + j Delta_Q) * (Sigma_I - j Sigma_Q)) / |Sigma|^2
    let real_num = delta_i * sigma_i + delta_q * sigma_q;
    Some(real_num / sigma_mag_sq)
}

/// ## Map: 第 n 階雷達盲速解算 (v_blind = n * lambda * PRF / 2)
/// @domain: wavelength_m [0.0001, 10.0]
/// @domain: prf_hz [1.0, 1000000.0]
/// @outputs: 1
pub fn radar_blind_speed_mps(wavelength_m: f32, prf_hz: f32, n_blind: u32) -> Option<f32> {
    if wavelength_m < 0.0001 || wavelength_m > 10.0 || prf_hz < 1.0 || prf_hz > 1_000_000.0 {
        return None;
    }
    Some((n_blind as f32) * wavelength_m * prf_hz * 0.5)
}

/// ## Map: Range-Doppler 2D 矩陣網格索引轉實體距離 (m) 與徑向速度 (m/s)
/// @outputs: 1
pub fn radar_range_doppler_index_to_physical(
    range_bin: usize,
    doppler_bin: usize,
    range_res_m: f32,
    vel_res_mps: f32,
    num_doppler_bins: usize,
) -> Option<(f32, f32)> {
    if range_res_m <= 0.0 || vel_res_mps <= 0.0 || num_doppler_bins == 0 || doppler_bin >= num_doppler_bins {
        return None;
    }
    let range = (range_bin as f32) * range_res_m;
    let half_doppler = (num_doppler_bins / 2) as f32;
    let vel = ((doppler_bin as f32) - half_doppler) * vel_res_mps;
    Some((range, vel))
}

/// ## Map: 雜波訊號比 (CSR, Clutter-to-Signal Ratio in dB: 10 * log10(Pc / Ps))
/// @outputs: 1
pub fn radar_clutter_to_signal_ratio_db(clutter_power: f32, signal_power: f32) -> Option<f32> {
    if clutter_power <= 0.0 || signal_power <= 0.0 || clutter_power.is_nan() || signal_power.is_nan() {
        return None;
    }
    let ratio = clutter_power / signal_power;
    // 零依賴自然對數與 log10
    let ln_val = fast_ln_f32(ratio);
    Some(10.0 * ln_val * 0.4342944819032518) // ln to log10
}

/// ## Map: FMCW 雷達最大無模糊探測距離 (R_max = fs * c * Tc / (2 * B))
/// @domain: sampling_rate_hz [1000.0, 100000000.0]
/// @domain: chirp_bandwidth_hz [100000.0, 10000000000.0]
/// @domain: chirp_duration_s [0.000001, 1.0]
/// @outputs: 1
pub fn radar_fmcw_max_unambiguous_range(
    sampling_rate_hz: f32,
    chirp_bandwidth_hz: f32,
    chirp_duration_s: f32,
) -> Option<f32> {
    if sampling_rate_hz < 1000.0 || sampling_rate_hz > 100_000_000.0
        || chirp_bandwidth_hz < 100_000.0 || chirp_bandwidth_hz > 10_000_000_000.0
        || chirp_duration_s < 0.000_001 || chirp_duration_s > 1.0
    {
        return None;
    }
    let c = 299_792_458.0f32; // 光速 m/s
    let slope = chirp_bandwidth_hz / chirp_duration_s;
    let r_max = (sampling_rate_hz * c) / (2.0 * slope);
    Some(r_max)
}

/// ## Map: FMCW 雷達最大無模糊速度 (v_max = c / (4 * fc * Tc))
/// @domain: chirp_interval_s [0.000001, 1.0]
/// @domain: carrier_freq_hz [1000000.0, 300000000000.0]
/// @outputs: 1
pub fn radar_fmcw_max_unambiguous_velocity(chirp_interval_s: f32, carrier_freq_hz: f32) -> Option<f32> {
    if chirp_interval_s < 0.000_001 || chirp_interval_s > 1.0
        || carrier_freq_hz < 1_000_000.0 || carrier_freq_hz > 300_000_000_000.0
    {
        return None;
    }
    let c = 299_792_458.0f32;
    let v_max = c / (4.0 * carrier_freq_hz * chirp_interval_s);
    Some(v_max)
}

/// ## Map: 都卜勒頻心估算 (Doppler Spectral Centroid 一階矩: sum(f_i * P_i) / sum(P_i))
/// @outputs: 1
pub fn radar_doppler_centroid_hz(power_spectrum: &[f32], freq_step_hz: f32) -> Option<f32> {
    if power_spectrum.is_empty() || freq_step_hz <= 0.0 || freq_step_hz.is_nan() {
        return None;
    }
    let mut total_power = 0.0f32;
    let mut weighted_freq = 0.0f32;
    for (i, &p) in power_spectrum.iter().enumerate() {
        if p < 0.0 || p.is_nan() {
            continue;
        }
        let f = (i as f32) * freq_step_hz;
        total_power += p;
        weighted_freq += f * p;
    }
    if total_power < 1e-12 {
        return None;
    }
    Some(weighted_freq / total_power)
}

/// 內部快速 ln 逼近 (純函式)
/// @outputs: 1
fn fast_ln_f32(x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }
    let mut val = x;
    let mut k = 0i32;
    while val > 2.0 {
        val *= 0.5;
        k += 1;
    }
    while val < 1.0 {
        val *= 2.0;
        k -= 1;
    }
    let z = (val - 1.0) / (val + 1.0);
    let z2 = z * z;
    let ln_approx = 2.0 * z * (1.0 + z2 / 3.0 + (z2 * z2) / 5.0);
    ln_approx + (k as f32) * 0.69314718
}
