# 模組名稱: 影視字幕語速分級與外部基準對照微積木 (Subtitle CPS & External Reference VAD Evaluator)

## 1. 目標定位
- 目標模組檔: `src/subtitle_cps_vad.rs`
- 目標測試檔: `tests/msa_subtitle_cps_vad.rs`
- 功能簡述: 提供符合 **Netflix Timed Text Style Guide (Traditional Chinese)** 單一外部規格來源之字幕每秒字數（CPS，以十分之一字/秒 `tenths` 定點表示）計算、兒童（`7.0 CPS = 70`）／成人警戒（`9.0 CPS = 90`）／硬性紅線（`12.0 CPS = 120`）三段分級，以及相對於外部人工精修黃金基準字幕之起訖時間漂移（Drift）驗收判定。
- 範圍限制: 純定點整數毫秒（`ms`）與十分之一字/秒（`tenths`）計算，不含檔案 I/O 或正則表達式。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: 影視字幕 CPS 定點語速計算、Netflix 繁中三段分級與外部基準時間漂移驗收
// Inputs: readable_chars: u32, duration_ms: u32, cps_tenths: u32, children_mode: u8, cand/ref timestamps (u32)
// Outputs: Option<u32>, Option<u8>, Option<(u8, u32)>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 2
/// @domain: readable_chars [0..5000]
/// @domain: duration_ms [1..600000]
pub fn subtitle_cps_tenths(readable_chars: u32, duration_ms: u32) -> Option<u32>;

/// @outputs: 4
/// @domain: cps_tenths [0..10000]
/// @domain: children_mode [0..1]
pub fn subtitle_cps_band(cps_tenths: u32, children_mode: u8) -> Option<u8>;

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
) -> Option<(u8, u32)>;
```

## 3. 演算法與邊界 (Domain Guards)
- `subtitle_cps_tenths(readable_chars, duration_ms)`:
  - 若 `readable_chars > 5000 || duration_ms < 1 || duration_ms > 600_000`，回傳 `None`。
  - 以四捨五入計算十分之一字/秒：`((readable_chars as u64 * 10_000 + (duration_ms as u64 / 2)) / (duration_ms as u64)) as u32`。
- `subtitle_cps_band(cps_tenths, children_mode)`:
  - 若 `cps_tenths > 10_000 || children_mode > 1`，回傳 `None`。
  - 依 `@source: Netflix Timed Text Style Guide (Traditional Chinese)`：
    - 若 `cps_tenths > 120`（> 12.0 CPS 硬性紅線），回傳 `Some(2)`（Red）。
    - 若 `children_mode == 1` 且 `cps_tenths > 70`（> 7.0 CPS 兒童節目上限），回傳 `Some(1)`（Orange）。
    - 若 `children_mode == 0` 且 `cps_tenths > 90`（> 9.0 CPS 成人節目警戒線），回傳 `Some(1)`（Orange）。
    - 其餘回傳 `Some(0)`（Green）。
- `subtitle_vad_drift_ok(cand_start_ms, cand_end_ms, ref_start_ms, ref_end_ms, tol_ms)`:
  - 若任一時間戳 `> 86_400_000`、`cand_end_ms <= cand_start_ms`、`ref_end_ms <= ref_start_ms`、或 `tol_ms < 1 || tol_ms > 5000`，回傳 `None`。
  - `d_start = cand_start_ms.abs_diff(ref_start_ms)`，`d_end = cand_end_ms.abs_diff(ref_end_ms)`，`max_drift = d_start.max(d_end)`。
  - 若 `max_drift <= tol_ms` 回傳 `Some((1, max_drift))`，否則回傳 `Some((0, max_drift))`。

## 4. 確定性黃金值測試要求
```golden
// @source: Netflix Timed Text Style Guide (Traditional Chinese) — 9 chars in 1000ms = 9.0 CPS (90 tenths, adult green boundary)
subtitle_cps_tenths(9, 1000) == Some(90)
// @source: Netflix Timed Text Style Guide (Traditional Chinese) — 18 chars in 2500ms = 7.2 CPS (72 tenths)
subtitle_cps_tenths(18, 2500) == Some(72)
// @source: Netflix Timed Text Style Guide (Traditional Chinese) — Adult <= 9.0 CPS (90) is Green (0), 9.1 CPS (91) is Orange (1), 12.1 CPS (121) is Red (2)
subtitle_cps_band(90, 0) == Some(0)
subtitle_cps_band(91, 0) == Some(1)
subtitle_cps_band(120, 0) == Some(1)
subtitle_cps_band(121, 0) == Some(2)
// @source: Netflix Timed Text Style Guide (Traditional Chinese) — Children <= 7.0 CPS (70) is Green (0), 7.2 CPS (72) is Orange (1)
subtitle_cps_band(70, 1) == Some(0)
subtitle_cps_band(72, 1) == Some(1)
// @source: External Reference SRT 100ms sync tolerance — 40ms drift passes (1, 40), 350ms drift fails (0, 350)
subtitle_vad_drift_ok(1040, 3480, 1000, 3500, 100) == Some((1, 40))
subtitle_vad_drift_ok(1350, 3850, 1000, 3500, 100) == Some((0, 350))
// @source: Domain guards (zero duration, inverted timestamps, out-of-range parameters)
subtitle_cps_tenths(10, 0).is_none()
subtitle_cps_tenths(5001, 1000).is_none()
subtitle_cps_band(100, 2).is_none()
subtitle_vad_drift_ok(3500, 1000, 1000, 3500, 100).is_none()
subtitle_vad_drift_ok(1000, 3500, 1000, 3500, 0).is_none()
```
