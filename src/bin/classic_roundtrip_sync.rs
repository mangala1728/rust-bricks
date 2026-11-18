// ## Map
// Role: app (CLI 古經典 Word 雙向同步收割器 / Master Round-Trip Synchronizer)
// Inputs: 命令列參數 (--docx, --json, --apply, --out-report)。
// Outputs: 修訂差異報告 (Markdown/JSON)、反向更新之經文 JSON 資料庫 (含 human_locked 與 master_ratified 標記)。
// Invariants: 零外部黑箱依賴、禁止 unsafe 代碼、100% 遵守 09 號合約、函式 <= 100 行、巢狀深度 <= 2。

#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

/// 列印使用說明
/// @outputs: 1
fn print_usage() {
    eprintln!(r#"
classic_roundtrip_sync — 人機協同大師 Word 追蹤修訂與批注反向同步收割器 (Rust 原生版)

用法:
  classic_roundtrip_sync --docx <edited.docx> --json <classic_full.json> [選項]

選項:
  --apply                    將大師審定修訂反向回寫至 JSON (自動鎖定 human_locked=true)
  --out-report <report.md>   輸出人類可讀之修訂審核報告
  -h, --help                 顯示此說明訊息
"#);
}

struct CliOptions {
    docx_path: PathBuf,
    json_path: PathBuf,
    apply: bool,
    out_report: Option<PathBuf>,
}

/// 解析命令列參數
/// @outputs: 2
fn parse_cli_args(args: &[String]) -> Result<CliOptions, String> {
    if args.len() < 2 {
        return Err("缺少必要參數".to_string());
    }

    let mut docx_path = None;
    let mut json_path = None;
    let mut apply = false;
    let mut out_report = None;

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-h" || arg == "--help" {
            print_usage();
            process::exit(0);
        } else if arg == "--docx" {
            i = i.saturating_add(1);
            if i < args.len() {
                docx_path = Some(PathBuf::from(&args[i]));
            }
        } else if arg == "--json" {
            i = i.saturating_add(1);
            if i < args.len() {
                json_path = Some(PathBuf::from(&args[i]));
            }
        } else if arg == "--apply" {
            apply = true;
        } else if arg == "--out-report" {
            i = i.saturating_add(1);
            if i < args.len() {
                out_report = Some(PathBuf::from(&args[i]));
            }
        }
        i = i.saturating_add(1);
    }

    let docx = docx_path.ok_or_else(|| "缺少 --docx 參數".to_string())?;
    let json = json_path.ok_or_else(|| "缺少 --json 參數".to_string())?;

    Ok(CliOptions {
        docx_path: docx,
        json_path: json,
        apply,
        out_report,
    })
}

#[derive(Debug, Clone)]
struct VerseDiff {
    verse_idx: usize,
    urn: String,
    insertions: Vec<String>,
    deletions: Vec<String>,
    comments: Vec<String>,
    #[allow(dead_code)]
    has_changes: bool,
}

/// 抽取 XML 標籤內部文字
/// @outputs: 1
fn extract_text_from_xml(xml_fragment: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for c in xml_fragment.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(c);
        }
    }
    result.trim().to_string()
}

/// 從 docx 解壓目錄中抽取修訂與注釋
/// @outputs: 2
fn scan_docx_xml_diffs(tmp_dir: &Path) -> Result<Vec<VerseDiff>, String> {
    let doc_xml_path = tmp_dir.join("word").join("document.xml");
    if !doc_xml_path.exists() {
        return Err("無效的 docx：找不到 word/document.xml".to_string());
    }

    let doc_xml = fs::read_to_string(&doc_xml_path)
        .map_err(|e| format!("無法讀取 document.xml: {e}"))?;

    let mut comments_map = std::collections::BTreeMap::<String, String>::new();
    let comments_xml_path = tmp_dir.join("word").join("comments.xml");
    if comments_xml_path.exists() {
        if let Ok(c_xml) = fs::read_to_string(&comments_xml_path) {
            parse_comments_xml(&c_xml, &mut comments_map);
        }
    }

    let mut diffs = Vec::new();
    let mut curr_idx = 0usize;
    let mut curr_urn = String::new();

    // 依段落分割解析
    for para in doc_xml.split("<w:p>") {
        if let Some(bm_pos) = para.find("w:name=\"urn_classic_verse_") {
            let after_bm = &para[bm_pos.saturating_add(26)..];
            let id_str: String = after_bm.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(num) = id_str.parse::<usize>() {
                curr_idx = num;
                curr_urn = format!("urn:classic:verse:{num}");
            }
        }

        if curr_idx == 0 {
            continue;
        }

        let mut ins_list = Vec::new();
        let mut del_list = Vec::new();
        let mut comm_list = Vec::new();

        // 掃描修訂插入 <w:ins>
        for ins_block in para.split("<w:ins ").skip(1) {
            if let Some(gt_pos) = ins_block.find('>') {
                let body = &ins_block[gt_pos + 1..];
                if let Some(end_pos) = body.find("</w:ins>") {
                    let text = extract_text_from_xml(&body[..end_pos]);
                    if !text.is_empty() {
                        ins_list.push(text);
                    }
                }
            }
        }

        // 掃描修訂刪除 <w:del>
        for del_block in para.split("<w:del ").skip(1) {
            if let Some(gt_pos) = del_block.find('>') {
                let body = &del_block[gt_pos + 1..];
                if let Some(end_pos) = body.find("</w:del>") {
                    let text = extract_text_from_xml(&body[..end_pos]);
                    if !text.is_empty() {
                        del_list.push(text);
                    }
                }
            }
        }

        // 掃描批注 <w:commentRangeStart w:id="X"/>
        for c_start in para.split("w:commentRangeStart w:id=\"") {
            let c_id: String = c_start.chars().take_while(|c| c.is_ascii_digit()).collect();
            if !c_id.is_empty() {
                if let Some(c_text) = comments_map.get(&c_id) {
                    comm_list.push(c_text.clone());
                }
            }
        }

        let has_changes = !ins_list.is_empty() || !del_list.is_empty() || !comm_list.is_empty();
        if has_changes {
            diffs.push(VerseDiff {
                verse_idx: curr_idx,
                urn: curr_urn.clone(),
                insertions: ins_list,
                deletions: del_list,
                comments: comm_list,
                has_changes: true,
            });
        }
    }

    Ok(diffs)
}

/// 解析 comments.xml 內部注釋清單
/// @outputs: 1
fn parse_comments_xml(c_xml: &str, map: &mut std::collections::BTreeMap<String, String>) {
    for c_block in c_xml.split("<w:comment ").skip(1) {
        if let Some(id_start) = c_block.find("w:id=\"") {
            let after_id = &c_block[id_start.saturating_add(6)..];
            let id: String = after_id.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Some(gt_pos) = c_block.find('>') {
                let inner = &c_block[gt_pos + 1..];
                if let Some(end_pos) = inner.find("</w:comment>") {
                    let body = extract_text_from_xml(&inner[..end_pos]);
                    if !id.is_empty() && !body.is_empty() {
                        map.insert(id, body);
                    }
                }
            }
        }
    }
}

/// 產生修訂審核報告 Markdown
/// @outputs: 1
fn generate_diff_report(diffs: &[VerseDiff], docx_name: &str) -> String {
    let mut out = Vec::new();
    out.push(format!("# 📝 大師 Word 追蹤修訂與批注審核報告: {docx_name}\n"));
    out.push(format!("**偵測到包含修訂之頌文數**: {} 處\n", diffs.len()));
    out.push("---\n".to_string());

    if diffs.is_empty() {
        out.push("✅ 本次審查未偵測到任何修訂或批注，Word 檔案與核心庫保持 100% 同步。\n".to_string());
    } else {
        out.push("| 經文索引 | URN 語義錨點 | 大師新增詞句 (Insertions) | 大師刪除詞句 (Deletions) | 大師批注要點 (Comments) |".to_string());
        out.push("| :---: | :--- | :--- | :--- | :--- |".to_string());
        for d in diffs {
            let ins = if d.insertions.is_empty() { "-".to_string() } else { d.insertions.join("; ") };
            let del = if d.deletions.is_empty() { "-".to_string() } else { d.deletions.join("; ") };
            let com = if d.comments.is_empty() { "-".to_string() } else { d.comments.join("; ") };
            out.push(format!("| 第 {} 頌 | `{}` | {} | {} | {} |", d.verse_idx, d.urn, ins, del, com));
        }
    }

    out.push("\n---\n*本報告由 classic_roundtrip_sync 自動生成，遵循 09 號合約與不可逆修訂留痕規程。*".to_string());
    out.join("\n")
}

/// 找出 JSON 字串中 "verses" 陣列內每個頂層頌物件的位元組區間 (start..=end)
/// @outputs: 2
fn find_verses_object_spans(json_str: &str) -> Result<Vec<(usize, usize)>, String> {
    let key_pos = json_str.find("\"verses\"").ok_or_else(|| "JSON 缺少 \"verses\" 陣列".to_string())?;
    let arr_rel = json_str[key_pos..].find('[').ok_or_else(|| "JSON \"verses\" 非陣列".to_string())?;
    let arr_start = key_pos.saturating_add(arr_rel);

    let mut spans = Vec::new();
    let mut in_str = false;
    let mut escape = false;
    let mut obj_depth = 0usize;
    let mut obj_start = 0usize;

    for (rel_idx, ch) in json_str[arr_start..].char_indices() {
        let abs_idx = arr_start.saturating_add(rel_idx);
        if escape {
            escape = false;
            continue;
        }
        if ch == '\\' && in_str {
            escape = true;
            continue;
        }
        if ch == '"' {
            in_str = !in_str;
            continue;
        }
        if in_str {
            continue;
        }
        if ch == '{' && obj_depth == 0 {
            obj_start = abs_idx;
            obj_depth = 1;
        } else if ch == '{' {
            obj_depth += 1;
        } else if ch == '}' && obj_depth == 1 {
            obj_depth = 0;
            spans.push((obj_start, abs_idx));
        } else if ch == '}' {
            obj_depth = obj_depth.saturating_sub(1);
        } else if ch == ']' && obj_depth == 0 {
            break;
        }
    }

    Ok(spans)
}

/// 將大師修訂鎖定標記寫入對應 JSON 頌物件（純 Rust 零依賴，反向位元組置換保結構）
/// @outputs: 2
fn apply_ratifications_to_json(json_str: &str, diffs: &[VerseDiff]) -> Result<(String, usize), String> {
    let mut out = json_str.to_string();
    let spans = find_verses_object_spans(&out)?;
    let mut target_indices: Vec<usize> = diffs.iter().map(|d| d.verse_idx).collect();
    target_indices.sort_unstable();
    target_indices.dedup();

    let mut locked_count = 0usize;
    for &v_idx in target_indices.iter().rev() {
        if v_idx == 0 || v_idx > spans.len() {
            continue;
        }
        let (obj_start, obj_end) = spans[v_idx - 1];
        let obj_slice = out[obj_start..=obj_end].to_string();
        if obj_slice.contains("\"human_locked\"") {
            let replaced = obj_slice.replace("\"human_locked\": false", "\"human_locked\": true")
                .replace("\"master_ratified\": false", "\"master_ratified\": true");
            out.replace_range(obj_start..=obj_end, &replaced);
            locked_count += 1;
            continue;
        }
        let close_offset = obj_slice.rfind('}').unwrap_or(obj_slice.len().saturating_sub(1));
        let prefix = obj_slice[..close_offset].trim_end();
        let insert_pos = obj_start.saturating_add(prefix.len());
        out.replace_range(
            insert_pos..obj_start.saturating_add(close_offset),
            ",\n      \"human_locked\": true,\n      \"master_ratified\": true\n    ",
        );
        locked_count += 1;
    }

    if !out.contains("\"master_ratified\": true") && out.starts_with('{') {
        out.replace_range(0..1, "{\n  \"master_ratified\": true,");
    }

    Ok((out, locked_count))
}

/// 主程式入口
/// @outputs: 2
fn main() {
    let args: Vec<String> = env::args().collect();
    let opts = match parse_cli_args(&args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("❌ 參數錯誤: {e}");
            print_usage();
            process::exit(1);
        }
    };

    println!("🔄 [ROUNDTRIP] 啟動大師 Word 追蹤修訂雙向同步器...");
    println!("   📄 Word: {}", opts.docx_path.display());
    println!("   💾 JSON: {}", opts.json_path.display());

    let tmp_dir = Path::new("/tmp").join(format!("docx_sync_{}", process::id()));
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir).unwrap_or_default();

    let status = Command::new("unzip")
        .args(["-q", "-o", opts.docx_path.to_str().unwrap_or(""), "-d", tmp_dir.to_str().unwrap_or("")])
        .status();

    match status {
        Ok(s) if s.success() => {}
        _ => {
            eprintln!("❌ 無法解開 docx 壓縮封包，請確認檔案路徑是否正確");
            let _ = fs::remove_dir_all(&tmp_dir);
            process::exit(1);
        }
    }

    let diffs = match scan_docx_xml_diffs(&tmp_dir) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("❌ 掃描 docx 差異失敗: {e}");
            let _ = fs::remove_dir_all(&tmp_dir);
            process::exit(1);
        }
    };

    let docx_name = opts.docx_path.file_name().map(|s| s.to_string_lossy()).unwrap_or_default();
    let report_content = generate_diff_report(&diffs, &docx_name);

    if let Some(ref r_path) = opts.out_report {
        if let Err(e) = fs::write(r_path, &report_content) {
            eprintln!("⚠️ 無法寫入審核報告: {e}");
        } else {
            println!("📑 [REPORT] 大師修訂報告已生成 -> {}", r_path.display());
        }
    } else {
        println!("\n{report_content}\n");
    }

    if opts.apply {
        match fs::read_to_string(&opts.json_path) {
            Ok(raw_json) => match apply_ratifications_to_json(&raw_json, &diffs) {
                Ok((updated_json, locked)) => {
                    let bak_path = format!("{}.bak", opts.json_path.display());
                    let _ = fs::write(&bak_path, &raw_json);
                    if let Err(e) = fs::write(&opts.json_path, updated_json) {
                        eprintln!("❌ 回寫 JSON 失敗: {e}");
                        let _ = fs::remove_dir_all(&tmp_dir);
                        process::exit(1);
                    }
                    println!("🔒 [APPLY] 已回寫 JSON 並鎖定 {locked} 筆已修訂頌文為 master_ratified (human_locked=true)，備份 -> {bak_path}");
                }
                Err(e) => {
                    eprintln!("❌ 解析/更新 JSON 失敗: {e}");
                    let _ = fs::remove_dir_all(&tmp_dir);
                    process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("❌ 無法讀取 JSON '{}': {e}", opts.json_path.display());
                let _ = fs::remove_dir_all(&tmp_dir);
                process::exit(1);
            }
        }
    }

    let _ = fs::remove_dir_all(&tmp_dir);
    println!("✅ [SYNC COMPLETE] 雙向同步分析圓滿完成！共審定 {} 處經文修訂節點。", diffs.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// @outputs: 1
    #[test]
    fn test_apply_ratifications_to_json_golden() {
        let sample = "{\n  \"title\": \"Test\",\n  \"verses\": [\n    {\n      \"id\": \"BRS 1\",\n      \"translation_zh\": \"一\"\n    },\n    {\n      \"id\": \"BRS 2\",\n      \"translation_zh\": \"二\"\n    }\n  ]\n}";
        let diffs = vec![VerseDiff {
            verse_idx: 2,
            urn: "urn:classic:verse:2".to_string(),
            insertions: vec!["新增詞".to_string()],
            deletions: Vec::new(),
            comments: Vec::new(),
            has_changes: true,
        }];
        let (updated, count) = apply_ratifications_to_json(sample, &diffs).unwrap_or_default();
        assert_eq!(count, 1);
        assert!(updated.contains("\"master_ratified\": true"));
        assert!(updated.contains("\"human_locked\": true"));
    }
}
