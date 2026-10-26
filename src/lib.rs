// ## Map
// Role: core/lib — rust-bricks 純淨函式積木庫總導出入口。
// Inputs: 子模組 (agent_terminal, ai_tensor_train, audio_dsp, audio_vad, audio_wav, bezier_curve, ble_adv_parser, can_bus_frame, can_lin_bus, catalog, chaos_audit, clock_pll_rtc, comm_controller, crypto_dukpt, crypto_kdf, crypto_vault, dsp_fixed_point, env_sensor, fast_math, fin_apdu_track, geo_collision, guard, instrument_scpi_vna, iso7816_smartcard, math_time, motion_dead_reckon, motion_render_timeline, motor_stepper, net_ethernet, net_telemetry, net_tsn_macsec, office_calc_layout, packet_checksum, pcb_cad_fem, physics_raycast_3d, receipt, ring_buffer, rpa_screen_control, sensor_baro_als, sensor_hardware, sensor_motion, sensor_power_meter, spatial_physics, sys_path, text_cleaner, thermal_palette, thermal_pid, touch_control, vision_feature, vision_yuv, wish_board)。
// Outputs: 導出安全、有界、零依賴之純函式介面。
// Invariants: 零全域可變狀態、禁止 unsafe 代碼、100% 符合 09 號合約。

#![forbid(unsafe_code)]

pub mod agent_terminal;
pub mod ai_tensor_train;
pub mod audio_dsp;
pub mod audio_vad;
pub mod audio_wav;
pub mod bezier_curve;
pub mod ble_adv_parser;
pub mod can_bus_frame;
pub mod can_lin_bus;
pub mod catalog;
pub mod chaos_audit;
pub mod clock_pll_rtc;
pub mod comm_controller;
pub mod crypto_dukpt;
pub mod crypto_kdf;
pub mod crypto_vault;
pub mod dsp_fixed_point;
pub mod env_sensor;
pub mod fast_math;
pub mod fin_apdu_track;
pub mod geo_collision;
pub mod guard;
pub mod instrument_scpi_vna;
pub mod iso7816_smartcard;
pub mod math_time;
pub mod motion_dead_reckon;
pub mod motion_render_timeline;
pub mod motor_stepper;
pub mod net_ethernet;
pub mod net_telemetry;
pub mod net_tsn_macsec;
pub mod office_calc_layout;
pub mod packet_checksum;
pub mod pcb_cad_fem;
pub mod physics_raycast_3d;
pub mod receipt;
pub mod ring_buffer;
pub mod rpa_screen_control;
pub mod sensor_baro_als;
pub mod sensor_hardware;
pub mod sensor_motion;
pub mod sensor_power_meter;
pub mod spatial_physics;
pub mod sys_path;
pub mod text_cleaner;
pub mod thermal_palette;
pub mod thermal_pid;
pub mod touch_control;
pub mod vision_feature;
pub mod vision_yuv;
pub mod wish_board;
