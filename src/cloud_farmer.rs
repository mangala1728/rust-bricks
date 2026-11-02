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
    raw.replace("\\n", "\n")
        .replace("\\r", "\r")
        .replace("\\t", "\t")
        .replace("\\\x22", "\x22")
        .replace("\\\\", "\\")
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
    cmd.args(["-s", "-S", "-X", "POST", url]);
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

/// @outputs: 2
/// @domain: attempt [1..20]
fn dispatch_cloud_llm(attempt: usize, prompt: &str, repo_root: &Path) -> Result<(String, String), String> {
    if attempt < 1 || attempt > 20 {
        return Err("嘗試次數超出定義域 [1..20]".to_string());
    }
    let escaped = escape_json_string(prompt);
    let gemini_raw = env::var("GEMINI_API_KEYS")
        .or_else(|_| env::var("GEMINI_API_KEY"))
        .unwrap_or_default();
    let gemini_pool = parse_api_key_pool(&gemini_raw);

    if let Some(gkey) = select_rotating_key(&gemini_pool, attempt) {
        let model = env::var("FARMER_GEMINI_MODEL").unwrap_or_else(|_| "gemini-2.5-flash".to_string());
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model, gkey
        );
        let body = [
            "\x7B\x22contents\x22:[\x7B\x22parts\x22:[\x7B\x22text\x22:\x22",
            &escaped,
            "\x22\x7D]\x7D],\x22generationConfig\x22:\x7B\x22temperature\x22:0.1\x7D\x7D",
        ]
        .concat();
        let headers = vec!["Content-Type: application/json".to_string()];
        let text = call_curl_json(&url, &headers, &body, repo_root)?;
        return Ok((format!("gemini:{}", model), text));
    }

    let gh_or_groq = env::var("GROQ_API_KEY")
        .map(|k| ("https://api.groq.com/openai/v1/chat/completions", k, "llama-3.3-70b-versatile"))
        .or_else(|_| {
            env::var("GITHUB_TOKEN").map(|k| {
                ("https://models.inference.ai.azure.com/chat/completions", k, "gpt-4o-mini")
            })
        });

    if let Ok((url, token, default_model)) = gh_or_groq {
        let model = env::var("FARMER_OPENAI_MODEL").unwrap_or_else(|_| default_model.to_string());
        let body = [
            "\x7B\x22model\x22:\x22",
            &model,
            "\x22,\x22temperature\x22:0.1,\x22messages\x22:[\x7B\x22role\x22:\x22user\x22,\x22content\x22:\x22",
            &escaped,
            "\x22\x7D]\x7D",
        ]
        .concat();
        let headers = vec![
            "Content-Type: application/json".to_string(),
            format!("Authorization: Bearer {}", token),
        ];
        let text = call_curl_json(url, &headers, &body, repo_root)?;
        return Ok((format!("cloud:{}", model), text));
    }

    Err("未偵測到免費 API 金鑰！請設定 GEMINI_API_KEY、GEMINI_API_KEYS、GROQ_API_KEY 或 GITHUB_TOKEN".to_string())
}

/// @outputs: 2
fn verify_farmed_module(repo_root: &Path, test_target: &str) -> Result<(), String> {
    let test_out = Command::new("cargo")
        .args(["test", "--test", test_target])
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
