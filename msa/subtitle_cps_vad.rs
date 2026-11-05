// ## Map
// Role: msa/subtitle_cps_vad — 影視字幕 CPS 與外部基準 VAD 時間漂移微積木之量測系統分析（MSA）零件。
// Inputs: 字數、毫秒時長、兒童/成人模式、候選與基準起訖時間戳。
// Outputs: 參考實作結果、挑戰件（含舊版 11/7.5 CPS 雙標與自己量自己缺陷）結果、20,000 筆盲測比對。
// Invariants: 獨立於 src/subtitle_cps_vad.rs（不得 use rust_bricks::subtitle_cps_vad）；先過公開標準件才可量產品。
//
// @msa-standard-source: Netflix Timed Text Style Guide (Traditional Chinese) — Adult 9.0 CPS (90 tenths), Children 7.0 CPS (70 tenths), Hard Red-Line 12.0 CPS (120 tenths)
// @msa-blind: subtitle_cps_tenths
// @msa-risk: 字幕超速或音畫不同步直接流向觀眾端與交付驗收
// @msa-escape: customer
// @msa-station-cost: high

pub const STANDARD_PARTS: &[((u32, u32), u32, &str)] = &[
    ((9, 1000), 90, "Netflix Traditional Chinese Adult 9.0 CPS boundary"),
    ((7, 1000), 70, "Netflix Traditional Chinese Children 7.0 CPS boundary"),
    ((12, 1000), 120, "Netflix Traditional Chinese Hard Red-Line 12.0 CPS boundary"),
    ((18, 2500), 72, "Reference SRT line 1: 18 chars in 2.5s = 7.2 CPS"),
];

/// 參考實作（標準）：對齊 Netflix 繁中規格（70 / 90 / 120 tenths）與外部基準比對
pub mod standard {
    pub fn subtitle_cps_tenths(readable_chars: u32, duration_ms: u32) -> Option<u32> {
        if readable_chars > 5000 || duration_ms < 1 || duration_ms > 600_000 {
            return None;
        }
        let num = (readable_chars as u64) * 10_000 + (duration_ms as u64) / 2;
        Some((num / (duration_ms as u64)) as u32)
    }

    pub fn subtitle_cps_band(cps_tenths: u32, children_mode: u8) -> Option<u8> {
        if cps_tenths > 10_000 || children_mode > 1 {
            return None;
        }
        if cps_tenths > 120 {
            return Some(2);
        }
        let orange_limit = if children_mode == 1 { 70 } else { 90 };
        Some(if cps_tenths > orange_limit { 1 } else { 0 })
    }

    pub fn subtitle_vad_drift_ok(
        cand_start_ms: u32,
        cand_end_ms: u32,
        ref_start_ms: u32,
        ref_end_ms: u32,
        tol_ms: u32,
    ) -> Option<(u8, u32)> {
        if cand_start_ms > 86_400_000
            || cand_end_ms > 86_400_000
            || ref_start_ms > 86_400_000
            || ref_end_ms > 86_400_000
            || cand_end_ms <= cand_start_ms
            || ref_end_ms <= ref_start_ms
            || tol_ms < 1
            || tol_ms > 5000
        {
            return None;
        }
        let d_start = cand_start_ms.abs_diff(ref_start_ms);
        let d_end = cand_end_ms.abs_diff(ref_end_ms);
        let max_drift = d_start.max(d_end);
        Some((u8::from(max_drift <= tol_ms), max_drift))
    }
}

/// 挑戰件 1：舊版 `cps_checker_and_splitter.py` 的雙重標準（成人用 11.0 CPS = 110、兒童用 7.5 CPS = 75）
pub mod challenge_legacy_conflicting_cps {
    pub fn subtitle_cps_tenths(readable_chars: u32, duration_ms: u32) -> Option<u32> {
        super::standard::subtitle_cps_tenths(readable_chars, duration_ms)
    }
    pub fn subtitle_cps_band(cps_tenths: u32, children_mode: u8) -> Option<u8> {
        if cps_tenths > 10_000 || children_mode > 1 {
            return None;
        }
        if cps_tenths > 120 {
            return Some(2);
        }
        let orange_limit = if children_mode == 1 { 75 } else { 110 };
        Some(if cps_tenths > orange_limit { 1 } else { 0 })
    }
    pub fn subtitle_vad_drift_ok(cs: u32, ce: u32, rs: u32, re: u32, tol: u32) -> Option<(u8, u32)> {
        super::standard::subtitle_vad_drift_ok(cs, ce, rs, re, tol)
    }
}

/// 挑戰件 2：舊版 `process_project_vad.py`「自己量自己」缺陷（拿 cand 時間跟 cand 自己比，誤差永遠為 0）
pub mod challenge_self_referencing_vad {
    pub fn subtitle_cps_tenths(readable_chars: u32, duration_ms: u32) -> Option<u32> {
        super::standard::subtitle_cps_tenths(readable_chars, duration_ms)
    }
    pub fn subtitle_cps_band(cps_tenths: u32, children_mode: u8) -> Option<u8> {
        super::standard::subtitle_cps_band(cps_tenths, children_mode)
    }
    pub fn subtitle_vad_drift_ok(cs: u32, ce: u32, rs: u32, re: u32, tol: u32) -> Option<(u8, u32)> {
        super::standard::subtitle_vad_drift_ok(cs, ce, cs, ce, tol).filter(|_| re > rs)
    }
}

/// 挑戰件 3：整數除法截斷（未做四捨五入且少乘 10，算成整數 CPS 而非 tenths）
pub mod challenge_unscaled_cps {
    pub fn subtitle_cps_tenths(readable_chars: u32, duration_ms: u32) -> Option<u32> {
        if readable_chars > 5000 || duration_ms < 1 || duration_ms > 600_000 {
            return None;
        }
        Some((readable_chars * 1000) / duration_ms)
    }
    pub fn subtitle_cps_band(cps_tenths: u32, children_mode: u8) -> Option<u8> {
        super::standard::subtitle_cps_band(cps_tenths, children_mode)
    }
    pub fn subtitle_vad_drift_ok(cs: u32, ce: u32, rs: u32, re: u32, tol: u32) -> Option<(u8, u32)> {
        super::standard::subtitle_vad_drift_ok(cs, ce, rs, re, tol)
    }
}

/// 挑戰件 4：邊界 off-by-one（把 `>= 90` 判成黃燈，導致剛好 9.0 CPS 被誤殺）
pub mod challenge_off_by_one_band {
    pub fn subtitle_cps_tenths(readable_chars: u32, duration_ms: u32) -> Option<u32> {
        super::standard::subtitle_cps_tenths(readable_chars, duration_ms)
    }
    pub fn subtitle_cps_band(cps_tenths: u32, children_mode: u8) -> Option<u8> {
        if cps_tenths > 10_000 || children_mode > 1 {
            return None;
        }
        if cps_tenths >= 120 {
            return Some(2);
        }
        let orange_limit = if children_mode == 1 { 70 } else { 90 };
        Some(if cps_tenths >= orange_limit { 1 } else { 0 })
    }
    pub fn subtitle_vad_drift_ok(cs: u32, ce: u32, rs: u32, re: u32, tol: u32) -> Option<(u8, u32)> {
        super::standard::subtitle_vad_drift_ok(cs, ce, rs, re, tol)
    }
}

/// 挑戰件 5：缺少時間倒錯守衛（`cand_end_ms <= cand_start_ms` 未攔截）
pub mod challenge_missing_inversion_guard {
    pub fn subtitle_cps_tenths(readable_chars: u32, duration_ms: u32) -> Option<u32> {
        super::standard::subtitle_cps_tenths(readable_chars, duration_ms)
    }
    pub fn subtitle_cps_band(cps_tenths: u32, children_mode: u8) -> Option<u8> {
        super::standard::subtitle_cps_band(cps_tenths, children_mode)
    }
    pub fn subtitle_vad_drift_ok(
        cand_start_ms: u32,
        cand_end_ms: u32,
        ref_start_ms: u32,
        ref_end_ms: u32,
        tol_ms: u32,
    ) -> Option<(u8, u32)> {
        if tol_ms < 1 || tol_ms > 5000 {
            return None;
        }
        let max_drift = cand_start_ms.abs_diff(ref_start_ms).max(cand_end_ms.abs_diff(ref_end_ms));
        Some((u8::from(max_drift <= tol_ms), max_drift))
    }
}

pub fn standard_parts_ok() -> bool {
    let basic = STANDARD_PARTS
        .iter()
        .all(|&((chars, dur), expected, _)| standard::subtitle_cps_tenths(chars, dur) == Some(expected));
    let bands = standard::subtitle_cps_band(90, 0) == Some(0)
        && standard::subtitle_cps_band(91, 0) == Some(1)
        && standard::subtitle_cps_band(121, 0) == Some(2)
        && standard::subtitle_cps_band(70, 1) == Some(0)
        && standard::subtitle_cps_band(71, 1) == Some(1);
    basic && bands
}

pub fn blind_check(candidate: fn(u32, u32) -> Option<u32>, n: usize) -> (usize, usize) {
    let mut state: u64 = 0xCAFE_BABE_1234_5678;
    let mut mismatches = 0usize;
    for _ in 0..n {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let chars = (state % 5500) as u32;
        let dur = ((state >> 16) % 650_000) as u32;
        if candidate(chars, dur) != standard::subtitle_cps_tenths(chars, dur) {
            mismatches += 1;
        }
    }
    (mismatches, n)
}
