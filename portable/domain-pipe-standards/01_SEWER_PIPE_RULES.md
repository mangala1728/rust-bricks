# 下水道管線十條鐵律 (The 10 Iron Sewer Pipe Rules)

任何新領域工具（包括天文物理學、星曆表、天體週期、占星術相位、地磁觀測等）要對接進本系統，必須 100% 遵守以下十條底層管線規則。

---

## 規則 1：純整數定點數化（Pure Integer Fixed-Point Math Only）
- **痛點**：IEEE-754 浮點數（`f32`/`f64`）在跨語言（Python/Rust/C）、跨平台（x86/ARM/GPU）與跨進程傳輸時，存在尾數截斷誤差、NaN、負零以及非確定性，嚴禁直接進入二進位匯流排。
- **剛性規範**：
  - **角度／經緯度**：統一採用**毫度（Milli-Degrees, $10^{-3}$ deg）**，範圍 `0 .. 360_000`（`u32`）。
    - 範例：$180.5^\circ \to 180500$ 毫度；黃道十二宮每宮為 $30^\circ = 30000$ 毫度。
  - **天體距離**：採用天文單位定點數 `AU_fp4`（$10^{-4}\text{ AU}$，`u32`）。
    - 範例：地日平均距離 $1.0000\text{ AU} \to 10000$。
  - **相位容許度（Orb）**：採用**基點（Basis Points, bp）**或毫度，$1\text{ bp} = 0.01^\circ$。
    - 範例：容許度 $1.5^\circ = 150\text{ bp} = 1500\text{ milli-deg}$。
  - **週期比率／相角**：統一為百分比整數 `0 .. 100` 或定點整數 `0 .. 10_000`。

---

## 規則 2：時間戳記絕對整數化（Strict Integer Timestamp）
- **痛點**：ISO 8601 字串（如 `"2026-11-11T00:53:15Z"`）或帶時區的字串在微秒級高速處理時造成嚴重的字串解析與記憶體負擔。
- **剛性規範**：
  - **高速串流（Bus 1）**：一律使用 **Unix Epoch 微秒（`ts_us: u64`）**，UTC 基準，嚴禁本地時區。
  - **地層資料（Bus 2）**：一律使用 **Unix Epoch 秒（`ts_sec: u64`）** 或微秒。
  - **日曆快照**：一律使用 `trading_date_yyyymmdd: u32`（如 `20261111`）。

---

## 規則 3：Bus 1 封包嚴格定長 32 位元組（Strict 32-Byte Packet）
- **痛點**：可變長度二進位封包會引發記憶體碎片化與 cache-miss。
- **剛性規範**：
  - Bus 1 封包大小必須嚴格等於 32 Bytes（剛好填滿 CPU L1 Cache Line 64 Bytes 的半槽）。
  - 封包內部嚴禁包含指標、堆疊記憶體引用（`Vec` / `String`）、動態陣列。
  - 欄位序列化必須遵循小端序（Little-Endian）。
  - 代碼中必須具備編譯期或測試期長度斷言：`assert_eq!(std::mem::size_of::<CelestialPacket32>(), 32);`。

---

## 規則 4：雙時態因果律與阻絕未來窺探（Bitemporal Causality & Anti-Lookahead）
- **痛點**：天文曆法具備高度未來可預測性（例如現在已知 2030 年日蝕或水星逆行精確時間）。若回測模型在 2024 年時段讀取了當時尚未公開發布的曆表或修正數據，會造成嚴重的「未來窺探漂移（Lookahead Leak）」。
- **剛性規範**：
  - 所有地層記錄必須區分雙時間戳記：
    1. `event_period_ts`：天體物理事件實際發生的時間。
    2. `public_known_ts`：該星曆觀測報告、排程或預報對外界系統**正式公開且已知**的時間。
  - **地層守衛不變量**：
    $$\text{event\_period\_ts} \le \text{public\_known\_ts} \le \text{sim\_clock\_ts}$$
    模擬時鐘（`sim_clock_ts`）之前未公開發布的星曆，在下水道查詢中一律強制遮蔽、拒絕讀取。

---

## 規則 5：多面向事件正交遮罩（Orthogonal Facet Masking）
- **痛點**：傳統標籤採用字串列表，查詢時需耗費大量字串匹配。
- **剛性規範**：
  - Bus 3 事件知識圖譜採用 16 位元或 32 位元正交位元遮罩（Bitmask）。
  - 天文／周期領域指定註冊專屬遮罩位元：
    ```rust
    pub const FACET_ASTRO_CELESTIAL: u16 = 1 << 6; // 0x0040 天體相位與星象事件
    pub const FACET_SOLAR_GEOMAGNETIC: u16 = 1 << 7; // 0x0080 太陽黑子與地磁風暴
    ```
  - 事件稀有度評分（`rarity_score`）與因果強度（`edge_value_score`）一律為 1 到 5 星整數。

---

## 規則 6：量化向量空間維度正規化（Bus 4 Vector Normalization）
- **痛點**：量化模型（DTW 距離計算、跳躍擴散蒙地卡羅）需要固定長度、數值有界的特徵陣列。
- **剛性規範**：
  - 進入 Bus 4 的天體特徵必須壓縮為固定維度純整數向量（例如 `[i32; 32]` 或專用 Struct）：
    - 月相週期相角：`0 .. 100`（0=新月, 50=滿月, 100=次一新月）。
    - 行星運動狀態：`0=Direct(順行), 1=Retrograde(逆行), 2=Stationary(留)`。
    - 黃道星座區間：`1 .. 12`（牡羊到雙魚）。
    - 總體天象衝合相位分值：`dim_aspect_power_bp: i16`（-10000 .. +10000）。

---

## 規則 7：09 號合約邊界約束（09-Contract Boundary Enforcement）
- **剛性規範**：
  - **檔案接線圖**：程式檔案開頭必須附帶 `//! ## Map`，標明 Role、Inputs、Outputs、Invariants。
  - **輸出狀態上界**：具名函式上方必須標注 `/// @outputs: N`（$1 \le N \le 32$）。
  - **定義域與守衛**：數值參數上方必須標注 `/// @domain: param [min..max]`，並在函式開頭第一行實作執行期守衛：
    ```rust
    if param < min || param > max { return None; }
    ```
  - **結構限制**：單一函式行數 $\le 100$ 行，控制流巢狀深度 $\le 2$。

---

## 規則 8：零全域可變狀態（Zero Mutable Global State）
- **剛性規範**：
  - 嚴禁使用 `static mut`、模組層級全域變數、或全域快取單例。
  - 所有計算核心必須為純函式（Pure Function）或由外層明確傳入狀態參照（`&mut Context`）。

---

## 規則 9：無遞迴與顯式有界迴圈（No Recursion & Bounded Loops）
- **剛性規範**：
  - 嚴禁任何形式的自我呼叫或間接遞迴（`A -> B -> A`）。
  - 天象歷表搜尋、開普勒軌道迭代、相位夾角計算必須具備明確的常數上限：
    ```rust
    const MAX_KEPLER_ITER: usize = 64;
    while iter < MAX_KEPLER_ITER {
        // ...
        iter += 1;
    }
    ```

---

## 規則 10：三層架構與 Fail-Closed 拒絕機制（Three-Tier Architecture & Fail-Closed）
- **剛性規範**：
  - 工具實作依序支援三種存取層級：
    1. `InProcessFn`：Rust 原生零拷貝函式。
    2. `HttpOrGrpcApi`：微服務網絡管線。
    3. `McpServerTool`：JSON-RPC 2.0 stdio 標準工具介面。
  - 任何參數越界、欄位缺漏或因果衝突時，一律採取 **Fail-Closed（失敗關閉）** 策略，回傳安全預設值或錯誤代碼，絕不允許 panic 或讓髒資料流入下一級管線。
