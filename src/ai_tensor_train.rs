// ## Map
// Role: ai/ai_tensor_train — 零依賴神經網絡與 Transformer 核心訓練及推論算子微積木 (AI Tensor Training Micro-Bricks)。
// Inputs: Logits 切片、張量激活值、優化器狀態參數 (梯度、動量、學習率)。
// Outputs: 數值穩定 Softmax 機率、LayerNorm/RMSNorm 標準化特徵、RoPE 旋轉編碼、AdamW 參數步進、交叉熵損失與梯度。
// Invariants: 零外部依賴、純 Rust 實現、防浮點溢位 (Log-Sum-Exp 技巧)、零全域可變狀態、單一函式 <= 60 行。

/// @outputs: 2
/// @domain: logits.len() [1..65536]
pub fn softmax_stable(logits: &[f32], out_probs: &mut [f32]) -> bool {
    if logits.is_empty() || logits.len() != out_probs.len() || logits.len() > 65_536 {
        return false;
    }
    let mut max_val = logits[0];
    for &x in logits {
        if !x.is_finite() {
            return false;
        }
        if x > max_val {
            max_val = x;
        }
    }
    let mut sum = 0.0f32;
    for (i, &x) in logits.iter().enumerate() {
        let p = (x - max_val).exp();
        out_probs[i] = p;
        sum += p;
    }
    if sum <= 0.0 || !sum.is_finite() {
        return false;
    }
    let inv_sum = 1.0 / sum;
    for p in out_probs.iter_mut() {
        *p *= inv_sum;
    }
    true
}

/// @outputs: 2
/// @domain: eps [1e-9..1.0]
pub fn layer_norm(x: &[f32], gamma: &[f32], beta: &[f32], eps: f32, out: &mut [f32]) -> bool {
    let n = x.len();
    if n == 0 || gamma.len() != n || beta.len() != n || out.len() != n || eps <= 0.0 || !eps.is_finite() || n > 65_536 {
        return false;
    }
    let mut mean = 0.0f32;
    for i in 0..n {
        if !x[i].is_finite() || !gamma[i].is_finite() || !beta[i].is_finite() {
            return false;
        }
        mean += x[i];
    }
    mean /= n as f32;
    let mut var = 0.0f32;
    for &val in x {
        let diff = val - mean;
        var += diff * diff;
    }
    var /= n as f32;
    let inv_std = 1.0 / (var + eps).sqrt();
    for i in 0..n {
        out[i] = (x[i] - mean) * inv_std * gamma[i] + beta[i];
    }
    true
}

/// @outputs: 2
/// @domain: eps [1e-9..1.0]
pub fn rmsnorm_step(x: &[f32], weight: &[f32], eps: f32, out: &mut [f32]) -> bool {
    let n = x.len();
    if n == 0 || weight.len() != n || out.len() != n || eps <= 0.0 || !eps.is_finite() || n > 65_536 {
        return false;
    }
    let mut sum_sq = 0.0f32;
    for i in 0..n {
        if !x[i].is_finite() || !weight[i].is_finite() {
            return false;
        }
        sum_sq += x[i] * x[i];
    }
    let rms = ((sum_sq / (n as f32)) + eps).sqrt();
    if rms <= 0.0 || !rms.is_finite() {
        return false;
    }
    let inv_rms = 1.0 / rms;
    for i in 0..n {
        out[i] = x[i] * inv_rms * weight[i];
    }
    true
}

/// @outputs: 1
pub fn rope_rotary_embedding_step(vec_pair: (f32, f32), angle_rad: f32) -> (f32, f32) {
    if !vec_pair.0.is_finite() || !vec_pair.1.is_finite() || !angle_rad.is_finite() {
        return (0.0, 0.0);
    }
    let cos = angle_rad.cos();
    let sin = angle_rad.sin();
    let x0 = vec_pair.0 * cos - vec_pair.1 * sin;
    let x1 = vec_pair.0 * sin + vec_pair.1 * cos;
    (x0, x1)
}

/// @outputs: 1
/// @domain: pos [0..1000000]
/// @domain: base [1.0..10000000.0]
pub fn rope_frequency_theta(pos: usize, dim_idx: usize, head_dim: usize, base: f32) -> f32 {
    if head_dim == 0 || dim_idx >= head_dim / 2 || base <= 0.0 || !base.is_finite() || pos > 1_000_000 {
        return 0.0;
    }
    let exponent = -2.0 * (dim_idx as f32) / (head_dim as f32);
    let freq = base.powf(exponent);
    (pos as f32) * freq
}

/// @outputs: 2
/// @domain: step [1..10000000]
/// @domain: lr [1e-7..10.0]
pub fn adamw_step(
    param: &mut f32,
    grad: f32,
    m: &mut f32,
    v: &mut f32,
    lr: f32,
    beta1: f32,
    beta2: f32,
    weight_decay: f32,
    eps: f32,
    step: u32,
) -> bool {
    if step == 0 || lr <= 0.0 || !lr.is_finite() || !grad.is_finite() || !param.is_finite()
        || beta1 < 0.0 || beta1 >= 1.0 || beta2 < 0.0 || beta2 >= 1.0
        || eps <= 0.0 || !eps.is_finite() || weight_decay < 0.0 || !weight_decay.is_finite() {
        return false;
    }
    *m = beta1 * (*m) + (1.0 - beta1) * grad;
    *v = beta2 * (*v) + (1.0 - beta2) * grad * grad;
    let bc1 = 1.0 - beta1.powi(step as i32);
    let bc2 = 1.0 - beta2.powi(step as i32);
    if bc1 <= 0.0 || bc2 <= 0.0 {
        return false;
    }
    let m_hat = *m / bc1;
    let v_hat = *v / bc2;
    *param *= 1.0 - lr * weight_decay;
    *param -= lr * m_hat / (v_hat.sqrt() + eps);
    param.is_finite()
}

/// @outputs: 2
/// @domain: logits.len() [1..65536]
pub fn cross_entropy_loss_and_grad(logits: &[f32], target_idx: usize, out_grad: &mut [f32]) -> Option<f32> {
    let n = logits.len();
    if n == 0 || target_idx >= n || out_grad.len() != n || n > 65_536 {
        return None;
    }
    let mut max_val = logits[0];
    for &x in logits {
        if !x.is_finite() {
            return None;
        }
        if x > max_val {
            max_val = x;
        }
    }
    let mut sum_exp = 0.0f32;
    for &x in logits {
        sum_exp += (x - max_val).exp();
    }
    if sum_exp <= 0.0 || !sum_exp.is_finite() {
        return None;
    }
    let loss = -(logits[target_idx] - max_val) + sum_exp.ln();
    let inv_sum = 1.0 / sum_exp;
    for (i, &x) in logits.iter().enumerate() {
        let prob = (x - max_val).exp() * inv_sum;
        let target_indicator = if i == target_idx { 1.0 } else { 0.0 };
        out_grad[i] = prob - target_indicator;
    }
    Some(if loss < 0.0 { 0.0 } else { loss })
}

/// @outputs: 1
pub fn gelu_forward(x: f32) -> f32 {
    if !x.is_finite() {
        return 0.0;
    }
    const SQRT_2_OVER_PI: f32 = 0.7978845608;
    const COEFF: f32 = 0.044715;
    let inner = SQRT_2_OVER_PI * (x + COEFF * x * x * x);
    0.5 * x * (1.0 + inner.tanh())
}
