// ## Map
// Role: core/log_lifecycle_dag — 日誌取代鏈一跳終端壓縮（Pointer-Jumping）與領域 PIVOT 世代淘汰狀態判定微積木。
// Inputs: start_id: usize, targets: &[usize], max_hops: usize, entry/pivot metadata。
// Outputs: Option<(usize, u8)>, Option<u8>。
// Invariants: 零外部依賴、第一行領域邊界守衛、顯式有界迴圈 <= 64、單一函式 <= 60 行、無遞迴。

/// @outputs: 3
/// @domain: start_id [0..4095]
/// @domain: targets [1..4096]
/// @domain: max_hops [1..64]
pub fn resolve_terminal_node(
    start_id: usize,
    targets: &[usize],
    max_hops: usize,
) -> Option<(usize, u8)> {
    if start_id > 4095
        || targets.is_empty()
        || targets.len() > 4096
        || start_id >= targets.len()
        || max_hops < 1
        || max_hops > 64
    {
        return None;
    }
    let mut cur = start_id;
    let mut hop = 0usize;
    while hop < max_hops {
        let next = targets[cur];
        if next >= targets.len() {
            return None;
        }
        if next == cur {
            return Some((cur, 0));
        }
        cur = next;
        hop += 1;
    }
    if targets[cur] == cur {
        return Some((cur, 0));
    }
    Some((cur, 1))
}

/// @outputs: 4
/// @domain: entry_id [1..100000]
/// @domain: entry_scope [0..65535]
/// @domain: pivot_id [0..100000]
/// @domain: pivot_scope [0..65535]
/// @domain: already_superseded [0..1]
pub fn classify_epoch_pivot(
    entry_id: usize,
    entry_scope: u16,
    pivot_id: usize,
    pivot_scope: u16,
    already_superseded: u8,
) -> Option<u8> {
    if entry_id < 1 || entry_id > 100_000 || pivot_id > 100_000 || already_superseded > 1 {
        return None;
    }
    if already_superseded == 1 {
        return Some(1);
    }
    if pivot_id > 0 && entry_scope == pivot_scope && entry_id < pivot_id {
        return Some(2);
    }
    Some(0)
}
