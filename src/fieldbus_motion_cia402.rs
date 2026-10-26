//! # Industrial Drive Motion Profile (CiA 402 / IEC 61800-7) Micro-Bricks
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 CANopen / EtherCAT CoE CiA 402 驅動器狀態機躍遷、Controlword/Statusword 編解碼、
//! 輪廓位置 (PP) / 輪廓速度 (PV) / 原點回歸 (Homing) / 轉矩模式 (CST) 參數轉換、
//! 跟隨誤差 (Following Error) 越界檢驗、快速停機 (Quick Stop) 減速時間精算。

/// CiA 402 狀態枚舉常數
pub const CIA402_STATE_NOT_READY: u8 = 0;
pub const CIA402_STATE_SWITCH_ON_DISABLED: u8 = 1;
pub const CIA402_STATE_READY_TO_SWITCH_ON: u8 = 2;
pub const CIA402_STATE_SWITCHED_ON: u8 = 3;
pub const CIA402_STATE_OPERATION_ENABLED: u8 = 4;
pub const CIA402_STATE_QUICK_STOP_ACTIVE: u8 = 5;
pub const CIA402_STATE_FAULT_REACTION_ACTIVE: u8 = 6;
pub const CIA402_STATE_FAULT: u8 = 7;

/// ## Map: CiA 402 驅動器狀態機命令躍遷 (Drive State Transition)
/// 依據目前狀態與控制字 (0x6040 Controlword) 的 Bit 0..3 及 Bit 7 判定下一狀態
/// @outputs: 1
/// @domain: current_state [0, 7], controlword [0, 65535]
pub fn cia402_state_machine_transition(current_state: u8, controlword: u16) -> u8 {
    let fault_reset = (controlword & 0x0080) != 0;
    if current_state == CIA402_STATE_FAULT {
        return if fault_reset { CIA402_STATE_SWITCH_ON_DISABLED } else { CIA402_STATE_FAULT };
    }
    let b0 = (controlword & 0x0001) != 0; // Switch On
    let b1 = (controlword & 0x0002) != 0; // Enable Voltage
    let b2 = (controlword & 0x0004) != 0; // Quick Stop
    let b3 = (controlword & 0x0008) != 0; // Enable Operation

    match current_state {
        CIA402_STATE_SWITCH_ON_DISABLED => {
            if b1 && b2 && !b0 { CIA402_STATE_READY_TO_SWITCH_ON } else { CIA402_STATE_SWITCH_ON_DISABLED }
        }
        CIA402_STATE_READY_TO_SWITCH_ON => {
            if !b1 || !b2 { CIA402_STATE_SWITCH_ON_DISABLED }
            else if b0 { CIA402_STATE_SWITCHED_ON }
            else { CIA402_STATE_READY_TO_SWITCH_ON }
        }
        CIA402_STATE_SWITCHED_ON => {
            if !b1 || !b2 { CIA402_STATE_SWITCH_ON_DISABLED }
            else if !b0 { CIA402_STATE_READY_TO_SWITCH_ON }
            else if b3 { CIA402_STATE_OPERATION_ENABLED }
            else { CIA402_STATE_SWITCHED_ON }
        }
        CIA402_STATE_OPERATION_ENABLED => {
            if !b1 { CIA402_STATE_SWITCH_ON_DISABLED }
            else if !b2 { CIA402_STATE_QUICK_STOP_ACTIVE }
            else if !b0 { CIA402_STATE_READY_TO_SWITCH_ON }
            else if !b3 { CIA402_STATE_SWITCHED_ON }
            else { CIA402_STATE_OPERATION_ENABLED }
        }
        CIA402_STATE_QUICK_STOP_ACTIVE => {
            if !b1 { CIA402_STATE_SWITCH_ON_DISABLED }
            else { CIA402_STATE_QUICK_STOP_ACTIVE }
        }
        _ => CIA402_STATE_SWITCH_ON_DISABLED,
    }
}

/// ## Decode: 從 Statusword (0x6041) 遮罩解碼目前驅動器狀態
/// @outputs: 1
/// @domain: statusword [0, 65535]
pub fn cia402_statusword_decode_state(statusword: u16) -> u8 {
    let masked = statusword & 0x006F;
    if (masked & 0x004F) == 0x0000 { CIA402_STATE_NOT_READY }
    else if (masked & 0x004F) == 0x0040 { CIA402_STATE_SWITCH_ON_DISABLED }
    else if (masked & 0x006F) == 0x0021 { CIA402_STATE_READY_TO_SWITCH_ON }
    else if (masked & 0x006F) == 0x0023 { CIA402_STATE_SWITCHED_ON }
    else if (masked & 0x006F) == 0x0027 { CIA402_STATE_OPERATION_ENABLED }
    else if (masked & 0x006F) == 0x0007 { CIA402_STATE_QUICK_STOP_ACTIVE }
    else if (masked & 0x004F) == 0x000F { CIA402_STATE_FAULT_REACTION_ACTIVE }
    else if (masked & 0x004F) == 0x0008 { CIA402_STATE_FAULT }
    else { CIA402_STATE_SWITCH_ON_DISABLED }
}

/// ## Encode: 狀態與標誌封裝為 CiA 402 Statusword (0x6041)
/// @outputs: 1
/// @domain: state [0, 7]
pub fn cia402_statusword_encode(state: u8, voltage_enabled: bool, target_reached: bool) -> u16 {
    let base = match state {
        CIA402_STATE_NOT_READY => 0x0000,
        CIA402_STATE_SWITCH_ON_DISABLED => 0x0040,
        CIA402_STATE_READY_TO_SWITCH_ON => 0x0021,
        CIA402_STATE_SWITCHED_ON => 0x0023,
        CIA402_STATE_OPERATION_ENABLED => 0x0027,
        CIA402_STATE_QUICK_STOP_ACTIVE => 0x0007,
        CIA402_STATE_FAULT_REACTION_ACTIVE => 0x000F,
        CIA402_STATE_FAULT => 0x0008,
        _ => 0x0040,
    };
    let mut word = base;
    if voltage_enabled { word |= 0x0010; }
    if target_reached { word |= 0x0400; }
    word
}

/// ## Map: 常見控制命令碼映射為 Controlword (0x6040)
/// 命令碼：0=Shutdown, 1=SwitchOn, 2=EnableOperation, 3=DisableVoltage, 4=QuickStop, 5=FaultReset
/// @outputs: 1
/// @domain: command_code [0, 5]
pub fn cia402_controlword_from_command(command_code: u8) -> u16 {
    match command_code {
        0 => 0x0006, // Shutdown (Switch On Disabled -> Ready to Switch On)
        1 => 0x0007, // Switch On
        2 => 0x000F, // Enable Operation
        3 => 0x0000, // Disable Voltage
        4 => 0x0002, // Quick Stop
        5 => 0x0080, // Fault Reset
        _ => 0x0000,
    }
}

/// ## Verify: 輪廓位置 (PP) 目標位置邊界有效性防護
/// @outputs: 1
/// @domain: target [i32::MIN, i32::MAX], min_pos [i32::MIN, i32::MAX], max_pos [i32::MIN, i32::MAX]
pub fn cia402_pp_target_position_valid(target: i32, min_pos: i32, max_pos: i32) -> bool {
    if min_pos >= max_pos {
        return false;
    }
    target >= min_pos && target <= max_pos
}

/// ## Convert: 輪廓速度 (PV) 模式 RPM 轉編碼器頻率 (Counts per Second)
/// 公式：cps = (rpm / 60.0) * cpr
/// @outputs: 1
/// @domain: rpm [-100000.0, 100000.0], cpr [1, 100000000]
pub fn cia402_pv_speed_rpm_to_cps(rpm: f64, cpr: u32) -> i32 {
    if !rpm.is_finite() || cpr == 0 {
        return 0;
    }
    let rps = rpm / 60.0;
    let cps = rps * (cpr as f64);
    if cps > (i32::MAX as f64) {
        i32::MAX
    } else if cps < (i32::MIN as f64) {
        i32::MIN
    } else {
        cps.round() as i32
    }
}

/// ## Step: 原點回歸偏移量換算 (Homing Offset Calculation)
/// 公式：actual_zero = current_pos - home_offset
/// @outputs: 1
/// @domain: current_pos [i32::MIN, i32::MAX], home_offset [i32::MIN, i32::MAX]
pub fn cia402_homing_offset_step(current_pos: i32, home_offset: i32) -> i32 {
    current_pos.wrapping_sub(home_offset)
}

/// ## Calc: 快速停機 (Quick Stop) 減速煞車時間預估 (秒)
/// 公式：time_s = |current_speed_rpm| / decel_rpm_per_s
/// @outputs: 1
/// @domain: current_speed_rpm [-100000.0, 100000.0], decel_rpm_per_s [0.001, 1e9]
pub fn cia402_quick_stop_deceleration_time(current_speed_rpm: f64, decel_rpm_per_s: f64) -> f64 {
    if !current_speed_rpm.is_finite() || !decel_rpm_per_s.is_finite() || decel_rpm_per_s <= 0.0 {
        return 0.0;
    }
    current_speed_rpm.abs() / decel_rpm_per_s
}

/// ## Convert: 實體轉矩換算 (千分比轉牛頓米 Nm)
/// 0x6077 Actual Torque 為額定轉矩千分比 (Permille: 1000 = 100% 額定轉矩)
/// @outputs: 1
/// @domain: raw_torque [-3000, 3000], rated_torque_nm [0.0, 10000.0]
pub fn cia402_torque_permille_to_nm(raw_torque: i16, rated_torque_nm: f64) -> f64 {
    if !rated_torque_nm.is_finite() || rated_torque_nm <= 0.0 {
        return 0.0;
    }
    (raw_torque as f64 / 1000.0) * rated_torque_nm
}

/// ## Detect: 跟隨誤差超限判定 (Following Error Limit Detect, 0x6065)
/// 若 |actual_pos - demand_pos| > following_error_window 則回傳 true (觸發保護)
/// @outputs: 1
/// @domain: actual_pos [i32::MIN, i32::MAX], demand_pos [i32::MIN, i32::MAX], window [0, u32::MAX]
pub fn cia402_following_error_detect(actual_pos: i32, demand_pos: i32, window: u32) -> bool {
    let diff = (actual_pos as i64 - demand_pos as i64).abs();
    diff > (window as i64)
}
