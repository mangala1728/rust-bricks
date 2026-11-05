// 本檔由 cargo run -- msa-gen 依 msa/canfd_crc.rs 與 specs/canfd_crc.md 的 ```golden 區塊自動產生，請勿手改。
#![allow(unused)]
#[path = "../msa/canfd_crc.rs"]
mod parts;
use rust_bricks::msa::*;

fn gauge_on_standard() -> Vec<usize> {
    use parts::standard::*;
    let mut fails = Vec::new();
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(b"123456789") == Some(0x04F03))).unwrap_or(false) { fails.push(1); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0x80]) == Some(0x0587B))).unwrap_or(false) { fails.push(2); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0x12, 0x34]) == Some(0x1ACDB))).unwrap_or(false) { fails.push(3); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0xFF]) == Some(0x19052))).unwrap_or(false) { fails.push(4); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[]) == Some(0))).unwrap_or(false) { fails.push(5); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0, 1) == Some(0x1685B))).unwrap_or(false) { fails.push(6); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0x10000, 0) == Some(0x1685B))).unwrap_or(false) { fails.push(7); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0x10000, 1) == Some(0))).unwrap_or(false) { fails.push(8); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(1, 0) == Some(2))).unwrap_or(false) { fails.push(9); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0u8; 16]) == Some(0))).unwrap_or(false) { fails.push(10); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0u8; 17]).is_none())).unwrap_or(false) { fails.push(11); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0x20000, 0).is_none())).unwrap_or(false) { fails.push(12); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0, 2).is_none())).unwrap_or(false) { fails.push(13); }
    fails
}

fn gauge_on_challenge_no_augment() -> Vec<usize> {
    use parts::challenge_no_augment::*;
    let mut fails = Vec::new();
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(b"123456789") == Some(0x04F03))).unwrap_or(false) { fails.push(1); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0x80]) == Some(0x0587B))).unwrap_or(false) { fails.push(2); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0x12, 0x34]) == Some(0x1ACDB))).unwrap_or(false) { fails.push(3); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0xFF]) == Some(0x19052))).unwrap_or(false) { fails.push(4); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[]) == Some(0))).unwrap_or(false) { fails.push(5); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0, 1) == Some(0x1685B))).unwrap_or(false) { fails.push(6); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0x10000, 0) == Some(0x1685B))).unwrap_or(false) { fails.push(7); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0x10000, 1) == Some(0))).unwrap_or(false) { fails.push(8); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(1, 0) == Some(2))).unwrap_or(false) { fails.push(9); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0u8; 16]) == Some(0))).unwrap_or(false) { fails.push(10); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0u8; 17]).is_none())).unwrap_or(false) { fails.push(11); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0x20000, 0).is_none())).unwrap_or(false) { fails.push(12); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0, 2).is_none())).unwrap_or(false) { fails.push(13); }
    fails
}

fn gauge_on_challenge_iso_frame_init() -> Vec<usize> {
    use parts::challenge_iso_frame_init::*;
    let mut fails = Vec::new();
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(b"123456789") == Some(0x04F03))).unwrap_or(false) { fails.push(1); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0x80]) == Some(0x0587B))).unwrap_or(false) { fails.push(2); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0x12, 0x34]) == Some(0x1ACDB))).unwrap_or(false) { fails.push(3); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0xFF]) == Some(0x19052))).unwrap_or(false) { fails.push(4); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[]) == Some(0))).unwrap_or(false) { fails.push(5); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0, 1) == Some(0x1685B))).unwrap_or(false) { fails.push(6); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0x10000, 0) == Some(0x1685B))).unwrap_or(false) { fails.push(7); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0x10000, 1) == Some(0))).unwrap_or(false) { fails.push(8); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(1, 0) == Some(2))).unwrap_or(false) { fails.push(9); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0u8; 16]) == Some(0))).unwrap_or(false) { fails.push(10); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0u8; 17]).is_none())).unwrap_or(false) { fails.push(11); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0x20000, 0).is_none())).unwrap_or(false) { fails.push(12); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0, 2).is_none())).unwrap_or(false) { fails.push(13); }
    fails
}

fn gauge_on_challenge_wrong_poly() -> Vec<usize> {
    use parts::challenge_wrong_poly::*;
    let mut fails = Vec::new();
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(b"123456789") == Some(0x04F03))).unwrap_or(false) { fails.push(1); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0x80]) == Some(0x0587B))).unwrap_or(false) { fails.push(2); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0x12, 0x34]) == Some(0x1ACDB))).unwrap_or(false) { fails.push(3); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0xFF]) == Some(0x19052))).unwrap_or(false) { fails.push(4); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[]) == Some(0))).unwrap_or(false) { fails.push(5); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0, 1) == Some(0x1685B))).unwrap_or(false) { fails.push(6); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0x10000, 0) == Some(0x1685B))).unwrap_or(false) { fails.push(7); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0x10000, 1) == Some(0))).unwrap_or(false) { fails.push(8); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(1, 0) == Some(2))).unwrap_or(false) { fails.push(9); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0u8; 16]) == Some(0))).unwrap_or(false) { fails.push(10); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0u8; 17]).is_none())).unwrap_or(false) { fails.push(11); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0x20000, 0).is_none())).unwrap_or(false) { fails.push(12); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0, 2).is_none())).unwrap_or(false) { fails.push(13); }
    fails
}

fn gauge_on_challenge_lsb_first() -> Vec<usize> {
    use parts::challenge_lsb_first::*;
    let mut fails = Vec::new();
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(b"123456789") == Some(0x04F03))).unwrap_or(false) { fails.push(1); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0x80]) == Some(0x0587B))).unwrap_or(false) { fails.push(2); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0x12, 0x34]) == Some(0x1ACDB))).unwrap_or(false) { fails.push(3); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0xFF]) == Some(0x19052))).unwrap_or(false) { fails.push(4); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[]) == Some(0))).unwrap_or(false) { fails.push(5); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0, 1) == Some(0x1685B))).unwrap_or(false) { fails.push(6); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0x10000, 0) == Some(0x1685B))).unwrap_or(false) { fails.push(7); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0x10000, 1) == Some(0))).unwrap_or(false) { fails.push(8); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(1, 0) == Some(2))).unwrap_or(false) { fails.push(9); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0u8; 16]) == Some(0))).unwrap_or(false) { fails.push(10); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0u8; 17]).is_none())).unwrap_or(false) { fails.push(11); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0x20000, 0).is_none())).unwrap_or(false) { fails.push(12); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0, 2).is_none())).unwrap_or(false) { fails.push(13); }
    fails
}

fn gauge_on_challenge_len_64() -> Vec<usize> {
    use parts::challenge_len_64::*;
    let mut fails = Vec::new();
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(b"123456789") == Some(0x04F03))).unwrap_or(false) { fails.push(1); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0x80]) == Some(0x0587B))).unwrap_or(false) { fails.push(2); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0x12, 0x34]) == Some(0x1ACDB))).unwrap_or(false) { fails.push(3); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0xFF]) == Some(0x19052))).unwrap_or(false) { fails.push(4); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[]) == Some(0))).unwrap_or(false) { fails.push(5); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0, 1) == Some(0x1685B))).unwrap_or(false) { fails.push(6); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0x10000, 0) == Some(0x1685B))).unwrap_or(false) { fails.push(7); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0x10000, 1) == Some(0))).unwrap_or(false) { fails.push(8); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(1, 0) == Some(2))).unwrap_or(false) { fails.push(9); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0u8; 16]) == Some(0))).unwrap_or(false) { fails.push(10); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_slice(&[0u8; 17]).is_none())).unwrap_or(false) { fails.push(11); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0x20000, 0).is_none())).unwrap_or(false) { fails.push(12); }
    if !std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| canfd_crc17_bit(0, 2).is_none())).unwrap_or(false) { fails.push(13); }
    fails
}

#[test]
fn msa_canfd_crc() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let input = MsaInput {
        module: "canfd_crc".to_string(),
        golden_total: 13,
        reference_ok: parts::standard_parts_ok(),
        standard_fails: gauge_on_standard(),
        challenges: vec![("challenge_no_augment".to_string(), gauge_on_challenge_no_augment()), ("challenge_iso_frame_init".to_string(), gauge_on_challenge_iso_frame_init()), ("challenge_wrong_poly".to_string(), gauge_on_challenge_wrong_poly()), ("challenge_lsb_first".to_string(), gauge_on_challenge_lsb_first()), ("challenge_len_64".to_string(), gauge_on_challenge_len_64())],
        tolerance_ratio_pct: None,
        brick_fails: None,
        blind: None,
    };
    std::panic::set_hook(hook);
    let line = result_line(&input);
    println!("{}", line);
    let gauge = gauge_verdict(&input);
    assert!(gauge != GaugeVerdict::Broken && gauge != GaugeVerdict::Redo, "量具不可信：{}", line);
    assert!(product_verdict(&input) != ProductVerdict::Reject, "產品拒收：{}", line);
}
