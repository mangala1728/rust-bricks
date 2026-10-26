// ## Map
// Role: ai/decision_mcts — 零依賴蒙地卡羅樹狀搜尋 (MCTS) 決策核心：UCB1 評分、樹路徑選擇、反向傳播與溫度策略映射微積木。
// Inputs: 節點勝率/獎勵、拜訪次數、父節點拜訪次數、探索常數 (C)、搜索路徑、採樣溫度。
// Outputs: UCB1 分數、最佳子節點索引、反向更新狀態、行動機率分布 (Policy)。
// Invariants: 零外部依賴、純 Rust 實現、未拜訪節點最高優先級保證、除零剛性防護、單一函式 <= 60 行。

/// @outputs: 1
/// @domain: exploration_c [0.0..10.0]
pub fn mcts_ucb1_score(wins: f32, visits: u32, parent_visits: u32, exploration_c: f32) -> f32 {
    if visits == 0 {
        return f32::INFINITY;
    }
    if parent_visits == 0 || exploration_c < 0.0 || !wins.is_finite() {
        return wins / (visits as f32);
    }
    let exploitation = wins / (visits as f32);
    let exploration = exploration_c * ((parent_visits as f32).ln() / (visits as f32)).sqrt();
    exploitation + exploration
}

/// @outputs: 2
/// @domain: wins.len() [1..1000]
pub fn mcts_select_best_child(
    wins: &[f32],
    visits: &[u32],
    parent_visits: u32,
    exploration_c: f32,
) -> Option<usize> {
    let n = wins.len();
    if n == 0 || visits.len() != n {
        return None;
    }
    let mut best_idx = 0;
    let mut best_score = f32::NEG_INFINITY;

    for i in 0..n {
        let score = mcts_ucb1_score(wins[i], visits[i], parent_visits, exploration_c);
        if score.is_infinite() && score.is_sign_positive() {
            return Some(i); // 優先探索未拜訪之子節點
        }
        if score > best_score {
            best_score = score;
            best_idx = i;
        }
    }
    Some(best_idx)
}

/// @outputs: 2
/// @domain: path_indices.len() [1..256]
pub fn mcts_backpropagate_step(
    wins: &mut [f32],
    visits: &mut [u32],
    path_indices: &[usize],
    reward: f32,
) -> bool {
    if path_indices.is_empty() || !reward.is_finite() {
        return false;
    }
    let max_nodes = wins.len().min(visits.len());
    for &idx in path_indices {
        if idx >= max_nodes {
            return false;
        }
        wins[idx] += reward;
        visits[idx] = visits[idx].saturating_add(1);
    }
    true
}

/// @outputs: 2
/// @domain: temperature [0.0..10.0]
pub fn mcts_visit_count_temperature_policy(visits: &[u32], temperature: f32, out_policy: &mut [f32]) -> bool {
    let n = visits.len();
    if n == 0 || out_policy.len() != n || temperature < 0.0 || !temperature.is_finite() {
        return false;
    }
    for p in out_policy.iter_mut() {
        *p = 0.0;
    }

    // 溫度趨近於 0: Argmax 貪婪策略
    if temperature < 1e-4 {
        let mut max_visits = 0;
        let mut best_idx = 0;
        for (i, &v) in visits.iter().enumerate() {
            if v > max_visits {
                max_visits = v;
                best_idx = i;
            }
        }
        out_policy[best_idx] = 1.0;
        return true;
    }

    let inv_tau = 1.0 / temperature;
    let mut sum = 0.0f32;
    for (i, &v) in visits.iter().enumerate() {
        let val = (v as f32).powf(inv_tau);
        out_policy[i] = val;
        sum += val;
    }
    if sum <= 0.0 || !sum.is_finite() {
        return false;
    }
    let inv_sum = 1.0 / sum;
    for p in out_policy.iter_mut() {
        *p *= inv_sum;
    }
    true
}
