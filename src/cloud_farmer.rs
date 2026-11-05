// ## Map
// Role: core/cloud_farmer — 零外部依賴之雙軌雲端自動農積木引擎（支援多免費帳號 Key 池輪替、實機 Guard 閉環自癒與自動排程）。
// Inputs: 規格書路徑 (specs/*.md 或 wishes/*.md)、目標模組名稱、免費 API 金鑰池環境變數。
// Outputs: 驗證通過並入庫之純 Rust 模組與單元測試，或詳細排障錯誤報告。
// Invariants: 零外部依賴、無遞迴、零全域可變狀態、控制流巢狀深度 <= 2、單一函式 <= 60 行、失敗自動回滾。

use crate::farm_tree::{
    classify_failure, dead_end_prompt, dead_end_samples, extract_golden_lines, first_error_line, fresh_dead_ends,
    golden_assert_count, load_dead_ends, module_state, next_strike, record_abandoned_round, render_golden_test,
    spec_blob_hash, spec_suspect, strike_action, strike_note, AbandonInput, DeadEnd, ModuleState, RoundRecord,
    StrikeAction, StrikeState, BLOCK_ROUNDS, INFRA_TAG,
};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 一次農作的固定資料（規格、原始 lib.rs、已知死路、規格黃金向量測試）
pub struct FarmContext {
    pub repo_root: PathBuf,
    pub module: String,
    pub orig_lib: String,
    pub spec_text: String,
    pub memory: String,
    pub golden_test: Option<String>,
}

/// 農作成功的結果
#[derive(Clone, Debug, PartialEq)]
pub struct FarmSuccess {
    pub provider: String,
    pub test_summary: String,
    pub golden_count: usize,
    pub attempts_used: usize,
    pub spec_hash: String,
    pub resolves: Vec<String>,
}

/// 農作失敗的結果：outcome 為 abandoned／blocked／suspect／escalated／infra／skip-claimed／skip-blocked／skip-suspect
#[derive(Clone, Debug, PartialEq)]
pub struct FarmFailure {
    pub outcome: String,
    pub detail: String,
    pub attempt_branch: Option<String>,
}

/// 一輪重試迴圈的結果
enum LoopResult {
    Success(String, String, usize),
    Failed(Vec<RoundRecord>, String, bool),
    Infra(String),
}

/// @outputs: 1
pub fn parse_api_key_pool(raw_keys: &str) -> Vec<String> {
    let mut pool = Vec::new();
    for part in raw_keys.split(',') {
        let trimmed = part.trim();
        if !trimmed.is_empty() && trimmed.len() <= 256 {
            pool.push(trimmed.to_string());
        }
    }
    pool
}

/// @outputs: 2
/// @domain: attempt [1..100]
pub fn select_rotating_key(pool: &[String], attempt: usize) -> Option<String> {
    if attempt < 1 || attempt > 100 || pool.is_empty() {
        return None;
    }
    let idx = (attempt - 1) % pool.len();
    Some(pool[idx].clone())
}

/// @outputs: 1
pub fn escape_json_string(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 64);
    for ch in input.chars() {
        if ch == '\x22' {
            out.push_str("\\\x22");
        } else if ch == '\x5C' {
            out.push_str("\\\\");
        } else if ch == '\n' {
            out.push_str("\\n");
        } else if ch == '\r' {
            out.push_str("\\r");
        } else if ch == '\t' {
            out.push_str("\\t");
        } else if !ch.is_control() {
            out.push(ch);
        }
    }
    out
}

/// @outputs: 1
pub fn build_farmer_prompt(module_name: &str, spec_text: &str, memory: &str, error_feedback: &str) -> String {
    let feedback_section = if error_feedback.is_empty() {
        String::new()
    } else {
        format!("【前次執行本地 cargo test / guard 報錯如下，請攻擊前提並徹底修復】：\n{}\n", error_feedback)
    };

    let rules = concat!(
        "【強制硬體級合約紅線（違反將被 guard.rs 當場攔截）】：\n",
        "1. 檔案開頭必備 `// ## Map`（含 Role, Inputs, Outputs, Invariants）。\n",
        "2. 零外部依賴（僅使用 Rust `std` 標準庫），禁止 `static mut`、禁止遞迴、控制流巢狀深度 <= 2。\n",
        "3. 每個具名函式上方必備 `/// @outputs: N`（1 <= N <= 32），且單一函式長度嚴格 <= 60 行（NASA JPL 微積木原則）。\n",
        "4. 所有數值參數必備 `/// @domain: param [min..max]`，且函式第一行必須實作 `if param < min || param > max` 執行期守衛。\n",
        "5. 優先使用 match 或 if let 進行安全解構，嚴禁未附帶 `// @safety-proof` 的 `.unwrap()`、`.expect()`、`panic!` 或 `unsafe`。\n",
        "6. 測試必須使用確定性黃金值（Golden Values），禁止弱斷言。\n",
        "7. 整合測試檔案位於 tests/ 目錄下，必須使用 `use rust_bricks::<模組名>::*;` 匯入，嚴禁使用 `use super::*;`！\n",
        "8. 規格書 ```golden 區塊會由系統自動產生 tests/golden_<模組名>.rs 驗證，函式名稱與簽章必須與規格完全一致；不得在測試中放寬或改寫黃金向量的預期值。\n"
    );

    format!(
        "你是專精零外部依賴純 Rust 演算法與 09 號合約邊界規範的硬體級工程師。\n請根據以下規格書，為 `rust-bricks` 編寫純 Rust 模組 `{0}` 與獨立整合測試：\n\n{1}\n\n{4}{2}\n{3}\n【輸出規範】：請嚴格使用 ```rust:module 與 ```rust:test 兩個標籤輸出完整代碼。\n",
        module_name, spec_text, feedback_section, rules, memory
    )
}

/// @outputs: 2
pub fn extract_fenced_block(raw_text: &str, tag: &str) -> Option<String> {
    let markers = [format!("```{}", tag), format!("```{}s", tag)];
    for marker in &markers {
        if let Some(start_pos) = raw_text.find(marker.as_str()) {
            let after_marker = &raw_text[start_pos + marker.len()..];
            if let Some(newline_pos) = after_marker.find('\n') {
                let body_start = &after_marker[newline_pos + 1..];
                if let Some(end_pos) = body_start.find("```") {
                    let extracted = body_start[..end_pos].trim();
                    if !extracted.is_empty() {
                        return Some(format!("{}\n", extracted));
                    }
                }
            }
        }
    }
    None
}

/// @outputs: 1
pub fn extract_second_rust_block(raw_text: &str) -> Option<String> {
    let first_marker = "```rust";
    let p1 = raw_text.find(first_marker)?;
    let after_p1 = &raw_text[p1 + first_marker.len()..];
    let end1 = after_p1.find("```")?;
    let rest = &after_p1[end1 + 3..];
    let p2 = rest.find(first_marker)?;
    let after_p2 = &rest[p2 + first_marker.len()..];
    let nl = after_p2.find('\n')?;
    let body = &after_p2[nl + 1..];
    let end2 = body.find("```")?;
    let extracted = body[..end2].trim();
    if extracted.is_empty() {
        return None;
    }
    Some(format!("{}\n", extracted))
}

/// @outputs: 2
fn should_insert_before(trimmed: &str, module_name: &str) -> bool {
    if !trimmed.starts_with("pub mod ") || !trimmed.ends_with(';') {
        return false;
    }
    let existing = trimmed.trim_start_matches("pub mod ").trim_end_matches(';').trim();
    module_name < existing
}

/// @outputs: 1
pub fn insert_mod_alphabetically(lib_content: &str, module_name: &str) -> String {
    let target_decl = format!("pub mod {};", module_name);
    let mut lines_out = Vec::new();
    let mut inserted = false;

    for line in lib_content.lines() {
        let trimmed = line.trim();
        if trimmed == target_decl {
            inserted = true;
        } else if !inserted && should_insert_before(trimmed, module_name) {
            lines_out.push(target_decl.clone());
            inserted = true;
        }
        lines_out.push(line.to_string());
    }
    if !inserted {
        lines_out.push(target_decl);
    }
    format!("{}\n", lines_out.join("\n"))
}

/// @outputs: 1
fn unescape_json_snippet(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let chars: Vec<char> = raw.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            match chars[i + 1] {
                'n' => { out.push('\n'); i += 2; }
                'r' => { out.push('\r'); i += 2; }
                't' => { out.push('\t'); i += 2; }
                '"' => { out.push('"'); i += 2; }
                '\\' => { out.push('\\'); i += 2; }
                'u' if i + 5 < chars.len() => {
                    let hex: String = chars[i + 2..=i + 5].iter().collect();
                    if let Ok(code) = u32::from_str_radix(&hex, 16) {
                        if let Some(ch) = char::from_u32(code) {
                            out.push(ch);
                            i += 6;
                            continue;
                        }
                    }
                    out.push('\\');
                    i += 1;
                }
                _ => { out.push('\\'); i += 1; }
            }
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

/// @outputs: 2
fn find_unescaped_quote(slice: &str) -> Option<usize> {
    let mut escaped = false;
    for (i, ch) in slice.char_indices() {
        if escaped {
            escaped = false;
        } else if ch == '\x5C' {
            escaped = true;
        } else if ch == '\x22' {
            return Some(i);
        }
    }
    None
}

/// @outputs: 2
fn extract_field_by_marker(resp_json: &str, marker: &str) -> Option<String> {
    let pos = resp_json.find(marker)?;
    let after = resp_json[pos + marker.len()..].trim_start();
    if !after.starts_with('\x22') {
        return None;
    }
    let content_slice = &after[1..];
    let end_idx = find_unescaped_quote(content_slice)?;
    if end_idx == 0 {
        return None;
    }
    Some(unescape_json_snippet(&content_slice[..end_idx]))
}

/// @outputs: 2
pub fn extract_llm_response_text(resp_json: &str) -> Option<String> {
    if let Some(txt) = extract_field_by_marker(resp_json, "\x22text\x22:") {
        return Some(txt);
    }
    extract_field_by_marker(resp_json, "\x22content\x22:")
}

/// @outputs: 2
fn call_curl_json(
    url: &str,
    headers: &[String],
    body_json: &str,
    repo_root: &Path,
    timeout_secs: u32,
) -> Result<String, String> {
    let tmp_dir = repo_root.join(".ai-memory");
    let _ = fs::create_dir_all(&tmp_dir);
    let payload_path = tmp_dir.join("farmer_payload.json");
    fs::write(&payload_path, body_json).map_err(|e| format!("寫入請求暫存檔失敗: {}", e))?;

    let timeout_str = timeout_secs.to_string();
    let mut cmd = Command::new("curl");
    cmd.args([
        "-s",
        "-S",
        "-X",
        "POST",
        url,
        "-H",
        "User-Agent: rust-bricks/1.0",
        "--max-time",
        &timeout_str,
    ]);
    for h in headers {
        cmd.args(["-H", h]);
    }
    let arg_data = format!("@{}", payload_path.display());
    cmd.args(["-d", &arg_data]);

    let out = cmd.output().map_err(|e| format!("呼叫 curl 失敗: {}", e))?;
    let _ = fs::remove_file(&payload_path);
    if !out.status.success() {
        return Err(format!("curl 回傳非零狀態: {}", String::from_utf8_lossy(&out.stderr)));
    }
    let resp_str = String::from_utf8_lossy(&out.stdout).to_string();
    extract_llm_response_text(&resp_str).ok_or_else(|| format!("無法解析 API 回應內容: {}", resp_str))
}

/// @outputs: 1
pub fn resolve_key_pool_strings(repo_root: &Path, env_plural: &str, env_single: &str, json_keys: &[&str]) -> Vec<String> {
    let env_val = env::var(env_plural).or_else(|_| env::var(env_single)).unwrap_or_default();
    let from_env: Vec<String> = parse_api_key_pool(&env_val).into_iter().map(|s| s.to_string()).collect();
    if !from_env.is_empty() {
        return from_env;
    }
    let p1 = repo_root.join("keys.local.json");
    let p2 = repo_root.join("..").join("工作流").join("keys.local.json");
    let raw = fs::read_to_string(&p1).or_else(|_| fs::read_to_string(&p2)).unwrap_or_default();
    if raw.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for k in json_keys {
        let marker = ["\x22", k, "\x22:"].concat();
        if let Some(val) = extract_field_by_marker(&raw, &marker) {
            out.push(val);
        }
    }
    out
}

/// @outputs: 2
fn call_gemini_endpoint(
    gkey: &str,
    model: &str,
    escaped_prompt: &str,
    repo_root: &Path,
    timeout_secs: u32,
) -> Result<(String, String), String> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, gkey
    );
    let body = [
        "\x7B\x22contents\x22:[\x7B\x22parts\x22:[\x7B\x22text\x22:\x22",
        escaped_prompt,
        "\x22\x7D]\x7D],\x22generationConfig\x22:\x7B\x22temperature\x22:0.1\x7D\x7D",
    ]
    .concat();
    let headers = vec!["Content-Type: application/json".to_string()];
    let text = call_curl_json(&url, &headers, &body, repo_root, timeout_secs)?;
    Ok((format!("gemini:{}", model), text))
}

/// @outputs: 2
fn call_openai_compatible_endpoint(
    url: &str,
    token: &str,
    model: &str,
    escaped_prompt: &str,
    repo_root: &Path,
    timeout_secs: u32,
) -> Result<(String, String), String> {
    let body = [
        "\x7B\x22model\x22:\x22",
        model,
        "\x22,\x22temperature\x22:0.1,\x22messages\x22:[\x7B\x22role\x22:\x22user\x22,\x22content\x22:\x22",
        escaped_prompt,
        "\x22\x7D]\x7D",
    ]
    .concat();
    let headers = vec![
        "Content-Type: application/json".to_string(),
        format!("Authorization: Bearer {}", token),
    ];
    let text = call_curl_json(url, &headers, &body, repo_root, timeout_secs)?;
    Ok((format!("cloud:{}", model), text))
}

/// Gemini 金鑰池：首選模型失敗時降級到 gemini-3.5-flash
/// @outputs: 2
/// @domain: idx [0..1000]
fn try_gemini_pool(idx: usize, escaped: &str, repo_root: &Path) -> Option<(String, String)> {
    if idx > 1000 {
        return None;
    }
    let pool = resolve_key_pool_strings(repo_root, "GEMINI_API_KEYS", "GEMINI_API_KEY", &["gemini", "gemini2", "gemini3", "gemini4"]);
    if pool.is_empty() {
        return None;
    }
    let gkey = &pool[idx % pool.len()];
    let primary = env::var("FARMER_GEMINI_MODEL").unwrap_or_else(|_| "gemini-3.8-flash".to_string());
    if let Ok(res) = call_gemini_endpoint(gkey, &primary, escaped, repo_root, 25) {
        return Some(res);
    }
    if primary == "gemini-3.5-flash" {
        return None;
    }
    call_gemini_endpoint(gkey, "gemini-3.5-flash", escaped, repo_root, 20).ok()
}

/// Groq 金鑰池（OpenAI 相容端點）
/// @outputs: 2
/// @domain: idx [0..1000]
fn try_groq_pool(idx: usize, escaped: &str, repo_root: &Path) -> Option<(String, String)> {
    if idx > 1000 {
        return None;
    }
    let pool = resolve_key_pool_strings(repo_root, "GROQ_API_KEYS", "GROQ_API_KEY", &["groq", "groq2", "groq3", "groq4"]);
    if pool.is_empty() {
        return None;
    }
    let qkey = &pool[idx % pool.len()];
    let url = "https://api.groq.com/openai/v1/chat/completions";
    let model = env::var("FARMER_GROQ_MODEL").unwrap_or_else(|_| "openai/gpt-oss-120b".to_string());
    call_openai_compatible_endpoint(url, qkey, &model, escaped, repo_root, 25).ok()
}

/// GitHub Models（以 GITHUB_TOKEN 呼叫）
/// @outputs: 2
fn try_github_models(escaped: &str, repo_root: &Path) -> Option<(String, String)> {
    let token = env::var("GITHUB_TOKEN").ok()?;
    let url = "https://models.inference.ai.azure.com/chat/completions";
    call_openai_compatible_endpoint(url, &token, "gpt-4o-mini", escaped, repo_root, 30).ok()
}

/// 供應商嘗試順序：0=Gemini、1=Groq、2=GitHub Models；二振出局時 shift 加一，強制換供應商
/// @outputs: 1
/// @domain: shift [0..1000]
pub fn provider_order(shift: usize) -> [usize; 3] {
    let s = shift.min(1000) % 3;
    [s, (s + 1) % 3, (s + 2) % 3]
}

/// 模擬回應只准在本機使用：GitHub Actions（GITHUB_ACTIONS=true）中一律停用，免得被環境變數繞過真正的模型與黃金向量
/// @outputs: 2
pub fn mock_allowed(github_actions: Option<&str>) -> bool {
    github_actions != Some("true")
}

/// 測試專用：設定 FARMER_MOCK_DIR 時改讀 `<dir>/<attempt>.txt` 當模型回應，供應商名稱帶 shift 以觀察二振換供應商
/// @outputs: 2
/// @domain: attempt [1..20]
/// @domain: shift [0..1000]
fn mock_llm_response(dir: &str, attempt: usize, shift: usize) -> Result<(String, String), String> {
    if attempt < 1 || attempt > 20 || shift > 1000 {
        return Err(format!("{} mock 參數超出定義域", INFRA_TAG));
    }
    let path = Path::new(dir).join(format!("{}.txt", attempt));
    fs::read_to_string(&path)
        .map(|text| (format!("mock:provider{}", provider_order(shift)[0]), text))
        .map_err(|e| format!("{} mock 回應讀取失敗 {}: {}", INFRA_TAG, path.display(), e))
}

/// @outputs: 2
/// @domain: attempt [1..20]
/// @domain: shift [0..1000]
fn dispatch_cloud_llm(attempt: usize, prompt: &str, repo_root: &Path, shift: usize) -> Result<(String, String), String> {
    if attempt < 1 || attempt > 20 || shift > 1000 {
        return Err(format!("{} 嘗試次數或供應商偏移超出定義域", INFRA_TAG));
    }
    if let Ok(dir) = env::var("FARMER_MOCK_DIR") {
        if !mock_allowed(env::var("GITHUB_ACTIONS").ok().as_deref()) {
            return Err(format!("{} FARMER_MOCK_DIR 只供本機測試，在 GitHub Actions 中一律停用", INFRA_TAG));
        }
        return mock_llm_response(&dir, attempt, shift);
    }
    let escaped = escape_json_string(prompt);
    let node_offset: usize = env::var("FARMER_NODE_ID").ok().and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
    let idx = ((attempt - 1) + node_offset) % 1000;
    for provider in provider_order(shift) {
        let res = match provider {
            0 => try_gemini_pool(idx, &escaped, repo_root),
            1 => try_groq_pool(idx, &escaped, repo_root),
            _ => try_github_models(&escaped, repo_root),
        };
        if let Some(ok) = res {
            return Ok(ok);
        }
    }
    Err(format!("{} 所有免費 API 金鑰池 (Gemini 4x / Groq 4x / GitHub Models) 皆無法連線或未設定！", INFRA_TAG))
}

/// 依序跑指定的整合測試，再跑 guard；回傳各測試的通過摘要
/// @outputs: 2
fn verify_farmed_module(repo_root: &Path, test_targets: &[String]) -> Result<String, String> {
    let mut summary = Vec::new();
    for target in test_targets {
        let out = Command::new("cargo")
            .args(["test", "--target-dir", "target/farmer_check", "--test", target.as_str()])
            .current_dir(repo_root)
            .output()
            .map_err(|e| format!("執行 cargo test 失敗: {}", e))?;
        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
        if !out.status.success() {
            return Err(format!("單元測試未通過（{}）:\n{}\n{}", target, stdout, String::from_utf8_lossy(&out.stderr)));
        }
        summary.push(format!("{} {}", target, crate::farm_tree::test_pass_summary(&stdout)));
    }
    if let Err(issues) = crate::guard::run_guard_checks(repo_root) {
        return Err(format!("硬體級守衛 (guard.rs) 攔截:\n{}", issues.join("\n")));
    }
    Ok(summary.join("；"))
}

/// 一次嘗試：呼叫模型、寫入模組與測試（含規格黃金向量測試）、驗證
/// 回傳 Ok((供應商, 測試摘要)) 或 Err((供應商, 錯誤輸出))
/// @outputs: 2
/// @domain: attempt [1..20]
/// @domain: shift [0..1000]
fn try_single_farm_attempt(ctx: &FarmContext, feedback: &str, attempt: usize, shift: usize) -> Result<(String, String), (String, String)> {
    if attempt < 1 || attempt > 20 || shift > 1000 {
        return Err(("none".to_string(), "attempt 超出範圍".to_string()));
    }
    let prompt = build_farmer_prompt(&ctx.module, &ctx.spec_text, &ctx.memory, feedback);
    let (provider, raw_resp) = dispatch_cloud_llm(attempt, &prompt, &ctx.repo_root, shift).map_err(|e| ("none".to_string(), e))?;
    let fail = |msg: &str| (provider.clone(), msg.to_string());
    let mod_code = extract_fenced_block(&raw_resp, "rust:module")
        .or_else(|| extract_fenced_block(&raw_resp, "rust"))
        .ok_or_else(|| fail("模型輸出缺少 ```rust:module 標籤"))?;
    let test_code = extract_fenced_block(&raw_resp, "rust:test")
        .or_else(|| extract_second_rust_block(&raw_resp))
        .ok_or_else(|| fail("模型輸出缺少 ```rust:test 標籤"))?;

    let root = &ctx.repo_root;
    let test_name = format!("test_{}", ctx.module);
    let _ = fs::write(root.join("src").join("lib.rs"), insert_mod_alphabetically(&ctx.orig_lib, &ctx.module));
    let _ = fs::write(root.join("src").join(format!("{}.rs", ctx.module)), mod_code);
    let target_use = format!("use rust_bricks::{}::*;", ctx.module);
    let _ = fs::write(root.join("tests").join(format!("{}.rs", test_name)), test_code.replace("use super::*;", &target_use));
    let mut targets = Vec::new();
    if let Some(golden) = &ctx.golden_test {
        let golden_name = format!("golden_{}", ctx.module);
        let _ = fs::write(root.join("tests").join(format!("{}.rs", golden_name)), golden);
        targets.push(golden_name);
    }
    let msa_src = fs::read_to_string(root.join("msa").join(format!("{}.rs", ctx.module))).ok();
    let golden_lines = crate::farm_tree::extract_golden_lines(&ctx.spec_text);
    if let Some(msa_test) = msa_src.and_then(|src| crate::msa::render_msa_test(&ctx.module, &src, &golden_lines, true)) {
        let _ = fs::write(root.join("tests").join(format!("msa_{}.rs", ctx.module)), msa_test);
        targets.push(format!("msa_{}", ctx.module));
    }
    targets.push(test_name);

    let summary = verify_farmed_module(root, &targets).map_err(|e| (provider.clone(), e))?;
    let _ = crate::catalog::generate_catalogs(root);
    let _ = crate::receipt::create_receipt(root, &provider, "PASS");
    Ok((provider, summary))
}

/// @outputs: 2
pub fn check_module_already_claimed(repo_root: &Path, module_name: &str) -> Result<(), String> {
    let mod_path = repo_root.join("src").join(format!("{}.rs", module_name));
    if mod_path.exists() {
        return Err(format!("【防撞車鎖定】本地已存在 `src/{}.rs`，跳過重複開農！", module_name));
    }
    let _ = Command::new("git").args(["fetch", "--all", "--quiet"]).current_dir(repo_root).status();
    let out = Command::new("git").args(["branch", "-r"]).current_dir(repo_root).output();
    if let Ok(o) = out {
        let branches = String::from_utf8_lossy(&o.stdout);
        let target_branch = format!("contrib/cloud-farm-{}", module_name);
        if branches.contains(&target_branch) {
            return Err(format!("【防撞車鎖定】遠端已存在雲端分支 `{}`，其他節點已完成或正在執行！", target_branch));
        }
    }
    Ok(())
}

/// 重試迴圈：二振出局換方向與供應商、第四振升級裁判、API 全斷視為基礎設施問題
/// @outputs: 3
/// @domain: max_attempts [1..10]
fn run_attempt_loop(ctx: &FarmContext, max_attempts: usize) -> LoopResult {
    let mut feedback = String::new();
    let mut strikes = StrikeState::default();
    let mut shift = 0usize;
    let mut rounds: Vec<RoundRecord> = Vec::new();
    let mut last_log = String::new();
    for attempt in 1..=max_attempts.clamp(1, 10) {
        println!("🌾 [cloud-farmer] 第 {}/{} 次嘗試生成微積木 `{}`...", attempt, max_attempts, ctx.module);
        let (provider, log) = match try_single_farm_attempt(ctx, &feedback, attempt, shift) {
            Ok((provider, summary)) => return LoopResult::Success(provider, summary, attempt),
            Err(pair) => pair,
        };
        let class = classify_failure(&log);
        if class == "infra" {
            return LoopResult::Infra(log);
        }
        println!("⚠️ [cloud-farmer] 第 {} 次未過：{}（{}）", attempt, class, provider);
        rounds.push(RoundRecord { model: provider, class: class.clone(), first_line: first_error_line(&log) });
        strikes = next_strike(&strikes, &class);
        match strike_action(&strikes) {
            StrikeAction::Escalate => return LoopResult::Failed(rounds, log, true),
            StrikeAction::SwitchDirection => {
                println!("🧢 [cloud-farmer] 二振出局：{} 連續 {} 次，換方向並換供應商", class, strikes.count);
                shift += 1;
                feedback = format!("{}{}", strike_note(&strikes), log);
            }
            StrikeAction::Continue => feedback = log.clone(),
        }
        last_log = log;
    }
    LoopResult::Failed(rounds, last_log, false)
}

/// 清回農作前的工作區
/// @outputs: 1
fn rollback_workspace(ctx: &FarmContext) {
    let root = &ctx.repo_root;
    let _ = fs::write(root.join("src").join("lib.rs"), &ctx.orig_lib);
    let _ = fs::remove_file(root.join("src").join(format!("{}.rs", ctx.module)));
    let _ = fs::remove_file(root.join("tests").join(format!("test_{}.rs", ctx.module)));
    let _ = fs::remove_file(root.join("tests").join(format!("golden_{}.rs", ctx.module)));
    let _ = crate::msa::sync_msa_tests(root);
}

/// 高風險模組（有 msa/<mod>.rs）開工前先驗量具：量具不可信就不呼叫模型（要修的是規格）
/// @outputs: 2
fn gauge_precheck(repo_root: &Path, module: &str) -> Result<(), String> {
    if !repo_root.join("msa").join(format!("{}.rs", module)).exists() {
        return Ok(());
    }
    let _ = crate::msa::sync_msa_tests(repo_root);
    let target = format!("msa_{}", module);
    let out = Command::new("cargo")
        .args(["test", "-q", "--target-dir", "target/farmer_check", "--test", target.as_str()])
        .current_dir(repo_root)
        .output()
        .map_err(|e| format!("無法執行量具驗收: {}", e))?;
    if out.status.success() {
        return Ok(());
    }
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    Err(first_error_line(&text))
}

/// 這一輪失敗的結論：規格可疑 → 升級裁判 → 卡關 → 一般放棄
/// @outputs: 4
pub fn failure_outcome(fresh: &[DeadEnd], rounds: &[RoundRecord], escalated: bool) -> String {
    let mut samples = dead_end_samples(fresh);
    samples.extend(rounds.iter().map(|r| (r.model.clone(), r.class.clone())));
    if spec_suspect(&samples).is_some() {
        return "suspect".to_string();
    }
    if escalated {
        return "escalated".to_string();
    }
    if fresh.len() + 1 >= BLOCK_ROUNDS {
        return "blocked".to_string();
    }
    "abandoned".to_string()
}

/// 開工前檢查：已入庫／已被領取／本版規格卡關或可疑就不呼叫模型
/// @outputs: 2
fn precheck_farm(repo_root: &Path, module: &str, fresh: &[DeadEnd]) -> Result<(), FarmFailure> {
    let skip = |outcome: &str, detail: String| FarmFailure { outcome: outcome.to_string(), detail, attempt_branch: None };
    check_module_already_claimed(repo_root, module).map_err(|e| skip("skip-claimed", e))?;
    match module_state(false, false, fresh) {
        ModuleState::Blocked(n) => Err(skip("skip-blocked", format!("本版規格已放棄 {} 輪，等待人類裁判", n))),
        ModuleState::Suspect(class) => Err(skip("skip-suspect", format!("不同模型卡在同一個黃金向量（{}），請先複核規格", class))),
        _ => Ok(()),
    }
}

/// 依規格農一塊積木；失敗時把這一輪記成 attempt/ 分支（push 時推上遠端）
/// @outputs: 2
/// @domain: max_attempts [1..10]
pub fn farm_brick_from_spec(repo_root: &Path, spec_path: &Path, module_name: &str, max_attempts: usize, push: bool) -> Result<FarmSuccess, FarmFailure> {
    let fail = |outcome: &str, detail: String| FarmFailure { outcome: outcome.to_string(), detail, attempt_branch: None };
    if max_attempts < 1 || max_attempts > 10 {
        return Err(fail("error", "max_attempts 必須介於 [1..10]".to_string()));
    }
    let spec_str = spec_path.to_string_lossy().replace('\\', "/");
    let spec_hash = spec_blob_hash(repo_root, &spec_str);
    let all_dead_ends = load_dead_ends(repo_root, module_name);
    let fresh = fresh_dead_ends(&all_dead_ends, &spec_hash);
    precheck_farm(repo_root, module_name, &fresh)?;
    gauge_precheck(repo_root, module_name).map_err(|e| fail("gauge-broken", format!("量具驗收未過，請先校正規格：{}", e)))?;
    let spec_text = fs::read_to_string(spec_path).map_err(|e| fail("error", format!("讀取規格書失敗: {}", e)))?;
    let orig_lib = fs::read_to_string(repo_root.join("src").join("lib.rs")).map_err(|e| fail("error", format!("讀取 lib.rs 失敗: {}", e)))?;
    let golden_lines = extract_golden_lines(&spec_text);
    let ctx = FarmContext {
        repo_root: repo_root.to_path_buf(),
        module: module_name.to_string(),
        orig_lib,
        spec_text,
        memory: dead_end_prompt(&fresh),
        golden_test: render_golden_test(module_name, &golden_lines),
    };
    let resolves: Vec<String> = all_dead_ends.iter().map(|d| d.branch.clone()).collect();
    match run_attempt_loop(&ctx, max_attempts) {
        LoopResult::Success(provider, test_summary, attempts_used) => Ok(FarmSuccess {
            provider,
            test_summary,
            golden_count: golden_assert_count(&golden_lines),
            attempts_used,
            spec_hash,
            resolves,
        }),
        LoopResult::Infra(log) => {
            rollback_workspace(&ctx);
            Err(fail("infra", log))
        }
        LoopResult::Failed(rounds, last_log, escalated) => {
            let outcome = failure_outcome(&fresh, &rounds, escalated);
            let input = AbandonInput {
                module: module_name.to_string(),
                spec_path: spec_str,
                spec_hash,
                round: fresh.len() + 1,
                outcome: outcome.clone(),
                rounds,
                evidence: crate::farm_tree::evidence_lines(&last_log),
            };
            let branch = record_abandoned_round(repo_root, module_name, &crate::farm_tree::abandon_message(&input), push).ok();
            rollback_workspace(&ctx);
            Err(FarmFailure { outcome, detail: first_error_line(&last_log), attempt_branch: branch })
        }
    }
}

/// @outputs: 2
fn git_show_file(repo_root: &Path, git_ref: &str) -> Option<String> {
    let out = Command::new("git")
        .args(["show", git_ref])
        .current_dir(repo_root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).to_string())
}

/// @outputs: 2
fn harvest_single_branch(repo_root: &Path, remote_branch: &str, module_name: &str) -> Result<(), String> {
    let mod_path = repo_root.join("src").join(format!("{}.rs", module_name));
    if mod_path.exists() {
        return Err(format!("本地已存在 src/{}.rs，跳過雲端分支 {}", module_name, remote_branch));
    }
    let mod_ref = format!("{}:src/{}.rs", remote_branch, module_name);
    let test_ref = format!("{}:tests/test_{}.rs", remote_branch, module_name);
    let mod_src = git_show_file(repo_root, &mod_ref)
        .ok_or_else(|| format!("無法從 {} 提取 src/{}.rs", remote_branch, module_name))?;
    let test_src = git_show_file(repo_root, &test_ref)
        .ok_or_else(|| format!("無法從 {} 提取 tests/test_{}.rs", remote_branch, module_name))?;
    let golden_src = git_show_file(repo_root, &format!("{}:tests/golden_{}.rs", remote_branch, module_name));

    let lib_path = repo_root.join("src").join("lib.rs");
    let orig_lib = fs::read_to_string(&lib_path).map_err(|e| format!("讀取 lib.rs 失敗: {}", e))?;
    let test_name = format!("test_{}", module_name);
    let test_path = repo_root.join("tests").join(format!("{}.rs", test_name));
    let golden_path = repo_root.join("tests").join(format!("golden_{}.rs", module_name));

    let _ = fs::write(&lib_path, insert_mod_alphabetically(&orig_lib, module_name));
    let _ = fs::write(&mod_path, mod_src);
    let _ = fs::write(&test_path, test_src);
    let mut targets = Vec::new();
    if let Some(golden) = golden_src {
        let _ = fs::write(&golden_path, golden);
        targets.push(format!("golden_{}", module_name));
    }
    if crate::msa::sync_msa_tests(repo_root).iter().any(|m| m == module_name) {
        targets.push(format!("msa_{}", module_name));
    }
    targets.push(test_name);

    if let Err(e) = verify_farmed_module(repo_root, &targets) {
        let _ = fs::write(&lib_path, orig_lib);
        let _ = fs::remove_file(&mod_path);
        let _ = fs::remove_file(&test_path);
        let _ = fs::remove_file(&golden_path);
        let _ = crate::msa::sync_msa_tests(repo_root);
        return Err(format!("收割驗證衝突或失敗 (已自動回滾): {}", e));
    }
    Ok(())
}

/// @outputs: 2
pub fn harvest_cloud_branches(repo_root: &Path) -> Result<Vec<String>, String> {
    let _ = Command::new("git").args(["fetch", "--all", "--prune"]).current_dir(repo_root).status();
    let out = Command::new("git")
        .args(["branch", "-r"])
        .current_dir(repo_root)
        .output()
        .map_err(|e| format!("git branch -r 失敗: {}", e))?;

    let stdout = String::from_utf8_lossy(&out.stdout);
    let prefix = "origin/contrib/cloud-farm-";
    let mut harvested = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        let mod_name = match trimmed.strip_prefix(prefix) {
            Some(m) => m,
            None => continue,
        };
        if let Err(e) = harvest_single_branch(repo_root, trimmed, mod_name) {
            eprintln!("⚠️ [harvest] 跳過 {}: {}", trimmed, e);
            continue;
        }
        harvested.push(mod_name.to_string());
    }
    if !harvested.is_empty() {
        let pruned = crate::farm_tree::prune_resolved_attempts(repo_root, false);
        if !pruned.is_empty() {
            println!("🧹 [harvest] 已入庫模組的本機死路分支已過期並清除：{:?}", pruned);
        }
        let _ = crate::catalog::generate_catalogs(repo_root);
        let _ = crate::receipt::create_receipt(repo_root, "cloud-harvester", "PASS");
    }
    Ok(harvested)
}
