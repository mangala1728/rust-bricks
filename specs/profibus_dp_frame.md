# 模組名稱: 工業現場匯流排 PROFIBUS DP SD2 可變長度訊框驗證與解碼器 (PROFIBUS DP SD2 Frame Validator)

## 1. 目標定位
- 目標模組檔: `src/profibus_dp_frame.rs`
- 目標測試檔: `tests/test_profibus_dp_frame.rs`
- 功能簡述: 依據 IEC 61158 / EN 50170 規範解析與校驗 PROFIBUS DP SD2 (0x68) 可變長度資料報。驗證長度重複核對 (LE == LEr)、啟始符重複 (SD2)、FCS (八位元模 256 累加和) 與結束符 (ED = 0x16)。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: PROFIBUS DP SD2 (0x68) variable data frame decoder and FCS verifier
// Inputs: frame: &[u8]
// Outputs: Option<(u8, u8, u8, usize)> (DA, SA, FC, payload_len)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 4
/// @domain: frame.len() >= 9, frame.len() <= 255
pub fn profibus_parse_sd2(frame: &[u8]) -> Option<(u8, u8, u8, usize)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若 `frame.len() < 9`（最少 1 個位元組 payload：4 byte header + 3 byte header/address + 1 byte data + FCS + ED = 10，或無 payload 時 LE=3 總長 9），立即回傳 `None`。
- 檢查標頭：
  - `frame[0] != 0x68` 或 `frame[3] != 0x68` $\rightarrow$ `None`
  - `let le = frame[1]; let ler = frame[2];`
  - 若 `le != ler || le < 3` $\rightarrow$ `None`
  - 預期總長度為 `le as usize + 6` (包含 SD2, LE, LEr, SD2 (4 bytes) + DA, SA, FC, Data (LE bytes) + FCS (1 byte) + ED (1 byte))
  - 若 `frame.len() < le as usize + 6` $\rightarrow$ `None`
  - 結束符校驗：`frame[le as usize + 5] != 0x16` $\rightarrow$ `None`
- FCS 校驗：
  - FCS 涵蓋範圍為第 4 位元組 (DA) 至第 `3 + le as usize` 位元組。
  - `sum = sum(frame[4..=3 + le]) % 256`。
  - 若 `sum as u8 != frame[4 + le as usize]` $\rightarrow$ `None`。
- 成功解析回傳 `Some((frame[4], frame[5], frame[6], le as usize - 3))`。

## 4. 確定性黃金值測試要求
- 構造合規訊框：
  - `SD2=0x68, LE=4, LEr=4, SD2=0x68, DA=0x02, SA=0x01, FC=0x49, Data=[0xFF]`
  - `sum = 0x02 + 0x01 + 0x49 + 0xFF = 0x14B` $\rightarrow$ `FCS = 0x4B`
  - `ED = 0x16`
  - 總長度 = 10 位元組。
  - 解析結果：`DA=2, SA=1, FC=0x49, payload_len=1`。
- 損壞 FCS 或長度不匹配時返回 `None`。

```golden
let valid_frame = [0x68, 0x04, 0x04, 0x68, 0x02, 0x01, 0x49, 0xFF, 0x4B, 0x16];
profibus_parse_sd2(&valid_frame) == Some((2, 1, 0x49, 1))
let mut bad_fcs = valid_frame; bad_fcs[8] = 0x00;
profibus_parse_sd2(&bad_fcs).is_none()
let mut bad_ed = valid_frame; bad_ed[9] = 0x15;
profibus_parse_sd2(&bad_ed).is_none()
let mut bad_ler = valid_frame; bad_ler[2] = 0x05;
profibus_parse_sd2(&bad_ler).is_none()
profibus_parse_sd2(&valid_frame[0..8]).is_none()
```
