# 雲端冷啟動架構與工程交接總綱 (Cloud Cold-Start Handover & Deployment Runbook)

> **給雲端維運／架構工程師與 AI 代理的冷啟動交接指南**  
> 本文件詳述如何在全新的雲端主機（Ubuntu / Debian / Amazon Linux / Docker 容器）中，從零開始（冷啟動 Cold-Start）搭建並運行 **純 Rust 微積木庫 (`rust-bricks`)**、**數位史官高頻磁軌 (`digital-historian`)** 以及 **全域下水道標準 (`domain-pipe-standards`)**。

---

```text
## Map（系統整體接線與冷啟動架構圖）
┌──────────────────────────────────────────────────────────────────────────────┐
│                    雲端伺服器 (Linux VM / VPS / Docker / CI Runner)          │
│       • CPU >= 2 vCPU, RAM >= 2GB, OS: Ubuntu 22.04/24.04 LTS 或 Debian 12   │
└──────────────────────────────────────▲───────────────────────────────────────┘
                                       │ 執行 scripts/cloud_bootstrap.sh
┌──────────────────────────────────────┴───────────────────────────────────────┐
│              [第一核心] 09 合約純 Rust 微積木庫 (`rust-bricks`)                │
│  • 879+ 純 Rust std 純淨函式（0 外部 crate 依賴、零全域可變狀態）              │
│  • NASA JPL <= 60 行規範、一階守衛入口、14 大工程鐵律防線                      │
│  • 自帶無人值守雲端農夫 (Cloud Farmer) + 失敗嘗試樹 (Dream-RSI Attempt Tree) │
│  • 內建 Brick-Guard 守衛、Diff 雙模型聯署驗收收據、零信任黑箱巡檢             │
└──────────────────────────────────────┬───────────────────────────────────────┘
                                       │ 雙向軟連結 / 跨倉庫 SHA-256 同步
┌──────────────────────────────────────┴───────────────────────────────────────┐
│              [第二核心] 數位史官與高頻事件磁軌 (`digital-historian`)          │
│  • 因果防偷看雙時態歷史地層 (Bitemporal Stratigraphy)                          │
│  • 32-Byte 定長二進位高頻磁軌流式記錄器 (CelestialPacket32 / VedicEventPacket32)│
│  • DTW 動態時間校整拓撲圖譜、微觀市場/天體模式匹配                             │
│  • 24/7 常駐防暴衝硬鎖 (--armed-24x7) 與 MCP JSON-RPC 2.0 stdio 伺服器       │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## 一、主機硬體與作業系統最低規格 (Hardware & OS Specifications)

| 規格項目 | 最低需求 (Minimum) | 推薦生產規格 (Recommended) | 說明 |
|---|---|---|---|
| **作業系統 (OS)** | Ubuntu 22.04 LTS / Debian 11 | **Ubuntu 24.04 LTS / Debian 12** | 具備完整 POSIX 環境與最新 Glibc |
| **中央處理器 (CPU)** | 2 vCPU (x86_64 或 aarch64) | **4 vCPU 以上** | 平行編譯與 120+ 測試全套跑測 < 15 秒 |
| **隨機記憶體 (RAM)** | 2 GB | **4 GB** | `rust-bricks` 與史官記憶體峰值 < 64MB，極度節省 |
| **磁碟空間 (Disk)** | 10 GB SSD | **20 GB SSD** | Cargo build 快取 (`target/`) 約需 2~4 GB |
| **網路環境** | 對外可存取 GitHub 與 HTTPS | 具備穩定連線能力 | 用於拉取代碼與呼叫免費 AI API 金鑰池 |

---

## 二、雲端冷啟動方式（二選一）

### 方案 A：一鍵全自動引導 (Recommended One-Click Bootstrap)

在全新雲端主機終端機中，直接執行主庫內附的引導腳本：

```bash
# 1. 建立工作目錄並進入
mkdir -p ~/workspace && cd ~/workspace

# 2. 克隆主庫
git clone https://github.com/mangala1728/rust-bricks.git
cd rust-bricks

# 3. 賦予權限並執行一鍵冷啟動引導腳本
chmod +x scripts/cloud_bootstrap.sh
./scripts/cloud_bootstrap.sh
```

**`cloud_bootstrap.sh` 會自動依序完成**：
1. 安裝系統編譯工具（`build-essential`, `curl`, `git`, `python3`, `jq`, `pkg-config`）。
2. 安裝官方最新 Rust 穩定版工具鏈 (`rustup` / `cargo`)。
3. 產生金鑰設定檔範本 `.env`。
4. 執行下水道標準一致性校驗 (`sync_domain_standards.py`)。
5. 執行 09 合約品質守衛 (`cargo run -- guard`) 與全庫測試 (`cargo test`)。

---

### 方案 B：手動分步冷啟動流程 (Manual Step-by-Step)

若您處於特殊容器、唯讀環境或需要嚴格稽核每一步：

#### 步驟 1：安裝基礎依賴
```bash
# Debian / Ubuntu
sudo apt-get update
sudo apt-get install -y build-essential curl git python3 python3-pip jq pkg-config libssl-dev

# RHEL / CentOS / Amazon Linux
sudo dnf groupinstall -y "Development Tools"
sudo dnf install -y curl git python3 python3-pip jq openssl-devel
```

#### 步驟 2：安裝 Rust 工具鏈
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
source "$HOME/.cargo/env"

# 驗證安裝
rustc --version
cargo --version
```

#### 步驟 3：配置 Git 身份與權限
```bash
git config --global user.name "Cloud Operator"
git config --global user.email "operator@cloud.invalid"
```

#### 步驟 4：克隆專案
```bash
mkdir -p ~/workspace && cd ~/workspace
git clone https://github.com/mangala1728/rust-bricks.git
```

#### 步驟 5：配置金鑰環境變數
在 `rust-bricks` 根目錄下建立 `.env`：
```bash
cat << 'EOF' > ~/workspace/rust-bricks/.env
# GitHub Token (用於讀取 Actions 狀態、提交 PR 或派工)
GITHUB_TOKEN="ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"

# 免費模型金鑰池 (以逗號分隔多組 Key，自動輪詢抗 Rate Limit)
GEMINI_API_KEYS="AIzaSyA...,AIzaSyB..."
GROQ_API_KEYS="gsk_xxxxA...,gsk_xxxxB..."

# 執行期日誌
RUST_BACKTRACE=1
RUST_LOG=info
EOF
```

---

## 三、五階剛性驗收關卡 (Verification Runbook)

在冷啟動完成後，執行驗收腳本確認系統處於 100% 健康狀態：

```bash
cd ~/workspace/rust-bricks
chmod +x scripts/cold_start_verify.sh
./scripts/cold_start_verify.sh
```

### 五階檢驗標準說明：
1. **[Stage 1] 零外部依賴檢查**：
   查驗 `Cargo.toml` 的 `[dependencies]` 區塊，確保 **100% 依賴為零（0 External Crates）**。
2. **[Stage 2] 09 合約與 NASA JPL 60 行容量守衛**：
   執行 `cargo run -- guard`，掃描全部 879+ 個函式之開頭 `## Map`、`// @outputs: N`、`// @domain:` 執行期守衛、禁止 `static mut`、禁止遞迴、單一函式行數 $\le 60$ 行。
3. **[Stage 3] 全庫單元測試驗收**：
   執行 `cargo test`，驗證 120+ 項整合測試、微積木測試與黃金向量測試全部通過（0 失敗、0 忽略）。
4. **[Stage 4] 便攜下水道標準套件驗收**：
   執行 `python3 portable/domain-pipe-standards/test_astro_pipeline.py`，驗證 32-Byte 定長二進位小端序排佈與五分曆純整數算術。
5. **[Stage 5] 雲端農夫佇列與死路樹狀態**：
   執行 `cargo run -- farm-status`，確認當前規格佇列與嘗試樹無鎖死現象。

---

## 四、雲端常駐服務啟動與日常維運 SOP (Operations Runbook)

冷啟動完成後，可依場景啟用以下三種常駐或背景模式：

### 模式 A：無人值守雲端農夫 (Autonomous Cloud Farmer)

讓伺服器背景自動消耗免費 API 額度，根據 `specs/*.md` 規格自動寫碼、自跑測試、自動二振出局並記錄死路樹：

```bash
# 查詢目前佇列中最優先的待辦規格
cargo run -- farm-queue

# 本地單次施工指定規格
cargo run -- farm specs/astro_panchanga_tithi.md astro_panchanga_tithi

# 或啟用背景 Cron 定時排程 (每 3 小時自動推進一塊)
crontab -e
# 加入以下排程：
# 0 */3 * * * cd /home/ubuntu/workspace/rust-bricks && cargo run -q -- farm-queue | grep next_spec && cargo run -q -- farm "$(cargo run -q -- farm-queue | grep next_spec | cut -d= -f2)" "$(cargo run -q -- farm-queue | grep next_module | cut -d= -f2)" --push >> /var/log/cloud_farmer.log 2>&1
```

---

### 模式 B：巡邏與一鍵收割 (Patrol & Harvest SOP)

當外部 GitHub Actions 雲端農夫或本機背景農耕產出分支時：

```bash
# 1. 巡邏遠端是否有待審核的農夫分支
cargo run -- patrol

# 2. 一鍵零衝突收割 (自動隔離編譯測試、按字母插入 lib.rs、更新目錄)
cargo run -- harvest

# 3. 簽發 Diff 綁定之雙模型聯署驗收收據 (以 kev-4b 或 arbiter 簽名)
cargo run -- receipt kev-4b PASS
cargo run -- verify-receipt

# 4. 提交並推送至 main 主分支
git add .
git commit -m "feat(harvest): integrate farmed bricks"
git push origin main
```

---

### 模式 C：數位史官 MCP 伺服器掛載 (AI Agent Stdio Integration)

供 Claude Code、Antigravity、Cursor 或自研 AI 代理作為外掛 MCP 工具掛載：

在代理工具設定檔（如 Claude Desktop `claude_desktop_config.json` 或 Antigravity MCP 設定）中加入：
```json
{
  "mcpServers": {
    "digital-historian": {
      "command": "/home/ubuntu/workspace/digital-historian/target/release/digital-historian",
      "args": ["mcp-serve", "--armed-24x7"]
    }
  }
}
```
*註：`--armed-24x7` 為安全防護硬鎖，未帶旗標時系統剛性拒絕實盤連線（Fail-Closed 原則）。*

---

## 五、常見冷啟動問題與排障指南 (Troubleshooting FAQ)

| 症狀現象 | 根本原因 | 解除步驟 |
|---|---|---|
| **`fatal: Unable to create '.git/index.lock': File exists`** | 舊 Git 行程異常殘留或子行程繼承 Git Hook 環境變數 | 1. 執行 `rm -f .git/index.lock`<br>2. 確認腳本中呼叫 `git` 時已帶有 `.env_remove("GIT_INDEX_FILE")` |
| **`Permission denied (publickey)`** | 雲端伺服器未配置 GitHub SSH Key | 改用 HTTPS 網址或於主機執行 `gh auth login` 進行 Token 授權 |
| **`error: linker cc not found`** | 系統缺少 C 語言編譯器工具組 | 執行 `sudo apt-get install -y build-essential` 或 `sudo yum groupinstall "Development Tools"` |
| **`Rate limit exceeded (429)`** | 單一 API Key 請求過於頻繁 | 在 `.env` 中於 `GEMINI_API_KEYS` 填入多組 Key（逗號隔開），系統會自動輪換 |
| **`Receipt verification failed (SHA-256 mismatch)`** | 修改了代碼但未重新簽收據 | 重新執行 `cargo run -- receipt <model> PASS` 產生綁定目前 diff 的簽章 |
