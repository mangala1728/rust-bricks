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

    // 3. 09 號合約靜態規則檢查
    all_issues.extend(check_contract_rules(repo_root));

    if !all_issues.is_empty() {
        return Err(all_issues);
    }

    // 4. 單元測試驗證
    if let Err(test_err) = run_unit_tests(repo_root) {
        all_issues.push(test_err);
        return Err(all_issues);
    }

    Ok(())
}
