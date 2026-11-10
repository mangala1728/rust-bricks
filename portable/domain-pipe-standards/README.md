# 領域管線標準與單位格式便攜包 (Domain Pipe & Wire Standards Portable Kit)

```text
## Map
┌──────────────────────────────────────────────────────────────────────────────┐
│                    外部領域（天文／星曆／占星術／物理感測器）                  │
│       例如：NASA JPL HORIZONS / Swiss Ephemeris / NOAA 太陽黑子地磁           │
└───────────────────────────────────────▲──────────────────────────────────────┘
                                        │ 遵守本便攜包規格（定點純整數、雙時態、32B）
┌───────────────────────────────────────┴──────────────────────────────────────┐
│                  數位史官底層下水道四大標準匯流排架構                          │
│                                                                              │
│  [Bus 1] 高速二進位事件磁軌 (32B 定長)   ───> CelestialPacket32 (小端序)       │
│  [Bus 2] 雙時態歷史地層 TSV (因果無偷看) ───> EphemerisStratigraphyRow         │
│  [Bus 3] 因果知識圖譜與正交面向遮罩      ───> FACET_ASTRO_CELESTIAL (1<<6)     │
│  [Bus 4] 量化時空向量空間 (純整數維度)   ───> AstroPhysicsDimensionVector      │
└──────────────────────────────────────────────────────────────────────────────┘
```

本資料夾為專門設計的**獨立便攜包（Portable Kit）**，用於定義任何外部領域（如天文物理、星曆表推算、占星週期、幾何相位、太陽黑子與地磁風暴等）對接進本系統時必須遵守的**單位格式、定點數精度、二進位排佈、雙時態因果律及 09 號合約守門規則**。

---

## 一、便攜包檔案目錄清單

| 檔案名稱 | 類型 | 說明 |
|---|---|---|
| [`01_SEWER_PIPE_RULES.md`](file:///c:/For%20Gemini/digital-historian/standards/domain-pipe-standards/01_SEWER_PIPE_RULES.md) | 規範指南 | **下水道管線十條鐵律**（純整數化、定長 32B、雙時態阻絕未來偷看、09 號合約約束） |
| [`02_UNITS_AND_PRECISION_SPEC.json`](file:///c:/For%20Gemini/digital-historian/standards/domain-pipe-standards/02_UNITS_AND_PRECISION_SPEC.json) | 機器可讀定義 | **全域單位與精度綱要**（包含毫度、天文單位 fp4、基點、天體代碼、相位代碼與旗標遮罩） |
| [`03_BUS_ALIGNMENT_MATRIX.md`](file:///c:/For%20Gemini/digital-historian/portable/domain-pipe-standards/03_BUS_ALIGNMENT_MATRIX.md) | 對接矩陣 | **四條標準匯流排對接矩陣**（Bus 1~4 欄位精確佈局、TSV 表頭、圖譜遮罩、特徵向量） |
| [`04_VEDIC_ASTRO_SPATIOTEMPORAL_EXT.md`](file:///c:/For%20Gemini/digital-historian/portable/domain-pipe-standards/04_VEDIC_ASTRO_SPATIOTEMPORAL_EXT.md) | 擴充標準 | **吠陀占星、五分曆、事件記錄器與時空數位孿生規範**（純整數五分曆算式、VedicEventPacket32、分層時空座標系、Ayanamsha/BTR 校準與跨倉同步協議） |
| [`astro_pipeline_template.rs`](file:///c:/For%20Gemini/digital-historian/portable/domain-pipe-standards/astro_pipeline_template.rs) | Rust 參考代碼 | **100% 零依賴參考實作**（含 32B 封包編解碼、毫度相位計算、TSV 雙時態因果守衛與單元測試） |
| [`test_astro_pipeline.py`](file:///c:/For%20Gemini/digital-historian/standards/domain-pipe-standards/test_astro_pipeline.py) | Python 驗證器 | **獨立測試腳本**（驗證二進位小端序 32 Bytes、因果不變量與 JSON Schema 正確性） |

---

## 二、新領域開發者必讀：四大關鍵下水道規則

如果你要開發一個天文或占星工具（例如計算水星逆行、日月食、土木大合相、滿月潮汐）：

### 1. 嚴禁任何浮點數進入管線（Pure Integer Fixed-Point Only）
- ❌ **錯誤做法**：`let longitude: f64 = 142.3857;`（跨平台浮點捨入誤差、NaN 隱患）。
- ✅ **標準下水道做法**：
  - 使用**毫度（Milli-Degrees, $10^{-3}$ deg）**：`let lon_milli_deg: u32 = 142385;`（範圍 `0 .. 360000`）。
  - 相位容許度（Orb）使用**基點（Basis Points, bp）**：$1.5^\circ \to 150\text{ bp}$。
  - 天體距離使用天文單位定點數 `AU_fp4`：$1.0000\text{ AU} \to 10000$。

### 2. 時間戳記純粹化（Strict Monotonic Epoch）
- ❌ **錯誤做法**：傳遞 `"2026-11-11T00:53:15+08:00"` 或本地時間字串。
- ✅ **標準下水道做法**：
  - 高速串流（Bus 1）：Unix 微秒（`ts_us: u64`），從 1970-01-01 UTC 起算。
  - 歷史地層（Bus 2）：Unix 秒（`ts_sec: u64`），配合 `date_yyyymmdd: u32`（如 `20261111`）。

### 3. 雙時態因果律：絕不允許「未來星曆偷看」（Bitemporal Anti-Lookahead）
- 天文學具備可預知性（例如 2030 年日全蝕時間在 100 年前已知）。
- 但在歷史回測或因果圖譜中，必須嚴格區分：
  - `event_period_ts`：天體交角在宇宙中真實發生的物理時間。
  - `public_known_ts`：該星曆數據對外部世界正式發布已知的時間。
- 查詢守門公式：
  $$\text{event\_period\_ts} \le \text{public\_known\_ts} \le \text{sim\_clock\_ts}$$
  在當前模擬時鐘 `sim_clock_ts` 之前未公開的數據，嚴禁被模型讀取！

### 4. 09 號合約剛性約束（09-Contract Boundary）
- 每個檔案頂部必須附帶 `//! ## Map`。
- 每個函式必須標注 `/// @outputs: N`（$1 \le N \le 32$）。
- 每個數值參數必須標注 `/// @domain: param [min..max]`，並在函式開頭第一行進行邊界檢查：
  ```rust
  if lon_milli_deg > 360_000 { return None; }
  ```
- 零全域可變狀態（禁止 `static mut`），禁止遞迴，單一函式行數 $\le 100$ 行，巢狀深度 $\le 2$。

---

## 三、如何驗證你的工具是否對齊？

1. **執行 Python 格式與封包驗證**：
   ```bash
   python standards/domain-pipe-standards/test_astro_pipeline.py
   ```
2. **執行 Rust 09 號合約靜態檢查**：
   ```bash
   python "c:/For Gemini/工作流/scripts/rust-lint.py" standards/domain-pipe-standards
   ```
3. **執行單元測試**：
   所有測試皆內建於 `astro_pipeline_template.rs`，可直接併入專案執行 `cargo test`。
