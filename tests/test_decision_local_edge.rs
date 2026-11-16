use rust_bricks::decision_local_edge::*;

#[test]
fn test_select_local_edge_profile_rtx3050_boundaries() {
    // 1. Typical laptop with IDE/browser open: ~2800 MiB free VRAM -> chooses LiquidAI d1-3B Q4_K_M (leaves ~400-1560 MiB buffer)
    assert_eq!(
        select_local_edge_profile(2800, 4800, true, true, true, true),
        LocalEdgeModelId::D1_3bQ4KM
    );
    // 2. Kev-4B Q4_K_M strictly requires >= 3900 MiB free VRAM (measured 3899 MiB on RTX 3050) and English preference
    assert_eq!(
        select_local_edge_profile(3899, 4800, false, true, true, true),
        LocalEdgeModelId::D1_3bQ4KM
    );
    assert_eq!(
        select_local_edge_profile(3962, 4800, false, true, true, true),
        LocalEdgeModelId::Kev4bQ4KM
    );
    // 3. Tight 2550 MiB free VRAM -> falls back to LiquidAI d1-2B Q8_0 (measured 2381 MiB)
    assert_eq!(
        select_local_edge_profile(2550, 4800, true, true, true, true),
        LocalEdgeModelId::D1_2bQ8_0
    );
    // 4. Below 2500 MiB free VRAM -> falls back cleanly to Kev-0.8B CPU
    assert_eq!(
        select_local_edge_profile(1800, 4800, true, true, true, true),
        LocalEdgeModelId::Kev08bCpuFp32
    );
    // 5. Out-of-domain bounds guard
    assert_eq!(
        select_local_edge_profile(200_000, 4800, false, true, true, true),
        LocalEdgeModelId::Kev08bCpuFp32
    );
}

#[test]
fn test_compute_llamacpp_ubatch_prevents_http_500() {
    // Prevents `input (799 tokens) > batch size (512)` HTTP 500 error observed on LiquidAI d1-2B
    assert_eq!(compute_llamacpp_ubatch(799, 4096), Some((2048, 2048)));
    assert_eq!(compute_llamacpp_ubatch(2500, 4096), Some((4096, 4096)));
    assert_eq!(compute_llamacpp_ubatch(4097, 4096), None);
    assert_eq!(compute_llamacpp_ubatch(0, 4096), None);
    assert_eq!(compute_llamacpp_ubatch(100, 256), None);
}

#[test]
fn test_sanitize_kev_delimiters_and_softmax() {
    let raw = "  hello <|fim_suffix|> world |>  ";
    assert_eq!(sanitize_kev_delimiters(raw, 256), "hello <¦fim_suffix¦> world ¦>");

    let mut probs = [0.0f64; 3];
    let (idx, conf) = pointer_head_softmax_calibrated(&[5.0, 1.0, 1.0], 2.3511, &mut probs).unwrap();
    assert_eq!(idx, 0);
    assert!((probs.iter().sum::<f64>() - 1.0).abs() < 1e-12);
    assert!(conf > 0.5 && conf < 1.0);
    // Out-of-domain temperature rejected
    assert_eq!(pointer_head_softmax_calibrated(&[1.0, 2.0], 0.01, &mut probs), None);
}

#[test]
fn test_track1_guards_and_path_normalization() {
    // Regression test: preserve leading dot in `.agents/hooks.json` (unlike Python lstrip("./"))
    assert_eq!(
        normalize_repo_rel_path(r"C:\For Gemini\ai-interpreter\.agents\hooks.json", r"C:\For Gemini\ai-interpreter"),
        ".agents/hooks.json"
    );
    assert_eq!(normalize_repo_rel_path("./.agents/hooks.json", "/repo"), ".agents/hooks.json");

    assert_eq!(classify_track1_edit(".agents/hooks.json", "{}"), ("BLOCK", true));
    assert_eq!(
        classify_track1_edit("src/main.rs", "let k = \"AIzaSyABCDEFGHIJKLMNOP\";"),
        ("BLOCK", true)
    );
    assert_eq!(classify_track1_edit("evals/cases/exemplars.jsonl", "{}"), ("FUZZY", false));

    assert_eq!(classify_track1_command("git commit --no-verify -m x"), ("BLOCK", true));
    assert_eq!(classify_track1_command("git clean -fdx"), ("FUZZY", false));
    assert_eq!(classify_track1_command("cargo test"), ("ALLOW", true));

    assert_eq!(apply_low_conf_allow_escalation("ALLOW", 0.3278, 0.60), ("ESCALATE", 0.6722));
    assert_eq!(apply_low_conf_allow_escalation("ALLOW", 0.9678, 0.60), ("ALLOW", 0.9678));
    assert_eq!(resolve_dual_track_route_order("AUTO"), &["local_edge", "cloud_nimble"]);
    assert!(resolve_dual_track_route_order("OFF").is_empty());
}

#[test]
fn test_benchmark_specs_match_measured_rtx3050_data() {
    let d1_3b = get_model_benchmark_spec(LocalEdgeModelId::D1_3bQ4KM);
    assert_eq!(d1_3b.measured_vram_mb, 2403);
    assert!(d1_3b.supports_zh);
    let kev4b = get_model_benchmark_spec(LocalEdgeModelId::Kev4bQ4KM);
    assert_eq!(kev4b.measured_vram_mb, 3899);
    assert!(!kev4b.supports_zh);
}
