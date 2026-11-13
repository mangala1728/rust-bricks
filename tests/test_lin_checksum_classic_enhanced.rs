// ## Map
// Role: Verify LIN checksum implementation against golden vectors
// Inputs: none (uses public API)
// Outputs: test results (assertions)
// Invariants: zero external dependencies, pure Rust test harness

use rust_bricks::lin_checksum_classic_enhanced::*;

/// Classic checksum – payload only.
#[test]
fn classic_checksum_example() {
    let pid = 0x00;
    let data = &[0x01, 0x02, 0x03];
    let result = lin_compute_checksum(pid, data, false);
    assert_eq!(result, Some(0xF9));
}

/// Enhanced checksum – PID included, non‑diagnostic frame.
#[test]
fn enhanced_checksum_example() {
    let pid = 0x22;
    let data = &[0x4A, 0x55, 0x93, 0xE5];
    let result = lin_compute_checksum(pid, data, true);
    assert_eq!(result, Some(0xC4));
}

/// Diagnostic frame forces Classic rule even when `enhanced` is true.
#[test]
fn diagnostic_forces_classic() {
    let pid = 0x3C;
    let data = &[0x01, 0x02];
    let result = lin_compute_checksum(pid, data, true);
    assert_eq!(result, Some(0xFC));
}

/// Empty payload – should be rejected.
#[test]
fn reject_empty_payload() {
    let result = lin_compute_checksum(0x00, &[], false);
    assert!(result.is_none());
}

/// Over‑length payload – should be rejected.
#[test]
fn reject_oversized_payload() {
    let data = &[1, 2, 3, 4, 5, 6, 7, 8, 9];
    let result = lin_compute_checksum(0x00, data, false);
    assert!(result.is_none());
}
