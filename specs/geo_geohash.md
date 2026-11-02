# 規格書：地理空間 Geohash 二進位位元交錯編解碼器 (Geohash Bit-Interleaving Codec)

## 模組名稱
`geo_geohash`

## 業務目標與適用場景
在地理資訊系統（GIS）、適地性服務（LBS, 如叫車、外送、周邊店家檢索）與空間索引資料庫中，Geohash 透過將經緯度區間二分搜尋並交錯交疊二進位位元（Even 位元為經度，Odd 位元為緯度），將二維座標轉換為一維整數或字串，具備卓越的前綴鄰近聚集特性。
本模組提供零外部依賴、純 `std` 實作的任意位元長度（1~64 bits）Geohash 整數編解碼微積木。

## 09 號合約邊界規範

### 函式 1：經緯度編碼為二進位 Geohash 整數 (Encode Lat/Lon to Geohash Bits)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: lat [-90.0..=90.0], lon [-180.0..=180.0], precision_bits [1..=64]
  pub fn geo_geohash_encode_bits(lat: f64, lon: f64, precision_bits: u8) -> Option<u64>
  ```
- **演算法規範**：
  - 邊界檢查：`if !lat.is_finite() || !lon.is_finite() || lat < -90.0 || lat > 90.0 || lon < -180.0 || lon > 180.0 || precision_bits == 0 || precision_bits > 64 { return None; }`
  - 初始搜尋邊界：
    `lat_min = -90.0, lat_max = 90.0;`
    `lon_min = -180.0, lon_max = 180.0;`
  - 迭代 `i` 由 `0` 至 `precision_bits - 1`：
    - 若 `i % 2 == 0`（經度分步）：
      - `mid = (lon_min + lon_max) / 2.0;`
      - 若 `lon >= mid`：`hash = (hash << 1) | 1; lon_min = mid;`
      - 否則：`hash = hash << 1; lon_max = mid;`
    - 若 `i % 2 == 1`（緯度分步）：
      - `mid = (lat_min + lat_max) / 2.0;`
      - 若 `lat >= mid`：`hash = (hash << 1) | 1; lat_min = mid;`
      - 否則：`hash = hash << 1; lat_max = mid;`
  - 回傳 `Some(hash)`。

### 函式 2：Geohash 位元解碼為經緯度包圍盒 (Decode Geohash Bits to Bounding Box)
- **函式簽名**：
  ```rust
  /// @outputs: 4
  /// @domain: precision_bits [1..=64]
  pub fn geo_geohash_decode_bounds(hash: u64, precision_bits: u8) -> Option<((f64, f64), (f64, f64))>
  ```
- **定義**：
  - 邊界檢查：`if precision_bits == 0 || precision_bits > 64 { return None; }`
  - 初始區間同上。
  - 逆向依位元由高位至低位（從 `bit = precision_bits - 1` 降至 `0`）解碼：
    - `let bit_val = (hash >> bit) & 1;`
    - 根據步驟順序收斂 `lat_min/max` 與 `lon_min/max`。
  - 回傳 `Some(((lat_min, lat_max), (lon_min, lon_max)))`。

## 黃金測試向量 (Golden Vectors)
- 向量 1 (原點 0.0, 0.0):
  - 4 bits 編碼:
    - bit 0 (lon >= 0.0): 1, lon 區間 [0.0, 180.0]
    - bit 1 (lat >= 0.0): 1, lat 區間 [0.0, 90.0]
    - bit 2 (lon >= 90.0): 0, lon 區間 [0.0, 90.0]
    - bit 3 (lat >= 45.0): 0, lat 區間 [0.0, 45.0]
    - 期望二進位: `0b1100` = `12`
  - `geo_geohash_encode_bits(0.0, 0.0, 4)` 應回傳 `Some(12)`
- 向量 2 (台北座標 25.0330, 121.5654):
  - 10 bits 編碼應精確收斂且 `decode_bounds` 必須包含原始座標。
