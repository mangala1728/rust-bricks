# 模組名稱: 雙音多頻電話信號解碼器 (DTMF Decoder)

## 1. 目標定位
- 目標模組檔: `src/dsp_dtmf_decoder.rs`
- 目標測試檔: `tests/test_dsp_dtmf_decoder.rs`
- 功能簡述: 依據 ITU-T Q.23 規範，自 8 個標準 DTMF 頻率功率中識別高低頻群有效按鍵並映射至標準字元 ('0'-'9', 'A'-'D', '*', '#')。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: DTMF dual-tone multi-frequency detector and key mapper (ITU-T Q.23)
// Inputs: 8-frequency power spectrum array, threshold: f64, low_idx: u8, high_idx: u8
// Outputs: Option<(u8, u8)> or Option<char>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 2
/// @domain: threshold >= 0.0, powers elements are finite and >= 0.0
pub fn dtmf_detect_frequencies(powers: &[f64; 8], threshold: f64) -> Option<(u8, u8)>;

/// @outputs: 1
/// @domain: low_idx < 4, high_idx < 4
pub fn dtmf_indices_to_char(low_idx: u8, high_idx: u8) -> Option<char>;
```

## 3. 邊界與異常條件 (Domain Guards)
- `dtmf_detect_frequencies`: 若 `threshold < 0.0 || !threshold.is_finite()` 或任何元素 `< 0.0` 或非有限數，立即回傳 `None`。低頻群 (0..4) 與高頻群 (4..8) 最大值若小於 `threshold` 亦回傳 `None`。
- `dtmf_indices_to_char`: 若 `low_idx >= 4 || high_idx >= 4`，立即回傳 `None`。

## 4. 確定性黃金值測試要求
- 低頻群: 697Hz (0), 770Hz (1), 852Hz (2), 941Hz (3)
- 高頻群: 1209Hz (0), 1336Hz (1), 1477Hz (2), 1633Hz (3)
- 映射表:
  - (0, 0) -> '1', (0, 1) -> '2', (0, 2) -> '3', (0, 3) -> 'A'
  - (1, 0) -> '4', (1, 1) -> '5', (1, 2) -> '6', (1, 3) -> 'B'
  - (2, 0) -> '7', (2, 1) -> '8', (2, 2) -> '9', (2, 3) -> 'C'
  - (3, 0) -> '*', (3, 1) -> '0', (3, 2) -> '#', (3, 3) -> 'D'

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_dsp_dtmf_decoder.rs）
```golden
dtmf_indices_to_char(0, 0) == Some('1')
dtmf_indices_to_char(3, 1) == Some('0')
dtmf_indices_to_char(3, 2) == Some('#')
dtmf_indices_to_char(1, 3) == Some('B')
dtmf_indices_to_char(4, 0).is_none()
let p = [10.0, 1.0, 1.0, 1.0, 1.0, 12.0, 1.0, 1.0];
dtmf_detect_frequencies(&p, 5.0) == Some((0, 1))
let p_low = [1.0, 1.0, 1.0, 1.0, 1.0, 12.0, 1.0, 1.0];
dtmf_detect_frequencies(&p_low, 5.0).is_none()
```
