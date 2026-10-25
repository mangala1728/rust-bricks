// ## Map
// Role: app/main — rust-bricks 命令列自測、索引產生與守護入口 (CLI Runner)。
// Inputs: 命令列參數（selftest / catalog / guard）。
// Outputs: 退出代碼與終端執行報告。
// Invariants: 零全域可變狀態、無遞迴、零外部依賴、控制流巢狀深度 <= 2。

use rust_bricks::{audio_vad, catalog, crypto_vault, guard, math_time, sys_path, text_cleaner};
use std::env;
use std::path::Path;
use std::process;

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
fn print_help() {
    println!("rust-bricks v0.1.0 — Pure Zero-Dependency Function Blocks (09 Contract)");
    println!("Usage:");
    println!("  cargo run -- selftest          # Run internal contract self-test");
    println!("  cargo run -- catalog           # Generate CATALOG.yaml & CATALOG.json (Pure Rust)");
    println!("  cargo run -- guard             # Run capacity & contract guard hook (Pure Rust)");
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
        "receipt" => {
            let model = args.get(2).map(|s| s.as_str()).unwrap_or("kev-4b");
            let verdict = args.get(3).map(|s| s.as_str()).unwrap_or("PASS");
            run_receipt_cmd(model, verdict);
        }
        "verify-receipt" => run_verify_receipt_cmd(),
        _ => print_help(),
    }
}
