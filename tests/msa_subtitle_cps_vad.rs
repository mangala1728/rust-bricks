// 本檔由 cargo run -- msa-gen 依 msa/subtitle_cps_vad.rs 與 specs/subtitle_cps_vad.md 的 ```golden 區塊自動產生，請勿手改。
#![allow(unused)]
#[path = "../msa/subtitle_cps_vad.rs"]
mod parts;
use rust_bricks::msa::*;

fn gauge_on_standard() -> Vec<usize> {
    use parts::standard::*;
    let mut fails = Vec::new();
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(9, 1000) == Some(90))).unwrap_or(false) { fails.push(1); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(18, 2500) == Some(72))).unwrap_or(false) { fails.push(2); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(90, 0) == Some(0))).unwrap_or(false) { fails.push(3); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(91, 0) == Some(1))).unwrap_or(false) { fails.push(4); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(120, 0) == Some(1))).unwrap_or(false) { fails.push(5); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(121, 0) == Some(2))).unwrap_or(false) { fails.push(6); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(70, 1) == Some(0))).unwrap_or(false) { fails.push(7); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(72, 1) == Some(1))).unwrap_or(false) { fails.push(8); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(1040, 3480, 1000, 3500, 100) == Some((1, 40)))).unwrap_or(false) { fails.push(9); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(1350, 3850, 1000, 3500, 100) == Some((0, 350)))).unwrap_or(false) { fails.push(10); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(10, 0).is_none())).unwrap_or(false) { fails.push(11); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(5001, 1000).is_none())).unwrap_or(false) { fails.push(12); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(100, 2).is_none())).unwrap_or(false) { fails.push(13); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(3500, 1000, 1000, 3500, 100).is_none())).unwrap_or(false) { fails.push(14); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(1000, 3500, 1000, 3500, 0).is_none())).unwrap_or(false) { fails.push(15); }
    fails
}

fn gauge_on_challenge_legacy_conflicting_cps() -> Vec<usize> {
    use parts::challenge_legacy_conflicting_cps::*;
    let mut fails = Vec::new();
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(9, 1000) == Some(90))).unwrap_or(false) { fails.push(1); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(18, 2500) == Some(72))).unwrap_or(false) { fails.push(2); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(90, 0) == Some(0))).unwrap_or(false) { fails.push(3); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(91, 0) == Some(1))).unwrap_or(false) { fails.push(4); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(120, 0) == Some(1))).unwrap_or(false) { fails.push(5); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(121, 0) == Some(2))).unwrap_or(false) { fails.push(6); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(70, 1) == Some(0))).unwrap_or(false) { fails.push(7); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(72, 1) == Some(1))).unwrap_or(false) { fails.push(8); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(1040, 3480, 1000, 3500, 100) == Some((1, 40)))).unwrap_or(false) { fails.push(9); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(1350, 3850, 1000, 3500, 100) == Some((0, 350)))).unwrap_or(false) { fails.push(10); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(10, 0).is_none())).unwrap_or(false) { fails.push(11); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(5001, 1000).is_none())).unwrap_or(false) { fails.push(12); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(100, 2).is_none())).unwrap_or(false) { fails.push(13); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(3500, 1000, 1000, 3500, 100).is_none())).unwrap_or(false) { fails.push(14); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(1000, 3500, 1000, 3500, 0).is_none())).unwrap_or(false) { fails.push(15); }
    fails
}

fn gauge_on_challenge_self_referencing_vad() -> Vec<usize> {
    use parts::challenge_self_referencing_vad::*;
    let mut fails = Vec::new();
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(9, 1000) == Some(90))).unwrap_or(false) { fails.push(1); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(18, 2500) == Some(72))).unwrap_or(false) { fails.push(2); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(90, 0) == Some(0))).unwrap_or(false) { fails.push(3); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(91, 0) == Some(1))).unwrap_or(false) { fails.push(4); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(120, 0) == Some(1))).unwrap_or(false) { fails.push(5); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(121, 0) == Some(2))).unwrap_or(false) { fails.push(6); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(70, 1) == Some(0))).unwrap_or(false) { fails.push(7); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(72, 1) == Some(1))).unwrap_or(false) { fails.push(8); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(1040, 3480, 1000, 3500, 100) == Some((1, 40)))).unwrap_or(false) { fails.push(9); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(1350, 3850, 1000, 3500, 100) == Some((0, 350)))).unwrap_or(false) { fails.push(10); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(10, 0).is_none())).unwrap_or(false) { fails.push(11); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(5001, 1000).is_none())).unwrap_or(false) { fails.push(12); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(100, 2).is_none())).unwrap_or(false) { fails.push(13); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(3500, 1000, 1000, 3500, 100).is_none())).unwrap_or(false) { fails.push(14); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(1000, 3500, 1000, 3500, 0).is_none())).unwrap_or(false) { fails.push(15); }
    fails
}

fn gauge_on_challenge_unscaled_cps() -> Vec<usize> {
    use parts::challenge_unscaled_cps::*;
    let mut fails = Vec::new();
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(9, 1000) == Some(90))).unwrap_or(false) { fails.push(1); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(18, 2500) == Some(72))).unwrap_or(false) { fails.push(2); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(90, 0) == Some(0))).unwrap_or(false) { fails.push(3); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(91, 0) == Some(1))).unwrap_or(false) { fails.push(4); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(120, 0) == Some(1))).unwrap_or(false) { fails.push(5); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(121, 0) == Some(2))).unwrap_or(false) { fails.push(6); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(70, 1) == Some(0))).unwrap_or(false) { fails.push(7); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(72, 1) == Some(1))).unwrap_or(false) { fails.push(8); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(1040, 3480, 1000, 3500, 100) == Some((1, 40)))).unwrap_or(false) { fails.push(9); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(1350, 3850, 1000, 3500, 100) == Some((0, 350)))).unwrap_or(false) { fails.push(10); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(10, 0).is_none())).unwrap_or(false) { fails.push(11); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(5001, 1000).is_none())).unwrap_or(false) { fails.push(12); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(100, 2).is_none())).unwrap_or(false) { fails.push(13); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(3500, 1000, 1000, 3500, 100).is_none())).unwrap_or(false) { fails.push(14); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(1000, 3500, 1000, 3500, 0).is_none())).unwrap_or(false) { fails.push(15); }
    fails
}

fn gauge_on_challenge_off_by_one_band() -> Vec<usize> {
    use parts::challenge_off_by_one_band::*;
    let mut fails = Vec::new();
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(9, 1000) == Some(90))).unwrap_or(false) { fails.push(1); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(18, 2500) == Some(72))).unwrap_or(false) { fails.push(2); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(90, 0) == Some(0))).unwrap_or(false) { fails.push(3); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(91, 0) == Some(1))).unwrap_or(false) { fails.push(4); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(120, 0) == Some(1))).unwrap_or(false) { fails.push(5); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(121, 0) == Some(2))).unwrap_or(false) { fails.push(6); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(70, 1) == Some(0))).unwrap_or(false) { fails.push(7); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(72, 1) == Some(1))).unwrap_or(false) { fails.push(8); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(1040, 3480, 1000, 3500, 100) == Some((1, 40)))).unwrap_or(false) { fails.push(9); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(1350, 3850, 1000, 3500, 100) == Some((0, 350)))).unwrap_or(false) { fails.push(10); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(10, 0).is_none())).unwrap_or(false) { fails.push(11); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(5001, 1000).is_none())).unwrap_or(false) { fails.push(12); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(100, 2).is_none())).unwrap_or(false) { fails.push(13); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(3500, 1000, 1000, 3500, 100).is_none())).unwrap_or(false) { fails.push(14); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(1000, 3500, 1000, 3500, 0).is_none())).unwrap_or(false) { fails.push(15); }
    fails
}

fn gauge_on_challenge_missing_inversion_guard() -> Vec<usize> {
    use parts::challenge_missing_inversion_guard::*;
    let mut fails = Vec::new();
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(9, 1000) == Some(90))).unwrap_or(false) { fails.push(1); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(18, 2500) == Some(72))).unwrap_or(false) { fails.push(2); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(90, 0) == Some(0))).unwrap_or(false) { fails.push(3); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(91, 0) == Some(1))).unwrap_or(false) { fails.push(4); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(120, 0) == Some(1))).unwrap_or(false) { fails.push(5); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(121, 0) == Some(2))).unwrap_or(false) { fails.push(6); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(70, 1) == Some(0))).unwrap_or(false) { fails.push(7); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(72, 1) == Some(1))).unwrap_or(false) { fails.push(8); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(1040, 3480, 1000, 3500, 100) == Some((1, 40)))).unwrap_or(false) { fails.push(9); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(1350, 3850, 1000, 3500, 100) == Some((0, 350)))).unwrap_or(false) { fails.push(10); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(10, 0).is_none())).unwrap_or(false) { fails.push(11); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(5001, 1000).is_none())).unwrap_or(false) { fails.push(12); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(100, 2).is_none())).unwrap_or(false) { fails.push(13); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(3500, 1000, 1000, 3500, 100).is_none())).unwrap_or(false) { fails.push(14); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(1000, 3500, 1000, 3500, 0).is_none())).unwrap_or(false) { fails.push(15); }
    fails
}

fn gauge_on_brick() -> Vec<usize> {
    use rust_bricks::subtitle_cps_vad::*;
    let mut fails = Vec::new();
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(9, 1000) == Some(90))).unwrap_or(false) { fails.push(1); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(18, 2500) == Some(72))).unwrap_or(false) { fails.push(2); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(90, 0) == Some(0))).unwrap_or(false) { fails.push(3); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(91, 0) == Some(1))).unwrap_or(false) { fails.push(4); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(120, 0) == Some(1))).unwrap_or(false) { fails.push(5); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(121, 0) == Some(2))).unwrap_or(false) { fails.push(6); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(70, 1) == Some(0))).unwrap_or(false) { fails.push(7); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(72, 1) == Some(1))).unwrap_or(false) { fails.push(8); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(1040, 3480, 1000, 3500, 100) == Some((1, 40)))).unwrap_or(false) { fails.push(9); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(1350, 3850, 1000, 3500, 100) == Some((0, 350)))).unwrap_or(false) { fails.push(10); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(10, 0).is_none())).unwrap_or(false) { fails.push(11); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_tenths(5001, 1000).is_none())).unwrap_or(false) { fails.push(12); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_cps_band(100, 2).is_none())).unwrap_or(false) { fails.push(13); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(3500, 1000, 1000, 3500, 100).is_none())).unwrap_or(false) { fails.push(14); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| subtitle_vad_drift_ok(1000, 3500, 1000, 3500, 0).is_none())).unwrap_or(false) { fails.push(15); }
    fails
}

#[test]
fn msa_subtitle_cps_vad() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let input = MsaInput {
        module: "subtitle_cps_vad".to_string(),
        tier: RiskTier::Strict,
        golden_total: 15,
        reference_ok: parts::standard_parts_ok(),
        standard_fails: gauge_on_standard(),
        challenges: vec![("challenge_legacy_conflicting_cps".to_string(), gauge_on_challenge_legacy_conflicting_cps()), ("challenge_self_referencing_vad".to_string(), gauge_on_challenge_self_referencing_vad()), ("challenge_unscaled_cps".to_string(), gauge_on_challenge_unscaled_cps()), ("challenge_off_by_one_band".to_string(), gauge_on_challenge_off_by_one_band()), ("challenge_missing_inversion_guard".to_string(), gauge_on_challenge_missing_inversion_guard())],
        tolerance_ratio_pct: None,
        brick_fails: Some(gauge_on_brick()),
        blind: Some(parts::blind_check(rust_bricks::subtitle_cps_vad::subtitle_cps_tenths, 20000)),
    };
    std::panic::set_hook(hook);
    let line = result_line(&input);
    println!("{}", line);
    let gauge = gauge_verdict(&input);
    assert!(gauge != GaugeVerdict::Broken && gauge != GaugeVerdict::Redo, "量具不可信：{}", line);
    assert!(product_verdict(&input) != ProductVerdict::Reject, "產品拒收：{}", line);
}
