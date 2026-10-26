// ## Map
// Role: dsp/dsp_fft_radix2 — 零依賴 Cooley-Tukey Radix-2 快速傅立葉轉換 (FFT/IFFT)、Q15 定點數蝶形運算與位元反轉微積木。
// Inputs: 實部與虛部陣列 (2 的冪次方)、位元反轉長度、Q15 複數取樣點與旋轉因子 (Twiddle Factors)。
// Outputs: 頻域複數頻譜、時域逆轉換信號、定點數蝶形運算結果對。
// Invariants: 零外部依賴、純 Rust 實現、2 的冪次方邊界校驗、蝶形縮放防溢位、單一函式 <= 60 行。

/// @outputs: 1
/// @domain: bits [1..16]
pub fn fft_bit_reverse_u32(val: u32, bits: u32) -> u32 {
    if bits == 0 || bits > 16 {
        return 0;
    }
    let mut rev = 0u32;
    let mut v = val;
    for _ in 0..bits {
        rev = (rev << 1) | (v & 1);
        v >>= 1;
    }
    rev
}

/// @outputs: 2
/// @domain: real.len() [2..4096]
pub fn fft_radix2_f32_inplace(real: &mut [f32], imag: &mut [f32]) -> bool {
    let n = real.len();
    if n < 2 || n > 4096 || imag.len() != n || (n & (n - 1)) != 0 {
        return false;
    }
    let bits = n.trailing_zeros();

    // 1. 位元反轉置換
    for i in 0..n {
        let j = fft_bit_reverse_u32(i as u32, bits) as usize;
        if j > i {
            real.swap(i, j);
            imag.swap(i, j);
        }
    }

    // 2. 蝶形運算階層
    let mut len = 2;
    while len <= n {
        let half = len / 2;
        let angle = -2.0 * std::f32::consts::PI / (len as f32);
        let w_step_re = angle.cos();
        let w_step_im = angle.sin();

        let mut i = 0;
        while i < n {
            let mut w_re = 1.0f32;
            let mut w_im = 0.0f32;
            for j in 0..half {
                let u_re = real[i + j];
                let u_im = imag[i + j];
                let v_re = real[i + j + half] * w_re - imag[i + j + half] * w_im;
                let v_im = real[i + j + half] * w_im + imag[i + j + half] * w_re;

                real[i + j] = u_re + v_re;
                imag[i + j] = u_im + v_im;
                real[i + j + half] = u_re - v_re;
                imag[i + j + half] = u_im - v_im;

                let next_w_re = w_re * w_step_re - w_im * w_step_im;
                let next_w_im = w_re * w_step_im + w_im * w_step_re;
                w_re = next_w_re;
                w_im = next_w_im;
            }
            i += len;
        }
        len <<= 1;
    }
    true
}

/// @outputs: 2
/// @domain: real.len() [2..4096]
pub fn ifft_radix2_f32_inplace(real: &mut [f32], imag: &mut [f32]) -> bool {
    let n = real.len();
    if n < 2 || n > 4096 || imag.len() != n || (n & (n - 1)) != 0 {
        return false;
    }
    for im in imag.iter_mut() {
        *im = -*im;
    }
    if !fft_radix2_f32_inplace(real, imag) {
        return false;
    }
    let inv_n = 1.0 / (n as f32);
    for i in 0..n {
        real[i] *= inv_n;
        imag[i] = -imag[i] * inv_n;
    }
    true
}

/// @outputs: 1
pub fn fft_radix2_q15_step(
    ar: i16,
    ai: i16,
    br: i16,
    bi: i16,
    wr: i16,
    wi: i16,
) -> ((i16, i16), (i16, i16)) {
    // Q15 複數旋轉: T = B * W / 32768
    // T_r = (br * wr - bi * wi) >> 15
    // T_i = (br * wi + bi * wr) >> 15
    let tr = (((br as i32) * (wr as i32) - (bi as i32) * (wi as i32)) >> 15) as i16;
    let ti = (((br as i32) * (wi as i32) + (bi as i32) * (wr as i32)) >> 15) as i16;

    // 階層 1 位元縮放除以 2，杜絕 Q15 蝶形累加溢位
    let out_ar = (((ar as i32) + (tr as i32)) >> 1) as i16;
    let out_ai = (((ai as i32) + (ti as i32)) >> 1) as i16;
    let out_br = (((ar as i32) - (tr as i32)) >> 1) as i16;
    let out_bi = (((ai as i32) - (ti as i32)) >> 1) as i16;

    ((out_ar, out_ai), (out_br, out_bi))
}
