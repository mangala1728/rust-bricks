// ## Map
// Role: ai/autograd_train — 極簡神經網路訓練、標量自動求導 (Micrograd)、BPE 詞元合併與餘弦退火排程微積木。
// Inputs: 計算圖標量節點與梯度、詞元切片、因果遮罩矩陣、學習率超參數、梯度向量。
// Outputs: 標量梯度對 (grad_a, grad_b)、BPE 合併詞元列、遮罩後注意力權重、退火學習率、裁剪後梯度範數。
// Invariants: 零外部依賴、純 Rust 實現、除零與 NaN 剛性防護、有界長度切片、單一函式 <= 60 行。

/// @outputs: 2
pub fn autograd_scalar_add(a_val: f32, b_val: f32, out_grad: f32) -> (f32, f32) {
    if !a_val.is_finite() || !b_val.is_finite() || !out_grad.is_finite() {
        return (0.0, 0.0);
    }
    (out_grad, out_grad)
}

/// @outputs: 2
pub fn autograd_scalar_mul(a_val: f32, b_val: f32, out_grad: f32) -> (f32, f32) {
    if !a_val.is_finite() || !b_val.is_finite() || !out_grad.is_finite() {
        return (0.0, 0.0);
    }
    (b_val * out_grad, a_val * out_grad)
}

/// @outputs: 1
/// @domain: exp [-100.0..100.0]
pub fn autograd_scalar_pow(val: f32, exp: f32, out_grad: f32) -> f32 {
    if !val.is_finite() || !exp.is_finite() || !out_grad.is_finite() || val == 0.0 {
        return 0.0;
    }
    let dval = exp * val.powf(exp - 1.0);
    if dval.is_finite() {
        dval * out_grad
    } else {
        0.0
    }
}

/// @outputs: 1
pub fn autograd_scalar_relu(val: f32, out_grad: f32) -> f32 {
    if !val.is_finite() || !out_grad.is_finite() {
        return 0.0;
    }
    if val > 0.0 {
        out_grad
    } else {
        0.0
    }
}

/// @outputs: 2
/// @domain: tokens.len() [2..65536]
pub fn bpe_find_most_frequent_pair(tokens: &[u32]) -> Option<(u32, u32, usize)> {
    let n = tokens.len();
    if n < 2 {
        return None;
    }
    let mut best_pair = (0u32, 0u32);
    let mut max_count = 0usize;

    for i in 0..(n - 1) {
        let p0 = tokens[i];
        let p1 = tokens[i + 1];
        let mut count = 0usize;
        for j in 0..(n - 1) {
            if tokens[j] == p0 && tokens[j + 1] == p1 {
                count += 1;
            }
        }
        if count > max_count {
            max_count = count;
            best_pair = (p0, p1);
        }
    }
    if max_count > 0 {
        Some((best_pair.0, best_pair.1, max_count))
    } else {
        None
    }
}

/// @outputs: 2
pub fn bpe_merge_pair(tokens: &[u32], pair: (u32, u32), new_id: u32, out: &mut [u32]) -> usize {
    let n = tokens.len();
    if n == 0 || out.is_empty() {
        return 0;
    }
    let mut in_idx = 0usize;
    let mut out_idx = 0usize;

    while in_idx < n && out_idx < out.len() {
        if in_idx + 1 < n && tokens[in_idx] == pair.0 && tokens[in_idx + 1] == pair.1 {
            out[out_idx] = new_id;
            in_idx += 2;
        } else {
            out[out_idx] = tokens[in_idx];
            in_idx += 1;
        }
        out_idx += 1;
    }
    out_idx
}

/// @outputs: 2
/// @domain: seq_len [1..2048]
pub fn causal_attention_mask_apply(scores: &mut [f32], seq_len: usize, neg_inf: f32) -> bool {
    let total_elements = seq_len * seq_len;
    if seq_len == 0 || scores.len() < total_elements || !neg_inf.is_finite() {
        return false;
    }
    for row in 0..seq_len {
        for col in (row + 1)..seq_len {
            scores[row * seq_len + col] = neg_inf;
        }
    }
    true
}

/// @outputs: 1
/// @domain: base_lr [1e-7..10.0]
/// @domain: min_lr [0.0..10.0]
pub fn lr_schedule_cosine_warmup(
    step: usize,
    warmup_steps: usize,
    max_steps: usize,
    base_lr: f32,
    min_lr: f32,
) -> f32 {
    if base_lr <= 0.0 || min_lr < 0.0 || max_steps == 0 || !base_lr.is_finite() || !min_lr.is_finite() {
        return 0.0;
    }
    if step < warmup_steps {
        let alpha = (step as f32) / (warmup_steps.max(1) as f32);
        return min_lr + alpha * (base_lr - min_lr);
    }
    if step >= max_steps {
        return min_lr;
    }
    let decay_steps = max_steps - warmup_steps;
    let current_decay = step - warmup_steps;
    let pi = std::f32::consts::PI;
    let progress = (current_decay as f32) / (decay_steps as f32);
    let cosine_decay = 0.5 * (1.0 + (pi * progress).cos());
    min_lr + cosine_decay * (base_lr - min_lr)
}

/// @outputs: 2
/// @domain: max_norm [1e-6..10000.0]
pub fn gradient_clip_l2_norm(grads: &mut [f32], max_norm: f32) -> f32 {
    if grads.is_empty() || max_norm <= 0.0 || !max_norm.is_finite() {
        return 0.0;
    }
    let mut sum_sq = 0.0f32;
    for &g in grads.iter() {
        if g.is_finite() {
            sum_sq += g * g;
        }
    }
    let norm = sum_sq.sqrt();
    if norm > max_norm && norm > 1e-12 {
        let scale = max_norm / norm;
        for g in grads.iter_mut() {
            if g.is_finite() {
                *g *= scale;
            }
        }
    }
    norm
}

/// @outputs: 1
pub fn log_sum_exp_f32(logits: &[f32]) -> Option<f32> {
    if logits.is_empty() {
        return None;
    }
    let mut max_val = f32::NEG_INFINITY;
    for &val in logits {
        if !val.is_finite() {
            return None;
        }
        if val > max_val {
            max_val = val;
        }
    }
    let mut sum_exp = 0.0f32;
    for &val in logits {
        sum_exp += (val - max_val).exp();
    }
    if sum_exp <= 0.0 || !sum_exp.is_finite() {
        return None;
    }
    Some(max_val + sum_exp.ln())
}
