// SPDX-License-Identifier: MIT OR Apache-2.0
//! ## Map
//! - Role: core/decision_local_edge — Local Edge Decision Model (GGUF / llama.cpp System One) Hardware Profile Router, Boundary Guards, PointerHead Softmax & Track-1 Command/Edit Gatekeeper
//! - Inputs: Free VRAM (MiB), available system RAM (MiB), GGUF availability, prompt token count, Kev raw pointer logits, shell commands, repo-relative edit paths & snippets
//! - Outputs: Optimal `LocalEdgeModelId` selection, validated `llama.cpp` `(batch, ubatch)` sizes, delimiter-safe prompt text, calibrated `(argmax, confidence)`, and Track-1 `ALLOW`/`BLOCK`/`FUZZY` verdicts
//! - Invariants: Zero external dependencies; `#![forbid(unsafe_code)]`; zero mutable globals; zero recursion; nesting depth <= 2; explicit `@outputs` & `@domain` guards.
//!
//! ## Empirical Benchmarks & Hardware Boundary Conditions (Measured 2026-11-16/17 on NVIDIA RTX 3050 Laptop 4GB / 15.3 GB RAM)
//! 1. **Kev-4B bf16 (CPU, 8.67 GB weights)**: Loaded in 17.80s; dropped free RAM from 4.81 GB (68%) to 0.12 GB (99%); 9,010-10,716 ms/query. Unusable for interactive PreToolUse hooks on 16 GB laptops.
//! 2. **ggml-org/Kev-4B-GGUF (`Kev-4B-Q4_K_M.gguf`, 2.83 GiB GPU CUDA)**:
//!    - VRAM: 3,807 MiB (`-c 4096`), **3,899 MiB / 4,096 MiB (`64 MiB` free)** even at `-c 2048 -b 2048 -ub 2048`. Must ONLY auto-select if free VRAM >= 3,900 MiB (risk of spill/OOM when IDE/browser active).
//!    - Speed & Accuracy: 84.9-128.8 ms (short), 979.4-1518.8 ms (747-tok git diff `YES 0.7041`), 9/10 English tooling cases; Traditional Chinese nuanced diff `0.0286` conf (English-only training).
//! 3. **LiquidAI/d1-3B-GGUF (`d1-3B-Q4_K_M.gguf`, 1.56 GiB GPU CUDA — Default Recommended for 4GB VRAM)**:
//!    - VRAM: 1,821-2,403 MiB (leaves **1,560 MiB VRAM free** alongside IDE/browser). Min free VRAM >= 2,600 MiB.
//!    - Speed & Accuracy: **40.8-63.1 ms** (short), **290.4-375.7 ms** (797-tok diff `YES 0.5213`), 9/10 incl. Traditional Chinese safety (`8_zh_fuzzy` BLOCK `0.9038`). Note: misled by deceptive Chinese commit (`9_zh_sneaky_commit`).
//! 4. **LiquidAI/d1-2B-GGUF (`d1-2B-Q8_0.gguf`, 1.72 GiB GPU CUDA)**:
//!    - VRAM: 1,891-2,381 MiB (leaves **1,582 MiB VRAM free**). Min free VRAM >= 2,500 MiB.
//!    - Speed & Accuracy: **30.8-57.4 ms** (short), **251.1 ms** (799-tok diff). Very sharp calibration (`0.95-0.998`) on short prompts, cautious `UNSURE` on sneaky zh commit, false `NO` (`0.5167`) on 799-tok diff.
//! 5. **Kev-0.8B (`fp32` CPU Fallback)**: Loaded in 2.28-3.89s, **519-600 ms/query**, safe RAM baseline when no GPU/GGUF present.
//! 6. **Modal `Nimble-9B` (Cloud L4 GPU)**: 0 MB local VRAM/RAM, **854-1014 ms** warm, 5/5 confidence (`0.986-0.999`), but cold start after 300s `scaledown_window` exceeded `>90s` in test -> **local-first `AUTO` routing (`local_edge -> cloud_nimble`) prevents cold-start stalls**.
//! 7. **llama.cpp `-ub` Batch Boundary**: Stock `llama-server` defaults `-ub 512`, causing HTTP 500 on git commits >= 513 tokens (observed on 799-token `7_real_diff`). Always enforce `compute_llamacpp_ubatch` with `-b == -ub == ctx_limit` (>= 2048).

const MAX_SOFTMAX_CLASSES: usize = 64;
const MAX_SCAN_BYTES: usize = 65536;
const MAX_PATH_BYTES: usize = 1024;

/// Supported local edge decision model tiers ordered by hardware suitability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalEdgeModelId {
    D1_3bQ4KM,
    D1_2bQ8_0,
    Kev4bQ4KM,
    Kev08bCpuFp32,
}

impl LocalEdgeModelId {
    /// Returns canonical profile identifier string.
    /// @outputs: 4
    pub fn profile_id(&self) -> &'static str {
        match self {
            LocalEdgeModelId::D1_3bQ4KM => "d1-3b",
            LocalEdgeModelId::D1_2bQ8_0 => "d1-2b",
            LocalEdgeModelId::Kev4bQ4KM => "kev-4b",
            LocalEdgeModelId::Kev08bCpuFp32 => "kev-0.8b",
        }
    }
}

/// Measured hardware requirements and latency envelopes for a local decision model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HardwareProfileSpec {
    pub model: LocalEdgeModelId,
    pub min_free_vram_mb: u32,
    pub measured_vram_mb: u32,
    pub min_free_ram_mb: u32,
    pub short_latency_ms_max: u32,
    pub long_diff_latency_ms_max: u32,
    pub supports_zh: bool,
}

/// Returns empirical hardware specs and latency boundaries measured on RTX 3050 / 16GB RAM host.
/// @outputs: 4
pub fn get_model_benchmark_spec(model: LocalEdgeModelId) -> HardwareProfileSpec {
    match model {
        LocalEdgeModelId::D1_3bQ4KM => HardwareProfileSpec {
            model,
            min_free_vram_mb: 2600,
            measured_vram_mb: 2403,
            min_free_ram_mb: 1024,
            short_latency_ms_max: 64,
            long_diff_latency_ms_max: 376,
            supports_zh: true,
        },
        LocalEdgeModelId::D1_2bQ8_0 => HardwareProfileSpec {
            model,
            min_free_vram_mb: 2500,
            measured_vram_mb: 2381,
            min_free_ram_mb: 1024,
            short_latency_ms_max: 58,
            long_diff_latency_ms_max: 252,
            supports_zh: true,
        },
        LocalEdgeModelId::Kev4bQ4KM => HardwareProfileSpec {
            model,
            min_free_vram_mb: 3900,
            measured_vram_mb: 3899,
            min_free_ram_mb: 1536,
            short_latency_ms_max: 129,
            long_diff_latency_ms_max: 1519,
            supports_zh: false,
        },
        LocalEdgeModelId::Kev08bCpuFp32 => HardwareProfileSpec {
            model,
            min_free_vram_mb: 0,
            measured_vram_mb: 0,
            min_free_ram_mb: 3600,
            short_latency_ms_max: 600,
            long_diff_latency_ms_max: 6000,
            supports_zh: false,
        },
    }
}

/// Selects the safest high-accuracy local edge model given live free VRAM, RAM, and available GGUFs.
/// @outputs: 4
/// @domain: free_vram_mb [0..131072]
/// @domain: avail_ram_mb [0..524288]
pub fn select_local_edge_profile(
    free_vram_mb: u32,
    avail_ram_mb: u32,
    prefer_multilingual: bool,
    has_d1_3b: bool,
    has_d1_2b: bool,
    has_kev_4b: bool,
) -> LocalEdgeModelId {
    if free_vram_mb > 131_072 || avail_ram_mb > 524_288 {
        return LocalEdgeModelId::Kev08bCpuFp32;
    }
    if has_kev_4b && !prefer_multilingual && free_vram_mb >= 3900 && avail_ram_mb >= 1536 {
        return LocalEdgeModelId::Kev4bQ4KM;
    }
    if has_d1_3b && free_vram_mb >= 2600 && avail_ram_mb >= 1024 {
        return LocalEdgeModelId::D1_3bQ4KM;
    }
    if has_d1_2b && free_vram_mb >= 2500 && avail_ram_mb >= 1024 {
        return LocalEdgeModelId::D1_2bQ8_0;
    }
    if has_kev_4b && free_vram_mb >= 3900 && avail_ram_mb >= 1536 {
        return LocalEdgeModelId::Kev4bQ4KM;
    }
    LocalEdgeModelId::Kev08bCpuFp32
}

/// Computes `(batch_size, ubatch_size)` preventing `llama.cpp` HTTP 500 on prompts > 512 tokens.
/// @outputs: 2
/// @domain: prompt_tokens [1..65536]
/// @domain: ctx_limit [512..65536]
pub fn compute_llamacpp_ubatch(prompt_tokens: usize, ctx_limit: usize) -> Option<(usize, usize)> {
    if prompt_tokens == 0 || prompt_tokens > 65536 || ctx_limit < 512 || ctx_limit > 65536 {
        return None;
    }
    if prompt_tokens > ctx_limit {
        return None;
    }
    let min_batch = if prompt_tokens > 512 { prompt_tokens } else { 512 };
    let aligned = if min_batch <= 2048 && ctx_limit >= 2048 {
        2048
    } else {
        ctx_limit
    };
    Some((aligned, aligned))
}

/// Rewrites `<|name|>` control delimiter tokens into `<¦name¦>` within a bounded byte limit.
/// @outputs: 2
/// @domain: max_bytes [16..65536]
pub fn sanitize_kev_delimiters(text: &str, max_bytes: usize) -> String {
    if max_bytes < 16 || max_bytes > 65536 {
        return String::new();
    }
    let limit = max_bytes.min(MAX_SCAN_BYTES);
    let mut end = text.len().min(limit);
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    let slice = text[..end].trim();
    slice.replace("<|", "<¦").replace("|>", "¦>")
}

/// Computes calibrated PointerHead probabilities and System One confidence `(p_max - 1/K)/(1 - 1/K)`.
/// @outputs: 2
/// @domain: temperature [0.1..10.0]
pub fn pointer_head_softmax_calibrated(
    logits: &[f64],
    temperature: f64,
    out_probs: &mut [f64],
) -> Option<(usize, f64)> {
    let k = logits.len();
    if k == 0 || k > MAX_SOFTMAX_CLASSES || out_probs.len() < k {
        return None;
    }
    if !temperature.is_finite() || temperature < 0.1 || temperature > 10.0 {
        return None;
    }

    let mut max_z = f64::NEG_INFINITY;
    for i in 0..k {
        if !logits[i].is_finite() {
            return None;
        }
        let z = logits[i] / temperature;
        if z > max_z {
            max_z = z;
        }
    }

    let mut sum_exp = 0.0f64;
    for i in 0..k {
        let e = ((logits[i] / temperature) - max_z).exp();
        out_probs[i] = e;
        sum_exp += e;
    }
    if sum_exp <= 0.0 || !sum_exp.is_finite() {
        return None;
    }

    let mut best_i = 0usize;
    let mut max_p = -1.0f64;
    for i in 0..k {
        out_probs[i] /= sum_exp;
        if out_probs[i] > max_p {
            max_p = out_probs[i];
            best_i = i;
        }
    }

    let conf = if k == 1 {
        1.0
    } else {
        let kf = k as f64;
        ((max_p - 1.0 / kf) / (1.0 - 1.0 / kf)).max(0.0)
    };
    Some((best_i, conf))
}

/// Escalates low-confidence fuzzy ALLOW verdicts to ESCALATE to eliminate uncertain false passes.
/// @outputs: 2
/// @domain: confidence [0.0..1.0]
/// @domain: min_conf [0.1..1.0]
pub fn apply_low_conf_allow_escalation(
    choice: &str,
    confidence: f64,
    min_conf: f64,
) -> (&'static str, f64) {
    if !confidence.is_finite()
        || !min_conf.is_finite()
        || confidence < 0.0
        || confidence > 1.0
        || min_conf < 0.1
        || min_conf > 1.0
    {
        return ("ESCALATE", 0.0);
    }
    if choice == "ALLOW" && confidence < min_conf {
        return ("ESCALATE", ((1.0 - confidence) * 10000.0).round() / 10000.0);
    }
    if choice == "BLOCK" {
        return ("BLOCK", confidence);
    }
    if choice == "ESCALATE" {
        return ("ESCALATE", confidence);
    }
    if choice == "BYPASS" {
        return ("BYPASS", confidence);
    }
    ("ALLOW", confidence)
}

/// Resolves System One routing tier order (`AUTO` prefers local GPU 8008 before cloud Nimble-9B).
/// @outputs: 4
pub fn resolve_dual_track_route_order(mode: &str) -> &'static [&'static str] {
    let trimmed = mode.trim();
    if trimmed.eq_ignore_ascii_case("OFF") {
        &[]
    } else if trimmed.eq_ignore_ascii_case("CLOUD_NIMBLE") || trimmed.eq_ignore_ascii_case("CLOUD_SPAN") {
        &["cloud_nimble"]
    } else if trimmed.eq_ignore_ascii_case("LOCAL_EDGE") {
        &["local_edge"]
    } else {
        &["local_edge", "cloud_nimble"]
    }
}

/// Normalizes a file path relative to repo root while preserving leading dots (e.g. `.agents/hooks.json`).
/// @outputs: 3
pub fn normalize_repo_rel_path(raw_path: &str, repo_root: &str) -> String {
    if raw_path.is_empty() || raw_path.len() > MAX_PATH_BYTES || repo_root.len() > MAX_PATH_BYTES {
        return String::new();
    }
    let norm_raw = raw_path.replace('\\', "/");
    let norm_root = format!("{}/", repo_root.replace('\\', "/").trim_end_matches('/'));
    let low_raw = norm_raw.to_ascii_lowercase();
    let low_root = norm_root.to_ascii_lowercase();
    if low_raw.starts_with(&low_root) {
        return low_raw[low_root.len()..].to_string();
    }
    if let Some(stripped) = low_raw.strip_prefix("./") {
        return stripped.to_string();
    }
    low_raw
}

/// Track-1 deterministic command classifier: returns `("BLOCK"|"ALLOW"|"FUZZY", is_terminal)`.
/// @outputs: 3
pub fn classify_track1_command(cmd: &str) -> (&'static str, bool) {
    let low = cmd.trim().to_ascii_lowercase();
    if low.is_empty() || low.len() > MAX_SCAN_BYTES {
        return ("ALLOW", true);
    }
    let hard_block = (low.contains("git commit") && (low.contains("--no-verify") || low.contains(" -n")))
        || low.contains("core.hookspath")
        || low.contains("rm -rf /")
        || (low.contains("del ") && low.contains("c:\\"));
    if hard_block {
        return ("BLOCK", true);
    }
    if is_fuzzy_command(&low) {
        return ("FUZZY", false);
    }
    ("ALLOW", true)
}

/// Checks whether a lowercase shell command touches deletion, git clean/reset/push, or evals.
/// @outputs: 1
fn is_fuzzy_command(low: &str) -> bool {
    let triggers = [
        "rm ",
        "rm\t",
        "remove-item",
        "del ",
        "rmdir",
        "erase ",
        "git clean",
        "git reset",
        "git push",
        "git checkout",
        "git restore",
        "evals/",
        "exemplars.jsonl",
        "allow_gauge_edit",
        "drop table",
        "adb uninstall",
    ];
    for kw in triggers {
        if low.contains(kw) {
            return true;
        }
    }
    false
}

/// Track-1 deterministic file-edit classifier: protects constitution paths and leaked secrets.
/// @outputs: 4
pub fn classify_track1_edit(rel_path: &str, snippet: &str) -> (&'static str, bool) {
    let low_path = rel_path.trim().to_ascii_lowercase();
    let protected = [
        ".agents/hooks.json",
        ".claude/settings.json",
        ".git/hooks/",
        "claude.md",
        "docs/spec.md",
        "keys.local.json",
    ];
    for prefix in protected {
        if low_path == prefix || low_path.starts_with(prefix) {
            return ("BLOCK", true);
        }
    }
    if contains_secret_prefix(snippet) {
        return ("BLOCK", true);
    }
    if is_eval_or_test_path(&low_path) {
        return ("FUZZY", false);
    }
    ("ALLOW", true)
}

/// Scans an edit snippet up to 64 KB for common API credential token prefixes.
/// @outputs: 2
fn contains_secret_prefix(snippet: &str) -> bool {
    let limit = snippet.len().min(MAX_SCAN_BYTES);
    let view = &snippet[..limit];
    let prefixes = ["AIzaSy", "sk-ant-", "gsk_", "ghp_", "AKIA"];
    for p in prefixes {
        if view.contains(p) {
            return true;
        }
    }
    false
}

/// Returns true when normalized path targets evaluation suites, tests, or safety gate logic.
/// @outputs: 1
fn is_eval_or_test_path(low_path: &str) -> bool {
    low_path.starts_with("evals/")
        || low_path.contains("/evals/")
        || low_path.starts_with("tests/")
        || low_path.contains("/tests/")
        || low_path.ends_with("test.kt")
        || low_path.ends_with("_test.rs")
        || low_path.ends_with("governance_gates.rs")
        || low_path.contains("/test_")
}
