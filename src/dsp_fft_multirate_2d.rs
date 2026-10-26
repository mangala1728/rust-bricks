//! # Multirate DSP, 2D Spectral Filtering & Chirp Z-Transform Micro-Bricks
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 Goertzel 演算法單點頻率能量 (DTMF 偵測)、Chirp Z-Transform (CZT) 局部超解析度變換核、
//! 基-4 (Radix-4) 蝶形運算單元、2D 頻譜中心化 (-1)^(u+v)、2D 高斯低通與高通頻域濾波核、
//! 希爾伯特解析信號瞬時相位解算、相位一階微分瞬時頻率求解步進。

/// ## Map: Goertzel 演算法單點目標頻率能量計算 (Single-Bin Energy / DTMF)
/// 運算複雜度 O(N)，記憶體 O(1)，免除完整 FFT 浪費，適用於單頻檢測與雷達特定都卜勒通道
/// @outputs: 1
/// @domain: target_bin [0, 1000000]
pub fn dsp_goertzel_single_bin_energy(samples: &[f64], target_bin: usize, n: usize) -> f64 {
    if samples.is_empty() || n == 0 || target_bin >= n || samples.len() < n {
        return 0.0;
    }
    const TWO_PI: f64 = 6.283185307179586;
    let omega = (TWO_PI * (target_bin as f64)) / (n as f64);
    let coeff = 2.0 * omega.cos();

    let mut q0: f64;
    let mut q1: f64 = 0.0;
    let mut q2: f64 = 0.0;

    for i in 0..n {
        let x = samples[i];
        if !x.is_finite() { return 0.0; }
        q0 = coeff * q1 - q2 + x;
        q2 = q1;
        q1 = q0;
    }
    let energy = q1 * q1 + q2 * q2 - coeff * q1 * q2;
    if energy < 0.0 || !energy.is_finite() { 0.0 } else { energy }
}

/// ## Step: 基-4 (Radix-4) 蝶形復數運算單步 (Radix-4 Butterfly Unit)
/// 接收 4 個輸入節點與 3 個旋轉因子 (tw1, tw2, tw3)，回傳 4 個輸出節點 (Re, Im)
/// @outputs: 1
pub fn dsp_radix4_butterfly_step(
    a: (f64, f64),
    b: (f64, f64),
    c: (f64, f64),
    d: (f64, f64),
    tw1: (f64, f64),
    tw2: (f64, f64),
    tw3: (f64, f64),
) -> [(f64, f64); 4] {
    // 旋轉因子乘法: (x.0*tw.0 - x.1*tw.1, x.0*tw.1 + x.1*tw.0)
    let b_tw = (b.0 * tw1.0 - b.1 * tw1.1, b.0 * tw1.1 + b.1 * tw1.0);
    let c_tw = (c.0 * tw2.0 - c.1 * tw2.1, c.0 * tw2.1 + c.1 * tw2.0);
    let d_tw = (d.0 * tw3.0 - d.1 * tw3.1, d.0 * tw3.1 + d.1 * tw3.0);

    let t0 = (a.0 + c_tw.0, a.1 + c_tw.1);
    let t1 = (a.0 - c_tw.0, a.1 - c_tw.1);
    let t2 = (b_tw.0 + d_tw.0, b_tw.1 + d_tw.1);
    let t3 = (b_tw.0 - d_tw.0, b_tw.1 - d_tw.1);

    // 乘以 -j: (re, im) * -j = (im, -re)
    let jt3 = (t3.1, -t3.0);

    [
        (t0.0 + t2.0, t0.1 + t2.1),
        (t1.0 + jt3.0, t1.1 + jt3.1),
        (t0.0 - t2.0, t0.1 - t2.1),
        (t1.0 - jt3.0, t1.1 - jt3.1),
    ]
}

/// ## Map: Chirp Z-Transform (CZT) 藍寶石調頻旋轉權重 (Chirp Kernel)
/// 公式：W^{k^2 / 2} = exp(-j * pi * k^2 / M)
/// @outputs: 1
/// @domain: k [0, 1000000], m [1, 1000000]
pub fn dsp_czt_chirp_weight(k: usize, m: usize) -> (f64, f64) {
    if m == 0 {
        return (1.0, 0.0);
    }
    const PI: f64 = 3.141592653589793;
    let k_f = k as f64;
    let angle = -PI * (k_f * k_f) / (m as f64);
    (angle.cos(), angle.sin())
}

/// ## Step: 2D 頻譜中心化象限搬移調變因子 (2D Spectrum Shift / FFT-Shift)
/// 公式：factor = (-1)^(u + v)，用於在空間域預乘實現頻譜正中央零頻點對齊
/// @outputs: 1
pub fn dsp_2d_spectrum_center_shift_factor(u: usize, v: usize) -> f64 {
    if ((u + v) & 1) != 0 { -1.0 } else { 1.0 }
}

/// ## Calc: 2D 頻域高斯低通濾波核 (2D Gaussian Low-Pass Filter Kernel)
/// 公式：H(u, v) = exp(-D(u, v)^2 / (2 * D0^2))
/// 其中 D(u, v) 為至頻譜中心 (M/2, N/2) 之尤拉距離，D0 為截止半徑
/// @outputs: 1
/// @domain: cutoff_d0 [1e-3, 1e6]
pub fn dsp_2d_gaussian_lowpass_filter_kernel(
    u: usize,
    v: usize,
    width: usize,
    height: usize,
    cutoff_d0: f64,
) -> f64 {
    if width == 0 || height == 0 || !cutoff_d0.is_finite() || cutoff_d0 <= 0.0 {
        return 0.0;
    }
    let u_center = (width as f64) / 2.0;
    let v_center = (height as f64) / 2.0;
    let du = (u as f64) - u_center;
    let dv = (v as f64) - v_center;
    let d_sq = du * du + dv * dv;

    (-d_sq / (2.0 * cutoff_d0 * cutoff_d0)).exp()
}

/// ## Calc: 2D 頻域高斯高通濾波核 (2D Gaussian High-Pass Filter Kernel)
/// 公式：H_hp(u, v) = 1.0 - H_lp(u, v)
/// @outputs: 1
/// @domain: cutoff_d0 [1e-3, 1e6]
pub fn dsp_2d_gaussian_highpass_filter_kernel(
    u: usize,
    v: usize,
    width: usize,
    height: usize,
    cutoff_d0: f64,
) -> f64 {
    let lp = dsp_2d_gaussian_lowpass_filter_kernel(u, v, width, height, cutoff_d0);
    (1.0 - lp).clamp(0.0, 1.0)
}

/// ## Calc: 希爾伯特解析信號瞬時相位 (Instantaneous Phase)
/// 公式：phase = atan2(q_sample, i_sample)，回傳 [-pi, pi] 弧度
/// @outputs: 1
/// @domain: i_sample [-1e9, 1e9], q_sample [-1e9, 1e9]
pub fn dsp_analytic_signal_instantaneous_phase(i_sample: f64, q_sample: f64) -> f64 {
    if !i_sample.is_finite() || !q_sample.is_finite() {
        return 0.0;
    }
    q_sample.atan2(i_sample)
}

/// ## Step: 相位一階微分瞬時頻率求解單步 (Instantaneous Frequency Step)
/// 內建 [-pi, pi] 相位解纏繞 (Phase Unwrap)，公式：f = (unwrap(delta_phase) * fs) / (2 * pi)
/// @outputs: 1
/// @domain: phase_cur [-100.0, 100.0], phase_prev [-100.0, 100.0], sample_rate [1.0, 1e9]
pub fn dsp_instantaneous_frequency_step(
    phase_cur: f64,
    phase_prev: f64,
    sample_rate: f64,
) -> f64 {
    if !phase_cur.is_finite() || !phase_prev.is_prev_finite() || sample_rate <= 0.0 {
        return 0.0;
    }
    let mut delta = phase_cur - phase_prev;
    const PI: f64 = 3.141592653589793;
    const TWO_PI: f64 = 6.283185307179586;

    while delta > PI { delta -= TWO_PI; }
    while delta < -PI { delta += TWO_PI; }

    (delta * sample_rate) / TWO_PI
}

trait FloatExt {
    fn is_prev_finite(&self) -> bool;
}
impl FloatExt for f64 {
    fn is_prev_finite(&self) -> bool { self.is_finite() }
}
