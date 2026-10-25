# 缺陷通報與懸案追蹤制度（Incident Ledger & Defect Gate）

本目錄為 `rust-bricks` 專屬之缺陷通報與懸案追蹤庫（Incident Tracking）。
依據工作流核心憲章與 `guard.rs` 第三道硬體級防線規定：
> **「排障優先於新功能開發」**：若本目錄存在任何 `status: OPEN` 或 `status: UNRESOLVED` 的通報單，守護程序（`cargo run -- guard` 與 `pre-commit` hook）將**強制阻擋任何新代碼入庫**，迫使工兵代理優先修復現有缺陷。

---

## 缺陷通報單生命週期（Incident Lifecycle）

```mermaid
flowchart LR
    A["OPEN\n(人類或調用代理通報)"] --> B["INVESTIGATING\n(工兵確認重現與排障)"]
    B --> C["RESOLVED\n(附帶迴歸測試與修復 Commit)"]
```

1. **OPEN**：通報者（人類或外部專案代理）發現函式計算錯誤、記憶體異常、邊界溢出或文檔不符，按範本開單。
2. **INVESTIGATING**：負責工兵接單排障中（守衛仍持續阻擋新功能入庫）。
3. **RESOLVED**：工兵撰寫最小失敗迴歸測試（Red Test）、實施修復、全數通過（Green Test），並在通報單填寫 `resolution_commit` 後將狀態改為 `RESOLVED`，守衛自動放行。

---

## 缺陷通報單標準範本（Defect Report Template）

通報檔案命名格式：`incidents/INC-YYYYMMDD-<編號>.md`

```yaml
---
id: INC-20261026-01
target_brick: math_time::dasha_proportion_days
reporter: agent-autosubtitle
severity: SEV2-WRONG-OUTPUT # SEV1-FATAL | SEV2-WRONG-OUTPUT | SEV3-PERF | SEV4-DOC
status: OPEN                # OPEN | INVESTIGATING | RESOLVED
created_at: 2026-10-26T01:30:00+08:00
resolved_at: null
resolution_commit: null
---

# 缺陷通報：math_time::dasha_proportion_days 計算浮點精度異常

## 1. 發現情境與影響
調用專案在處理跨度 3600 天時，傳入負數週期未返回 None。

## 2. 最小重現代碼（Reproduction Snippet）
```rust
let res = math_time::dasha_proportion_days(-10.0, 5.0, 100.0);
assert_eq!(res, None); // 當前傳回了 Some(-0.5)
```

## 3. 預期行為 vs 實際行為
- **預期**：依合約 `@domain` 守衛，負數輸入應立即返回 `None`。
- **實際**：計算出負數天數，破壞下游時間軸單調性。

## 4. 排障日誌與修復方案（由工兵填寫）
（排障完成後，填入修復 commit hash 並將 status 改為 RESOLVED）
```
