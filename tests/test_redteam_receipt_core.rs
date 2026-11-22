// SPDX-License-Identifier: MIT
use rust_bricks::redteam_receipt_core::{
    redteam_countersign_status, redteam_format_tsv_row, redteam_parse_tsv_line,
    redteam_sanitize_tsv, redteam_summarize_rows,
};

#[test]
fn test_redteam_tsv_format_parse_and_summary() {
    assert_eq!(redteam_sanitize_tsv("a\tb\nc\rd"), "a b c d");
    let row_str = redteam_format_tsv_row(
        "2026-11-22T00:00:00Z", "stat-gate", "abc123", "flash", 8, 7, "Argon", "PASS", "c1,c2", "ok\tnote"
    ).expect("valid row");
    assert!(!row_str.contains("ok\tnote"));
    let parsed = redteam_parse_tsv_line(&format!("{}\r\n", row_str)).expect("parsed CRLF");
    assert_eq!(parsed.scope, "stat-gate");
    assert_eq!(parsed.verdict, "PASS");
    assert_eq!(parsed.shots, 8);
    assert_eq!(parsed.confirmed_hits, 7);
    assert!(redteam_parse_tsv_line("2026\tstat-gate\tabc\tflash\tbadnum\t7\tArgon\tPASS").is_none());

    let sum = redteam_summarize_rows(&[parsed.clone()], "stat-gate");
    assert!(sum.ok);
    assert_eq!(sum.total_receipts, 1);
    assert_eq!(sum.matching_receipts, 1);
    assert_eq!(sum.verified_scopes, vec!["stat-gate".to_string()]);

    // Shot 9: Newer FAIL overrides earlier PASS
    let fail_str = redteam_format_tsv_row(
        "2026-11-22T01:00:00Z", "stat-gate", "def456", "flash", 8, 7, "Argon", "FAIL", "c1", "regressed"
    ).expect("fail row");
    let fail_parsed = redteam_parse_tsv_line(&fail_str).expect("parsed fail");
    let sum_fail = redteam_summarize_rows(&[parsed, fail_parsed], "stat-gate");
    assert!(!sum_fail.ok);
    assert!(sum_fail.verified_scopes.is_empty());
    assert_eq!(sum_fail.latest.as_ref().map(|r| r.verdict.as_str()), Some("FAIL"));

    assert_eq!(redteam_countersign_status(true, ""), "VERIFIED");
    assert_eq!(redteam_countersign_status(false, "docs only"), "SKIPPED");
    assert_eq!(redteam_countersign_status(false, ""), "MISSING");
}
