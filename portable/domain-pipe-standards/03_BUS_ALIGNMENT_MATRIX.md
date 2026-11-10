# 四條標準匯流排對接矩陣 (Bus Alignment Matrix)

本文件詳細規範天文學（Astronomy）、星曆表（Ephemeris）、占星術週期（Astrological Cycles）及外部領域資料如何精確對齊系統的四大底層匯流排。

---

## 匯流排對接全覽表

| 匯流排編號 | 匯流排名稱 | 物理介質 / 傳輸規格 | 天文／占星對應資料範例 | 關鍵約束 |
|---|---|---|---|---|
| **Bus 1** | 高速二進位事件磁軌 (Binary Tape Stream) | 嚴格 32 Bytes 定長二進位封包 (小端序) | 即時行星黃道座標串流、地磁風暴秒級感測、相位發生信標 | 零拷貝、零堆疊分配、無指標、單調時間戳記 |
| **Bus 2** | 雙時態歷史地層庫 (Bitemporal Stratigraphy) | 帶 `#` 表頭之 TSV 結構化純文字 | 每日行星星曆表 (JPL DE440/Swiss Ephemeris)、歷史月相交角庫 | 雙時態因果律：$T_{\text{event}} \le T_{\text{public}} \le T_{\text{sim}}$ |
| **Bus 3** | 多面向因果事件圖譜 (Chronicle OSINT Graph) | 節點 TSV + 因果邊 TSV | 世紀罕見天象 (如土木大合相、日全蝕) 關聯至宏觀大宗物資/股指衝擊 | 正交 Facet 遮罩、滯後天數、稀有度 1..5 星 |
| **Bus 4** | 量化時空向量與數位孿生 (Quant Vector & VME) | 固定維度特徵陣列 (`[i32; 32]`) | 行星相位角量化維度、月相週期相角、DTW 時空形狀檢索 | 定點正規化整數、跳躍擴散蒙地卡羅防破產 |

---

## 一、Bus 1：天體事件 32 位元組二進位封包（`CelestialPacket32`）

任何串流天體資料進入 Bus 1 必須嚴格填滿 32 個 Bytes，記憶體佈局如下：

```text
Offset (Bytes)   Field Name                   Type     Scale / Unit            Description
────────────────────────────────────────────────────────────────────────────────────────────────
00 .. 07 (8B)    ts_us                        u64      1 us (UTC Unix)         微秒級時間戳記
08 .. 11 (4B)    ecliptic_longitude_milli_deg u32      10^-3 deg (0..360000)   天體黃道經度 (0.000° ~ 360.000°)
12 .. 15 (4B)    aspect_angle_milli_deg       u32      10^-3 deg (0..360000)   與目標天體之相位角 (如 0, 90000, 180000)
16 .. 19 (4B)    speed_milli_deg_day          i32      10^-3 deg/day           天體日視速度 (負值為逆行)
20 .. 21 (2B)    primary_body_id              u16      1..999                  主天體代碼 (1=Sun, 3=Mercury, ...)
22 .. 23 (2B)    aspect_target_id             u16      0..999                  相位對象天體 (0=None, 1=Sun, ...)
24 .. 25 (2B)    distance_au_fp4              u16      10^-4 AU (0..65535)     地心或日心距 (0.0001 ~ 6.5535 AU)
26 .. 26 (1B)    aspect_type_code             u8       0..15                   相位類型 (0=合, 1=六合, 2=刑, 3=拱, 4=沖)
27 .. 27 (1B)    motion_flags                 u8       Bitmask 0x01..0x80      狀態旗標 (bit0:逆行, bit1:留, bit2:日蝕)
28 .. 29 (2B)    aspect_orb_bp                u16      1 bp = 0.01 deg         相位誤差容許度 (0 .. 1500 bp)
30 .. 31 (2B)    seq_mod                      u16      0..65535                環形序號 (用以檢測掉封包)
────────────────────────────────────────────────────────────────────────────────────────────────
總長度 (Total): 8 + 4 + 4 + 4 + 2 + 2 + 2 + 1 + 1 + 2 + 2 = 32 Bytes
```

---

## 二、Bus 2：雙時態星曆地層庫 TSV 格式（`EphemerisStratigraphyRow`）

Bus 2 儲存每日或每週期的天體曆表，必須確保「歷史回測不踩未來星曆偷看雷」。

### TSV 表頭規範：
```tsv
#entity_id	event_period_ts	public_known_ts	date_yyyymmdd	body_id	ecliptic_lon_milli_deg	speed_milli_deg_day	is_retrograde	moon_phase_pct	solar_flux_index	geomagnetic_kp_index
```

### 欄位釋義：
1. `entity_id`: 觀測對象代碼（例如地球觀測視角 = `1000`）。
2. `event_period_ts`: 物理交角或星象發生的物理真實時間（Unix 秒）。
3. `public_known_ts`: 該星曆資料正式出版／公開發布之時間戳記（Unix 秒）。
   - 若星曆是在 2020 年由 NASA JPL DE440 預先計算發布，則 `public_known_ts = 2020年發布戳記`。
   - 若為即時地磁儀觀測資料，則 `public_known_ts = 觀測報告出爐戳記`。
4. `date_yyyymmdd`: 日曆日（如 `20261111`）。
5. `body_id`: 天體代碼（如 3=水星）。
6. `ecliptic_lon_milli_deg`: 黃道經度毫度（`0..360000`）。
7. `speed_milli_deg_day`: 每日行進速度毫度（負數為逆行）。
8. `is_retrograde`: 逆行旗標（`0`=順行, `1`=逆行）。
9. `moon_phase_pct`: 月相亮度百分比（`0`=朔, `50`=弦, `100`=望/滿月）。
10. `solar_flux_index`: 太陽電波通量指標（SFU, 0..500）。
11. `geomagnetic_kp_index`: 地磁風暴 Kp 指標（定點數 0..90，Kp 9.0 記為 90）。

---

## 三、Bus 3：因果知識圖譜節點與邊緣規範

重大天象事件（如每 20 年一次的土木大合相、世紀日全蝕、水星逆行開始）作為因果拓撲節點。

### 1. 節點定義（`ChronicleNode`）：
- `node_id`: 唯一整數編號。
- `public_known_ts_sec`: 公開時間戳記。
- `facet_axes_mask`: 正交面向遮罩，天文事件固定包含：
  `FACET_ASTRO_CELESTIAL` (`1 << 6`) 或 `FACET_SOLAR_GEOMAGNETIC` (`1 << 7`)。
- `node_rarity_score`: 稀有度星等（$1 \le S \le 5$）。例如：
  - 1 星：常態滿月／新月。
  - 3 星：每年數次的水星逆行開始。
  - 5 星：土木二十年大合相或雙重日食。
- `title_slug`: 簡短標籤（如 `"mercury-retrograde-2026-q1"`）。

### 2. 因果邊定義（`CausalEdge`）：
- `from_node_id`: 天文事件節點 ID。
- `to_node_id`: 市場實體節點 ID（如台指期、黃金期貨、半導體指數）。
- `expected_lag_days`: 預期滯後反應天數（例如水星逆行發酵天數 `3 .. 7` 天）。
- `edge_value_score`: 因果歷史顯著性評分（1..5 星）。

---

## 四、Bus 4：量化特徵向量與五階 SOP 嵌入規範

當右半球量化模型（如 DTW 形狀匹配、五階白盒 SOP、跳躍擴散蒙地卡羅）評估交易時，天象參數作為特徵向量的附加維度：

```rust
pub struct AstroPhysicsDimensionVector {
    pub dim_moon_phase_pct: u8,        // [0..100] 月相亮度百分比
    pub dim_mercury_retrograde: u8,    // [0..2] 0=順行, 1=逆行, 2=留
    pub dim_aspect_tension_bp: i16,    // [-10000..10000] 凶相(刑沖)與吉相(拱六合)代數和 (bp)
    pub dim_geomagnetic_storm_kp: u8,  // [0..90] 地磁擾動指數
    pub dim_solar_flux_scaled: u16,    // [0..5000] 太陽黑子通量定點數
}
```

任何數值若越界，下水道執行期守衛（Runtime Guards）必須立即拋棄並觸發安全拒絕，防止髒數據污染策略決策。
