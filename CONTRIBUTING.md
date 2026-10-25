# rust-bricks 外部專案貢獻與防撞車規範（Contribution Guidelines）

歡迎其他各專案（如 `ai-interpreter`、字幕生成、量化分析等）向 `rust-bricks` 貢獻高品質純 Rust 微積木！
為確保多專案並行開發時**「零依賴、不撞車、代碼可追溯、去重收斂」**，所有外部貢獻者必須遵循以下剛性協議：

---

## 1. 嚴禁直接推送 `main` 分支

`main` 分支由核心策展管家（Core Curator）統一守護。外部專案一律使用貢獻分支：

- **分支命名格式**：`contrib/<專案名>-<功能名稱>`
  - 例如：`contrib/ai-interpreter-touch-debounce`
  - 例如：`contrib/video-subtitle-bezier-ease`

---

## 2. 外部貢獻者硬體級五項剛性紅線

任何提交至貢獻分支的代碼，必須通過 `cargo run -- guard` 審查，觸碰以下紅線者無法合併：

1. 🔴 **零外部依賴**：`Cargo.toml` 的 `[dependencies]` 必須為空！嚴禁 `cargo add` 任何第三方 crate。
2. 🔴 **NASA JPL 60 行原則**：具名函式長度超過 60 行一律阻擋！請拆解為子積木。
3. 🔴 **06/09 號合約守衛**：
   - 原始檔開頭必備 `// ## Map` 接線圖。
   - 具名函式必標 `/// @outputs: N`。
   - 數值參數必標 `/// @domain: param [min..max]`，且**函式第一行必須寫執行期守衛**。
   - 嚴禁 `static mut`；嚴禁遞迴（樹/圖走訪一律使用有界佇列）。
4. 🔴 **危險操作安全證明**：
   - 嚴禁裸寫 `.unwrap()`、`.expect()` 或 `panic!`。
   - 若經過邏輯前置校驗確信安全，必須在前一行加註 `// @safety-proof: <證明原因>`。
5. 🔴 **去重與排序守衛**：
   - 提交前請研讀 `CATALOG.yaml`，禁止重複發明現有函式。
   - `src/lib.rs` 的模組宣告必須依照英文字母順序排列。

---

## 3. 貢獻驗收與合併流程

```bash
# 1. 本地建立獨立貢獻分支
git checkout -b contrib/<專案名>-<功能>

# 2. 開發完成後執行本地驗收
cargo test                  # 單元測試全綠
cargo run -- guard          # 機械合約檢查通過
cargo run -- catalog        # 更新機器索引目錄

# 3. 推送至遠端貢獻分支
git push origin contrib/<專案名>-<功能>
```

推送後，`rust-bricks` 核心管家會在巡檢時（`cargo run -- patrol`）自動檢出您的分支，執行體檢、去重審查與簽署本地決策收據，並安全併入 `main` 主分支！
