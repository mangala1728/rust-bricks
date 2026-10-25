// ## Map
// Role: core/sys_path — 跨平台路徑防穿越校驗、斜線規範化與史實層只增不刪 (Append-Only) 差異校驗。
// Inputs: 路徑字串、新舊文本。
// Outputs: 安全性布林、規範化字串、單調只增不刪檢查結果元組。
// Invariants: 零全域可變狀態、無遞迴、字串長度有界防護（最大 65,536 字元）。

/// @outputs: 2
pub fn is_safe_rel_path(path_str: &str) -> bool {
    if path_str.is_empty() || path_str.len() > 4096 {
        return false;
    }
    // 禁止 null byte
    if path_str.contains('\0') {
        return false;
    }
    // 禁止絕對路徑開頭
    if path_str.starts_with('/') || path_str.starts_with('\\') {
        return false;
    }
    // 禁止 Windows 磁碟機代號 (例如 C:)
    let bytes = path_str.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' {
        return false;
    }
    // 禁止目錄穿越 (..)
    let normalized = path_str.replace('\\', "/");
    for seg in normalized.split('/') {
        if seg == ".." {
            return false;
        }
    }
    true
}

/// @outputs: 1
pub fn normalize_slashes(path_str: &str) -> String {
    if path_str.len() > 4096 {
        return String::new();
    }
    path_str.replace('\\', "/")
}

/// @outputs: 2
pub fn verify_append_only(old_text: &str, new_text: &str) -> (bool, Option<usize>) {
    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();

    if new_lines.len() < old_lines.len() {
        return (false, Some(new_lines.len() + 1));
    }

    for (idx, &old_line) in old_lines.iter().enumerate() {
        if new_lines[idx] != old_line {
            return (false, Some(idx + 1));
        }
    }

    (true, None)
}
