// ## Map
// Role: core/lib — rust-bricks 純淨函式積木庫總導出入口。
// Inputs: 子模組 (audio_dsp, audio_vad, catalog, crypto_vault, guard, math_time, sys_path, text_cleaner)。
// Outputs: 導出安全、有界、零依賴之純函式介面。
// Invariants: 零全域可變狀態、禁止 unsafe 代碼、100% 符合 09 號合約。

#![forbid(unsafe_code)]

pub mod audio_dsp;
pub mod audio_vad;
pub mod audio_wav;
pub mod catalog;
pub mod crypto_vault;
pub mod guard;
pub mod math_time;
pub mod packet_checksum;
pub mod receipt;
pub mod ring_buffer;
pub mod sensor_motion;
pub mod sys_path;
pub mod text_cleaner;
pub mod vision_yuv;
