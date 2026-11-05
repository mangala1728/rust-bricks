// ## Map
// Role: core/msa — 量測系統分析（MSA，IATF 16949 精神）：先驗量具（測試組）再驗產品（積木）。
// Inputs: msa/<模組>.rs 的 MSA 零件（參考實作、公開標準件、挑戰件、盲測）、規格書 ```golden 區塊、各候選實作的量測結果。
// Outputs: tests/msa_<模組>.rs 產生器、量具判定（通過／需簽核／重做／量具壞了）、產品判定（允收／拒收／未量測）、管理報告。
// Invariants: 零外部依賴、無遞迴、零全域可變狀態、單一函式 <= 60 行；門檻依風險等級集中於 thresholds()，可依實績校正。

use std::fs;
use std::path::Path;

/// 風險等級：決定量具門檻的鬆緊（見 docs/MSA.md「門檻怎麼定」）
/// 判斷軸：量測不準時，錯誤要過幾站才會被發現？站與站之間的驗收代價高不高？有沒有法規絕對值？
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RiskTier {
    /// 下一站（系統測試）就能便宜地檢出 → 放寬
    Relaxed,
    /// 下一站檢出但站間代價高，或要過多站但代價低 → 標準
    Standard,
    /// 要過多站且代價高，或可能流到客戶端 → 收緊
    Strict,
    /// 有 CE、FCC 等法規絕對值 → 沒有簽核放行區，難做也要做到
    Regulatory,
    /// 沒宣告或寫了看不懂的值 → 影響範圍未知：啟動調查程序，調查期間以收緊圍堵（收緊只多花機器時間，不多花付費 token）
    Undeclared,
}

/// 一個風險等級的門檻
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Thresholds {
    /// 挑戰件抓出率 ≥ 此值：鑑別力合格
    pub kill_pass_pct: f64,
    /// 挑戰件抓出率 < 此值：重做；介於兩者之間需簽核
    pub kill_redo_pct: f64,
    /// 容差比（斷言容差 ÷ 預期值，P/T 類比）< 此值：合格
    pub tol_pass_pct: f64,
    /// 容差比 > 此值：重做；介於兩者之間需簽核
    pub tol_redo_pct: f64,
    /// false = 沒有簽核放行區，落在中間一律重做
    pub sign_off_allowed: bool,
    /// 盲測筆數：只耗機器時間、不耗付費 token，所以收緊時直接加量
    pub blind_samples: usize,
}

/// 風險等級 → 門檻
/// @outputs: 4
pub fn thresholds(tier: RiskTier) -> Thresholds {
    match tier {
        RiskTier::Relaxed => Thresholds { kill_pass_pct: 80.0, kill_redo_pct: 60.0, tol_pass_pct: 15.0, tol_redo_pct: 40.0, sign_off_allowed: true, blind_samples: 2_000 },
        RiskTier::Standard => Thresholds { kill_pass_pct: 90.0, kill_redo_pct: 70.0, tol_pass_pct: 10.0, tol_redo_pct: 30.0, sign_off_allowed: true, blind_samples: 2_000 },
        RiskTier::Strict | RiskTier::Undeclared => Thresholds { kill_pass_pct: 100.0, kill_redo_pct: 90.0, tol_pass_pct: 5.0, tol_redo_pct: 10.0, sign_off_allowed: true, blind_samples: 20_000 },
        RiskTier::Regulatory => Thresholds { kill_pass_pct: 100.0, kill_redo_pct: 100.0, tol_pass_pct: 5.0, tol_redo_pct: 5.0, sign_off_allowed: false, blind_samples: 20_000 },
    }
}

/// 風險等級的白話名稱
/// @outputs: 5
pub fn tier_label(tier: RiskTier) -> String {
    match tier {
        RiskTier::Relaxed => "放寬（下一站可便宜檢出）".to_string(),
        RiskTier::Standard => "標準".to_string(),
        RiskTier::Strict => "收緊（多站才檢出／站間代價高／可能流到客戶）".to_string(),
        RiskTier::Regulatory => "法規死線（無簽核區）".to_string(),
        RiskTier::Undeclared => "🔍 影響未知：調查中，暫以收緊圍堵".to_string(),
    }
}

/// 從 msa/<模組>.rs 的標籤判定風險等級：
/// `// @msa-regulatory: <規範與絕對值>`（有就是法規死線）、
/// `// @msa-escape: next-station | multi-station | customer`、`// @msa-station-cost: low | high`
/// @outputs: 5
pub fn risk_tier(msa_src: &str) -> RiskTier {
    let tag = |name: &str| msa_src.lines().find_map(|l| l.trim().strip_prefix(name).map(|v| v.trim().to_ascii_lowercase()));
    if tag("// @msa-regulatory:").map(|v| !v.is_empty()).unwrap_or(false) {
        return RiskTier::Regulatory;
    }
    let escape = tag("// @msa-escape:").unwrap_or_default();
    let cost = tag("// @msa-station-cost:").unwrap_or_default();
    match (escape.as_str(), cost.as_str()) {
        ("customer", _) | ("multi-station", "high") => RiskTier::Strict,
        ("next-station", "low") => RiskTier::Relaxed,
        ("next-station", "high") | ("multi-station", "low") => RiskTier::Standard,
        _ => RiskTier::Undeclared,
    }
}
/// 測試輸出中 MSA 結果行的前綴
pub const RESULT_TAG: &str = "MSA-RESULT";

/// 量具（測試組）判定
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GaugeVerdict {
    Pass,
    SignOff,
    Redo,
    Broken,
}

/// 產品（積木）判定
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProductVerdict {
    Accept,
    Reject,
    NotMeasured,
}

/// 一次 MSA 量測的全部輸入
#[derive(Clone, Debug, PartialEq)]
pub struct MsaInput {
    pub module: String,
    pub tier: RiskTier,
    pub golden_total: usize,
    pub reference_ok: bool,
    pub standard_fails: Vec<usize>,
    pub challenges: Vec<(String, Vec<usize>)>,
    pub tolerance_ratio_pct: Option<f64>,
    pub brick_fails: Option<Vec<usize>>,
    pub blind: Option<(usize, usize)>,
}

/// 挑戰件抓出率（%）：至少一個黃金向量失敗的挑戰件比例；沒有挑戰件視為 0
/// @outputs: 2
pub fn kill_rate_pct(challenges: &[(String, Vec<usize>)]) -> f64 {
    if challenges.is_empty() {
        return 0.0;
    }
    let killed = challenges.iter().filter(|(_, fails)| !fails.is_empty()).count();
    killed as f64 * 100.0 / challenges.len() as f64
}

/// 量具判定：參考實作或標準件沒過 → 量具壞了；鑑別力或精度不足 → 重做／需簽核
/// @outputs: 4
pub fn gauge_verdict(input: &MsaInput) -> GaugeVerdict {
    if !input.reference_ok || !input.standard_fails.is_empty() || input.golden_total == 0 {
        return GaugeVerdict::Broken;
    }
    let t = thresholds(input.tier);
    let kill = kill_rate_pct(&input.challenges);
    let tol = input.tolerance_ratio_pct.unwrap_or(0.0);
    if kill < t.kill_redo_pct || tol > t.tol_redo_pct {
        return GaugeVerdict::Redo;
    }
    if kill < t.kill_pass_pct || tol >= t.tol_pass_pct {
        return if t.sign_off_allowed { GaugeVerdict::SignOff } else { GaugeVerdict::Redo };
    }
    GaugeVerdict::Pass
}

/// 產品判定：量具不可信時不量測；黃金向量或盲測有任何不符即拒收
/// @outputs: 3
pub fn product_verdict(input: &MsaInput) -> ProductVerdict {
    let gauge = gauge_verdict(input);
    let fails = match &input.brick_fails {
        Some(f) => f,
        None => return ProductVerdict::NotMeasured,
    };
    if gauge == GaugeVerdict::Broken || gauge == GaugeVerdict::Redo {
        return ProductVerdict::NotMeasured;
    }
    let blind_bad = input.blind.map(|(bad, _)| bad > 0).unwrap_or(false);
    if !fails.is_empty() || blind_bad {
        return ProductVerdict::Reject;
    }
    ProductVerdict::Accept
}

/// 單行黃金斷言的容差比（%）：`((x - e) / e).abs() < t` 視為相對容差 t；`(x - e).abs() < t` 為 t ÷ |e|；其他（含精確比對）回 None
/// @outputs: 3
pub fn line_tolerance_pct(line: &str) -> Option<f64> {
    let (before, after) = line.split_once(").abs() < ")?;
    let tol: f64 = after.trim().trim_end_matches(';').parse().ok()?;
    if let Some((_, denom)) = before.rsplit_once(") / ") {
        let _: f64 = denom.trim().trim_matches(|c| c == '(' || c == ')').parse().ok()?;
        return Some(tol * 100.0);
    }
    let (_, expected) = before.rsplit_once(" - ")?;
    let e: f64 = expected.trim().trim_matches(|c| c == '(' || c == ')').parse().ok()?;
    if e == 0.0 {
        return None;
    }
    Some(tol / e.abs() * 100.0)
}

/// 整組黃金向量的容差比：取最寬（最差）的一行；全是精確比對回 None
/// @outputs: 2
pub fn tolerance_ratio_pct(lines: &[String]) -> Option<f64> {
    lines.iter().filter_map(|l| line_tolerance_pct(l)).fold(None, |acc: Option<f64>, v| Some(acc.map_or(v, |a| a.max(v))))
}

/// 判定的中文標籤與燈號
/// @outputs: 4
pub fn gauge_label(v: GaugeVerdict) -> String {
    match v {
        GaugeVerdict::Pass => "🟢 量具合格".to_string(),
        GaugeVerdict::SignOff => "🟡 量具需簽核".to_string(),
        GaugeVerdict::Redo => "🔴 量具需重做".to_string(),
        GaugeVerdict::Broken => "🔴 量具壞了".to_string(),
    }
}

/// @outputs: 3
pub fn product_label(v: ProductVerdict) -> String {
    match v {
        ProductVerdict::Accept => "🟢 產品允收".to_string(),
        ProductVerdict::Reject => "🔴 產品拒收".to_string(),
        ProductVerdict::NotMeasured => "⚪ 產品未量測".to_string(),
    }
}

/// 失敗向量編號清單 → 「#1,#3」或「無」
/// @outputs: 2
fn fail_list(fails: &[usize]) -> String {
    if fails.is_empty() {
        return "無".to_string();
    }
    fails.iter().map(|k| format!("#{}", k)).collect::<Vec<_>>().join(",")
}

/// 測試輸出中的一行結果（key=value 以 | 分隔），供 `cargo run -- msa` 彙整
/// @outputs: 1
pub fn result_line(input: &MsaInput) -> String {
    let killed = input.challenges.iter().filter(|(_, f)| !f.is_empty()).count();
    let missed: Vec<String> = input.challenges.iter().filter(|(_, f)| f.is_empty()).map(|(n, _)| n.clone()).collect();
    let tol = input.tolerance_ratio_pct.map(|t| format!("{:.3}%", t)).unwrap_or_else(|| "N/A（精確比對）".to_string());
    let brick = input.brick_fails.as_ref().map(|f| fail_list(f)).unwrap_or_else(|| "（產品不存在）".to_string());
    let blind = input.blind.map(|(b, n)| format!("{}/{}", b, n)).unwrap_or_else(|| "（未執行）".to_string());
    format!(
        "{}|module={}|tier={}|gauge={}|product={}|golden={}|reference={}|standard_fail={}|kill={}/{}|missed={}|tol={}|brick_fail={}|blind_mismatch={}",
        RESULT_TAG, input.module, tier_label(input.tier), gauge_label(gauge_verdict(input)), product_label(product_verdict(input)), input.golden_total,
        if input.reference_ok { "通過公開標準件" } else { "未通過公開標準件" }, fail_list(&input.standard_fails),
        killed, input.challenges.len(), if missed.is_empty() { "無".to_string() } else { missed.join(",") }, tol, brick, blind
    )
}

/// 解析結果行 → (key, value) 清單
/// @outputs: 2
pub fn parse_result_line(line: &str) -> Option<Vec<(String, String)>> {
    let rest = line.trim().strip_prefix(RESULT_TAG)?;
    Some(
        rest.split('|')
            .filter_map(|kv| kv.split_once('='))
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    )
}

/// 管理報告（給不看程式的主管）：每個高風險模組一列，紅黃綠燈號與白話判定依據
/// @outputs: 1
pub fn report_markdown(rows: &[Vec<(String, String)>]) -> String {
    let get = |row: &Vec<(String, String)>, key: &str| row.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone()).unwrap_or_default();
    let mut out = String::from("## 量測系統分析（MSA）報告：先驗量具，再驗產品\n\n");
    out.push_str("| 模組 | 風險等級 | 量具 | 產品 | 參考實作 vs 公開標準件 | 標準件失敗 | 挑戰件抓出 | 漏抓的挑戰件 | 容差比 | 產品黃金向量失敗 | 盲測不一致 |\n|---|---|---|---|---|---|---|---|---|---|---|\n");
    for row in rows {
        out.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            get(row, "module"), get(row, "tier"), get(row, "gauge"), get(row, "product"), get(row, "reference"), get(row, "standard_fail"),
            get(row, "kill"), get(row, "missed"), get(row, "tol"), get(row, "brick_fail"), get(row, "blind_mismatch")
        ));
    }
    out.push_str("\n**判讀**：量具壞了 = 黃金向量連公認正確的參考實作都不過，要修規格而不是產品；量具需重做／需簽核 = 挑戰件（已知錯誤寫法）抓出率或容差比落在該風險等級的重做區／簽核區；");
    out.push_str("產品拒收 = 量具可信但產品沒過黃金向量或盲測（隨機輸入與參考實作比對）。\n\n");
    out.push_str("**各風險等級門檻**（抓出率 合格／重做線；容差比 合格／重做線；盲測筆數）：放寬 80/60%、15/40%、2000；標準 90/70%、10/30%、2000；");
    out.push_str("收緊與影響未知（調查中） 100/90%、5/10%、20000；法規死線 100%、5%、20000，無簽核放行區。\n");
    out
}

/// msa/<模組>.rs 的內容 → (挑戰件模組名清單, 盲測函式名)
/// @outputs: 1
pub fn scan_parts(msa_src: &str) -> (Vec<String>, Option<String>) {
    let challenges = msa_src
        .lines()
        .filter_map(|l| l.trim().strip_prefix("pub mod "))
        .map(|rest| rest.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect::<String>())
        .filter(|name| name.starts_with("challenge_"))
        .collect();
    let blind = msa_src
        .lines()
        .find_map(|l| l.trim().strip_prefix("// @msa-blind:").map(|f| f.trim().to_string()))
        .filter(|f| !f.is_empty());
    (challenges, blind)
}

/// 一個候選實作的量測函式：把每個黃金向量執行一次，回傳失敗的編號
/// @outputs: 1
fn render_gauge_fn(fn_name: &str, use_path: &str, lines: &[String]) -> String {
    let mut body = format!("fn {}() -> Vec<usize> {{\n    use {}::*;\n    let mut fails = Vec::new();\n", fn_name, use_path);
    let mut k = 0usize;
    for line in lines {
        if line.starts_with("let ") || line.ends_with(';') {
            let stmt = if line.ends_with(';') { line.clone() } else { format!("{};", line) };
            body.push_str(&format!("    {}\n", stmt));
            continue;
        }
        k += 1;
        // 挑戰件可能讓 unwrap() panic：一律記成「這個向量失敗」，不讓整個量測中斷
        body.push_str(&format!(
            "    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {})).unwrap_or(false) {{ fails.push({}); }}\n",
            line, k
        ));
    }
    body.push_str("    fails\n}\n\n");
    body
}

/// 產生 tests/msa_<模組>.rs：量具先量參考實作與各挑戰件；產品存在時再量產品並做盲測
/// @outputs: 2
pub fn render_msa_test(module: &str, msa_src: &str, golden_lines: &[String], brick_exists: bool) -> Option<String> {
    let total = golden_lines.iter().filter(|l| !l.starts_with("let ") && !l.ends_with(';')).count();
    if !msa_src.contains("pub mod standard") || total == 0 {
        return None;
    }
    let (challenges, blind) = scan_parts(msa_src);
    let tier = risk_tier(msa_src);
    let mut out = format!(
        "// 本檔由 cargo run -- msa-gen 依 msa/{0}.rs 與 specs/{0}.md 的 ```golden 區塊自動產生，請勿手改。\n#![allow(unused)]\n#[path = \"../msa/{0}.rs\"]\nmod parts;\nuse rust_bricks::msa::*;\n\n",
        module
    );
    out.push_str(&render_gauge_fn("gauge_on_standard", "parts::standard", golden_lines));
    for c in &challenges {
        out.push_str(&render_gauge_fn(&format!("gauge_on_{}", c), &format!("parts::{}", c), golden_lines));
    }
    if brick_exists {
        out.push_str(&render_gauge_fn("gauge_on_brick", &format!("rust_bricks::{}", module), golden_lines));
    }
    let challenge_list: Vec<String> = challenges.iter().map(|c| format!("(\"{0}\".to_string(), gauge_on_{0}())", c)).collect();
    let brick = if brick_exists { "Some(gauge_on_brick())".to_string() } else { "None".to_string() };
    let blind_expr = match (&blind, brick_exists) {
        (Some(f), true) => format!("Some(parts::blind_check(rust_bricks::{}::{}, {}))", module, f, thresholds(tier).blind_samples),
        _ => "None".to_string(),
    };
    let tol = match tolerance_ratio_pct(golden_lines) {
        Some(t) => format!("Some({:?})", t),
        None => "None".to_string(),
    };
    out.push_str(&format!(
        "#[test]\nfn msa_{0}() {{\n    let hook = std::panic::take_hook();\n    std::panic::set_hook(Box::new(|_| {{}}));\n    let input = MsaInput {{\n        module: \"{0}\".to_string(),\n        tier: RiskTier::{6:?},\n        golden_total: {1},\n        reference_ok: parts::standard_parts_ok(),\n        standard_fails: gauge_on_standard(),\n        challenges: vec![{2}],\n        tolerance_ratio_pct: {3},\n        brick_fails: {4},\n        blind: {5},\n    }};\n    std::panic::set_hook(hook);\n    let line = result_line(&input);\n    println!(\"{{}}\", line);\n    let gauge = gauge_verdict(&input);\n    assert!(gauge != GaugeVerdict::Broken && gauge != GaugeVerdict::Redo, \"量具不可信：{{}}\", line);\n    assert!(product_verdict(&input) != ProductVerdict::Reject, \"產品拒收：{{}}\", line);\n}}\n",
        module, total, challenge_list.join(", "), tol, brick, blind_expr, tier
    ));
    Some(out)
}

/// 依 msa/*.rs 與規格 ```golden 重新產生全部 tests/msa_<模組>.rs；回傳已同步的模組
/// @outputs: 1
pub fn sync_msa_tests(repo_root: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(repo_root.join("msa"))
        .map(|rd| rd.flatten().map(|e| e.file_name().to_string_lossy().to_string()).collect())
        .unwrap_or_default();
    names.retain(|n| n.ends_with(".rs"));
    names.sort();
    let mut synced = Vec::new();
    for file in names {
        let module = file.trim_end_matches(".rs").to_string();
        let msa_src = fs::read_to_string(repo_root.join("msa").join(&file)).unwrap_or_default();
        let spec = fs::read_to_string(repo_root.join("specs").join(format!("{}.md", module))).unwrap_or_default();
        let lines = crate::farm_tree::extract_golden_lines(&spec);
        let brick_exists = repo_root.join("src").join(format!("{}.rs", module)).exists();
        let path = repo_root.join("tests").join(format!("msa_{}.rs", module));
        if let Some(test) = render_msa_test(&module, &msa_src, &lines, brick_exists) {
            let _ = fs::write(&path, test);
            synced.push(module);
        }
    }
    synced
}
