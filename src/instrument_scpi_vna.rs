// ## Map
// Role: t&m/instrument_scpi_vna — 零依賴儀器控制、SCPI 指令解析、六位半電表校準與向量網絡分析儀 (VNA) 射頻阻抗解算微積木。
// Inputs: SCPI 命令字節、六位半讀值陣列、VNA S11 複數反射係數、示波器波形取樣。
// Outputs: SCPI 指令匹配與定點數值、DMM 漂移標準差、Smith 圓圖複數阻抗 (R + jX)、VSWR 與回波損耗、波形頻率 (Hz)。
// Invariants: 零外部依賴、純 Rust 實現、除零剛性防護、有界陣列切片、單一函式 <= 60 行。

/// @outputs: 2
pub fn scpi_parse_command<'a>(cmd: &'a [u8]) -> Option<(&'a [u8], bool)> {
    if cmd.is_empty() || cmd.len() > 1024 {
        return None;
    }
    let mut start = 0;
    while start < cmd.len() && (cmd[start] == b' ' || cmd[start] == b'\t' || cmd[start] == b'\r' || cmd[start] == b'\n') {
        start += 1;
    }
    let mut end = cmd.len();
    while end > start && (cmd[end - 1] == b' ' || cmd[end - 1] == b'\t' || cmd[end - 1] == b'\r' || cmd[end - 1] == b'\n' || cmd[end - 1] == b';') {
        end -= 1;
    }
    if start >= end {
        return None;
    }
    let trimmed = &cmd[start..end];
    let is_query = trimmed.ends_with(b"?");
    let name = if is_query {
        &trimmed[..trimmed.len() - 1]
    } else {
        trimmed
    };
    Some((name, is_query))
}

/// @outputs: 2
/// @domain: val_str.len() [1..64]
pub fn scpi_parse_numeric_value(val_str: &[u8]) -> Option<f64> {
    if val_str.is_empty() || val_str.len() > 64 {
        return None;
    }
    let mut i = 0;
    let mut sign = 1.0;
    if val_str[0] == b'-' {
        sign = -1.0;
        i += 1;
    } else if val_str[0] == b'+' {
        i += 1;
    }
    let mut integer_part = 0.0;
    let mut frac_part = 0.0;
    let mut frac_div = 1.0;
    let mut in_frac = false;
    let mut exp_sign = 1;
    let mut exp_val = 0;
    let mut in_exp = false;

    while i < val_str.len() {
        let b = val_str[i];
        if b == b'.' && !in_frac && !in_exp {
            in_frac = true;
        } else if (b == b'e' || b == b'E') && !in_exp {
            in_exp = true;
            if i + 1 < val_str.len() && val_str[i + 1] == b'-' {
                exp_sign = -1;
                i += 1;
            } else if i + 1 < val_str.len() && val_str[i + 1] == b'+' {
                i += 1;
            }
        } else if b.is_ascii_digit() {
            let digit = (b - b'0') as f64;
            if in_exp {
                exp_val = exp_val * 10 + (digit as i32);
            } else if in_frac {
                frac_div *= 10.0;
                frac_part += digit / frac_div;
            } else {
                integer_part = integer_part * 10.0 + digit;
            }
        } else {
            return None;
        }
        i += 1;
    }
    let base = (integer_part + frac_part) * sign;
    let factor = 10.0f64.powi(exp_val * exp_sign);
    let res = base * factor;
    if res.is_finite() { Some(res) } else { None }
}

/// @outputs: 2
/// @domain: readings.len() [2..65536]
pub fn dmm_reading_stats(readings: &[f64]) -> Option<(f64, f64, f64, f64)> {
    if readings.len() < 2 || readings.len() > 65_536 {
        return None;
    }
    let mut min = readings[0];
    let mut max = readings[0];
    let mut sum = 0.0;
    for &x in readings {
        if !x.is_finite() {
            return None;
        }
        if x < min { min = x; }
        if x > max { max = x; }
        sum += x;
    }
    let mean = sum / (readings.len() as f64);
    let mut var_sum = 0.0;
    for &x in readings {
        let diff = x - mean;
        var_sum += diff * diff;
    }
    let std_dev = (var_sum / ((readings.len() - 1) as f64)).sqrt();
    Some((min, max, mean, std_dev))
}

/// @outputs: 1
/// @domain: db_mag [-200.0..50.0]
/// @domain: phase_deg [-360.0..360.0]
pub fn vna_s11_to_complex(db_mag: f64, phase_deg: f64) -> (f64, f64) {
    if !db_mag.is_finite() || !phase_deg.is_finite() || db_mag > 100.0 || db_mag < -300.0 {
        return (0.0, 0.0);
    }
    let lin_mag = 10.0f64.powf(db_mag / 20.0);
    let rad = phase_deg.to_radians();
    let gamma_r = lin_mag * rad.cos();
    let gamma_i = lin_mag * rad.sin();
    (gamma_r, gamma_i)
}

/// @outputs: 2
/// @domain: z0 [1.0..1000.0]
pub fn vna_reflection_to_impedance(gamma_r: f64, gamma_i: f64, z0: f64) -> Option<(f64, f64)> {
    if !gamma_r.is_finite() || !gamma_i.is_finite() || z0 <= 0.0 || !z0.is_finite() {
        return None;
    }
    let denom = (1.0 - gamma_r) * (1.0 - gamma_r) + gamma_i * gamma_i;
    if denom < 1e-12 {
        return None; // 避免奇異點 (無限大阻抗開路)
    }
    let num_r = 1.0 - gamma_r * gamma_r - gamma_i * gamma_i;
    let num_i = 2.0 * gamma_i;
    let r = z0 * (num_r / denom);
    let x = z0 * (num_i / denom);
    Some((r, x))
}

/// @outputs: 2
/// @domain: gamma_mag [0.0..0.999999]
pub fn vna_calc_vswr_and_return_loss(gamma_mag: f64) -> Option<(f64, f64)> {
    if gamma_mag < 0.0 || gamma_mag >= 1.0 || !gamma_mag.is_finite() {
        return None;
    }
    let vswr = (1.0 + gamma_mag) / (1.0 - gamma_mag);
    let return_loss = if gamma_mag < 1e-6 {
        120.0 // 理想完美匹配極限
    } else {
        -20.0 * gamma_mag.log10()
    };
    Some((vswr, return_loss))
}

/// @outputs: 2
/// @domain: sample_rate [1.0..10000000000.0]
pub fn oscilloscope_calc_frequency_hz(
    samples: &[f32],
    sample_rate: f32,
    threshold_v: f32,
    hysteresis_v: f32,
) -> Option<f32> {
    if samples.len() < 4 || sample_rate <= 0.0 || !sample_rate.is_finite() || hysteresis_v < 0.0 {
        return None;
    }
    let high_th = threshold_v + hysteresis_v;
    let low_th = threshold_v - hysteresis_v;
    let mut state = samples[0] >= high_th;
    let mut edge_indices = Vec::new();

    for (i, &s) in samples.iter().enumerate() {
        if !s.is_finite() {
            return None;
        }
        if !state && s >= high_th {
            state = true;
            edge_indices.push(i);
        } else if state && s <= low_th {
            state = false;
        }
    }
    if edge_indices.len() < 2 {
        return None;
    }
    let first = edge_indices[0];
    let last = edge_indices[edge_indices.len() - 1];
    let total_periods = (edge_indices.len() - 1) as f32;
    let total_samples = (last - first) as f32;
    if total_samples <= 0.0 {
        return None;
    }
    let period_sec = (total_samples / total_periods) / sample_rate;
    if period_sec <= 0.0 {
        return None;
    }
    Some(1.0 / period_sec)
}
