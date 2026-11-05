# 模組名稱: LIN 匯流排 Classic 與 Enhanced 檢查碼演算法 (LIN Bus Checksum)

## 1. 目標定位
- 目標模組檔: `src/lin_checksum_classic_enhanced.rs`
- 目標測試檔: `tests/test_lin_checksum_classic_enhanced.rs`
- 功能簡述: 依據 LIN 1.3 與 LIN 2.0/2.1 規範計算 LIN 訊框校驗和。支援 Classic Checksum (僅資料欄位) 與 Enhanced Checksum (含受保護識別碼 PID)，並自動處理進位回折 (Carry Folding) 與診斷訊框 (PID 0x3C/0x3D) 強制 Classic 規則。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: LIN 1.3 Classic and LIN 2.x Enhanced checksum computation
// Inputs: pid: u8, data: &[u8], enhanced: bool
// Outputs: Option<u8>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: data.len() in [1..=8], pid <= 0xFF
pub fn lin_compute_checksum(pid: u8, data: &[u8], enhanced: bool) -> Option<u8>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若 `data.is_empty() || data.len() > 8`，立即回傳 `None`。
- 運算法則：
  - 若 `enhanced` 為 `true` 且 `pid != 0x3C && pid != 0x3D && (pid & 0x3F) != 0x3C && (pid & 0x3F) != 0x3D`，初始 `sum = pid as u16`。
  - 否則（包含 Classic 或 PID 為 0x3C/0x3D 診斷訊框），初始 `sum = 0u16`。
  - 對資料位元組逐一累加：`sum += byte as u16`。
  - 若 `sum > 0xFF`，執行進位回折：`sum = (sum & 0xFF) + (sum >> 8)`。
  - 最後若仍大於 0xFF 再折一次：`if sum > 0xFF { sum = (sum & 0xFF) + (sum >> 8); }`。
  - 最終反轉取反碼：`checksum = (!(sum as u8))`。

## 4. 確定性黃金值測試要求
- LIN 規格經典向量：
  - 資料 `[0x01, 0x02, 0x03]`，Classic：
    `sum = 1 + 2 + 3 = 6` -> `!6 = 0xF9`
  - 資料 `[0x4A, 0x55, 0x93, 0xE5]`，PID = 0x22, Enhanced：
    `sum = 0x22 + 0x4A + 0x55 + 0x93 + 0xE5 = 0x239`
    Carry fold: `0x39 + 2 = 0x3B` -> `!0x3B = 0xC4`
  - 診斷訊框 PID 0x3C，即使 `enhanced = true` 也強制不計入 PID：
    資料 `[0x01, 0x02]` -> `sum = 3` -> `!3 = 0xFC`

```golden
lin_compute_checksum(0x00, &[0x01, 0x02, 0x03], false) == Some(0xF9)
lin_compute_checksum(0x22, &[0x4A, 0x55, 0x93, 0xE5], true) == Some(0xC4)
lin_compute_checksum(0x3C, &[0x01, 0x02], true) == Some(0xFC)
lin_compute_checksum(0x00, &[], false).is_none()
lin_compute_checksum(0x00, &[1, 2, 3, 4, 5, 6, 7, 8, 9], false).is_none()
```
