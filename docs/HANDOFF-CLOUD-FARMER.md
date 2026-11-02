# 🤝 跨視窗與雲端自動農夫交接檔 (Handoff — 4+4 雙引擎金鑰池版)

## 1. 目前已完成之硬體級基建（100% 就緒）

1. **4 Gemini + 4 Groq 雙引擎自動輪替池 (`src/cloud_farmer.rs`)**：
   - **本地免設定自動載入**：[`resolve_key_pool_strings`](../src/cloud_farmer.rs) 會自動讀取 `./keys.local.json` 或 `../工作流/keys.local.json` 中的 `gemini`~`gemini4` 與 `groq`~`groq4`（共 8 把免費金鑰），且 `keys.local.json` 已列入 `.gitignore` 嚴禁入庫。
   - **雲端 GitHub Secrets 已自動同步**：已透過 `gh secret set` 將 4 把 Gemini 金鑰寫入 `GEMINI_API_KEYS`、4 把 Groq 金鑰寫入 `GROQ_API_KEYS`（目標倉庫：`mangala1728/rust-bricks`）。
   - **雙引擎無縫降級（Seamless Fallback）**：單次請求若 Gemini 金鑰池遇到限流或格式不符，同回合依序瀑布降級：**`gemini-3.8-flash` $\rightarrow$ `gemini-3.5-flash` $\rightarrow$ Groq (`openai/gpt-oss-120b`)**，絕不浪費重試次數。

2. **多模式同時啟動之四層防撞車機制**：
   - **開農前預檢**：[`check_module_already_claimed`](../src/cloud_farmer.rs) 自動檢查本地 `src/<mod>.rs` 與遠端 `origin/contrib/cloud-farm-<mod>`。
   - **分支隔離**：`cargo run -- farm <spec> <mod> --push` 一律推送到 `contrib/cloud-farm-<mod>`，不干擾 `main`。
   - **零衝突收割**：`cargo run -- harvest` 只從遠端分支摘取 `src/<mod>.rs` 與 `tests/test_<mod>.rs`，通過本地 `guard.rs`（含跨模組全域同名函式查重）後，一次重算 `CATALOG.yaml` 與 `.ai-memory/receipt.json`。

---

## 2. 給 `rust-bricks` 專屬視窗代理之直接貼上提示詞（Prompt）

若你在 `c:\For Gemini\rust-bricks` 視窗啟動新的 AI 代理，直接貼上以下提示詞即可無縫接手：

```markdown
請先閱讀 `GEMINI.md`、`docs/CLOUD-FREE-TIER-FARMING-GUIDE.md` 與 `docs/HANDOFF-CLOUD-FARMER.md`。
目前本專案已配置好「4 Gemini + 4 Groq 雙引擎免費金鑰池」（位於已 gitignore 的 `keys.local.json` 以及 GitHub Actions Secrets），並內建雲端農夫與防撞車收割機。

你的常駐任務如下：
1. 【巡檢與收割】開工先執行 `cargo run -- patrol` 與 `cargo run -- harvest`，將雲端自動農出的 `origin/contrib/cloud-farm-*` 分支或外部專案提交的 `origin/contrib/*` 分支進行零衝突收割、同名函式去重與五道硬體關卡驗收。
2. 【消化許願池】執行 `cargo run -- wish list` 查看待處理的許願單，或根據規格書執行 `cargo run -- farm <spec路徑> <模組名> --push` 自動農出符合 09 合約與 NASA JPL 60 行規範的純 Rust 微積木。
3. 【提交鐵律】每次推進 `main` 前必須依序通過：
   - `cargo run -- guard`
   - `cargo run -- catalog`
   - `cargo run -- receipt kev-4b PASS`
   - `cargo run -- verify-receipt`
```
