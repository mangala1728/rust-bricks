# 品質總監自動化出巡與事件觸發機制（Quality Patrol & Event-Driven Automation）

> **「常態化暗中巡視、突發事件立即出巡、夜間定時全面狂轟濫炸；全庫無死角防禦，杜絕任何僥倖。」**  
> 本指南規範 `rust-bricks` 庫中「零信任品質總監」的四大事件觸發機制與自動化排程設定。

---

## 一、四大事件觸發矩陣（The 4 Event Triggers）

品質總監並非被動等待召喚，而是深深嵌入本機開發、Git 生命週期、GitHub 雲端 CI 與定時排程中：

```text
┌────────────────────────────────────────────────────────────────────────┐
│                   零信任品質總監四大事件出巡矩陣                         │
├──────────────────┬──────────────────┬─────────────────┬────────────────┤
│ 1. 代碼提交事件   │ 2. 遠端推送事件   │ 3. 雲端 PR/合併 │ 4. 夜間定時夜巡 │
│ (Pre-commit)     │ (Pre-push)       │ (GitHub Actions)│ (Nightly Cron) │
├──────────────────┼──────────────────┼─────────────────┼────────────────┤
│ • 範圍：當前變更  │ • 範圍：本地分支  │ • 範圍：雲端 PR │ • 範圍：全庫全量│
│ • 檢查：guard.rs │ • 檢查：audit +  │ • 檢查：全流程  │ • 檢查：黑箱轟炸│
│   (含黑箱混沌)    │   patrol 分支    │   CI 矩陣驗證   │   + 許願池掃描 │
└──────────────────┴──────────────────┴─────────────────┴────────────────┘
```

### 1. 事件 A：代碼本地提交事件（Local Pre-commit Hook）
- **觸發時機**：工程師或 AI 代理下達 `git commit` 時。
- **執行動作**：`.git/hooks/pre-commit` 自動調用 `cargo run --quiet -- guard`。
- **守護重點**：容量審查、09 合約規則、單元測試、以及品質總監第五道防線（混沌黑箱抽檢）。

### 2. 事件 B：遠端代碼推送事件（Local Pre-push Hook）
- **觸發時機**：執行 `git push origin main` 時。
- **執行動作**：`.git/hooks/pre-push` 自動調用：
  1. `cargo run --quiet -- audit`（品質總監黑箱極端抽查）
  2. `cargo run --quiet -- patrol`（巡檢遠端是否有未結的外部 `contrib/*` 分支）
- **守護重點**：防止未經品質總監批准的代碼流出本機，防止與外部專案分支撞車。

### 3. 事件 C：雲端 PR 與推送事件（GitHub Actions CI）
- **觸發時機**：任何外部代理在 GitHub 發起 Pull Request，或有人 push 到 `main`。
- **執行腳本**：[`.github/workflows/quality_director_patrol.yml`](../.github/workflows/quality_director_patrol.yml)
- **雲端環境**：Ubuntu 最新純淨環境，自動秒級快取 Cargo，嚴格執行五步驗收（Test $\rightarrow$ Guard $\rightarrow$ Audit $\rightarrow$ Patrol $\rightarrow$ Wish List）。

### 4. 事件 D：每日定時夜巡事件（Scheduled Nightly Patrol）
- **雲端定時**：GitHub Actions 每日 UTC 20:00（台灣時間 04:00 AM）自動醒來出巡。
- **本機定時**：透過 Windows 工作排程器，每日清晨 03:00 自動執行 [`scripts/quality_director_patrol.ps1`](../scripts/quality_director_patrol.ps1)。

---

## 二、本機 Windows 工作排程器設定（一步啟用）

如果你希望本機電腦在開機聯網期間，每日清晨由品質總監自動突擊巡邏全庫：

### 1. 一鍵註冊每日清晨 03:00 排程
以管理員身份開啟 PowerShell，執行：
```powershell
powershell -ExecutionPolicy Bypass -File "c:\For Gemini\rust-bricks\scripts\quality_director_patrol.ps1" -RegisterTask
```

### 2. 手動隨時觸發出巡
```powershell
powershell -ExecutionPolicy Bypass -File "c:\For Gemini\rust-bricks\scripts\quality_director_patrol.ps1"
```

### 3. 取消排程
```powershell
powershell -ExecutionPolicy Bypass -File "c:\For Gemini\rust-bricks\scripts\quality_director_patrol.ps1" -UnregisterTask
```

---

## 三、品質總監出巡失敗之熔斷機制（Circuit Breaker）

若品質總監在任何一次出巡中發現某個子函式被極限數值（如 `NaN`、除以零、未閉合字元）擊穿：
1. **立即熔斷（Abort）**：退出代碼一律為 `1`，Git Hook 立即阻擋 Commit/Push，GitHub Actions 標記為紅叉（Failed）。
2. **自動開單阻擋（Trigger Defect Gate）**：
   - 審查員自動在 `incidents/` 生成 `INC-YYYYMMDD-CHAOS-FAILURE.md`，狀態標為 `OPEN`。
   - `guard.rs` 的防線三（Defect Gate）會立即鎖死全庫的新功能入庫，直到工兵修復該漏洞並在通報中填入 `resolution_commit` 為止！
