// ## Map
// Role: core/fast_math — 邊緣裝置極速倒數平方根、純整數開方與平滑內插快速數學微積木。
// Inputs: 浮點數、無號整數、邊界閾值、內插權重 t。
// Outputs: 快速倒數平方根 (1/sqrt)、整數截斷平方根、線性內插值、Hermite 平滑階躍值。
// Invariants: 零外部依賴、100% 零 unsafe、無遞迴、零全域可變狀態、浮點數防 NaN/Inf 守衛、控制流巢狀深度 <= 2。

/// @outputs: 2
/// @domain: x [0.000001..1000000.0]
pub fn fast_inv_sqrt_f32(x: f32) -> f32 {
    if x < 0.000001 || x > 1000000.0 || x.is_nan() {
        return 0.0;
    }
    let i = x.to_bits();
    let i = 0x5f3759df_u32.wrapping_sub(i >> 1);
    let y = f32::from_bits(i);
    y * (1.5 - (0.5 * x * y * y))
}

/// @outputs: 2
/// @domain: n [0..18446744073709551615]
pub fn isqrt_u64(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    let mut res: u64 = 0;
    let mut bit: u64 = 1 << 62;
    while bit > n {
        bit >>= 2;
    }
    let mut temp = n;
    while bit != 0 {
        if temp >= res + bit {
            temp -= res + bit;
            res = (res >> 1) + bit;
        } else {
            res >>= 1;
        }
        bit >>= 2;
    }
    res
}

/// @outputs: 1
/// @domain: start [-1000000.0..1000000.0]
/// @domain: end [-1000000.0..1000000.0]
/// @domain: t [0.0..1.0]
pub fn lerp_f32(start: f32, end: f32, t: f32) -> f32 {
    if start < -1000000.0 || start > 1000000.0 || end < -1000000.0 || end > 1000000.0 {
        return start;
    }
    if start.is_nan() || end.is_nan() || t.is_nan() {
        return 0.0;
    }
    let clamped_t = t.clamp(0.0, 1.0);
    start + clamped_t * (end - start)
}

/// @outputs: 2
/// @domain: edge0 [-10000.0..10000.0]
/// @domain: edge1 [-10000.0..10000.0]
/// @domain: x [-10000.0..10000.0]
pub fn smoothstep_f32(edge0: f32, edge1: f32, x: f32) -> f32 {
    if edge0 < -10000.0 || edge0 > 10000.0 || edge1 < -10000.0 || edge1 > 10000.0 {
        return 0.0;
    }
    if x < -10000.0 || x > 10000.0 || edge0.is_nan() || edge1.is_nan() || x.is_nan() {
        return 0.0;
    }
    let diff = edge1 - edge0;
    if diff.abs() < 1e-6 {
        return if x >= edge1 { 1.0 } else { 0.0 };
    }
    let t = ((x - edge0) / diff).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
