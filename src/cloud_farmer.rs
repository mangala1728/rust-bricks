// ## Map
// Role: core/cloud_farmer — 零外部依賴之雙軌雲端自動農積木引擎（支援多免費帳號 Key 池輪替、實機 Guard 閉環自癒與自動排程）。
// Inputs: 規格書路徑 (specs/*.md 或 wishes/*.md)、目標模組名稱、免費 API 金鑰池環境變數。
// Outputs: 驗證通過並入庫之純 Rust 模組與單元測試，或詳細排障錯誤報告。
// Invariants: 零外部依賴、無遞迴、零全域可變狀態、控制流巢狀深度 <= 2、單一函式 <= 60 行、失敗自動回滾。

use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

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
pub fn build_farmer_prompt(module_name: &str, spec_text: &str, error_feedback: &str) -> String {
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
        "5. 嚴禁未附帶 `// @safety-proof:` 的 `.unwrap()`、`.expect()`、`panic!` 或 `unsafe`。\n",
        "6. 測試必須使用確定性黃金值（Golden Values），禁止弱斷言。\n"
    );

    format!(
        "你是專精零外部依賴純 Rust 演算法與 09 號合約邊界規範的硬體級工程師。\n請根據以下規格書，為 `rust-bricks` 編寫純 Rust 模組 `{0}` 與獨立整合測試：\n\n{1}\n\n{2}\n{3}\n【輸出規範】：請嚴格使用 ```rust:module 與 ```rust:test 兩個標籤輸出完整代碼。\n",
        module_name, spec_text, feedback_section, rules
    )
}

/// @outputs: 2
pub fn extract_fenced_block(raw_text: &str, tag: &str) -> Option<String> {
    let marker = format!("```{}", tag);
    let start_pos = raw_text.find(&marker)?;
    let after_marker = &raw_text[start_pos + marker.len()..];
    let newline_pos = after_marker.find('\n')?;
    let body_start = &after_marker[newline_pos + 1..];
    let end_pos = body_start.find("```")?;
    let extracted = body_start[..end_pos].trim();
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
fn call_curl_json(url: &str, headers: &[String], body_json: &str, repo_root: &Path) -> Result<String, String> {
    let tmp_dir = repo_root.join(".ai-memory");
    let _ = fs::create_dir_all(&tmp_dir);
    let payload_path = tmp_dir.join("farmer_payload.json");
    fs::write(&payload_path, body_json).map_err(|e| format!("寫入請求暫存檔失敗: {}", e))?;

    let mut cmd = Command::new("curl");
    cmd.args(["-s", "-S", "-X", "POST", url, "-H", "User-Agent: rust-bricks/1.0", "--max-time", "60"]);
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
fn call_gemini_endpoint(gkey: &str, escaped_prompt: &str, repo_root: &Path) -> Result<(String, String), String> {
    let model = env::var("FARMER_GEMINI_MODEL").unwrap_or_else(|_| "gemini-3.5-flash".to_string());
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
    let text = call_curl_json(&url, &headers, &body, repo_root)?;
    Ok((format!("gemini:{}", model), text))
}

/// @outputs: 2
fn call_openai_compatible_endpoint(
    url: &str,
    token: &str,
    default_model: &str,
    escaped_prompt: &str,
    repo_root: &Path,
) -> Result<(String, String), String> {
    let model = env::var("FARMER_OPENAI_MODEL").unwrap_or_else(|_| default_model.to_string());
    let body = [
        "\x7B\x22model\x22:\x22",
        &model,
        "\x22,\x22temperature\x22:0.1,\x22messages\x22:[\x7B\x22role\x22:\x22user\x22,\x22content\x22:\x22",
        escaped_prompt,
        "\x22\x7D]\x7D",
    ]
    .concat();
    let headers = vec![
        "Content-Type: application/json".to_string(),
        format!("Authorization: Bearer {}", token),
    ];
    let text = call_curl_json(url, &headers, &body, repo_root)?;
    Ok((format!("cloud:{}", model), text))
}

/// @outputs: 2
/// @domain: attempt [1..20]
fn dispatch_cloud_llm(attempt: usize, prompt: &str, repo_root: &Path) -> Result<(String, String), String> {
    if attempt < 1 || attempt > 20 {
        return Err("嘗試次數超出定義域 [1..20]".to_string());
    }
    let escaped = escape_json_string(prompt);
    let node_offset: usize = env::var("FARMER_NODE_ID").ok().and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
    let idx = (attempt - 1) + node_offset;

    let gemini_pool = resolve_key_pool_strings(
        repo_root,
        "GEMINI_API_KEYS",
        "GEMINI_API_KEY",
        &["gemini", "gemini2", "gemini3", "gemini4"],
    );
    if !gemini_pool.is_empty() {
        let gkey = &gemini_pool[idx % gemini_pool.len()];
        if let Ok(res) = call_gemini_endpoint(gkey, &escaped, repo_root) {
            return Ok(res);
        }
    }

    let groq_pool = resolve_key_pool_strings(
        repo_root,
        "GROQ_API_KEYS",
        "GROQ_API_KEY",
        &["groq", "groq2", "groq3", "groq4"],
    );
    if !groq_pool.is_empty() {
        let qkey = &groq_pool[idx % groq_pool.len()];
        let url = "https://api.groq.com/openai/v1/chat/completions";
        if let Ok(res) = call_openai_compatible_endpoint(url, qkey, "openai/gpt-oss-120b", &escaped, repo_root) {
            return Ok(res);
        }
    }

    if let Ok(gh_token) = env::var("GITHUB_TOKEN") {
        let url = "https://models.inference.ai.azure.com/chat/completions";
        return call_openai_compatible_endpoint(url, &gh_token, "gpt-4o-mini", &escaped, repo_root);
    }

    Err("所有免費 API 金鑰池 (Gemini 4x / Groq 4x / GitHub Models) 皆無法連線或未設定！".to_string())
}

/// @outputs: 2
fn verify_farmed_module(repo_root: &Path, test_target: &str) -> Result<(), String> {
    let test_out = Command::new("cargo")
        .args(["test", "--target-dir", "target/farmer_check", "--test", test_target])
        .current_dir(repo_root)
        .output()
        .map_err(|e| format!("執行 cargo test 失敗: {}", e))?;

    if !test_out.status.success() {
        return Err(format!(
            "單元測試未通過:\n{}\n{}",
            String::from_utf8_lossy(&test_out.stdout),
            String::from_utf8_lossy(&test_out.stderr)
        ));
    }

    if let Err(issues) = crate::guard::run_guard_checks(repo_root) {
        return Err(format!("硬體級守衛 (guard.rs) 攔截:\n{}", issues.join("\n")));
    }
    Ok(())
}

/// @outputs: 2
/// @domain: attempt [1..20]
fn try_single_farm_attempt(
    repo_root: &Path,
    module_name: &str,
    orig_lib: &str,
    spec_text: &str,
    feedback: &str,
    attempt: usize,
) -> Result<String, String> {
    if attempt < 1 || attempt > 20 {
        return Err("attempt 超出範圍".to_string());
    }
    let prompt = build_farmer_prompt(module_name, spec_text, feedback);
    let (provider, raw_resp) = dispatch_cloud_llm(attempt, &prompt, repo_root)?;

    let mod_code = extract_fenced_block(&raw_resp, "rust:module")
        .or_else(|| extract_fenced_block(&raw_resp, "rust"))
        .ok_or_else(|| "模型輸出缺少 ```rust:module 標籤".to_string())?;
    let test_code = extract_fenced_block(&raw_resp, "rust:test")
        .ok_or_else(|| "模型輸出缺少 ```rust:test 標籤".to_string())?;

    let lib_path = repo_root.join("src").join("lib.rs");
    let mod_path = repo_root.join("src").join(format!("{}.rs", module_name));
    let test_name = format!("test_{}", module_name);
    let test_path = repo_root.join("tests").join(format!("{}.rs", test_name));

    let new_lib = insert_mod_alphabetically(orig_lib, module_name);
    let _ = fs::write(&lib_path, new_lib);
    let _ = fs::write(&mod_path, mod_code);
    let _ = fs::write(&test_path, test_code);

    verify_farmed_module(repo_root, &test_name)?;
    let _ = crate::catalog::generate_catalogs(repo_root);
    let _ = crate::receipt::create_receipt(repo_root, &provider, "PASS");
    Ok(provider)
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

/// @outputs: 2
/// @domain: max_attempts [1..10]
pub fn farm_brick_from_spec(
    repo_root: &Path,
    spec_path: &Path,
    module_name: &str,
    max_attempts: usize,
) -> Result<String, String> {
    if max_attempts < 1 || max_attempts > 10 {
        return Err("max_attempts 必須介於 [1..10]".to_string());
    }
    check_module_already_claimed(repo_root, module_name)?;
    let spec_text = fs::read_to_string(spec_path).map_err(|e| format!("讀取規格書失敗: {}", e))?;
    let lib_path = repo_root.join("src").join("lib.rs");
    let orig_lib = fs::read_to_string(&lib_path).map_err(|e| format!("讀取 lib.rs 失敗: {}", e))?;
    let mod_path = repo_root.join("src").join(format!("{}.rs", module_name));
    let test_path = repo_root.join("tests").join(format!("test_{}.rs", module_name));
    let mut feedback = String::new();

    for attempt in 1..=max_attempts {
        println!("🌾 [cloud-farmer] 第 {}/{} 次嘗試生成微積木 `{}`...", attempt, max_attempts, module_name);
        match try_single_farm_attempt(repo_root, module_name, &orig_lib, &spec_text, &feedback, attempt) {
            Ok(provider) => return Ok(provider),
            Err(err_log) => {
                println!("⚠️ [cloud-farmer] 第 {} 次驗證未過，準備自我修復...", attempt);
                feedback = err_log;
            }
        }
    }

    let _ = fs::write(&lib_path, orig_lib);
    let _ = fs::remove_file(&mod_path);
    let _ = fs::remove_file(&test_path);
    Err(format!("超過最大重試次數 ({})，已安全回滾工作區。最後報錯:\n{}", max_attempts, feedback))
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

    let lib_path = repo_root.join("src").join("lib.rs");
    let orig_lib = fs::read_to_string(&lib_path).map_err(|e| format!("讀取 lib.rs 失敗: {}", e))?;
    let test_name = format!("test_{}", module_name);
    let test_path = repo_root.join("tests").join(format!("{}.rs", test_name));

    let new_lib = insert_mod_alphabetically(&orig_lib, module_name);
    let _ = fs::write(&lib_path, new_lib);
    let _ = fs::write(&mod_path, mod_src);
    let _ = fs::write(&test_path, test_src);

    if let Err(e) = verify_farmed_module(repo_root, &test_name) {
        let _ = fs::write(&lib_path, orig_lib);
        let _ = fs::remove_file(&mod_path);
        let _ = fs::remove_file(&test_path);
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
        let _ = crate::catalog::generate_catalogs(repo_root);
        let _ = crate::receipt::create_receipt(repo_root, "cloud-harvester", "PASS");
    }
    Ok(harvested)
}
