// ## Map
// Role: msa/log_lifecycle_dag — 日誌生命週期一跳終端壓縮與方針轉移狀態機微積木之量測系統分析（MSA）零件。
// Inputs: 起點索引、取代目標陣列、最大跳數、條目與方針轉移世代編號。
// Outputs: 參考實作結果、挑戰件（含只走一步未做傳遞閉包、誤殺新世代條目、無分野全域淘汰）結果、20,000 筆盲測。
// Invariants: 獨立於 src/log_lifecycle_dag.rs（不得 use rust_bricks::log_lifecycle_dag）；先過公開標準件才可量產品。
//
// @msa-standard-source: Graph Theory Transitive Reduction / Pointer-Jumping & Antigravity Charter R10-11 Log Lifecycle Specification
// @msa-blind: classify_epoch_pivot
// @msa-risk: 若取代鏈或 PIVOT 算錯，現況快照 ACTIVE-STATE.md 會把過期決策當有效或把現行決策誤判為過期
// @msa-escape: multi-station
// @msa-station-cost: high

pub mod standard {
    pub fn resolve_terminal_node(
        start_id: usize,
        targets: &[usize],
        max_hops: usize,
    ) -> Option<(usize, u8)> {
        if targets.is_empty() || targets.len() > 4096 || start_id >= targets.len() || max_hops < 1 || max_hops > 64 {
            return None;
        }
        let mut cur = start_id;
        let mut step = 0usize;
        while step < max_hops {
            let next = *targets.get(cur)?;
            if next >= targets.len() {
                return None;
            }
            if next == cur {
                return Some((cur, 0));
            }
            cur = next;
            step += 1;
        }
        Some((cur, 1))
    }

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
}

/// 挑戰件 1：只看直接取代（1 步），未追到傳遞取代鏈終端（`0 -> 1 -> 2` 停在 `1`）
pub mod challenge_single_hop_only {
    pub fn resolve_terminal_node(start_id: usize, targets: &[usize], max_hops: usize) -> Option<(usize, u8)> {
        if targets.is_empty() || targets.len() > 4096 || start_id >= targets.len() || max_hops < 1 || max_hops > 64 {
            return None;
        }
        let next = targets[start_id];
        if next >= targets.len() {
            return None;
        }
        Some((next, 0))
    }
    pub fn classify_epoch_pivot(eid: usize, esc: u16, pid: usize, psc: u16, sup: u8) -> Option<u8> {
        super::standard::classify_epoch_pivot(eid, esc, pid, psc, sup)
    }
}

/// 挑戰件 2：PIVOT 誤將 `entry_id <= pivot_id` 淘汰，導致 PIVOT 決策本身也被標成過期
pub mod challenge_pivot_kills_itself {
    pub fn resolve_terminal_node(start_id: usize, targets: &[usize], max_hops: usize) -> Option<(usize, u8)> {
        super::standard::resolve_terminal_node(start_id, targets, max_hops)
    }
    pub fn classify_epoch_pivot(eid: usize, esc: u16, pid: usize, psc: u16, sup: u8) -> Option<u8> {
        if eid < 1 || eid > 100_000 || pid > 100_000 || sup > 1 {
            return None;
        }
        if sup == 1 {
            return Some(1);
        }
        if pid > 0 && esc == psc && eid <= pid {
            return Some(2);
        }
        Some(0)
    }
}

/// 挑戰件 3：PIVOT 未檢查 `entry_scope == pivot_scope`，跨領域無差別淘汰舊決策
pub mod challenge_ignore_scope_on_pivot {
    pub fn resolve_terminal_node(start_id: usize, targets: &[usize], max_hops: usize) -> Option<(usize, u8)> {
        super::standard::resolve_terminal_node(start_id, targets, max_hops)
    }
    pub fn classify_epoch_pivot(eid: usize, _esc: u16, pid: usize, _psc: u16, sup: u8) -> Option<u8> {
        if eid < 1 || eid > 100_000 || pid > 100_000 || sup > 1 {
            return None;
        }
        if sup == 1 {
            return Some(1);
        }
        if pid > 0 && eid < pid {
            return Some(2);
        }
        Some(0)
    }
}

/// 挑戰件 4：遇到循環取代時未標記 `cycle_detected = 1`（靜默回傳 `0`）
pub mod challenge_silent_cycle {
    pub fn resolve_terminal_node(start_id: usize, targets: &[usize], max_hops: usize) -> Option<(usize, u8)> {
        super::standard::resolve_terminal_node(start_id, targets, max_hops).map(|(id, _)| (id, 0))
    }
    pub fn classify_epoch_pivot(eid: usize, esc: u16, pid: usize, psc: u16, sup: u8) -> Option<u8> {
        super::standard::classify_epoch_pivot(eid, esc, pid, psc, sup)
    }
}

/// 挑戰件 5：缺少定義域守衛（`max_hops == 0` 或 `entry_id == 0` 未回傳 `None`）
pub mod challenge_missing_domain_guard {
    pub fn resolve_terminal_node(start_id: usize, targets: &[usize], max_hops: usize) -> Option<(usize, u8)> {
        if start_id >= targets.len() {
            return None;
        }
        let mut cur = start_id;
        for _ in 0..max_hops {
            let next = *targets.get(cur)?;
            if next == cur {
                return Some((cur, 0));
            }
            cur = next;
        }
        Some((cur, 0))
    }
    pub fn classify_epoch_pivot(eid: usize, esc: u16, pid: usize, psc: u16, sup: u8) -> Option<u8> {
        if sup == 1 {
            return Some(1);
        }
        if pid > 0 && esc == psc && eid < pid {
            return Some(2);
        }
        Some(0)
    }
}

pub fn standard_parts_ok() -> bool {
    let dag_ok = standard::resolve_terminal_node(0, &[1, 2, 2], 8) == Some((2, 0))
        && standard::resolve_terminal_node(0, &[1, 0], 4) == Some((0, 1));
    let pivot_ok = standard::classify_epoch_pivot(2, 10, 6, 10, 0) == Some(2)
        && standard::classify_epoch_pivot(6, 10, 6, 10, 0) == Some(0)
        && standard::classify_epoch_pivot(2, 99, 6, 10, 0) == Some(0);
    dag_ok && pivot_ok
}

pub fn blind_check(
    candidate: fn(usize, u16, usize, u16, u8) -> Option<u8>,
    n: usize,
) -> (usize, usize) {
    let mut state: u64 = 0x1357_9BDF_2468_ACE0;
    let mut mismatches = 0usize;
    for _ in 0..n {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let eid = (state % 120) as usize;
        let esc = ((state >> 8) % 4) as u16;
        let pid = ((state >> 16) % 120) as usize;
        let psc = ((state >> 24) % 4) as u16;
        let sup = ((state >> 32) % 3) as u8;
        if candidate(eid, esc, pid, psc, sup) != standard::classify_epoch_pivot(eid, esc, pid, psc, sup) {
            mismatches += 1;
        }
    }
    (mismatches, n)
}
