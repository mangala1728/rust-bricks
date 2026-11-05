// 本檔由 cargo run -- msa-gen 依 msa/log_lifecycle_dag.rs 與 specs/log_lifecycle_dag.md 的 ```golden 區塊自動產生，請勿手改。
#![allow(unused)]
#[path = "../msa/log_lifecycle_dag.rs"]
mod parts;
use rust_bricks::msa::*;

fn gauge_on_standard() -> Vec<usize> {
    use parts::standard::*;
    let mut fails = Vec::new();
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(0, &[1, 2, 2], 8) == Some((2, 0)))).unwrap_or(false) { fails.push(1); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(2, &[1, 2, 2], 8) == Some((2, 0)))).unwrap_or(false) { fails.push(2); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(0, &[1, 0], 4) == Some((0, 1)))).unwrap_or(false) { fails.push(3); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 10, 6, 10, 0) == Some(2))).unwrap_or(false) { fails.push(4); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(6, 10, 6, 10, 0) == Some(0))).unwrap_or(false) { fails.push(5); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(7, 10, 6, 10, 0) == Some(0))).unwrap_or(false) { fails.push(6); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 99, 6, 10, 0) == Some(0))).unwrap_or(false) { fails.push(7); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 10, 6, 10, 1) == Some(1))).unwrap_or(false) { fails.push(8); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(3, &[0, 1, 2], 8).is_none())).unwrap_or(false) { fails.push(9); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(0, &[0, 1, 2], 0).is_none())).unwrap_or(false) { fails.push(10); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(0, 10, 6, 10, 0).is_none())).unwrap_or(false) { fails.push(11); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 10, 6, 10, 2).is_none())).unwrap_or(false) { fails.push(12); }
    fails
}

fn gauge_on_challenge_single_hop_only() -> Vec<usize> {
    use parts::challenge_single_hop_only::*;
    let mut fails = Vec::new();
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(0, &[1, 2, 2], 8) == Some((2, 0)))).unwrap_or(false) { fails.push(1); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(2, &[1, 2, 2], 8) == Some((2, 0)))).unwrap_or(false) { fails.push(2); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(0, &[1, 0], 4) == Some((0, 1)))).unwrap_or(false) { fails.push(3); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 10, 6, 10, 0) == Some(2))).unwrap_or(false) { fails.push(4); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(6, 10, 6, 10, 0) == Some(0))).unwrap_or(false) { fails.push(5); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(7, 10, 6, 10, 0) == Some(0))).unwrap_or(false) { fails.push(6); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 99, 6, 10, 0) == Some(0))).unwrap_or(false) { fails.push(7); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 10, 6, 10, 1) == Some(1))).unwrap_or(false) { fails.push(8); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(3, &[0, 1, 2], 8).is_none())).unwrap_or(false) { fails.push(9); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(0, &[0, 1, 2], 0).is_none())).unwrap_or(false) { fails.push(10); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(0, 10, 6, 10, 0).is_none())).unwrap_or(false) { fails.push(11); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 10, 6, 10, 2).is_none())).unwrap_or(false) { fails.push(12); }
    fails
}

fn gauge_on_challenge_pivot_kills_itself() -> Vec<usize> {
    use parts::challenge_pivot_kills_itself::*;
    let mut fails = Vec::new();
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(0, &[1, 2, 2], 8) == Some((2, 0)))).unwrap_or(false) { fails.push(1); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(2, &[1, 2, 2], 8) == Some((2, 0)))).unwrap_or(false) { fails.push(2); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(0, &[1, 0], 4) == Some((0, 1)))).unwrap_or(false) { fails.push(3); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 10, 6, 10, 0) == Some(2))).unwrap_or(false) { fails.push(4); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(6, 10, 6, 10, 0) == Some(0))).unwrap_or(false) { fails.push(5); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(7, 10, 6, 10, 0) == Some(0))).unwrap_or(false) { fails.push(6); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 99, 6, 10, 0) == Some(0))).unwrap_or(false) { fails.push(7); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 10, 6, 10, 1) == Some(1))).unwrap_or(false) { fails.push(8); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(3, &[0, 1, 2], 8).is_none())).unwrap_or(false) { fails.push(9); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(0, &[0, 1, 2], 0).is_none())).unwrap_or(false) { fails.push(10); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(0, 10, 6, 10, 0).is_none())).unwrap_or(false) { fails.push(11); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 10, 6, 10, 2).is_none())).unwrap_or(false) { fails.push(12); }
    fails
}

fn gauge_on_challenge_ignore_scope_on_pivot() -> Vec<usize> {
    use parts::challenge_ignore_scope_on_pivot::*;
    let mut fails = Vec::new();
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(0, &[1, 2, 2], 8) == Some((2, 0)))).unwrap_or(false) { fails.push(1); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(2, &[1, 2, 2], 8) == Some((2, 0)))).unwrap_or(false) { fails.push(2); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(0, &[1, 0], 4) == Some((0, 1)))).unwrap_or(false) { fails.push(3); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 10, 6, 10, 0) == Some(2))).unwrap_or(false) { fails.push(4); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(6, 10, 6, 10, 0) == Some(0))).unwrap_or(false) { fails.push(5); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(7, 10, 6, 10, 0) == Some(0))).unwrap_or(false) { fails.push(6); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 99, 6, 10, 0) == Some(0))).unwrap_or(false) { fails.push(7); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 10, 6, 10, 1) == Some(1))).unwrap_or(false) { fails.push(8); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(3, &[0, 1, 2], 8).is_none())).unwrap_or(false) { fails.push(9); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(0, &[0, 1, 2], 0).is_none())).unwrap_or(false) { fails.push(10); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(0, 10, 6, 10, 0).is_none())).unwrap_or(false) { fails.push(11); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 10, 6, 10, 2).is_none())).unwrap_or(false) { fails.push(12); }
    fails
}

fn gauge_on_challenge_silent_cycle() -> Vec<usize> {
    use parts::challenge_silent_cycle::*;
    let mut fails = Vec::new();
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(0, &[1, 2, 2], 8) == Some((2, 0)))).unwrap_or(false) { fails.push(1); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(2, &[1, 2, 2], 8) == Some((2, 0)))).unwrap_or(false) { fails.push(2); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(0, &[1, 0], 4) == Some((0, 1)))).unwrap_or(false) { fails.push(3); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 10, 6, 10, 0) == Some(2))).unwrap_or(false) { fails.push(4); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(6, 10, 6, 10, 0) == Some(0))).unwrap_or(false) { fails.push(5); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(7, 10, 6, 10, 0) == Some(0))).unwrap_or(false) { fails.push(6); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 99, 6, 10, 0) == Some(0))).unwrap_or(false) { fails.push(7); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 10, 6, 10, 1) == Some(1))).unwrap_or(false) { fails.push(8); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(3, &[0, 1, 2], 8).is_none())).unwrap_or(false) { fails.push(9); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(0, &[0, 1, 2], 0).is_none())).unwrap_or(false) { fails.push(10); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(0, 10, 6, 10, 0).is_none())).unwrap_or(false) { fails.push(11); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 10, 6, 10, 2).is_none())).unwrap_or(false) { fails.push(12); }
    fails
}

fn gauge_on_challenge_missing_domain_guard() -> Vec<usize> {
    use parts::challenge_missing_domain_guard::*;
    let mut fails = Vec::new();
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(0, &[1, 2, 2], 8) == Some((2, 0)))).unwrap_or(false) { fails.push(1); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(2, &[1, 2, 2], 8) == Some((2, 0)))).unwrap_or(false) { fails.push(2); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(0, &[1, 0], 4) == Some((0, 1)))).unwrap_or(false) { fails.push(3); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 10, 6, 10, 0) == Some(2))).unwrap_or(false) { fails.push(4); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(6, 10, 6, 10, 0) == Some(0))).unwrap_or(false) { fails.push(5); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(7, 10, 6, 10, 0) == Some(0))).unwrap_or(false) { fails.push(6); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 99, 6, 10, 0) == Some(0))).unwrap_or(false) { fails.push(7); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 10, 6, 10, 1) == Some(1))).unwrap_or(false) { fails.push(8); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(3, &[0, 1, 2], 8).is_none())).unwrap_or(false) { fails.push(9); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(0, &[0, 1, 2], 0).is_none())).unwrap_or(false) { fails.push(10); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(0, 10, 6, 10, 0).is_none())).unwrap_or(false) { fails.push(11); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 10, 6, 10, 2).is_none())).unwrap_or(false) { fails.push(12); }
    fails
}

fn gauge_on_brick() -> Vec<usize> {
    use rust_bricks::log_lifecycle_dag::*;
    let mut fails = Vec::new();
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(0, &[1, 2, 2], 8) == Some((2, 0)))).unwrap_or(false) { fails.push(1); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(2, &[1, 2, 2], 8) == Some((2, 0)))).unwrap_or(false) { fails.push(2); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(0, &[1, 0], 4) == Some((0, 1)))).unwrap_or(false) { fails.push(3); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 10, 6, 10, 0) == Some(2))).unwrap_or(false) { fails.push(4); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(6, 10, 6, 10, 0) == Some(0))).unwrap_or(false) { fails.push(5); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(7, 10, 6, 10, 0) == Some(0))).unwrap_or(false) { fails.push(6); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 99, 6, 10, 0) == Some(0))).unwrap_or(false) { fails.push(7); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 10, 6, 10, 1) == Some(1))).unwrap_or(false) { fails.push(8); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(3, &[0, 1, 2], 8).is_none())).unwrap_or(false) { fails.push(9); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| resolve_terminal_node(0, &[0, 1, 2], 0).is_none())).unwrap_or(false) { fails.push(10); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(0, 10, 6, 10, 0).is_none())).unwrap_or(false) { fails.push(11); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| classify_epoch_pivot(2, 10, 6, 10, 2).is_none())).unwrap_or(false) { fails.push(12); }
    fails
}

#[test]
fn msa_log_lifecycle_dag() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let input = MsaInput {
        module: "log_lifecycle_dag".to_string(),
        tier: RiskTier::Strict,
        golden_total: 12,
        reference_ok: parts::standard_parts_ok(),
        standard_fails: gauge_on_standard(),
        challenges: vec![("challenge_single_hop_only".to_string(), gauge_on_challenge_single_hop_only()), ("challenge_pivot_kills_itself".to_string(), gauge_on_challenge_pivot_kills_itself()), ("challenge_ignore_scope_on_pivot".to_string(), gauge_on_challenge_ignore_scope_on_pivot()), ("challenge_silent_cycle".to_string(), gauge_on_challenge_silent_cycle()), ("challenge_missing_domain_guard".to_string(), gauge_on_challenge_missing_domain_guard())],
        tolerance_ratio_pct: None,
        brick_fails: Some(gauge_on_brick()),
        blind: Some(parts::blind_check(rust_bricks::log_lifecycle_dag::classify_epoch_pivot, 20000)),
    };
    std::panic::set_hook(hook);
    let line = result_line(&input);
    println!("{}", line);
    let gauge = gauge_verdict(&input);
    assert!(gauge != GaugeVerdict::Broken && gauge != GaugeVerdict::Redo, "量具不可信：{}", line);
    assert!(product_verdict(&input) != ProductVerdict::Reject, "產品拒收：{}", line);
}
