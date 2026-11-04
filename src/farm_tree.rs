// ## Map
// Role: core/farm_tree — 雲端農夫的失敗嘗試樹（attempt/ 分支）、二振出局、規格版本保鮮與黃金向量測試產生器。
// Inputs: cargo test / guard 錯誤輸出、規格書文字與 git blob 雜湊、attempt/ 分支的 commit 訊息、specs/ 佇列。
// Outputs: 失敗分類、二振動作、與 mangala-mods/git-handoff 相容的 commit 訊息、已知死路提示、tests/golden_<mod>.rs、佇列狀態。
// Invariants: 零外部依賴、無遞迴、零全域可變狀態、單一函式 <= 60 行；失敗紀錄只從 git 重生，不另存狀態檔（日誌保鮮）。

use std::fs;
use std::path::Path;
use std::process::Command;

/// 同一失敗分類連續出現幾次算「二振出局」（與 mangala-workflow 的 DEFAULT_STRIKES 一致）
pub const STRIKE_LIMIT: usize = 2;
/// 同一版規格累積幾輪放棄就卡關、交給人類裁判
pub const BLOCK_ROUNDS: usize = 3;
/// 失敗嘗試分支前綴（與 git-handoff 的 ATTEMPT_PREFIX 一致）
pub const ATTEMPT_PREFIX: &str = "attempt/";
/// 雲端 API 全部失敗時的錯誤標記：基礎設施問題，不算程式失敗、不記死路
pub const INFRA_TAG: &str = "[infra]";
/// 黃金向量斷言的訊息前綴：失敗時用來辨識是第幾個向量
pub const GOLDEN_TAG: &str = "spec golden vector #";
const MAX_LESSON_CHARS: usize = 160;
const MAX_SUBJECT_CHARS: usize = 72;
const MAX_EVIDENCE_LINES: usize = 12;

/// 一輪農作中的一次嘗試：哪個模型、失敗分類、第一行錯誤
#[derive(Clone, Debug, PartialEq)]
pub struct RoundRecord {
    pub model: String,
    pub class: String,
    pub first_line: String,
}

/// 一條已放棄的方向（一個 attempt/ 分支的尖端）
#[derive(Clone, Debug, PartialEq)]
pub struct DeadEnd {
    pub branch: String,
    pub failure: String,
    pub model: String,
    pub spec_hash: String,
    pub lesson: String,
    pub rounds: Vec<(String, String)>,
}

/// 二振出局的計數：目前連續失敗的分類與次數
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StrikeState {
    pub class: String,
    pub count: usize,
}

/// 二振出局的動作
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StrikeAction {
    Continue,
    SwitchDirection,
    Escalate,
}

/// 佇列中一份規格的狀態（全部從 git 與檔案系統推導，不手寫）
#[derive(Clone, Debug, PartialEq)]
pub enum ModuleState {
    Done,
    Claimed,
    Pending(usize),
    Blocked(usize),
    Suspect(String),
}

/// 佇列中的一份規格
#[derive(Clone, Debug, PartialEq)]
pub struct QueueEntry {
    pub spec: String,
    pub module: String,
    pub state: ModuleState,
}

/// 放棄一輪時寫進 attempt/ 分支的資料
#[derive(Clone, Debug, PartialEq)]
pub struct AbandonInput {
    pub module: String,
    pub spec_path: String,
    pub spec_hash: String,
    pub round: usize,
    pub outcome: String,
    pub rounds: Vec<RoundRecord>,
    pub evidence: String,
}

/// 農作成功時寫進 contrib/ 分支的資料
#[derive(Clone, Debug, PartialEq)]
pub struct DoneInput {
    pub module: String,
    pub spec_path: String,
    pub spec_hash: String,
    pub provider: String,
    pub test_summary: String,
    pub golden_count: usize,
    pub attempts_used: usize,
    pub resolves: Vec<String>,
}

/// 依字元（非位元組）截斷，超過時以「…」結尾，結果（含「…」）不超過 max_chars 個字元
/// @outputs: 2
/// @domain: max_chars [1..100000]
pub fn truncate_chars(text: &str, max_chars: usize) -> String {
    if max_chars < 1 || max_chars > 100_000 {
        return String::new();
    }
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max_chars - 1).collect();
    out.push('…');
    out
}

/// 把一次嘗試的錯誤輸出歸類（二振出局比對的是分類，不是原始訊息）
/// @outputs: 7
pub fn classify_failure(err_log: &str) -> String {
    if err_log.starts_with(INFRA_TAG) {
        return "infra".to_string();
    }
    if err_log.contains("模型輸出缺少") {
        return "format".to_string();
    }
    if err_log.contains("error[E") || err_log.contains("could not compile") {
        return "compile".to_string();
    }
    if let Some(k) = golden_index(err_log) {
        return format!("golden:#{}", k);
    }
    if let Some(rule) = guard_rule(err_log) {
        return format!("guard:{}", rule);
    }
    match failing_test_name(err_log) {
        Some(name) => format!("test:{}", name),
        None => "other".to_string(),
    }
}

/// @outputs: 2
fn golden_index(err_log: &str) -> Option<usize> {
    let pos = err_log.find(GOLDEN_TAG)?;
    let digits: String = err_log[pos + GOLDEN_TAG.len()..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse::<usize>().ok()
}

/// @outputs: 2
fn guard_rule(err_log: &str) -> Option<String> {
    let pos = err_log.find("guard.rs) 攔截")?;
    let rest = &err_log[pos..];
    let start = rest.find('【')?;
    let after = &rest[start + '【'.len_utf8()..];
    let end = after.find('】')?;
    Some(after[..end].to_string())
}

/// @outputs: 2
fn failing_test_name(err_log: &str) -> Option<String> {
    let pos = err_log.find("---- ")?;
    let rest = &err_log[pos + 5..];
    let end = rest.find(" stdout ----")?;
    Some(rest[..end].trim().to_string())
}

/// 錯誤輸出中最有資訊量的一行（給死路教訓與 commit 用）
/// @outputs: 2
pub fn first_error_line(err_log: &str) -> String {
    // 依資訊量排優先序：黃金向量編號 > 編譯錯誤 > guard 規則 > 斷言 > panic 位置
    let keys = [GOLDEN_TAG, "error[", "【", "assertion", "panicked at"];
    let picked = keys
        .iter()
        .find_map(|k| err_log.lines().map(|l| l.trim()).find(|l| l.contains(k)))
        .or_else(|| err_log.lines().map(|l| l.trim()).find(|l| !l.is_empty()));
    match picked {
        Some(line) => truncate_chars(line, MAX_LESSON_CHARS),
        None => "（沒有錯誤輸出）".to_string(),
    }
}

/// 更新二振計數：同分類連續累加，換分類就重算
/// @outputs: 2
pub fn next_strike(prev: &StrikeState, class: &str) -> StrikeState {
    if prev.class == class {
        return StrikeState { class: class.to_string(), count: prev.count.saturating_add(1) };
    }
    StrikeState { class: class.to_string(), count: 1 }
}

/// 剛好第 STRIKE_LIMIT 次：二振出局換方向；到第 STRIKE_LIMIT + 2 次：停止並升級裁判
/// @outputs: 3
pub fn strike_action(state: &StrikeState) -> StrikeAction {
    if state.count >= STRIKE_LIMIT + 2 {
        return StrikeAction::Escalate;
    }
    if state.count == STRIKE_LIMIT {
        return StrikeAction::SwitchDirection;
    }
    StrikeAction::Continue
}

/// 二振出局時加進提示的指示（對應 mangala-workflow 的 /figure-it-out「攻擊前提」）
/// @outputs: 1
pub fn strike_note(state: &StrikeState) -> String {
    format!(
        "【二振出局】前 {} 次都失敗在同一類問題（{}）。不要再微調同一個方向：先列出前幾次修法共用的隱含前提並逐一推翻，改用不同的演算法結構或寫法重新實作。規格書的黃金向量預期值不得修改。\n",
        state.count, state.class
    )
}

/// 依失敗分類給「下次避開什麼」
/// @outputs: 7
pub fn avoid_hint(class: &str) -> String {
    let hint = if class.starts_with("golden:") {
        "先依規格手算黃金向量再寫實作，不要調整公式去湊值；若不同模型都算出同一個值，懷疑規格的預期值"
    } else if class == "compile" {
        "型別與函式簽章完全照規格，不要引用 std 以外或不存在的 API"
    } else if class.starts_with("guard:") {
        "先滿足 09 合約該條規則（Map、@outputs、@domain 守衛、60 行、安全證明）再處理演算法"
    } else if class.starts_with("test:") {
        "對照該測試的預期值逐步推導，確認邊界條件與回傳型別"
    } else if class == "format" {
        "輸出只能有 ```rust:module 與 ```rust:test 兩個區塊"
    } else if class == "infra" {
        "雲端 API 無法連線，與程式無關"
    } else {
        "改用不同的演算法結構，不要沿用前次寫法"
    };
    hint.to_string()
}

/// commit 訊息標頭區（主旨、空行之後到下一個空行）的 `Key: value`
/// @outputs: 2
pub fn header_value(message: &str, key: &str) -> Option<String> {
    let prefix = format!("{}:", key);
    for line in message.lines().skip(2) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some(value) = trimmed.strip_prefix(prefix.as_str()) {
            return Some(value.trim().to_string());
        }
    }
    None
}

/// `## 段落` 的第一個條列項目
/// @outputs: 2
pub fn section_first_bullet(message: &str, section: &str) -> Option<String> {
    let heading = format!("## {}", section);
    let mut inside = false;
    for line in message.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            inside = trimmed == heading;
            continue;
        }
        let bullet = if inside { trimmed.strip_prefix("- ") } else { None };
        if let Some(text) = bullet {
            return Some(text.trim().to_string());
        }
    }
    None
}

/// `Farm-Rounds: 模型=分類 | 模型=分類` → [(模型, 分類)]
/// @outputs: 1
pub fn parse_rounds_header(value: &str) -> Vec<(String, String)> {
    value
        .split('|')
        .filter_map(|part| part.trim().split_once('='))
        .map(|(model, class)| (model.trim().to_string(), class.trim().to_string()))
        .filter(|(model, class)| !model.is_empty() && !class.is_empty())
        .collect()
}

/// 解析 attempt/ 分支尖端的 commit 訊息；不是 Status: abandoned 的不算死路
/// @outputs: 2
pub fn parse_dead_end(branch: &str, message: &str) -> Option<DeadEnd> {
    if header_value(message, "Status").as_deref() != Some("abandoned") {
        return None;
    }
    let failure = header_value(message, "Farm-Failure").unwrap_or_else(|| "unknown".to_string());
    let model = header_value(message, "Farm-Model").unwrap_or_else(|| "unknown".to_string());
    let mut rounds = header_value(message, "Farm-Rounds").map(|v| parse_rounds_header(&v)).unwrap_or_default();
    if rounds.is_empty() {
        rounds.push((model.clone(), failure.clone()));
    }
    Some(DeadEnd {
        branch: branch.to_string(),
        failure,
        model,
        spec_hash: header_value(message, "Farm-Spec-Hash").unwrap_or_default(),
        lesson: section_first_bullet(message, "Dead-End").unwrap_or_else(|| "（沒有寫原因）".to_string()),
        rounds,
    })
}

/// `attempt/20261104-farm-<模組>[-r2]`（可帶 origin/ 或 refs/ 前綴）→ 模組名
/// @outputs: 2
pub fn attempt_branch_module(branch: &str) -> Option<String> {
    let pos = branch.find(ATTEMPT_PREFIX)?;
    let rest = &branch[pos + ATTEMPT_PREFIX.len()..];
    let (_, tail) = rest.split_once("-farm-")?;
    let module = match tail.rfind("-r") {
        Some(i) if i + 2 < tail.len() && tail[i + 2..].chars().all(|c| c.is_ascii_digit()) => &tail[..i],
        _ => tail,
    };
    if module.is_empty() {
        return None;
    }
    Some(module.to_string())
}

/// 新的 attempt/ 分支名稱：同一天同模組已存在就加 -r2、-r3…
/// @outputs: 2
pub fn attempt_branch_name(date: &str, module: &str, existing: &[String]) -> String {
    let base = format!("{}{}-farm-{}", ATTEMPT_PREFIX, date, module);
    let taken = |name: &str| existing.iter().any(|e| e.trim_start_matches("origin/") == name);
    if !taken(&base) {
        return base;
    }
    for n in 2..=999 {
        let candidate = format!("{}-r{}", base, n);
        if !taken(&candidate) {
            return candidate;
        }
    }
    format!("{}-rx", base)
}

/// Unix 秒數 → UTC 日期 YYYYMMDD（Howard Hinnant civil_from_days，無外部依賴）
/// @outputs: 2
/// @domain: unix_secs [0..253402300799]
pub fn utc_yyyymmdd(unix_secs: u64) -> String {
    if unix_secs > 253_402_300_799 {
        return "99991231".to_string();
    }
    let z = (unix_secs / 86_400) as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + if month <= 2 { 1 } else { 0 };
    format!("{:04}{:02}{:02}", year, month, day)
}

/// 只保留以「目前這版規格」產生的死路；規格改過之後，舊死路不再算數（日誌保鮮）
/// @outputs: 1
pub fn fresh_dead_ends(all: &[DeadEnd], spec_hash: &str) -> Vec<DeadEnd> {
    all.iter().filter(|d| !spec_hash.is_empty() && d.spec_hash == spec_hash).cloned().collect()
}

/// 不同模型都卡在同一個黃金向量 → 懷疑規格的預期值寫錯，交給人類複核，禁止讓 AI 改預期值
/// @outputs: 2
pub fn spec_suspect(samples: &[(String, String)]) -> Option<String> {
    for (model, class) in samples.iter().filter(|(_, c)| c.starts_with("golden:")) {
        if samples.iter().any(|(m, c)| c == class && m != model) {
            return Some(class.clone());
        }
    }
    None
}

/// 死路清單裡每一次嘗試的 (模型, 分類)
/// @outputs: 1
pub fn dead_end_samples(fresh: &[DeadEnd]) -> Vec<(String, String)> {
    fresh.iter().flat_map(|d| d.rounds.iter().cloned()).collect()
}

/// 規格的狀態：已入庫 → 已被領取 → 規格可疑 → 卡關 → 待做
/// @outputs: 5
pub fn module_state(has_src: bool, has_contrib: bool, fresh: &[DeadEnd]) -> ModuleState {
    if has_src {
        return ModuleState::Done;
    }
    if has_contrib {
        return ModuleState::Claimed;
    }
    if let Some(class) = spec_suspect(&dead_end_samples(fresh)) {
        return ModuleState::Suspect(class);
    }
    if fresh.len() >= BLOCK_ROUNDS {
        return ModuleState::Blocked(fresh.len());
    }
    ModuleState::Pending(fresh.len())
}

/// 給模型看的「已知死路」段落
/// @outputs: 2
pub fn dead_end_prompt(fresh: &[DeadEnd]) -> String {
    if fresh.is_empty() {
        return String::new();
    }
    let mut out = String::from("【已知死路（同一版規格先前各輪已放棄的方向，禁止重踩）】：\n");
    for d in fresh {
        out.push_str(&format!("- {}（{}，模型 {}）：{}\n", d.branch, d.failure, d.model, d.lesson));
    }
    out.push_str("這一輪必須採用與上述死路不同的做法。\n");
    out
}

/// 證據段：錯誤輸出中最有資訊量的幾行，並遮蔽 URL 中的金鑰
/// @outputs: 2
pub fn evidence_lines(log: &str) -> String {
    let keys = ["error", "panicked", "【", "assert", "test result", GOLDEN_TAG, "-->"];
    let picked: Vec<String> = log
        .lines()
        .map(|l| l.trim())
        .filter(|l| keys.iter().any(|k| l.contains(k)))
        .take(MAX_EVIDENCE_LINES)
        .map(|l| format!("  {}", redact_keys(&truncate_chars(l, 200))))
        .collect();
    if picked.is_empty() {
        return "  （沒有可摘錄的錯誤輸出）".to_string();
    }
    picked.join("\n")
}

/// 遮蔽 `key=xxx`、`Bearer xxx` 形式的金鑰
/// @outputs: 1
pub fn redact_keys(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let markers = ["key=", "Bearer "];
    while let Some((pos, marker)) = markers.iter().filter_map(|m| rest.find(m).map(|p| (p, *m))).min() {
        out.push_str(&rest[..pos + marker.len()]);
        out.push_str("***");
        let after = &rest[pos + marker.len()..];
        let end = after.find(|c: char| c.is_whitespace() || c == '&' || c == '\x22').unwrap_or(after.len());
        rest = &after[end..];
    }
    out.push_str(rest);
    out
}

/// 放棄一輪的 commit 主旨與標頭區（git-handoff：AI-Handoff／Status／Tests／Map／Next 必填）
/// @outputs: 1
fn abandon_headers(input: &AbandonInput) -> String {
    let last = input.rounds.last();
    let failure = last.map(|r| r.class.clone()).unwrap_or_else(|| "unknown".to_string());
    let model = last.map(|r| r.model.clone()).unwrap_or_else(|| "none".to_string());
    let tests = if failure == "format" || failure == "unknown" {
        "none".to_string()
    } else {
        format!("fail cargo test --test {}", failing_target(&failure, &input.module))
    };
    let next = match input.outcome.as_str() {
        "abandoned" => "下一輪換方向重試（已知死路會自動帶入提示）".to_string(),
        _ => "停止自動重試，交給人類裁判（見 cloud-farmer issue）".to_string(),
    };
    let rounds: Vec<String> = input.rounds.iter().map(|r| format!("{}={}", r.model, r.class)).collect();
    let subject = format!("chore(cloud-farm): abandon {} round {}", input.module, input.round);
    format!(
        "{}\n\nAI-Handoff: 1\nStatus: abandoned\nTests: {}\nMap: unchanged\nNext: {}\nContinue: no\nFarm-Spec: {}\nFarm-Spec-Hash: {}\nFarm-Module: {}\nFarm-Round: {}\nFarm-Outcome: {}\nFarm-Failure: {}\nFarm-Model: {}\nFarm-Rounds: {}\n",
        truncate_chars(&subject, MAX_SUBJECT_CHARS), tests, next, input.spec_path, input.spec_hash,
        input.module, input.round, input.outcome, failure, model, rounds.join(" | ")
    )
}

/// 放棄一輪的完整 commit 訊息（Status: abandoned＋## Dead-End，進 attempt/ 分支）
/// @outputs: 1
pub fn abandon_message(input: &AbandonInput) -> String {
    let last = input.rounds.last();
    let failure = last.map(|r| r.class.clone()).unwrap_or_else(|| "unknown".to_string());
    let reason = last.map(|r| r.first_line.clone()).unwrap_or_else(|| "（沒有錯誤輸出）".to_string());
    let lesson = truncate_chars(&format!("{}：{}；下次避開：{}", failure, reason, avoid_hint(&failure)), MAX_LESSON_CHARS);
    let what: Vec<String> = input
        .rounds
        .iter()
        .enumerate()
        .map(|(i, r)| format!("- 第 {} 次：{} → {}：{}", i + 1, r.model, r.class, r.first_line))
        .collect();
    let models: Vec<String> = input.rounds.iter().map(|r| r.model.clone()).collect();
    let handoff = match input.outcome.as_str() {
        "suspect" => "不同模型卡在同一個黃金向量：請先複核規格書的預期值，不要讓 AI 改預期值。",
        "blocked" => "同一版規格已累積多輪放棄：請檢討規格或拆小，修改規格後舊死路自動失效。",
        "escalated" => "同一類失敗連續四次：二振後換方向仍無效，需要人類或高智力模型裁判。",
        _ => "失敗的程式碼保留在本分支，下一輪會帶入本死路並換方向。",
    };
    format!(
        "{}\n## What\n{}\n## Why\n- {}\n## Dead-End\n- {}\n- 試了什麼：{}，共 {} 次\n- 為什麼失敗：{}\n- 下次避開什麼：{}\n## Handoff\n- {}\n- 規格：{}（雜湊 {}）；規格改版後本死路自動失效\n## Evidence\n- `cargo test --test {}`：\n{}\n",
        abandon_headers(input), what.join("\n"), why_abandoned(&input.outcome), lesson, models.join("、"), input.rounds.len(), reason,
        avoid_hint(&failure), handoff, input.spec_path, input.spec_hash, failing_target(&failure, &input.module), input.evidence
    )
}

/// 失敗分類對應的測試目標：黃金向量失敗在 golden_<mod>，其餘在 test_<mod>
/// @outputs: 2
fn failing_target(failure: &str, module: &str) -> String {
    if failure.starts_with("golden:") {
        return format!("golden_{}", module);
    }
    format!("test_{}", module)
}

/// 這一輪為什麼停下來（## Why）
/// @outputs: 4
fn why_abandoned(outcome: &str) -> String {
    let reason = match outcome {
        "suspect" => "不同模型卡在同一個黃金向量，懷疑規格預期值，停止自動重試",
        "blocked" => "同一版規格累積放棄達上限，停止自動重試以免浪費免費額度",
        "escalated" => "同一類失敗連續四次（二振後仍無效），升級裁判",
        _ => "本輪重試次數用盡，記錄死路後下一輪換方向",
    };
    reason.to_string()
}

/// 農作成功的 commit 訊息（Status: done；Resolves 列出被這次成功取代的死路分支）
/// @outputs: 1
pub fn done_message(input: &DoneInput) -> String {
    let subject = format!("feat(cloud-farm): farm {} from spec", input.module);
    let golden = if input.golden_count > 0 {
        format!("{}/{} (spec ```golden)", input.golden_count, input.golden_count)
    } else {
        "unchecked (spec has no ```golden block)".to_string()
    };
    let resolves = if input.resolves.is_empty() { "none".to_string() } else { input.resolves.join(", ") };
    format!(
        "{}\n\nAI-Handoff: 1\nStatus: done\nTests: pass {}\nMap: changed\nNext: 本機執行 cargo run -- harvest 收割進 main（走 PR）\nContinue: no\nFarm-Spec: {}\nFarm-Spec-Hash: {}\nFarm-Module: {}\nFarm-Model: {}\nFarm-Attempts: {}\nGolden-Vectors: {}\nResolves: {}\n\n## What\n- 依 {} 產出 src/{}.rs 與 tests/test_{}.rs\n## Why\n- 規格佇列中的下一份待做規格；黃金向量測試由規格書 ```golden 區塊自動產生，AI 無法修改\n## Map\n- + src/lib.rs -> pub mod {}（新模組）\n## Handoff\n- 收割前確認 Golden-Vectors 不是 unchecked；unchecked 代表規格書缺 ```golden 區塊\n## Evidence\n- `cargo test`：{}\n- `cargo run -- guard`：PASS\n",
        truncate_chars(&subject, MAX_SUBJECT_CHARS), input.test_summary, input.spec_path, input.spec_hash,
        input.module, input.provider, input.attempts_used, golden, resolves, input.spec_path, input.module,
        input.module, input.module, input.test_summary
    )
}

/// 規格書 ```golden 區塊的每一行（略過空行與 // 註解）
/// @outputs: 1
pub fn extract_golden_lines(spec: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut inside = false;
    for line in spec.lines() {
        let trimmed = line.trim();
        if !inside {
            inside = trimmed == "```golden";
            continue;
        }
        if trimmed.starts_with("```") {
            break;
        }
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            lines.push(trimmed.to_string());
        }
    }
    lines
}

/// 由規格的黃金向量產生 tests/golden_<mod>.rs；`let` 開頭或分號結尾是準備敘述，其餘每行是一個斷言
/// @outputs: 2
pub fn render_golden_test(module: &str, lines: &[String]) -> Option<String> {
    let mut body = String::new();
    let mut count = 0usize;
    for line in lines {
        if line.starts_with("let ") || line.ends_with(';') {
            let stmt = if line.ends_with(';') { line.clone() } else { format!("{};", line) };
            body.push_str(&format!("    {}\n", stmt));
            continue;
        }
        count += 1;
        body.push_str(&format!("    assert!({}, \x22{}{}\x22);\n", line, GOLDEN_TAG, count));
    }
    if count == 0 {
        return None;
    }
    Some(format!(
        "// 本檔由 cloud_farmer 依 specs/{0}.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。\nuse rust_bricks::{0}::*;\n\n#[test]\nfn golden_vectors_from_spec() {{\n{1}}}\n",
        module, body
    ))
}

/// 黃金向量的斷言數（不含準備敘述）
/// @outputs: 1
pub fn golden_assert_count(lines: &[String]) -> usize {
    lines.iter().filter(|l| !l.starts_with("let ") && !l.ends_with(';')).count()
}

/// 執行 git 並取 stdout；失敗回 None
/// @outputs: 2
fn git_out(repo_root: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).current_dir(repo_root).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).to_string())
}

/// 本機與 origin 的 attempt/ 分支：(分支短名, commit sha)
/// @outputs: 1
pub fn list_attempt_refs(repo_root: &Path) -> Vec<(String, String)> {
    let args = [
        "for-each-ref",
        "--format=%(refname:short)\t%(objectname)",
        "refs/heads/attempt",
        "refs/remotes/origin/attempt",
    ];
    let raw = git_out(repo_root, &args).unwrap_or_default();
    raw.lines()
        .filter_map(|l| l.split_once('\t'))
        .map(|(name, sha)| (name.trim().to_string(), sha.trim().to_string()))
        .filter(|(name, sha)| !name.is_empty() && !sha.is_empty())
        .collect()
}

/// 某模組的全部死路（同一分支的本機與 origin/ 拷貝只算一次）
/// @outputs: 1
pub fn load_dead_ends(repo_root: &Path, module: &str) -> Vec<DeadEnd> {
    let mut seen: Vec<String> = Vec::new();
    let mut out = Vec::new();
    for (name, sha) in list_attempt_refs(repo_root) {
        let branch = name.trim_start_matches("origin/").to_string();
        if attempt_branch_module(&branch).as_deref() != Some(module) || seen.contains(&branch) {
            continue;
        }
        seen.push(branch.clone());
        let message = git_out(repo_root, &["log", "-1", "--format=%B", sha.as_str()]).unwrap_or_default();
        if let Some(dead) = parse_dead_end(&branch, &message) {
            out.push(dead);
        }
    }
    out
}

/// 規格書內容的 git blob 雜湊：規格一改，舊死路就失效
/// @outputs: 2
pub fn spec_blob_hash(repo_root: &Path, spec_path: &str) -> String {
    match git_out(repo_root, &["hash-object", spec_path]) {
        Some(hash) if !hash.trim().is_empty() => hash.trim().to_string(),
        _ => "unknown".to_string(),
    }
}

/// 現在的 UTC 日期 YYYYMMDD
/// @outputs: 1
pub fn today_utc() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    utc_yyyymmdd(secs)
}

/// 把這一輪失敗的程式碼 commit 到新的 attempt/ 分支（可選擇推送），再切回原分支
/// @outputs: 2
pub fn record_abandoned_round(repo_root: &Path, module: &str, message: &str, push: bool) -> Result<String, String> {
    let existing: Vec<String> = list_attempt_refs(repo_root).into_iter().map(|(n, _)| n).collect();
    let branch = attempt_branch_name(&today_utc(), module, &existing);
    let msg_dir = repo_root.join("target");
    let _ = fs::create_dir_all(&msg_dir);
    let msg_path = msg_dir.join("farm_attempt_msg.txt");
    fs::write(&msg_path, message).map_err(|e| format!("寫入 commit 訊息暫存檔失敗: {}", e))?;
    git_out(repo_root, &["checkout", "-q", "-b", branch.as_str()]).ok_or("無法建立 attempt 分支")?;
    let files = [
        format!("src/{}.rs", module),
        format!("tests/test_{}.rs", module),
        format!("tests/golden_{}.rs", module),
        "src/lib.rs".to_string(),
    ];
    for f in files.iter().filter(|f| repo_root.join(f.as_str()).exists()) {
        let _ = git_out(repo_root, &["add", "-f", f.as_str()]);
    }
    let msg_arg = msg_path.to_string_lossy().to_string();
    let committed = git_out(repo_root, &["commit", "-q", "--no-verify", "-F", msg_arg.as_str()]).is_some();
    if committed && push {
        let _ = git_out(repo_root, &["push", "-q", "-u", "origin", branch.as_str(), "--no-verify"]);
    }
    let _ = git_out(repo_root, &["checkout", "-q", "-"]);
    if !committed {
        let _ = git_out(repo_root, &["branch", "-D", branch.as_str()]);
        return Err("沒有可記錄的失敗程式碼，未建立 attempt 分支".to_string());
    }
    Ok(branch)
}

/// 已入庫模組的死路分支已經過期（被成功取代）：刪除本機與（可選）遠端分支
/// @outputs: 1
pub fn prune_resolved_attempts(repo_root: &Path, push: bool) -> Vec<String> {
    let mut pruned = Vec::new();
    for (name, _) in list_attempt_refs(repo_root) {
        let module = match attempt_branch_module(&name) {
            Some(m) => m,
            None => continue,
        };
        if !repo_root.join("src").join(format!("{}.rs", module)).exists() {
            continue;
        }
        match name.strip_prefix("origin/") {
            Some(remote) if push => {
                let _ = git_out(repo_root, &["push", "-q", "origin", "--delete", remote, "--no-verify"]);
            }
            Some(_) => continue,
            None => {
                let _ = git_out(repo_root, &["branch", "-q", "-D", name.as_str()]);
            }
        }
        pruned.push(name);
    }
    pruned
}

/// 已被雲端領取（有 contrib/cloud-farm-<mod> 分支）的模組名
/// @outputs: 1
fn claimed_modules(repo_root: &Path) -> Vec<String> {
    let args = ["for-each-ref", "--format=%(refname:short)", "refs/heads/contrib", "refs/remotes/origin/contrib"];
    let raw = git_out(repo_root, &args).unwrap_or_default();
    raw.lines()
        .filter_map(|l| l.trim().rsplit_once("contrib/cloud-farm-").map(|(_, m)| m.to_string()))
        .collect()
}

/// 掃描 specs/ 佇列：每份規格的狀態全部從 git 與檔案系統推導
/// @outputs: 1
pub fn scan_queue(repo_root: &Path) -> Vec<QueueEntry> {
    let mut specs: Vec<String> = fs::read_dir(repo_root.join("specs"))
        .map(|rd| rd.flatten().map(|e| e.file_name().to_string_lossy().to_string()).collect())
        .unwrap_or_default();
    specs.retain(|n| n.ends_with(".md") && n != "template.md");
    specs.sort();
    let claimed = claimed_modules(repo_root);
    let mut out = Vec::new();
    for file in specs {
        let module = file.trim_end_matches(".md").to_string();
        let spec = format!("specs/{}", file);
        let has_src = repo_root.join("src").join(format!("{}.rs", module)).exists();
        let fresh = if has_src {
            Vec::new()
        } else {
            fresh_dead_ends(&load_dead_ends(repo_root, &module), &spec_blob_hash(repo_root, &spec))
        };
        let state = module_state(has_src, claimed.contains(&module), &fresh);
        out.push(QueueEntry { spec, module, state });
    }
    out
}

/// 給 GitHub Actions 的 key=value（寫進 $GITHUB_OUTPUT）
/// @outputs: 1
pub fn queue_kv_lines(entries: &[QueueEntry]) -> String {
    let next = entries.iter().find(|e| matches!(e.state, ModuleState::Pending(_)));
    let pick = |f: fn(&ModuleState) -> bool| -> Vec<String> {
        entries.iter().filter(|e| f(&e.state)).map(|e| e.module.clone()).collect()
    };
    let blocked = pick(|s| matches!(s, ModuleState::Blocked(_)));
    let suspect = pick(|s| matches!(s, ModuleState::Suspect(_)));
    let pending = pick(|s| matches!(s, ModuleState::Pending(_)));
    let done = pick(|s| matches!(s, ModuleState::Done));
    format!(
        "next_spec={}\nnext_module={}\nblocked={}\nsuspect={}\npending_count={}\ndone_count={}\n",
        next.map(|e| e.spec.as_str()).unwrap_or(""),
        next.map(|e| e.module.as_str()).unwrap_or(""),
        blocked.join(","),
        suspect.join(","),
        pending.len(),
        done.len()
    )
}

/// 狀態的中文標籤
/// @outputs: 5
pub fn state_label(state: &ModuleState) -> String {
    match state {
        ModuleState::Done => "✅ 已入庫".to_string(),
        ModuleState::Claimed => "🌾 雲端已產出，待收割".to_string(),
        ModuleState::Pending(n) => format!("⏳ 待做（本版規格已放棄 {} 輪）", n),
        ModuleState::Blocked(n) => format!("⛔ 卡關（本版規格已放棄 {} 輪）", n),
        ModuleState::Suspect(class) => format!("❓ 規格可疑（{}）", class),
    }
}

/// 佇列狀態表（每次從 git 重生，寫進 Actions 的 job summary，不手寫）
/// @outputs: 1
pub fn status_markdown(entries: &[QueueEntry]) -> String {
    let mut out = String::from("## 雲端農夫佇列狀態（由 git 即時推導）\n\n| 規格 | 狀態 |\n|---|---|\n");
    for e in entries.iter().filter(|e| e.state != ModuleState::Done) {
        out.push_str(&format!("| `{}` | {} |\n", e.spec, state_label(&e.state)));
    }
    let done = entries.iter().filter(|e| e.state == ModuleState::Done).count();
    out.push_str(&format!("\n已入庫 {} 份；佇列共 {} 份規格。\n", done, entries.len()));
    out
}

/// 某模組卡關／規格可疑時的報告（issue 內文）
/// @outputs: 1
pub fn dead_end_report(module: &str, spec: &str, state: &ModuleState, fresh: &[DeadEnd]) -> String {
    let mut out = format!(
        "雲端農夫對 `{}`（`{}`）停止自動重試：**{}**。\n\n## 本版規格的死路（attempt/ 分支）\n",
        module, spec, state_label(state)
    );
    for d in fresh {
        out.push_str(&format!("- `{}`：{}（模型 {}）\n  - {}\n", d.branch, d.failure, d.model, d.lesson));
    }
    out.push_str("\n## 怎麼處理\n- 規格可疑：先複核規格書 ```golden 區塊的預期值與出處，修正後推送規格，舊死路自動失效、佇列自動重排。\n");
    out.push_str("- 卡關：檢討規格（拆小、補清楚介面或黃金向量），或改由本機人工實作。\n");
    out.push_str("- 要原樣重試：刪除上述 attempt/ 分支。\n\n（本 issue 由 cloud_farmer.yml 依 git 狀態自動開關，模組入庫或規格改版後會自動關閉。）\n");
    out
}

/// `cargo test` 輸出的通過摘要，例如「3 passed」
/// @outputs: 2
pub fn test_pass_summary(stdout: &str) -> String {
    let line = match stdout.lines().find(|l| l.trim_start().starts_with("test result:")) {
        Some(l) => l,
        None => return "（沒有 test result 行）".to_string(),
    };
    line.split(';')
        .map(|part| part.trim())
        .find(|part| part.ends_with("passed"))
        .map(|part| part.trim_start_matches("test result: ok.").trim().to_string())
        .unwrap_or_else(|| line.trim().to_string())
}

/// 依規格書 ```golden 區塊，為已入庫的模組重新產生 tests/golden_<mod>.rs；回傳 (模組, 斷言數) 清單
/// 規格與測試不同步時，CI 的 clean-tree 檢查會攔下來
/// @outputs: 1
pub fn sync_golden_tests(repo_root: &Path) -> Vec<(String, usize)> {
    let mut synced = Vec::new();
    for entry in scan_queue(repo_root).iter().filter(|e| e.state == ModuleState::Done) {
        let spec = fs::read_to_string(repo_root.join(&entry.spec)).unwrap_or_default();
        let lines = extract_golden_lines(&spec);
        let path = repo_root.join("tests").join(format!("golden_{}.rs", entry.module));
        match render_golden_test(&entry.module, &lines) {
            Some(test) => {
                let _ = fs::write(&path, test);
                synced.push((entry.module.clone(), golden_assert_count(&lines)));
            }
            None => {
                let _ = fs::remove_file(&path);
            }
        }
    }
    synced
}
