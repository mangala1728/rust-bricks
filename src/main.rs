// ## Map
// Role: app/main — rust-bricks 命令列自測、索引產生與守護入口 (CLI Runner)。
// Inputs: 命令列參數（selftest / catalog / guard）。
// Outputs: 退出代碼與終端執行報告。
// Invariants: 零全域可變狀態、無遞迴、零外部依賴、控制流巢狀深度 <= 2。

use rust_bricks::{
    agent_terminal, ai_autograd_train, ai_tensor_gemm, ai_tensor_train, audio_mfcc_pitch, audio_vad, bezier_curve, ble_adv_parser, cad_geom_model, can_bus_frame, can_lin_bus, catalog,
    clock_pll_rtc, comm_controller, comm_phy_bus, crypto_dukpt, crypto_kdf, crypto_modern_wire, crypto_vault, curve_fit_trend, decision_mcts, doc_layout_typography, dsp_fft_radix2,
    dsp_fixed_point, env_sensor, fast_math, fieldbus_industrial_eth, fin_apdu_track, geo_collision, gnss_nmea_geodesy, guard, instrument_industrial, instrument_lxi_hislip, instrument_scpi_vna,
    iot_wire_protocol, iso7816_smartcard, math_time, motion_dead_reckon, motion_render_timeline, motor_stepper, net_ethernet,
    net_telemetry, net_tsn_macsec, office_calc_layout, packet_checksum, pcb_cad_fem, physics_raycast_3d, quant_finance_market, quant_orderbook_l2,
    radar_aoa_mti, radar_biomed_dsp, radar_cfar_doppler, radar_mimo_tracking, ring_buffer, robotics_6dof_slerp, robotics_motion_profile, rpa_screen_control, sensor_baro_als, sensor_hardware, sensor_motion,
    sensor_power_meter, spatial_physics, spc_msa_advanced, spc_six_sigma_msa, sys_path, text_cleaner, thermal_palette, thermal_pid,
    touch_control, vision_feature, vision_morphology_fast, vision_optical_flow_homography, vision_yuv,
};
use std::env;
use std::path::Path;
use std::process::{self, Command};

/// @outputs: 1
fn verify_edge_sensors_and_packets() -> bool {
    let fused = sensor_motion::complementary_filter(10.0, 1.0, 10.5, 0.1, 0.98);
    let ok_sensor = fused > 0.0 && sensor_motion::haversine_distance_m(25.033, 121.565, 25.034, 121.565).is_some();
    let nv21_mock = vec![128u8; 16 * 16 * 3 / 2];
    let ok_vision = vision_yuv::nv21_extract_grayscale(&nv21_mock, 16, 16).is_ok();
    let ok_chk = packet_checksum::crc32_ieee(b"123456789") == 0xcbf4_3926;
    let ok_rb = ring_buffer::RingBufferPcm16::with_capacity(8).is_ok();
    let ok_touch = touch_control::pressure_touch_level(0.5, 0.2, 0.8, 0) == 1;
    let ok_feat = vision_feature::otsu_threshold(&[10, 20, 200, 210]) > 0;
    let ok_bez = bezier_curve::cubic_bezier_point(0.0, 0.5, 0.5, 1.0, 0.5) > 0.0;
    let ok_hw = sensor_hardware::flashlight_pwm_duty(1000, 50, 100);
    let ok_net = net_telemetry::is_private_ipv4([192, 168, 1, 1]);
    ok_sensor && ok_vision && ok_chk && ok_rb && ok_touch && ok_feat && ok_bez && ok_hw && ok_net
}

/// @outputs: 1
fn verify_new_kinematics_and_math() -> bool {
    let ok_inv = (fast_math::fast_inv_sqrt_f32(4.0) - 0.5).abs() < 0.01;
    let ok_isqrt = fast_math::isqrt_u64(144) == 12;
    let ok_lerp = (fast_math::lerp_f32(10.0, 20.0, 0.5) - 15.0).abs() < 1e-5;
    let ok_zupt = motion_dead_reckon::zupt_zero_velocity_detect(0.01, 0.02, 0.1, 0.1);
    let (vel, pos) = motion_dead_reckon::dead_reckon_1d_step(2.0, 1.0, false, 0.0, 0.0);
    let ok_reckon = (vel - 2.0).abs() < 1e-5 && (pos - 1.0).abs() < 1e-5;
    let ok_haptic = motion_dead_reckon::haptic_vibration_timing(0, 10);
    let ok_box = geo_collision::bounding_box_overlap((0.0, 0.0), (10.0, 10.0), (5.0, 5.0), (15.0, 15.0));
    let ok_seg = geo_collision::segment_intersection_2d((0.0, 0.0), (2.0, 2.0), (0.0, 2.0), (2.0, 0.0)).is_some();
    let poly = [(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0)];
    let ok_poly = geo_collision::point_in_polygon_raycast(2.0, 2.0, &poly);
    ok_inv && ok_isqrt && ok_lerp && ok_zupt && ok_reckon && ok_haptic && ok_box && ok_seg && ok_poly
}

/// @outputs: 1
fn verify_security_and_automotive() -> bool {
    let ok_luhn = iso7816_smartcard::luhn_checksum_validate(b"49927398716");
    let ok_sw = iso7816_smartcard::iso7816_sw_status(0x90, 0x00) == 0;
    let mut pb = [0u8; 8];
    let ok_pb = iso7816_smartcard::iso9564_pin_block_format0(b"1234", b"1111222233334444", &mut pb);
    let (is_eff, cid) = can_bus_frame::can_id_unpack(0x123);
    let ok_can = !is_eff && cid == 0x123;
    let ok_fd = can_bus_frame::can_fd_dlc_to_len(15) == 64;
    let ok_modbus = can_bus_frame::crc16_modbus(&[0x01, 0x03, 0x00, 0x00, 0x00, 0x0A]) == 0xCDC5;
    let timer = can_bus_frame::timer_calc_prescaler_and_period(72_000_000, 1_000).is_some();
    ok_luhn && ok_sw && ok_pb && ok_can && ok_fd && ok_modbus && timer
}

/// @outputs: 1
fn verify_agent_and_physics_pid() -> bool {
    let q = (1.0_f32, 0.0, 0.0, 0.0);
    let v_rot = spatial_physics::quaternion_rotate_vector(q, (1.0, 2.0, 3.0));
    let ok_vr = (v_rot.0 - 1.0).abs() < 1e-4;
    let (out_pid, _) = thermal_pid::pid_controller_step(100.0, 50.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.1, 0.0, 100.0);
    let ok_pid = out_pid == 50.0;
    let ok_glob = agent_terminal::simple_glob_match("*.rs", "main.rs");
    let mut clean_buf = [0u8; 16];
    let n = agent_terminal::strip_ansi_codes("\x1b[31mhi\x1b[0m", &mut clean_buf);
    let ok_ansi = n == 2 && &clean_buf[..2] == b"hi";
    ok_vr && ok_pid && ok_glob && ok_ansi
}

/// @outputs: 1
fn verify_thermal_ble_dsp_crypto() -> bool {
    let rgb = thermal_palette::thermal_ironbow_rgb(100.0, 0.0, 100.0);
    let ok_thermal = rgb == (255, 255, 255);
    let ble_pkt = [0x02, 0x0A, 0xEC];
    let ok_ble = ble_adv_parser::ble_adv_parse_tx_power(&ble_pkt) == Some(-20);
    let ok_dsp = dsp_fixed_point::q15_multiply(16384, 16384) >= 8191;
    let mut padded = [0u8; 16];
    let ok_crypto = crypto_kdf::pkcs7_pad(b"test", 8, &mut padded) == Some(8);
    ok_thermal && ok_ble && ok_dsp && ok_crypto
}

/// @outputs: 1
fn verify_motor_dukpt_and_env() -> bool {
    let d = motor_stepper::stepper_trapezoid_delay_us(50, 100, 20, 20, 100, 500);
    let ok_motor = d == 100;
    let mut ksn = [0u8; 10];
    let ok_dukpt = crypto_dukpt::dukpt_ksn_increment_counter(&mut ksn);
    let ok_crc = env_sensor::sht3x_crc8(&[0xBE, 0xEF]) == 0x92;
    let ok_aqi = env_sensor::aqi_pm25_us_standard(12.0) == 50;
    ok_motor && ok_dukpt && ok_crc && ok_aqi
}

/// @outputs: 1
fn verify_comm_and_clock() -> bool {
    let ok_i2c = comm_controller::i2c_address_frame(0x3C, false) == 0x78;
    let ok_spi = comm_controller::spi_clock_mode(true, true) == 3;
    let pll = clock_pll_rtc::pll_calculate_prescalers(8_000_000, 72_000_000);
    let ok_pll = pll.is_some();
    let ok_ntp = clock_pll_rtc::ntp_timestamp_to_unix_secs(3_900_000_000) == 1_691_011_200;
    ok_i2c && ok_spi && ok_pll && ok_ntp
}

/// @outputs: 1
fn verify_ethernet_and_apdu() -> bool {
    let mac = [0xFF; 6];
    let ok_mac = net_ethernet::is_broadcast_mac(&mac);
    let mut apdu_buf = [0u8; 16];
    let ok_apdu = fin_apdu_track::apdu_encode_command(0, 0xA4, 4, 0, &[], None, &mut apdu_buf).is_some();
    let ok_lrc = fin_apdu_track::lrc_checksum(b"A") == b'A';
    ok_mac && ok_apdu && ok_lrc
}

/// @outputs: 1
fn verify_power_and_baro() -> bool {
    let amps = sensor_power_meter::shunt_voltage_to_current_amperes(0.1, 0.1);
    let ok_amps = (amps - 1.0).abs() < 1e-4;
    let alt = sensor_baro_als::baro_pressure_to_altitude_m(1013.25, 1013.25);
    let ok_alt = matches!(alt, Some(a) if a.abs() < 0.1);
    let ok_lux = sensor_baro_als::bh1750_raw_to_lux(120, 69) > 0.0;
    ok_amps && ok_alt && ok_lux
}

/// @outputs: 1
fn verify_lin_and_raycast_3d() -> bool {
    let pid = can_lin_bus::lin_calc_pid(0x01);
    let ok_pid = pid == 0xC1;
    let refl = physics_raycast_3d::vector3_reflect((1.0, -1.0, 0.0), (0.0, 1.0, 0.0));
    let ok_refl = (refl.1 - 1.0).abs() < 1e-4;
    ok_pid && ok_refl
}

/// @outputs: 1
fn verify_tsn_and_macsec() -> bool {
    let ok_ptp = net_tsn_macsec::ptp_is_event_message(0);
    let ok_replay = net_tsn_macsec::macsec_replay_window_validate(10, 11, 4);
    ok_ptp && ok_replay
}

/// @outputs: 1
fn verify_ai_tensor_train() -> bool {
    let mut probs = [0.0f32; 2];
    let ok_sm = ai_tensor_train::softmax_stable(&[0.0, 0.0], &mut probs) && (probs[0] - 0.5).abs() < 1e-4;
    let (rx, ry) = ai_tensor_train::rope_rotary_embedding_step((1.0, 0.0), 0.0);
    let ok_rope = (rx - 1.0).abs() < 1e-4 && ry.abs() < 1e-4;
    let ok_gelu = (ai_tensor_train::gelu_forward(0.0) - 0.0).abs() < 1e-4;
    ok_sm && ok_rope && ok_gelu
}

/// @outputs: 1
fn verify_instrument_scpi_vna() -> bool {
    let ok_scpi = matches!(instrument_scpi_vna::scpi_parse_command(b"*IDN?"), Some((cmd, true)) if cmd == b"*IDN");
    let (gr, gi) = instrument_scpi_vna::vna_s11_to_complex(-20.0, 0.0);
    let ok_vna = (gr - 0.1).abs() < 0.01 && gi.abs() < 1e-4;
    let ok_vswr = matches!(instrument_scpi_vna::vna_calc_vswr_and_return_loss(0.1), Some((v, r)) if v > 1.2 && r > 19.0);
    ok_scpi && ok_vna && ok_vswr
}

/// @outputs: 1
fn verify_pcb_cad_fem() -> bool {
    let z0 = pcb_cad_fem::microstrip_impedance_hammerstad(3.0, 1.6, 4.4);
    let ok_z0 = matches!(z0, Some((z, _)) if z > 40.0 && z < 60.0);
    let dt = pcb_cad_fem::openems_fdtd_cfl_max_timestep_sec(1e-3, 1e-3, 1e-3);
    let ok_dt = matches!(dt, Some(t) if t > 1e-12);
    let ok_gerber = pcb_cad_fem::gerber_coord_to_microns(b"100000", 4) == Some(254000);
    ok_z0 && ok_dt && ok_gerber
}

/// @outputs: 1
fn verify_motion_render_timeline() -> bool {
    let ms = motion_render_timeline::remotion_frame_to_ms(30, 30, 1);
    let ok_ms = ms == 1000;
    let mut tc = [0u8; 11];
    let ok_tc = motion_render_timeline::smpte_timecode_format(30, 30, &mut tc) && &tc == b"00:00:01:00";
    let bez = motion_render_timeline::css_cubic_bezier_solve(0.25, 0.1, 0.25, 1.0, 0.5);
    let ok_bez = bez > 0.3 && bez < 0.9;
    ok_ms && ok_tc && ok_bez
}

/// @outputs: 1
fn verify_office_calc_layout() -> bool {
    let pmt = office_calc_layout::excel_pmt_loan(0.05 / 12.0, 360, 100000.0);
    let ok_pmt = matches!(pmt, Some(p) if p > 500.0 && p < 600.0);
    let twips = office_calc_layout::office_units_pt_to_twips(12.0);
    let ok_twips = twips == 240;
    let lines = office_calc_layout::word_greedy_wrap_line_count(&[40, 40, 40], 50, 10);
    let ok_wrap = lines == 3;
    ok_pmt && ok_twips && ok_wrap
}

/// @outputs: 1
fn verify_rpa_screen_control() -> bool {
    let diff = rpa_screen_control::rpa_color_diff_rgb((255, 0, 0), (250, 0, 0));
    let ok_diff = diff < 15;
    let (mx, my) = rpa_screen_control::rpa_mouse_human_trajectory_point((0.0, 0.0), (100.0, 100.0), (50.0, 0.0), 0.5);
    let ok_mouse = (mx - 50.0).abs() < 1e-3 && (my - 25.0).abs() < 1e-3;
    let tx_us = rpa_screen_control::rs485_calc_char_tx_time_us(115200, 8, 1, false);
    let ok_rs = tx_us > 80 && tx_us < 95;
    ok_diff && ok_mouse && ok_rs
}

/// @outputs: 1
fn verify_radar_cfar_doppler() -> bool {
    let vel = radar_cfar_doppler::radar_doppler_velocity_mps(1000.0, 77e9);
    let ok_vel = matches!(vel, Some(v) if v > 1.8 && v < 2.2);
    let disp = radar_cfar_doppler::radar_micro_doppler_chest_displacement_mm(0.1, 60.0);
    let ok_disp = matches!(disp, Some(d) if d > 0.03 && d < 0.05);
    ok_vel && ok_disp
}

/// @outputs: 1
fn verify_dsp_fft_radix2() -> bool {
    let mut real = [1.0f32, 0.0, 1.0, 0.0];
    let mut imag = [0.0f32; 4];
    let ok_fft = dsp_fft_radix2::fft_radix2_f32_inplace(&mut real, &mut imag);
    let ok_dc = (real[0] - 2.0).abs() < 1e-4;
    ok_fft && ok_dc
}

/// @outputs: 1
fn verify_decision_mcts() -> bool {
    let ucb = decision_mcts::mcts_ucb1_score(5.0, 10, 100, 1.414);
    let ok_ucb = ucb > 0.5 && ucb.is_finite();
    let best = decision_mcts::mcts_select_best_child(&[1.0, 5.0], &[10, 10], 100, 1.414);
    let ok_best = best == Some(1);
    ok_ucb && ok_best
}

/// @outputs: 1
fn verify_curve_fit_trend() -> bool {
    let res = curve_fit_trend::curve_linear_regression_slope_intercept(&[1.0, 2.0, 3.0], &[2.0, 4.0, 6.0]);
    let ok_reg = matches!(res, Some((m, c, r)) if (m - 2.0).abs() < 1e-4 && c.abs() < 1e-4 && (r - 1.0).abs() < 1e-4);
    let w = curve_fit_trend::curve_lowess_tricube_weight(0.0);
    let ok_w = (w - 1.0).abs() < 1e-4;
    let silu = curve_fit_trend::curve_silu_activation(0.0);
    let ok_silu = silu.abs() < 1e-4;
    ok_reg && ok_w && ok_silu
}

/// @outputs: 1
fn verify_ai_autograd_train() -> bool {
    let (ga, gb) = ai_autograd_train::autograd_scalar_add(2.0, 3.0, 1.0);
    let ok_grad = (ga - 1.0).abs() < 1e-4 && (gb - 1.0).abs() < 1e-4;
    let lr = ai_autograd_train::lr_schedule_cosine_warmup(100, 10, 100, 0.01, 0.001);
    let ok_lr = (lr - 0.001).abs() < 1e-4;
    let lse = ai_autograd_train::log_sum_exp_f32(&[0.0, 0.0]);
    let ok_lse = matches!(lse, Some(v) if (v - std::f32::consts::LN_2).abs() < 1e-4);
    ok_grad && ok_lr && ok_lse
}

/// @outputs: 1
fn verify_comm_phy_bus() -> bool {
    let hdr = comm_phy_bus::mipi_csi2_short_packet_header(0x24, 1, 0x00FF);
    let ok_mipi = hdr[0] == ((1 << 6) | 0x24);
    let taps = comm_phy_bus::rgmii_tx_delay_skew_taps(1500, 50);
    let ok_rgmii = taps == Some(30);
    let sgmii = comm_phy_bus::sgmii_auto_negotiation_advertisement(true, true, 2);
    let ok_sgmii = (sgmii & (1 << 15)) != 0 && (sgmii & 1) == 1;
    ok_mipi && ok_rgmii && ok_sgmii
}

/// @outputs: 1
fn verify_radar_biomed_dsp() -> bool {
    let res = radar_biomed_dsp::radar_fmcw_range_resolution_meters(1.0e9);
    let ok_res = matches!(res, Some(r) if (r - 0.149896).abs() < 1e-4);
    let unwrap = radar_biomed_dsp::radar_phase_unwrap_step(0.1, 0.0);
    let ok_unwrap = (unwrap - 0.1).abs() < 1e-4;
    let q7 = radar_biomed_dsp::dsp_quantize_f32_to_q7(1.0);
    let ok_q7 = q7 == 127;
    ok_res && ok_unwrap && ok_q7
}

/// @outputs: 1
fn verify_instrument_industrial() -> bool {
    let val = instrument_industrial::scpi_parse_scientific_float(b"+1.23456E+02");
    let ok_scpi = matches!(val, Some(v) if (v - 123.456).abs() < 1e-3);
    let lrc = instrument_industrial::modbus_ascii_compute_lrc(&[0x01, 0x03, 0x00, 0x01]);
    let ok_lrc = lrc != 0;
    let sweep = instrument_industrial::litevna_frequency_sweep_step(100_000, 1_000_000, 10, 0);
    let ok_sweep = sweep == Some(100_000);
    ok_scpi && ok_lrc && ok_sweep
}

/// @outputs: 1
fn verify_cad_geom_model() -> bool {
    let ring = cad_geom_model::kicad_via_annular_ring_width_mm(0.3, 0.6);
    let ok_ring = matches!(ring, Some(w) if (w - 0.15).abs() < 1e-4);
    let vol = cad_geom_model::freecad_extrude_cylinder_volume_mm3(1.0, 10.0);
    let ok_vol = matches!(vol, Some(v) if (v - 10.0 * std::f64::consts::PI).abs() < 1e-3);
    let zeta = cad_geom_model::remotion_spring_damping_ratio(1.0, 100.0, 20.0);
    let ok_zeta = matches!(zeta, Some(z) if (z - 1.0).abs() < 1e-4);
    ok_ring && ok_vol && ok_zeta
}

/// @outputs: 1
fn verify_spc_six_sigma_msa() -> bool {
    let cpk_res = spc_six_sigma_msa::spc_cpk_index(14.0, 6.0, 10.0, 1.0);
    let ok_cpk = matches!(cpk_res, Some((cpk, _, _)) if (cpk - 1.3333).abs() < 1e-3);
    let phi = spc_six_sigma_msa::spc_normal_cdf_phi(0.0);
    let ok_phi = (phi - 0.5).abs() < 1e-4;
    let grr = spc_six_sigma_msa::msa_gage_rr_ev_av(3.0, 4.0);
    let ok_grr = matches!(grr, Some(v) if (v - 5.0).abs() < 1e-4);
    ok_cpk && ok_phi && ok_grr
}

/// @outputs: 1
fn verify_quant_finance_market() -> bool {
    let ema = quant_finance_market::quant_ema_step(10.0, 12.0, 9);
    let ok_ema = matches!(ema, Some(v) if (v - 10.4).abs() < 1e-3);
    let rsi = quant_finance_market::quant_rsi_step(2.0, 1.0);
    let ok_rsi = matches!(rsi, Some(v) if (v - 66.666).abs() < 1e-2);
    let (basis, is_contango) = quant_finance_market::quant_futures_basis_spread(105.0, 100.0);
    let ok_fut = (basis - 5.0).abs() < 1e-4 && is_contango;
    ok_ema && ok_rsi && ok_fut
}

/// @outputs: 1
fn verify_robotics_motion_profile() -> bool {
    let ik = robotics_motion_profile::robotics_planar_2link_ik(1.0, 1.0, 1.0, 1.0, false);
    let ok_ik = ik.is_some();
    let d = robotics_motion_profile::robotics_trapezoidal_profile_distance(1.0, 10.0, 10.0, 100.0);
    let ok_d = matches!(d, Some(v) if (v - 5.0).abs() < 1e-3);
    let (s, c) = robotics_motion_profile::robotics_stepper_microstep_sin_cos(0, 16).unwrap_or((0, 0));
    let ok_step = s == 0 && c == 32767;
    ok_ik && ok_d && ok_step
}

/// @outputs: 1
fn verify_ai_tensor_gemm() -> bool {
    let a = [1.0f32, 2.0, 3.0, 4.0]; // 2x2
    let b = [1.0f32, 0.0, 0.0, 1.0]; // 2x2 I
    let mut c = [0.0f32; 4];
    let ok_gemm = ai_tensor_gemm::gemm_tiled_f32_step(&a, &b, &mut c, 2, 2, 2, 1);
    let ok_res = (c[0] - 1.0).abs() < 1e-4 && (c[3] - 4.0).abs() < 1e-4;
    let cos_sim = ai_tensor_gemm::cosine_similarity_f32(&[1.0, 0.0], &[1.0, 0.0]);
    let ok_cos = matches!(cos_sim, Some(v) if (v - 1.0).abs() < 1e-4);
    ok_gemm && ok_res && ok_cos
}

/// @outputs: 1
fn verify_iot_wire_protocol() -> bool {
    let mut mqtt_buf = [0u8; 4];
    let enc_len = iot_wire_protocol::mqtt_remaining_length_encode(321, &mut mqtt_buf);
    let ok_mqtt = enc_len == 2 && iot_wire_protocol::mqtt_remaining_length_decode(&mqtt_buf) == Some((321, 2));
    let mut coap_hdr = [0u8; 4];
    let ok_coap = iot_wire_protocol::coap_header_pack(0, 1, 0x1234, &mut coap_hdr) && coap_hdr[2] == 0x12 && coap_hdr[3] == 0x34;
    let mut slip_buf = [0u8; 10];
    let slip_len = iot_wire_protocol::slip_encode_packet(&[0xC0, 0x01], &mut slip_buf);
    let mut dec_buf = [0u8; 10];
    let dec_len = iot_wire_protocol::slip_decode_packet(&slip_buf[..slip_len], &mut dec_buf);
    let ok_slip = dec_len == Some(2) && dec_buf[0] == 0xC0 && dec_buf[1] == 0x01;
    ok_mqtt && ok_coap && ok_slip
}

/// @outputs: 1
fn verify_radar_aoa_mti() -> bool {
    let mti2 = radar_aoa_mti::radar_mti_two_pulse_canceler_step(10.0, 10.0);
    let ok_mti = mti2.abs() < 1e-6;
    let aoa = radar_aoa_mti::radar_aoa_phase_monopulse(0.0, 0.0039, 0.00195);
    let ok_aoa = matches!(aoa, Some(th) if th.abs() < 1e-4);
    let v_blind = radar_aoa_mti::radar_blind_speed_mps(0.004, 10000.0, 1);
    let ok_blind = matches!(v_blind, Some(v) if (v - 20.0).abs() < 1e-3);
    ok_mti && ok_aoa && ok_blind
}

/// @outputs: 1
fn verify_vision_morphology_fast() -> bool {
    let iou = vision_morphology_fast::vision_bbox_intersection_over_union([0, 0, 10, 10], [0, 0, 10, 10]);
    let ok_iou = (iou - 1.0).abs() < 1e-4;
    let (h, s, v) = vision_morphology_fast::vision_rgb_to_hsv_step(255, 0, 0);
    let ok_hsv = h == 0 && s == 255 && v == 255;
    let ok_in_range = vision_morphology_fast::vision_color_hsv_in_range(h, s, v, 350, 10, 200, 200);
    ok_iou && ok_hsv && ok_in_range
}

/// @outputs: 1
fn verify_audio_mfcc_pitch() -> bool {
    let mel = audio_mfcc_pitch::audio_hz_to_mel(1000.0);
    let ok_mel = (mel - 999.98).abs() < 50.0;
    let hz = audio_mfcc_pitch::audio_mel_to_hz(mel);
    let ok_hz = (hz - 1000.0).abs() < 50.0;
    let w = audio_mfcc_pitch::audio_mel_filterbank_weights(100.0, 50.0, 150.0, 100.0);
    let ok_w = (w - 1.0).abs() < 1e-4;
    ok_mel && ok_hz && ok_w
}

/// @outputs: 1
fn verify_instrument_lxi_hislip() -> bool {
    let mut xdr_buf = [0u8; 8];
    let ok_xdr = instrument_lxi_hislip::xdr_encode_u32(0x12345678, &mut xdr_buf) == 4
        && instrument_lxi_hislip::xdr_decode_u32(&xdr_buf) == Some((0x12345678, 4));
    let mut hislip_buf = [0u8; 16];
    let ok_hislip = instrument_lxi_hislip::hislip_packet_header_pack(1, 0, 100, 2048, &mut hislip_buf) == 16
        && instrument_lxi_hislip::hislip_packet_header_unpack(&hislip_buf) == Some((1, 0, 100, 2048));
    ok_xdr && ok_hislip
}

/// @outputs: 1
fn verify_spc_msa_advanced() -> bool {
    let pp = spc_msa_advanced::spc_pp_ppk_performance(10.0, 1.0, 13.0, 7.0);
    let ok_pp = matches!(pp, Some((p, pk)) if (p - 1.0).abs() < 1e-3 && (pk - 1.0).abs() < 1e-3);
    let r = spc_msa_advanced::spc_weibull_reliability_step(100.0, 100.0, 1.0);
    let ok_r = matches!(r, Some(v) if (v - 0.3678).abs() < 1e-2);
    ok_pp && ok_r
}

/// @outputs: 1
fn verify_fieldbus_industrial_eth() -> bool {
    let mut ec_buf = [0u8; 4];
    let ok_ec = fieldbus_industrial_eth::ethercat_frame_header_pack(100, &mut ec_buf) == 2
        && (ec_buf[0] as u16 | ((ec_buf[1] as u16) << 8)) == (100 | (1 << 12));
    let mut mbap_buf = [0u8; 8];
    let ok_mbap = fieldbus_industrial_eth::modbus_tcp_mbap_header_pack(1, 2, 5, &mut mbap_buf) == 7
        && fieldbus_industrial_eth::modbus_tcp_mbap_header_unpack(&mbap_buf) == Some((1, 2, 5));
    ok_ec && ok_mbap
}

/// @outputs: 1
fn verify_robotics_6dof_slerp() -> bool {
    let p_wc = robotics_6dof_slerp::robotics_wrist_center_decouple([0.0, 0.0, 10.0], &[1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0], 2.0);
    let ok_wc = (p_wc[2] - 8.0).abs() < 1e-4;
    let (s, v, a) = robotics_6dof_slerp::robotics_quintic_polynomial_traj(5.0, 10.0, 0.0, 100.0);
    let ok_traj = (s - 50.0).abs() < 1e-2 && v > 0.0 && a.abs() < 1e-2;
    ok_wc && ok_traj
}

/// @outputs: 1
fn verify_crypto_modern_wire() -> bool {
    let (mut a, mut b, mut c, mut d) = (0x11111111u32, 0x22222222u32, 0x33333333u32, 0x44444444u32);
    crypto_modern_wire::chacha20_quarter_round(&mut a, &mut b, &mut c, &mut d);
    let ok_qr = a != 0x11111111;
    let mut x = 10u32;
    let mut y = 20u32;
    crypto_modern_wire::constant_time_cswap_u32(&mut x, &mut y, true);
    let ok_swap = x == 20 && y == 10;
    let gf_res = crypto_modern_wire::gf256_mul_step(0x57, 0x13);
    let ok_gf = gf_res == 0xFE;
    ok_qr && ok_swap && ok_gf
}

/// @outputs: 1
fn verify_vision_optical_flow_homography() -> bool {
    let (ix2, iy2, ixiy) = vision_optical_flow_homography::vision_harris_structure_tensor_step(3, 4);
    let ok_tensor = ix2 == 9 && iy2 == 16 && ixiy == 12;
    let lk = vision_optical_flow_homography::vision_lucas_kanade_2x2_solve(2.0, 2.0, 0.0, 4.0, 6.0);
    let ok_lk = matches!(lk, Some((u, v)) if (u - (-2.0)).abs() < 1e-4 && (v - (-3.0)).abs() < 1e-4);
    let h_eye = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
    let proj = vision_optical_flow_homography::vision_homography_project_point_2d(&h_eye, 5.0, 7.0);
    let ok_proj = matches!(proj, Some((x, y)) if (x - 5.0).abs() < 1e-4 && (y - 7.0).abs() < 1e-4);
    ok_tensor && ok_lk && ok_proj
}

/// @outputs: 1
fn verify_gnss_nmea_geodesy() -> bool {
    let nmea_sent = b"$GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,*47";
    let chk = gnss_nmea_geodesy::gnss_nmea_checksum(nmea_sent);
    let ok_chk = chk.is_some();
    let lat = gnss_nmea_geodesy::gnss_nmea_lat_lon_to_decimal_deg(4807.038, b'N');
    let ok_lat = matches!(lat, Some(v) if (v - 48.1173).abs() < 1e-3);
    let ecef = gnss_nmea_geodesy::gnss_wgs84_llh_to_ecef(0.0, 0.0, 0.0);
    let ok_ecef = matches!(ecef, Some([x, _, _]) if (x - 6378137.0).abs() < 1.0);
    ok_chk && ok_lat && ok_ecef
}

/// @outputs: 1
fn verify_quant_orderbook_l2() -> bool {
    let bbo = quant_orderbook_l2::quant_l2_bbo_mid_and_spread(100.0, 102.0);
    let ok_bbo = matches!(bbo, Some((mid, spread, bps)) if (mid - 101.0).abs() < 1e-4 && (spread - 2.0).abs() < 1e-4 && bps > 0.0);
    let obi = quant_orderbook_l2::quant_l2_order_book_imbalance_obi(300.0, 100.0);
    let ok_obi = matches!(obi, Some(v) if (v - 0.5).abs() < 1e-4);
    let micro = quant_orderbook_l2::quant_l2_weighted_micro_price(100.0, 102.0, 300.0, 100.0);
    let ok_micro = matches!(micro, Some(p) if (p - 101.5).abs() < 1e-4);
    ok_bbo && ok_obi && ok_micro
}

/// @outputs: 1
fn verify_doc_layout_typography() -> bool {
    let twips = doc_layout_typography::doc_pt_to_twips(12.0);
    let ok_twips = twips == 240 && doc_layout_typography::doc_twips_to_pt(240) == 12.0;
    let badness = doc_layout_typography::doc_knuth_plass_badness(500.0, 480.0);
    let ok_badness = badness < 100;
    let (tx, ty, tw, th) = doc_layout_typography::doc_ppt_slide_fit_aspect_ratio(4.0, 3.0, 1920.0, 1080.0);
    let ok_ppt = tw > 0.0 && th > 0.0 && tx >= 0.0 && ty >= 0.0;
    ok_twips && ok_badness && ok_ppt
}

/// @outputs: 1
fn verify_radar_mimo_tracking() -> bool {
    let v_idx = radar_mimo_tracking::radar_mimo_virtual_array_index(2, 3, 4);
    let ok_idx = v_idx == 11;
    let b7 = radar_mimo_tracking::radar_mimo_barker_code_biphase(7, 0);
    let ok_b7 = b7 == Some(1.0);
    let (up_x, up_p, k) = radar_mimo_tracking::radar_kalman_filter_innovation_step(10.0, 8.0, 4.0, 1.0).unwrap_or((0.0, 0.0, 0.0));
    let ok_kf = (up_x - 9.6).abs() < 1e-3 && (up_p - 0.8).abs() < 1e-3 && (k - 0.8).abs() < 1e-3;
    ok_idx && ok_b7 && ok_kf
}

/// @outputs: 1
fn verify_all_subsystems() -> bool {
    verify_edge_sensors_and_packets()
        && verify_new_kinematics_and_math()
        && verify_security_and_automotive()
        && verify_agent_and_physics_pid()
        && verify_thermal_ble_dsp_crypto()
        && verify_motor_dukpt_and_env()
        && verify_comm_and_clock()
        && verify_ethernet_and_apdu()
        && verify_power_and_baro()
        && verify_lin_and_raycast_3d()
        && verify_tsn_and_macsec()
        && verify_ai_tensor_train()
        && verify_instrument_scpi_vna()
        && verify_pcb_cad_fem()
        && verify_motion_render_timeline()
        && verify_office_calc_layout()
        && verify_rpa_screen_control()
        && verify_radar_cfar_doppler()
        && verify_dsp_fft_radix2()
        && verify_decision_mcts()
        && verify_curve_fit_trend()
        && verify_ai_autograd_train()
        && verify_comm_phy_bus()
        && verify_radar_biomed_dsp()
        && verify_instrument_industrial()
        && verify_cad_geom_model()
        && verify_spc_six_sigma_msa()
        && verify_quant_finance_market()
        && verify_robotics_motion_profile()
        && verify_ai_tensor_gemm()
        && verify_iot_wire_protocol()
        && verify_radar_aoa_mti()
        && verify_vision_morphology_fast()
        && verify_audio_mfcc_pitch()
        && verify_instrument_lxi_hislip()
        && verify_spc_msa_advanced()
        && verify_fieldbus_industrial_eth()
        && verify_robotics_6dof_slerp()
        && verify_crypto_modern_wire()
        && verify_vision_optical_flow_homography()
        && verify_gnss_nmea_geodesy()
        && verify_quant_orderbook_l2()
        && verify_doc_layout_typography()
        && verify_radar_mimo_tracking()
}

/// @outputs: 1
fn run_selftest() {
    println!("🚀 [rust-bricks] Running Zero-Dependency Contract Self-Test...");

    if math_time::clamp_i64(150, 0, 100) != 100 || math_time::julian_day_from_ymd(2026, 10, 26).is_none() {
        println!("  ❌ math_time: FAIL");
        process::exit(1);
    }
    println!("  ✅ math_time: PASS");

    let pcm = vec![1000i16; 480];
    if audio_vad::pcm16_rms_energy(&pcm) != 1000 || audio_vad::cps_chars_per_second(15, 3000) != Some(5.0) {
        println!("  ❌ audio_vad: FAIL");
        process::exit(1);
    }
    println!("  ✅ audio_vad: PASS");

    let hash = crypto_vault::sha256_hex(b"rust-bricks");
    if hash.is_empty() || !crypto_vault::constant_time_eq(b"key", b"key") {
        println!("  ❌ crypto_vault: FAIL");
        process::exit(1);
    }
    println!("  ✅ crypto_vault: PASS");

    if !sys_path::is_safe_rel_path("crates/my_brick.rs") || sys_path::is_safe_rel_path("../bad/path") {
        println!("  ❌ sys_path: FAIL");
        process::exit(1);
    }
    println!("  ✅ sys_path: PASS");

    let truncated = text_cleaner::truncate_at_boundary("梵語專有名詞", 6);
    if truncated != "梵語" {
        println!("  ❌ text_cleaner: FAIL");
        process::exit(1);
    }
    println!("  ✅ text_cleaner: PASS");

    if !verify_all_subsystems() {
        println!("  ❌ edge_and_embedded_subsystems: FAIL");
        process::exit(1);
    }
    println!("  ✅ edge_and_embedded_subsystems: PASS");

    println!("✨ [rust-bricks] All core brick modules verified successfully! 100% PASS.");
}

/// @outputs: 1
fn run_catalog_cmd() {
    println!("📖 [rust-bricks] Scanning source files & generating machine catalogs...");
    let repo_root = Path::new(".");
    let res = catalog::generate_catalogs(repo_root);
    match res {
        Ok(count) => {
            println!("✨ [rust-bricks] Successfully generated CATALOG.yaml and CATALOG.json! Total {} functions indexed.", count);
        }
        Err(e) => {
            eprintln!("❌ [rust-bricks] Catalog generation failed: {}", e);
            process::exit(1);
        }
    }
}

/// @outputs: 1
fn run_guard_cmd() {
    println!("🛡️ [rust-bricks] Running Pure Rust Capacity & Contract Guard (brick-guard)...");
    let repo_root = Path::new(".");
    let res = guard::run_guard_checks(repo_root);
    match res {
        Ok(_) => {
            println!("✅ [rust-bricks] Guard checks passed! Zero dependencies, zero bloat, 09-compliant, all tests green.");
        }
        Err(issues) => {
            eprintln!("\n❌ [rust-bricks] Guard checks failed:");
            for issue in issues {
                eprintln!("  - {}", issue);
            }
            process::exit(1);
        }
    }
}

/// @outputs: 1
fn run_receipt_cmd(model: &str, verdict: &str) {
    println!("🔏 [rust-bricks] Generating Diff-bound Arbitration Receipt with [{}]...", model);
    let repo_root = Path::new(".");
    match rust_bricks::receipt::create_receipt(repo_root, model, verdict) {
        Ok(hash) => {
            println!("✅ [rust-bricks] Arbitration Receipt successfully written to .ai-memory/receipt.json");
            println!("  SHA-256 Diff Fingerprint: {}", hash);
        }
        Err(e) => {
            eprintln!("❌ [rust-bricks] Failed to create receipt: {}", e);
            process::exit(1);
        }
    }
}

/// @outputs: 1
fn run_verify_receipt_cmd() {
    println!("🔍 [rust-bricks] Verifying Local Arbitration Receipt against current git diff...");
    let repo_root = Path::new(".");
    match rust_bricks::receipt::verify_receipt(repo_root) {
        Ok(msg) => {
            println!("✅ [rust-bricks] Receipt verification passed! {}", msg);
        }
        Err(e) => {
            eprintln!("❌ [rust-bricks] Receipt verification failed: {}", e);
            process::exit(1);
        }
    }
}

/// @outputs: 1
fn run_audit_cmd() {
    println!("🕵️ [rust-bricks] Running Zero-Trust Quality Director Chaos Audit (Black-Box)...");
    match rust_bricks::chaos_audit::run_chaos_audit(0) {
        Ok(summary) => {
            println!("✨ [rust-bricks] 零信任黑箱巡檢完成！共抽查 {} 項極限反向輸入，全部安全收斂 (0 崩潰，100% 通過)！", summary.total_inspections);
        }
        Err(issues) => {
            eprintln!("\n🚨 [rust-bricks] 品質總監零信任黑箱抽查發現嚴重隱患：");
            for issue in issues {
                eprintln!("  - {}", issue);
            }
            process::exit(1);
        }
    }
}

/// @outputs: 1
fn run_patrol_cmd() {
    println!("🚨 [rust-bricks] Running Remote Branch Patrol & Collision Check...");
    let _ = Command::new("git").args(["fetch", "--all"]).output();
    let branches_out = Command::new("git").args(["branch", "-r"]).output();
    let stdout = match branches_out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(e) => {
            eprintln!("❌ [rust-bricks] git branch -r 失敗: {}", e);
            return;
        }
    };
    let mut contrib_branches = Vec::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.contains("contrib/") {
            contrib_branches.push(trimmed.to_string());
        }
    }
    if contrib_branches.is_empty() {
        println!("✨ [rust-bricks] 巡檢完成：無待處理之外部 contrib 分支，主分支保持純淨。");
    } else {
        println!("⚠️ [rust-bricks] 發現外部貢獻分支待檢驗整合：");
        for b in contrib_branches {
            println!("  - {}", b);
        }
        println!("請檢出並實施硬體級五項體檢：git checkout -b review-<name> <branch>");
    }
}

/// @outputs: 1
fn run_wish_cmd(args: &[String]) {
    let sub = args.get(2).map(|s| s.as_str()).unwrap_or("list");
    let wishes_dir = Path::new("wishes");
    if sub == "new" {
        let id = args.get(3).map(|s| s.as_str()).unwrap_or("WISH-NEW");
        let title = args.get(4).map(|s| s.as_str()).unwrap_or("未命名需求");
        let req = args.get(5).map(|s| s.as_str()).unwrap_or("用戶/代理");
        match rust_bricks::wish_board::create_wish_template(wishes_dir, id, title, req) {
            Ok(path) => println!("✨ [rust-bricks] 成功建立許願單模板：{}", path.display()),
            Err(e) => eprintln!("❌ [rust-bricks] 建立許願單失敗: {}", e),
        }
        return;
    }
    println!("🌟 [rust-bricks] 掃描許願池留言板 (wishes/)...");
    match rust_bricks::wish_board::list_wishes(wishes_dir, 100) {
        Ok(wishes) => {
            if wishes.is_empty() {
                println!("✨ 許願池目前為空，歡迎外部專案 AI 代理或用戶許願！(指令: cargo run -- wish new <id> <title> <requester>)");
            } else {
                println!("📋 目前許願單清單（共 {} 項）：", wishes.len());
                for w in wishes {
                    println!("  [{}] {} — 申請者: {} (檔案: {})", w.status, w.title, w.requester, w.file_name);
                }
            }
        }
        Err(e) => eprintln!("❌ [rust-bricks] 讀取許願池失敗: {}", e),
    }
}

/// @outputs: 1
fn print_help() {
    println!("rust-bricks v0.1.0 — Pure Zero-Dependency Function Blocks (09 Contract)");
    println!("Usage:");
    println!("  cargo run -- selftest          # Run internal contract self-test");
    println!("  cargo run -- catalog           # Generate CATALOG.yaml & CATALOG.json (Pure Rust)");
    println!("  cargo run -- guard             # Run capacity & contract guard hook (Pure Rust)");
    println!("  cargo run -- patrol            # Patrol external contrib branches & collisions");
    println!("  cargo run -- audit             # Run Zero-Trust Quality Director Chaos Audit");
    println!("  cargo run -- wish [list|new]   # Wishlist board for AI agents & users");
    println!("  cargo run -- receipt <model>   # Sign Diff-bound Arbitration Receipt");
    println!("  cargo run -- verify-receipt    # Verify Receipt against current Git Diff");
    println!("  cargo test                     # Run full unit test suite");
}

/// @outputs: 1
fn main() {
    let args: Vec<String> = env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match cmd {
        "selftest" | "test" => run_selftest(),
        "catalog" => run_catalog_cmd(),
        "guard" => run_guard_cmd(),
        "patrol" => run_patrol_cmd(),
        "audit" | "chaos" => run_audit_cmd(),
        "wish" | "wishes" => run_wish_cmd(&args),
        "receipt" => {
            let model = args.get(2).map(|s| s.as_str()).unwrap_or("kev-4b");
            let verdict = args.get(3).map(|s| s.as_str()).unwrap_or("PASS");
            run_receipt_cmd(model, verdict);
        }
        "verify-receipt" => run_verify_receipt_cmd(),
        _ => print_help(),
    }
}
