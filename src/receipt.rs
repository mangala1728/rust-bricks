// ## Map
// Role: core/receipt — 本地 AI 決策模型審查收據簽發與 Diff 綁定校驗。
// Inputs: Git Diff 輸出、模型標籤、裁決狀態。
// Outputs: 收據 JSON 檔案與校驗結果 Result<(), String>。
// Invariants: 零外部依賴、無遞迴、零全域可變狀態、控制流巢狀深度 <= 2。

use crate::crypto_vault;
use std::fs;
use std::path::Path;
use std::process::Command;

/// @outputs: 2
fn get_git_diff() -> Result<Vec<u8>, String> {
    let output = Command::new("git")
        .args(["diff", "HEAD"])
        .output()
        .map_err(|e| format!("執行 git diff 失敗: {}", e))?;

    if !output.status.success() {
        return Err("git diff 執行失敗".to_string());
    }
    if output.stdout.is_empty() {
        let head_prev = Command::new("git")
            .args(["diff", "HEAD~1..HEAD"])
            .output()
            .map_err(|e| format!("執行 git diff HEAD~1 失敗: {}", e))?;
        return Ok(head_prev.stdout);
    }
    Ok(output.stdout)
}

/// @outputs: 1
/// @domain: model_len [1..128]
fn generate_receipt_json(model: &str, verdict: &str, diff_hash: &str) -> String {
    let model_len = model.len();
    if model_len == 0 || model_len > 128 {
        return String::new();
    }
    let mut json = String::with_capacity(256);
    json.push_str("{\n");
    json.push_str(&format!("  \"schema\": \"agy-arbitration-receipt-v1\",\n"));
    json.push_str(&format!("  \"model\": \"{}\",\n", model.replace('\x22', "")));
    json.push_str(&format!("  \"verdict\": \"{}\",\n", verdict.replace('\x22', "")));
    json.push_str(&format!("  \"diff_sha256\": \"{}\"\n", diff_hash));
    json.push_str("}\n");
    json
}

/// @outputs: 2
pub fn create_receipt(repo_root: &Path, model: &str, verdict: &str) -> Result<String, String> {
    let diff = get_git_diff()?;
    if diff.is_empty() {
        return Err("當前工作區與 HEAD 無任何 Diff 變更，無須簽發收據".to_string());
    }
    let diff_hash = crypto_vault::sha256_hex(&diff);
    let receipt_json = generate_receipt_json(model, verdict, &diff_hash);
    if receipt_json.is_empty() {
        return Err("模型名稱不符合邊界規範".to_string());
    }

    let mem_dir = repo_root.join(".ai-memory");
    if !mem_dir.exists() {
        let _ = fs::create_dir_all(&mem_dir);
    }
    let receipt_path = mem_dir.join("receipt.json");
    fs::write(&receipt_path, &receipt_json).map_err(|e| format!("寫入 receipt.json 失敗: {}", e))?;

    Ok(diff_hash)
}

/// @outputs: 1
fn parse_json_value(content: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\":", key);
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&pattern) {
            let after = trimmed[pattern.len()..].trim();
            let clean = after.trim_matches(|c| c == '\x22' || c == ',' || c == ' ');
            return Some(clean.to_string());
        }
    }
    None
}

/// @outputs: 2
pub fn verify_receipt(repo_root: &Path) -> Result<String, String> {
    let receipt_path = repo_root.join(".ai-memory").join("receipt.json");
    if !receipt_path.exists() {
        return Err("找不到 .ai-memory/receipt.json 收據檔案".to_string());
    }
    let content = fs::read_to_string(&receipt_path).map_err(|e| format!("讀取收據失敗: {}", e))?;
    let expected_hash = match parse_json_value(&content, "diff_sha256") {
        Some(h) => h,
        None => return Err("收據格式損壞：缺少 diff_sha256".to_string()),
    };
    let model = parse_json_value(&content, "model").unwrap_or_else(|| "unknown".to_string());
    let verdict = parse_json_value(&content, "verdict").unwrap_or_else(|| "UNKNOWN".to_string());

    let current_diff = get_git_diff()?;
    let current_hash = crypto_vault::sha256_hex(&current_diff);

    if !crypto_vault::constant_time_eq(expected_hash.as_bytes(), current_hash.as_bytes()) {
        return Err(format!(
            "【收據校驗失敗】代碼 Diff 雜湊與收據不吻合！\n  預期（收據簽署）：{}\n  實際（當前變更）：{}\n  警告：代碼在簽發收據後遭到竄改！",
            expected_hash, current_hash
        ));
    }

    Ok(format!("模型 [{}] 裁決 [{}]，SHA-256 驗證完全吻合", model, verdict))
}