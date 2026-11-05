use rust_bricks::msa::*;

fn input(reference_ok: bool, standard: Vec<usize>, killed: usize, total: usize, tol: Option<f64>, brick: Option<Vec<usize>>, blind: Option<(usize, usize)>) -> MsaInput {
    let challenges = (0..total).map(|i| (format!("c{}", i), if i < killed { vec![1] } else { vec![] })).collect();
    MsaInput {
        module: "m".to_string(),
        tier: RiskTier::Standard,
        golden_total: 5,
        reference_ok,
        standard_fails: standard,
        challenges,
        tolerance_ratio_pct: tol,
        brick_fails: brick,
        blind,
    }
}

#[test]
fn gauge_is_broken_when_reference_or_standard_parts_fail() {
    assert_eq!(gauge_verdict(&input(false, vec![], 5, 5, None, None, None)), GaugeVerdict::Broken);
    assert_eq!(gauge_verdict(&input(true, vec![2, 7], 5, 5, None, None, None)), GaugeVerdict::Broken);
    let mut empty = input(true, vec![], 5, 5, None, None, None);
    empty.golden_total = 0;
    assert_eq!(gauge_verdict(&empty), GaugeVerdict::Broken);
}

#[test]
fn kill_rate_tiers_follow_70_and_90_percent() {
    assert_eq!(kill_rate_pct(&[]), 0.0);
    assert_eq!(gauge_verdict(&input(true, vec![], 10, 10, None, None, None)), GaugeVerdict::Pass);
    assert_eq!(gauge_verdict(&input(true, vec![], 9, 10, None, None, None)), GaugeVerdict::Pass);
    assert_eq!(gauge_verdict(&input(true, vec![], 8, 10, None, None, None)), GaugeVerdict::SignOff);
    assert_eq!(gauge_verdict(&input(true, vec![], 7, 10, None, None, None)), GaugeVerdict::SignOff);
    assert_eq!(gauge_verdict(&input(true, vec![], 6, 10, None, None, None)), GaugeVerdict::Redo);
    assert_eq!(gauge_verdict(&input(true, vec![], 0, 0, None, None, None)), GaugeVerdict::Redo);
}

#[test]
fn tolerance_ratio_tiers_follow_10_and_30_percent() {
    assert_eq!(gauge_verdict(&input(true, vec![], 5, 5, Some(9.9), None, None)), GaugeVerdict::Pass);
    assert_eq!(gauge_verdict(&input(true, vec![], 5, 5, Some(10.0), None, None)), GaugeVerdict::SignOff);
    assert_eq!(gauge_verdict(&input(true, vec![], 5, 5, Some(30.0), None, None)), GaugeVerdict::SignOff);
    assert_eq!(gauge_verdict(&input(true, vec![], 5, 5, Some(30.1), None, None)), GaugeVerdict::Redo);
}

#[test]
fn product_is_measured_only_with_a_trustworthy_gauge() {
    assert_eq!(product_verdict(&input(true, vec![], 5, 5, None, Some(vec![]), Some((0, 2000)))), ProductVerdict::Accept);
    assert_eq!(product_verdict(&input(true, vec![], 5, 5, None, Some(vec![1]), Some((0, 2000)))), ProductVerdict::Reject);
    assert_eq!(product_verdict(&input(true, vec![], 5, 5, None, Some(vec![]), Some((3, 2000)))), ProductVerdict::Reject);
    assert_eq!(product_verdict(&input(true, vec![], 8, 10, None, Some(vec![]), None)), ProductVerdict::Accept);
    assert_eq!(product_verdict(&input(true, vec![2], 5, 5, None, Some(vec![]), Some((0, 2000)))), ProductVerdict::NotMeasured);
    assert_eq!(product_verdict(&input(true, vec![], 1, 5, None, Some(vec![]), None)), ProductVerdict::NotMeasured);
    assert_eq!(product_verdict(&input(true, vec![], 5, 5, None, None, None)), ProductVerdict::NotMeasured);
}

#[test]
fn tolerance_ratio_reads_absolute_and_relative_forms() {
    // EWMA 舊版：絕對容差 1e-4、預期值 0.00043 → 約 23.26%，落在 10~30% 需簽核區
    let old = line_tolerance_pct("(ewma_variance_step(0.0004, 0.03, 0.94).unwrap() - 0.000430).abs() < 1e-4");
    assert!(matches!(old, Some(v) if (v - 23.2558).abs() < 0.001));
    assert_eq!(gauge_verdict(&input(true, vec![], 5, 5, old, None, None)), GaugeVerdict::SignOff);
    // EWMA 新版：相對容差 1e-9 → 1e-7%
    let new = line_tolerance_pct("((ewma_variance_step(0.0004, 0.03, 0.94).unwrap() - 0.00043) / 0.00043).abs() < 1e-9");
    assert!(matches!(new, Some(v) if (v - 1e-7).abs() < 1e-12));
    assert_eq!(line_tolerance_pct("(c[3] - 0.0).abs() < 1e-4"), None);
    assert_eq!(line_tolerance_pct("canfd_crc17_slice(&[]) == Some(0)"), None);
    let lines = vec!["(f(1.0).unwrap() - 2.0).abs() < 0.1".to_string(), "(f(2.0).unwrap() - 4.0).abs() < 0.1".to_string()];
    assert!(matches!(tolerance_ratio_pct(&lines), Some(v) if (v - 5.0).abs() < 1e-9));
    assert_eq!(tolerance_ratio_pct(&["x == 1".to_string()]), None);
}

#[test]
fn result_line_round_trips_into_the_management_report() {
    let line = result_line(&input(true, vec![2, 7], 4, 5, None, None, None));
    assert!(line.starts_with("MSA-RESULT|module=m|tier=標準|gauge=🔴 量具壞了|product=⚪ 產品未量測|"));
    let row = match parse_result_line(&format!("    {}", line)) {
        Some(r) => r,
        None => panic!("結果行必須能被解析"),
    };
    let get = |k: &str| row.iter().find(|(key, _)| key == k).map(|(_, v)| v.clone()).unwrap_or_default();
    assert_eq!(get("standard_fail"), "#2,#7");
    assert_eq!(get("kill"), "4/5");
    assert_eq!(get("missed"), "c4");
    let report = report_markdown(&[row]);
    assert!(report.contains("| `m` | 標準 | 🔴 量具壞了 | ⚪ 產品未量測 |"));
    assert_eq!(parse_result_line("other output"), None);
}

#[test]
fn generator_needs_a_standard_part_and_golden_assertions() {
    let src = "pub mod standard {}\npub mod challenge_a {\n}\npub mod challenge_b {}\npub mod helper {}\n// @msa-blind: f_slice\n";
    let (challenges, blind) = scan_parts(src);
    assert_eq!(challenges, vec!["challenge_a".to_string(), "challenge_b".to_string()]);
    assert_eq!(blind.as_deref(), Some("f_slice"));
    let lines = vec!["let x = 1;".to_string(), "f(x) == Some(2)".to_string()];
    let with_brick = match render_msa_test("m", src, &lines, true) {
        Some(t) => t,
        None => panic!("有標準件與斷言時必須產生"),
    };
    assert!(with_brick.contains("fn gauge_on_challenge_b() -> Vec<usize>"));
    assert!(with_brick.contains("use rust_bricks::m::*;"));
    assert!(with_brick.contains("Some(parts::blind_check(rust_bricks::m::f_slice, 20000))"));
    assert!(with_brick.contains("tier: RiskTier::Undeclared,"));
    let without = render_msa_test("m", src, &lines, false).unwrap_or_default();
    assert!(!without.contains("rust_bricks::m::") && without.contains("brick_fails: None"));
    assert_eq!(render_msa_test("m", "pub mod challenge_a {}", &lines, true), None);
    assert_eq!(render_msa_test("m", src, &["let x = 1;".to_string()], true), None);
}

#[test]
fn risk_tier_follows_escape_distance_station_cost_and_regulation() {
    assert_eq!(risk_tier("// @msa-escape: next-station\n// @msa-station-cost: low"), RiskTier::Relaxed);
    assert_eq!(risk_tier("// @msa-escape: next-station\n// @msa-station-cost: high"), RiskTier::Standard);
    assert_eq!(risk_tier("// @msa-escape: multi-station\n// @msa-station-cost: low"), RiskTier::Standard);
    assert_eq!(risk_tier("// @msa-escape: multi-station\n// @msa-station-cost: high"), RiskTier::Strict);
    assert_eq!(risk_tier("// @msa-escape: Customer"), RiskTier::Strict);
    assert_eq!(risk_tier("// @msa-escape: next-station\n// @msa-regulatory: FCC Part 15 輻射上限"), RiskTier::Regulatory);
    assert_eq!(risk_tier("// @msa-escape: someday"), RiskTier::Undeclared);
    assert_eq!(risk_tier(""), RiskTier::Undeclared);
    assert_eq!(risk_tier("// @msa-regulatory:"), RiskTier::Undeclared);
}

#[test]
fn the_same_gauge_is_judged_by_its_risk_tier() {
    // 抓出 8/10、容差比 20%：放寬可過、標準要簽核、收緊要重做、法規沒有簽核區
    let mut x = input(true, vec![], 8, 10, None, None, None);
    for (tier, want) in [(RiskTier::Relaxed, GaugeVerdict::Pass), (RiskTier::Standard, GaugeVerdict::SignOff), (RiskTier::Strict, GaugeVerdict::Redo), (RiskTier::Undeclared, GaugeVerdict::Redo)] {
        x.tier = tier;
        assert_eq!(gauge_verdict(&x), want, "{:?}", tier);
    }
    let mut y = input(true, vec![], 10, 10, Some(4.0), None, None);
    y.tier = RiskTier::Regulatory;
    assert_eq!(gauge_verdict(&y), GaugeVerdict::Pass);
    y.tolerance_ratio_pct = Some(5.0);
    assert_eq!(gauge_verdict(&y), GaugeVerdict::Redo);
    let mut z = input(true, vec![], 9, 10, Some(1.0), None, None);
    z.tier = RiskTier::Strict;
    assert_eq!(gauge_verdict(&z), GaugeVerdict::SignOff);
    z.tier = RiskTier::Regulatory;
    assert_eq!(gauge_verdict(&z), GaugeVerdict::Redo);
    assert!(thresholds(RiskTier::Strict).blind_samples > thresholds(RiskTier::Standard).blind_samples);
}
