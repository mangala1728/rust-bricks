use rust_bricks::stat_gate_core::*;

#[test]
fn test_wilson_interval_golden_and_edges() {
    // STAT-GATE-SPEC §5.1 golden vector: 27/30 = 90% -> Wilson 95% CI [0.7438, 0.9654]
    let (lo, hi) = stat_wilson_interval_95(27, 30).expect("valid 27/30");
    assert!((lo - 0.743_792).abs() < 1e-4);
    assert!((hi - 0.965_405).abs() < 1e-4);

    // Lower/upper edge k=0 and k=n must NOT collapse to 0 width (anti-Wald)
    let (lo0, hi0) = stat_wilson_interval_95(0, 10).expect("valid 0/10");
    assert!((lo0 - 0.0).abs() < 1e-12);
    assert!((hi0 - 0.277_533).abs() < 1e-4);

    let (lon, hin) = stat_wilson_interval_95(10, 10).expect("valid 10/10");
    assert!((lon - 0.722_467).abs() < 1e-4);
    assert!((hin - 1.0).abs() < 1e-12);

    // Invalid boundaries rejected
    assert!(stat_wilson_interval_95(0, 0).is_none());
    assert!(stat_wilson_interval_95(11, 10).is_none());
}

#[test]
fn test_rule_of_three_and_bayesian_ppv_golden() {
    // MSA §8 / STAT-GATE C01: 0/54 with c=1.0 -> 3/54 = 0.055555...
    let ub = stat_rule_of_three_upper_bound(54, 1.0).expect("valid");
    assert!((ub - 0.055_556).abs() < 1e-5);
    let ub_adj = stat_rule_of_three_upper_bound(30, 0.5).expect("valid");
    assert!((ub_adj - 0.2).abs() < 1e-12);
    assert!(stat_rule_of_three_upper_bound(0, 1.0).is_none());
    assert!(stat_rule_of_three_upper_bound(10, 0.0).is_none());

    // STAT-GATE §5.1 golden: base=5%, recall=95%, fpr=10% -> 0.0475 / 0.1425 = 1/3
    let ppv = stat_bayesian_ppv(0.05, 0.95, 0.10).expect("valid");
    assert!((ppv - (1.0 / 3.0)).abs() < 1e-9);
    assert!(stat_bayesian_ppv(-0.01, 0.95, 0.10).is_none());
    assert!(stat_bayesian_ppv(f64::NAN, 0.95, 0.10).is_none());
}

#[test]
fn test_beta_cdf_closed_form_and_bayesian_upgrade() {
    // STAT-GATE §8.1 closed-form golden values (tol 1e-9):
    // Beta(6, 1) CDF at 0.8 = 0.8^6 = 0.262144
    let b61 = stat_beta_cdf_regularized(0.8, 6.0, 1.0).expect("Beta(6,1)");
    assert!((b61 - 0.262_144).abs() < 1e-9);

    // Beta(7, 2) CDF at 0.8 = 0.8^7 * 2.4 = 0.50331648
    let b72 = stat_beta_cdf_regularized(0.8, 7.0, 2.0).expect("Beta(7,2)");
    assert!((b72 - 0.503_316_48).abs() < 1e-9);

    // STAT-GATE §5.2 table (p0=0.8, confidence=0.9):
    // 6 consecutive successes -> Jeffreys passes, Uniform & Beta(2,2) fail
    let (pass6, u6, j6, b6) = stat_bayesian_trial_upgrade(6, 0, 0.8, 0.9).expect("s=6");
    assert!(!pass6);
    assert!(j6 >= 0.9);
    assert!(u6 < 0.9);
    assert!(b6 < 0.9);

    // 10 consecutive successes -> Uniform & Jeffreys pass, Beta(2,2) fails
    let (pass10, u10, j10, b10) = stat_bayesian_trial_upgrade(10, 0, 0.8, 0.9).expect("s=10");
    assert!(!pass10);
    assert!(u10 >= 0.9);
    assert!(j10 >= 0.9);
    assert!(b10 < 0.9);

    // 14 fails conservative Beta(2,2), 15 passes all three priors
    let (pass14, _, _, b14) = stat_bayesian_trial_upgrade(14, 0, 0.8, 0.9).expect("s=14");
    assert!(!pass14);
    assert!(b14 < 0.9);

    let (pass15, u15, j15, b15) = stat_bayesian_trial_upgrade(15, 0, 0.8, 0.9).expect("s=15");
    assert!(pass15);
    assert!(u15 >= 0.9);
    assert!(j15 >= 0.9);
    assert!(b15 >= 0.9);

    assert!(stat_beta_cdf_regularized(-0.1, 2.0, 2.0).is_none());
    assert!(stat_beta_cdf_regularized(0.5, 0.0, 2.0).is_none());
}

#[test]
fn test_holm_bonferroni_golden() {
    // STAT-GATE §8.1 exact golden: p = [0.01, 0.03, 0.04], alpha = 0.05 -> [true, false, false]
    let r = stat_holm_bonferroni_reject(&[0.01, 0.03, 0.04], 0.05).expect("valid");
    assert_eq!(r, vec![true, false, false]);

    // Unordered input preserves original positions: 0.04, 0.01, 0.02 -> 0.01<=0.0167(T), 0.02<=0.025(T), 0.04<=0.05(T)
    let r2 = stat_holm_bonferroni_reject(&[0.04, 0.01, 0.02], 0.05).expect("valid");
    assert_eq!(r2, vec![true, true, true]);

    assert!(stat_holm_bonferroni_reject(&[], 0.05).is_none());
    assert!(stat_holm_bonferroni_reject(&[1.2], 0.05).is_none());
}

#[test]
fn test_median_mad_robust_z_power_and_p_chart() {
    // Median & MAD: [1, 2, 2, 3, 4, 7, 9] -> median=3.0, abs devs=[0, 1, 1, 1, 2, 4, 6] -> MAD=1.0
    let (med, mad) = stat_median_and_mad(&[9.0, 1.0, 4.0, 2.0, 7.0, 2.0, 3.0]).expect("valid");
    assert!((med - 3.0).abs() < 1e-12);
    assert!((mad - 1.0).abs() < 1e-12);

    // OUT-v1 robust z = (9.0 - 3.0) / 1.4826 = 4.0469445... (> 3.5 outlier candidate)
    let rz = stat_robust_z_score(9.0, med, mad).expect("valid");
    assert!((rz - 4.046_944_557).abs() < 1e-6);
    assert!(stat_robust_z_score(9.0, med, 0.0).is_none());

    // STAT-GATE §8.1 power golden: alpha'=0.05/3 (z=2.39398), power=0.8 (z=0.84162), delta=0.1
    // sd=0.3 -> 95, sd=0.4 -> 168
    let n95 = stat_required_paired_sample_size(0.3, 0.1, 2.393_98, 0.841_62).expect("sd=0.3");
    assert_eq!(n95, 95);
    let n168 = stat_required_paired_sample_size(0.4, 0.1, 2.393_98, 0.841_62).expect("sd=0.4");
    assert_eq!(n168, 168);

    // Shewhart p-chart: p_bar=0.05, n=20, k=4.0 -> LCL must clamp at 0.0 (not negative!)
    let (lcl, ucl) = stat_p_chart_limits(0.05, 20, 4.0).expect("p-chart");
    assert!((lcl - 0.0).abs() < 1e-12);
    assert!((ucl - 0.244_935_887).abs() < 1e-6);
    assert!(stat_p_chart_limits(1.2, 20, 3.0).is_none());
}

#[test]
fn test_permutation_bootstrap_wording_and_t4_golden() {
    // Stratified paired permutation: clear positive diffs -> small p-value, deterministic on seed
    let diffs = [0.25, 0.30, 0.20, 0.35, 0.28, 0.22, 0.31, 0.27];
    let strata = [1, 1, 1, 1, 2, 2, 2, 2];
    let p1 = stat_stratified_permutation_p(&diffs, &strata, 1000, 42).expect("perm1");
    let p2 = stat_stratified_permutation_p(&diffs, &strata, 1000, 42).expect("perm2");
    assert!((p1 - p2).abs() < 1e-12);
    assert!(p1 < 0.02);
    assert!(stat_stratified_permutation_p(&diffs, &[1], 1000, 42).is_none());

    // Paired bootstrap CI at Holm step 1 (98.3% => alpha = 0.017)
    let (mean, lo, hi) = stat_paired_bootstrap_ci(&diffs, 0.017, 1000, 42).expect("boot");
    assert!((mean - 0.2725).abs() < 1e-9);
    assert!(lo > 0.20 && hi < 0.35 && lo < mean && mean < hi);

    // Wording level checks (§6 & §9 non-claim exemption)
    assert!(stat_check_wording_level(0, "因為樣本不足，所以只列描述").is_empty());
    assert!(!stat_check_wording_level(0, "因為樣本數夠多，本系統已獲實證可靠").is_empty());
    let hits0 = stat_check_wording_level(0, "此修法改善了良率並保證有效");
    assert!(hits0.contains(&"改善") && hits0.contains(&"保證") && hits0.contains(&"有效"));
    let hits1 = stat_check_wording_level(1, "新 hook 與較低假完成率伴隨出現");
    assert!(hits1.is_empty());

    // T4 trigger detection (§3): version tags like v2 and single decimal 1.5 must not false-trigger
    assert!(stat_detect_t4_trigger("裝上防呆後假完成率從 25% 降到 5%"));
    assert!(!stat_detect_t4_trigger("npm test -> 12/12"));
    assert!(!stat_detect_t4_trigger("FPY v2 提升至 95%"));
    assert!(!stat_detect_t4_trigger("耗時改善 1.5 秒"));

    // Tiny non-zero diffs must not collapse to p=1.0; large alpha+beta converges cleanly
    let tiny = [1.0e-13; 8];
    let strata_t = [1, 1, 1, 1, 2, 2, 2, 2];
    let pt = stat_stratified_permutation_p(&tiny, &strata_t, 500, 42).expect("tiny");
    assert!(pt < 0.05);
    assert!(stat_beta_cdf_regularized(0.8, 50_000.0, 12_500.0).is_some());
}
