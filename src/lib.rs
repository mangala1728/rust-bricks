// ## Map
// Role: core/lib — rust-bricks 純淨函式積木庫總導出入口。
// Inputs: 子模組 (agent_terminal, audio_dsp, audio_vad, audio_wav, bezier_curve, ble_adv_parser, can_bus_frame, catalog, chaos_audit, crypto_dukpt, crypto_kdf, crypto_vault, dsp_fixed_point, env_sensor, fast_math, geo_collision, guard, iso7816_smartcard, math_time, motion_dead_reckon, motor_stepper, net_telemetry, packet_checksum, receipt, ring_buffer, sensor_hardware, sensor_motion, spatial_physics, sys_path, text_cleaner, thermal_palette, thermal_pid, touch_control, vision_feature, vision_yuv, wish_board)。
// Outputs: 導出安全、有界、零依賴之純函式介面。
// Invariants: 零全域可變狀態、禁止 unsafe 代碼、100% 符合 09 號合約。

#![forbid(unsafe_code)]

pub mod agent_terminal;
pub mod audio_dsp;
pub mod audio_vad;
pub mod audio_wav;
pub mod bezier_curve;
pub mod ble_adv_parser;
pub mod can_bus_frame;
pub mod catalog;
pub mod chaos_audit;
pub mod crypto_dukpt;
pub mod crypto_kdf;
pub mod crypto_vault;
pub mod dsp_fixed_point;
pub mod env_sensor;
pub mod fast_math;
pub mod geo_collision;
pub mod guard;
pub mod iso7816_smartcard;
pub mod math_time;
pub mod motion_dead_reckon;
pub mod motor_stepper;
pub mod net_telemetry;
pub mod packet_checksum;
pub mod receipt;
pub mod ring_buffer;
pub mod sensor_hardware;
pub mod sensor_motion;
pub mod spatial_physics;
pub mod sys_path;
pub mod text_cleaner;
pub mod thermal_palette;
pub mod thermal_pid;
pub mod touch_control;
pub mod vision_feature;
pub mod vision_yuv;
pub mod wish_board;
