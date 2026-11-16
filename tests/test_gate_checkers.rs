// SPDX-License-Identifier: MIT OR Apache-2.0
//! ## Map
//! - Role: test/test_gate_checkers — Comprehensive 4-Corner & Golden Vector Unit Tests for `gate_*` Bricks
//! - Inputs: Deterministic TSV, Markdown, HTML/JS, Rust source, and MSA threshold fixtures
//! - Outputs: Verified exact counts, error messages, and mutation kills
//! - Invariants: Zero weak assertions; deterministic exact equality checks only.

use std::collections::BTreeSet;
use rust_bricks::gate_acceptance::{
    parse_matrix_tsv, run_acceptance_mutation_selftest, verify_req_sample_coverage,
    verify_spec_case_links_pairs,
};
use rust_bricks::gate_anti_hardcode::{
    run_anti_hardcode_calibration, scan_html_anti_hardcode, scan_rust_anti_hardcode,
};
use rust_bricks::gate_log_lifecycle::{
    extract_decision_targets, extract_known_decisions, parse_decision_relations,
    parse_lessons_rows, render_decisions_status_md,
};
use rust_bricks::gate_msa_verdict::{
    classify_protected_path_tier, eval_security_hook_standard_cmd, evaluate_gauge_verdict_x10,
    evaluate_product_verdict, lint_rust_snippet_in_memory, render_msa_markdown_rows,
    run_msa_mutation_selftest, MsaModuleRow,
};

#[test]
fn test_gate_anti_hardcode_calibration_and_defects() {
    let (ok, std_pass, std_total, chal_killed, chal_total) = run_anti_hardcode_calibration();
    assert!(ok);
    assert_eq!(std_pass, 3);
    assert_eq!(std_total, 3);
    assert_eq!(chal_killed, 7);
    assert_eq!(chal_total, 7);

    let float_hits = scan_rust_anti_hardcode("src/core/ephem.rs", "// comment f64 ok\npub x: f64");
    assert_eq!(float_hits.len(), 1);
    assert_eq!(float_hits[0].category, "FLOAT-IN-PURE-INT");

    let comment_only = scan_rust_anti_hardcode("src/core/ephem.rs", "// pure integer f64 avoided\npub x: i64");
    assert!(comment_only.is_empty());

    let dead_opt = scan_html_anti_hardcode("<select><option value=\"shravana\">S</option></select><script>const y=2;</script>");
    assert_eq!(dead_opt.len(), 1);
    assert_eq!(dead_opt[0].category, "DEAD-UI-OPTION");
}

#[test]
fn test_gate_acceptance_tsv_coverage_and_mutations() {
    let tsv = "\
REQ_ID\tREQ_DESC\tSPEC_FILE\tCASE_NAME\tSAMPLE_TYPE\tSTATUS\tVERIFIED_COMMIT
REQ-0001\tCore\tmsa/s.json\tC1\tGoldenSample\tPASS\t1083193
REQ-0001\tCore\tmsa/s.json\tC2\tLowerBoundPass\tPASS\t1083193
REQ-0001\tCore\tmsa/s.json\tC3\tUpperBoundPass\tPASS\t1083193
REQ-0001\tCore\tmsa/s.json\tC4\tLowerBoundFail\tPASS\t1083193
REQ-0001\tCore\tmsa/s.json\tC5\tUpperBoundFail\tPASS\t1083193
REQ-0001\tCore\tmsa/s.json\tC6\tNoiseAdversarial\tPASS\t1083193
";
    assert!(parse_matrix_tsv(tsv, 5).is_err());
    let rows = parse_matrix_tsv(tsv, 100).expect("valid matrix");
    assert_eq!(rows.len(), 6);
    assert_eq!(verify_req_sample_coverage(&rows), Ok(1));

    let specs = [
        ("msa/s.json", "C1", "GoldenSample"),
        ("msa/s.json", "C2", "LowerBoundPass"),
        ("msa/s.json", "C3", "UpperBoundPass"),
        ("msa/s.json", "C4", "LowerBoundFail"),
        ("msa/s.json", "C5", "UpperBoundFail"),
        ("msa/s.json", "C6", "NoiseAdversarial"),
    ];
    assert!(verify_spec_case_links_pairs(&rows, &specs).is_ok());
    assert!(run_acceptance_mutation_selftest(&rows, &specs));
}

#[test]
fn test_gate_log_lifecycle_decisions_and_lessons() {
    let extracted = extract_decision_targets("D-0054（部分：依 D-0076 查證），D-0060");
    assert_eq!(extracted.len(), 2);
    assert_eq!(extracted[0].0, "D-0054");
    assert_eq!(extracted[1].0, "D-0060");

    let dec_md = "## D-0001 初版\n- 內容\n## D-0002 修訂\n- **更正**：D-0001（依 D-0001 說明修正）\n";
    let rels = parse_decision_relations(dec_md).expect("valid monotonic correction");
    assert_eq!(rels.len(), 1);
    assert_eq!(rels[0].target, "D-0001");
    assert_eq!(rels[0].source, "D-0002");
    assert!(render_decisions_status_md(&rels).contains("| D-0001 | 更正 | D-0002 |"));
    let dec_partial = "## D-0001 初版\n## D-0002 修訂\n- **更正**：D-0001（部分：第二項）\n";
    let rels_p = parse_decision_relations(dec_partial).expect("valid partial");
    assert!(render_decisions_status_md(&rels_p).contains("| D-0001 | 部分更正 | D-0002 |"));

    let bad_order = "## D-0001 初版\n- **取代**：D-0002\n## D-0002 較新\n";
    assert!(parse_decision_relations(bad_order).is_err());

    let known: BTreeSet<String> = extract_known_decisions(dec_md);
    let lessons_ok = "| L-001 | 避免假值 | `gate:scripts/a.py` | D-0001 | D-0002 |\n";
    let parsed = parse_lessons_rows(lessons_ok, &known).expect("valid lesson");
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].id, "L-001");

    let lessons_bad = "| L-002 | 少一欄 | gate:a | D-0001 |\n";
    assert!(parse_lessons_rows(lessons_bad, &known).is_err());
}

#[test]
fn test_gate_msa_verdict_rule0_and_seeds() {
    assert!(run_msa_mutation_selftest());
    assert_eq!(evaluate_gauge_verdict_x10("Strict", true, 4, 0, 7, 7, 0), "🟢 量具合格");
    assert_eq!(evaluate_gauge_verdict_x10("Standard", true, 4, 0, 10, 8, 50), "🟡 量具需簽核");
    assert_eq!(evaluate_gauge_verdict_x10("Relaxed", true, 4, 0, 10, 8, 100), "🟢 量具合格");
    assert_eq!(rust_bricks::gate_msa_verdict::get_tier_label("relaxed"), "放寬（下一站可便宜檢出）");
    assert_eq!(evaluate_product_verdict("🟢 量具合格", 6, 0), "🔴 產品拒收");
    assert_eq!(evaluate_product_verdict("🟢 量具合格", 0, 0), "🟢 產品允收");

    assert!(eval_security_hook_standard_cmd("git status", false));
    assert!(eval_security_hook_standard_cmd("git commit --no-verify", true));
    assert_eq!(classify_protected_path_tier("context/DECISIONS.md"), 1);
    assert_eq!(classify_protected_path_tier("scripts/a.py"), 2);
    assert_eq!(classify_protected_path_tier("docs/standards/CHARTER.md"), 3);

    assert!(lint_rust_snippet_in_memory("pub fn x() {}").contains(&"missing-map"));
    let md = render_msa_markdown_rows(&[MsaModuleRow {
        id: "anti-hardcode-e2e-gauge".into(),
        name: "E2E".into(),
        risk_tier: "Strict".into(),
        tier_label: "收緊".into(),
        source_citation: "@source: test".into(),
        gauge_agent_id: "agent-a".into(),
        product_agent_id: "agent-b".into(),
        separation_ok: true,
        gauge_verdict: "🟢 量具合格",
        product_verdict: "🔴 產品拒收",
        std_total: 4,
        std_fails: 0,
        chal_total: 7,
        chal_killed: 7,
        tol_pct_x10: 0,
        product_fails: 6,
        blind_mismatches: 0,
        blind_total: 2000,
    }]);
    assert!(md.contains("🟢 量具合格") && md.contains("🔴 產品拒收"));
}
