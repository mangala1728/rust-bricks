# 🌾 rust-bricks 雲端蹭免費版帳號・自動化農積木完整實戰指南

> 本指南將《私人子函數積木庫：雙軌雲端自動化佈署指南》全面升級為 **100% 純 Rust 零依賴架構 (`src/cloud_farmer.rs`)**。  
> 針對「**0 成本蹭免費雲端容器 + 多免費 API 帳號池自動輪替 + 硬體級 `guard.rs` 真實編譯閉環**」量身打造。

---

## 一、關鍵架構升級：為什麼不能只用 Gemini 內建 `code_execution` 沙盒？

原版 Python 指南使用了 Gemini API 的 `ToolCodeExecution` 雲端沙盒，但**該沙盒僅支援 Python，無法編譯 Rust，更無法執行我們嚴格的 `cargo run -- guard`（09 合約、60 行微積木、14 條工程鐵律與混沌測試）**。

因此，我們直接在 `rust-bricks` 核心內建了純 Rust 引擎 **`src/cloud_farmer.rs` (`cargo run -- farm`)**，將「免費雲端容器算力」與「免費 LLM 推理額度」完美結合：

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│                 免費雲端算力底座 (三選一，皆具備實體 rustc + cargo)            │
│  • 方案 A-1: GitHub Actions (網頁一鍵觸發 / 定時排程，連瀏覽器都不用開)        │
│  • 方案 A-2: GitHub Codespaces (每月免費 60 小時，tmux 背景掛網後關機睡覺)    │
│  • 方案 B  : Firebase Studio / IDX (免費無限時數，2vCPU/8GB，即時秒級驗收)   │
└──────────────────────────────────────┬───────────────────────────────────────┘
                                       │ 執行 `cargo run -- farm <spec> <mod>`
                                       ▼
┌──────────────────────────────────────────────────────────────────────────────┐
│           純 Rust 自動農夫引擎 (`src/cloud_farmer.rs` — 零外部 Crate)         │
│  1. 多免費 Key 自動輪替池 (`GEMINI_API_KEYS="key1,key2,key3"` / `GROQ` / `GH`)│
│  2. 自動注入 09 合約 + NASA JPL 60 行 + GEMINI.md 14 條鐵律 Prompt           │
│  3. 自動按字母序插入 `src/lib.rs`，寫入 `src/<mod>.rs` 與 `tests/test_<mod>.rs`│
│  4. 實機執行 `cargo test` + `cargo run -- guard`                             │
│     ├─ [失敗] 擷取編譯器/守衛報錯 → 換下一把免費 Key 自動自我修復 (最多 4 輪) │
│     ├─ [耗盡] 自動乾淨回滾 (Rollback)，絕不污染倉庫                          │
│     └─ [通過] 自動更新 CATALOG.yaml + 簽發 SHA-256 收據 + 自動 Push 分支！   │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## 二、「蹭免費版帳號」雙層白嫖配額攻略（算力 ＋ 模型 Token）

### 1. 免費模型推理池（`cloud_farmer.rs` 已內建三合一與多 Key 輪替）

| 免費來源 | 環境變數名稱 | 免費額度與優勢 | 如何蹭到極致？ |
| :--- | :--- | :--- | :--- |
| **Google AI Studio (首選雙階)** | `GEMINI_API_KEYS` | 免費層支援 `gemini-3.8-flash`（旗艦推理）與 `gemini-3.5-flash`（極速快刀），單帳號每日 **1,500 次請求 (RPD)**、每分鐘 15 次 | 引擎預設依序瀑布降級：**`gemini-3.8-flash` $\rightarrow$ `gemini-3.5-flash`**。準備 2~3 個免費 Google 帳號申請 API Key，以逗號串接：`GEMINI_API_KEYS="AIzaSyA...,AIzaSyB..."`，自動輪替避開 429！ |
| **Groq Cloud (三階快刀)** | `GROQ_API_KEYS` / `GROQ_API_KEY` | 免費層提供極速 `openai/gpt-oss-120b`（每日數十萬免費 Token） | 若 Gemini 兩階皆限流或連線受阻，引擎自動切換至 Groq 接手。 |
| **GitHub Models (零配置保底)** | `GITHUB_TOKEN` | GitHub Actions 內建免費權限，直接呼叫 `gpt-4o-mini` | 在 GitHub Actions 中無須額外申請，開箱即用。 |

### 2. 三大免費雲端主機底座對比

| 比較維度 | 方案 A-1：GitHub Actions (雲端排程/網頁點選) | 方案 A-2：GitHub Codespaces (離線掛網型) | 方案 B：Firebase Studio (同步快速閉環型) |
| :--- | :--- | :--- | :--- |
| **環境設定檔** | [`.github/workflows/cloud_farmer.yml`](../.github/workflows/cloud_farmer.yml) | [`.devcontainer/devcontainer.json`](../.devcontainer/devcontainer.json) | [`.idx/dev.nix`](../.idx/dev.nix) |
| **運算硬體** | 2-core / 7GB RAM (Ubuntu) | 2-core / 8GB RAM | 2-core / 8GB RAM |
| **免費額度** | Public 庫**無限免費**；Private 庫每月 2,000 分鐘 | 每月 **120 core-hours (60 小時)** 免費 | **免費、無固定月時數封頂**（閒置 15~30 分鐘休眠） |
| **適合情境** | 手機點一下或定時自動農，**完全不開電腦** | 睡前用 `tmux` 丟一排規格書，**直接關機睡覺** | 人在電腦前，30 秒內打磨單一高難度積木即刻入庫 |

---

## 三、三種雲端農積木操作手冊

### 玩法 1：GitHub Actions「手機一鍵開農」（方案 A-1，最省事）

1. 到 GitHub 倉庫的 **Settings $\rightarrow$ Secrets and variables $\rightarrow$ Actions**，新增 Repository Secret：
   - 名稱：`GEMINI_API_KEYS`
   - 內容：`你的Key1,你的Key2`（或只填一把 `GEMINI_API_KEY`）
2. 點選 GitHub 上方的 **Actions $\rightarrow$ Cloud Free-Tier Auto Brick Farmer $\rightarrow$ Run workflow**：
   - `spec_path`：填入規格書或願望單路徑（如 `specs/template.md` 或 `wishes/WISH-20261026-001-thermal-flir-palette.md`）
   - `module_name`：填入想產出的模組名（如 `dsp_ema_smoother`）
3. 點擊綠色的 **Run workflow** 後即可關掉網頁！雲端會自動生成、通過 `guard.rs` 五道防線、簽發收據並推送到 `contrib/cloud-farm-<模組名>` 分支。

---

### 玩法 2：GitHub Codespaces「拋出後關機睡覺」（方案 A-2）

我們已建好 [`.devcontainer/devcontainer.json`](../.devcontainer/devcontainer.json)，開啟 Codespaces 時會自動安裝 Rust 與 `tmux`。

1. 在 GitHub 倉庫頁面點擊 **Code $\rightarrow$ Codespaces $\rightarrow$ Create codespace**。
2. 在終端機啟動 `tmux` 背景工作階段，就算斷線或關電腦也會繼續跑：
   ```bash
   # 1. 設定免費 Key 池
   export GEMINI_API_KEYS="你的Key1,你的Key2"

   # 2. 建立持久化 tmux 視窗
   tmux new -s farm_bricks

   # 3. 執行自動農夫並於通過後自動 git commit & push
   cargo run -- farm specs/template.md dsp_ema_smoother --push

   # 4. 按下 Ctrl+B 再按 D 脫離背景，直接關閉瀏覽器去睡覺！
   ```

---

### 玩法 3：Firebase Studio「在線極速打磨閉環」（方案 B）

我們已建好 [`.idx/dev.nix`](../.idx/dev.nix)，進入 [Firebase Studio](https://studio.firebase.google.com/) 匯入本 GitHub 倉庫後，Nix 容器會自動裝好 `rustup`、`cargo`、`gcc` 與 `curl`。

1. 在 [`specs/`](../specs/template.md) 複製一份規格書（如 `specs/kalman_1d.md`）寫好目標簽章與黃金測試值。
2. 在 Firebase Studio 終端機執行：
   ```bash
   export GEMINI_API_KEYS="你的Key1,你的Key2"
   cargo run -- farm specs/kalman_1d.md dsp_kalman_1d --push
   ```
3. 終端機將即時顯示：
   - 呼叫免費 API 產出模組與測試
   - 自動執行 `cargo test` 與 `cargo run -- guard`
   - 若有 `@domain` 漏寫或行數超過 60 行，自動將 `guard` 報錯餵回模型修正
   - 印出 `🎉 微積木已產出並 100% 通過 Guard！` 後自動推回 GitHub，當場關閉分頁即可！

---

## 四、慢農積木的黃金紀律（防爆倉、防假合規）

1. **寧缺勿濫，每次只農一個微積木**：
   - 免費模型在單次專注於 1~3 個純函式（$\le 60$ 行）時，成功率高達 95% 以上。
2. **規格書必寫「確定性黃金測試值（Golden Values）」**：
   - 參考 [`specs/template.md`](../specs/template.md)，在規格書中明確給出一組標準輸入與精確預期輸出（例如 CRC32 `"123456789"` $\rightarrow$ `0xcbf43926`），防止模型寫出弱斷言。
3. **分支隔離安全網**：
   - 雲端農夫預設推送至 `contrib/cloud-farm-*` 分支，或是通過全套 `guard.rs`（含零信任黑箱混沌測試 `chaos_audit`）後才入庫，永遠不會把壞代碼推進 `main`！

---

## 五、多模式同時啟動之「四層防撞車與零衝突收割」協議

當 **本地端（Local Mode）** 與 **雲端三模式（GitHub Actions / Codespaces / Firebase Studio）** 同時啟動時，為了徹底杜絕「Git `main` 撞車」、「`CATALOG.yaml` / `receipt.json` 合併衝突」與「免費 API Key 429 搶頻寬」，系統於 [`src/cloud_farmer.rs`](../src/cloud_farmer.rs) 實作了四道硬體級防撞防線：

### 1. 第一層：開農前遠端領地預檢（`check_module_already_claimed`）
每次執行 `cargo run -- farm <spec> <mod>` 時，在消耗任何免費 API Token 之前，引擎會先自動執行：
- 檢查本地是否已存在 `src/<mod>.rs`
- 執行 `git fetch --all --quiet` 並檢查遠端是否已有 `origin/contrib/cloud-farm-<mod>` 分支
- **一旦發現已被本地或其他雲端節點認領，立即攔截退出**，絕不重複白農同一塊積木。

### 2. 第二層：主幹道與雲端專用車道物理隔離（Branch Isolation）
- **本地端（Local Mode）**：獨佔 `main` 主幹道話語權。
- **所有雲端模式（Cloud Modes）**：加上 `--push` 時，[`auto_commit_and_push_farm`](../src/main.rs) **強制切換並推送至獨立車道 `contrib/cloud-farm-<模組名>`**，絕不直接推 `main`，因此本地與多個雲端容器同時 Push 永遠是 $0$ 衝突。

### 3. 第三層：繞過熱點檔的「零衝突收割機」（`cargo run -- harvest`）
如果直接 `git merge` 雲端分支，`CATALOG.yaml`、`CATALOG.json`、`src/lib.rs` 與 `.ai-memory/receipt.json` 這四個共用熱點檔案 100% 會發生 Merge Conflict。
因此，本地端只需在空檔執行一鍵收割指令：
```bash
cargo run -- harvest
```
收割機（[`harvest_cloud_branches`](../src/cloud_farmer.rs)）會自動完成：
1. 掃描所有 `origin/contrib/cloud-farm-*` 遠端分支。
2. **只精準摘取純代碼果實**：用 `git show` 抽出該分支的 `src/<mod>.rs` 與 `tests/test_<mod>.rs`（完全跳過雲端分支上的舊版 `CATALOG` 與 `receipt`）。
3. 自動按字母序將 `pub mod <mod>;` 插入本地最新的 `src/lib.rs`。
4. 實機執行 `cargo test` 與 `guard.rs`（含**跨模組全域同名函式查重** `check_function_deduplication`，若雲端跟本地剛寫好的函式撞名，當場攔截並自動回滾該分支）。
5. 全部通過後，在本地**一次性重新生成** `CATALOG.yaml`、`CATALOG.json` 並簽發最新仲裁收據！

### 4. 第四層：免費 API Key 池分片錯開（`FARMER_NODE_ID` Sharding）
當多個環境共用同一組 `GEMINI_API_KEYS="Key0,Key1,Key2,Key3"` 時，為避免同時打第一把 Key 觸發每分鐘 15 次 (15 RPM) 限流，可在不同環境設定節點編號：
- 本地端：`FARMER_NODE_ID=0`（從 Key0 開始輪替）
- GitHub Actions：`FARMER_NODE_ID=1`（從 Key1 開始輪替）
- GitHub Codespaces：`FARMER_NODE_ID=2`（從 Key2 開始輪替）
- Firebase Studio：`FARMER_NODE_ID=3`（從 Key3 開始輪替）
