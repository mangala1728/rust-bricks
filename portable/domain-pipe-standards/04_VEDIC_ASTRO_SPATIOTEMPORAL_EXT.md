# 吠陀占星、五分曆、事件日誌與時空數位孿生擴充規範
# (Vedic Astrology, Panchanga, Life Event Recorder & Spatiotemporal Digital Twin Specification)

```text
## Map
┌──────────────────────────────────────────────────────────────────────────────┐
│                    上層應用：吠陀占星、人生事件記錄器、數位孿生              │
│       (Web/Mobile/Desktop App, Vimshottari Dasha, VR Celestial Sky)          │
└──────────────────────────────────────▲───────────────────────────────────────┘
                                       │ 32B 定長二進位封包 / 純整數定點數 / 分層時空
┌──────────────────────────────────────┴───────────────────────────────────────┐
│                       四大標準匯流排與下水道延伸介面                         │
│                                                                              │
│  [Bus 1] 高速事件磁軌 (32B)  ───> VedicEventPacket32 (時間+經緯+運限+過運)   │
│  [Bus 2] 雙時態歷史地層 TSV  ───> PanchangaStratigraphyRow (五分曆+日出落)  │
│  [Bus 3] 因果事件知識圖譜    ───> 歲差微調校準 (BTR) 與大宗歷史事件因果邊    │
│  [Bus 4] 分層時空數位孿生    ───> ICRF -> ECEF -> ENU -> 相機層 (浮點防抖)   │
└──────────────────────────────────────┬───────────────────────────────────────┘
                                       │
┌──────────────────────────────────────▼───────────────────────────────────────┐
│                      跨倉庫架構權威與同步機制                                 │
│  權威主幹 (Upstream Master): digital-historian/portable/domain-pipe-standards │
│  純 Rust 引擎 (Downstream):  rust-bricks/portable/domain-pipe-standards       │
└──────────────────────────────────────────────────────────────────────────────┘
```

本規範為 `domain-pipe-standards` 便攜包之第 04 號擴充標準，依據使用者未來發展「吠陀占星軟體（Web/Desktop/Mobile）」、「人生重大事件記錄器（Life Event Recorder）」、「分點歲差校準方法學（Ayanamsha Calibration & BTR）」以及「高精度時空數位孿生（Spatiotemporal Digital Twin）」之需求，制定跨語言、跨平台的資料結構、純整數算術公式、二進位排佈與跨倉庫治理協議。

---

## 一、純整數吠陀五分曆算術規範（Pure Integer Panchanga Math）

吠陀占星與印度五分曆（Panchanga）計算的核心在於**太陽（Surya）**與**月亮（Chandra）**的黃道經度關係。為杜絕 IEEE-754 浮點數截斷誤差引發的「換日/換宮臨界點跳躍」，一律採用**毫度（Milli-Degrees, $10^{-3}$ deg）**與**角分（Arcminutes, $' = 1/60^\circ$）**之純整數運算。

### 1. 角度單位轉換基礎
- 黃道全週：$360^\circ = 360,000\text{ milli-deg} = 21,600\text{ arcmin}$。
- 毫度轉角分純整數公式（誤差為 0）：
  $$\text{arcmin} = \frac{\text{milli\_deg} \times 3}{50}$$
  （因為 $1\text{ milli-deg} = 0.001^\circ = 0.06' = \frac{6}{100}' = \frac{3}{50}'$）

### 2. 五分曆（Panchanga）五大要素算術定義

| 要素名稱 | 梵文名稱 | 物理定義與範圍 | 純整數計算公式（零浮點數） | 輸出定義域 |
|---|---|---|---|---|
| **第 (Tithi)** | Tithi | 月亮相對於太陽的離角，每 $12^\circ$（$12,000$ 毫度）為一第。 | $\Delta = (M_{\text{milli}} + 360000 - S_{\text{milli}}) \pmod{360000}$<br>$\text{Tithi} = \lfloor \Delta / 12000 \rfloor + 1$ | $1 .. 30$<br>(1..15 白月 Shukla,<br>16..30 黑月 Krishna) |
| **宿 (Nakshatra)** | Nakshatra | 月球所落之 27 宿，每宿跨度 $13^\circ 20' = 800'$ 角分。 | $M_{\text{min}} = (M_{\text{milli}} \times 3) / 50$<br>$\text{Nakshatra} = \lfloor M_{\text{min}} / 800 \rfloor + 1$ | $1 .. 27$<br>(1=Ashwini .. 27=Revati) |
| **足 (Pada)** | Pada | 每一宿細分為 4 個足，每足跨度 $3^\circ 20' = 200'$ 角分（對應 D9 九分盤 Navamsha）。 | $\text{Pada} = \lfloor (M_{\text{min}} \pmod{800}) / 200 \rfloor + 1$ | $1 .. 4$ |
| **曜 (Yoga)** | Yoga | 太陽與月球黃道經度之和，每 $13^\circ 20' = 800'$ 角分為一曜。 | $S_{\text{min}} = (S_{\text{milli}} \times 3) / 50$<br>$\text{Sum}_{\text{min}} = (S_{\text{min}} + M_{\text{min}}) \pmod{21600}$<br>$\text{Yoga} = \lfloor \text{Sum}_{\text{min}} / 800 \rfloor + 1$ | $1 .. 27$<br>(1=Vishkambha .. 27=Vaidhriti) |
| **關 (Karana)** | Karana | 半個 Tithi，每 $6^\circ$（$6,000$ 毫度）為一關。全月共有 60 個 Karana。 | $\text{Karana} = \lfloor \Delta / 6000 \rfloor + 1$ | $1 .. 60$<br>(含 7 種循環關 + 4 種固定關) |
| **曜 (Vara)** | Vara | 太陽日星期（以日出為一日之始，非午夜 00:00）。 | $\text{Vara} = (\lfloor (\text{ts\_sunrise\_sec} + 86400 \times 3) / 86400 \rfloor) \pmod 7$ | $0 .. 6$<br>(0=日 Ravivara .. 6=土 Shanivara) |

### 3. 特殊 Tithi 判定規則（Kshaya & Vriddhi Tithi）
在吠陀曆法中，當天的官方 Tithi 是以**當日日出（Sunrise）瞬間**正在生效的 Tithi 為準：
- **常態（Normal Tithi）**：連續兩個日出之間剛好跨越一個 Tithi 邊界。
- **缺日 / 虧日（Kshaya Tithi）**：某個 Tithi 在前一日日出之後開始，且在次日日出之前就已結束（即該 Tithi 的時間區間**完全沒有涵蓋到任何一次日出**），在民用曆法中被跳過。
- **增日（Vriddhi Tithi）**：由於月球運動較慢，同一個 Tithi 的時間跨度**涵蓋了連續兩次日出**，導致民用曆法中連續兩天擁有相同的 Tithi。
- **純整數判定守衛**：
  ```rust
  // t1: 該 Tithi 起始時間 (ts_sec)
  // t2: 該 Tithi 結束時間 (ts_sec)
  // sunrise_curr: 今日日出時間 (ts_sec)
  // sunrise_next: 次日日出時間 (ts_sec)
  pub fn is_kshaya_tithi(t1: u64, t2: u64, sunrise_curr: u64, sunrise_next: u64) -> bool {
      t1 > sunrise_curr && t2 < sunrise_next
  }
  ```

---

## 二、人生事件記錄器與 Bus 1 32-Byte 封包規範（`VedicEventPacket32`）

當使用者或代理觸發「人生事件記錄器（Life Event Recorder）」時（例如：遭遇重大決策、車禍、求職錄取、重大健康徵兆、金融投資），系統必須在微秒級瞬間捕獲當下的時空環境、運限與過運狀態，並封裝進嚴格 32 Bytes 的二進位封包。

### 1. `VedicEventPacket32` 記憶體佈局（嚴格 32 Bytes）

```text
Offset (Bytes)   Field Name                   Type     Scale / Unit            Description
────────────────────────────────────────────────────────────────────────────────────────────────
00 .. 07 (8B)    ts_us                        u64      1 us (Unix Epoch UTC)   精確事件發生時間戳記
08 .. 11 (4B)    lat_milli_deg                i32      10^-3 deg (-90000..90000) 事件發生地緯度 (南緯為負)
12 .. 15 (4B)    lon_milli_deg                i32      10^-3 deg (-180k..180k)  事件發生地經度 (西經為負)
16 .. 19 (4B)    ayanamsha_arcsec             u32      1 arcsec (0..1296000)   當下採用的歲差值 (Lahiri J2000=85871")
20 .. 23 (4B)    dasha_packed                 u32      Bitpacked Dasha         當下大運/小運緊湊編碼 (詳見下方)
24 .. 27 (4B)    transit_summary_hash         u32      Compact Hash            九大行星過運星座與逆行狀態摘要雜湊
28 .. 29 (2B)    event_category_code          u16      0..65535                事件類別 (職業/財務/健康/婚姻/靈修)
30 .. 31 (2B)    flags                        u16      Bitmask 0x0001..0x8000  返照盤旗標/驗證等級/私密性遮罩
────────────────────────────────────────────────────────────────────────────────────────────────
總長度 (Total): 8 + 4 + 4 + 4 + 4 + 4 + 2 + 2 = 32 Bytes (Little-Endian, Cache-Line Aligned)
```

### 2. Vimshottari 運限緊湊位元打包（`dasha_packed: u32`）
吠陀 Vimshottari Dasha 系統以 120 年為一循環，九大行星代碼為：
`1=Ketu, 2=Venus, 3=Sun, 4=Moon, 5=Mars, 6=Rahu, 7=Jupiter, 8=Saturn, 9=Mercury`（範圍 `1..9`，可用 4 位元表示 `0..15`）：

```text
Bit 00 .. 03 (4 bits): Mahadasha (大運, 1..9)
Bit 04 .. 07 (4 bits): Antardasha (小運, 1..9)
Bit 08 .. 11 (4 bits): Pratyantardasha (次小運, 1..9)
Bit 12 .. 15 (4 bits): Sookshmadasha (微運, 1..9)
Bit 16 .. 19 (4 bits): Pranadasha (呼吸運, 1..9)
Bit 20 .. 31 (12 bits): Balance Remainder / Sub-level (運限剩餘比例或千分比 0..4095)
```

### 3. 過運狀態摘要雜湊（`transit_summary_hash: u32`）
九大曜（Sun, Moon, Mars, Mercury, Jupiter, Venus, Saturn, Rahu, Ketu）當下所在的黃道十二宮（`1..12`，每個宮位 4 位元，共 $9 \times 4 = 36$ 位元；若僅取核心 7 曜 + Rahu 共 8 曜，剛好 $8 \times 4 = 32$ 位元；或以 32-bit FNV-1a 雜湊保存全星盤過運指紋）。

---

## 三、分層時空數位孿生與座標參考系規範（Spatiotemporal Frames & Digital Twin）

在時空數位孿生系統中，為避免將「地心重力」、「觀測者腳下」、「飛行器高速位移」及「廣義相對論時間膨脹」混為一談，採用類似圖像軟體「圖層（Layer）」之嚴格分層轉換架構。

### 1. 四大標準時空參考系圖層（Coordinate Reference Layers）

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│ [Layer 0] 天文慣性系 (ICRF / BCRS)                                           │
│   • 零點 (Origin)：太陽系質心 (SSB) 或 地心慣性系 (GCRS)                     │
│   • 軸向 (Axes)：國際天球參考架 (ICRF)，相對於遙遠類星體保持靜止             │
│   • 職責：恆星、行星、月球之開普勒軌道與星曆真值計算                         │
└──────────────────────────────────────▲───────────────────────────────────────┘
                                       │ 經過歲差 (Precession)、章動 (Nutation)、地球自轉角 (ERA)
┌──────────────────────────────────────┴───────────────────────────────────────┐
│ [Layer 1] 地固座標系 (ECEF / WGS-84)                                         │
│   • 零點 (Origin)：地球質心 (Earth Center of Mass)                           │
│   • 軸向 (Axes)：隨地球自轉，X 軸穿過本初子午線與赤道交點，Z 軸穿過北極點     │
│   • 單位：微米（Micrometers, $10^{-6}\text{ m}$，`i64`）或毫米               │
│   • 職責：全球經緯度（LLA）轉換與大尺度地表定位                              │
└──────────────────────────────────────▲───────────────────────────────────────┘
                                       │ 經過局部切平面旋轉矩陣 (Topocentric Tangent Plane)
┌──────────────────────────────────────┴───────────────────────────────────────┐
│ [Layer 2] 局部站心座標系 (ENU: East-North-Up / NED)                          │
│   • 零點 (Origin)：觀測者「腳下地表接觸點」（浮動原點 Floating Origin）       │
│   • 軸向 (Axes)：東 (X)、北 (Y)、天頂 Up (Z，垂直於重力橢球面)               │
│   • 單位：毫米（Millimeters, `i32`）                                         │
│   • 職責：角色周邊 10 公里內街景、建築物、VR 物件互動，徹底消滅 32-bit 浮點抖動│
└──────────────────────────────────────▲───────────────────────────────────────┘
                                       │ 經過視角旋轉矩陣 (Yaw / Pitch / Roll)
┌──────────────────────────────────────┴───────────────────────────────────────┐
│ [Layer 3] 相機與視網膜座標系 (Local Camera / View Frame)                     │
│   • 零點 (Origin)：角色雙眼或鏡頭光心                                        │
│   • 軸向 (Axes)：右 (X)、上 (Y)、視線前方 (Z)                                │
│   • 職責：星空天球投影渲染（將 Layer 0 天體經由 Topocentric 修正後繪入天空球） │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 2. 為什麼底層管線嚴禁直接使用單一浮點數座標？
- **32 位元浮點數（`f32`）的大尺度抖動**：
  - 地球半徑約 $6,378,137\text{ m}$。若以地心為原點使用 `f32`，其有效位數僅 24 bits（約 7 位十進位有效數字），在地球表面的解析度僅能達到 $\approx 1\text{ 米}$！角色在街景中移動會產生劇烈抽搐與物理碰撞穿模。
- **解決方案：浮動原點技術（Floating Origin）+ 整數毫米定點數**：
  - 遠程資料庫與串流傳輸：使用整數毫度經緯度（`lat_milli_deg`, `lon_milli_deg`）與公尺高程。
  - 近場物理引擎：以當前相機所在網格中心作為局部原點，相對位移使用純整數毫米（`local_x_mm: i32`）。

### 3. 相對論微小時間偏移（Relativistic Time Dilation）
當觀測者搭乘高速飛機（速度 $v$）或位於強重力場差（高度差 $h$）時，相對於地表靜止基準時鐘會產生狹義與廣義相對論時間漂移（愛因斯坦方程式）：
$$\frac{d\tau}{dt} \approx 1 - \frac{\Delta \Phi}{c^2} - \frac{v^2}{2c^2} = 1 - \frac{g h}{c^2} - \frac{v^2}{2c^2}$$
- 下水道管線規格統一採用**皮秒級整數偏移量（Picoseconds, $10^{-12}\text{ s}$，`dtau_ps: i64`）**：
  - 範例：GPS 衛星每日相對於地面快約 $+38\text{ 微秒} = +38,000,000\text{ ps}$。
  - 定義於 `02_UNITS_AND_PRECISION_SPEC.json` 之 `relativistic_drift_ps`。

---

## 四、分點歲差校準與個人生時校正方法學（Ayanamsha Calibration & BTR）

### 1. 恆星占星（Sidereal）與熱帶占星（Tropical）之橋樑：歲差（Ayanamsha）
- 熱帶黃道經度（$\lambda_{\text{tropical}}$）與恆星黃道經度（$\lambda_{\text{sidereal}}$）的轉換式為：
  $$\lambda_{\text{sidereal}} = (\lambda_{\text{tropical}} + 360000 - \text{Ayanamsha}) \pmod{360000}$$
- **主流歲差公版常數（J2000.0 基準 Epoch: 2000-01-01 12:00:00 TT）**：

| 歲差系統代碼 | 歲差系統名稱 (Ayanamsha) | J2000.0 角秒值 ($\text{arcsec}$) | 等效度數 (deg) | 核心依據 |
|---|---|---|---|---|
| `0` | **Lahiri (Chitra Paksha)** | $85,871''$ | $23^\circ 51' 11''$ | 印度官方曆法委員會公版，角宿一（Spica）固定於 $180^\circ$ 對面 |
| `1` | **Krishnamurti (KP)** | $85,410''$ | $23^\circ 43' 30''$ | KP 占星術子星限（Sub-Lord）系統標準 |
| `2` | **Raman (B.V. Raman)** | $77,934''$ | $21^\circ 38' 54''$ | 依據古籍 Surya Siddhanta 推算之公版 |
| `3` | **Fagan-Bradley** | $88,963''$ | $24^\circ 42' 43''$ | 西方恆星占星術公版，以 Aldebaran/Antares 為基準 |
| `4` | **Yukteshwar (聖尤地斯瓦爾)** | $77,990''$ | $21^\circ 39' 50''$ | 《神聖科學》（The Holy Science）年代學週期 |

### 2. 生時校正（Birth Time Rectification, BTR）與群體擬合方法學
1. **單人高階細分盤（Varga Charts）擬合**：
   - D1（本命盤，上升星座每 2 小時更換一次）。
   - D9（九分盤 Navamsha，每 13 分 20 秒更換一個 Navamsha 上升）。
   - D60（六十分盤 Shashtiamsa，每 2 分鐘更換一個上升）。
   - 當人生事件記錄器積累 $\ge 5$ 個重大事件（如結婚、重病、破產、出國、生子），系統透過事發時的 Vimshottari Dasha 運限主管星與過運星對宮位的觸發，逆推生時誤差範圍至 $\pm 30$ 秒以內。
2. **多人群體大數據反饋微調 Ayanamsha**：
   - 當數千位經過嚴密生時校正的個案資料彙整後，透過 Bus 3 因果邊統計分析，以梯度下降法最小化「事件發生時刻與 Dasha 運限交接點」的整體殘差，進而微調更精確的「經驗擬合分點歲差值（Empirical Calibrated Ayanamsha）」。

---

## 五、跨倉庫架構權威與同步機制（Cross-Repository Synchronization）

為確保本專案（`rust-bricks`）與上位系統（`digital-historian`）保持 100% 格式同構，且兩者皆能獨立編譯、獨立通過 CI 關卡，制定以下治理機制：

### 1. 單一真實來源（Single Source of Truth, SSOT）
- **規範權威主幹（Upstream Authority）**：
  `c:\For Gemini\digital-historian\portable\domain-pipe-standards`
  所有欄位定義、JSON Schema、下水道十條鐵律由此倉庫維護並簽核。
- **純 Rust 微積木實作庫（Downstream Pure-Rust Engine）**：
  `c:\For Gemini\rust-bricks\portable\domain-pipe-standards`
  作為鏡像副本，提供 `rust-bricks` 本地 CI、MSA 量具檢驗及雲端農夫（Cloud Farm）自動化造積木之依賴。

### 2. 同步工具與腳本
在 `rust-bricks` 中提供跨平台同步腳本 `scripts/sync-standards.py`：
- 自動比對兩邊檔案之 SHA-256 雜湊。
- 若 Upstream 有更新，一鍵同步鏡像至 Downstream 並驗證 `test_astro_pipeline.py` 通過。
- 嚴禁在代碼中以相對路徑 `../../` 穿透至外部資料夾編譯，確保兩倉庫可完全獨立於任何環境 clone 運作。
