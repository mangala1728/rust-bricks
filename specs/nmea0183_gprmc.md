# 規格書：NMEA 0183 GPRMC 定位語句校驗與座標解析 (NMEA 0183 GPRMC Parser)

## 模組名稱
`nmea0183_gprmc`

## 業務目標與適用場景
在車載導航、船載 AIS、無人機 GPS 模組中，NMEA 0183 協定的 GPRMC (Recommended Minimum Specific GPS/TRANSIT Data) 是最廣泛使用的標準定位語句。
本模組提供零外部依賴、純 `std` 實作的 NMEA XOR 校驗和驗證與度分格式 (ddmm.mmmm) 經緯度十進位轉換微積木。

## 09 號合約邊界規範

### 函式 1：NMEA 語句 XOR 校驗和驗證
- **函式簽名**：
  ```rust
  /// @outputs: 1
  pub fn nmea_validate_checksum(sentence: &str) -> bool
  ```
- **運算過程**：
  - 語句開頭必須為 `$`，若無回傳 `false`。
  - 尋找 `*` 符號：若無或 `*` 之後字元不足 2 位，回傳 `false`。
  - 計算 `$` 與 `*` 之間所有位元組的 XOR 累加值 `expected_csum`。
  - 解析 `*` 後兩位十六進位字串為 `u8`。
  - 兩者相等回傳 `true`，否則回傳 `false`。

### 函式 2：NMEA 度分座標轉十進位度數 (ddmm.mmmm to Decimal Degrees)
- **函式簽名**：
  ```rust
  /// @outputs: 2
  pub fn nmea_parse_lat_lon(
      raw_lat: &str,
      lat_dir: char,
      raw_lon: &str,
      lon_dir: char,
  ) -> Option<(f64, f64)>
  ```
- **運算過程**：
  - 緯度 `raw_lat` 格式為 `ddmm.mmmm` (前 2 位為度，其餘為分)：
    - `lat_deg = deg + (min / 60.0)`
    - 若 `lat_dir == 'S'` 則 `lat_deg = -lat_deg`；若非 `'N'` 亦非 `'S'` 回傳 `None`。
  - 經度 `raw_lon` 格式為 `dddmm.mmmm` (前 3 位為度，其餘為分)：
    - `lon_deg = deg + (min / 60.0)`
    - 若 `lon_dir == 'W'` 則 `lon_deg = -lon_deg`；若非 `'E'` 亦非 `'W'` 回傳 `None`。
  - 邊界檢查：`lat_deg` 在 `[-90.0, 90.0]`，`lon_deg` 在 `[-180.0, 180.0]`。
  - 回傳 `Some((lat_deg, lon_deg))`。

## 黃金測試向量 (Golden Vectors - NMEA Standard)
- 測試語句：`$GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W*6A`
  - XOR 校驗和應為 `true`。
  - 緯度：`4807.038`, `'N'` $\implies 48 + 7.038 / 60 = 48.1173^\circ$。
  - 經度：`01131.000`, `'E'` $\implies 11 + 31.000 / 60 \approx 11.516667^\circ$。
- 測試容差使用 `(val - golden).abs() < 1e-4`。
- 錯誤語句：校驗和錯誤或方向非 N/S/E/W 應安全返回 `false` 或 `None`。

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_nmea0183_gprmc.rs）
```golden
// 出處：規格向量（NMEA 標準範例語句，Python 獨立 XOR 驗算 0x6A）
nmea_validate_checksum("$GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W*6A")
// 出處：規格錯誤語句：校驗和錯誤應回傳 false
!nmea_validate_checksum("$GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W*6B")
// 出處：函式 1 運算過程：開頭非 '$'、或無 '*' 時回傳 false
!nmea_validate_checksum("GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W*6A")
!nmea_validate_checksum("$GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W")
// 出處：規格向量：48 + 7.038/60 = 48.1173、11 + 31/60 ≈ 11.516667，容差 1e-4
let (lat_ne, lon_ne) = nmea_parse_lat_lon("4807.038", 'N', "01131.000", 'E').unwrap();
(lat_ne - 48.1173).abs() < 1e-4
(lon_ne - 11.516667).abs() < 1e-4
// 出處：函式 2 運算過程：S / W 取負號
let (lat_sw, lon_sw) = nmea_parse_lat_lon("4807.038", 'S', "01131.000", 'W').unwrap();
(lat_sw + 48.1173).abs() < 1e-4
(lon_sw + 11.516667).abs() < 1e-4
// 出處：規格錯誤語句：方向非 N/S/E/W 應回傳 None
nmea_parse_lat_lon("4807.038", 'X', "01131.000", 'E').is_none()
nmea_parse_lat_lon("4807.038", 'N', "01131.000", 'X').is_none()
```
