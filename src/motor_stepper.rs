// ## Map
// Role: core/motor_stepper — 工控步進馬達梯形加減速時序與正交編碼器 4x 解碼微積木。
// Inputs: 當前步數、總步數、加減速段步數、微秒延遲區間、正交 AB 相電位。
// Outputs: 下一步定時器脈衝延遲 (us)、位置計數器增量、行程換算 (mm)。
// Invariants: 零外部依賴、零 Heap 分配、無遞迴、零全域狀態、控制流巢狀深度 <= 2、除以零防禦。

/// @outputs: 1
/// @domain: current_step [0..1000000]
/// @domain: total_steps [1..1000000]
/// @domain: min_delay_us [1..1000000]
/// @domain: max_delay_us [1..1000000]
pub fn stepper_trapezoid_delay_us(
    current_step: u32,
    total_steps: u32,
    accel_steps: u32,
    decel_steps: u32,
    min_delay_us: u32,
    max_delay_us: u32,
) -> u32 {
    if total_steps == 0 || min_delay_us > max_delay_us || min_delay_us == 0 {
        return max_delay_us;
    }
    if current_step >= total_steps {
        return max_delay_us;
    }
    let range = max_delay_us - min_delay_us;

    // 1. 加速區間
    if accel_steps > 0 && current_step < accel_steps {
        let frac = current_step as u64;
        let delta = ((range as u64) * frac) / (accel_steps as u64);
        return max_delay_us.saturating_sub(delta as u32);
    }

    // 2. 減速區間
    let decel_start = total_steps.saturating_sub(decel_steps);
    if decel_steps > 0 && current_step >= decel_start {
        let remaining = total_steps.saturating_sub(current_step);
        let frac = remaining as u64;
        let delta = ((range as u64) * frac) / (decel_steps as u64);
        return max_delay_us.saturating_sub(delta as u32);
    }

    // 3. 等速巡航區間
    min_delay_us
}

/// @outputs: 1
/// @domain: last_state [0..3]
pub fn quadrature_encoder_update(last_state: u8, a: bool, b: bool, position: &mut i32) -> u8 {
    let curr_state = ((a as u8) << 1) | (b as u8);
    if last_state > 3 {
        return curr_state;
    }
    // 狀態轉移表：00->01(+1), 01->11(+1), 11->10(+1), 10->00(+1) 為正轉
    // 反向為逆轉 (-1)，跳態（如 00->11）為異常抖動忽視
    match (last_state, curr_state) {
        (0b00, 0b01) | (0b01, 0b11) | (0b11, 0b10) | (0b10, 0b00) => {
            *position = position.saturating_add(1);
        }
        (0b01, 0b00) | (0b11, 0b01) | (0b10, 0b11) | (0b00, 0b10) => {
            *position = position.saturating_sub(1);
        }
        _ => {}
    }
    curr_state
}

/// @outputs: 1
/// @domain: steps [0..10000000]
/// @domain: steps_per_rev [1..65536]
/// @domain: pitch_mm [0.0..1000.0]
pub fn stepper_steps_to_distance_mm(steps: u32, steps_per_rev: u32, pitch_mm: f32) -> f32 {
    if steps_per_rev == 0 || pitch_mm.is_nan() || pitch_mm < 0.0 {
        return 0.0;
    }
    let revs = (steps as f32) / (steps_per_rev as f32);
    revs * pitch_mm
}
