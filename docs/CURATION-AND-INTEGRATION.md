# rust-bricks 核心策展管家與防撞車整合協議（Curation & Integration Protocol）

> **「杜絕重複發明輪子、杜絕多專案並行衝突、守護 main 分支黃金品質。」**  
> 本協議為 `rust-bricks` 核心管家團隊（Core Curator & Integration Agent）執行巡檢、審查、去重與分支合併的硬體級標準作業程序。

---

## 一、四項核心管家職責

```text
┌────────────────────────────────────────────────────────┐
│ 1. 外部貢獻巡檢（Branch Patrol & Verification）        │
│    • 執行 cargo run -- patrol 監控 origin/contrib/*     │
│    • 對外部代碼執行硬體級五項體檢                      │
└───────────────────────────┬────────────────────────────┘
                            │
                            ▼
┌────────────────────────────────────────────────────────┐
│ 2. 重複輪子識別與合併收斂（Deduplication & Refactor）  │
│    • 機械守門員 guard.rs 全域掃描 duplicate functions │
│    • 融合為唯一邊界最完善、零依賴之黃金版本            │
└───────────────────────────┬────────────────────────────┘
                            │
                            ▼
┌────────────────────────────────────────────────────────┐
│ 3. 分支隔離與無衝突上線（Conflict-Free Landing）        │
│    • 嚴禁在 main 開發：開 worker/* 或 integrate/*      │
│    • rebase main，lib.rs 強制按字母排序防衝突          │
│    • 跑通 test -> guard -> catalog -> receipt          │
└───────────────────────────┬────────────────────────────┘
                            │
                            ▼
┌────────────────────────────────────────────────────────┐
│ 4. 缺陷通報硬鎖阻擋（Incident Ledger & Defect Gate）   │
│    • 外部代碼有坑不包庇，開 incidents/INC-*.md (OPEN)   │
│    • guard.rs 強制封鎖全庫入庫，直至修復為 RESOLVED   │
└────────────────────────────────────────────────────────┘
```

---

## 二、任務一：外部貢獻分支巡檢與五項體檢

每當啟動或完成一個工作週期，執行：
```bash
cargo run -- patrol
# 或底層 Git 指令：
# git fetch --all && git branch -r
```

若發現待審查之分支（如 `origin/contrib/<專案名>-<功能>`）：
```bash
git checkout -b review-<分支名> origin/<分支名>
```

### 硬體級五項體檢表
1. **依賴純淨度**：`Cargo.toml` 的 `[dependencies]` 必須為空，嚴禁夾帶外部 crate。
2. **NASA JPL 60 行原則**：每個函式長度 $\le 60$ 行，超標者退回或拆解為微積木。
3. **06/09 號合約邊界**：檔案開頭必附 `// ## Map`；具名函式必標 `/// @outputs: N`；數值參數必標 `/// @domain:` 且第一行必須實作執行期守衛；禁止 `static mut`；禁止遞迴。
4. **安全證明標注**：禁止隨意寫 `.unwrap()`、`.expect()` 或 `panic!`，唯有具備 `// @safety-proof:` 註解方可例外通過。
5. **機械守衛驗收**：`cargo test` 與 `cargo run -- guard` 全數綠燈通過。

---

## 三、任務二：子函數去重與合併治理（機械防線已接線）

為杜絕不同專案 AI 代理重複發明同名或同義輪子：
- **機械守衛全面攔截**：`guard.rs` 內建 `check_function_deduplication`，當不同模組出現相同函式名時，立即報錯攔截入庫！
- **黃金版本融合**：若發現語意相似函式，挑選邊界最完備版本，保留單一實作。

---

## 四、任務三：分支隔離與無衝突上線協議（Conflict-Free Landing Protocol）

1. **嚴禁直接在 `main` 上寫碼**：
   - 自己的開發任務開 `worker/<模組名>`。
   - 整合外部貢獻開 `integrate/<專案名>`。
2. **合併進 `main` 前自主變軌（Rebase）**：
   ```bash
   git checkout main
   git pull origin main
   git checkout <你的工作分支>
   git rebase main
   ```
3. **`lib.rs` 模組宣告字母排序鐵律**：
   - `guard.rs` 自動檢查 `src/lib.rs` 的 `pub mod <模組>;` 是否依英文字母順序排列。
   - 排序一致可徹底避免 Git 在 `lib.rs` 產生合併衝突（Merge Conflict）。
4. **驗收與簽收收據**：
   ```bash
   cargo test
   cargo run -- guard
   cargo run -- catalog
   cargo run -- receipt kev-4b PASS
   cargo run -- verify-receipt
   ```
5. **合併入主線**：
   ```bash
   git checkout main
   git merge --ff-only <你的工作分支>
   git push origin main
   ```

---

## 五、任務四：外部代碼有坑時的缺陷通報（Incident Ledger）

若外部代碼存在未定義行為、邊界溢出或測試失敗，但無法通靈其業務意圖：
1. 嚴禁直接調低測試標準或假裝沒看見。
2. 在 `incidents/INC-YYYYMMDD-<編號>.md` 建立通報單（`status: OPEN`），並提供 Failing Test Snippet。
3. `guard.rs` Defect Gate 自動封鎖全庫入庫，直至原工兵或核心團隊修復並標記為 `RESOLVED`！
