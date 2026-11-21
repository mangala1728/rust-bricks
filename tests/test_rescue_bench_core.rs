use rust_bricks::rescue_bench_core::*;

#[test]
fn test_rescue_claimed_done_and_laundering_guard() {
    // Generic-only "未完成：無" must NOT escape claimed_done=true
    assert!(rescue_classify_claimed_done("已完成\n未完成：無", false));
    assert!(rescue_classify_claimed_done("未解決：邊界情況", false));
    // Concrete file/test blocker sets claimed_done=false (including half-width colon & markdown list)
    assert!(!rescue_classify_claimed_done("未解決：tests/test_stat_gate_core.rs 尚缺實測", false));
    assert!(!rescue_classify_claimed_done("- **未完成**: backend/app.py 測試失敗", false));
    // Blind judge fallback sets claimed_done=false
    assert!(!rescue_classify_claimed_done("我還沒測完 backend/app.py", true));
}

#[test]
fn test_rescue_fisher_yates_conservation() {
    // 1 task, 2 reps: AB has 4 passes ([1,1], [1,1]), C has 0 passes ([0,0]).
    // Under true without-replacement Fisher-Yates permutation, AB ALWAYS gets >=2 passes -> strict rescue MUST be 0!
    // (With-replacement bug previously allowed AB to draw 0 passes and falsely trigger strict rescue!)
    let a = [1, 1];
    let b = [1, 1];
    let c = [0, 0];
    let ci = [1, 1];
    let (strict, rev, med, p95) = rescue_chance_baseline_strict(&a, &b, &c, &ci, 2, 300, 77).expect("fy");
    assert_eq!((strict, rev, med, p95), (0, 1, 0.0, 0.0));
}

#[test]
fn test_rescue_validity_arm_leak_chance_and_honest_metric() {
    // A11/A13/P20
    assert_eq!(rescue_validate_outcome("runner-1", false, true, false, true), (true, "VALID"));
    assert_eq!(rescue_validate_outcome("fpy-recorder", false, true, false, true), (false, "INVALID_OUTCOME_AUTHOR"));
    assert_eq!(rescue_validate_outcome("runner-1", true, true, false, true), (false, "INVALID_PLUGINS_SET"));
    assert_eq!(rescue_validate_outcome("runner-1", false, true, true, true), (false, "UNKNOWN_EVENT_EFFORT"));
    assert_eq!(rescue_validate_outcome("runner-1", false, true, false, false), (false, "HIDDEN_SUITE_MISMATCH"));

    // A12 opaque arm check
    assert!(rescue_check_arm_leak("running arm 丙1 on T-01", &["丙1", "arm_c1"]));
    assert!(!rescue_check_arm_leak("running arm k7q2 on T-01", &["丙1", "arm_c1"]));

    // §5.1 strict rescue + chance baseline (2 tasks, 2 reps each: task0 rescued by C, task1 reversed)
    let a = [0, 0, 1, 0];
    let b = [0, 0, 0, 1];
    let c = [1, 1, 0, 0];
    let ci = [1, 1, 0, 0];
    let (strict, rev, med, p95) = rescue_chance_baseline_strict(&a, &b, &c, &ci, 2, 500, 99).expect("chance");
    assert_eq!((strict, rev), (1, 1));
    assert!(p95 >= med);

    // §12.6 honest metric formatting: n<10 never %, k=0 never 0%
    let m_small = rescue_format_honest_metric(3, 5, 1.0).expect("3/5");
    assert!(m_small.starts_with("3/5 (95% CI"));
    let m_zero = rescue_format_honest_metric(0, 54, 1.0).expect("0/54");
    assert!(m_zero.contains("upper bound <= 5.6%") && !m_zero.contains("0%"));
}
