# 吠陀占星、天文下水道管線與時空數位孿生架構開發者指南
# (Vedic Astrology, Celestial Domain Pipe & Spatiotemporal Digital Twin Developer Guide)

> **給專案開發 AI 代理的交接總綱**  
> 本文件由底層下水道與微積木庫架構師編寫，旨在為後續負責「吠陀占星系統（Web/Desktop/Mobile）」、「人生重大事件記錄器（Life Event Recorder）」、「分點歲差與生時校正（BTR）」及「時空數位孿生（Spatiotemporal Digital Twin）」的開發代理，提供完整、權威、無歧義的資料格式、純整數算術公式、二進位排佈、物理誤差分析與 API 呼叫指南。

---

```text
## Map（系統整體接線與匯流排總覽）
┌──────────────────────────────────────────────────────────────────────────────┐
│                    上層跨端應用：吠陀占星 / 事件記錄器 / 數位孿生            │
│       • Web (Wasm) / Desktop (Tauri) / Mobile (Android NDK) / VR (OpenXR)    │
└──────────────────────────────────────▲───────────────────────────────────────┘
                                       │ 32B 定長二進位封包 / 毫度 / 毫角秒 / 純整數
┌──────────────────────────────────────┴───────────────────────────────────────┐
│                 四大標準匯流排下水道管線 (Domain Pipe Standards)              │
│                                                                              │
│  [Bus 1] 高速事件磁軌 (32B 定長) ───> VedicEventPacket32 (小端序、微秒時間)   │
│  [Bus 2] 雙時態歷史地層 (TSV)    ───> PanchangaStratigraphy (因果防偷看)     │
│  [Bus 3] 因果事件拓撲圖譜        ───> 歲差殘差最小化 (BTR) 與歷史大宗事件    │
│  [Bus 4] 時空數位孿生四圖層      ───> ICRF -> ECEF -> ENU (浮動原點) -> 相機  │
└──────────────────────────────────────▲───────────────────────────────────────┘
                                       │
┌──────────────────────────────────────┴───────────────────────────────────────┐
│                  純 Rust 09 合約微積木庫 (`rust-bricks`)                      │
│  • astro_pipeline: 32B 封包編解碼 (encode_vedic_event_packet_32)             │
│  • astro_historical_dst: 歷史夏令時法規查表防呆 & Clock-to-UTC 負號修正      │
│  • astro_prenatal_dasha: 產前大限逆算 (Vimshottari Reverse) & D9/D12 校正     │
│  • astro_varga_super: 超高階分盤 (D81/D108/D144) 角秒純整數幾何求解器         │
│  • astro_varga_cusp_analysis: D300 臨界歧度 (Sandhi) 300倍誤差放大與站心視差 │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## 一、核心哲學：為什麼全鏈條嚴禁浮點數？（The Pure Integer Axiom）

在傳統占星與遊戲開發中，開發者常隨意使用 `f32` 或 `f64`。這在專業工程中會引發三大毀滅性災難：
1. **跨平台截斷漂移（Non-determinism）**：x86、ARM（手機）、WebAssembly 的 FPU 捨入邏輯在臨界邊界（如 $29^\circ 59' 59.9''$）會產生不同的宮位判定，導致同一個人在手機上排盤是金牛座，在伺服器上卻是雙子座！
2. **大尺度浮點抽搐（32-bit Floating Jitter）**：地球半徑約 6,378 公里。以地心為原點使用 `f32`，在地表解析度僅剩約 1 公尺，角色在街景中移動會劇烈抽搐穿模。
3. **時間下溢與時區混亂**：傳統字串時間或無符號時間戳記，遇到西元前（BCE）或 1970 以前直接爆掉。

**解決之道：全鏈條純整數定點數（Pure Integer Fixed-Point Math）！**

---

## 二、全域下水道度量衡刻度表（Universal Precision & Unit Scales）

任何對接進系統的數值，必須嚴格依照以下刻度進行縮放（Scale-Up）：

| 領域刻度 | 規格欄位名稱 | 標度型別 | 比例因數 (Scaling Factor) | 數值範圍 (Domain) | 物理意義與轉換範例 |
|---|---|---|---|---|---|
| **時間 (Bus 1)** | `ts_us` | `u64` | $1\text{ us}$ | `0 .. 1.84e19` | Unix Epoch UTC 微秒（高速串流磁軌專用） |
| **時間 (Bus 2/3)** | `ts_sec` | `i64` | $1\text{ s}$ | $-6.21\times 10^{10} .. 2.53\times 10^{11}$ | Unix Epoch 秒（**支援西元前至西元 9999 年**） |
| **時間 (日曆)** | `date_yyyymmdd` | `u32` | 1 | `19000101 .. 20991231` | 整數型日曆日快照（例如 `20261111`） |
| **時間 (相對論)** | `relativistic_drift_ps` | `i64` | $1\text{ ps} (10^{-12}\text{ s})$ | $-2^{63} .. 2^{63}-1$ | 愛因斯坦廣義/狹義相對論原時時間膨脹累積偏移 |
| **角度 (黃道/經緯)** | `angle_milli_deg` | `u32` / `i32` | $1,000$ ($10^{-3}\text{ deg}$) | `0 .. 360000`<br>($-90k..90k$ / $-180k..180k$) | 毫度：$180.5^\circ \to 180500$；黃道 1 宮 $= 30000$ 毫度 |
| **角度 (角分)** | `angle_arcmin` | `u32` | $60$ ($1/60\text{ deg}$) | `0 .. 21600` | 毫度轉角分純整數：$(\text{milli\_deg} \times 3) / 50$ |
| **角度 (角秒)** | `angle_arcsec` | `u32` | $3,600$ ($1/3600\text{ deg}$) | `0 .. 1296000` | 角秒：全天球 $= 1,296,000''$；1宮 $= 108,000''$ |
| **角度 (極限微度)** | `angle_mas` | `u64` | $3,600,000$ ($10^{-3}''$) | `0 .. 1296000000000` | 毫角秒（mas）：**D300 臨界歧度專用，1宮 $= 108,000,000\text{ mas}$** |
| **距離 (天文)** | `distance_au_fp4` | `u16` / `u32` | $10,000$ ($10^{-4}\text{ AU}$) | `1 .. 65535` / `1e6` | 地日平均距離 $1.0000\text{ AU} \to 10000$ |
| **距離 (局部站心)** | `local_enu_mm` | `i32` | $1\text{ mm} (10^{-3}\text{ m})$ | $-2\times 10^9 .. 2\times 10^9$ | 浮動原點毫米位移，消滅 32-bit 浮點抽搐穿模 |
| **容許度與比率** | `orb_tolerance_bp` | `u16` / `i16` | $100$ ($0.01^\circ$ 或 $0.01\%$) | `0 .. 1500` / $\pm 10000$ | 基點 (Basis Points)：$1.5^\circ = 150\text{ bp} = 1500\text{ milli-deg}$ |

---

## 三、吠陀五分曆（Panchanga）純整數計算公式

吠陀占星與五分曆核心在於太陽（Surya）與月亮（Chandra）的幾何關係。所有計算 100% 採用純整數：

1. **角度轉換基礎**：
   - 毫度轉角分（零捨入誤差）：
     $$\text{arcmin} = \frac{\text{milli\_deg} \times 3}{50}$$
2. **五大要素算術模型**：
   - **第 (Tithi 1..30)**：月日經度差，每 $12^\circ = 12,000\text{ milli-deg}$ 一第。
     $$\Delta = (\text{moon\_milli} + 360000 - \text{sun\_milli}) \pmod{360000}$$
     $$\text{Tithi} = \lfloor \Delta / 12000 \rfloor + 1 \in [1..30]$$
     （1..15 白月 Shukla, 16..30 黑月 Krishna；進度基點 $\text{progress\_bp} = (\Delta \pmod{12000} \times 10000) / 12000$）
   - **宿 (Nakshatra 1..27)**：月球所落宿度，每宿跨度 $13^\circ 20' = 800\text{ arcmin}$。
     $$M_{\text{min}} = (\text{moon\_milli} \times 3) / 50 \implies \text{Nakshatra} = \lfloor M_{\text{min}} / 800 \rfloor + 1 \in [1..27]$$
   - **足 (Pada 1..4)**：每宿切分為 4 足，每足 $3^\circ 20' = 200\text{ arcmin}$（對應 D9 九分盤 Navamsha）。
     $$\text{Pada} = \lfloor (M_{\text{min}} \pmod{800}) / 200 \rfloor + 1 \in [1..4]$$
   - **曜 (Yoga 1..27)**：日月經度之和，每 $13^\circ 20' = 800\text{ arcmin}$ 一曜。
     $$\text{Yoga} = \lfloor ((S_{\text{min}} + M_{\text{min}}) \pmod{21600}) / 800 \rfloor + 1 \in [1..27]$$
   - **關 (Karana 1..60)**：半個 Tithi，每 $6^\circ = 6000\text{ milli-deg}$ 一關。
     $$\text{Karana} = \lfloor \Delta / 6000 \rfloor + 1 \in [1..60]$$
3. **特殊 Tithi 判定規則（Kshaya & Vriddhi Tithi）**：
   - 官方 Tithi 以**當日日出瞬間（Sunrise）正在生效的 Tithi** 為準。
   - **缺日 / 虧日 (Kshaya)**：該 Tithi 在今日日出後開始，且在次日日出前結束（**完全未涵蓋任何日出**，民曆跳過）：
     `tithi_start_ts > sunrise_curr_ts && tithi_end_ts < sunrise_next_ts`
   - **增日 (Vriddhi)**：該 Tithi 橫跨連續兩次日出（民曆連續兩天相同 Tithi）：
     `tithi_start_ts <= sunrise_curr_ts && tithi_end_ts >= sunrise_next_ts`

---

## 四、人生事件記錄器與 Bus 1 32-Byte 封包規範（`VedicEventPacket32`）

當使用者或代理在 App 中點擊「記錄事件」時，微秒級捕獲當下環境並封裝至 32-Byte 定長二進位封包：

```text
Offset (Bytes)   Field Name                   Type     Scale / Unit            Description
────────────────────────────────────────────────────────────────────────────────────────────────
00 .. 07 (8B)    ts_us                        u64      1 us (Unix Epoch UTC)   精確事件發生時間戳記
08 .. 11 (4B)    lat_milli_deg                i32      10^-3 deg (-90k..90k)   事件發生地緯度 (南緯為負)
12 .. 15 (4B)    lon_milli_deg                i32      10^-3 deg (-180k..180k) 事件發生地經度 (西經為負)
16 .. 19 (4B)    ayanamsha_arcsec             u32      1 arcsec (0..1296000)   當下採用的歲差值 (Lahiri J2000=85871")
20 .. 23 (4B)    dasha_packed                 u32      Bitpacked Dasha         當下大運/小運緊湊編碼
24 .. 27 (4B)    transit_summary_hash         u32      Compact Hash            九大行星過運星座與逆行狀態摘要雜湊
28 .. 29 (2B)    event_category_code          u16      0..65535                事件類別 (職業/財務/健康/婚姻/靈修)
30 .. 31 (2B)    flags                        u16      Bitmask 0x0001..0x8000  返照盤/生前事件/BTR校正標誌
────────────────────────────────────────────────────────────────────────────────────────────────
總長度 (Total): 8 + 4 + 4 + 4 + 4 + 4 + 2 + 2 = 32 Bytes (Little-Endian, Cache-Line Aligned)
```

- **旗標位元定義（`flags: u16`）**：
  - `0x0001`: `FLAG_SOLAR_RETURN`（太陽返照盤事件）
  - `0x0002`: `FLAG_PRE_NATAL_EVENT`（**產前生前事件，代表 dasha_packed 為逆算大限**）
  - `0x0004`: `FLAG_BTR_CALIBRATED`（**生時校正已標定完成**）
  - `0x0008`: `FLAG_LUNAR_RETURN`（月亮返照盤事件）
- **運限打包格式（`dasha_packed: u32`）**：
  - Bit 00..03: Mahadasha (大運 1..9)
  - Bit 04..07: Antardasha (小運 1..9)
  - Bit 08..11: Pratyantardasha (次小運 1..9)
  - Bit 12..15: Sookshmadasha (微運 1..9)
  - Bit 16..19: Pranadasha (呼吸運 1..9)
  - Bit 20..31: Balance Remainder (運限千分比或餘額 0..4095)

---

## 五、歷史極端應用處理指南

### 1. 日光節約時間（DST）公理與 1900 年防呆查表
- **正負號數學公理**：
  夏令時間將時鐘人為撥快 1 小時（Clock = 12:00，實際平太陽時為 11:00）。
  $$\text{UTC} = \text{Clock Time} - \Delta T_{\text{dst}} - \text{Timezone Offset}$$
  **DST 在換算時永遠是「負向修正項（從時鐘時間扣除）」！絕不能寫成加號！**
- **1900 年 7 月 5 日歷史真相**：
  全世界首個立法實施 DST 的國家是德國（1916 年 4 月 30 日）。**1900 年全世界任何主權國家都尚未實施 DST**。
  若有人輸入 1900 年夏令時，調用 `astro_validate_historical_dst` 會剛性攔截回傳 `DST_RESULT_MISMATCH_DST_NOT_ALLOWED`（代碼 3）。

### 2. 生前事件與大限逆算（Pre-natal Dasha Reverse Progression）
- **逆算物理原理**：
  出生時間 $T_{\text{birth}}$ 決定了大限起運餘量。對於出生前發生的事件（$T_{\text{event}} < T_{\text{birth}}$），其倒退時間差為 $\Delta t = T_{\text{birth}} - T_{\text{event}}$。
  Vimshottari 120 年九曜循環依照**逆時針方向回溯**：
  $$\text{Sun (6y)} \leftarrow \text{Moon (10y)} \leftarrow \text{Mars (7y)} \leftarrow \text{Rahu (18y)} \leftarrow \text{Jupiter (16y)} \leftarrow \text{Saturn (19y)} \leftarrow \text{Mercury (17y)} \leftarrow \text{Ketu (7y)} \leftarrow \text{Venus (20y)} \leftarrow \text{Sun (6y)}...$$
- 調用 `astro_vimshottari_reverse_step`，純整數輸出產前大限星與剩餘天數。
- 搭配細分盤：**D9（九分盤，父母婚姻）** 與 **D12（十二分盤，父母祖輩、祖業興衰）** 進行生時校正（BTR）。

### 3. 超高階分盤（D81, D108, D144）的幾何整除奇蹟
在角秒刻度（$360^\circ = 1,296,000''$，1宮 $= 108,000''$）下，分盤刻度與 27 星宿的 Pada（$3^\circ 20' = 12,000''$）呈現**完美整數整除**：
- **$D_{108}$（$12 \times 9$）**：每格剛好整整 **$1,000''$ 角秒**！1 個 Pada 剛好容納整整 **12 格**！
- **$D_{144}$（$12 \times 12$）**：每格剛好整整 **$750''$ 角秒**！1 個 Pada 剛好容納整整 **16 格**！
- **$D_{81}$（$9 \times 9$）**：全天球 $1,296,000'' \div 81 = 16,000''$ 整除！1 個 Pada 剛好容納整整 **9 格**！

### 4. $D_{300}$ 臨界歧度（Sandhi）的 300 倍誤差放大與物理真相
- **300 倍放大效應**：$D_{300}$ 盤上距邊界 $0.95''$（如水星入 $29^\circ 59' 59.05''$），倒推回真實天空只有 **$3.17$ 毫角秒（mas）**！
- **站心視差（Topocentric Parallax）碾壓**：
  水星在近地點對地球觀測點的站心視差高達 **$16''$ 角秒**。
  $16''$ 在 $D_{300}$ 盤上被放大 300 倍為：
  $$16'' \times 300 = 4,800'' = \mathbf{1^\circ 20' 00''}\text{（整整偏差 1.33 度！）}$$
- **工程結論**：
  若在 $D_{300}$ 盤看星宿歧度，**必須開啟站心視差修正（Topocentric Correction）**，且資料格式必須切換至**毫角秒（`mas: u64`，1宮 $= 108,000,000\text{ mas}$）**，以純整數避免截斷！

### 5. 千年古人星盤（成吉思汗、孔子、摩訶婆羅多）長跨度與 $\Delta T$ 物理誤差極限
當使用者或 AI 代理嘗試排定數百至數千年前古人星盤時，必須嚴格區分**「代碼計算誤差」**與**「真實天文物理誤差」**：

1. **代碼計算誤差（零飄移）**：
   - 本架構採用 `i64` Unix 秒（Bus 2/3）與純整數定點數（毫度/角秒/毫角秒），對於任何西元前歷史時間（如孔子生年 $-551$ 年、摩訶婆羅多時代 $-3102$ 年），**電腦二進位數值運算誤差為絕對的 $0.000$**，絕無浮點數累積截斷或下溢。
2. **真實天文物理誤差（潮汐減速與 $\Delta T$ 拋物線放大）**：
   - 地球自轉受月球潮汐摩擦影響持續減速，使得「平太陽日之世界時（UT）」與「均勻流逝的力學時/地球時（TT/TDB）」產生隨時間平方增長的內在時差：
     $$\Delta T = \text{TT} - \text{UT} \approx -20 + 32 \times \left(\frac{\text{Year} - 1820}{100}\right)^2 \quad (\text{秒})$$
   - **三大歷史代表人物星盤物理誤差對照表**：
     | 歷史人物 / 紀元 | 距今時間 | 估算 $\Delta T$ 時差 | 月球經度誤差 ($\sim 0.55^\circ/\text{hr}$) | 太陽/外行星誤差 ($\sim 0.04^\circ/\text{day}$) | 吠陀分盤與宮位影響評估 |
     |---|---|---|---|---|---|
     | **成吉思汗 (1162 CE)** | 約 860 年 | 約 **$15$ 分鐘** | 約 **$0.14^\circ$** ($8'$ 角分) | $< 0.0005^\circ$ (完全忽略) | 本命盤 (D1)、九分盤 (D9) 100% 穩定；上升點微偏 $3^\circ$ |
     | **孔子 (551 BCE)** | 約 2,570 年 | 約 **$5.5$ 小時** | 約 **$3.0^\circ$** (可跨半個 Pada) | $\le 0.01^\circ$ (外行星仍極準) | **月亮宿度 (Nakshatra) 可能跨界**；生時校正 (BTR) 需重定上升點 |
     | **摩訶婆羅多 (3102 BCE)** | 約 5,120 年 | 約 **$20 \sim 24$ 小時** | 約 **$11^\circ \sim 13^\circ$** (整整跨一個星座) | 太陽 $\sim 1^\circ$；土木 $\le 0.1^\circ$ | **嚴禁單憑現代反推月亮與上升點！** 必須利用古籍記載之「七星連珠/五星會聚」天象反向校準 $\Delta T$ 與曆元 |
3. **工程落地建議**：
   - 記錄古人事件時，封包之 `flags` 必須保留標注「真實日曆時（UT）」或「天體星曆力學時（TT/TDB）」。
   - 若使用者輸入古籍觀測之天象（如日月食日、行星合相），可反向調用 BTR 殘差最小化函式，擬合出當下時代的最佳分點歲差（Ayanamsha）與 $\Delta T$ 實測補償值。

---

## 六、時空數位孿生四圖層（Photoshop Layer 架構）

在 3D 街景、VR 虛擬實境與天文星空對齊中，嚴禁直接使用單一浮點數坐標，採用四大圖層嚴格分離：

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ [Layer 0] 天文慣性系 (ICRF / BCRS)                                           │
│   • 零點：太陽系質心 (SSB) 或 地心慣性系 (GCRS)                              │
│   • 職責：星曆真值、行星開普勒軌道計算                                      │
└──────────────────────────────────────▲───────────────────────────────────────┘
                                       │ 經過歲差、章動、地球自轉角 (ERA)
┌──────────────────────────────────────┴───────────────────────────────────────┐
│ [Layer 1] 地固座標系 (ECEF / WGS-84)                                         │
│   • 零點：地球質心 (Center of Mass)                                          │
│   • 單位：微米 (10^-6 m, i64)                                               │
│   • 職責：全球大尺度經緯高 (LLA) 轉換                                       │
└──────────────────────────────────────▲───────────────────────────────────────┘
                                       │ 經過局部切平面轉換 (Topocentric Tangent Plane)
┌──────────────────────────────────────┴───────────────────────────────────────┐
│ [Layer 2] 局部站心座標系 (ENU: East-North-Up)                                │
│   • 零點：角色「腳下地面接觸點」（浮動原點 Floating Origin）                 │
│   • 單位：毫米 (Millimeters, i32)                                            │
│   • 職責：消除 32-bit 浮點抽搐，周邊 10 公里內街景建築與物理碰撞精確無抖動  │
└──────────────────────────────────────▲───────────────────────────────────────┘
                                       │ 經過視角旋轉矩陣 (Yaw / Pitch / Roll)
┌──────────────────────────────────────┴───────────────────────────────────────┐
│ [Layer 3] 相機與視網膜視角 (Camera / View Frame)                             │
│   • 零點：鏡頭光心或角色雙眼                                                │
│   • 職責：天體經站心視差修正後投影繪入星空球                                │
└──────────────────────────────────────────────────────────────────────────────┘
```

- **愛因斯坦相對論時間膨脹**：
  載具高速運動（$v$）或高度差（$h$）產生的原時時間漂移，以**皮秒（$10^{-12}\text{ s}$，`dtau_ps: i64`）**整數累計：
  $$\Delta \tau = \int \left( 1 - \frac{gh}{c^2} - \frac{v^2}{2c^2} \right) dt$$

---

## 七、純 Rust 微積木 API 速查手冊

所有積木均收錄於 [`rust-bricks`](https://github.com/mangala1728/rust-bricks.git)，零外部依賴、純 Rust `std` 實作：

### 1. 引用方式 (`Cargo.toml`)
```toml
[dependencies]
rust-bricks = { git = "https://github.com/mangala1728/rust-bricks.git" }
```

### 2. 常用子函數簽章與範例

#### ① 歷史夏令時檢驗與鐘面轉 UTC (`src/astro_historical_dst.rs`)
```rust
use rust_bricks::astro_historical_dst::*;

// 1. 鐘面轉 UTC (負向扣除 DST)
let utc_sec = astro_clock_to_utc_seconds(clock_epoch_sec, -300 /* UTC-5 */, 60 /* DST +1h */).unwrap();

// 2. 歷史法規查表防呆 (1900 年美國宣稱有 DST -> 攔截)
let check = astro_validate_historical_dst(840 /* USA */, 1900, 7, 5, 60).unwrap();
assert_eq!(check, DST_RESULT_MISMATCH_DST_NOT_ALLOWED); // 代碼 3
```

#### ② 產前大限逆算與細分盤校正 (`src/astro_prenatal_dasha.rs`)
```rust
use rust_bricks::astro_prenatal_dasha::*;

// 出生於 Sun (3) 大運剩餘 1000 天，倒推生前 1500 天父母結婚之產前大限
let (maha, rem_days) = astro_vimshottari_reverse_step(3, 1000, 1500).unwrap();
assert_eq!(maha, 2); // Venus 大運

// 查詢金牛座 15度 (45,000 milli-deg) 在 D9 與 D12 的落入宮位
let d9_sign = astro_btr_varga_subdivision(45_000, 9).unwrap();   // 2 (Taurus)
let d12_sign = astro_btr_varga_subdivision(45_000, 12).unwrap(); // 8 (Scorpio)
```

#### ③ 超高階分盤角秒純整數求解 (`src/astro_varga_super.rs`)
```rust
use rust_bricks::astro_varga_super::*;

// D108 純整數 1000 角秒解算
let (sign, sub_idx, permille) = astro_super_varga_arcsec(1_000 /* 1000" */, 108).unwrap();
assert_eq!((sign, sub_idx, permille), (2, 1, 0));

// 查詢 1 個 Pada 包含多少個高階分盤步進
assert_eq!(astro_pada_to_super_varga_steps(108), Some(12)); // 剛好 12 步！
```

#### ④ D300 臨界歧度與毫角秒分析 (`src/astro_varga_cusp_analysis.rs`)
```rust
use rust_bricks::astro_varga_cusp_analysis::*;

// 毫角秒輸入 (1度 = 3,600,000 mas)，分析水星距邊界距離
let (sign, dist_mas, risk) = astro_d300_cusp_analysis_mas(physical_mas).unwrap();
if risk == CUSP_ZONE_CRITICAL {
    // 距邊界 < 1"，必須開啟站心視差校正！
}
```

#### ⑤ 32 位元組高速封包編解碼 (`src/astro_pipeline.rs`)
```rust
use rust_bricks::astro_pipeline::*;

let pkt = VedicEventPacket32 {
    ts_us: 1_700_000_000_987_654,
    lat_milli_deg: 25_033,       // 25.033° N
    lon_milli_deg: 121_565,      // 121.565° E
    ayanamsha_arcsec: 85_871,    // Lahiri J2000
    dasha_packed: 0x0001_0703,   // Sun-Jup-Ketu
    transit_summary_hash: 0xABCD_1234,
    event_category_code: 101,    // Milestone
    flags: FLAG_PRE_NATAL_EVENT | FLAG_BTR_CALIBRATED,
};
assert_eq!(std::mem::size_of::<VedicEventPacket32>(), 32);

let raw_bytes = encode_vedic_event_packet_32(&pkt); // 32 Bytes
let decoded = decode_vedic_event_packet_32(&raw_bytes).unwrap();
```

---

## 八、結語與維護契約

1. **單一真實來源**：下水道標準總綱維護於 `digital-historian/portable/domain-pipe-standards`，微積木實作在庫 `rust-bricks`。
2. **自動同步腳本**：任何規範微調，於 `rust-bricks` 執行 `python scripts/sync_domain_standards.py --sync` 即可秒級同步與校驗 SHA-256。
3. **品質保證**：全庫符合 NASA JPL $\le 60$ 行、09 合約無全域狀態、14 大工程鐵律，並經由 `brick-guard` 與雙模型聯署驗收收據保護。
