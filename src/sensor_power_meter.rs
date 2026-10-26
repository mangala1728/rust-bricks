// ## Map
// Role: core/sensor_power_meter — 分流電阻電流採樣、庫侖計電量積分與鋰電池 SOC 開路電壓換算微積木。
// Inputs: 分流電壓 (V)、電阻 (歐姆)、取樣間隔 (秒)、單節電池電壓 (V)。
// Outputs: 電流 (A)、累計毫安時 (mAh)、電池剩餘電量百分比 (0..100%)。
// Invariants: 零外部依賴、零 Heap 分配、無遞迴、零全域狀態、控制流巢狀深度 <= 2、除以零防禦。

/// @outputs: 1
/// @domain: v_shunt [-10.0..10.0]
/// @domain: r_shunt [0.0001..1000.0]
pub fn shunt_voltage_to_current_amperes(v_shunt_volts: f32, r_shunt_ohms: f32) -> f32 {
    if v_shunt_volts.is_nan() || r_shunt_ohms.is_nan() || r_shunt_ohms <= 0.0 {
        return 0.0;
    }
    v_shunt_volts / r_shunt_ohms
}

/// @outputs: 1
/// @domain: current_amps [-100.0..100.0]
/// @domain: dt_seconds [0.0001..3600.0]
pub fn coulomb_counter_integrate_mah(current_amps: f32, dt_seconds: f32, accum_mah: &mut f32) {
    if current_amps.is_nan() || dt_seconds.is_nan() || dt_seconds <= 0.0 || accum_mah.is_nan() {
        return;
    }
    // 毫安時增量: delta_mah = (I * dt) / 3.6
    let delta_mah = (current_amps * dt_seconds) / 3.6;
    *accum_mah += delta_mah;
}

const LIPO_LUT: [(f32, f32); 11] = [
    (3.27, 0.0),
    (3.61, 10.0),
    (3.69, 20.0),
    (3.74, 30.0),
    (3.79, 40.0),
    (3.84, 50.0),
    (3.91, 60.0),
    (3.99, 70.0),
    (4.08, 80.0),
    (4.15, 90.0),
    (4.20, 100.0),
];

/// @outputs: 1
/// @domain: cell_voltage [2.5..4.5]
pub fn lipo_voltage_to_soc_percent(cell_voltage: f32) -> f32 {
    if cell_voltage.is_nan() {
        return 0.0;
    }
    if cell_voltage <= LIPO_LUT[0].0 {
        return 0.0;
    }
    let last = LIPO_LUT.len() - 1;
    if cell_voltage >= LIPO_LUT[last].0 {
        return 100.0;
    }
    for i in 0..last {
        let (v_low, soc_low) = LIPO_LUT[i];
        let (v_high, soc_high) = LIPO_LUT[i + 1];
        if cell_voltage >= v_low && cell_voltage <= v_high {
            let frac = (cell_voltage - v_low) / (v_high - v_low);
            return soc_low + frac * (soc_high - soc_low);
        }
    }
    0.0
}
