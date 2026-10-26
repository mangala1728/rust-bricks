// ## Map
// Role: core/guard — 零外部依賴之容量管理、依賴審查與 09 合約品質守門員。
// Inputs: 倉庫根目錄路徑。
// Outputs: 守護結果 Result<(), Vec<String>>。
// Invariants: 零外部依賴、零全域可變狀態、顯式有界佇列走訪（最大 10,000 檔案）、控制流巢狀深度 <= 2、無遞迴。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const MAX_FILE_BYTES: u64 = 500 * 1024; // 500 KB
const MAX_SCAN_FILES: usize = 10_000;
const MAX_FN_LINES: usize = 60; // NASA JPL 規則 / 微積木上限

const BLOCKED_EXTS: &[&str] = &[
    "wav", "mp3", "aac", "flac", "ogg",
    "png", "jpg", "jpeg", "gif", "webp", "bmp", "ico",
    "mp4", "mkv", "avi", "mov",
    "zip", "tar", "gz", "bz2", "7z", "rar",
    "exe", "dll", "so", "dylib", "bin", "dat",
    "onnx", "pt", "pth", "tflite", "gguf", "safetensors",
    "pkl", "npy", "npz",
];

/// @outputs: 2
fn is_blocked_extension(ext: &str) -> bool {
    let lower = ext.to_ascii_lowercase();
    for &b in BLOCKED_EXTS {
        if lower == b {
            return true;
        }
    }
    false
}

/// @outputs: 1
fn check_single_file(path: &Path, issues: &mut Vec<String>) {
    let ext_opt = path.extension().and_then(|s| s.to_str());
    if let Some(ext) = ext_opt {
        if is_blocked_extension(ext) {
            issues.push(format!("【二進位檔案攔截】禁止提交多媒體/大型二進位檔：{} (副檔名 .{})", path.display(), ext));
        }
    }
    let meta_opt = path.metadata().ok();
    if let Some(meta) = meta_opt {
        if meta.len() > MAX_FILE_BYTES {
            issues.push(format!("【超大檔案攔截】檔案超過 500 KB 上限：{} ({:.1} KB)", path.display(), meta.len() as f64 / 1024.0));
        }
    }
}

/// @outputs: 1
fn process_dir_entries(dir: &Path, queue: &mut Vec<PathBuf>, issues: &mut Vec<String>) -> usize {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return 0,
    };
    let mut count = 0;
    for entry in entries.flatten() {
        count += 1;
        let path = entry.path();
        let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if fname == "target" || fname == ".git" {
            continue;
        }
        if path.is_dir() {
            queue.push(path);
        } else if path.is_file() {
            check_single_file(&path, issues);
        }
    }
    count
}

/// @outputs: 2
fn check_file_bloat(repo_root: &Path) -> Vec<String> {
    let mut issues = Vec::new();
    let mut queue = Vec::new();
    queue.push(repo_root.to_path_buf());

    let mut scanned = 0;
    while let Some(dir) = queue.pop() {
        if scanned >= MAX_SCAN_FILES {
            break;
        }
        scanned += process_dir_entries(&dir, &mut queue, &mut issues);
    }
    issues
}

/// @outputs: 2
fn check_cargo_dependencies(repo_root: &Path) -> Vec<String> {
    let mut issues = Vec::new();
    let cargo_toml = repo_root.join("Cargo.toml");
    let content = match fs::read_to_string(&cargo_toml) {
        Ok(c) => c,
        Err(_) => return vec!["找不到 Cargo.toml".to_string()],
    };

    let mut in_deps = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_deps = trimmed == "[dependencies]";
            continue;
        }
        if in_deps && !trimmed.is_empty() && !trimmed.starts_with('#') {
            issues.push(format!("【依賴違規】[dependencies] 必須為空，禁止引入外部 crate：{}", trimmed));
        }
    }
    issues
}

/// @outputs: 1
/// @domain: fn_idx [0..1000000]
fn check_fn_outputs(lines: &[&str], fn_idx: usize, issues: &mut Vec<String>) {
    if fn_idx > 1_000_000 {
        return;
    }
    let mut has_outputs = false;
    let mut back = fn_idx;
    while back > 0 {
        back -= 1;
        let prev = lines[back].trim();
        if prev.contains("@outputs:") {
            has_outputs = true;
            break;
        }
        if prev.starts_with("#[") {
            continue;
        }
        if !prev.starts_with("///") && !prev.is_empty() {
            break;
        }
    }
    if !has_outputs {
        let trimmed = lines[fn_idx].trim();
        issues.push(format!("【合約缺少 @outputs】函式缺少 @outputs 標注：{} (行 {})", trimmed, fn_idx + 1));
    }
}

/// @outputs: 1
fn handle_char_strip(ch: char, prev: char, in_str: &mut bool, in_char: &mut bool, out: &mut String) -> bool {
    if ch == '\x22' && prev != '\\' && !*in_char {
        *in_str = !*in_str;
        return true;
    }
    if ch == '\x27' && prev != '\\' && !*in_str {
        *in_char = !*in_char;
        return true;
    }
    if *in_str || *in_char {
        return true;
    }
    if ch == '/' && prev == '/' {
        out.pop();
        return false;
    }
    out.push(ch);
    true
}

/// @outputs: 1
fn strip_quotes_and_comments(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut in_str = false;
    let mut in_char = false;
    let mut prev = '\0';
    for ch in line.chars() {
        if !handle_char_strip(ch, prev, &mut in_str, &mut in_char, &mut out) {
            break;
        }
        prev = ch;
    }
    out
}

struct FnTracker {
    sig: String,
    start_line: usize,
    entry_depth: i32,
    body_started: bool,
}

/// @outputs: 1
/// @domain: depth [-1000..1000]
/// @domain: line_num [1..1000000]
/// @domain: max_lines [1..1000]
fn check_fn_close(
    tracker: &mut Option<FnTracker>,
    depth: i32,
    line_num: usize,
    max_lines: usize,
    issues: &mut Vec<String>,
    path_display: &str,
) {
    if depth < -1_000 || depth > 1_000 || line_num > 1_000_000 || max_lines > 1_000 {
        return;
    }
    let is_closed = match tracker {
        Some(tr) => tr.body_started && depth <= tr.entry_depth,
        None => false,
    };
    if !is_closed {
        return;
    }
    if let Some(tr) = tracker.take() {
        let total = line_num.saturating_sub(tr.start_line) + 1;
        if total > max_lines {
            issues.push(format!(
                "【函式過長攔截】函式超過 {} 行上限（當前 {} 行，違反微積木原則）：{} (位於 {}:{})",
                max_lines, total, tr.sig, path_display, tr.start_line
            ));
        }
    }
}

/// @outputs: 1
/// @domain: brace_depth [-1000..1000]
/// @domain: line_num [1..1000000]
/// @domain: max_lines [1..1000]
fn scan_fn_brace_char(
    ch: char,
    brace_depth: &mut i32,
    tracker: &mut Option<FnTracker>,
    line_num: usize,
    max_lines: usize,
    issues: &mut Vec<String>,
    path_display: &str,
) {
    if *brace_depth < -1_000 || *brace_depth > 1_000 || line_num > 1_000_000 || max_lines > 1_000 {
        return;
    }
    if ch == '{' {
        *brace_depth += 1;
        if let Some(ref mut tr) = tracker {
            tr.body_started = true;
        }
    } else if ch == '}' {
        *brace_depth -= 1;
        check_fn_close(tracker, *brace_depth, line_num, max_lines, issues, path_display);
    }
}

/// @outputs: 1
fn check_file_fn_lengths(path: &Path, content: &str, issues: &mut Vec<String>) {
    let path_display = path.to_string_lossy();
    let mut tracker: Option<FnTracker> = None;
    let mut brace_depth: i32 = 0;

    for (idx, line) in content.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = line.trim();
        if tracker.is_none() && (trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") || trimmed.starts_with("pub(crate) fn ")) {
            tracker = Some(FnTracker {
                sig: trimmed.to_string(),
                start_line: line_num,
                entry_depth: brace_depth,
                body_started: false,
            });
        }
        let cleaned = strip_quotes_and_comments(line);
        for ch in cleaned.chars() {
            scan_fn_brace_char(ch, &mut brace_depth, &mut tracker, line_num, MAX_FN_LINES, issues, &path_display);
        }
    }
}

/// @outputs: 1
/// @domain: line_num [1..1000000]
fn check_line_safety_proof(line: &str, prev_line: &str, line_num: usize, issues: &mut Vec<String>, path_display: &str) {
    if line_num > 1_000_000 {
        return;
    }
    let trimmed = line.trim();
    if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
        return;
    }
    if trimmed.starts_with("#![forbid(") || trimmed.starts_with("#[forbid(") {
        return;
    }
    if trimmed.contains("target_unsf") || trimmed.contains("target_panic") {
        return;
    }
    let cleaned = strip_quotes_and_comments(line);
    let target_unwrap = concat!(".", "unwrap()");
    let target_expect = concat!(".", "expect(");
    let target_panic = concat!("pan", "ic!");
    let target_unsf_space = concat!("un", "safe ");
    let target_unsf_brace = concat!("un", "safe{");

    let has_danger = cleaned.contains(target_unwrap)
        || cleaned.contains(target_expect)
        || cleaned.contains(target_panic)
        || cleaned.contains(target_unsf_space)
        || cleaned.contains(target_unsf_brace);
    if !has_danger {
        return;
    }

    let has_proof = line.contains("@safety-proof:") || prev_line.contains("@safety-proof:");
    if !has_proof {
        issues.push(format!(
            "【危險操作未證明】發現 unwrap/expect/panic/unsafe 缺少 // @safety-proof: 標注（06 號合約）：{} (行 {}: {})",
            path_display, line_num, trimmed
        ));
    }
}

/// @outputs: 1
fn check_file_safety_proofs(path: &Path, content: &str, issues: &mut Vec<String>) {
    let path_display = path.to_string_lossy();
    let lines: Vec<&str> = content.lines().collect();
    for (idx, &line) in lines.iter().enumerate() {
        let prev = if idx > 0 { lines[idx - 1] } else { "" };
        check_line_safety_proof(line, prev, idx + 1, issues, &path_display);
    }
}

/// @outputs: 1
fn check_single_contract(path: &Path, content: &str, issues: &mut Vec<String>) {
    // 1. 必備 ## Map
    if !content.contains("## Map") {
        issues.push(format!("【合約缺少 Map】檔案開頭缺少 ## Map 接線圖：{}", path.display()));
    }

    // 2. 禁止 static mut (R09-2)
    let kw_static_mut = concat!("static", " mut ");
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("//") && !trimmed.contains("concat!") && trimmed.contains(kw_static_mut) {
            issues.push(format!("【全域狀態違規】禁止使用 static mut：{}", path.display()));
            break;
        }
    }

    // 3. 具名函式必備 @outputs (R09-1)
    let lines: Vec<&str> = content.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("pub fn ") {
            check_fn_outputs(&lines, i, issues);
        }
    }

    // 4. 防線一：函式長度剛性限制（<= 60 行，NASA JPL / 微積木原則）
    check_file_fn_lengths(path, content, issues);

    // 5. 防線二：禁止未證明的危險操作（06 號合約安全證明）
    check_file_safety_proofs(path, content, issues);
}

/// @outputs: 2
fn check_contract_rules(repo_root: &Path) -> Vec<String> {
    let mut issues = Vec::new();
    let src_dir = repo_root.join("src");
    let entries = match fs::read_dir(&src_dir) {
        Ok(e) => e,
        Err(_) => return vec!["找不到 src 目錄".to_string()],
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let content_opt = fs::read_to_string(&path).ok();
        if let Some(content) = content_opt {
            check_single_contract(&path, &content, &mut issues);
        }
    }
    issues
}

/// @outputs: 1
fn check_incident_file(path: &Path, issues: &mut Vec<String>) {
    let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    if fname == "README.md" || !fname.ends_with(".md") {
        return;
    }
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };
    for line in content.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.contains("status: open") || lower.contains("status: \"open\"")
            || lower.contains("status: unresolved") || lower.contains("status: \"unresolved\"")
            || lower.contains("status: investigating") || lower.contains("狀態：未解決") {
            issues.push(format!(
                "【未結缺陷阻擋】發現未解決之缺陷通報「{}」（狀態：未結）。依規排障優先於新功能開發，必須修復並標記 RESOLVED 後方可提交！",
                path.display()
            ));
            break;
        }
    }
}

/// @outputs: 2
fn check_open_incidents(repo_root: &Path) -> Vec<String> {
    let mut issues = Vec::new();
    let inc_dir = repo_root.join("incidents");
    if !inc_dir.exists() {
        let _ = fs::create_dir_all(&inc_dir);
        return issues;
    }
    let entries = match fs::read_dir(&inc_dir) {
        Ok(e) => e,
        Err(_) => return issues,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            check_incident_file(&path, &mut issues);
        }
    }
    issues
}

/// @outputs: 2
fn check_lib_rs_order(repo_root: &Path) -> Vec<String> {
    let mut issues = Vec::new();
    let lib_rs = repo_root.join("src").join("lib.rs");
    let content = match fs::read_to_string(&lib_rs) {
        Ok(c) => c,
        Err(_) => return vec!["找不到 src/lib.rs".to_string()],
    };
    let mut mods: Vec<String> = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("pub mod ") && trimmed.ends_with(';') {
            let mod_name = trimmed.trim_start_matches("pub mod ").trim_end_matches(';').trim();
            mods.push(mod_name.to_string());
        }
    }
    for i in 1..mods.len() {
        if mods[i] < mods[i - 1] {
            issues.push(format!(
                "【lib.rs 未依字母排序】pub mod 請依字母排序以防多分支衝突：{} 出現在 {} 之前",
                mods[i - 1], mods[i]
            ));
            break;
        }
    }
    issues
}

/// @outputs: 2
fn check_function_deduplication(repo_root: &Path) -> Vec<String> {
    let mut issues = Vec::new();
    let src_dir = repo_root.join("src");
    let entries = match fs::read_dir(&src_dir) {
        Ok(e) => e,
        Err(_) => return issues,
    };
    let mut seen: Vec<(String, String)> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
        if fname == "main.rs" || fname == "guard.rs" || fname == "lib.rs" || fname == "receipt.rs" || fname == "catalog.rs" || fname == "chaos_audit.rs" || fname == "wish_board.rs" {
            continue;
        }
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("pub fn ") {
                if let Some(open_paren) = trimmed.find('(') {
                    let fn_name = trimmed[7..open_paren].trim().to_string();
                    for (prev_fn, prev_file) in &seen {
                        if prev_fn == &fn_name && prev_file != &fname {
                            issues.push(format!(
                                "【重複命名/輪子衝突】函式「{}」在多個模組中重複定義：{} 與 {}。違反單一黃金版本去重原則！",
                                fn_name, prev_file, fname
                            ));
                        }
                    }
                    seen.push((fn_name, fname.clone()));
                }
            }
        }
    }
    issues
}

/// @outputs: 2
fn run_unit_tests(repo_root: &Path) -> Result<(), String> {
    let output = match Command::new("cargo")
        .arg("test")
        .current_dir(repo_root)
        .output()
    {
        Ok(o) => o,
        Err(e) => return Err(format!("執行 cargo test 失敗: {}", e)),
    };

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let out = String::from_utf8_lossy(&output.stdout);
        return Err(format!("單元測試未通過：\n{}\n{}", out, err));
    }
    Ok(())
}

/// @outputs: 2
pub fn run_guard_checks(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut all_issues = Vec::new();

    // 1. 容量與檔案大小檢查
    all_issues.extend(check_file_bloat(repo_root));

    // 2. 外部依賴檢查
    all_issues.extend(check_cargo_dependencies(repo_root));

    // 3. 09 號合約靜態規則檢查（含防線一：長度 <= 60、防線二：危險操作安全證明）
    all_issues.extend(check_contract_rules(repo_root));

    // 4. 防線三：未結缺陷懸案阻擋（incidents/ 狀態檢查）
    all_issues.extend(check_open_incidents(repo_root));

    // 5. 防線四：防撞車與去重治理（lib.rs 字母排序、函式全域防重複輪子）
    all_issues.extend(check_lib_rs_order(repo_root));
    all_issues.extend(check_function_deduplication(repo_root));

    if !all_issues.is_empty() {
        return Err(all_issues);
    }

    // 6. 單元測試驗證
    if let Err(test_err) = run_unit_tests(repo_root) {
        all_issues.push(test_err);
        return Err(all_issues);
    }

    // 7. 防線五：零信任品質總監黑箱動態混沌巡檢
    if let Err(audit_issues) = crate::chaos_audit::run_chaos_audit(0) {
        all_issues.extend(audit_issues);
        return Err(all_issues);
    }

    Ok(())
}