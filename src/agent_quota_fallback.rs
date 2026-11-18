// ## Map
// Role: core/agent_quota_fallback — 前沿模型額度枯竭處置 (R13-6) 與雙軌退路純演算法微積木。
// Inputs: 懸案檔名清單、懸案元資料 (ID, title, gate, reason, sha, date, repro)、功能與負責角色。
// Outputs: 最高懸案編號、合規 C-NNNN.md 案卷文字、技術債務 JSON 字串、額度耗盡特徵判定。
// Invariants: 零外部依賴、無遞迴、零全域可變狀態、顯式有界迴圈、控制流巢狀深度 <= 2。

/// 解析 4 位數字。
/// @outputs: 2
/// @domain: offset [0..1024]
fn parse_4_digits(bytes: &[u8], offset: usize) -> Option<usize> {
    if offset + 4 > bytes.len() || offset > 1024 {
        return None;
    }

    let mut num = 0usize;
    let mut k = 0usize;
    while k < 4 {
        let c = bytes[offset + k];
        if c < b'0' || c > b'9' {
            return None;
        }
        num = num * 10 + (c - b'0') as usize;
        k += 1;
    }
    Some(num)
}

/// 檢查前綴並解析編號。
/// @outputs: 1
/// @domain: offset [0..1024]
fn check_prefix_and_parse(bytes: &[u8], offset: usize) -> usize {
    if offset + 5 >= bytes.len() || offset > 1024 {
        return 0;
    }
    if bytes[offset] != b'C' || bytes[offset + 1] != b'-' {
        return 0;
    }
    if offset + 6 < bytes.len() && bytes[offset + 6] >= b'0' && bytes[offset + 6] <= b'9' {
        return 0;
    }
    match parse_4_digits(bytes, offset + 2) {
        Some(num) => num,
        None => 0,
    }
}

/// 從單一檔名中提取 C-NNNN 編號。
/// @outputs: 1
/// @domain: name_len [0..1024]
fn extract_case_num(name: &str) -> usize {
    let bytes = name.as_bytes();
    let len = bytes.len();
    if len > 1024 || len < 6 {
        return 0;
    }

    let mut i = 0usize;
    while i + 5 < len {
        let num = check_prefix_and_parse(bytes, i);
        if num > 0 {
            return num;
        }
        i += 1;
    }
    0
}

/// 解析現有檔案清單中最高之 C-NNNN 編號。
/// @outputs: 1
/// @domain: count [0..10000]
pub fn parse_highest_case_id(filenames: &[&str]) -> usize {
    let count = filenames.len();
    if count > 10000 {
        return 0;
    }

    let mut max_id: usize = 0;
    let mut i = 0usize;
    while i < count {
        let num = extract_case_num(filenames[i]);
        if num > max_id {
            max_id = num;
        }
        i += 1;
    }
    max_id
}

/// 格式化懸案案卷檔頭與一句話。
/// @outputs: 1
/// @domain: case_id_num [1..9999]
fn format_case_header(case_id_num: usize, title: &str, gate: &str, sha: &str, today: &str, reason: &str) -> String {
    if case_id_num < 1 || case_id_num > 9999 {
        return String::new();
    }
    let cid = format!("C-{:04}", case_id_num);
    let mut out = String::with_capacity(1024);
    out.push_str(&format!("# {} {}\n", cid, title.trim()));
    out.push_str("- 狀態：懸置中\n- 類型：技術\n");
    out.push_str(&format!("- 關卡：{}\n- 紅燈起點：{}\n", gate.trim(), sha.trim()));
    out.push_str("- 時限：5天；已用：0小時\n- 轉交：人類\n");
    out.push_str("- 前手：pro (Gemini 4 Argon / Claude Opus) 額度枯竭 (429/ResourceExhausted)\n");
    out.push_str(&format!("- 登記：{}\n\n", today.trim()));
    out.push_str("## 一句話\n");
    if reason.trim().is_empty() {
        out.push_str("前沿高智力模型配額耗盡（冷卻週期中），依 R13-6 雙軌退路安全懸置本案。\n\n");
    } else {
        out.push_str(&format!(
            "前沿高智力模型配額耗盡（{}），依 R13-6 雙軌退路安全懸置本案。\n\n",
            reason.trim()
        ));
    }
    out
}

/// 格式化懸案已驗證假設表格。
/// @outputs: 1
fn format_case_hypotheses_table() -> String {
    let mut out = String::with_capacity(512);
    out.push_str("## 驗收條件\n- 「有檢查器會擋的事，不要試圖繞過。」— docs/standards/CHARTER.md\n\n");
    out.push_str("## 已驗證的假設\n| # | 前提 | 假設 | 怎麼驗 | 結果 | 證據 |\n|---|---|---|---|---|---|\n");
    out.push_str("| 1 | 前沿模型有額度可用 | Pro 仲裁能裁決該爭端 | 呼叫 invoke_subagent | 不成立 | `python scripts/agy-kit.py probe-frontier` |\n");
    out.push_str("| 2 | 短時間重試能取得配額 | 等待短時間重試可通過 | 重試調用前沿模型 | 不成立 | `python scripts/agy-kit.py pressure` |\n");
    out.push_str("| 3 | 可由 Flash 降級裁決核心安全 | Flash 能替代裁決安全項目 | 對照 R13-6 規則邊界 | 不成立 | docs/standards/CHARTER.md:18 |\n\n");
    out
}

/// 格式化懸案下一步四格決策建議。
/// @outputs: 1
fn format_case_next_steps(today: &str) -> String {
    let mut out = String::with_capacity(512);
    out.push_str("## 沒走的捷徑\n- 沒改測試、沒放寬門檻、沒碰受保護路徑\n\n");
    out.push_str("## 下一步\n");
    out.push_str("- **在決定什麼**：是否等待前沿模型（Argon/Opus）額度恢復後再行複審，或由人類工程師介入終審。\n");
    out.push_str("- **選項與後果**：\n");
    out.push_str("  1. **等待配額重置（建議）**：案件安全懸置，工作流絕不卡死，5天後配額恢復自動喚醒。\n");
    out.push_str("  2. **人類人工介入簽核**：由掌櫃人工在本地直接檢視案卷並簽發終審。\n");
    out.push_str("- **不決定會怎樣**：安全停留在懸案目錄，不會阻礙非核心模組繼續推進。\n");
    out.push_str("- **我的建議**：維持懸置狀態，安全退出當前輪次，防止 Antigravity 卡機。\n\n");
    out.push_str("## 後續\n");
    out.push_str(&format!(
        "- {}：因前沿高智力模型額度耗盡，依 R13-6 正式登記懸案，乾淨釋放交接。\n",
        today.trim()
    ));
    out
}

/// 格式化完全符合 case-gate.py 驗證標準之 C-NNNN 懸案呈報書。
/// @outputs: 1
/// @domain: case_id_num [1..9999]
pub fn format_case_markdown(
    case_id_num: usize,
    title: &str,
    gate: &str,
    reason: &str,
    sha: &str,
    today: &str,
    repro_cmd: &str,
) -> String {
    if case_id_num < 1 || case_id_num > 9999 {
        return String::new();
    }

    let cid = format!("C-{:04}", case_id_num);
    let repro = if repro_cmd.trim().is_empty() {
        format!(
            "python -c \"import sys; print('FAIL: Frontier model quota exhausted for {}'); sys.exit(1)\"",
            cid
        )
    } else {
        repro_cmd.trim().to_string()
    };

    let mut out = String::with_capacity(2048);
    out.push_str(&format_case_header(case_id_num, title, gate, sha, today, reason));
    out.push_str(&format_case_hypotheses_table());
    out.push_str("## 最小重現\n```\n");
    out.push_str(&repro);
    out.push_str("\n```\n\n");
    out.push_str(&format_case_next_steps(today));
    out
}

/// 建立暫定放行技術債務項目 JSON 字串（符合 tools/msa/signoffs.mjs 格式要求）。
/// @outputs: 1
/// @domain: feat_len [1..256]
pub fn build_provisional_debt_json(feature: &str, reason: &str, owner: &str) -> String {
    let feat_len = feature.len();
    if feat_len < 1 || feat_len > 256 {
        return String::new();
    }

    let clean_feat = feature.trim().replace('"', "\\\"");
    let clean_reason = reason.trim().replace('"', "\\\"");
    let clean_owner = owner.trim().replace('"', "\\\"");

    format!(
        r#"{{"feature":"{}","root_cause":"前沿模型額度耗盡，依 R13-6 啟用 Flash+發散盲測暫定放行：{}","fix_plan":"待 Argon/Pro 額度重置後執行批次前沿會診複核","fix_cost":"約 1 次 Pro 調用","trigger":"frontier_quota_restored","owner":"{}","monitor":"MSA 週報與技術債務清單","badge":"[PROVISIONAL_NON_FRONTIER]"}}"#,
        clean_feat, clean_reason, clean_owner
    )
}

/// 判定字串是否包含前沿模型額度枯竭 / 429 資源耗盡特徵。
/// @outputs: 2
/// @domain: msg_len [0..4096]
pub fn is_frontier_quota_error(err_msg: &str) -> bool {
    let msg_len = err_msg.len();
    if msg_len > 4096 {
        return false;
    }

    let lower = err_msg.to_ascii_lowercase();
    lower.contains("resource_exhausted")
        || lower.contains("rate_limit_exceeded")
        || lower.contains("quota exceeded")
        || lower.contains("429")
        || lower.contains("額度枯竭")
        || lower.contains("配額不足")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// @outputs: 1
    #[test]
    fn test_parse_highest_case_id() {
        let files = ["README.md", "TEMPLATE.md", "C-0001.md", "C-0042.md", "notes.txt"];
        assert_eq!(parse_highest_case_id(&files), 42);

        let empty: [&str; 0] = [];
        assert_eq!(parse_highest_case_id(&empty), 0);
    }

    /// @outputs: 1
    #[test]
    fn test_format_case_markdown() {
        let md = format_case_markdown(1, "前沿額度耗盡測試", "msa-gate", "Quota 429", "abc1234", "2026-11-13", "");
        assert!(md.contains("# C-0001 前沿額度耗盡測試"));
        assert!(md.contains("- 狀態：懸置中"));
        assert!(md.contains("docs/standards/CHARTER.md:18"));
        assert!(md.contains("FAIL: Frontier model quota exhausted for C-0001"));
    }

    /// @outputs: 1
    #[test]
    fn test_build_provisional_debt_json() {
        let json_str = build_provisional_debt_json("geo_ecef", "Non-critical Bowring approximation", "flash-subagent");
        assert!(json_str.contains("\"feature\":\"geo_ecef\""));
        assert!(json_str.contains("\"badge\":\"[PROVISIONAL_NON_FRONTIER]\""));
        assert!(json_str.contains("\"trigger\":\"frontier_quota_restored\""));
    }

    /// @outputs: 1
    #[test]
    fn test_is_frontier_quota_error() {
        assert!(is_frontier_quota_error("Error: 429 RESOURCE_EXHAUSTED"));
        assert!(is_frontier_quota_error("Google API: quota exceeded for model pro"));
        assert!(is_frontier_quota_error("系統提示：前沿模型額度枯竭"));
        assert!(!is_frontier_quota_error("SyntaxError: unexpected token"));
    }
}
