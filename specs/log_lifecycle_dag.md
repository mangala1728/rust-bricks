# 模組名稱: 日誌生命週期一跳終端壓縮與方針轉移狀態機微積木 (Log Lifecycle 1-Hop DAG & Pivot Epoch Resolver)

## 1. 目標定位
- 目標模組檔: `src/log_lifecycle_dag.rs`
- 目標測試檔: `tests/msa_log_lifecycle_dag.rs`
- 功能簡述: 提供外部決策／史實日誌（如 `R10-11` Materialized Active Truth Snapshot）所需之**有界非遞迴傳遞取代鏈一跳終端壓縮（1-Hop Terminal Path Compression）**與**領域大方針轉移（Domain Pivot Epoch）批次狀態判定**。
- 範圍限制: 純無狀態切片與整數索引運算，不涉及檔案讀寫或字串解析。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: 日誌取代鏈一跳終端壓縮（Pointer-Jumping）與領域 PIVOT 世代淘汰狀態判定
// Inputs: start_id: usize, targets: &[usize], max_hops: usize, entry/pivot metadata
// Outputs: Option<(usize, u8)>, Option<u8>
// Invariants: 0 external dependencies, line 1 domain guard, bounded loop <= 64, functions <= 60 lines

/// @outputs: 3
/// @domain: start_id [0..4095]
/// @domain: max_hops [1..64]
pub fn resolve_terminal_node(
    start_id: usize,
    targets: &[usize],
    max_hops: usize,
) -> Option<(usize, u8)>;

/// @outputs: 4
/// @domain: entry_id [1..100000]
/// @domain: pivot_id [0..100000]
/// @domain: already_superseded [0..1]
pub fn classify_epoch_pivot(
    entry_id: usize,
    entry_scope: u16,
    pivot_id: usize,
    pivot_scope: u16,
    already_superseded: u8,
) -> Option<u8>;
```

## 3. 演算法與邊界 (Domain Guards)
- `resolve_terminal_node(start_id, targets, max_hops)`:
  - 若 `targets.is_empty() || targets.len() > 4096 || start_id >= targets.len() || max_hops < 1 || max_hops > 64`，回傳 `None`。
  - `targets[k] == k` 代表節點 `k` 為現行有效終端（Active Terminal）；`targets[k] = next`（`next != k`）代表節點 `k` 被 `next` 取代。
  - 以顯式有界迴圈走訪至多 `max_hops` 步：
    - 若某步 `next >= targets.len()`，回傳 `None`。
    - 若 `next == cur`，代表已抵達終端有效節點，回傳 `Some((cur, 0))`（`0` = 無環收斂）。
    - 若走滿 `max_hops` 步仍未收斂（含循環取代 `A -> B -> A` 或超過跳數上限），回傳 `Some((cur, 1))`（`1` = 偵測到循環或超限）。
- `classify_epoch_pivot(entry_id, entry_scope, pivot_id, pivot_scope, already_superseded)`:
  - 若 `entry_id < 1 || entry_id > 100_000 || pivot_id > 100_000 || already_superseded > 1`，回傳 `None`。
  - 若 `already_superseded == 1`，回傳 `Some(1)`（`1 = SUPERSEDED`）。
  - 若 `pivot_id > 0 && entry_scope == pivot_scope && entry_id < pivot_id`，回傳 `Some(2)`（`2 = PIVOTED_OBSOLETE`，同領域舊世代條目因大方針轉移而淘汰）。
  - 其餘回傳 `Some(0)`（`0 = ACTIVE`）。

## 4. 確定性黃金值測試要求
```golden
// @source: Pointer-jumping / path compression on DAG: 0 -> 1 -> 2 -> 2 converges to (2, 0) in 2 hops
resolve_terminal_node(0, &[1, 2, 2], 8) == Some((2, 0))
// @source: Already-terminal node 2 -> 2 converges in 0 hops to (2, 0)
resolve_terminal_node(2, &[1, 2, 2], 8) == Some((2, 0))
// @source: 2-cycle 0 -> 1 -> 0 hits max_hops and flags cycle_detected = 1
resolve_terminal_node(0, &[1, 0], 4) == Some((0, 1))
// @source: R10-11 Domain Pivot rule — older entry (id=2) in same scope (10) before pivot (id=6, scope=10) is PIVOTED_OBSOLETE (2)
classify_epoch_pivot(2, 10, 6, 10, 0) == Some(2)
// @source: R10-11 Domain Pivot rule — pivot entry itself (id=6) or newer entry (id=7) in same scope remains ACTIVE (0)
classify_epoch_pivot(6, 10, 6, 10, 0) == Some(0)
classify_epoch_pivot(7, 10, 6, 10, 0) == Some(0)
// @source: R10-11 Domain Pivot rule — older entry (id=2) in different scope (99 != 10) remains ACTIVE (0)
classify_epoch_pivot(2, 99, 6, 10, 0) == Some(0)
// @source: R10-11 Supersession precedence — already superseded entry returns SUPERSEDED (1)
classify_epoch_pivot(2, 10, 6, 10, 1) == Some(1)
// @source: Domain guards (out-of-bounds start_id, zero max_hops, invalid already_superseded)
resolve_terminal_node(3, &[0, 1, 2], 8).is_none()
resolve_terminal_node(0, &[0, 1, 2], 0).is_none()
classify_epoch_pivot(0, 10, 6, 10, 0).is_none()
classify_epoch_pivot(2, 10, 6, 10, 2).is_none()
```
