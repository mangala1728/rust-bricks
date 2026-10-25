// ## Map
// Role: app/main — rust-bricks 命令列自測檢驗入口 (CLI Self-Test Runner)。
// Inputs: 命令列參數（例如 selftest）。
// Outputs: 測試通過狀態與報告。
// Invariants: 零全域可變狀態、無遞迴、錯誤皆安全退出。

use rust_bricks::{audio_vad, crypto_vault, math_time, sys_path, text_cleaner};
use std::env;

/// @outputs: 1
fn main() {
    let args: Vec<String> = env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match cmd {
        "selftest" | "test" => {
            println!("🚀 [rust-bricks] Running Zero-Dependency Contract Self-Test...");

            // 1. math_time
            if math_time::clamp_i64(150, 0, 100) != 100 {
                println!("  ❌ math_time: FAIL");
                return;
            }
            if math_time::julian_day_from_ymd(2026, 10, 26).is_none() {
                println!("  ❌ math_time: FAIL");
                return;
            }
            println!("  ✅ math_time: PASS");

            // 2. audio_vad
            let pcm = vec![1000i16; 480];
            if audio_vad::pcm16_rms_energy(&pcm) != 1000 {
                println!("  ❌ audio_vad: FAIL");
                return;
            }
            if audio_vad::cps_chars_per_second(15, 3000) != Some(5.0) {
                println!("  ❌ audio_vad: FAIL");
                return;
            }
            println!("  ✅ audio_vad: PASS");

            // 3. crypto_vault
            let hash = crypto_vault::sha256_hex(b"rust-bricks");
            if hash.is_empty() || !crypto_vault::constant_time_eq(b"key", b"key") {
                println!("  ❌ crypto_vault: FAIL");
                return;
            }
            println!("  ✅ crypto_vault: PASS");

            // 4. sys_path
            if !sys_path::is_safe_rel_path("crates/my_brick.rs") || sys_path::is_safe_rel_path("../bad/path") {
                println!("  ❌ sys_path: FAIL");
                return;
            }
            println!("  ✅ sys_path: PASS");

            // 5. text_cleaner
            let truncated = text_cleaner::truncate_at_boundary("梵語專有名詞", 6);
            if truncated != "梵語" {
                println!("  ❌ text_cleaner: FAIL");
                return;
            }
            println!("  ✅ text_cleaner: PASS");

            println!("✨ [rust-bricks] All 5 core brick modules verified successfully! 100% PASS.");
        }
        _ => {
            println!("rust-bricks v0.1.0 — Pure Zero-Dependency Function Blocks (09 Contract)");
            println!("Usage:");
            println!("  cargo run -- selftest   # Run internal contract self-test");
            println!("  cargo test              # Run full unit test suite");
        }
    }
}
