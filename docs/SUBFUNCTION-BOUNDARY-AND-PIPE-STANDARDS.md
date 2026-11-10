# 通用領域下水道標準與子函數規格邊界指南
# (Universal Domain Pipe & Sub-function Contract Standards)

```text
## Map
┌──────────────────────────────────────────────────────────────────────────────┐
│                    定義者 (Architect / Spec Definer)                         │
│   • 制定規格書 (specs/*.md)、界定物理定義域、撰寫機器驗證黃金向量 (```golden)  │
└──────────────────────────────────────▲───────────────────────────────────────┘
                                       │ 統一量尺：單位精度、定點數標度、09 合約
┌──────────────────────────────────────┴───────────────────────────────────────┐
│                    碼工 (Coder / Cloud Farmer / Rust Engine)                 │
│   • 實作純 Rust 09 微積木、第一行剛性邊界守衛、NASA JPL <= 60 行、14 大鐵律  │
└──────────────────────────────────────▲───────────────────────────────────────┘
                                       │ 機器索引 (CATALOG.yaml / json)、零拷貝 32B 匯流排
┌──────────────────────────────────────┴───────────────────────────────────────┐
│                    用戶 / AI 代理 (Consumer / Calling Agent)                 │
│   • 業務端呼叫、前置尺度縮放 (Scale-Up)、雙時態防偷看、Option/Result 安全解包│
└──────────────────────────────────────────────────────────────────────────────┘
```

> **「不求快開發，一切以狀態可窮舉、邊界可靜態驗證、出事 3 分鐘內定位為最高原則。」**  
> 本指南是 `rust-bricks` 專案與所有上游系統（如 `digital-historian`、Android App、Web/Desktop/Mobile 占星軟體、時空數位孿生引擎）的**「同一把尺」**。  
> 無論是規格定義者、寫碼工兵（含雲端自動化農夫 Cloud Farmer），還是呼叫子函數的用戶與 AI 代理，全體必須在完全相同的刻度與邊界條件下協同工作。

---

## 一、全域下水道度量衡與精度刻度表（Universal Precision & Unit Scales）

為杜絕 IEEE-754 浮點數截斷誤差、跨平台/跨語言序列化歧義及 NaN 毒化，進入下水道與微積木的數值一律採用**純整數定點數（Pure Integer Fixed-Point）**。

| 領域類別 | 規格欄位名稱 | 標度型別 | 比例係數 (Scaling Factor) | 數值範圍 (Domain) | 物理意義與轉換範例 |
|---|---|---|---|---|---|
| **時間 (Bus 1)** | `ts_us` | `u64` | $1\text{ us}$ | `0 .. 1.84e19` | Unix Epoch UTC 微秒（高速事件磁軌專用） |
| **時間 (Bus 2/3)** | `ts_sec` | `u64` | $1\text{ s}$ | `0 .. 4.10e9` | Unix Epoch UTC 秒（歷史地層庫，支援至 2100 年） |
| **時間 (日曆)** | `date_yyyymmdd` | `u32` | 1 | `19000101 .. 20991231` | 整數型日曆日快照（例如 `20261111`） |
| **時間 (相對論)** | `relativistic_drift_ps` | `i64` | $1\text{ ps} (10^{-12}\text{ s})$ | $-2^{63} .. 2^{63}-1$ | 愛因斯坦廣義/狹義相對論原時時間膨脹累積偏移 |
| **角度 (黃道/經緯)** | `angle_milli_deg` | `u32` / `i32` | $1,000$ ($10^{-3}\text{ deg}$) | `0 .. 360000`<br>($-90k..90k$ / $-180k..180k$) | 毫度：$180.5^\circ \to 180500$；黃道 1 宮 $= 30000$ 毫度 |
| **角度 (高精度)** | `angle_micro_deg` | `u32` | $1,000,000$ ($10^{-6}\text{ deg}$) | `0 .. 360000000` | 微度：天體赤經赤緯高精度星曆 |
| **角度 (角分)** | `angle_arcmin` | `u32` | $60$ ($1/60\text{ deg}$) | `0 .. 21600` | 角分：吠陀 27 宿每宿 $800'$；毫度轉角分 $= (\text{milli} \times 3) / 50$ |
| **距離 (天文)** | `distance_au_fp4` | `u16` / `u32` | $10,000$ ($10^{-4}\text{ AU}$) | `1 .. 65535` / `1e6` | 天文單位：地日平均距離 $1.0000\text{ AU} \to 10000$ |
| **距離 (局部站心)** | `local_enu_mm` | `i32` | $1\text{ mm}$ ($10^{-3}\text{ m}$) | $-2\times 10^9 .. 2\times 10^9$ | 站心浮動原點（Floating Origin）東/北/天毫米位移 |
| **容許度與比例** | `orb_tolerance_bp` | `u16` / `i16` | $100$ ($0.01^\circ$ 或 $0.01\%$) | `0 .. 1500` / `\pm 10000` | 基點（Basis Points）：$1.5^\circ = 150\text{ bp} = 1500\text{ milli-deg}$ |
| **速度 (天體日視)** | `speed_milli_deg_day` | `i32` | $1,000$ | $-100000 .. 100000$ | 每日視位移毫度（正值為順行 Direct，負值為逆行 Retrograde） |
| **吠陀第 (Tithi)** | `vedic_tithi_id` | `u8` | 1 | `1 .. 30` | 1..15 白月 (Shukla), 16..30 黑月 (Krishna) |
| **吠陀宿 (Nakshatra)**| `vedic_nakshatra_id` | `u8` | 1 | `1 .. 27` | 1=Ashwini .. 27=Revati (每宿 800 角分) |
| **吠陀足 (Pada)** | `vedic_pada_id` | `u8` | 1 | `1 .. 4` | 每足 200 角分，對應 D9 九分盤 (Navamsha) |
| **吠陀曜 (Yoga)** | `vedic_yoga_id` | `u8` | 1 | `1 .. 27` | 1=Vishkambha .. 27=Vaidhriti |
| **吠陀關 (Karana)** | `vedic_karana_id` | `u8` | 1 | `1 .. 60` | 半個 Tithi，全月 60 關位 |
| **吠陀曜星 (Graha)** | `vedic_dasha_planet_id`| `u8` | 1 | `1 .. 9` | 1=Ketu, 2=Venus, 3=Sun, 4=Moon, 5=Mars, 6=Rahu, 7=Jup, 8=Sat, 9=Merc |
| **歲差角秒 (Ayanamsha)**| `ayanamsha_arcsec` | `u32` | $1''$ ($1/3600\text{ deg}$) | `0 .. 1296000` | 分點歲差角秒：Lahiri J2000 = $85871''$ ($23^\circ 51' 11''$) |

---

## 二、定義者守則（Architect / Spec Definer Guide）

定義者是「量尺的製造者」。當在 `specs/<module_name>.md` 中起草新規格時，必須嚴格遵守以下範式：

### 1. 規格書五大剛性板塊
1. **模組定位**：明確指定 `src/<module_name>.rs` 與 `tests/test_<module_name>.rs`，簡述工程應用價值。
2. **09 合約介面定義**：
   - 宣告輸出狀態基數上限：`/// @outputs: N`（$1 \le N \le 32$）。
   - 每個數值參數必附定義域：`/// @domain: param [min..max]`。
   - 宣告純整數定點數型別，避免使用浮點數。
3. **邊界與異常條件（Domain Guards）**：
   - 明確寫出違反定義域時的回傳行為（一律回傳 `None` 或安全錯誤 `Err`，嚴禁 `panic!`）。
   - 宣告 NASA JPL $\le 60$ 行與巢狀深度 $\le 2$ 要求。
4. **確定性黃金值測試要求（Golden Value Requirements）**：
   - 必須涵蓋恆等基準點（如單位訊號輸入、零延遲增益 $= 1.0$）。
   - 必須覆蓋邊界極值點與非法越界拒絕點。
5. **機器可驗證黃金向量（` ```golden ` 區塊）**：
   - 包含具體數值的斷言表達式，農夫會自動轉為獨立測試檔，模型無權修改。
   - 預期值必須來自標準規範（RFC、IEEE、古籍算式、CRC 標準向量），嚴禁跑錯代碼反填。

---

## 三、碼工與雲端農夫守則（Coder & Cloud Farmer Contract）

寫碼工兵（不論是人類還是自主作業的 Cloud Farmer AI）是「量尺的執行者」：

### 1. 09 號合約四大鐵律
1. **檔案頭接線圖（`## Map`）**：
   ```rust
   // SPDX-License-Identifier: LicenseRef-Project-License
   //! ## Map
   //! - Role: [模組職責]
   //! - Inputs: [輸入規格與單位]
   //! - Outputs: [輸出規格與型別]
   //! - Invariants: [不可違背之物理/記憶體不變量]
   ```
2. **輸出基數標注**：具名函式上方標注 `/// @outputs: N`（$1 \le N \le 32$）。
3. **定義域標注與第一行剛性守衛**：
   ```rust
   /// @outputs: 1
   /// @domain: moon_milli_deg [0..360000]
   /// @domain: sun_milli_deg [0..360000]
   pub fn calculate_tithi(moon_milli_deg: u32, sun_milli_deg: u32) -> Option<u8> {
       // 【剛性守衛】：函式入口第一行必須實作邊界檢核
       if moon_milli_deg > 360_000 || sun_milli_deg > 360_000 {
           return None;
       }
       // 演算法核心（純整數）
       let delta = (moon_milli_deg + 360_000 - sun_milli_deg) % 360_000;
       Some(((delta / 12_000) + 1) as u8)
   }
   ```
4. **結構限制**：
   - 單一具名函式行數嚴格 $\le 60$ 行（NASA JPL 規範）。
   - 控制流巢狀深度 $\le 2$。
   - 零全域可變狀態（禁止 `static mut`）。
   - 禁止任何直接與間接遞迴（一律改顯式有界迴圈）。
   - 嚴禁使用未標注 `// @safety-proof:` 的 `.unwrap()`。

### 2. 14 大工程鐵律重點提醒
- **標準庫數學優先**：使用 `std` 內建函數，嚴禁手刻粗糙殘缺多項式。
- **全鏈條防溢位**：啟用 `saturating_sub` 後，後續運算嚴禁退回裸加號（改用 `saturating_add` 或擴展型別）。
- **整數除法與下向取整**：可能為負數之除法模運算，必須使用 `div_euclid` 與 `rem_euclid`。

---

## 四、用戶與 AI 代理呼叫端守則（Consumer & Calling Agent Guide）

用戶或上層系統的 AI 代理（如 `digital-historian` 或前端應用）是「量尺的使用者」：

### 1. 呼叫前必備：尺度對齊（Scale-Up）
- **❌ 致命錯誤**：
  ```rust
  // 錯誤：直接傳入未縮放的度數浮點或字串
  let tithi = calculate_tithi(142.5, 30.0); // 編譯失敗或型別錯亂
  ```
- **✅ 正確做法**：
  ```rust
  // 依照度量衡刻度表：乘以 1000 轉為毫度整數
  let moon_milli = (moon_deg * 1000.0).round() as u32;
  let sun_milli = (sun_deg * 1000.0).round() as u32;
  let tithi = calculate_tithi(moon_milli, sun_milli);
  ```

### 2. 查閱機器索引（Machine Catalog）
AI 代理在動手撰寫呼叫邏輯前，可直接檢索 [`CATALOG.yaml`](../CATALOG.yaml) 或 [`CATALOG.json`](../CATALOG.json)：
- 檢索 `name`、`outputs`、`domain` 與參數型別。
- 確保輸入值完全落在定義域內，避免非預期之 `None`。

### 3. 錯誤處理與防禦性解包
所有回傳 `Option<T>` 或 `Result<T, E>` 的積木，呼叫端必須優雅處理：
```rust
match calculate_tithi(moon_milli, sun_milli) {
    Some(t) => println!("當前 Tithi: {}", t),
    None => eprintln!("警告：輸入天體經度超出黃道 0..360000 毫度合法範圍"),
}
```

### 4. 高速二進位匯流排封包對接（Bus 1 32 Bytes）
當需要將事件推入 Bus 1 串流磁軌時，一律使用專屬定長 32-Byte 結構體（如 `CelestialPacket32` 或 `VedicEventPacket32`）：
```rust
let pkt = VedicEventPacket32 {
    ts_us: 1_700_000_000_123_456,
    lat_milli_deg: 25_033,       // 25.033° N -> 25033
    lon_milli_deg: 121_565,      // 121.565° E -> 121565
    ayanamsha_arcsec: 85_871,    // Lahiri J2000
    dasha_packed: 0x0001_0703,   // Sun-Jup-Ketu
    transit_summary_hash: 0xABCD_1234,
    event_category_code: 101,    // Career
    flags: 0x0001,
};
assert_eq!(std::mem::size_of::<VedicEventPacket32>(), 32);
let raw_bytes: [u8; 32] = encode_vedic_event_packet_32(&pkt);
// 零拷貝推入磁軌...
```

---

## 五、二進位匯流排對接矩陣速查（Bus Alignment Quick Reference）

| 匯流排編號 | 物理介質 / 傳輸規格 | 天文／占星／時空資料對齊 | 關鍵防線約束 |
|---|---|---|---|
| **Bus 1** | 嚴格 32 Bytes 定長二進位 (小端序) | 行星即時座標、事件日誌點 (`VedicEventPacket32`) | 零指標、無動態配置、CPU 快取行對齊 |
| **Bus 2** | 雙時態歷史地層 TSV | 每日行星星曆、五分曆地層表 | 雙時態因果律：$T_{\text{event}} \le T_{\text{public}} \le T_{\text{sim}}$ |
| **Bus 3** | 多面向因果事件圖譜 (OSINT Graph) | 世紀罕見天象、歲差微調校準殘差因果邊 | 正交 Facet 位元遮罩 (`1<<6`)、稀有度 1..5 星 |
| **Bus 4** | 量化時空向量空間 (`[i32; 32]`) | 站心浮動原點 ENU 毫米位移、月相相角 | 純整數正規化、消滅 32-bit 浮點抽搐 |
