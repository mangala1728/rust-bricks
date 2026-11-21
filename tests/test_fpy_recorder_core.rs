use rust_bricks::fpy_recorder_core::*;

#[test]
fn test_fpy_redaction_and_utf16_clip() {
    let raw = concat!(
        "curl -H 'Authorization: bearer secret_token_1234567890' ",
        "https://admin:pass123@api.test?k=AIza",
        "SyAbCdEfGhIjKlMnOpQrStUvWxYz123456789 ",
        "stripe=sk_test_1234567890abcdefghijklmnop"
    );
    let (redacted, kinds) = fpy_redact_secrets(raw);
    assert!(!redacted.contains("pass123"));
    assert!(!redacted.contains("secret_token"));
    assert!(!redacted.contains("sk_test_1234"));
    assert!(kinds.contains(&"uri-cred") && kinds.contains(&"google-api-key") && kinds.contains(&"bearer-token") && kinds.contains(&"stripe-key"));

    // Truncated PEM key without END must redact body to end (no self-matching BEGIN leak!)
    let trunc_pem = concat!("-----", "BEGIN RSA ", "PRIVATE KEY-----\nMIIEpAIBAAKCAQEAsecretbody1234567890");
    let (pem_redacted, pem_kinds) = fpy_redact_secrets(trunc_pem);
    assert!(!pem_redacted.contains("MIIEpAIBAAKCAQEA"));
    assert!(pem_kinds.contains(&"private-key-block"));

    // Normal URL with port + email query param must NOT trigger uri-cred false positive
    let url_ok = "https://host:8443/path?email=alice@example.com";
    let (url_res, url_kinds) = fpy_redact_secrets(url_ok);
    assert_eq!(url_res, url_ok);
    assert!(url_kinds.is_empty());

    // Surrogate pair emoji (2 UTF-16 code units) must never split
    let msg = "測試😀".repeat(40);
    let (clipped, was_cut) = fpy_clip_utf16(&msg, 120);
    assert!(was_cut);
    let units: usize = clipped.chars().map(|c| c.len_utf16()).sum();
    assert!(units <= 120);
    assert!(clipped.ends_with("（已截斷）"));
}

#[test]
fn test_fpy_n1_all_13_n1b_quotes_and_shell_write() {
    // N1 last-segment exception rule + quote safety + multiline newline split
    let (n1_a, _, _) = fpy_detect_n1_and_n1b("grep x f; false", Some(1), true, false);
    assert!(n1_a);
    let (n1_b, _, _) = fpy_detect_n1_and_n1b("false; grep x f", Some(1), true, false);
    assert!(!n1_b);
    let (n1_q, _, _) = fpy_detect_n1_and_n1b("rg \"a;b\" Cargo.toml", Some(1), true, false);
    assert!(!n1_q);
    let (_, n1b_nl, _) = fpy_detect_n1_and_n1b("pytest -q\ntrue", Some(0), false, false);
    assert!(n1b_nl);

    // All 13 N1b masked patterns + comment/whitespace variants
    let patterns = [
        "cmd; true",
        "cmd; :",
        "cmd; exit 0",
        "cmd || true",
        "cmd || true # ok",
        "cmd || :",
        "cmd || exit 0",
        "cmd || (exit 0)",
        "cmd || echo ok",
        "cmd | Out-Null",
        "cmd -ErrorAction SilentlyContinue",
        "cmd -ea SilentlyContinue",
        "cmd -ErrorAction Ignore",
        "cmd 2>$null",
        "cmd 2> $null",
    ];
    for p in patterns {
        let (_, masked, _) = fpy_detect_n1_and_n1b(p, Some(0), false, false);
        assert!(masked, "Expected N1b masked for: {}", p);
    }
    let (_, _, sw1) = fpy_detect_n1_and_n1b("echo pwn>out.txt", Some(0), false, false);
    let (_, _, sw2) = fpy_detect_n1_and_n1b("echo pwn | out-file out.txt", Some(0), false, false);
    assert!(sw1 && sw2);
}

#[test]
fn test_fpy_n4_n6_hash_chain_k3_and_proposal_transitions() {
    // N4 40-cp + word boundary (desktop must NOT match stop) + sdk/bridge (A14)
    let (tag_desk, _, _) = fpy_detect_n4_prompt("desktop ui layout check", "composer", false, false, false);
    assert_eq!(tag_desk, "none");
    let (tag_sdk, _, _) = fpy_detect_n4_prompt("錯了重做", "sdk", true, false, false);
    assert_eq!(tag_sdk, "none");
    let (tag_br, _, _) = fpy_detect_n4_prompt("不對請改", "bridge", false, false, false);
    assert_eq!(tag_br, "n4_bridge");
    let (tag_wk, _, _) = fpy_detect_n4_prompt("這樣不對嗎？", "composer", false, false, false);
    assert_eq!(tag_wk, "n4_weak");

    // N6 hedge detection (abandoned / 未完成 must NOT trigger done)
    let (done_bad, _) = fpy_detect_n6_turn_end("status abandoned");
    assert!(done_bad.is_empty());
    let (done_ok, hedge) = fpy_detect_n6_turn_end("已完成所有修改\n未驗證：arm64 真機實測");
    assert!(!done_ok.is_empty() && hedge == vec!["unverified"]);

    // K3 seq continuity + K4 hash chain
    let l1 = r#"{"v":"fpy/1","seq":1,"prev":"0"}"#;
    let h1 = fpy_hash16(l1);
    let l2 = format!(r#"{{"v":"fpy/1","seq":2,"prev":"{}"}}"#, h1);
    let l2_bad_seq = format!(r#"{{"v":"fpy/1","seq":99,"prev":"{}"}}"#, h1);
    assert_eq!(fpy_verify_hash_chain(&[l1, &l2]), (2, 0));
    assert_eq!(fpy_verify_hash_chain(&[l1, &l2_bad_seq]), (2, 1));

    // P3/P22/Q5 + no resurrection of retired/rejected
    assert_eq!(fpy_evaluate_proposal_transition("provisional", 15, 0, 3, 2, 10, 0, false), ("trusted", true));
    assert_eq!(fpy_evaluate_proposal_transition("provisional", 5, 0, 2, 2, 35, 0, false), ("provisional_overdue", false));
    assert_eq!(fpy_evaluate_proposal_transition("retired", 20, 0, 5, 5, 10, 0, false), ("retired", false));
    assert_eq!(fpy_evaluate_proposal_transition("rejected", 20, 0, 5, 5, 10, 0, false), ("rejected", false));
}
