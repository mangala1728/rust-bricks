# rust-bricks 冷啟動交接指南（Cold-Start Handover Packet）

> **專案定位**：純淨零依賴 Rust 跨語言演算法轉譯微積木庫（09 合約規範）。  
> **唯一戰場**：`c:\For Gemini\rust-bricks`（GitHub: `mangala1728/rust-bricks`）。  
> **編制組織**：指揮官（架構統籌與任務派發）＋ 工兵（純淨演算法轉譯）＋ 審查員（三道防線與收據驗收）。  
> **當前狀態**：19 大核心模組、78 個純函式、37 個單元測試全綠、Guard 剛性檢查 100% 通過、0 OPEN 缺陷。  
> **更新日期**：2026-10-26  

---

## 1. 專案使命與戰略定位

1. **主流語言演算法純淨轉譯**：
   - 研讀 Python (NumPy/SciPy/Audio)、TypeScript、Kotlin、C/C++ 高頻數學、字串、影音 DSP、密碼編解碼、校驗演算法。
   - 剝離龐大框架依賴，轉譯為純 Rust 標準庫重寫的「零依賴、純記憶體、狀態可窮舉之微積木」。
2. **服務下游邊緣與高放權專案**：
   - 為 `ai-interpreter`（Android 面對面口譯 App）、`資料整理助手`、量化交易與字幕音訊處理等專案提供可秒級編譯（x86_64, aarch64 NDK, wasm32）的高性能純函式庫。

---

## 2. 三位一體編制與策展整合（Curator + Engineer + Arbiter）

本團隊兼任「核心策展管家（Core Curator & Integrator）」，落實四大防撞車職責（詳見 [`docs/CURATION-AND-INTEGRATION.md`](docs/CURATION-AND-INTEGRATION.md) 與 [`docs/TRANSPILATION-METHODOLOGY.md`](docs/TRANSPILATION-METHODOLOGY.md)）：
1. **外部巡檢**：`cargo run -- patrol` 監控 `origin/contrib/*`。
2. **機械去重**：`guard.rs` 自動掃描全庫函式重名與重複造輪子。
3. **無衝突變軌**：分支隔離 (`worker/*`)、`lib.rs` 字母排序防線、`rebase main`。
4. **缺陷通報開單**：發現瑕疵開單 `incidents/INC-*.md (OPEN)` 觸發 Defect Gate。

```text
┌────────────────────────────────────────────────────────┐
│   指揮官 (Commander / Coordinator)                      │
│   • 統籌全域架構、跨專案積木需求對齊、任務拆解與排程    │
│   • 監控代理壓力指數、產出 Ketchup 四格終審報告          │
└───────────────────────────┬────────────────────────────┘
                            │ 任務指派與合約規範
                            ▼
┌────────────────────────────────────────────────────────┐
│   工兵 (Engineer / Worker)                             │
│   • 深入原語言底層算法數學原理，轉譯純 Rust 微積木     │
│   • 嚴格落實 JPL 60 行原則、邊界守衛與防溢出安全       │
└───────────────────────────┬────────────────────────────┘
                            │ 提交流程與防線檢查
                            ▼
┌────────────────────────────────────────────────────────┐
│   審查員 / 守衛 (Reviewer / Guard / Arbiter)            │
│   • 執行 `cargo run -- guard`（三道硬體級防線剛性紅線）│
│   • Defect Gate（未結缺陷單阻擋）與簽署 Kev-4B 收據     │
└────────────────────────────────────────────────────────┘
```

---

## 3. 三道硬體級防線與品質守衛（`guard.rs`）

任何 PR 或 Commit 必須通過 `cargo run -- guard`，觸碰以下紅線者立即阻擋：
- 🔴 **NASA JPL 60 行原則**：函式長度超過 60 行一律阻擋！必須分解為子積木。
- 🔴 **06/09 號合約守衛**：具名函式必備 `/// @outputs: N`；數值參數必備 `/// @domain: param [min..max]` 且第一行必須寫執行期守衛；禁止 `static mut`；禁止遞迴（一律顯式有界佇列）。
- 🔴 **危險操作安全證明**：禁止隨意寫 `.unwrap()` 或 `panic!`，唯有具備 `// @safety-proof:` 註解方可例外通過。
- 🔴 **未結缺陷通報硬鎖（Defect Gate）**：若 `incidents/` 目錄存在任何 `status: OPEN` 或 `UNRESOLVED` 的通報單，守衛將封鎖全庫新功能入庫！必須優先修復缺陷單。

---

## 4. 驗收與收據簽發閉環手冊

完成模組編寫後，執行標準驗收五板斧指令：
```bash
cargo test                                # 1. 單元測試全綠
cargo run -- guard                        # 2. 機械合約檢查通過
cargo run -- catalog                      # 3. 重建 CATALOG.yaml / JSON
cargo run -- receipt kev-4b PASS          # 4. 本地決策模型驗證語義並簽發 SHA-256 收據
cargo run -- verify-receipt               # 5. 確保收據與當前 Git Diff 100% 吻合
```

---

## 5. 模組現況清單

| 模組 | 原始檔 | 主要能力 |
|---|---|---|
| `math_time` | `src/math_time.rs` | 數值夾止、滑動均線、儒略日曆法、占星大限週期比例、XorShift64 PRNG |
| `audio_vad` | `src/audio_vad.rs` | PCM16 RMS 能量、靜音幀偵測、字幕 CPS 字速審查 |
| `audio_dsp` | `src/audio_dsp.rs` | 幀功率/能量、零交叉率 (ZCR)、Biquad 高通濾波、噪聲能量縮放、減法 SNR |
| `audio_wav` | `src/audio_wav.rs` | 零依賴 16-bit PCM WAV 檔頭解析與構建 |
| `sensor_motion` | `src/sensor_motion.rs` | 陀螺儀/加速度計互補濾波姿態融合、1D 卡爾曼濾波、Haversine GPS 大圓距離與航向角 |
| `sensor_hardware` | `src/sensor_hardware.rs` | 霍爾磁力翻蓋開合偵測、3軸磁力計傾斜補償指南針、閃光燈 PWM 頻閃與 SOS 時序 |
| `net_telemetry` | `src/net_telemetry.rs` | RTT 網路延遲與抖動平滑、零配置 IPv4/IPv6 CIDR 子網比對、私有網段判定 |
| `vision_yuv` | `src/vision_yuv.rs` | Android Camera2 (NV21 / YUV420sp) 預覽幀灰階快速抽取、2x2 平均降採樣、亮度直方圖 |
| `packet_checksum` | `src/packet_checksum.rs` | 零依賴 CRC32 (IEEE 802.3)、Adler-32 校驗、16-bit PCM/感測器專用校驗 |
| `ring_buffer` | `src/ring_buffer.rs` | 純記憶體無鎖固定容量 PCM16 環形緩衝區、批次讀寫、溢出守衛 |
| `crypto_vault` | `src/crypto_vault.rs` | 純 Rust SHA-256、常數時間相等性比較、XOR 流混淆 |
| `sys_path` | `src/sys_path.rs` | 相對路徑穿越防禦、斜線標準化、史實層只增不刪校驗 |
| `touch_control` | `src/touch_control.rs` | 觸控螢幕消抖、EMA 滑動速度估算、壓力觸控三階遲滯分級、手勢狀態機 |
| `vision_feature` | `src/vision_feature.rs` | Otsu 自適應雙峰二值化、Sobel 3x3 邊緣梯度、積分圖 O(1) 區域像素求和 |
| `bezier_curve` | `src/bezier_curve.rs` | 三次貝茲曲線座標內插、切線速度向量、曲線曲率幾何計算 |
| `motion_dead_reckon` | `src/motion_dead_reckon.rs` | 零速更新 (ZUPT)、1D 航位推算積分、觸覺震動馬達多模式時序生成 |
| `fast_math` | `src/fast_math.rs` | Safe Quake 快速倒數平方根 (0x5f3759df)、純整數開方 isqrt_u64、線性內插 lerp、Hermite 平滑階躍 smoothstep |
| `geo_collision` | `src/geo_collision.rs` | 軸對齊包圍盒 (AABB) 碰撞重疊、2D 線段空間精確交點、Jordan 射線法點在多邊形內外判定 |
| `text_cleaner` | `src/text_cleaner.rs` | UTF-8 字元邊界安全截斷、CJK 字元計數 |
| `guard` | `src/guard.rs` | 純 Rust 內建合約守衛、檔案防膨脹、依賴審查、缺陷硬鎖 |
| `catalog` | `src/catalog.rs` | 純 Rust 自動產生 CATALOG.yaml / CATALOG.json 機器索引 |
| `receipt` | `src/receipt.rs` | Git Diff 綁定之 SHA-256 本地決策收據簽發與校驗 |
