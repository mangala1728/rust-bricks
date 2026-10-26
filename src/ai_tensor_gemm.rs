// ## Map
// Role: ai/tensor_gemm — 神經網路高效張量運算 (分塊 GEMM、Conv2D、MaxPool、BatchNorm、Dropout、Embedding、餘弦相似度) 微積木。
// Inputs: 權重矩陣、輸入張量、均值方差、Token ID、遺失率 (p)、平滑閾值。
// Outputs: 矩陣乘積、卷積特徵圖、最大池化、標準化張量、嵌入向量、餘弦相似度。
// Invariants: 零外部依賴、純 Rust 實現、除零與 NaN 剛性防護、單一函式 <= 60 行。

/// @outputs: 1
/// @domain: m [1..4096]
/// @domain: n [1..4096]
/// @domain: k [1..4096]
pub fn gemm_tiled_f32_step(
    a: &[f32],
    b: &[f32],
    c: &mut [f32],
    m: usize,
    n: usize,
    k: usize,
    tile_size: usize,
) -> bool {
    if m == 0 || n == 0 || k == 0 || tile_size == 0
        || a.len() < m * k || b.len() < k * n || c.len() < m * n {
        return false;
    }
    let ts = tile_size.max(1);
    let mut i0 = 0usize;
    while i0 < m {
        let i_end = (i0 + ts).min(m);
        let mut k0 = 0usize;
        while k0 < k {
            let k_end = (k0 + ts).min(k);
            let mut j0 = 0usize;
            while j0 < n {
                let j_end = (j0 + ts).min(n);
                for i in i0..i_end {
                    for kk in k0..k_end {
                        let a_ik = a[i * k + kk];
                        for j in j0..j_end {
                            c[i * n + j] += a_ik * b[kk * n + j];
                        }
                    }
                }
                j0 += ts;
            }
            k0 += ts;
        }
        i0 += ts;
    }
    true
}

/// @outputs: 1
pub fn conv2d_valid_forward_step(
    input: &[f32],
    kernel: &[f32],
    output: &mut [f32],
    in_h: usize,
    in_w: usize,
    k_h: usize,
    k_w: usize,
) -> bool {
    if in_h < k_h || in_w < k_w || k_h == 0 || k_w == 0
        || input.len() < in_h * in_w || kernel.len() < k_h * k_w {
        return false;
    }
    let out_h = in_h - k_h + 1;
    let out_w = in_w - k_w + 1;
    if output.len() < out_h * out_w {
        return false;
    }
    for oh in 0..out_h {
        for ow in 0..out_w {
            let mut sum = 0.0f32;
            for kh in 0..k_h {
                for kw in 0..k_w {
                    let in_val = input[(oh + kh) * in_w + (ow + kw)];
                    let k_val = kernel[kh * k_w + kw];
                    sum += in_val * k_val;
                }
            }
            output[oh * out_w + ow] = sum;
        }
    }
    true
}

/// @outputs: 1
pub fn maxpool2d_2x2_stride2_step(
    input: &[f32],
    output: &mut [f32],
    in_h: usize,
    in_w: usize,
) -> bool {
    if in_h < 2 || in_w < 2 || (in_h % 2) != 0 || (in_w % 2) != 0
        || input.len() < in_h * in_w {
        return false;
    }
    let out_h = in_h / 2;
    let out_w = in_w / 2;
    if output.len() < out_h * out_w {
        return false;
    }
    for oh in 0..out_h {
        for ow in 0..out_w {
            let ih = oh * 2;
            let iw = ow * 2;
            let p0 = input[ih * in_w + iw];
            let p1 = input[ih * in_w + (iw + 1)];
            let p2 = input[(ih + 1) * in_w + iw];
            let p3 = input[(ih + 1) * in_w + (iw + 1)];
            let max_val = p0.max(p1).max(p2).max(p3);
            output[oh * out_w + ow] = max_val;
        }
    }
    true
}

/// @outputs: 1
/// @domain: eps [1.0e-12..1.0e-3]
pub fn batch_norm_forward_inference(
    x: &[f32],
    mean: &[f32],
    var: &[f32],
    gamma: &[f32],
    beta: &[f32],
    eps: f32,
    out: &mut [f32],
) -> bool {
    let channels = mean.len();
    if channels == 0 || var.len() != channels || gamma.len() != channels
        || beta.len() != channels || x.len() != out.len() || (x.len() % channels) != 0
        || eps <= 0.0 || !eps.is_finite() {
        return false;
    }
    let spatial = x.len() / channels;
    for c in 0..channels {
        let std_inv = 1.0 / (var[c] + eps).sqrt();
        let g = gamma[c];
        let b = beta[c];
        let m = mean[c];
        for s in 0..spatial {
            let idx = c * spatial + s;
            out[idx] = g * (x[idx] - m) * std_inv + b;
        }
    }
    true
}

/// @outputs: 1
/// @domain: p_drop [0.0..0.999]
pub fn dropout_inverted_mask_apply(
    x: &mut [f32],
    mask_rand: &[u32],
    p_drop: f32,
) -> bool {
    if p_drop < 0.0 || p_drop >= 1.0 || !p_drop.is_finite() || mask_rand.len() < x.len() {
        return false;
    }
    let keep_prob = 1.0 - p_drop;
    let inv_scale = 1.0 / keep_prob;
    let threshold = (keep_prob * (u32::MAX as f32)) as u32;

    for i in 0..x.len() {
        if mask_rand[i] < threshold {
            x[i] *= inv_scale;
        } else {
            x[i] = 0.0;
        }
    }
    true
}

/// @outputs: 1
pub fn embedding_lookup_1d(
    weights: &[f32],
    token_id: usize,
    embedding_dim: usize,
    out_vec: &mut [f32],
) -> bool {
    if embedding_dim == 0 || out_vec.len() < embedding_dim {
        return false;
    }
    let start_idx = token_id * embedding_dim;
    let end_idx = start_idx + embedding_dim;
    if end_idx > weights.len() {
        return false;
    }
    for (i, &w) in weights[start_idx..end_idx].iter().enumerate() {
        out_vec[i] = w;
    }
    true
}

/// @outputs: 1
/// @domain: negative_slope [0.0..1.0]
pub fn leaky_relu_f32(x: f32, negative_slope: f32) -> f32 {
    if !x.is_finite() || negative_slope < 0.0 || !negative_slope.is_finite() {
        return 0.0;
    }
    if x >= 0.0 {
        x
    } else {
        negative_slope * x
    }
}

/// @outputs: 1
/// @domain: beta [1.0e-6..100.0]
pub fn smooth_l1_loss_step(pred: f32, target: f32, beta: f32) -> f32 {
    if beta <= 0.0 || !pred.is_finite() || !target.is_finite() || !beta.is_finite() {
        return 0.0;
    }
    let diff = (pred - target).abs();
    if diff < beta {
        0.5 * (diff * diff) / beta
    } else {
        diff - 0.5 * beta
    }
}

/// @outputs: 1
pub fn cosine_similarity_f32(vec_a: &[f32], vec_b: &[f32]) -> Option<f32> {
    let n = vec_a.len();
    if n == 0 || vec_b.len() != n {
        return None;
    }
    let mut dot = 0.0f32;
    let mut norm_a_sq = 0.0f32;
    let mut norm_b_sq = 0.0f32;
    for i in 0..n {
        let a = vec_a[i];
        let b = vec_b[i];
        if !a.is_finite() || !b.is_finite() { return None; }
        dot += a * b;
        norm_a_sq += a * a;
        norm_b_sq += b * b;
    }
    let denom = (norm_a_sq * norm_b_sq).sqrt();
    if denom <= 1e-12 {
        return None;
    }
    Some((dot / denom).clamp(-1.0, 1.0))
}

/// @outputs: 1
pub fn one_hot_encode_u8(class_id: usize, num_classes: usize, out_buf: &mut [u8]) -> bool {
    if class_id >= num_classes || out_buf.len() < num_classes {
        return false;
    }
    for b in out_buf[..num_classes].iter_mut() {
        *b = 0;
    }
    out_buf[class_id] = 1;
    true
}
