# 雲端農夫：失敗嘗試樹、二振出局、壓力紅燈與日誌保鮮

> 對齊 `mangala-mods` 的 `git-handoff`（失敗嘗試樹、commit 格式）與 `mangala-workflow`（二振出局、壓力紅燈）。
> 程式：[`src/farm_tree.rs`](../src/farm_tree.rs)、[`src/cloud_farmer.rs`](../src/cloud_farmer.rs)；工作流：[`.github/workflows/cloud_farmer.yml`](../.github/workflows/cloud_farmer.yml)。

## 一、每一輪的流程

```text
觸發：每 3 小時排程／推送 specs/** 到 main／手動
  │
  ├─ 日誌保鮮：刪除「已入庫模組」的 attempt/*-farm-* 死路分支（farm-prune）
  ├─ 壓力紅燈：連續 5 輪失敗 → 開 issue 並凍結（scripts/farmer_pressure.sh）
  ├─ 選規格：farm-queue 從 git 推導每份規格的狀態，取第一份「待做」
  │     已入庫 → 已被領取（contrib 分支）→ 規格可疑 → 卡關 → 待做
  ├─ 農作：最多 4 次嘗試
  │     ├─ 提示帶入「已知死路」（同一版規格先前各輪的 ## Dead-End）
  │     ├─ 規格 ```golden 區塊 → 自動產生 tests/golden_<mod>.rs（模型改不到）
  │     ├─ 失敗分類：compile／golden:#N／guard:<規則>／test:<名稱>／format／other／infra
  │     ├─ 同一分類連續 2 次 → 二振出局：提示要求推翻共用前提、強制換 API 供應商
  │     ├─ 同一分類連續 4 次 → 升級裁判（停止本輪）
  │     └─ API 全部無法連線 → infra：不記死路（與程式無關）
  │   成功 → contrib/cloud-farm-<mod>（Status: done，Resolves 列出被取代的死路）
  │   失敗 → attempt/<YYYYMMDD>-farm-<mod>（Status: abandoned + ## Dead-End）
  └─ issue 同步：依 git 狀態開關「卡關／規格可疑／佇列空」issue（scripts/farmer_issues.sh）
```

## 二、失敗嘗試樹 = git 分支
- 每輪失敗把**失敗的程式碼**連同 git-handoff 格式的 commit 訊息，存成一個 `attempt/<YYYYMMDD>-farm-<模組>` 分支並推上 origin；同一天第二輪起加 `-r2`、`-r3`。
- commit 訊息通過 git-handoff 的 `validate()`（必填 `AI-Handoff`、`Status`、`Tests`、`Map`、`Next`，段落 `What`、`Why`、`Dead-End`、`Handoff`、`Evidence`），所以本機 Claude Code 的 git-handoff 會把它們列進 `.ai-memory/ATTEMPTS.md`。
- 額外的機器欄位：`Farm-Spec`、`Farm-Spec-Hash`、`Farm-Module`、`Farm-Round`、`Farm-Outcome`、`Farm-Failure`、`Farm-Model`、`Farm-Rounds`（每次嘗試的「模型=分類」）。
- `## Dead-End` 第一條是給下一輪與 ATTEMPTS.md 的教訓：`<分類>：<錯誤>；下次避開：<建議>`。
- `## Evidence` 由程式從真實的 `cargo test`／guard 輸出摘錄，並遮蔽 `key=`、`Bearer` 金鑰。

## 三、停止條件（避免燒免費額度）
| 狀態 | 條件 | 處理 |
|---|---|---|
| 規格可疑 | 不同模型卡在同一個 `golden:#N` | 停止自動重試，開 `farmer:spec-suspect` issue；**禁止讓 AI 改預期值** |
| 卡關 | 同一版規格累積 3 輪放棄 | 停止自動重試，開 `farmer:blocked` issue |
| 升級裁判 | 同一輪內同一分類連續 4 次 | 本輪停止，記為 `escalated` |
| 壓力紅燈 | 不分規格連續 5 輪失敗 | 開 `farmer:pressure-red` issue，關閉前每輪凍結 |

要原樣重試一份卡關的規格：刪除它的 `attempt/*-farm-<模組>*` 分支即可。

## 四、日誌保鮮（舊紀錄不會冒充現況）
- **規格版本**：每筆死路記錄 `Farm-Spec-Hash`（規格書的 git blob 雜湊）。規格一改，舊死路自動不算數：不帶入提示、不計入卡關輪數，模組重新回到「待做」。
- **成功取代死路**：成功的 commit 以 `Resolves:` 列出被取代的死路分支；模組入庫後，下一輪 `farm-prune` 刪除這些分支，`harvest` 也會清掉本機的。
- **狀態不手寫**：佇列狀態（`farm-status`）與 issue 每輪都從 git 重新推導；狀態改變時，過期的 issue 自動關閉。

## 五、本機指令
```bash
cargo run -- farm-status              # 佇列狀態表
cargo run -- farm-queue               # 下一份待做規格與卡關清單（key=value）
cargo run -- farm-report <模組>        # 某模組本版規格的死路報告
cargo run -- farm-prune [--push]      # 清除已入庫模組的死路分支
cargo run -- farm <spec> <模組> [--push]
```

## 六、已知限制
- git-handoff 目前只讀本機 `refs/heads/attempt/*`；雲端推上去的分支在本機 `git fetch` 後位於 `refs/remotes/origin/attempt/*`，**不會自動進 ATTEMPTS.md**。需要在 mangala-mods 讓 git-handoff 也讀 `refs/remotes/origin/attempt/*`，或在本機執行 `git fetch origin 'refs/heads/attempt/*:refs/heads/attempt/*'`。
- 既有規格沒有 ```golden 區塊時照常農作，但成功 commit 會標 `Golden-Vectors: unchecked`，收割前請人工確認。
- GitHub 的排程觸發是盡力而為，可能延遲或略過；推送新規格到 main 會立即觸發。
