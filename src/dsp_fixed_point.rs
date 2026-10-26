// ## Map
// Role: core/dsp_fixed_point — 嵌入式與無 FPU MCU 之 Q15 定點數 DSP 濾波與數值微積木。
// Inputs: 浮點數、Q15 整數 (i16)、濾波係數陣列、時域歷史取樣陣列。
// Outputs: Q15 乘積、FIR 單步卷積輸出、轉換浮點數。
// Invariants: 零外部依賴、零 Heap 分配、無遞迴、零全域狀態、控制流巢狀深度 <= 2、乘法飽和防溢位。

/// @outputs: 1
/// @domain: val [-1000.0..1000.0]
pub fn q15_from_f32(val: f32) -> i16 {
    if val.is_nan() {
        return 0;
    }
    let clamped = val.clamp(-1.0, 0.9999695);
    (clamped * 32768.0).round() as i16
}

/// @outputs: 1
/// @domain: val [-32768..32767]
pub fn q15_to_f32(val: i16) -> f32 {
    val as f32 / 32768.0
}

/// @outputs: 1
/// @domain: a [-32768..32767]
/// @domain: b [-32768..32767]
pub fn q15_multiply(a: i16, b: i16) -> i16 {
    // 邊界極端值：-1.0 * -1.0 = +1.0 (超出 Q15 最大正數 32767，必須飽和)
    if a == -32768 && b == -32768 {
        return 32767;
    }
    let prod = (a as i32) * (b as i32);
    // 加上 0x4000 (即 16384) 進行對稱四捨五入，隨後右移 15 位
    let rounded = prod + 0x4000;
    (rounded >> 15) as i16
}

/// @outputs: 1
/// @domain: coeffs_len [1..128]
pub fn q15_fir_filter_step(coeffs: &[i16], history: &[i16]) -> i16 {
    if coeffs.is_empty() || coeffs.len() != history.len() || coeffs.len() > 128 {
        return 0;
    }
    let mut acc: i64 = 0;
    let n = coeffs.len();
    for i in 0..n {
        let c = coeffs[i] as i64;
        let x = history[i] as i64;
        acc += c * x;
    }
    // 右移 15 位以維持 Q15 比例
    let shifted = (acc + 0x4000) >> 15;
    if shifted > 32767 {
        32767
    } else if shifted < -32768 {
        -32768
    } else {
        shifted as i16
    }
}

/// @outputs: 1
/// @domain: buffer_len [2..2048]
pub fn q15_apply_gain(buffer: &mut [i16], gain_q15: i16) {
    if buffer.is_empty() || buffer.len() > 2048 {
        return;
    }
    for sample in buffer.iter_mut() {
        *sample = q15_multiply(*sample, gain_q15);
    }
}
