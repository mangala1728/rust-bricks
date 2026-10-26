// ## Map
// Role: core/env_sensor — 恆溫恆濕機與環境監控之 SHT3x CRC8、露點溫度與 AQI 空氣指標微積木。
// Inputs: 感測器位元組切片、攝氏溫度、相對濕度 (0..100%)、PM2.5 濃度 (ug/m3)。
// Outputs: CRC8 校驗值 u8、露點溫度 Option<f32>、空氣品質指標 AQI (0..500)。
// Invariants: 零外部依賴、零 Heap 分配、無遞迴、零全域狀態、Magnus 公式防止對數小於等於零守衛。

/// @outputs: 1
/// @domain: data_len [0..64]
pub fn sht3x_crc8(data: &[u8]) -> u8 {
    if data.len() > 64 {
        return 0;
    }
    let mut crc = 0xFFu8;
    for &b in data {
        crc ^= b;
        for _ in 0..8 {
            if (crc & 0x80) != 0 {
                crc = (crc << 1) ^ 0x31;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

/// @outputs: 2
/// @domain: temp_c [-50.0..100.0]
/// @domain: humidity_rh [0.1..100.0]
pub fn dew_point_magnus(temp_c: f32, humidity_rh: f32) -> Option<f32> {
    if temp_c.is_nan() || humidity_rh.is_nan() {
        return None;
    }
    if humidity_rh <= 0.0 || humidity_rh > 100.0 || temp_c < -50.0 || temp_c > 100.0 {
        return None;
    }
    let a = 17.27f32;
    let b = 237.7f32;
    let rh_ratio = (humidity_rh / 100.0).clamp(0.001, 1.0);
    let alpha = ((a * temp_c) / (b + temp_c)) + rh_ratio.ln();
    let denominator = a - alpha;
    if denominator.abs() < 1e-4 {
        return None;
    }
    let dew = (b * alpha) / denominator;
    Some(dew)
}

/// 美國 EPA PM2.5 24 小時 AQI（2024-05-06 生效之新表，EPA-403/B-26-003 Table 6）。
/// 超過 325.4 µg/m³ 時依 Table 6 註 4 沿用 Hazardous 段公式外推，AQI 可大於 500。
/// @outputs: 1
/// @domain: pm25 [0.0..1000.0]
pub fn aqi_pm25_us_standard(pm25: f32) -> u16 {
    if pm25.is_nan() || pm25 <= 0.0 {
        return 0;
    }
    // 越界收斂至 domain 上限；EPA 規定濃度先截斷至 0.1 µg/m³，使斷點間隙 (如 9.0~9.1) 不會落空
    let pm25 = (pm25.min(1000.0) * 10.0).floor() / 10.0;
    // [(0.0..9.0, 0..50), (9.1..35.4, 51..100), (35.5..55.4, 101..150), (55.5..125.4, 151..200),
    //  (125.5..225.4, 201..300), (225.5..325.4, 301..500)]
    let (c_low, c_high, i_low, i_high) = if pm25 <= 9.0 {
        (0.0, 9.0, 0.0, 50.0)
    } else if pm25 <= 35.4 {
        (9.1, 35.4, 51.0, 100.0)
    } else if pm25 <= 55.4 {
        (35.5, 55.4, 101.0, 150.0)
    } else if pm25 <= 125.4 {
        (55.5, 125.4, 151.0, 200.0)
    } else if pm25 <= 225.4 {
        (125.5, 225.4, 201.0, 300.0)
    } else {
        (225.5, 325.4, 301.0, 500.0)
    };

    let aqi = ((i_high - i_low) / (c_high - c_low)) * (pm25 - c_low) + i_low;
    aqi.round().clamp(0.0, u16::MAX as f32) as u16
}
