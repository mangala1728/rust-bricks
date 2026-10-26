// ## Map
// Role: core/wish_board — AI 代理與使用者跨專案許願池留言板（Wishlist & Feature Requests）。
// Inputs: wishes/ 目錄路徑、許願單 ID、標題、提報代理名稱。
// Outputs: 許願單清單 Vec<WishItem>、生成之許願單路徑 Result<String, String>。
// Invariants: 零外部依賴、無遞迴、零全域可變狀態、有界檔案掃描 (<= 1000 檔案)、控制流巢狀深度 <= 2。

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct WishItem {
    pub file_name: String,
    pub title: String,
    pub requester: String,
    pub status: String,
}

/// @outputs: 1
fn parse_wish_content(fname: &str, content: &str) -> WishItem {
    let mut title = "未命名需求".to_string();
    let mut requester = "匿名代理".to_string();
    let mut status = "PENDING".to_string();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") || trimmed.starts_with("title:") {
            let t = trimmed.trim_start_matches("# ").trim_start_matches("title:").trim();
            if !t.is_empty() { title = t.to_string(); }
        } else if trimmed.starts_with("requester:") {
            let r = trimmed.trim_start_matches("requester:").trim();
            if !r.is_empty() { requester = r.to_string(); }
        } else if trimmed.starts_with("status:") {
            let s = trimmed.trim_start_matches("status:").trim();
            if !s.is_empty() { status = s.to_ascii_uppercase(); }
        }
    }

    WishItem {
        file_name: fname.to_string(),
        title,
        requester,
        status,
    }
}

/// @outputs: 2
/// @domain: max_scan [1..1000]
pub fn list_wishes(wishes_dir: &Path, max_scan: usize) -> Result<Vec<WishItem>, String> {
    if max_scan == 0 || max_scan > 1000 {
        return Err("掃描數量超出有界範圍".to_string());
    }
    if !wishes_dir.exists() {
        let _ = fs::create_dir_all(wishes_dir);
        return Ok(Vec::new());
    }
    let entries = fs::read_dir(wishes_dir).map_err(|e| format!("無法讀取 wishes 目錄: {}", e))?;
    let mut list = Vec::new();
    let mut count = 0usize;

    for entry in entries.flatten() {
        if count >= max_scan {
            break;
        }
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
        if fname == "README.md" {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&path) {
            list.push(parse_wish_content(&fname, &content));
            count += 1;
        }
    }
    Ok(list)
}

/// @outputs: 2
/// @domain: id_len [1..64]
/// @domain: title_len [1..128]
pub fn create_wish_template(
    wishes_dir: &Path,
    wish_id: &str,
    title: &str,
    requester: &str,
) -> Result<PathBuf, String> {
    let id_len = wish_id.len();
    let title_len = title.len();
    if id_len == 0 || id_len > 64 || title_len == 0 || title_len > 128 {
        return Err("許願單 ID 或標題長度超出有界範圍".to_string());
    }
    if !wishes_dir.exists() {
        fs::create_dir_all(wishes_dir).map_err(|e| format!("建立 wishes 目錄失敗: {}", e))?;
    }
    let fname = format!("{}.md", wish_id);
    let target_path = wishes_dir.join(&fname);

    let template = format!(
        "# {}\n\n\
        wish_id: {}\n\
        requester: {}\n\
        status: PENDING\n\
        priority: MEDIUM\n\
        created_at: 2026-10-26\n\n\
        ## 1. 需求背景與場景（Context）\n\
        - 使用專案：\n\
        - 遇到瓶頸：\n\n\
        ## 2. 原語言代碼或演算法本質（Source Algorithm）\n\
        - 原語言：(Python / C++ / Java / TypeScript)\n\
        - 原始碼或公式連結：\n\n\
        ## 3. 期望的 Rust 微積木簽章（Target Signature）\n\
        ```rust\n\
        // /// @outputs: N\n\
        // /// @domain: param [min..max]\n\
        // pub fn my_brick(param: f32) -> Result<f32, ()>;\n\
        ```\n\n\
        ## 4. 效益與指標（ROI）\n\
        - 記憶體優化：\n\
        - 速度提升：\n",
        title, wish_id, requester
    );

    fs::write(&target_path, template).map_err(|e| format!("寫入許願單失敗: {}", e))?;
    Ok(target_path)
}
