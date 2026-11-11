use rust_bricks::cloud_farmer::{failure_outcome, mock_allowed, provider_order};
use rust_bricks::farm_tree::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn round(model: &str, class: &str) -> RoundRecord {
    RoundRecord { model: model.to_string(), class: class.to_string(), first_line: format!("{} 的錯誤", class) }
}

fn dead(branch: &str, model: &str, failure: &str, hash: &str) -> DeadEnd {
    DeadEnd {
        branch: branch.to_string(),
        failure: failure.to_string(),
        model: model.to_string(),
        spec_hash: hash.to_string(),
        lesson: "教訓".to_string(),
        rounds: vec![(model.to_string(), failure.to_string())],
    }
}

#[test]
fn classify_failure_covers_every_category() {
    assert_eq!(classify_failure("[infra] 所有免費 API 金鑰池皆無法連線"), "infra");
    assert_eq!(classify_failure("模型輸出缺少 ```rust:module 標籤"), "format");
    assert_eq!(classify_failure("error[E0425]: cannot find value `x`\nerror: could not compile"), "compile");
    let golden = "---- golden_vectors_from_spec stdout ----\nthread 'golden_vectors_from_spec' panicked at tests/golden_m.rs:7:5:\nspec golden vector #2\n";
    assert_eq!(classify_failure(golden), "golden:#2");
    assert_eq!(first_error_line(golden), "spec golden vector #2");
    assert_eq!(first_error_line("warning: x\nerror[E0425]: cannot find value"), "error[E0425]: cannot find value");
    let guard = "硬體級守衛 (guard.rs) 攔截:\n【函式過長攔截】函式超過 60 行上限";
    assert_eq!(classify_failure(guard), "guard:函式過長攔截");
    let test = "---- test_window_edges stdout ----\nassertion failed: (x - 1.0).abs() < 1e-6";
    assert_eq!(classify_failure(test), "test:test_window_edges");
    assert_eq!(classify_failure("something odd"), "other");
}

#[test]
fn two_strikes_switch_direction_and_fourth_escalates() {
    let mut s = StrikeState::default();
    s = next_strike(&s, "compile");
    assert_eq!(strike_action(&s), StrikeAction::Continue);
    s = next_strike(&s, "compile");
    assert_eq!((s.count, strike_action(&s)), (2, StrikeAction::SwitchDirection));
    s = next_strike(&s, "compile");
    assert_eq!(strike_action(&s), StrikeAction::Continue);
    s = next_strike(&s, "compile");
    assert_eq!(strike_action(&s), StrikeAction::Escalate);
    let reset = next_strike(&s, "golden:#1");
    assert_eq!((reset.count, strike_action(&reset)), (1, StrikeAction::Continue));
    assert!(strike_note(&StrikeState { class: "compile".into(), count: 2 }).contains("二振出局"));
}

#[test]
fn provider_rotates_on_each_switch() {
    assert_eq!(provider_order(0), [0, 1, 2]);
    assert_eq!(provider_order(1), [1, 2, 0]);
    assert_eq!(provider_order(2), [2, 0, 1]);
    assert_eq!(provider_order(3), [0, 1, 2]);
}

#[test]
fn utc_date_matches_independent_reference() {
    // 參考值以 Python calendar.timegm 計算
    assert_eq!(utc_yyyymmdd(0), "19700101");
    assert_eq!(utc_yyyymmdd(86_399), "19700101");
    assert_eq!(utc_yyyymmdd(951_782_400), "20000229");
    assert_eq!(utc_yyyymmdd(1_793_836_799), "20261104");
    assert_eq!(utc_yyyymmdd(4_107_542_400), "21000301");
    assert_eq!(utc_yyyymmdd(253_402_300_799), "99991231");
    assert_eq!(utc_yyyymmdd(u64::MAX), "99991231");
}

#[test]
fn attempt_branch_names_round_trip() {
    assert_eq!(attempt_branch_module("attempt/20261104-farm-dsp_goertzel").as_deref(), Some("dsp_goertzel"));
    assert_eq!(attempt_branch_module("origin/attempt/20261104-farm-dsp_goertzel-r2").as_deref(), Some("dsp_goertzel"));
    assert_eq!(attempt_branch_module("refs/remotes/origin/attempt/20261104-farm-a_r2"), Some("a_r2".to_string()));
    assert_eq!(attempt_branch_module("attempt/20261104-manual-idea"), None);
    assert_eq!(attempt_branch_module("contrib/cloud-farm-x"), None);
    let existing = vec!["origin/attempt/20261104-farm-m".to_string(), "attempt/20261104-farm-m-r2".to_string()];
    assert_eq!(attempt_branch_name("20261104", "m", &existing), "attempt/20261104-farm-m-r3");
    assert_eq!(attempt_branch_name("20261105", "m", &existing), "attempt/20261105-farm-m");
}

#[test]
fn abandon_message_parses_back_into_dead_end() {
    let input = AbandonInput {
        module: "dsp_goertzel".into(),
        spec_path: "specs/dsp_goertzel.md".into(),
        spec_hash: "abc123".into(),
        round: 2,
        outcome: "abandoned".into(),
        rounds: vec![round("gemini:g1", "compile"), round("cloud:q1", "golden:#2")],
        evidence: "  error[E0425]".into(),
    };
    let msg = abandon_message(&input);
    assert!(msg.starts_with("chore(cloud-farm): abandon dsp_goertzel round 2\n\nAI-Handoff: 1\nStatus: abandoned\n"));
    let parsed = parse_dead_end("attempt/20261104-farm-dsp_goertzel", &msg);
    let d = match parsed {
        Some(d) => d,
        None => panic!("abandon_message 必須能被 parse_dead_end 讀回"),
    };
    assert_eq!(d.failure, "golden:#2");
    assert_eq!(d.model, "cloud:q1");
    assert_eq!(d.spec_hash, "abc123");
    assert_eq!(d.rounds, vec![("gemini:g1".to_string(), "compile".to_string()), ("cloud:q1".to_string(), "golden:#2".to_string())]);
    assert!(d.lesson.starts_with("golden:#2："));
    assert_eq!(parse_dead_end("x", "feat(x): y\n\nStatus: done\n"), None);
}

#[test]
fn stale_dead_ends_from_old_spec_versions_are_ignored() {
    let all = vec![dead("a1", "m1", "compile", "old"), dead("a2", "m1", "compile", "new"), dead("a3", "m2", "compile", "")];
    let fresh = fresh_dead_ends(&all, "new");
    assert_eq!(fresh.len(), 1);
    assert_eq!(fresh[0].branch, "a2");
    assert!(fresh_dead_ends(&all, "").is_empty());
}

#[test]
fn spec_suspect_needs_two_different_models_on_same_golden_vector() {
    let same_model = vec![("m1".to_string(), "golden:#1".to_string()), ("m1".to_string(), "golden:#1".to_string())];
    assert_eq!(spec_suspect(&same_model), None);
    let two_models = vec![("m1".to_string(), "golden:#1".to_string()), ("m2".to_string(), "golden:#1".to_string())];
    assert_eq!(spec_suspect(&two_models).as_deref(), Some("golden:#1"));
    let different_vectors = vec![("m1".to_string(), "golden:#1".to_string()), ("m2".to_string(), "golden:#2".to_string())];
    assert_eq!(spec_suspect(&different_vectors), None);
    let compile = vec![("m1".to_string(), "compile".to_string()), ("m2".to_string(), "compile".to_string())];
    assert_eq!(spec_suspect(&compile), None);
}

#[test]
fn module_state_precedence() {
    let one = vec![dead("a", "m1", "compile", "h")];
    let three = vec![dead("a", "m1", "compile", "h"), dead("b", "m2", "test:t", "h"), dead("c", "m1", "compile", "h")];
    let suspect = vec![dead("a", "m1", "golden:#3", "h"), dead("b", "m2", "golden:#3", "h")];
    assert_eq!(module_state(true, true, &three), ModuleState::Done);
    assert_eq!(module_state(false, true, &three), ModuleState::Claimed);
    assert_eq!(module_state(false, false, &suspect), ModuleState::Suspect("golden:#3".into()));
    assert_eq!(module_state(false, false, &three), ModuleState::Blocked(3));
    assert_eq!(module_state(false, false, &one), ModuleState::Pending(1));
    assert_eq!(module_state(false, false, &[]), ModuleState::Pending(0));
}

#[test]
fn failure_outcome_precedence() {
    let fresh_two = vec![dead("a", "m1", "compile", "h"), dead("b", "m1", "compile", "h")];
    assert_eq!(failure_outcome(&[], &[round("m1", "compile")], false), "abandoned");
    assert_eq!(failure_outcome(&fresh_two, &[round("m1", "compile")], false), "blocked");
    assert_eq!(failure_outcome(&[], &[round("m1", "compile")], true), "escalated");
    let golden = [round("m1", "golden:#1"), round("m2", "golden:#1")];
    assert_eq!(failure_outcome(&[], &golden, true), "suspect");
}

#[test]
fn golden_block_becomes_assertions_that_ai_cannot_edit() {
    let spec = "# x\n```golden\n// 準備\nlet prices = [100.0, 120.0, 90.0];\n(fin_max_drawdown(&prices).unwrap() - 0.25).abs() < 1e-4\nfin_calmar_ratio(0.2, 0.0).is_none()\n```\n其他文字\n```golden\nignored()\n```\n";
    let lines = extract_golden_lines(spec);
    assert_eq!(lines.len(), 3);
    assert_eq!(golden_assert_count(&lines), 2);
    let rendered = match render_golden_test("fin_calmar_ratio", &lines) {
        Some(r) => r,
        None => panic!("有斷言時必須產生測試檔"),
    };
    assert!(rendered.contains("use rust_bricks::fin_calmar_ratio::*;"));
    assert!(rendered.contains("    let prices = [100.0, 120.0, 90.0];\n"));
    assert!(rendered.contains("assert!((fin_max_drawdown(&prices).unwrap() - 0.25).abs() < 1e-4, \"spec golden vector #1\");"));
    assert!(rendered.contains("assert!(fin_calmar_ratio(0.2, 0.0).is_none(), \"spec golden vector #2\");"));
    assert_eq!(render_golden_test("m", &["let x = 1;".to_string()]), None);
    assert!(extract_golden_lines("沒有區塊").is_empty());
}

#[test]
fn evidence_redacts_keys_and_summaries_parse() {
    assert_eq!(redact_keys("GET /v1?key=AIzaSECRET&alt=json"), "GET /v1?key=***&alt=json");
    assert_eq!(redact_keys("Authorization: Bearer gsk_SECRET more"), "Authorization: Bearer *** more");
    assert_eq!(redact_keys("no secrets"), "no secrets");
    assert!(evidence_lines("error: boom key=SECRET\nnoise").contains("key=***"));
    let out = "running 3 tests\ntest result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n";
    assert_eq!(test_pass_summary(out), "3 passed");
    assert_eq!(truncate_chars("一二三四", 2), "一…");
    assert_eq!(truncate_chars("一二三四", 4), "一二三四");
    assert_eq!(truncate_chars(&"x".repeat(80), 72).chars().count(), 72);
    assert_eq!(truncate_chars("abc", 0), "");
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env_remove("GIT_DIR")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_PREFIX")
        .output();
    match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        Ok(o) => panic!("git {:?} 失敗：{}", args, String::from_utf8_lossy(&o.stderr)),
        Err(e) => panic!("無法執行 git：{}", e),
    }
}

fn scratch_repo(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
    for sub in ["src", "tests", "specs"] {
        let _ = fs::create_dir_all(dir.join(sub));
    }
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["config", "user.name", "farm-test"]);
    git(&dir, &["config", "user.email", "farm-test@example.invalid"]);
    let _ = fs::write(dir.join("src/lib.rs"), "pub mod done_mod;\n");
    let _ = fs::write(dir.join("src/done_mod.rs"), "// done\n");
    let _ = fs::write(dir.join("specs/done_mod.md"), "# done\n");
    let _ = fs::write(dir.join("specs/new_mod.md"), "# new v1\n");
    let _ = fs::write(dir.join("specs/template.md"), "# template\n");
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "chore: init"]);
    dir
}

fn abandon_round(dir: &Path, module: &str, model: &str, class: &str) -> String {
    let hash = spec_blob_hash(dir, &format!("specs/{}.md", module));
    let _ = fs::write(dir.join("src").join(format!("{}.rs", module)), "// failed attempt\n");
    let input = AbandonInput {
        module: module.into(),
        spec_path: format!("specs/{}.md", module),
        spec_hash: hash,
        round: 1,
        outcome: "abandoned".into(),
        rounds: vec![round(model, class)],
        evidence: "  error".into(),
    };
    match record_abandoned_round(dir, module, &abandon_message(&input), false) {
        Ok(branch) => branch,
        Err(e) => panic!("record_abandoned_round 失敗：{}", e),
    }
}

#[test]
fn attempt_tree_lives_in_git_and_drives_the_queue() {
    let dir = scratch_repo("farm_tree_queue");
    let b1 = abandon_round(&dir, "new_mod", "m1", "compile");
    assert!(b1.starts_with("attempt/") && b1.ends_with("-farm-new_mod"));
    assert!(!dir.join("src/new_mod.rs").exists(), "切回原分支後失敗的程式碼不應留在工作區");
    assert_eq!(git(&dir, &["rev-parse", "--abbrev-ref", "HEAD"]).trim(), "main");

    let queue = scan_queue(&dir);
    assert_eq!(queue.len(), 2, "template.md 不算佇列");
    assert_eq!(queue[0].state, ModuleState::Done);
    assert_eq!(queue[1].state, ModuleState::Pending(1));
    assert!(queue_kv_lines(&queue).contains("next_spec=specs/new_mod.md\nnext_module=new_mod\n"));

    let b2 = abandon_round(&dir, "new_mod", "m1", "compile");
    let _ = abandon_round(&dir, "new_mod", "m2", "test:t");
    assert!(b2.ends_with("-r2"));
    assert_eq!(scan_queue(&dir)[1].state, ModuleState::Blocked(3));
    assert!(queue_kv_lines(&scan_queue(&dir)).contains("next_spec=\nnext_module=\nblocked=new_mod\n"));

    // 日誌保鮮：規格改版後舊死路自動失效，模組重新回到待做
    let _ = fs::write(dir.join("specs/new_mod.md"), "# new v2（修正黃金向量）\n");
    git(&dir, &["commit", "-q", "-am", "docs(spec): fix new_mod"]);
    assert_eq!(scan_queue(&dir)[1].state, ModuleState::Pending(0));

    // 入庫後死路分支過期並清除
    let _ = fs::write(dir.join("src/new_mod.rs"), "// harvested\n");
    let pruned = prune_resolved_attempts(&dir, false);
    assert_eq!(pruned.len(), 3);
    assert!(list_attempt_refs(&dir).is_empty());
}

#[test]
fn local_and_remote_copies_of_one_attempt_count_once() {
    let dir = scratch_repo("farm_tree_remote");
    let remote = Path::new(env!("CARGO_TARGET_TMPDIR")).join("farm_tree_remote_origin.git");
    let _ = fs::remove_dir_all(&remote);
    git(&dir, &["init", "-q", "--bare", remote.to_string_lossy().as_ref()]);
    git(&dir, &["remote", "add", "origin", remote.to_string_lossy().as_ref()]);
    let branch = abandon_round(&dir, "new_mod", "m1", "compile");
    git(&dir, &["push", "-q", "origin", branch.as_str()]);
    git(&dir, &["fetch", "-q", "origin"]);
    assert_eq!(list_attempt_refs(&dir).len(), 2, "本機與 origin/ 各一份");
    assert_eq!(load_dead_ends(&dir, "new_mod").len(), 1);
    assert_eq!(scan_queue(&dir)[1].state, ModuleState::Pending(1));
}

#[test]
fn farmer_prompt_carries_dead_ends_before_error_feedback() {
    use rust_bricks::cloud_farmer::build_farmer_prompt;
    let fresh = vec![dead("attempt/20261104-farm-m", "m1", "golden:#1", "h")];
    let memory = dead_end_prompt(&fresh);
    let prompt = build_farmer_prompt("m", "SPEC", &memory, "ERR-LOG");
    let mem_pos = prompt.find("已知死路");
    let err_pos = prompt.find("ERR-LOG");
    assert!(mem_pos.is_some() && err_pos.is_some() && mem_pos < err_pos);
    assert!(prompt.contains("attempt/20261104-farm-m（golden:#1，模型 m1）：教訓"));
    assert!(prompt.contains("```golden"));
    assert!(dead_end_prompt(&[]).is_empty());
}

#[test]
fn mock_responses_are_refused_inside_github_actions() {
    assert!(!mock_allowed(Some("true")));
    assert!(mock_allowed(None));
    assert!(mock_allowed(Some("false")));
}
