// ## Map
// Role: core/chaos_audit — 零信任品質總監黑箱動態巡檢引擎 (Zero-Trust Chaos Auditor)。
// Inputs: 倉庫全模組公開函式、惡意與極限邊界測試案例集。
// Outputs: 巡檢抽查報告 Result<AuditSummary, Vec<String>>。
// Invariants: 零外部依賴、無遞迴、零全域可變狀態、黑箱使用者視角、控制流巢狀深度 <= 2、單一函式 <= 60 行。

use crate::{
    agent_terminal, ai_autograd_train, ai_rl_policy_gradient, ai_tensor_gemm, ai_tensor_train, ai_transformer_attention, analyzer_sigrok_protocol, audio_beamforming_doa, audio_dsp, audio_mfcc_pitch, audio_spectral_codec, audio_vad, ble_adv_parser, cad_geom_model, can_bus_frame, can_lin_bus,
    clock_pll_rtc, cnc_gcode_toolpath, codec_container_multimedia, codec_lossless_compression, comm_controller, comm_phy_bus, control_systems_estimation, crypto_dukpt, crypto_kdf, crypto_modern_wire, curve_fit_trend, decision_mcts, decision_minimax_alphabeta, doc_layout_typography, doc_office_drawing_shape, dsp_fft_multirate_2d, dsp_fft_radix2,
    dsp_fixed_point, env_sensor, fast_math, fieldbus_building_automation, fieldbus_industrial_eth, fieldbus_motion_cia402, fin_apdu_track, geo_collision, geo_spatial_projection, gnss_nmea_geodesy, instrument_industrial, instrument_lxi_hislip, instrument_scpi_vna,
    iot_wire_protocol, iso7816_smartcard, math_time, motion_dead_reckon, motion_render_timeline, motor_stepper,
    net_ethernet, net_tsn_macsec, nfc_iso14443_ndef, numerical_optimization_gradient, office_calc_layout, pcb_cad_fem, physics_gjk_epa_contact, physics_raycast_3d, quant_finance_market, quant_orderbook_l2,
    radar_aoa_mti, radar_biomed_dsp, radar_cfar_doppler, radar_clustering_dbscan_tracker, radar_mimo_tracking, robotics_6dof_slerp, robotics_dynamics_jacobian, robotics_motion_profile, rpa_desktop_macro_driver, rpa_mobile_touch_macro, rpa_screen_control, sensor_baro_als, sensor_hardware, sensor_motion,
    sensor_power_meter, spatial_physics, spc_msa_advanced, spc_six_sigma_msa, storage_block_geometry, sys_path, thermal_palette, thermal_pid,
    vision_camera_calibration_pnp, vision_morphology_fast, vision_optical_flow_homography,
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
    ok &= comm_controller::i2c_address_frame(200, false) == 0x00;
    let mut q_buf = [0u8; 8];
    ok &= comm_controller::qspi_flash_fast_read_cmd(0x1000_0000, 8, &mut q_buf) == 0;
    ok &= comm_controller::qspi_flash_fast_read_cmd(0x1000, 30, &mut q_buf) == 0;
    ok &= comm_controller::uart_baud_rate_divisor(16000000, 0, 16).is_none();
    ok &= comm_controller::uart_baud_rate_divisor(16000000, 115200, 4).is_none();
    ok &= comm_controller::dma_align_padding_bytes(13, 0) == 0;
    ok &= comm_controller::dma_align_padding_bytes(13, 7) == 0;
    ok &= clock_pll_rtc::pll_calculate_prescalers(0, 72000000).is_none();
    ok &= clock_pll_rtc::ntp_timestamp_to_unix_secs(1000) == 0;
    ok
}

/// @outputs: 1
fn audit_ethernet_and_apdu() -> bool {
    let mut ok = true;
    let short_eth = [0u8; 10];
    ok &= net_ethernet::parse_ethernet_frame_header(&short_eth).is_none();
    let mut vlan_pkt = [0u8; 16];
    vlan_pkt[12] = 0x81;
    vlan_pkt[13] = 0x00;
    ok &= net_ethernet::parse_ethernet_frame_header(&vlan_pkt).is_none();
    ok &= fin_apdu_track::apdu_parse_response(&[0x90]).is_none();
    ok &= fin_apdu_track::track2_parse_pan_and_exp("").is_none();
    ok &= fin_apdu_track::track2_parse_pan_and_exp(";123=45?").is_none();
    ok
}

/// @outputs: 1
fn audit_power_and_baro() -> bool {
    let mut ok = true;
    ok &= sensor_power_meter::shunt_voltage_to_current_amperes(1.0, -0.01) == 0.0;
    ok &= sensor_power_meter::lipo_voltage_to_soc_percent(f32::NAN) == 0.0;
    ok &= sensor_baro_als::baro_pressure_to_altitude_m(-100.0, 1013.25).is_none();
    ok &= sensor_baro_als::baro_pressure_to_altitude_m(1000.0, 0.0).is_none();
    ok &= sensor_baro_als::bh1750_raw_to_lux(100, 0) == 0.0;
    ok
}

/// @outputs: 1
fn audit_lin_and_raycast_3d() -> bool {
    let mut ok = true;
    ok &= can_lin_bus::lin_checksum(None, &[]) == 0;
    let hit_parallel = physics_raycast_3d::ray_triangle_intersect_moller_trumbore(
        (0.0, 0.0, 0.0), (0.0, 1.0, 0.0),
        (0.0, 0.0, 1.0), (1.0, 0.0, 1.0), (0.0, 0.0, 2.0),
    );
    ok &= hit_parallel.is_none();
    let hit_nan = physics_raycast_3d::ray_triangle_intersect_moller_trumbore(
        (f32::NAN, 0.0, 0.0), (0.0, 0.0, 1.0),
        (0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0),
    );
    ok &= hit_nan.is_none();
    ok
}

/// @outputs: 1
fn audit_tsn_and_macsec() -> bool {
    let mut ok = true;
    // 1. PTP 畸形截斷與時間倒流
    let short_ptp = [0u8; 30];
    ok &= net_tsn_macsec::ptp_parse_header(&short_ptp).is_none();
    ok &= net_tsn_macsec::ptp_pdelay_mean_path_delay_ns(1000, 500, 200, 100, 0.0).is_none();
    // 2. MACsec 截斷 SecTAG 與零 PN 攻擊
    let short_tag = [0u8; 6];
    ok &= net_tsn_macsec::macsec_parse_sectag(&short_tag).is_none();
    ok &= !net_tsn_macsec::macsec_replay_window_validate(100, 0, 32);
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

/// @outputs: 1
fn audit_ai_tensor_train() -> bool {
    let mut ok = true;
    let mut out_probs = [0.0f32; 3];
    ok &= !ai_tensor_train::softmax_stable(&[], &mut []);
    ok &= !ai_tensor_train::softmax_stable(&[1.0, f32::NAN, 3.0], &mut out_probs);
    ok &= !ai_tensor_train::softmax_stable(&[1.0, 2.0], &mut out_probs);
    let mut ln_out = [0.0f32; 2];
    ok &= !ai_tensor_train::layer_norm(&[1.0, 2.0], &[1.0, 1.0], &[0.0, 0.0], -1e-5, &mut ln_out);
    ok &= !ai_tensor_train::layer_norm(&[f32::NAN, 2.0], &[1.0, 1.0], &[0.0, 0.0], 1e-5, &mut ln_out);
    ok &= !ai_tensor_train::rmsnorm_step(&[], &[], 1e-5, &mut []);
    let rope_nan = ai_tensor_train::rope_rotary_embedding_step((f32::NAN, 1.0), 0.5);
    ok &= rope_nan == (0.0, 0.0);
    ok &= ai_tensor_train::rope_frequency_theta(10, 100, 64, 10000.0) == 0.0;
    let mut p = 1.0f32;
    let mut m = 0.0f32;
    let mut v = 0.0f32;
    ok &= !ai_tensor_train::adamw_step(&mut p, 0.1, &mut m, &mut v, -0.01, 0.9, 0.999, 0.01, 1e-8, 1);
    ok &= !ai_tensor_train::adamw_step(&mut p, 0.1, &mut m, &mut v, 0.001, 0.9, 0.999, 0.01, 1e-8, 0);
    let mut grad = [0.0f32; 2];
    ok &= ai_tensor_train::cross_entropy_loss_and_grad(&[1.0, 2.0], 99, &mut grad).is_none();
    ok &= (ai_tensor_train::gelu_forward(f32::NAN) - 0.0).abs() < 1e-6;
    ok
}

/// @outputs: 1
fn audit_instrument_scpi_vna() -> bool {
    let mut ok = true;
    ok &= instrument_scpi_vna::scpi_parse_command(b"").is_none();
    ok &= instrument_scpi_vna::scpi_parse_numeric_value(b"").is_none();
    ok &= instrument_scpi_vna::scpi_parse_numeric_value(b"12.34.56").is_none();
    ok &= instrument_scpi_vna::dmm_reading_stats(&[]).is_none();
    ok &= instrument_scpi_vna::dmm_reading_stats(&[1.0]).is_none();
    ok &= instrument_scpi_vna::dmm_reading_stats(&[1.0, f64::NAN]).is_none();
    let (gr, gi) = instrument_scpi_vna::vna_s11_to_complex(f64::NAN, 0.0);
    ok &= gr == 0.0 && gi == 0.0;
    ok &= instrument_scpi_vna::vna_reflection_to_impedance(1.0, 0.0, 50.0).is_none();
    ok &= instrument_scpi_vna::vna_calc_vswr_and_return_loss(-1.0).is_none();
    ok &= instrument_scpi_vna::vna_calc_vswr_and_return_loss(1.5).is_none();
    ok &= instrument_scpi_vna::oscilloscope_calc_frequency_hz(&[1.0, 2.0], 1000.0, 0.0, 0.1).is_none();
    ok
}

/// @outputs: 1
fn audit_pcb_cad_fem() -> bool {
    let mut ok = true;
    ok &= pcb_cad_fem::microstrip_impedance_hammerstad(-1.0, 1.6, 4.4).is_none();
    ok &= pcb_cad_fem::microstrip_impedance_hammerstad(1.0, 0.0, 4.4).is_none();
    ok &= pcb_cad_fem::microstrip_impedance_hammerstad(1.0, 1.6, 0.5).is_none();
    ok &= pcb_cad_fem::openems_fdtd_cfl_max_timestep_sec(0.0, 1e-3, 1e-3).is_none();
    ok &= pcb_cad_fem::gerber_coord_to_microns(b"", 4).is_none();
    ok &= pcb_cad_fem::gerber_coord_to_microns(b"X123", 4).is_none();
    ok &= pcb_cad_fem::pcb_rf_mitered_bend_fraction(-1.0, 1.0).is_none();
    ok &= pcb_cad_fem::polygon_shoelace_area_2d(&[(0.0, 0.0), (1.0, 1.0)]).is_none();
    ok
}

/// @outputs: 1
fn audit_motion_render_timeline() -> bool {
    let mut ok = true;
    ok &= motion_render_timeline::remotion_frame_to_ms(100, 0, 1) == 0;
    ok &= motion_render_timeline::remotion_ms_to_frame(1000, 30, 0) == 0;
    let mut tc = [0u8; 11];
    ok &= !motion_render_timeline::smpte_timecode_format(100, 0, &mut tc);
    ok &= motion_render_timeline::css_cubic_bezier_solve(0.25, 0.1, 0.25, 1.0, f64::NAN) == 0.0;
    ok &= motion_render_timeline::svg_stroke_dashoffset(-100.0, 0.5) == 0.0;
    ok
}

/// @outputs: 1
fn audit_office_calc_layout() -> bool {
    let mut ok = true;
    ok &= office_calc_layout::excel_xlookup_f64(1.0, &[], &[], true).is_none();
    ok &= office_calc_layout::excel_pmt_loan(0.05, 0, 1000.0).is_none();
    ok &= office_calc_layout::excel_pmt_loan(0.05, 12, -100.0).is_none();
    ok &= office_calc_layout::excel_npv_discounted(-1.5, &[100.0]).is_none();
    ok &= office_calc_layout::excel_percentile_linear(&[], 0.5).is_none();
    ok &= office_calc_layout::word_greedy_wrap_line_count(&[], 100, 5) == 0;
    let mut out_pos = [0.0f32; 2];
    ok &= !office_calc_layout::ppt_distribute_elements_1d(&[100.0, 200.0], 50.0, &mut out_pos);
    ok
}

/// @outputs: 1
fn audit_rpa_screen_control() -> bool {
    let mut ok = true;
    ok &= rpa_screen_control::rpa_screen_find_color_patch(&[], 0, 0, (255, 0, 0), 10).is_none();
    let pt_nan = rpa_screen_control::rpa_mouse_human_trajectory_point((0.0, 0.0), (100.0, 100.0), (50.0, 50.0), f32::NAN);
    ok &= pt_nan == (0.0, 0.0);
    ok &= !rpa_screen_control::rpa_click_double_click_detect(1000, 500, 300);
    ok &= rpa_screen_control::rs485_calc_char_tx_time_us(0, 8, 1, false) == 0;
    ok &= rpa_screen_control::gpib_ieee488_handshake_step(false, false, false) == 0;
    ok
}

/// @outputs: 1
fn audit_radar_cfar_doppler() -> bool {
    let mut ok = true;
    let mut det = [false; 4];
    ok &= radar_cfar_doppler::radar_ca_cfar_detect_1d(&[], 1, 2, 3.0, &mut det) == 0;
    ok &= radar_cfar_doppler::radar_doppler_velocity_mps(100.0, 0.0).is_none();
    ok &= radar_cfar_doppler::radar_micro_doppler_chest_displacement_mm(0.1, -60.0).is_none();
    let mut clusters = [0usize; 2];
    ok &= radar_cfar_doppler::radar_cluster_points_1d(&[], 1.0, &mut clusters) == 0;
    ok &= radar_cfar_doppler::radar_mimo_virtual_array_positions(&[], &[], &mut []) == 0;
    ok
}

/// @outputs: 1
fn audit_dsp_fft_radix2() -> bool {
    let mut ok = true;
    ok &= dsp_fft_radix2::fft_bit_reverse_u32(1, 0) == 0;
    let mut r = [1.0f32; 3];
    let mut im = [0.0f32; 3];
    ok &= !dsp_fft_radix2::fft_radix2_f32_inplace(&mut r, &mut im);
    ok &= !dsp_fft_radix2::ifft_radix2_f32_inplace(&mut r, &mut im);
    let (a, b) = dsp_fft_radix2::fft_radix2_q15_step(0, 0, 0, 0, 0, 0);
    ok &= a == (0, 0) && b == (0, 0);
    ok
}

/// @outputs: 1
fn audit_decision_mcts() -> bool {
    let mut ok = true;
    let ucb_unvisited = decision_mcts::mcts_ucb1_score(0.0, 0, 10, 1.414);
    ok &= ucb_unvisited.is_infinite();
    ok &= decision_mcts::mcts_select_best_child(&[], &[], 10, 1.414).is_none();
    let mut wins = [0.0f32; 2];
    let mut visits = [0u32; 2];
    ok &= !decision_mcts::mcts_backpropagate_step(&mut wins, &mut visits, &[99], 1.0);
    let mut pol = [0.0f32; 2];
    ok &= !decision_mcts::mcts_visit_count_temperature_policy(&[], 1.0, &mut pol);
    ok
}

/// @outputs: 1
fn audit_curve_fit_trend() -> bool {
    let mut ok = true;
    ok &= curve_fit_trend::curve_linear_regression_slope_intercept(&[], &[]).is_none();
    ok &= curve_fit_trend::curve_r_squared_score(&[1.0], &[1.0]).is_none();
    ok &= (curve_fit_trend::curve_huber_loss_single(f64::NAN, 1.0) - 0.0).abs() < 1e-6;
    ok &= (curve_fit_trend::curve_lowess_tricube_weight(1.5) - 0.0).abs() < 1e-6;
    ok &= curve_fit_trend::curve_lowess_smooth_point(0.0, &[], &[], 1.0).is_none();
    ok &= (curve_fit_trend::curve_kernel_rbf(0.0, 1.0, 0.0, 1.0) - 0.0).abs() < 1e-6;
    ok &= (curve_fit_trend::curve_silu_activation(f32::NAN) - 0.0).abs() < 1e-6;
    ok
}

/// @outputs: 1
fn audit_ai_autograd_train() -> bool {
    let mut ok = true;
    let (ga, gb) = ai_autograd_train::autograd_scalar_add(f32::NAN, 1.0, 1.0);
    ok &= ga == 0.0 && gb == 0.0;
    let (ma, mb) = ai_autograd_train::autograd_scalar_mul(f32::NAN, 1.0, 1.0);
    ok &= ma == 0.0 && mb == 0.0;
    ok &= ai_autograd_train::autograd_scalar_pow(0.0, 2.0, 1.0) == 0.0;
    ok &= ai_autograd_train::autograd_scalar_relu(f32::NAN, 1.0) == 0.0;
    ok &= ai_autograd_train::bpe_find_most_frequent_pair(&[]).is_none();
    ok &= ai_autograd_train::bpe_merge_pair(&[], (0, 1), 2, &mut []) == 0;
    let mut mask = [0.0f32; 4];
    ok &= !ai_autograd_train::causal_attention_mask_apply(&mut mask, 0, -1e9);
    ok &= (ai_autograd_train::lr_schedule_cosine_warmup(10, 5, 0, 0.01, 0.001) - 0.0).abs() < 1e-6;
    ok &= ai_autograd_train::gradient_clip_l2_norm(&mut [], 1.0) == 0.0;
    ok &= ai_autograd_train::log_sum_exp_f32(&[]).is_none();
    ok
}

/// @outputs: 1
fn audit_comm_phy_bus() -> bool {
    let mut ok = true;
    let hdr = comm_phy_bus::mipi_csi2_short_packet_header(0x12, 0, 0x1234);
    ok &= hdr[0] == 0x12;
    ok &= comm_phy_bus::mipi_csi2_packet_crc16(&[]) == 0xFFFF;
    ok &= comm_phy_bus::mipi_dphy_lp_state_decode(true, true) == 3;
    ok &= comm_phy_bus::rgmii_tx_delay_skew_taps(20000, 100).is_none();
    ok &= comm_phy_bus::rgmii_tx_delay_skew_taps(1500, 0).is_none();
    ok &= comm_phy_bus::rgmii_speed_duplex_control_byte(999, true).is_none();
    ok &= comm_phy_bus::sgmii_auto_negotiation_advertisement(true, true, 2) & 1 == 1;
    let mut words = [0u32; 2];
    ok &= comm_phy_bus::bus_pack_8bit_to_32bit_words(&[1, 2], &mut words, true) == 0;
    let mut bytes = [0u8; 8];
    ok &= comm_phy_bus::bus_unpack_32bit_to_8bit_bytes(&[], &mut bytes, true) == 0;
    ok
}

/// @outputs: 1
fn audit_radar_biomed_dsp() -> bool {
    let mut ok = true;
    ok &= radar_biomed_dsp::radar_hadamard_cdma_code(0, 0).is_none();
    ok &= radar_biomed_dsp::radar_hadamard_cdma_code(10, 4).is_none();
    ok &= !radar_biomed_dsp::radar_tdma_tx_burst_active(0, 0, 0);
    ok &= radar_biomed_dsp::radar_fmcw_range_resolution_meters(0.0).is_none();
    ok &= radar_biomed_dsp::radar_fmcw_beat_frequency_to_range(-1.0, 1.0, 1.0).is_none();
    ok &= (radar_biomed_dsp::radar_phase_unwrap_step(f32::NAN, 0.0) - 0.0).abs() < 1e-6;
    let mut state = [0.0f32; 2];
    ok &= (radar_biomed_dsp::radar_respiration_bandpass_step(f32::NAN, &mut state, 0.9, 0.1) - 0.0).abs() < 1e-6;
    ok &= radar_biomed_dsp::dsp_quantize_f32_to_q7(f32::NAN) == 0;
    ok &= radar_biomed_dsp::dsp_quantize_f64_to_q31(f64::NAN) == 0;
    ok
}

/// @outputs: 1
fn audit_instrument_industrial() -> bool {
    let mut ok = true;
    ok &= instrument_industrial::scpi_parse_scientific_float(b"").is_none();
    ok &= instrument_industrial::scpi_parse_scientific_float(b"INVALID").is_none();
    let mut buf = [0u8; 32];
    ok &= instrument_industrial::scpi_format_dmm_voltage_range_cmd(0.0, 1.0, &mut buf) == 0;
    ok &= instrument_industrial::canopen_sdo_download_expedited_req(0, 0x1000, 0, &[1]).is_none();
    ok &= !instrument_industrial::canopen_sdo_parse_download_response(&[0x80, 0, 0, 0, 0, 0, 0, 0]);
    ok &= instrument_industrial::dslogic_sample_depth_to_time_ms(100, 0).is_none();
    ok &= instrument_industrial::litevna_frequency_sweep_step(100, 10, 10, 0).is_none();
    ok
}

/// @outputs: 1
fn audit_cad_geom_model() -> bool {
    let mut ok = true;
    ok &= cad_geom_model::kicad_via_annular_ring_width_mm(0.5, 0.4).is_none();
    ok &= cad_geom_model::kicad_trace_clearance_violation((f64::NAN, 0.0), (0.0, 0.0), 0.2);
    ok &= cad_geom_model::ipc2152_trace_temp_rise_celsius(0.0, 100.0, false).is_none();
    ok &= cad_geom_model::freecad_fillet_chord_length_mm(-1.0, 90.0).is_none();
    ok &= cad_geom_model::freecad_extrude_cylinder_volume_mm3(1.0, 0.0).is_none();
    ok &= !cad_geom_model::freecad_bounding_box_3d_containment((1.0, 1.0, 1.0), (0.0, 0.0, 0.0), (0.5, 0.5, 0.5));
    ok &= cad_geom_model::openems_mur_absorbing_boundary_coeff(0.0, 1e-12, 1e-3).is_none();
    ok &= cad_geom_model::openems_mesh_nyquist_cell_size_m(1e9, 0.5, 10).is_none();
    ok &= cad_geom_model::svg_path_arc_bounding_box((0.0, 0.0), 0.0).is_none();
    ok &= cad_geom_model::remotion_spring_damping_ratio(0.0, 100.0, 10.0).is_none();
    ok
}

/// @outputs: 1
fn audit_spc_six_sigma_msa() -> bool {
    let mut ok = true;
    ok &= spc_six_sigma_msa::spc_cp_index(10.0, 10.0, 1.0).is_none();
    ok &= spc_six_sigma_msa::spc_cpk_index(10.0, 10.0, 10.0, 1.0).is_none();
    ok &= (spc_six_sigma_msa::spc_normal_cdf_phi(f64::NEG_INFINITY) - 0.0).abs() < 1e-6;
    ok &= spc_six_sigma_msa::spc_spec_in_boundary_probability(10.0, 12.0, 11.0, 1.0).is_none();
    ok &= spc_six_sigma_msa::spc_truncated_normal_stats(5.0, 5.0, 0.0, 1.0).is_none();
    ok &= spc_six_sigma_msa::spc_truncated_normal_pdf(0.0, 1.0, 2.0, 0.0, 1.0).is_none();
    ok &= spc_six_sigma_msa::msa_gage_rr_ev_av(-1.0, 1.0).is_none();
    ok &= spc_six_sigma_msa::msa_gage_rr_percent_grr(0.0, 0.0).is_none();
    ok &= spc_six_sigma_msa::msa_precision_to_tolerance_ratio(1.0, 10.0, 10.0).is_none();
    ok &= spc_six_sigma_msa::msa_number_of_distinct_categories_ndc(10.0, 0.0).is_none();
    ok
}

/// @outputs: 1
fn audit_quant_finance_market() -> bool {
    let mut ok = true;
    ok &= quant_finance_market::quant_ema_step(10.0, 12.0, 0).is_none();
    ok &= quant_finance_market::quant_rsi_step(-1.0, 1.0).is_none();
    ok &= quant_finance_market::quant_bollinger_bands(&[10.0], 2.0).is_none();
    ok &= (quant_finance_market::quant_macd_line(f64::NAN, 1.0) - 0.0).abs() < 1e-6;
    ok &= quant_finance_market::quant_pe_pb_ratio(0.0, 1.0, 1.0).is_none();
    ok &= quant_finance_market::quant_roe_and_fcf_yield(1.0, 0.0, 1.0, 10.0).is_none();
    ok &= quant_finance_market::quant_vwap_step(0.0, 0.0, 0.0, 1.0).is_none();
    ok &= quant_finance_market::quant_order_flow_net_inflow_ratio(0.0, 0.0).is_none();
    ok &= quant_finance_market::quant_futures_annualized_carry_yield(100.0, 0.0, 30.0).is_none();
    ok &= quant_finance_market::quant_fx_adjusted_return(0.1, 0.0, 1.0).is_none();
    ok
}

/// @outputs: 1
fn audit_robotics_motion_profile() -> bool {
    let mut ok = true;
    ok &= robotics_motion_profile::robotics_planar_2link_ik(100.0, 100.0, 1.0, 1.0, false).is_none();
    ok &= robotics_motion_profile::robotics_planar_2link_fk(0.0, 0.0, 0.0, 1.0).is_none();
    ok &= (robotics_motion_profile::robotics_planar_2link_jacobian_det(f64::NAN, 1.0, 1.0) - 0.0).abs() < 1e-6;
    ok &= robotics_motion_profile::robotics_trapezoidal_profile_distance(-1.0, 10.0, 5.0, 100.0).is_none();
    ok &= robotics_motion_profile::robotics_scurve_jerk_velocity(1.0, 0.0).is_none();
    ok &= robotics_motion_profile::robotics_stepper_microstep_sin_cos(0, 0).is_none();
    let (_, warn) = robotics_motion_profile::robotics_joint_soft_limit_clamp(10.0, 5.0, 0.0, 0.1);
    ok &= warn;
    ok &= robotics_motion_profile::robotics_lead_screw_linear_distance(10.0, 0.0, 1.0).is_none();
    ok &= robotics_motion_profile::robotics_arm_payload_torque(0.0, 1.0, 9.8, 1.0).is_none();
    ok &= robotics_motion_profile::robotics_scara_z_axis_feed_rate(0.0, 1000.0).is_none();
    ok
}

/// @outputs: 1
fn audit_ai_tensor_gemm() -> bool {
    let mut ok = true;
    let mut c = [0.0f32; 4];
    ok &= !ai_tensor_gemm::gemm_tiled_f32_step(&[], &[], &mut c, 2, 2, 2, 1);
    let mut out_conv = [0.0f32; 1];
    ok &= !ai_tensor_gemm::conv2d_valid_forward_step(&[], &[], &mut out_conv, 1, 1, 2, 2);
    let mut out_pool = [0.0f32; 1];
    ok &= !ai_tensor_gemm::maxpool2d_2x2_stride2_step(&[], &mut out_pool, 3, 3);
    let mut out_bn = [0.0f32; 2];
    ok &= !ai_tensor_gemm::batch_norm_forward_inference(&[1.0, 2.0], &[0.0], &[1.0], &[1.0], &[0.0], 0.0, &mut out_bn);
    let mut x_drop = [1.0f32; 2];
    ok &= !ai_tensor_gemm::dropout_inverted_mask_apply(&mut x_drop, &[1], 1.5);
    let mut emb = [0.0f32; 4];
    ok &= !ai_tensor_gemm::embedding_lookup_1d(&[], 0, 4, &mut emb);
    ok &= (ai_tensor_gemm::leaky_relu_f32(f32::NAN, 0.1) - 0.0).abs() < 1e-6;
    ok &= (ai_tensor_gemm::smooth_l1_loss_step(0.0, 1.0, 0.0) - 0.0).abs() < 1e-6;
    ok &= ai_tensor_gemm::cosine_similarity_f32(&[0.0], &[0.0]).is_none();
    let mut one_hot = [0u8; 3];
    ok &= !ai_tensor_gemm::one_hot_encode_u8(5, 3, &mut one_hot);
    ok
}

/// @outputs: 1
fn audit_iot_wire_protocol() -> bool {
    let mut ok = true;
    let mut mqtt_buf = [0u8; 2];
    ok &= iot_wire_protocol::mqtt_remaining_length_encode(300_000_000, &mut mqtt_buf) == 0;
    ok &= iot_wire_protocol::mqtt_remaining_length_decode(&[]).is_none();
    let mut p_hdr = 0u8;
    ok &= !iot_wire_protocol::mqtt_publish_header_pack(false, 3, false, &mut p_hdr);
    let mut coap_hdr = [0u8; 4];
    ok &= !iot_wire_protocol::coap_header_pack(5, 0, 0, &mut coap_hdr);
    let mut coap_opt = [0u8; 4];
    ok &= iot_wire_protocol::coap_option_delta_encode(10, 5, 1, &mut coap_opt) == 0;
    let (mtype, major) = iot_wire_protocol::lorawan_mhdr_unpack(0xFF);
    ok &= mtype == 7 && major == 3;
    ok &= iot_wire_protocol::lorawan_frame_counter_unroll(0, 0x8000) == 0x8000;
    let mut modbus_buf = [0u8; 10];
    ok &= iot_wire_protocol::modbus_fc10_request_pack(250, 0, &[1, 2], &mut modbus_buf) == 0;
    let mut slip_buf = [0u8; 4];
    ok &= iot_wire_protocol::slip_encode_packet(&[1, 2, 3, 4, 5], &mut slip_buf) == 0;
    ok &= iot_wire_protocol::slip_decode_packet(&[0xDB], &mut slip_buf).is_none();
    ok
}

/// @outputs: 1
fn audit_radar_aoa_mti() -> bool {
    let mut ok = true;
    ok &= (radar_aoa_mti::radar_mti_two_pulse_canceler_step(f32::NAN, 1.0) - 0.0).abs() < 1e-6;
    ok &= (radar_aoa_mti::radar_mti_three_pulse_canceler_step(1.0, f32::NAN, 0.0) - 0.0).abs() < 1e-6;
    ok &= radar_aoa_mti::radar_aoa_phase_monopulse(10.0, 0.001, 0.0005).is_none(); // 超出 -1..1
    ok &= radar_aoa_mti::radar_monopulse_ratio_slope(1.0, 0.0, 0.0, 0.0).is_none(); // 除零
    ok &= radar_aoa_mti::radar_blind_speed_mps(0.001, 0.0, 1).is_none(); // PRF 超界
    ok &= radar_aoa_mti::radar_range_doppler_index_to_physical(1, 10, 1.0, 1.0, 4).is_none(); // 越界
    ok &= radar_aoa_mti::radar_clutter_to_signal_ratio_db(-1.0, 10.0).is_none(); // 負功率
    ok &= radar_aoa_mti::radar_fmcw_max_unambiguous_range(10.0, 1e6, 0.001).is_none(); // 採樣率超界
    ok &= radar_aoa_mti::radar_fmcw_max_unambiguous_velocity(0.0, 77e9).is_none(); // 區間超界
    ok &= radar_aoa_mti::radar_doppler_centroid_hz(&[], 1.0).is_none(); // 空陣列
    ok
}

/// @outputs: 1
fn audit_vision_morphology_fast() -> bool {
    let mut ok = true;
    let mut out_morph = [0u8; 4];
    ok &= !vision_morphology_fast::morphology_binary_dilation_3x3(&[], 2, 2, &mut out_morph);
    ok &= !vision_morphology_fast::morphology_binary_erosion_3x3(&[], 2, 2, &mut out_morph);
    ok &= !vision_morphology_fast::morphology_gradient_edge_3x3(&[], 2, 2, &mut out_morph);
    ok &= !vision_morphology_fast::fast9_corner_detect_point(&[], 10, 10, 1, 1, 20); // 邊緣越界
    ok &= vision_morphology_fast::vision_hough_line_accumulate_point(10, 10, 200, 100).is_none(); // 角度 > 180
    let mut prof = [0u32; 2];
    ok &= !vision_morphology_fast::vision_projection_integral_profile(&[], 0, 0, true, &mut prof);
    let iou = vision_morphology_fast::vision_bbox_intersection_over_union([0, 0, 0, 0], [10, 10, 10, 10]);
    ok &= iou == 0.0;
    ok &= vision_morphology_fast::vision_normalized_cross_correlation_step(&[10, 10], &[10, 10]).is_none(); // 零方差
    let (h, s, v) = vision_morphology_fast::vision_rgb_to_hsv_step(0, 0, 0);
    ok &= h == 0 && s == 0 && v == 0;
    ok &= !vision_morphology_fast::vision_color_hsv_in_range(400, 100, 100, 0, 180, 50, 50); // H > 360
    ok
}

/// @outputs: 1
fn audit_audio_mfcc_pitch() -> bool {
    let mut ok = true;
    ok &= (audio_mfcc_pitch::audio_hz_to_mel(f32::NAN) - 0.0).abs() < 1e-6;
    ok &= (audio_mfcc_pitch::audio_mel_to_hz(f32::NAN) - 0.0).abs() < 1e-6;
    ok &= (audio_mfcc_pitch::audio_mel_filterbank_weights(100.0, 50.0, 150.0, f32::NAN) - 0.0).abs() < 1e-6;
    ok &= audio_mfcc_pitch::audio_dct_type2_step(&[], 0, 10).is_none();
    ok &= audio_mfcc_pitch::audio_autocorrelation_pitch_hz(&[0; 10], 1000, 50.0, 500.0).is_none(); // 採樣率超界
    ok &= audio_mfcc_pitch::audio_yin_difference_step(&[0; 5], 10, 5).is_none(); // 越界
    ok &= (audio_mfcc_pitch::audio_yin_cumulative_mean_normalized_difference(10.0, 0.0, 1) - 1.0).abs() < 1e-6; // 除零歸一
    ok &= (audio_mfcc_pitch::audio_drc_compressor_gain_db(-20.0, -10.0, 0.5, 5.0) - (-20.0)).abs() < 1e-6; // 比率無效
    ok &= audio_mfcc_pitch::audio_pre_emphasis_filter_step(0, 0, 0) == 0;
    ok &= audio_mfcc_pitch::audio_spectral_flatness_measure(&[]).is_none(); // 空頻譜
    ok
}

/// @outputs: 1
fn audit_instrument_lxi_hislip() -> bool {
    let mut ok = true;
    let mut buf = [0u8; 2];
    ok &= instrument_lxi_hislip::xdr_encode_u32(100, &mut buf) == 0; // 空間不足
    ok &= instrument_lxi_hislip::xdr_decode_u32(&buf).is_none();
    ok &= instrument_lxi_hislip::xdr_encode_opaque_padded(b"123", &mut buf) == 0;
    ok &= instrument_lxi_hislip::vxi11_rpc_header_pack(1, 2, 3, 4, &mut buf) == 0;
    ok &= instrument_lxi_hislip::vxi11_create_link_params_pack(1, false, 0, "inst0", &mut buf) == 0;
    ok &= instrument_lxi_hislip::vxi11_device_read_params_pack(1, 0, 0, 0, 0, &mut buf) == 0;
    ok &= instrument_lxi_hislip::hislip_packet_header_pack(0, 0, 0, 0, &mut buf) == 0;
    ok &= instrument_lxi_hislip::hislip_packet_header_unpack(&[0; 16]).is_none(); // 幻數不匹配
    ok &= instrument_lxi_hislip::lxi_mdns_service_name_format("", "123", &mut buf) == 0;
    ok
}

/// @outputs: 1
fn audit_spc_msa_advanced() -> bool {
    let mut ok = true;
    ok &= spc_msa_advanced::spc_pp_ppk_performance(10.0, 0.0, 15.0, 5.0).is_none(); // 標準差為0
    ok &= spc_msa_advanced::spc_skewness_kurtosis(&[1.0, 2.0]).is_none(); // 樣本數不足 4
    ok &= spc_msa_advanced::spc_box_cox_transform_step(-1.0, 0.5).is_none(); // x <= 0
    let (ucl, cl, lcl) = spc_msa_advanced::spc_shewhart_xbar_limits(f64::NAN, 1.0, 1.0);
    ok &= ucl == 0.0 && cl == 0.0 && lcl == 0.0;
    let (r_ucl, r_cl, r_lcl) = spc_msa_advanced::spc_shewhart_r_limits(1.0, 2.0, 1.0); // D4 < D3
    ok &= r_ucl == 0.0 && r_cl == 0.0 && r_lcl == 0.0;
    ok &= spc_msa_advanced::spc_durbin_watson_autocorr(&[]).is_none();
    ok &= spc_msa_advanced::spc_weibull_reliability_step(-1.0, 100.0, 1.5).is_none(); // t < 0
    ok &= spc_msa_advanced::spc_anderson_darling_a2_step(&[]).is_none();
    ok &= spc_msa_advanced::spc_chi_square_cdf_approx(-1.0, 5).is_none(); // chi2 < 0
    ok &= spc_msa_advanced::spc_msa_part_to_part_variation(20.0, 10.0).is_none(); // PV > TV
    ok
}

/// @outputs: 1
fn audit_fieldbus_industrial_eth() -> bool {
    let mut ok = true;
    let mut buf = [0u8; 1];
    ok &= fieldbus_industrial_eth::ethercat_frame_header_pack(2000, &mut buf) == 0; // 超界
    ok &= fieldbus_industrial_eth::ethercat_datagram_header_pack(1, 0, 0, 2000, false, 0, &mut buf) == 0;
    ok &= !fieldbus_industrial_eth::ethercat_working_counter_verify(0, 0); // WKC 必須 > 0
    ok &= fieldbus_industrial_eth::ethercat_mailbox_coe_header_pack(20, &mut buf) == 0; // 服務號 > 15
    ok &= fieldbus_industrial_eth::profinet_rt_frame_id_classify(0) == 0; // 無效 ID
    ok &= fieldbus_industrial_eth::profinet_io_cycle_counter_step(0, 0) == 0;
    ok &= fieldbus_industrial_eth::powerlink_soc_packet_pack(0, &mut buf) == 0;
    ok &= fieldbus_industrial_eth::powerlink_preq_pres_parse(&[0x05, 0, 0]).is_none(); // 非 PReq/PRes
    ok &= fieldbus_industrial_eth::modbus_tcp_mbap_header_pack(1, 1, 0, &mut buf) == 0; // 空間不足
    ok &= fieldbus_industrial_eth::modbus_tcp_mbap_header_unpack(&[0; 5]).is_none(); // 長度不足 7
    ok
}

/// @outputs: 1
fn audit_robotics_6dof_slerp() -> bool {
    let mut ok = true;
    let p_wc = robotics_6dof_slerp::robotics_wrist_center_decouple([0.0, 0.0, 10.0], &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0], 2.0);
    ok &= (p_wc[2] - 8.0).abs() < 1e-4;
    let q_slerp = robotics_6dof_slerp::robotics_quaternion_slerp_step([1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], 1.5);
    ok &= q_slerp[0] == 1.0; // t 超界回退
    let rot = robotics_6dof_slerp::robotics_axis_angle_to_matrix([0.0, 0.0, 0.0], 0.0);
    ok &= (rot[0] - 1.0).abs() < 1e-4 && (rot[4] - 1.0).abs() < 1e-4; // 單位矩陣
    let (s, v, a) = robotics_6dof_slerp::robotics_quintic_polynomial_traj(-1.0, 10.0, 0.0, 10.0);
    ok &= s == 0.0 && v == 0.0 && a == 0.0;
    ok &= (robotics_6dof_slerp::robotics_gravity_load_torque(-1.0, 1.0, 0.0) - 0.0).abs() < 1e-6; // 負質量
    ok &= robotics_6dof_slerp::robotics_cartesian_distance_3d([0.0; 3], [0.0; 3]) == 0.0;
    ok &= robotics_6dof_slerp::robotics_quaternion_normalize([0.0; 4]).is_none();
    let p_blend = robotics_6dof_slerp::robotics_cartesian_linear_blend([1.0, 2.0, 3.0], [4.0, 5.0, 6.0], -0.1);
    ok &= p_blend == [1.0, 2.0, 3.0]; // alpha 越界
    ok
}

/// @outputs: 1
fn audit_crypto_modern_wire() -> bool {
    let mut ok = true;
    let (mut a, mut b, mut c, mut d) = (0u32, 0u32, 0u32, 0u32);
    crypto_modern_wire::chacha20_quarter_round(&mut a, &mut b, &mut c, &mut d);
    ok &= a == 0 && b == 0 && c == 0 && d == 0;
    let mut r = [0xFFu8; 16];
    crypto_modern_wire::poly1305_clamp_key(&mut r);
    ok &= r[3] & 0xF0 == 0 && r[4] & 3 == 0;
    let mut v = [0u32; 16];
    crypto_modern_wire::blake2s_g_round(&mut v, 20, 0, 0, 0, 0, 0); // 索引越界防護
    ok &= crypto_modern_wire::gf256_mul_step(0, 100) == 0;
    ok &= crypto_modern_wire::shamir_eval_polynomial_step(&[], 10) == 0; // 空多項式
    ok &= crypto_modern_wire::hotp_truncate_code(&[0; 20]) == 0;
    ok
}

/// @outputs: 1
fn audit_vision_optical_flow_homography() -> bool {
    let mut ok = true;
    let (ix2, iy2, ixiy) = vision_optical_flow_homography::vision_harris_structure_tensor_step(10, -5);
    ok &= ix2 == 100 && iy2 == 25 && ixiy == -50;
    // 奇異點/不可逆光流方程
    ok &= vision_optical_flow_homography::vision_lucas_kanade_2x2_solve(0.0, 0.0, 0.0, 1.0, 1.0).is_none();
    // 退化單應矩陣 (w' = 0)
    let h_deg = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0];
    ok &= vision_optical_flow_homography::vision_homography_project_point_2d(&h_deg, 1.0, 1.0).is_none();
    // 雙線性採樣越界
    ok &= vision_optical_flow_homography::vision_bilinear_sample_u8(&[10, 20, 30, 40], 2, 2, -1.0, 0.5).is_none();
    // 直線擬合點數過少
    ok &= vision_optical_flow_homography::vision_least_squares_line_2d(&[[1.0, 2.0]]).is_none();
    ok
}

/// @outputs: 1
fn audit_gnss_nmea_geodesy() -> bool {
    let mut ok = true;
    // 畸形 NMEA 語句
    ok &= gnss_nmea_geodesy::gnss_nmea_checksum(b"NOT_NMEA").is_none();
    // 緯度超出 90 度
    ok &= gnss_nmea_geodesy::gnss_nmea_lat_lon_to_decimal_deg(9500.0, b'N').is_none();
    // 經緯度 NaN
    ok &= gnss_nmea_geodesy::gnss_wgs84_llh_to_ecef(f64::NAN, 0.0, 0.0).is_none();
    // 地心零距離奇異點
    ok &= gnss_nmea_geodesy::gnss_satellite_elevation_azimuth_deg([0.0, 0.0, 0.0]).is_none();
    // 負協方差 DOP 崩潰防護
    ok &= gnss_nmea_geodesy::gnss_dilution_of_precision_dop([-1.0, 1.0, 1.0, 1.0]).is_none();
    ok
}

/// @outputs: 1
fn audit_quant_orderbook_l2() -> bool {
    let mut ok = true;
    // 買賣價倒置 (bid > ask)
    ok &= quant_orderbook_l2::quant_l2_bbo_mid_and_spread(105.0, 100.0).is_none();
    // 零深度 OBI
    ok &= quant_orderbook_l2::quant_l2_order_book_imbalance_obi(0.0, 0.0).is_none();
    // 空訂單簿多檔加權
    ok &= quant_orderbook_l2::quant_l2_multi_level_depth_weighted_mid(&[], &[], 5).is_none();
    // 零訂單流 Kyle's Lambda
    ok &= quant_orderbook_l2::quant_kyle_lambda_price_impact(1.0, 0.0).is_none();
    // 零成交額 Amihud
    ok &= quant_orderbook_l2::quant_amihud_illiquidity_ratio(0.05, 0.0).is_none();
    // 零容量 VPIN
    ok &= quant_orderbook_l2::quant_vpin_bucket_toxicity_step(10.0, 5.0, 0.0).is_none();
    ok
}

/// @outputs: 1
fn audit_doc_layout_typography() -> bool {
    let mut ok = true;
    ok &= doc_layout_typography::doc_pt_to_twips(f64::NAN) == 0;
    ok &= doc_layout_typography::doc_font_scale_em_px(100.0, 0, 12.0, 96.0) == 0.0;
    ok &= doc_layout_typography::doc_text_justified_space_distribution(100.0, 150.0, 3).is_none(); // 溢出
    ok &= doc_layout_typography::doc_knuth_plass_badness(100.0, 150.0) == 100_000;
    ok &= !doc_layout_typography::doc_table_column_proportional_width(&[10.0, -5.0], 100.0, &mut [0.0; 2]);
    ok &= doc_layout_typography::doc_hyphenation_break_cost(3, 1, false) == 10000; // 過短單詞
    ok
}

/// @outputs: 1
fn audit_radar_mimo_tracking() -> bool {
    let mut ok = true;
    ok &= radar_mimo_tracking::radar_mimo_virtual_array_index(2, 3, 0) == 0; // 零RX防護
    ok &= radar_mimo_tracking::radar_mimo_barker_code_biphase(99, 0).is_none(); // 非法編碼長度
    ok &= radar_mimo_tracking::radar_gnn_euclidean_distance_2d([f32::NAN, 0.0], [0.0, 0.0]) == f32::INFINITY;
    ok &= radar_mimo_tracking::radar_kalman_filter_innovation_step(10.0, 10.0, 0.0, 1.0).is_none(); // 零方差
    let (keep, _) = radar_mimo_tracking::radar_track_state_coast_decay_score(10, 5, 5);
    ok &= !keep; // 超出最大丟失
    ok
}

/// @outputs: 1
fn audit_cnc_gcode_toolpath() -> bool {
    let mut ok = true;
    ok &= cnc_gcode_toolpath::cnc_g01_linear_interpolation_step([0.0; 3], [1.0; 3], f64::NAN) == [0.0; 3];
    ok &= cnc_gcode_toolpath::cnc_arc_radius_and_angles([0.0, 0.0], [5.0, 5.0], [0.0, 0.0]).is_none(); // 半徑不符
    ok &= cnc_gcode_toolpath::cnc_cutter_radius_compensation_2d([0.0, 0.0], [0.0, 0.0], 1.5, true).is_none(); // 零長度
    ok &= cnc_gcode_toolpath::cnc_spindle_speed_surface_speed(0.0, 100.0).is_none(); // 零直徑
    ok &= cnc_gcode_toolpath::cnc_trapezoidal_velocity_profile_step(0.0, 100.0, 10.0, 2.0) == 0.0;
    ok
}

/// @outputs: 1
fn audit_robotics_dynamics_jacobian() -> bool {
    let mut ok = true;
    let (vx, vy) = robotics_dynamics_jacobian::robotics_jacobian_2link_planar_vel(0.0, 0.0, 1.0, 1.0, 0.0, 0.0);
    ok &= vx == 0.0 && vy == 0.0;
    ok &= robotics_dynamics_jacobian::robotics_manipulability_index_2d(f64::NAN, 0.0, 0.0, 0.0) == 0.0;
    ok &= robotics_dynamics_jacobian::robotics_joint_limit_repulsive_potential(0.0, 1.0, 0.0, 1.0, 0.1) == 0.0; // 邊界倒置
    let (q, q_dot) = robotics_dynamics_jacobian::robotics_forward_euler_dynamics_step(1.0, 2.0, 3.0, -0.1);
    ok &= q == 0.0 && q_dot == 0.0; // 負步長
    let f_imp = robotics_dynamics_jacobian::robotics_cartesian_compliance_spring_force([f64::NAN; 3], [0.0; 3], [100.0; 3]);
    ok &= f_imp == [0.0, 0.0, 0.0];
    ok
}

/// @outputs: 1
fn audit_nfc_iso14443_ndef() -> bool {
    let mut ok = true;
    let mut crc_a = 0x6363u16;
    nfc_iso14443_ndef::nfc_iso14443a_crc_step(0x00, &mut crc_a);
    ok &= crc_a != 0x6363;
    ok &= nfc_iso14443_ndef::nfc_iso14443b_crc_calculate(&[]) == 0xFFFF;
    ok &= nfc_iso14443_ndef::nfc_ndef_uri_prefix_expand(0xFF).is_none(); // 非法前綴
    ok &= nfc_iso14443_ndef::nfc_mifare_classic_sector_trailer_block_index(50).is_none(); // 扇區溢出 (>39)
    ok &= !nfc_iso14443_ndef::nfc_felica_frame_validate(&[0x01]); // 長度不足
    ok
}

/// @outputs: 1
fn audit_ai_transformer_attention() -> bool {
    let mut ok = true;
    ok &= ai_transformer_attention::ai_attention_scaled_dot_product_score(10.0, 0) == 0.0;
    ok &= ai_transformer_attention::ai_attention_causal_mask(1, 2) == -10000.0;
    ok &= ai_transformer_attention::ai_rmsnorm_inv_rms_factor(&[], 1e-5).is_none();
    ok &= !ai_transformer_attention::ai_kv_cache_insert_token_slice(&mut [0.0; 10], 5, 2, 4, &[1.0; 4]); // 索引超出最大序列
    let mut probs = [0.0; 2];
    ok &= !ai_transformer_attention::ai_temperature_softmax(&[1.0, 2.0], -1.0, &mut probs); // 負溫度
    ok
}

/// @outputs: 1
fn audit_analyzer_sigrok_protocol() -> bool {
    let mut ok = true;
    let (r, f) = analyzer_sigrok_protocol::analyzer_digital_edge_detect(0, 1, 9); // 通道大於 7
    ok &= !r && !f;
    ok &= analyzer_sigrok_protocol::analyzer_uart_sample_point_offset(0, 1000).is_none();
    ok &= analyzer_sigrok_protocol::analyzer_onewire_bit_decode(-5.0).is_none();
    ok &= analyzer_sigrok_protocol::analyzer_manchester_decode_transition(true, true, true).is_none(); // 無跳變
    let mut last = 0;
    let mut run = 0;
    ok &= analyzer_sigrok_protocol::analyzer_rle_compress_step(5, &mut last, &mut run, 0).is_none(); // 零長度
    ok
}

/// @outputs: 1
fn audit_decision_minimax_alphabeta() -> bool {
    let mut ok = true;
    let mut alpha = -100;
    ok &= decision_minimax_alphabeta::decision_alphabeta_window_step(&mut alpha, 50, 60); // 觸發截斷
    ok &= decision_minimax_alphabeta::decision_negamax_score_step(10) == -10;
    ok &= decision_minimax_alphabeta::decision_discounted_future_reward(&[], 0.9).is_none();
    ok &= decision_minimax_alphabeta::decision_epsilon_greedy_action(&[], 0.1, 0.05, 0).is_none();
    ok &= !decision_minimax_alphabeta::decision_null_move_pruning_allowed(2, false, 5); // 深度不足 3
    ok
}

/// @outputs: 1
fn audit_rpa_mobile_touch_macro() -> bool {
    let mut ok = true;
    ok &= rpa_mobile_touch_macro::rpa_screen_rotate_coords(100, 100, 0, 0, 90) == (0, 0); // 零螢幕尺寸
    ok &= rpa_mobile_touch_macro::rpa_human_jitter_offset(0.0, 0.0, 5.0) == (0.0, 0.0);
    ok &= rpa_mobile_touch_macro::rpa_parse_accessibility_bounds_text(b"NOT_BOUNDS").is_none();
    ok &= rpa_mobile_touch_macro::rpa_android_keycode_valid(99999).is_none();
    ok &= rpa_mobile_touch_macro::rpa_touch_gesture_state(0.0, 100, 10.0, 500) == 0;
    ok
}

/// @outputs: 1
fn audit_audio_spectral_codec() -> bool {
    let mut ok = true;
    let alaw = audio_spectral_codec::audio_g711_alaw_encode_sample(0);
    ok &= alaw != 0;
    let pcm = audio_spectral_codec::audio_g711_alaw_decode_sample(alaw);
    ok &= pcm.abs() < 100;
    ok &= (audio_spectral_codec::audio_spectral_subtraction_gain(0.0, 10.0, 2.0, 0.05) - 0.05).abs() < 1e-4; // 零信號功率底噪
    ok &= (audio_spectral_codec::audio_wiener_filter_gain(0.0) - 0.0).abs() < 1e-4;
    ok &= (audio_spectral_codec::audio_hz_to_bark(-10.0) - 0.0).abs() < 1e-4; // 負頻率
    ok
}

/// @outputs: 1
fn audit_codec_lossless_compression() -> bool {
    let mut ok = true;
    ok &= codec_lossless_compression::codec_leb128_u64_encode(0, &mut []) == 0; // 空輸出防護
    ok &= codec_lossless_compression::codec_leb128_u64_decode(&[]).1 == 0; // 空輸入防護
    ok &= codec_lossless_compression::codec_zigzag_i64_decode(codec_lossless_compression::codec_zigzag_i64_encode(-12345)) == -12345;
    ok &= codec_lossless_compression::codec_rle_count_run(&[], 10) == (0, 0); // 空切片防護
    ok &= codec_lossless_compression::codec_lz77_find_longest_match(&[], &[], 10) == (0, 0); // 空滑動視窗防護
    ok &= codec_lossless_compression::codec_bit_pack_extract_u16(&[], 0, 8) == 0; // 空數據防護
    let crc = codec_lossless_compression::codec_crc32c_castagnoli_step(0, 0x55);
    ok &= crc != 0;
    ok
}

/// @outputs: 1
fn audit_numerical_optimization_gradient() -> bool {
    let mut ok = true;
    let (a, b) = numerical_optimization_gradient::opt_golden_section_search_step(f64::NAN, 1.0, 0.0, 0.0);
    ok &= a.is_nan() && b == 1.0;
    ok &= !numerical_optimization_gradient::opt_wolfe_condition_armijo(1.0, 2.0, 1.0, 0.1, 0.1); // 非下降方向
    ok &= !numerical_optimization_gradient::opt_wolfe_condition_curvature(1.0, -1.0, 0.9); // 非下降方向
    ok &= numerical_optimization_gradient::opt_conjugate_gradient_polak_ribiere(&[], &[]) == 0.0;
    let mut h = [1.0];
    ok &= !numerical_optimization_gradient::opt_bfgs_hessian_inv_update_step(&mut h, &[1.0], &[-2.0], 1); // 負曲率
    ok &= numerical_optimization_gradient::opt_gauss_newton_step_2d(&[], &[]).is_none();
    ok &= numerical_optimization_gradient::opt_levenberg_marquardt_damping(f64::NAN, 0.5) == 1.0;
    ok &= !numerical_optimization_gradient::opt_nelder_mead_reflection_point(&[], &[], 1.0, &mut []);
    ok &= numerical_optimization_gradient::opt_learning_rate_warmup_cosine(0, 0, 0, 1.0, 0.0) == 0.0;
    ok
}

/// @outputs: 1
fn audit_fieldbus_motion_cia402() -> bool {
    let mut ok = true;
    let s = fieldbus_motion_cia402::cia402_state_machine_transition(99, 0);
    ok &= s == fieldbus_motion_cia402::CIA402_STATE_SWITCH_ON_DISABLED;
    ok &= fieldbus_motion_cia402::cia402_statusword_decode_state(0xFFFF) == fieldbus_motion_cia402::CIA402_STATE_SWITCH_ON_DISABLED;
    ok &= fieldbus_motion_cia402::cia402_controlword_from_command(99) == 0x0000;
    ok &= !fieldbus_motion_cia402::cia402_pp_target_position_valid(100, 200, 100); // 倒置邊界
    ok &= fieldbus_motion_cia402::cia402_pv_speed_rpm_to_cps(f64::NAN, 1000) == 0;
    ok &= (fieldbus_motion_cia402::cia402_quick_stop_deceleration_time(1000.0, -10.0) - 0.0).abs() < 1e-4; // 負減速度
    ok &= (fieldbus_motion_cia402::cia402_torque_permille_to_nm(1000, -5.0) - 0.0).abs() < 1e-4; // 負額定轉矩
    ok
}

/// @outputs: 1
fn audit_ai_rl_policy_gradient() -> bool {
    let mut ok = true;
    ok &= ai_rl_policy_gradient::rl_ppo_clipped_surrogate_loss(f64::NAN, 0.0, 1.0, 0.2) == 0.0;
    ok &= ai_rl_policy_gradient::rl_gae_advantage_step(0.0, 0.0, 0.0, false, 0.0, -1.0, 0.95) == 0.0; // 負折扣率
    ok &= ai_rl_policy_gradient::rl_gaussian_policy_log_prob(0.0, 0.0, -1.0) == -1e9; // 負標準差
    ok &= ai_rl_policy_gradient::rl_discrete_policy_entropy(&[]) == 0.0;
    ok &= ai_rl_policy_gradient::rl_gumbel_sample_perturb_score(0.0, -0.1) == -1e9; // 越界隨機數
    ok &= ai_rl_policy_gradient::rl_discounted_return_step(10.0, 100.0, false, -0.5) == 0.0;
    ok &= ai_rl_policy_gradient::rl_polyak_target_soft_update(1.0, 2.0, -0.1) == 2.0; // 越界tau
    ok &= ai_rl_policy_gradient::rl_value_huber_loss(10.0, 0.0, -1.0) == 0.0;
    ok
}

/// @outputs: 1
fn audit_rpa_desktop_macro_driver() -> bool {
    let mut ok = true;
    ok &= rpa_desktop_macro_driver::rpa_win32_vk_code_map("").is_none();
    ok &= rpa_desktop_macro_driver::rpa_x11_keysym_map("INVALID_KEY").is_none();
    let (nx, ny) = rpa_desktop_macro_driver::rpa_screen_coords_to_sendinput_normalized(-10, 0, 1920, 1080);
    ok &= nx == 0 && ny == 0;
    ok &= rpa_desktop_macro_driver::rpa_mouse_event_flags_encode(99, true) == 0;
    ok &= rpa_desktop_macro_driver::rpa_keyboard_modifiers_mask(false, false, false, false) == 0;
    ok &= rpa_desktop_macro_driver::rpa_ocr_text_levenshtein_distance("", "") == 0;
    ok &= rpa_desktop_macro_driver::rpa_screen_find_first_color(&[], 0, 0, [0, 0, 0], 5).is_none();
    ok &= rpa_desktop_macro_driver::rpa_key_delay_jitter_ms(0, 10, 5) == 0;
    ok
}

/// @outputs: 1
fn audit_dsp_fft_multirate_2d() -> bool {
    let mut ok = true;
    ok &= dsp_fft_multirate_2d::dsp_goertzel_single_bin_energy(&[], 0, 0) == 0.0;
    let b = dsp_fft_multirate_2d::dsp_radix4_butterfly_step(
        (0.0, 0.0), (0.0, 0.0), (0.0, 0.0), (0.0, 0.0),
        (1.0, 0.0), (1.0, 0.0), (1.0, 0.0),
    );
    ok &= b[0] == (0.0, 0.0);
    ok &= dsp_fft_multirate_2d::dsp_czt_chirp_weight(0, 0) == (1.0, 0.0);
    ok &= dsp_fft_multirate_2d::dsp_2d_spectrum_center_shift_factor(0, 0) == 1.0;
    ok &= dsp_fft_multirate_2d::dsp_2d_gaussian_lowpass_filter_kernel(0, 0, 0, 0, 10.0) == 0.0;
    ok &= dsp_fft_multirate_2d::dsp_analytic_signal_instantaneous_phase(f64::NAN, 0.0) == 0.0;
    ok &= dsp_fft_multirate_2d::dsp_instantaneous_frequency_step(0.0, 0.0, -100.0) == 0.0;
    ok
}

/// @outputs: 1
fn audit_doc_office_drawing_shape() -> bool {
    let mut ok = true;
    ok &= doc_office_drawing_shape::doc_points_to_emu(f64::NAN) == 0;
    let m = doc_office_drawing_shape::doc_shape_affine_matrix_step(f64::NAN, 0.0, 0.0, false, false);
    ok &= m[0] == 1.0;
    let (min_x, _, _, _) = doc_office_drawing_shape::doc_shape_rotated_aabb(0.0, 0.0, -10.0, 10.0, 0.0);
    ok &= min_x == 0.0;
    let corner = doc_office_drawing_shape::doc_orthogonal_connector_corner((f64::NAN, 0.0), (10.0, 10.0), true);
    ok &= corner.0.is_nan();
    ok &= doc_office_drawing_shape::doc_gradient_stop_lerp_rgb([0, 0, 0], [255, 255, 255], f64::NAN) == [0, 0, 0];
    ok &= doc_office_drawing_shape::doc_textbox_autofit_font_scale(0, 100.0, 100.0, 12.0) == 1.0;
    ok &= doc_office_drawing_shape::doc_distribute_elements_spacing(100.0, &[60.0, 60.0]).is_none(); // 溢出
    let (dx, dy) = doc_office_drawing_shape::doc_drop_shadow_offset(-5.0, 0.0);
    ok &= dx == 0.0 && dy == 0.0;
    ok
}

/// @outputs: 1
fn audit_radar_clustering_dbscan_tracker() -> bool {
    let mut ok = true;
    ok &= !radar_clustering_dbscan_tracker::radar_dbscan_density_reachable(-1.0, 1.0);
    ok &= radar_clustering_dbscan_tracker::radar_cluster_centroid_and_radial_velocity(&[], &[], &[]).is_none();
    ok &= !radar_clustering_dbscan_tracker::radar_jpda_association_weights(&[], 0.1, &mut []);
    let mut probs = [0.33, 0.33, 0.34];
    let trans = [[0.8, 0.1, 0.1], [0.1, 0.8, 0.1], [0.1, 0.1, 0.8]];
    ok &= radar_clustering_dbscan_tracker::radar_imm_mode_probabilities_step(&mut probs, &trans, &[1.0, 0.5, 0.2]);
    let state = radar_clustering_dbscan_tracker::radar_ctrv_motion_model_predict([0.0; 5], -0.1);
    ok &= state == [0.0; 5];
    ok &= radar_clustering_dbscan_tracker::radar_vital_signs_frequency_separation(&[], 100.0) == (0.0, 0.0);
    ok &= radar_clustering_dbscan_tracker::radar_swerling1_snr_fluctuation(-10.0, 0.5) == 0.0;
    ok &= radar_clustering_dbscan_tracker::radar_track_lifecycle_hysteresis_step(0, 0, 10, 3, 5) == 2;
    ok
}

/// @outputs: 1
fn audit_fieldbus_building_automation() -> bool {
    let mut ok = true;
    ok &= fieldbus_building_automation::bacnet_mstp_crc8(&[]) == 0;
    ok &= fieldbus_building_automation::bacnet_mstp_crc8(&[0u8; 70]) == 0;
    ok &= fieldbus_building_automation::bacnet_mstp_crc16(&[0u8; 1025]) == 0xFFFF;
    ok &= fieldbus_building_automation::mbus_frame_checksum(&[]).is_none();
    ok &= fieldbus_building_automation::mbus_dif_data_len(0x04) == 4;
    ok &= fieldbus_building_automation::mbus_dif_data_len(0x0F) == 0;
    ok &= fieldbus_building_automation::dlms_hdlc_fcs16(&[0u8; 520]) == 0xFFFF;
    ok &= !fieldbus_building_automation::dlms_obis_is_valid(&[0x10, 0, 0, 0, 0, 0]);
    ok &= fieldbus_building_automation::dlms_obis_is_valid(&[0x01, 0, 1, 8, 0, 255]);
    ok &= fieldbus_building_automation::dnp3_crc16(&[0u8; 300]) == 0;
    ok &= fieldbus_building_automation::hart_lrc_checksum(&[0u8; 300]) == 0;
    ok &= fieldbus_building_automation::knx_tp1_checksum(&[]) == 0xFF;
    ok &= fieldbus_building_automation::smart_meter_pulse_to_kwh(1000, 0).is_none();
    ok &= fieldbus_building_automation::smart_meter_pulse_to_kwh(1000, 1000) == Some(1.0);
    ok
}

/// @outputs: 1
fn audit_physics_gjk_epa_contact() -> bool {
    let mut ok = true;
    ok &= physics_gjk_epa_contact::gjk_support_sphere([0.0; 3], -1.0, [1.0, 0.0, 0.0]).is_none();
    ok &= physics_gjk_epa_contact::gjk_support_sphere([0.0; 3], 1.0, [0.0; 3]) == Some([0.0; 3]);
    ok &= physics_gjk_epa_contact::gjk_support_box([-1.0, 1.0, 1.0], [1.0, 0.0, 0.0]).is_none();
    let m = physics_gjk_epa_contact::gjk_minkowski_support_point([f32::NAN, 0.0, 0.0], [0.0; 3]);
    ok &= m == [0.0; 3];
    let (_, n) = physics_gjk_epa_contact::gjk_simplex_line_step([0.0; 3], [0.0; 3]);
    ok &= n == 1 || n == 2;
    ok &= physics_gjk_epa_contact::gjk_simplex_triangle_normal([0.0; 3], [0.0; 3], [0.0; 3]).is_none();
    ok &= physics_gjk_epa_contact::epa_triangle_normal_and_distance([0.0; 3], [0.0; 3], [0.0; 3]).is_none();
    let vr = physics_gjk_epa_contact::physics_contact_relative_velocity([0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3]);
    ok &= vr == [0.0; 3];
    ok &= physics_gjk_epa_contact::physics_contact_normal_impulse(-5.0, 0.0, 0.5, 0.0, 0.2) == 0.0;
    ok &= physics_gjk_epa_contact::physics_coulomb_friction_clamp(-1.0, 0.5, 10.0) == 0.0;
    ok &= physics_gjk_epa_contact::physics_rigid_body_angular_impulse([0.0; 3], [0.0; 3], [-1.0, 1.0, 1.0]) == [0.0; 3];
    ok
}

/// @outputs: 1
fn audit_codec_container_multimedia() -> bool {
    let mut ok = true;
    ok &= codec_container_multimedia::isobmff_box_header_parse(&[]).is_none();
    ok &= codec_container_multimedia::isobmff_box_header_parse(&[0u8; 7]).is_none();
    ok &= codec_container_multimedia::ebml_vint_decode(&[]).is_none();
    ok &= codec_container_multimedia::ebml_vint_decode(&[0u8; 10]).is_none();
    ok &= codec_container_multimedia::mpegts_sync_packet_validate(&[]).is_none();
    ok &= codec_container_multimedia::mpegts_pcr_to_timestamp_ns(u64::MAX, 500).is_none();
    ok &= codec_container_multimedia::flv_tag_header_parse(&[0u8; 11]).is_none();
    ok &= codec_container_multimedia::id3v2_syncsafe_to_u32([0x80, 0, 0, 0]).is_none();
    ok &= codec_container_multimedia::id3v2_u32_to_syncsafe(0xFFFF_FFFF).is_none();
    ok &= codec_container_multimedia::ogg_page_header_validate(&[]).is_none();
    ok &= codec_container_multimedia::audio_samples_to_duration_ms(100, 0).is_none();
    ok &= codec_container_multimedia::audio_samples_to_duration_ms(44100, 44100) == Some(1000);
    ok
}

/// @outputs: 1
fn audit_control_systems_estimation() -> bool {
    let mut ok = true;
    ok &= control_systems_estimation::rts_smoother_gain_1d(-1.0, 1.0, 1.0).is_none();
    ok &= control_systems_estimation::rts_smoother_gain_1d(1.0, 1.0, 0.0).is_none();
    ok &= control_systems_estimation::rts_smoother_state_step_1d(f32::NAN, 0.5, 10.0, 8.0).is_nan();
    ok &= control_systems_estimation::lqr_gain_step_1d(1.0, 1.0, -1.0, 1.0, 1.0).is_none();
    ok &= control_systems_estimation::lqr_gain_step_1d(1.0, 1.0, 1.0, 0.0, 1.0).is_none();
    let (x_next, _) = control_systems_estimation::state_space_step_2d([[1.0, 0.0], [0.0, 1.0]], [1.0, 0.0], [1.0, 0.0], 0.0, [f32::NAN, 0.0], 1.0);
    ok &= x_next[0].is_nan();
    ok &= control_systems_estimation::rls_parameter_step_1d(0.0, 1.0, 1.0, 1.0, 0.1).is_none();
    ok &= control_systems_estimation::tustin_bilinear_first_order(0.0, 0.01).is_none();
    ok &= control_systems_estimation::pid_anti_windup_back_calc_step(10.0, 5.0, 0.0, 2.0, 0.01) == 2.0;
    ok &= control_systems_estimation::lead_lag_max_phase_lead_rad(0.0).is_none();
    ok &= control_systems_estimation::lead_lag_max_phase_lead_rad(1.0).is_none();
    ok &= control_systems_estimation::sliding_mode_saturation_control(5.0, 0.0, 10.0) == 0.0;
    ok &= control_systems_estimation::ljung_box_q_stat_lag1(&[]).is_none();
    ok &= control_systems_estimation::ljung_box_q_stat_lag1(&[1.0, 2.0]).is_none();
    ok
}

/// @outputs: 1
fn audit_geo_spatial_projection() -> bool {
    let mut ok = true;
    ok &= geo_spatial_projection::wgs84_lla_to_ecef(91.0, 0.0, 0.0).is_none();
    ok &= geo_spatial_projection::wgs84_lla_to_ecef(0.0, 181.0, 0.0).is_none();
    ok &= geo_spatial_projection::ecef_to_enu_step(0.0, 0.0, 0.0, 100.0, 0.0).is_none();
    ok &= geo_spatial_projection::web_mercator_forward_epsg3857(86.0, 0.0).is_none();
    ok &= geo_spatial_projection::web_mercator_inverse_epsg3857(30000000.0, 0.0).is_none();
    ok &= geo_spatial_projection::slippy_tile_xy_from_latlon(0.0, 0.0, 30).is_none();
    ok &= geo_spatial_projection::utm_zone_from_longitude(200.0).is_none();
    ok &= geo_spatial_projection::utm_central_meridian_lon(0).is_none();
    ok &= geo_spatial_projection::utm_central_meridian_lon(61).is_none();
    ok &= geo_spatial_projection::vincenty_reduced_latitude_rad(f64::NAN).is_none();
    ok &= geo_spatial_projection::rhumb_line_bearing_deg(0.0, 0.0, f64::NAN, 1.0).is_none();
    ok &= geo_spatial_projection::spherical_polygon_area_m2(-1.0).is_none();
    ok &= geo_spatial_projection::spherical_polygon_area_m2(0.0) == Some(0.0);
    ok
}

/// @outputs: 1
fn audit_audio_beamforming_doa() -> bool {
    let mut ok = true;
    ok &= audio_beamforming_doa::acoustic_sound_speed_air(-60.0).is_none();
    ok &= audio_beamforming_doa::acoustic_sound_speed_air(100.0).is_none();
    ok &= audio_beamforming_doa::mic_array_spatial_aliasing_limit_hz(0.0, 340.0).is_none();
    ok &= audio_beamforming_doa::mic_array_steering_delay_sec(0.0, 0.0, 0.0, 100.0).is_none();
    let coeffs = audio_beamforming_doa::dsp_lagrange_fractional_delay_coeffs(f32::NAN);
    ok &= coeffs == [0.0, 1.0, 0.0];
    let phat = audio_beamforming_doa::srp_phat_cross_phase_weight(0.0, 0.0);
    ok &= phat == (0.0, 0.0);
    ok &= audio_beamforming_doa::ula_steering_vector_phase(0, 0.0, 1.0, 0.0).is_none();
    ok &= audio_beamforming_doa::music_pseudospectrum_value(-1.0).is_none();
    ok &= audio_beamforming_doa::beamformer_delay_and_sum_step(&[], &[]).is_none();
    ok &= audio_beamforming_doa::directivity_index_db(-1.0, 1.0).is_none();
    ok &= audio_beamforming_doa::gsc_blocking_matrix_step(10.0, 10.0) == 0.0;
    ok
}

/// @outputs: 1
fn audit_storage_block_geometry() -> bool {
    let mut ok = true;
    ok &= storage_block_geometry::mbr_chs_to_lba(0, 0, 0, 16, 63).is_none();
    ok &= storage_block_geometry::mbr_chs_to_lba(0, 16, 1, 16, 63).is_none();
    ok &= storage_block_geometry::mbr_partition_entry_parse(&[0xFF; 16]).is_none();
    ok &= !storage_block_geometry::gpt_header_signature_valid(&[]);
    ok &= !storage_block_geometry::gpt_header_signature_valid(&[0u8; 16]);
    ok &= storage_block_geometry::gpt_partition_span_sectors(100, 50).is_none();
    ok &= !storage_block_geometry::raid5_parity_block_xor_step(&mut [], &[]);
    ok &= !storage_block_geometry::raid6_syndrome_q_step(&mut [], &[], 2);
    ok &= storage_block_geometry::storage_sector_alignment_offset(100, 0, 4096).is_none();
    ok &= storage_block_geometry::nvme_lbad_to_sector_size(8).is_none();
    ok &= storage_block_geometry::flash_wear_leveling_health_pct(100, 0).is_none();
    ok &= storage_block_geometry::storage_capacity_gib_from_lba(1000, 100).is_none();
    ok
}

/// @outputs: 1
fn audit_vision_camera_calibration_pnp() -> bool {
    let mut ok = true;
    ok &= vision_camera_calibration_pnp::camera_pinhole_project_point([0.0, 0.0, 0.0], 500.0, 500.0, 320.0, 240.0).is_none();
    ok &= vision_camera_calibration_pnp::camera_pinhole_project_point([0.0, 0.0, 1.0], -500.0, 500.0, 320.0, 240.0).is_none();
    let r_dist = vision_camera_calibration_pnp::camera_radial_distortion_apply(f32::NAN, 0.0, 0.1, 0.0, 0.0);
    ok &= r_dist[0].is_nan();
    let t_dist = vision_camera_calibration_pnp::camera_tangential_distortion_apply(0.0, 0.0, 0.01, 0.01);
    ok &= t_dist == [0.0, 0.0];
    ok &= vision_camera_calibration_pnp::epipolar_line_from_fundamental([[0.0; 3]; 3], [0.0, 0.0]).is_none();
    ok &= vision_camera_calibration_pnp::epipolar_sampson_distance_error([[0.0; 3]; 3], [0.0, 0.0], [0.0, 0.0]) == Some(0.0);
    ok &= vision_camera_calibration_pnp::vision_dlt_point_normalize_scale(&[]).is_none();
    ok &= vision_camera_calibration_pnp::vision_dlt_point_normalize_scale(&[[0.0, 0.0]; 3]).is_none();
    ok &= vision_camera_calibration_pnp::camera_reprojection_rmse(&[], &[]).is_none();
    ok &= vision_camera_calibration_pnp::camera_reprojection_rmse(&[[0.0, 0.0]], &[[0.0, 0.0]]) == Some(0.0);
    ok &= vision_camera_calibration_pnp::p3p_grunert_cosine_law_step(1.0, 1.0, 1.0) == 0.0;
    ok
}

/// @outputs: 1
fn check_audit_domain(ok: bool, fail_msg: &str, total: &mut usize, passed: &mut usize, failures: &mut Vec<String>) {
    *total += 1;
    if ok {
        *passed += 1;
    } else {
        failures.push(fail_msg.to_string());
    }
}

/// @outputs: 1
fn run_chaos_audit_batch_a(total: &mut usize, passed: &mut usize, failures: &mut Vec<String>) {
    check_audit_domain(audit_math_and_geometry(), "【數值幾何黑箱抽查失敗】存在未防護之邊界發散", total, passed, failures);
    check_audit_domain(audit_audio_and_sensors(), "【音訊感測器黑箱抽查失敗】存在畸形輸入崩潰", total, passed, failures);
    check_audit_domain(audit_thermal_and_dsp(), "【熱成像DSP黑箱抽查失敗】存在溢位或倒置崩潰", total, passed, failures);
    check_audit_domain(audit_security_and_firmware(), "【金融韌體黑箱抽查失敗】存在安全性或長度溢出", total, passed, failures);
    check_audit_domain(audit_ble_and_crypto(), "【藍芽密碼黑箱抽查失敗】存在Padding或截斷漏洞", total, passed, failures);
    check_audit_domain(audit_motor_env_and_dukpt(), "【馬達環境DUKPT黑箱抽查失敗】存在溢出或除零崩潰", total, passed, failures);
    check_audit_domain(audit_comm_and_clock(), "【通信時脈RTC黑箱抽查失敗】存在除數或對齊崩潰", total, passed, failures);
    check_audit_domain(audit_ethernet_and_apdu(), "【以太網APDU黑箱抽查失敗】存在截斷或解析漏洞", total, passed, failures);
    check_audit_domain(audit_power_and_baro(), "【電力氣壓光感黑箱抽查失敗】存在除零或負壓崩潰", total, passed, failures);
    check_audit_domain(audit_lin_and_raycast_3d(), "【車載LIN射線3D黑箱抽查失敗】存在平行或空數據崩潰", total, passed, failures);
    check_audit_domain(audit_tsn_and_macsec(), "【TSN與MACsec黑箱抽查失敗】存在時間倒流或重放漏洞", total, passed, failures);
    check_audit_domain(audit_agent_and_system(), "【代理馬具終端抽查失敗】存在路徑穿越或退格下溢", total, passed, failures);
    check_audit_domain(audit_ai_tensor_train(), "【AI訓練張量算子抽查失敗】存在溢出除零或NaN穿透", total, passed, failures);
    check_audit_domain(audit_instrument_scpi_vna(), "【儀器控制SCPI與VNA抽查失敗】存在除零或阻抗奇異點", total, passed, failures);
    check_audit_domain(audit_pcb_cad_fem(), "【PCB與電磁FDTD抽查失敗】存在CFL除零或無效介電常數", total, passed, failures);
    check_audit_domain(audit_motion_render_timeline(), "【影音動態Remotion時序抽查失敗】存在FPS除零或牛頓法發散", total, passed, failures);
    check_audit_domain(audit_office_calc_layout(), "【辦公試算表與排版抽查失敗】存在PMT除零或溢出折行崩潰", total, passed, failures);
    check_audit_domain(audit_rpa_screen_control(), "【RPA按鍵螢幕控制抽查失敗】存在螢幕色彩或GPIB握手崩潰", total, passed, failures);
    check_audit_domain(audit_radar_cfar_doppler(), "【雷達CFAR與都卜勒抽查失敗】存在頻率除零或點雲邊界崩潰", total, passed, failures);
    check_audit_domain(audit_dsp_fft_radix2(), "【快速傅立葉FFT抽查失敗】存在非2的冪次方或溢出崩潰", total, passed, failures);
    check_audit_domain(audit_decision_mcts(), "【MCTS蒙地卡羅決策抽查失敗】存在無效路徑或零拜訪崩潰", total, passed, failures);
    check_audit_domain(audit_curve_fit_trend(), "【曲線擬合與趨勢分析抽查失敗】存在除零或核半徑崩潰", total, passed, failures);
}

/// @outputs: 1
fn run_chaos_audit_batch_b(total: &mut usize, passed: &mut usize, failures: &mut Vec<String>) {
    check_audit_domain(audit_ai_autograd_train(), "【自動求導與神經訓練抽查失敗】存在除零溢出或NaN穿透", total, passed, failures);
    check_audit_domain(audit_comm_phy_bus(), "【實體層總線MIPI與RGMII抽查失敗】存在除零溢出或無效速率", total, passed, failures);
    check_audit_domain(audit_radar_biomed_dsp(), "【雷達生醫與FMCW測距抽查失敗】存在除零溢出或無效頻寬", total, passed, failures);
    check_audit_domain(audit_instrument_industrial(), "【工控與儀器協議抽查失敗】存在除零溢出或無效浮點", total, passed, failures);
    check_audit_domain(audit_cad_geom_model(), "【CAD與PCB機構幾何抽查失敗】存在除零溢出或倒置包圍盒", total, passed, failures);
    check_audit_domain(audit_spc_six_sigma_msa(), "【六標準差與車用MSA抽查失敗】存在除零溢出或無效規格", total, passed, failures);
    check_audit_domain(audit_quant_finance_market(), "【量化金融與股市指標抽查失敗】存在除零溢出或無效股價", total, passed, failures);
    check_audit_domain(audit_robotics_motion_profile(), "【機器人運動學與馬達輪廓抽查失敗】存在奇異點發散或除零崩潰", total, passed, failures);
    check_audit_domain(audit_ai_tensor_gemm(), "【AI張量算子分塊GEMM抽查失敗】存在維度不匹配或除零崩潰", total, passed, failures);
    check_audit_domain(audit_iot_wire_protocol(), "【物聯網通訊與線路協議抽查失敗】存在溢出或轉義漏洞", total, passed, failures);
    check_audit_domain(audit_radar_aoa_mti(), "【雷達測角與動目標顯示抽查失敗】存在除零溢出或無效視角", total, passed, failures);
    check_audit_domain(audit_vision_morphology_fast(), "【視覺形態學與角點特徵抽查失敗】存在越界或無效色彩", total, passed, failures);
    check_audit_domain(audit_audio_mfcc_pitch(), "【音訊MFCC與基頻特徵抽查失敗】存在溢出或除零崩潰", total, passed, failures);
    check_audit_domain(audit_instrument_lxi_hislip(), "【網路儀控LXI與HiSLIP抽查失敗】存在XDR解碼或封包頭溢出", total, passed, failures);
    check_audit_domain(audit_spc_msa_advanced(), "【進階SPC與車規MSA抽查失敗】存在偏度溢出或無效變異", total, passed, failures);
    check_audit_domain(audit_fieldbus_industrial_eth(), "【工業乙太網現場總線抽查失敗】存在訊框溢出或WKC校驗崩潰", total, passed, failures);
    check_audit_domain(audit_robotics_6dof_slerp(), "【機器人6軸解耦與四元數SLERP抽查失敗】存在退化或除零崩潰", total, passed, failures);
    check_audit_domain(audit_crypto_modern_wire(), "【現代高階線路加密抽查失敗】存在置換越界或GF256崩潰", total, passed, failures);
    check_audit_domain(audit_vision_optical_flow_homography(), "【視覺光流與透視單應抽查失敗】存在奇異矩陣或越界採樣崩潰", total, passed, failures);
    check_audit_domain(audit_gnss_nmea_geodesy(), "【GNSS導航與大地測量抽查失敗】存在坐標奇異點或除零崩潰", total, passed, failures);
    check_audit_domain(audit_quant_orderbook_l2(), "【量化L2訂單簿與微觀結構抽查失敗】存在除零溢出或無效買賣價", total, passed, failures);
    check_audit_domain(audit_doc_layout_typography(), "【Word與PPT排版字體抽查失敗】存在除零溢出或惡化度崩潰", total, passed, failures);
    check_audit_domain(audit_radar_mimo_tracking(), "【雷達MIMO虛擬陣列與追蹤抽查失敗】存在除零溢出或航跡發散", total, passed, failures);
    check_audit_domain(audit_cnc_gcode_toolpath(), "【CNC工具路徑與G-Code抽查失敗】存在除零溢出或幾何畸形", total, passed, failures);
    check_audit_domain(audit_robotics_dynamics_jacobian(), "【機器人動力學與雅可比抽查失敗】存在除零溢出或發散", total, passed, failures);
    check_audit_domain(audit_nfc_iso14443_ndef(), "【NFC非接觸協議與NDEF抽查失敗】存在CRC溢出或前綴解析崩潰", total, passed, failures);
    check_audit_domain(audit_ai_transformer_attention(), "【AI注意力與大模型運算抽查失敗】存在除零溢出或NaN發散", total, passed, failures);
    check_audit_domain(audit_analyzer_sigrok_protocol(), "【邏輯分析儀與協議解碼抽查失敗】存在除零溢出或無效通道", total, passed, failures);
    check_audit_domain(audit_decision_minimax_alphabeta(), "【對抗搜尋與極大極小決策抽查失敗】存在除零溢出或剪枝發散", total, passed, failures);
    check_audit_domain(audit_rpa_mobile_touch_macro(), "【手機自動化連點與觸控巨集抽查失敗】存在除零溢出或坐標越界", total, passed, failures);
    check_audit_domain(audit_audio_spectral_codec(), "【音訊頻譜降噪與語音編解碼抽查失敗】存在除零溢出或壓擴失真", total, passed, failures);
    check_audit_domain(audit_codec_lossless_compression(), "【無損數據壓縮與線路編碼抽查失敗】存在溢出或越界解碼崩潰", total, passed, failures);
    check_audit_domain(audit_numerical_optimization_gradient(), "【數值最佳化與梯度求解抽查失敗】存在奇異發散或除零崩潰", total, passed, failures);
    check_audit_domain(audit_fieldbus_motion_cia402(), "【工業驅動器CiA402運動協議抽查失敗】存在狀態機或換算崩潰", total, passed, failures);
    check_audit_domain(audit_ai_rl_policy_gradient(), "【強化學習與策略梯度抽查失敗】存在除零溢出或NaN發散", total, passed, failures);
    check_audit_domain(audit_rpa_desktop_macro_driver(), "【桌面RPA與虛擬輸入抽查失敗】存在坐標越界或鍵碼崩潰", total, passed, failures);
    check_audit_domain(audit_dsp_fft_multirate_2d(), "【二維FFT與多速率DSP抽查失敗】存在除零溢出或頻譜發散", total, passed, failures);
    check_audit_domain(audit_doc_office_drawing_shape(), "【文書向量圖形與排版抽查失敗】存在除零溢出或坐標發散", total, passed, failures);
    check_audit_domain(audit_radar_clustering_dbscan_tracker(), "【雷達點雲聚類與追蹤抽查失敗】存在除零溢出或發散", total, passed, failures);
    check_audit_domain(audit_fieldbus_building_automation(), "【樓宇自動化與智慧電表協定抽查失敗】存在CRC溢出或無效常數", total, passed, failures);
    check_audit_domain(audit_physics_gjk_epa_contact(), "【剛體動力學與GJK/EPA抽查失敗】存在除零溢出或法向發散", total, passed, failures);
    check_audit_domain(audit_codec_container_multimedia(), "【多媒體容器與封裝流抽查失敗】存在標頭溢出或VINT解析崩潰", total, passed, failures);
    check_audit_domain(audit_control_systems_estimation(), "【控制系統與卡爾曼平滑抽查失敗】存在Riccati發散或除零崩潰", total, passed, failures);
    check_audit_domain(audit_geo_spatial_projection(), "【地理空間與大地投影抽查失敗】存在奇異點發散或除零崩潰", total, passed, failures);
    check_audit_domain(audit_audio_beamforming_doa(), "【聲學陣列與波束成形抽查失敗】存在除零溢出或相位發散", total, passed, failures);
    check_audit_domain(audit_storage_block_geometry(), "【儲存幾何與磁碟分區抽查失敗】存在溢出或非法邊界", total, passed, failures);
    check_audit_domain(audit_vision_camera_calibration_pnp(), "【相機標定與對極位姿抽查失敗】存在除零溢出或退化矩陣崩潰", total, passed, failures);
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

    run_chaos_audit_batch_a(&mut total, &mut passed, &mut failures);
    run_chaos_audit_batch_b(&mut total, &mut passed, &mut failures);

    if !failures.is_empty() {
        return Err(failures);
    }
    Ok(AuditSummary {
        total_inspections: total,
        passed_inspections: passed,
        failed_inspections: 0,
    })
}
