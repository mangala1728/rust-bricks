// ## Map
// Role: core/sensor_baro_als — 巴羅米特氣壓感測器海拔高度解算與環境光感測器 (OPT3001/BH1750) 照度換算微積木。
// Inputs: 大氣壓強 (hPa)、海平面參考氣壓 (hPa)、光感測器原始暫存器數值。
// Outputs: 海拔高度 (公尺)、環境光照度 (Lux)。
// Invariants: 零外部依賴、零 Heap 分配、無遞迴、零全域狀態、控制流巢狀深度 <= 2、除以零防禦。

/// @outputs: 2
/// @domain: pressure_hpa [300.0..1200.0]
/// @domain: sea_level_hpa [800.0..1200.0]
pub fn baro_pressure_to_altitude_m(pressure_hpa: f32, sea_level_hpa: f32) -> Option<f32> {
    if pressure_hpa.is_nan() || sea_level_hpa.is_nan() || pressure_hpa <= 0.0 || sea_level_hpa <= 0.0 {
        return None;
    }
    let ratio = pressure_hpa / sea_level_hpa;
    if ratio <= 0.0 {
        return None;
    }
    // 國際標準大氣 (ISA) 公式: h = 44330 * (1 - (P / P0)^0.1902949)
    let alt = 44330.0 * (1.0 - ratio.powf(0.1902949));
    Some(alt)
}

/// @outputs: 1
pub fn opt3001_raw_to_lux(raw_reg: u16) -> f32 {
    // 高 4 位元為指數 E (0..11)，低 12 位元為有效數值 R
    let exponent = ((raw_reg >> 12) & 0x0F) as u32;
    let fractional = (raw_reg & 0x0FFF) as u32;
    let lsb_size = 0.01f32 * (1u32 << exponent) as f32;
    (fractional as f32) * lsb_size
}

/// @outputs: 1
/// @domain: mtreg [31..254]
pub fn bh1750_raw_to_lux(raw_count: u16, mtreg: u8) -> f32 {
    if mtreg == 0 {
        return 0.0;
    }
    // BH1750 標竿公式: Lux = (Raw / 1.2) * (69 / MTreg)
    let base_lux = (raw_count as f32) / 1.2;
    let factor = 69.0f32 / (mtreg as f32);
    base_lux * factor
}
