// ## Map
// Role: media/subtitle_cps_vad — 影視字幕 CPS 定點語速計算、Netflix 繁中三段分級與外部基準時間漂移驗收微積木。
// Inputs: readable_chars: u32, duration_ms: u32, cps_tenths: u32, children_mode: u8, cand/ref timestamps (u32)。
// Outputs: Option<u32>, Option<u8>, Option<(u8, u32)>。
// Invariants: 零外部依賴、第一行領域邊界守衛、單一函式 <= 60 行、無遞迴。

/// @outputs: 2
/// @domain: readable_chars [0..5000]
/// @domain: duration_ms [1..600000]
pub fn subtitle_cps_tenths(readable_chars: u32, duration_ms: u32) -> Option<u32> {
    if readable_chars > 5000 || duration_ms < 1 || duration_ms > 600_000 {
        return None;
    }
    let numer = (readable_chars as u64) * 10_000 + (duration_ms as u64 / 2);
    let tenths = (numer / (duration_ms as u64)) as u32;
    Some(tenths)
}

/// @outputs: 4
/// @domain: cps_tenths [0..10000]
/// @domain: children_mode [0..1]
pub fn subtitle_cps_band(cps_tenths: u32, children_mode: u8) -> Option<u8> {
    if cps_tenths > 10_000 || children_mode > 1 {
        return None;
    }
    if cps_tenths > 120 {
        return Some(2);
    }
    if children_mode == 1 && cps_tenths > 70 {
        return Some(1);
    }
    if children_mode == 0 && cps_tenths > 90 {
        return Some(1);
    }
    Some(0)
}

/// @outputs: 2
/// @domain: cand_start_ms [0..86400000]
/// @domain: cand_end_ms [1..86400000]
/// @domain: ref_start_ms [0..86400000]
/// @domain: ref_end_ms [1..86400000]
/// @domain: tol_ms [1..5000]
pub fn subtitle_vad_drift_ok(
    cand_start_ms: u32,
    cand_end_ms: u32,
    ref_start_ms: u32,
    ref_end_ms: u32,
    tol_ms: u32,
) -> Option<(u8, u32)> {
    if cand_start_ms > 86_400_000
        || cand_end_ms < 1
        || cand_end_ms > 86_400_000
        || ref_start_ms > 86_400_000
        || ref_end_ms < 1
        || ref_end_ms > 86_400_000
        || tol_ms < 1
        || tol_ms > 5000
    {
        return None;
    }
    if cand_end_ms <= cand_start_ms || ref_end_ms <= ref_start_ms {
        return None;
    }
    let d_start = cand_start_ms.abs_diff(ref_start_ms);
    let d_end = cand_end_ms.abs_diff(ref_end_ms);
    let max_drift = d_start.max(d_end);
    let ok = if max_drift <= tol_ms { 1 } else { 0 };
    Some((ok, max_drift))
}
