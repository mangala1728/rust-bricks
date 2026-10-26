//! # Decision Minimax & Game Tree Search Micro-Bricks (對抗搜尋決策與極大極小微積木)
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 Alpha-Beta 剪枝視窗步進與 Beta 截斷判定、Negamax 負值極大分數轉換、
//! 靜態搜尋 (Quiescence Search) 停擺檢驗 (消滅地平線效應)、Zobrist 雜湊 O(1) 增量步進、
//! 置換表 (Transposition Table) 精確/上下界命中判讀、歷史啟發式著法排序加權、
//! 空著裁剪 (Null Move Pruning) 准許判定、MDP 折扣未來回報 (Bellman)、
//! Epsilon-Greedy 探索與利用動作選擇、Q-Learning 貝爾曼最優時序差分更新。

/// ## Map: Alpha-Beta 剪枝視窗步進與 Beta 截斷 (Cutoff) 判定
/// 回傳是否觸發 Beta 剪枝截斷 (Score >= Beta)
/// @outputs: 1
/// @domain: score [-1000000, 1000000]
/// @domain: beta [-1000000, 1000000]
pub fn decision_alphabeta_window_step(alpha: &mut i32, beta: i32, score: i32) -> bool {
    if score >= beta {
        return true; // 觸發 Beta 剪枝截斷
    }
    if score > *alpha {
        *alpha = score;
    }
    false
}

/// ## Transform: Negamax 負值極大化子節點回報反轉 (Negamax Score Inversion)
/// @outputs: 1
/// @domain: child_score [-1000000, 1000000]
pub fn decision_negamax_score_step(child_score: i32) -> i32 {
    let clamped = child_score.clamp(-1_000_000, 1_000_000);
    -clamped
}

/// ## Search: 靜態搜尋 (Quiescence Search) 停擺 (Stand-Pat) 檢定
/// 避免在劇烈吃子戰術局面因搜尋深度截斷而引發地平線效應 (Horizon Effect)
/// 回傳 (是否引發 Beta 截斷, 更新後 Alpha 分數)
/// @outputs: 2
/// @domain: static_eval [-1000000, 1000000]
/// @domain: beta [-1000000, 1000000]
pub fn decision_quiescence_stand_pat_eval(
    static_eval: i32,
    alpha: &mut i32,
    beta: i32,
) -> (bool, i32) {
    if static_eval >= beta {
        return (true, beta);
    }
    if static_eval > *alpha {
        *alpha = static_eval;
    }
    (false, *alpha)
}

/// ## Hash: Zobrist 棋局狀態雜湊 O(1) 增量異或更新
/// @outputs: 1
pub fn decision_zobrist_hash_update(
    current_hash: u64,
    piece_type: usize,
    square: usize,
    zobrist_table: &[u64],
) -> u64 {
    let index = piece_type.saturating_mul(64).saturating_add(square);
    if let Some(&random_val) = zobrist_table.get(index) {
        current_hash ^ random_val
    } else {
        current_hash
    }
}

/// ## Cache: 置換表 (Transposition Table) 條目邊界相容性比對
/// flag: 1 = EXACT, 2 = LOWERBOUND (alpha), 3 = UPPERBOUND (beta)
/// 回傳可剪枝之可用分數 Option<score>
/// @outputs: 1
/// @domain: entry_score [-1000000, 1000000]
pub fn decision_transposition_table_flag_check(
    entry_score: i32,
    entry_flag: u8,
    alpha: i32,
    beta: i32,
) -> Option<i32> {
    match entry_flag {
        1 => Some(entry_score), // EXACT
        2 => {
            // LOWERBOUND: 分數至少為 entry_score
            if entry_score >= beta { Some(entry_score) } else { None }
        }
        3 => {
            // UPPERBOUND: 分數至多為 entry_score
            if entry_score <= alpha { Some(entry_score) } else { None }
        }
        _ => None,
    }
}

/// ## Order: 歷史啟發式 (History Heuristic) 著法加權步進
/// 深度越深引發剪枝的著法獲得更高加分 (depth^2)
/// @outputs: 1
/// @domain: search_depth [1, 64]
pub fn decision_history_heuristic_score_step(
    stats_table: &mut [u32],
    move_idx: usize,
    search_depth: u8,
) {
    if move_idx >= stats_table.len() || search_depth == 0 {
        return;
    }
    let d = (search_depth as u32).min(64);
    let bonus = d * d;
    stats_table[move_idx] = stats_table[move_idx].saturating_add(bonus);
}

/// ## Prune: 空著裁剪 (Null Move Pruning) 安全防護條件檢驗
/// 在殘局、處於被將軍狀態、或剩餘深度過淺時嚴格禁止空著
/// @outputs: 1
/// @domain: depth [0, 64]
pub fn decision_null_move_pruning_allowed(
    depth: u8,
    is_in_check: bool,
    major_pieces_count: u32,
) -> bool {
    // 至少需 3 層深度、未被將軍、且擁有足夠大子 (防 zugzwang 逼和/受困陷阱)
    depth >= 3 && !is_in_check && major_pieces_count >= 2
}

/// ## Calc: 馬可夫決策過程 (MDP) 折扣未來回報 (Discounted Return G_t)
/// G_t = sum_{k=0}^{N-1} gamma^k * R_{t+k}
/// @outputs: 1
/// @domain: gamma [0.0, 1.0]
pub fn decision_discounted_future_reward(rewards: &[f32], gamma: f32) -> Option<f32> {
    if rewards.is_empty() || !gamma.is_finite() || gamma < 0.0 || gamma > 1.0 {
        return None;
    }
    let mut g = 0.0f32;
    let mut discount = 1.0f32;
    for &r in rewards {
        if !r.is_finite() { return None; }
        g += discount * r;
        discount *= gamma;
    }
    Some(g)
}

/// ## Select: Epsilon-Greedy 探索與利用動作選擇
/// @outputs: 1
/// @domain: epsilon [0.0, 1.0]
/// @domain: rand_sample [0.0, 1.0]
pub fn decision_epsilon_greedy_action(
    q_values: &[f32],
    epsilon: f32,
    rand_sample: f32,
    rand_action_idx: usize,
) -> Option<usize> {
    if q_values.is_empty() || !epsilon.is_finite() || !rand_sample.is_finite() {
        return None;
    }
    let n = q_values.len();
    if rand_sample < epsilon {
        // 探索 (Exploration)
        Some(rand_action_idx % n)
    } else {
        // 利用 (Exploitation: 取最大 Q 值之動作)
        let mut best_idx = 0;
        let mut best_q = f32::NEG_INFINITY;
        for (i, &q) in q_values.iter().enumerate() {
            if q > best_q {
                best_q = q;
                best_idx = i;
            }
        }
        Some(best_idx)
    }
}

/// ## Update: Q-Learning 貝爾曼最優時序差分更新 (Bellman Optimality TD Step)
/// Q(s,a) = Q(s,a) + alpha * (reward + gamma * max_Q(s',a') - Q(s,a))
/// @outputs: 1
/// @domain: alpha [0.0, 1.0]
/// @domain: gamma [0.0, 1.0]
pub fn decision_bellman_optimality_step(
    current_q: f32,
    reward: f32,
    max_next_q: f32,
    alpha: f32,
    gamma: f32,
) -> f32 {
    if !current_q.is_finite() || !reward.is_finite() || !max_next_q.is_finite() ||
       !alpha.is_finite() || !gamma.is_finite() {
        return 0.0;
    }
    let target = reward + gamma * max_next_q;
    let td_error = target - current_q;
    current_q + alpha * td_error
}
