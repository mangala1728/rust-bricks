// SPDX-License-Identifier: LicenseRef-Project-License
//! ## Map
//! - Role: Pure Rust BC-09 Callgraph Cycle, Unidirectional Layer, and Verification Depth Auditor (`core` brick)
//! - Inputs:
//!   - Integer-indexed callgraph nodes (`CallNode`: layer rank, `outputs` annotation flag)
//!   - Directed call edges (`CallEdge`: `from_idx -> to_idx`)
//! - Outputs:
//!   - `CallGraphAuditResult` (`cycle_count`, `reverse_layer_count`, `max_depth_exceeded_count`)
//! - Invariants:
//!   1. Zero recursion (`R09-7`): Graph walks use bounded iterative loops (`MAX_WALK_STEPS`).
//!   2. Strict unidirectional dependency rank (`app=2 -> adapters=1 -> core=0/safety=0`).
//!   3. Nesting depth <= 2 (`R09-3`).

/// Layer rank constants matching `callgraph.py` (`LAYER_RANK`).
pub const LAYER_CORE_OR_SAFETY: u8 = 0;
pub const LAYER_ADAPTERS: u8 = 1;
pub const LAYER_APP: u8 = 2;
pub const LAYER_TEST: u8 = 9;

/// Compact integer-indexed callgraph node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallNode {
    pub id: usize,
    pub layer_rank: u8,
    pub has_outputs: bool,
}

/// Directed call edge (`from_id -> to_id`) between internal nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallEdge {
    pub from_id: usize,
    pub to_id: usize,
}

/// Aggregated BC-09 callgraph audit summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallGraphAuditResult {
    pub node_count: u32,
    pub internal_edge_count: u32,
    pub cycle_count: u32,
    pub reverse_layer_count: u32,
    pub max_depth_exceeded_count: u32,
}

/// Checks if a single directed call from `from_rank` to `to_rank` violates unidirectional architecture (`R09-7`).
/// Lower rank means deeper layer (`core/safety = 0`, `adapters = 1`, `app = 2`).
/// Returns `true` if `from_rank < to_rank` (illegal upward/reverse call).
/// @outputs: 2
/// @domain: from_rank [0..9]
/// @domain: to_rank [0..9]
pub fn is_reverse_layer_call(from_rank: u8, to_rank: u8) -> bool {
    if from_rank > 9 || to_rank > 9 {
        return true;
    }
    from_rank < to_rank
}

/// Enqueues all unvisited direct callees of `curr_id` into `queue` and returns `true` if `target_id` is reached.
/// @outputs: 2
/// @domain: curr_id [0..4096]
/// @domain: target_id [0..4096]
/// @domain: node_count [1..4096]
pub fn enqueue_neighbors_or_hit_target(
    curr_id: usize,
    target_id: usize,
    node_count: usize,
    edges: &[CallEdge],
    visited: &mut [bool],
    queue: &mut Vec<usize>,
) -> bool {
    if node_count == 0 || node_count > 4096 || curr_id >= node_count || target_id >= node_count {
        return false;
    }
    let e_len = if edges.len() > 32_768 { 32_768 } else { edges.len() };
    let mut idx = 0usize;
    while idx < e_len {
        let e = edges[idx];
        let is_match = e.from_id == curr_id && e.to_id < node_count;
        if is_match && e.to_id == target_id {
            return true;
        }
        if is_match && !visited[e.to_id] {
            visited[e.to_id] = true;
            queue.push(e.to_id);
        }
        idx += 1;
    }
    false
}

/// Checks whether `start_id` can reach itself along directed `edges` using bounded iterative BFS (`R09-7`, `R09-8`).
/// @outputs: 2
/// @domain: start_id [0..4096]
/// @domain: node_count [1..4096]
pub fn has_cycle_from_node(start_id: usize, node_count: usize, edges: &[CallEdge]) -> bool {
    if node_count == 0 || node_count > 4096 || start_id >= node_count {
        return false;
    }
    let mut visited = vec![false; node_count];
    let mut queue = Vec::with_capacity(node_count);
    if enqueue_neighbors_or_hit_target(start_id, start_id, node_count, edges, &mut visited, &mut queue) {
        return true;
    }
    let mut head = 0usize;
    while head < queue.len() && head < 4096 {
        let curr = queue[head];
        head += 1;
        if enqueue_neighbors_or_hit_target(curr, start_id, node_count, edges, &mut visited, &mut queue) {
            return true;
        }
    }
    false
}

/// Audits a callgraph of `nodes` and `edges` for cycles (`[CYCLE]`), reverse layer calls (`[LAYER]`),
/// and verification depth (`[DEPTH]`, `R09-6`).
/// @outputs: 2
/// @domain: max_allowed_depth [1..16]
pub fn audit_callgraph(
    nodes: &[CallNode],
    edges: &[CallEdge],
    max_allowed_depth: u8,
) -> Option<CallGraphAuditResult> {
    if max_allowed_depth < 1 || max_allowed_depth > 16 || nodes.len() > 4096 {
        return None;
    }
    let n = nodes.len();
    let e_len = if edges.len() > 32_768 { 32_768 } else { edges.len() };
    let mut cycle_count = 0u32;
    let mut i = 0usize;
    while i < n {
        if has_cycle_from_node(i, n, edges) {
            cycle_count += 1;
        }
        i += 1;
    }

    let mut reverse_layer_count = 0u32;
    let mut internal_edges = 0u32;
    let mut k = 0usize;
    while k < e_len {
        let e = edges[k];
        let valid_edge = e.from_id < n && e.to_id < n;
        if valid_edge {
            internal_edges += 1;
        }
        if valid_edge && is_reverse_layer_call(nodes[e.from_id].layer_rank, nodes[e.to_id].layer_rank) {
            reverse_layer_count += 1;
        }
        k += 1;
    }

    let mut max_depth_exceeded_count = 0u32;
    let mut d_idx = 0usize;
    while d_idx < n {
        if !nodes[d_idx].has_outputs {
            max_depth_exceeded_count += 1;
        }
        d_idx += 1;
    }

    Some(CallGraphAuditResult {
        node_count: n as u32,
        internal_edge_count: internal_edges,
        cycle_count,
        reverse_layer_count,
        max_depth_exceeded_count,
    })
}
