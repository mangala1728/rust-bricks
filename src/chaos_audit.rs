// ## Map
// Role: core/chaos_audit — 零信任品質總監黑箱動態巡檢引擎 (Zero-Trust Chaos Auditor)。
// Inputs: 倉庫全模組公開函式、惡意與極限邊界測試案例集。
// Outputs: 巡檢抽查報告 Result<AuditSummary, Vec<String>>。
// Invariants: 零外部依賴、無遞迴、零全域可變狀態、黑箱使用者視角、控制流巢狀深度 <= 2、單一函式 <= 60 行。

use crate::{
    agent_terminal, audio_dsp, audio_vad, ble_adv_parser, can_bus_frame, clock_pll_rtc,
    comm_controller, crypto_dukpt, crypto_kdf, dsp_fixed_point, env_sensor, fast_math,
    geo_collision, iso7816_smartcard, math_time, motion_dead_reckon, motor_stepper,
    sensor_hardware, sensor_motion, spatial_physics, sys_path, thermal_palette, thermal_pid,
};

pub struct AuditSummary {
    pub total_inspections: usize,
    pub passed_inspections: usize,
    pub failed_inspections: usize,
}

/// @outputs: 1
fn audit_math_and_geometry() -> bool {
    let mut ok = true;
    ok &= math_time::clamp_i64(50, 100, 0) == 50;
    ok &= (fast_math::fast_inv_sqrt_f32(f32::NAN) - 0.0).abs() < 1e-6;
    ok &= (fast_math::fast_inv_sqrt_f32(-10.0) - 0.0).abs() < 1e-6;
    ok &= fast_math::isqrt_u64(0) == 0;
    let box_nan = geo_collision::bounding_box_overlap((f32::NAN, 0.0), (1.0, 1.0), (0.0, 0.0), (1.0, 1.0));
    ok &= !box_nan;
    let seg_parallel = geo_collision::segment_intersection_2d((0.0, 0.0), (1.0, 1.0), (0.0, 0.0), (1.0, 1.0));
    ok &= seg_parallel.is_none();
    let q_nan = spatial_physics::quaternion_multiply((f32::NAN, 0.0, 0.0, 0.0), (1.0, 0.0, 0.0, 0.0));
    ok &= q_nan.0 == 1.0;
    let ray_neg_radius = spatial_physics::ray_sphere_intersect((0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (2.0, 0.0, 0.0), -5.0);
    ok &= ray_neg_radius.is_none();
    ok
}

/// @outputs: 1
fn audit_audio_and_sensors() -> bool {
    let mut ok = true;
    ok &= audio_vad::pcm16_rms_energy(&[]) == 0;
    ok &= audio_dsp::pcm16_frame_power(&[]) == 0.0;
    ok &= audio_vad::cps_chars_per_second(10, 0).is_none();
    ok &= sensor_motion::haversine_distance_m(999.0, 0.0, 0.0, 0.0).is_none();
    ok &= (sensor_motion::complementary_filter(f32::NAN, 0.0, 10.0, 0.1, 0.98) - 0.0).abs() < 1e-6;
    ok &= !motion_dead_reckon::zupt_zero_velocity_detect(-1.0, 0.0, 0.1, 0.1);
    let (vel, pos) = motion_dead_reckon::dead_reckon_1d_step(f32::NAN, 0.1, false, 0.0, 0.0);
    ok &= vel == 0.0 && pos == 0.0;
    ok &= !sensor_hardware::hall_magnetic_cover_state(50.0, 10.0, 20.0, false);
    ok
}

/// @outputs: 1
fn audit_thermal_and_dsp() -> bool {
    let mut ok = true;
    let rgb_nan = thermal_palette::thermal_ironbow_rgb(f32::NAN, 20.0, 100.0);
    ok &= rgb_nan == (0, 0, 0);
    let rgb_inv = thermal_palette::thermal_rainbow_rgb(50.0, 100.0, 20.0);
    ok &= rgb_inv == (0, 0, 255);
    let empty_stats = thermal_palette::thermal_matrix_stats(&[]);
    ok &= empty_stats.is_none();
    let nan_stats = thermal_palette::thermal_matrix_stats(&[f32::NAN, f32::NAN]);
    ok &= nan_stats.is_none();
    let sat_mul = dsp_fixed_point::q15_multiply(-32768, -32768);
    ok &= sat_mul == 32767;
    let fir_empty = dsp_fixed_point::q15_fir_filter_step(&[], &[]);
    ok &= fir_empty == 0;
    let fir_mismatch = dsp_fixed_point::q15_fir_filter_step(&[100, 200], &[100]);
    ok &= fir_mismatch == 0;
    ok
}

/// @outputs: 1
fn audit_security_and_firmware() -> bool {
    let mut ok = true;
    ok &= !iso7816_smartcard::luhn_checksum_validate(b"");
    ok &= !iso7816_smartcard::luhn_checksum_validate(b"4992739871A");
    let mut pb = [0u8; 8];
    ok &= !iso7816_smartcard::iso9564_pin_block_format0(b"1234", b"12345", &mut pb);
    let truncated_tlv = [0x9F, 0x02, 0x82, 0x01];
    ok &= iso7816_smartcard::ber_tlv_parse_tag_len(&truncated_tlv).is_none();
    ok &= can_bus_frame::can_fd_dlc_to_len(255) == 0;
    ok &= can_bus_frame::timer_calc_prescaler_and_period(0, 1000).is_none();
    ok &= can_bus_frame::timer_calc_prescaler_and_period(72000000, 0).is_none();
    let mut sf = [0u8; 8];
    ok &= !can_bus_frame::iso_tp_single_frame_encode(&[], &mut sf);
    ok
}

/// @outputs: 1
fn audit_ble_and_crypto() -> bool {
    let mut ok = true;
    let malformed_ble = [0x05, 0x09, b'H', b'i'];
    ok &= ble_adv_parser::ble_adv_find_ad_type(&malformed_ble, 0x09).is_none();
    ok &= ble_adv_parser::ble_adv_find_ad_type(&[], 0x09).is_none();
    ok &= ble_adv_parser::ble_adv_parse_tx_power(&[]).is_none();
    ok &= ble_adv_parser::ble_adv_parse_device_name(&[]).is_none();
    ok &= crypto_kdf::pkcs7_unpad(&[], 16).is_none();
    let corrupt_pad = [0x01, 0x02, 0x03, 0x02];
    ok &= crypto_kdf::pkcs7_unpad(&corrupt_pad, 4).is_none();
    let mut stretch_out = [0u8; 16];
    ok &= !crypto_kdf::pbkdf_xor_stretch(&[], b"salt", 100, &mut stretch_out);
    let mut k_out = [0u8; 16];
    ok &= !crypto_kdf::simple_kdf_expand(&[], b"ctx", &mut k_out);
    ok
}

/// @outputs: 1
fn audit_motor_env_and_dukpt() -> bool {
    let mut ok = true;
    let d_zero = motor_stepper::stepper_trapezoid_delay_us(10, 0, 5, 5, 100, 500);
    ok &= d_zero == 500;
    let d_inv = motor_stepper::stepper_trapezoid_delay_us(10, 100, 5, 5, 800, 200);
    ok &= d_inv == 200;
    let dist_nan = motor_stepper::stepper_steps_to_distance_mm(100, 0, f32::NAN);
    ok &= dist_nan == 0.0;
    let mut ksn_max = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x1F, 0xFF, 0xFF];
    ok &= !crypto_dukpt::dukpt_ksn_increment_counter(&mut ksn_max);
    ok &= env_sensor::dew_point_magnus(25.0, -10.0).is_none();
    ok &= env_sensor::dew_point_magnus(f32::NAN, 50.0).is_none();
    ok &= env_sensor::aqi_pm25_us_standard(-5.0) == 0;
    ok
}

/// @outputs: 1
fn audit_comm_and_clock() -> bool {
    let mut ok = true;
    // 1. I2C 越界地址
    ok &= comm_controller::i2c_address_frame(200, false) == 0x00;
    // 2. QSPI 越界 Dummy
    let mut q_buf = [0u8; 8];
    ok &= comm_controller::qspi_flash_fast_read_cmd(0x1000_0000, 8, &mut q_buf) == 0;
    ok &= comm_controller::qspi_flash_fast_read_cmd(0x1000, 30, &mut q_buf) == 0;
    // 3. UART 零波特率與非標準超取樣
    ok &= comm_controller::uart_baud_rate_divisor(16000000, 0, 16).is_none();
    ok &= comm_controller::uart_baud_rate_divisor(16000000, 115200, 4).is_none();
    // 4. DMA 對齊非 2 的冪次
    ok &= comm_controller::dma_align_padding_bytes(13, 0) == 0;
    ok &= comm_controller::dma_align_padding_bytes(13, 7) == 0;
    // 5. PLL 異常輸入
    ok &= clock_pll_rtc::pll_calculate_prescalers(0, 72000000).is_none();
    // 6. NTP 極端時戳
    ok &= clock_pll_rtc::ntp_timestamp_to_unix_secs(1000) == 0;
    ok
}

/// @outputs: 1
fn audit_agent_and_system() -> bool {
    let mut ok = true;
    ok &= !sys_path::is_safe_rel_path("../../../etc/shadow");
    ok &= !sys_path::is_safe_rel_path("C:\\Windows\\System32");
    ok &= !sys_path::is_safe_rel_path("");
    let bs_bomb = vec![0x08u8; 50];
    let mut pty_out = [0u8; 16];
    let pty_len = agent_terminal::pty_line_buffer_edit(&bs_bomb, &mut pty_out);
    ok &= pty_len == 0;
    let incomplete_ansi = "\x1b[31";
    let mut clean_buf = [0u8; 16];
    let n = agent_terminal::strip_ansi_codes(incomplete_ansi, &mut clean_buf);
    ok &= n == 0;
    ok &= thermal_pid::ntc_resistance_to_celsius(-100.0, 10000.0, 3950.0, 25.0).is_none();
    ok &= thermal_pid::ntc_resistance_to_celsius(10000.0, 10000.0, 3950.0, -300.0).is_none();
    ok
}

/// @outputs: 2
/// @domain: dummy_level [0..10]
pub fn run_chaos_audit(dummy_level: u8) -> Result<AuditSummary, Vec<String>> {
    if dummy_level > 10 {
        return Err(vec!["巡檢等級超出有界範圍".to_string()]);
    }
    let mut failures = Vec::new();
    let mut total = 0usize;
    let mut passed = 0usize;

    total += 1;
    if audit_math_and_geometry() { passed += 1; } else { failures.push("【數值幾何黑箱抽查失敗】存在未防護之邊界發散".to_string()); }

    total += 1;
    if audit_audio_and_sensors() { passed += 1; } else { failures.push("【音訊感測器黑箱抽查失敗】存在畸形輸入崩潰".to_string()); }

    total += 1;
    if audit_thermal_and_dsp() { passed += 1; } else { failures.push("【熱成像DSP黑箱抽查失敗】存在溢位或倒置崩潰".to_string()); }

    total += 1;
    if audit_security_and_firmware() { passed += 1; } else { failures.push("【金融韌體黑箱抽查失敗】存在安全性或長度溢出".to_string()); }

    total += 1;
    if audit_ble_and_crypto() { passed += 1; } else { failures.push("【藍芽密碼黑箱抽查失敗】存在Padding或截斷漏洞".to_string()); }

    total += 1;
    if audit_motor_env_and_dukpt() { passed += 1; } else { failures.push("【馬達環境DUKPT黑箱抽查失敗】存在溢出或除零崩潰".to_string()); }

    total += 1;
    if audit_comm_and_clock() { passed += 1; } else { failures.push("【通信時脈RTC黑箱抽查失敗】存在除數或對齊崩潰".to_string()); }

    total += 1;
    if audit_agent_and_system() { passed += 1; } else { failures.push("【代理馬具終端抽查失敗】存在路徑穿越或退格下溢".to_string()); }

    if !failures.is_empty() {
        return Err(failures);
    }
    Ok(AuditSummary {
        total_inspections: total,
        passed_inspections: passed,
        failed_inspections: 0,
    })
}
