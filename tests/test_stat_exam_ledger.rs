// SPDX-License-Identifier: MIT
use rust_bricks::encoding_base64_json::JsonVal;
use rust_bricks::stat_exam_ledger::{
    stat_claim_ledger_register, stat_exam_check_item, stat_exam_level_code,
    stat_exam_runs_consistent, stat_exam_validate_bank_counts, stat_receipt_canonical_hash16,
};

#[test]
fn test_claim_ledger_bidirectional_c13() {
    let entries = vec![("fpy|main_vs_a|turn".to_string(), "C-001".to_string())];
    assert_eq!(
        stat_claim_ledger_register(&entries, "fpy", "main_vs_a", "turn", "C-001"),
        Ok(("fpy|main_vs_a|turn".to_string(), "C-001".to_string()))
    );
    assert_eq!(
        stat_claim_ledger_register(&entries, "fpy", "main_vs_a", "turn", "C-002"),
        Err("C13_CLAIM_ID_ALIAS_REJECTED")
    );
    assert_eq!(
        stat_claim_ledger_register(&entries, "latency", "main_vs_a", "turn", "C-001"),
        Err("C13_CLAIM_ID_COLLISION")
    );
    assert_eq!(
        stat_claim_ledger_register(&entries, " ", "main_vs_a", "turn", "C-003"),
        Err("EMPTY_CLAIM_FIELD")
    );
    // Shot 3: Pipe escaping prevents ("acc|armA", "armB", "%") vs ("acc", "armA|armB", "%") collision
    let pipe_a = stat_claim_ledger_register(&[], "acc|armA", "armB", "%", "C-10").expect("pipe a");
    let pipe_entries = vec![pipe_a.clone()];
    let pipe_b = stat_claim_ledger_register(&pipe_entries, "acc", "armA|armB", "%", "C-11").expect("pipe b distinct");
    assert_ne!(pipe_a.0, pipe_b.0);
}

#[test]
fn test_receipt_hash_and_exam_scorer() {
    let h1 = stat_receipt_canonical_hash16(
        "stat_gate_rs_v1.0",
        "deadbeef12345678",
        &["wilson".to_string(), "8".to_string(), "10".to_string()],
        &[
            ("n".to_string(), JsonVal::Num(10.0)),
            ("k".to_string(), JsonVal::Num(8.0)),
            ("cli_args".to_string(), JsonVal::Null),
        ],
    );
    let h2 = stat_receipt_canonical_hash16(
        "stat_gate_rs_v1.0",
        "deadbeef12345678",
        &["wilson".to_string(), "8".to_string(), "10".to_string()],
        &[("k".to_string(), JsonVal::Num(8.0)), ("n".to_string(), JsonVal::Num(10.0))],
    );
    assert_eq!(h1, h2);
    assert_eq!(h1.len(), 16);

    assert!(stat_exam_validate_bank_counts(114, 54, 54, 6, 114).is_ok());
    assert!(stat_exam_validate_bank_counts(114, 54, 54, 6, 113).is_err());
    assert_eq!(stat_exam_level_code("描述"), 0);
    assert_eq!(stat_exam_level_code("關聯"), 1);
    assert_eq!(stat_exam_level_code("因果"), 2);

    assert!(stat_exam_check_item(
        "REWRITE", "描述", "positive", &["C01".to_string()], true,
        "REWRITE", "描述", &["C01".to_string()], "因為樣本不足僅供參考"
    ));
    assert!(!stat_exam_check_item(
        "PASS", "描述", "negative", &[], false,
        "PASS", "描述", &["C01".to_string()], ""
    ));
    // Shot 8: Positive item trap-spamming must be rejected
    assert!(!stat_exam_check_item(
        "FAIL", "描述", "positive", &["C01".to_string()], false,
        "FAIL", "描述", &["C01".to_string(), "C02".to_string()], ""
    ));
    assert!(stat_exam_runs_consistent(
        "PASS", "描述", &["C02".to_string(), "C01".to_string()], " ok ",
        "PASS", "描述", &["C01".to_string(), "C02".to_string()], "ok"
    ));
}
