//! # Reinforcement Learning & Policy Gradient Micro-Bricks (強化學習與策略梯度微積木)
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 PPO (Proximal Policy Optimization) 截斷目標函數步進、
//! GAE (Generalized Advantage Estimation) 廣義優勢估計遞迴單步、
//! 高斯策略 (Gaussian Policy) 連續動作對數似然與梯度、
//! 策略分佈香農熵 (Entropy Bonus) 探索度計算、Gumbel-Softmax 重參數化取樣、
//! 折扣回報 (Discounted Return) 反向累加、目標網路 Polyak 軟更新、
//! 時序差分 TD(0) 價值評估更新步進。

/// ## Map: PPO 截斷代理損失單步計算 (PPO Clipped Surrogate Objective)
/// 公式：L^{CLIP} = min(r * A, clip(r, 1 - eps, 1 + eps) * A)
/// 其中 r = exp(log_prob - old_log_prob) 為重要性採樣比率，A 為優勢值 (Advantage)
/// @outputs: 1
/// @domain: log_prob [-1e3, 1e3], old_log_prob [-1e3, 1e3], advantage [-1e6, 1e6], epsilon [0.0, 1.0]
pub fn rl_ppo_clipped_surrogate_loss(
    log_prob: f64,
    old_log_prob: f64,
    advantage: f64,
    epsilon: f64,
) -> f64 {
    if !log_prob.is_finite() || !old_log_prob.is_finite() || !advantage.is_finite() || !epsilon.is_finite() {
        return 0.0;
    }
    if epsilon <= 0.0 || epsilon >= 1.0 {
        return 0.0;
    }
    let log_ratio = (log_prob - old_log_prob).clamp(-20.0, 20.0);
    let ratio = log_ratio.exp();
    let surr1 = ratio * advantage;
    let clipped_ratio = ratio.clamp(1.0 - epsilon, 1.0 + epsilon);
    let surr2 = clipped_ratio * advantage;

    if surr1 < surr2 { surr1 } else { surr2 }
}

/// ## Step: GAE (Generalized Advantage Estimation) 廣義優勢估計單步遞迴
/// 公式：A_t = delta_t + (gamma * lambda) * A_{t+1}
/// 其中 TD-error delta_t = r_t + gamma * V(s_{t+1}) * (1 - done) - V(s_t)
/// @outputs: 1
/// @domain: r_t [-1e6, 1e6], v_current [-1e6, 1e6], v_next [-1e6, 1e6], gamma [0.0, 1.0], lambda [0.0, 1.0]
pub fn rl_gae_advantage_step(
    r_t: f64,
    v_current: f64,
    v_next: f64,
    done: bool,
    next_advantage: f64,
    gamma: f64,
    lambda: f64,
) -> f64 {
    if !r_t.is_finite() || !v_current.is_finite() || !v_next.is_finite() || !next_advantage.is_finite() {
        return 0.0;
    }
    if gamma < 0.0 || gamma > 1.0 || lambda < 0.0 || lambda > 1.0 {
        return 0.0;
    }
    let non_terminal = if done { 0.0 } else { 1.0 };
    let delta_t = r_t + gamma * v_next * non_terminal - v_current;
    delta_t + gamma * lambda * non_terminal * next_advantage
}

/// ## Calc: 一維連續動作高斯策略對數似然 (Gaussian Policy Log-Likelihood)
/// 公式：log pi(a) = -0.5 * ((a - mu) / sigma)^2 - ln(sigma) - 0.5 * ln(2 * pi)
/// @outputs: 1
/// @domain: action [-1e6, 1e6], mean [-1e6, 1e6], std [1e-6, 1e6]
pub fn rl_gaussian_policy_log_prob(action: f64, mean: f64, std: f64) -> f64 {
    if !action.is_finite() || !mean.is_finite() || !std.is_finite() || std <= 0.0 {
        return -1e9;
    }
    let z = (action - mean) / std;
    const HALF_LOG_2PI: f64 = 0.9189385332046727; // 0.5 * ln(2 * pi)
    -0.5 * z * z - std.ln() - HALF_LOG_2PI
}

/// ## Calc: 離散機率分佈香農熵 (Policy Distribution Shannon Entropy)
/// 公式：H(pi) = -sum(p_i * ln(p_i))，用於強化學習策略探索激勵項 (Entropy Bonus)
/// @outputs: 1
pub fn rl_discrete_policy_entropy(probs: &[f64]) -> f64 {
    if probs.is_empty() {
        return 0.0;
    }
    let mut entropy = 0.0;
    for &p in probs.iter() {
        if !p.is_finite() || p <= 0.0 {
            continue;
        }
        if p > 1.0 {
            return 0.0;
        }
        entropy -= p * p.ln();
    }
    if entropy < 0.0 { 0.0 } else { entropy }
}

/// ## Sample: Gumbel-Max 離散動作重參數化抽樣步進 (Gumbel-Max Perturbed Score)
/// 公式：score_i = logit_i - ln(-ln(u_i))，其中 u_i in (0, 1) 為均勻分佈隨機數
/// @outputs: 1
/// @domain: logit [-1e3, 1e3], uniform_u [1e-7, 0.9999999]
pub fn rl_gumbel_sample_perturb_score(logit: f64, uniform_u: f64) -> f64 {
    if !logit.is_finite() || !uniform_u.is_finite() || uniform_u <= 0.0 || uniform_u >= 1.0 {
        return -1e9;
    }
    let gumbel_noise = -(-uniform_u.ln()).ln();
    logit + gumbel_noise
}

/// ## Step: 反向時序折扣回報累加 (Discounted Cumulative Return Step)
/// 公式：G_t = r_t + gamma * G_{t+1} * (1 - done)
/// @outputs: 1
/// @domain: reward [-1e6, 1e6], next_return [-1e6, 1e6], gamma [0.0, 1.0]
pub fn rl_discounted_return_step(
    reward: f64,
    next_return: f64,
    done: bool,
    gamma: f64,
) -> f64 {
    if !reward.is_finite() || !next_return.is_finite() || gamma < 0.0 || gamma > 1.0 {
        return 0.0;
    }
    let mask = if done { 0.0 } else { 1.0 };
    reward + gamma * next_return * mask
}

/// ## Step: 目標網路參數 Polyak 軟更新步進 (Polyak Target Network Soft Update)
/// 公式：theta_target = tau * theta_source + (1 - tau) * theta_target
/// 常用於 DDPG, TD3, SAC 演算法
/// @outputs: 1
/// @domain: source [-1e9, 1e9], target [-1e9, 1e9], tau [0.0, 1.0]
pub fn rl_polyak_target_soft_update(source: f64, target: f64, tau: f64) -> f64 {
    if !source.is_finite() || !target.is_finite() || !tau.is_finite() || tau < 0.0 || tau > 1.0 {
        return target;
    }
    tau * source + (1.0 - tau) * target
}

/// ## Step: 時序差分 TD(0) 狀態價值評估更新單步 (TD(0) Value Update Step)
/// 公式：V(s) = V(s) + alpha * (r + gamma * V(s') * (1 - done) - V(s))
/// @outputs: 1
/// @domain: v_s [-1e6, 1e6], reward [-1e6, 1e6], v_next [-1e6, 1e6], alpha [0.0, 1.0], gamma [0.0, 1.0]
pub fn rl_td0_value_update_step(
    v_s: f64,
    reward: f64,
    v_next: f64,
    done: bool,
    alpha: f64,
    gamma: f64,
) -> f64 {
    if !v_s.is_finite() || !reward.is_finite() || !v_next.is_finite() || alpha < 0.0 || alpha > 1.0 || gamma < 0.0 || gamma > 1.0 {
        return v_s;
    }
    let non_terminal = if done { 0.0 } else { 1.0 };
    let td_target = reward + gamma * v_next * non_terminal;
    let td_error = td_target - v_s;
    v_s + alpha * td_error
}

/// ## Loss: 價值網路 Huber 平滑損失 (Value Huber Loss Step)
/// 防範大 TD-error 導致梯度爆炸
/// @outputs: 1
/// @domain: v_pred [-1e6, 1e6], v_target [-1e6, 1e6], delta [1e-6, 100.0]
pub fn rl_value_huber_loss(v_pred: f64, v_target: f64, delta: f64) -> f64 {
    if !v_pred.is_finite() || !v_target.is_finite() || delta <= 0.0 {
        return 0.0;
    }
    let error = (v_pred - v_target).abs();
    if error <= delta {
        0.5 * error * error
    } else {
        delta * (error - 0.5 * delta)
    }
}
