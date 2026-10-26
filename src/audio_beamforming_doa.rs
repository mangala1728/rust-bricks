//! # 聲學聲場、麥克風陣列與 DOA 空間波束成形 (Aeroacoustics, Microphone Arrays & DOA Beamforming)
//!
//! 提供空氣聲速隨溫補償、空間混疊頻率極限、幾何延遲時間差 (TDOA)、
//! 2階 Lagrange 分數延遲濾波器、SRP-PHAT 相位變換核、ULA 空間導向向量、
//! MUSIC 虛擬譜估計、延遲求和 (DAS) 波束成形與 GSC 阻斷矩陣微積木。
//! 零外部依賴，符合 NASA JPL 規範與合約 09。

const PI: f32 = 3.14159265;

/// ## Map: 空氣中聲速隨攝氏溫度補償 (Acoustic Sound Speed in Air)
/// @outputs: 1
/// @domain: temp_celsius [-50.0..=80.0]
pub fn acoustic_sound_speed_air(temp_celsius: f32) -> Option<f32> {
    if !temp_celsius.is_finite() || temp_celsius < -50.0 || temp_celsius > 80.0 {
        return None;
    }
    let kelvin = temp_celsius + 273.15;
    if kelvin <= 0.0 {
        return None;
    }
    let c = 331.3 * (kelvin / 273.15).sqrt();
    Some(c)
}

/// ## Calc: 麥克風陣列空間混疊極限頻率 (Spatial Nyquist Aliasing Frequency)
/// @outputs: 1
/// @domain: mic_spacing_m [0.001..=1.0] && sound_speed [200.0..=500.0]
pub fn mic_array_spatial_aliasing_limit_hz(mic_spacing_m: f32, sound_speed: f32) -> Option<f32> {
    if !mic_spacing_m.is_finite() || !sound_speed.is_finite() {
        return None;
    }
    if mic_spacing_m < 0.001 || mic_spacing_m > 1.0 || sound_speed < 200.0 || sound_speed > 500.0 {
        return None;
    }
    let f_alias = sound_speed / (2.0 * mic_spacing_m);
    Some(f_alias)
}

/// ## Calc: 陣列單元幾何導向時延解算 (Microphone Array Steering Delay)
/// 回傳以秒為單位之傳播時延 tau = (x*cos(az) + y*sin(az)) / c
/// @outputs: 1
/// @domain: sound_speed >= 200.0 && sound_speed <= 500.0
pub fn mic_array_steering_delay_sec(
    mic_x: f32,
    mic_y: f32,
    azimuth_rad: f32,
    sound_speed: f32,
) -> Option<f32> {
    if !mic_x.is_finite() || !mic_y.is_finite() || !azimuth_rad.is_finite() || !sound_speed.is_finite() {
        return None;
    }
    if sound_speed < 200.0 || sound_speed > 500.0 {
        return None;
    }
    let proj = mic_x * azimuth_rad.cos() + mic_y * azimuth_rad.sin();
    Some(proj / sound_speed)
}

/// ## Calc: 2階 Lagrange 分數時延插值濾波係數 (2nd-Order Lagrange Fractional Delay)
/// 回傳濾波係數 [h0, h1, h2]
/// @outputs: 1
/// @domain: fraction [0.0..=1.0]
pub fn dsp_lagrange_fractional_delay_coeffs(fraction: f32) -> [f32; 3] {
    if !fraction.is_finite() {
        return [0.0, 1.0, 0.0];
    }
    let d = fraction.clamp(0.0, 1.0);
    let h0 = (d - 1.0) * (d - 2.0) * 0.5;
    let h1 = -d * (d - 2.0);
    let h2 = d * (d - 1.0) * 0.5;
    [h0, h1, h2]
}

/// ## Step: SRP-PHAT 頻域相位變換正規化權重 (SRP-PHAT Phase Transform Kernel)
/// 回傳 (正規化實部, 正規化虛部) = (re, im) / |re + j*im|
/// @outputs: 1
/// @domain: re, im 為有限實數
pub fn srp_phat_cross_phase_weight(re: f32, im: f32) -> (f32, f32) {
    if !re.is_finite() || !im.is_finite() {
        return (0.0, 0.0);
    }
    let mag_sq = re * re + im * im;
    if mag_sq <= 1e-12 {
        (0.0, 0.0)
    } else {
        let inv_mag = 1.0 / mag_sq.sqrt();
        (re * inv_mag, im * inv_mag)
    }
}

/// ## Calc: 均勻線陣 (ULA) 空間導向向量相位 (ULA Steering Vector Phase)
/// 回傳導向相位 (弧度) psi = -2 * PI * idx * (d / lambda) * sin(theta)
/// @outputs: 1
/// @domain: mic_spacing_m > 0.0 && wavelength_m > 0.0
pub fn ula_steering_vector_phase(
    element_idx: usize,
    mic_spacing_m: f32,
    wavelength_m: f32,
    azimuth_rad: f32,
) -> Option<f32> {
    if !mic_spacing_m.is_finite() || !wavelength_m.is_finite() || !azimuth_rad.is_finite() {
        return None;
    }
    if mic_spacing_m <= 1e-6 || wavelength_m <= 1e-6 {
        return None;
    }
    let phase = -2.0 * PI * (element_idx as f32) * (mic_spacing_m / wavelength_m) * azimuth_rad.sin();
    Some(phase)
}

/// ## Calc: MUSIC 空間虛擬譜值解算 (MUSIC Pseudo-Spectrum Value)
/// 回傳 P(theta) = 1 / proj_noise_norm_sq
/// @outputs: 1
/// @domain: proj_noise_norm_sq >= 0.0
pub fn music_pseudospectrum_value(proj_noise_norm_sq: f32) -> Option<f32> {
    if !proj_noise_norm_sq.is_finite() || proj_noise_norm_sq < 0.0 {
        return None;
    }
    if proj_noise_norm_sq <= 1e-12 {
        Some(1e12) // 極大峰值
    } else {
        Some(1.0 / proj_noise_norm_sq)
    }
}

/// ## Calc: 延遲求和 (DAS) 波束成形單步加權平均 (Delay-and-Sum Beamformer Step)
/// @outputs: 1
/// @domain: samples.len() == delays_fraction.len() && samples.len() > 0 && samples.len() <= 64
pub fn beamformer_delay_and_sum_step(samples: &[f32], delays_fraction: &[f32]) -> Option<f32> {
    if samples.is_empty() || samples.len() != delays_fraction.len() || samples.len() > 64 {
        return None;
    }
    let mut sum = 0.0f32;
    for (&s, &d) in samples.iter().zip(delays_fraction.iter()) {
        if !s.is_finite() || !d.is_finite() {
            return None;
        }
        let lag = dsp_lagrange_fractional_delay_coeffs(d);
        sum += s * lag[0];
    }
    Some(sum / samples.len() as f32)
}

/// ## Calc: 陣列指向性係數 (Directivity Index in dB)
/// 計算 DI = 10 * log10(main_power / avg_diffuse_power)
/// @outputs: 1
/// @domain: main_lobe_power > 0.0 && avg_diffuse_power > 0.0
pub fn directivity_index_db(main_lobe_power: f32, avg_diffuse_power: f32) -> Option<f32> {
    if !main_lobe_power.is_finite() || !avg_diffuse_power.is_finite() {
        return None;
    }
    if main_lobe_power <= 1e-12 || avg_diffuse_power <= 1e-12 {
        return None;
    }
    let ratio = main_lobe_power / avg_diffuse_power;
    Some(10.0 * ratio.log10())
}

/// ## Step: 廣義副瓣消除器 (GSC) 一階阻斷矩陣單步 (GSC First-Order Blocking Step)
/// 計算相鄰通道相消輸出 x1 - x2 (消除正入射目標信號)
/// @outputs: 1
/// @domain: x1, x2 皆為有限實數
pub fn gsc_blocking_matrix_step(x1: f32, x2: f32) -> f32 {
    if !x1.is_finite() || !x2.is_finite() {
        0.0
    } else {
        x1 - x2
    }
}
