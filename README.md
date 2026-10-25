# rust-bricks — 純淨零依賴 Rust 函式積木庫（09 合約規範）

[![Zero Dependencies](https://img.shields.io/badge/dependencies-0-brightgreen.svg)](#)
[![09 Contract Compliant](https://img.shields.io/badge/09--contract-100%25-blue.svg)](#)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#)

> **「不求快開發，一切以狀態可窮舉、邊界可靜態驗證、出事 3 分鐘內定位為最高原則。」**  
> 專為 Antigravity 高放權 AI 代理工作流、Android 跨端 (Kotlin JNI / NDK) 與邊緣節點打造的純 Rust 高性能函式積木庫。

---

## 一、核心設計哲學（Pillars）

1. **100% 零第三方依賴（Zero 3rd-party Dependencies）**
   - 拒絕 `cargo add` 隨機拉入數十個黑箱傳染性依賴（Transitive Dependencies）。
   - 僅依賴 Rust `core` / `alloc` / `std`，可秒級交叉編譯至 Windows、Linux、macOS、Android (`aarch64-linux-android`) 與 WebAssembly (`wasm32`)。
2. **全面落實 09 號合約邊界規範（Contract Boundary）**
   - 每個原始檔開頭必附 `## Map` 接線圖（Role / Inputs / Outputs / Invariants）。
   - 每個函式必標 `// @outputs: N`（輸出狀態基數上界）與 `// @domain: param [min..max]`。
   - **函式入口剛性實作邊界守衛**：越界立即收斂或回傳安全錯誤。
   - **零全域可變狀態（`R09-2`，Zero `static mut`）**、**禁止遞迴（`R09-7`，No Recursion）**、**有界迴圈（`R09-8`）**。
3. **Python 語意對照與資產複利（Rosetta Stone）**
   - 針對 Python 標準庫或常用演算法（如時間差、二分法、滑動視窗、音訊能量計算），提煉其純粹數學與演算法核心，以 Rust 嚴密語法重現，成為跨專案長期技術複利。

---

## 二、模組方塊圖（Module Map）

| 模組 | 職責 | 典型應用場景 |
|---|---|---|
| [`math_time`](src/math_time.rs) | 高精度數值夾止、滑動均線、儒略日曆法轉換、占星大限 (Dasha) 比例運算、XorShift64 | 股市量化、時間序列平滑、極速亂數生成 |
| [`audio_vad`](src/audio_vad.rs) | PCM16 音訊取樣點能量 (RMS) 計算、靜音幀判定、CPS 字幕字速審查 | 語音活動檢測 (VAD)、即時翻譯 App、多模態字幕對齊 |
| [`audio_dsp`](src/audio_dsp.rs) | 聲學功率、dBFS 能量、零交越率、Butterworth 高通濾波、語音預加重、軟削峰防破音 | 麥克風音訊前處理、即時降噪前置濾波、音訊串流 |
| [`audio_wav`](src/audio_wav.rs) | 零依賴 16-bit PCM WAV 檔頭解析與二進位編碼 | 音訊錄音檔存檔、跨平台 WAV 串流解析 |
| [`ring_buffer`](src/ring_buffer.rs) | 純記憶體無鎖固定容量 PCM16 環形緩衝區、批次讀寫、溢出守衛 | 手機麥克風串流 (Oboe/AudioRecord)、感測器事件佇列、消滅 GC 停頓 |
| [`sensor_motion`](src/sensor_motion.rs) | 陀螺儀/加速度計互補濾波姿態融合、1D 卡爾曼濾波、Haversine GPS 大圓距離與航向角 | 手機小遊戲體感控制、防手震、GPS 經緯度去噪與定位 |
| [`vision_yuv`](src/vision_yuv.rs) | Android Camera2 (NV21 / YUV420sp) 預覽幀灰階快速抽取、2x2 平均降採樣、亮度直方圖 | 手機相機邊緣運算、OCR 前處理、條碼辨識、自動曝光評估 |
| [`packet_checksum`](src/packet_checksum.rs) | 零依賴 CRC32 (IEEE 802.3)、Adler-32 校驗、16-bit PCM/感測器專用校驗 | 檔案總管校驗、感測器封包傳輸、PNG/ZIP 區塊校驗 |
| [`crypto_vault`](src/crypto_vault.rs) | 零依賴 SHA-256 摘要、常數時間比較 (防時序側信道攻擊)、XOR 流式混淆 | 機密分片校驗、快照雜湊校驗、敏感字串安全防護 |
| [`sys_path`](src/sys_path.rs) | 跨平台相對路徑防穿越校驗、史實層只增不刪 (Append-Only) 差異校驗 | 檔案系統守衛、檔案總管路徑校驗、日誌保鮮關卡 |
| [`text_cleaner`](src/text_cleaner.rs) | UTF-8 邊界安全截斷、字詞邊界智能折行、CJK 字元計數 | 字幕視線居中 (RSVP)、翻譯文字流緩衝管理 |

---

## 三、快速使用範例

### 1. 引用方式 (在其他專案的 `Cargo.toml`)

```toml
[dependencies]
rust-bricks = { git = "https://github.com/mangala1728/rust-bricks.git" }
# 或本機路徑引入
# rust-bricks = { path = "../rust-bricks" }
```

### 2. 程式碼調用

```rust
use rust-bricks::math_time::clamp_i64;
use rust-bricks::crypto_vault::sha256_hex;
use rust-bricks::audio_vad::cps_chars_per_second;

fn main() {
    // 1. 安全數值邊界夾止
    let safe_val = clamp_i64(150, 0, 100);
    assert_eq!(safe_val, 100);

    // 2. 零依賴 SHA-256
    let hash = sha256_hex(b"hello rust-bricks");
    println!("SHA-256: {}", hash);

    // 3. 字幕 CPS 計算（字數 10，耗時 2000ms -> 5.0 字/秒）
    let cps = cps_chars_per_second(10, 2000).unwrap();
    assert_eq!(cps, 5.0);
}
```

---

## 四、AI 代理專用機器索引（Machine-Readable Catalog）

> **【甩手掌櫃免讀】** 本庫所有函式均已被結構化抽取至索引清單。  
> AI 碼工代理在跨專案開發時，請直接讀取下列檔案即可在 O(1) 時間內獲得可用積木、型別簽章與邊界定義，無需逐一掃描各個 `.rs` 原始檔：

- **YAML 格式（推薦代理檢索）**：[`CATALOG.yaml`](CATALOG.yaml)
- **JSON 格式（程式化讀取）**：[`CATALOG.json`](CATALOG.json)
- **自動更新指令**：新增積木後請執行 `cargo run -- catalog` 秒級同步最新機器索引。

---

## 五、倉庫容量與合約守門員（100% 純 Rust 內建守護）

為防止 AI 代理隨意提交二進位大檔或破壞 09 合約，本庫內建純 Rust 剛性守門員（`cargo run -- guard`）並接線至 Git `pre-commit` 與 `.agents/hooks.json`：

1. **二進位多媒體直擋**：嚴禁提交音訊（.wav/.mp3）、圖像（.png/.jpg）、封裝檔（.zip/.tar）或模型權重檔（.onnx/.pt）。
2. **單檔 500 KB 硬上限**：嚴禁提交超大日誌或龐大硬編碼測試數據，測試數據一律在代碼中以虛擬陣列動態生成。
3. **零外部依賴防線**：`Cargo.toml` 的 `[dependencies]` 必須為空。
4. **09 號合約自動審查**：提交前自動檢查 `## Map`、`@outputs`、`@domain` 守衛，禁止 `static mut` 與遞迴。
5. **單元測試全綠保證**：提交前自動執行 `cargo test`。

```bash
# 執行容量與合約守護檢查（100% 純 Rust）
cargo run -- guard

# 自動產生/更新機器索引（100% 純 Rust）
cargo run -- catalog

# 執行 CLI 自我檢驗程式
cargo run -- selftest

# 執行全部單元測試
cargo test

# 巡檢外部 contrib 貢獻分支（防撞車與體檢驗收）
cargo run -- patrol
```

---

## 六、多專案防撞車與核心策展管家協議（Curation & Integration）

為防止跨專案多 AI 代理並行上傳 Rust 代碼造成版本衝突與重複造輪子，本庫建立了「核心策展管家與防撞車協議」（詳見 [`docs/CURATION-AND-INTEGRATION.md`](docs/CURATION-AND-INTEGRATION.md)、[`docs/TRANSPILATION-METHODOLOGY.md`](docs/TRANSPILATION-METHODOLOGY.md)、[`docs/HUNTING-HIGH-VALUE-BRICKS.md`](docs/HUNTING-HIGH-VALUE-BRICKS.md) 與 [`CONTRIBUTING.md`](CONTRIBUTING.md)）：

1. **巡檢外部貢獻分支**：執行 `cargo run -- patrol` 主動拉取並檢查 `origin/contrib/*`。
2. **機械防重發明輪子**：`guard.rs` 自動掃描全庫函式名稱，禁止重複命名。
3. **`lib.rs` 字母排序防線**：`guard.rs` 自動審查 `src/lib.rs` 宣告排序，防止分支合併衝突。
4. **分支隔離上線**：本地開 `worker/*` 或 `integrate/*` 分支開發，經 `rebase main` 與五板斧驗收後方可 merge。
5. **缺陷硬鎖（Defect Gate）**：外部代碼有坑開單 `incidents/INC-*.md (OPEN)`，守衛剛性阻擋全庫入庫直至修復。
