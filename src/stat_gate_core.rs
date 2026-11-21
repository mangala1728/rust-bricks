// SPDX-License-Identifier: MIT
// ## Map
// - Role: core/stat_gate_core — Zero-dependency pure Rust statistical gate engines (STAT-GATE-SPEC v1.0 Q8)
// - Inputs: Bounded counts, rates, p-values, sample slices, and SPC/power parameters
// - Outputs: Deterministic Option<T> intervals, Bayesian posteriors, Holm flags, robust z, power n, and p-chart limits
// - Invariants: Zero external crates, zero recursion (R09-7), bounded loops (R09-8), nesting <= 2 (R09-3), JPL <= 60 lines/fn, full @domain runtime guards (R09-4)

const MAX_BETA_CF_ITER: usize = 2000;
const MAX_SAMPLE_LEN: usize = 100_000;
const Z_95_TWO_SIDED: f64 = 1.959_963_984_540_054;
const MAD_TO_SIGMA: f64 = 1.4826;

/// Computes 95% Wilson score binomial confidence interval (never collapses to 0 width at k=0 or k=n).
/// @outputs: 2
/// @domain: k [0..1000000000]
/// @domain: n [1..1000000000]
pub fn stat_wilson_interval_95(k: u64, n: u64) -> Option<(f64, f64)> {
    if n < 1 || n > 1_000_000_000 || k > n {
        return None;
    }
    let nf = n as f64;
    let p = (k as f64) / nf;
    let z2 = Z_95_TWO_SIDED * Z_95_TWO_SIDED;
    let denom = 1.0 + z2 / nf;
    let center = (p + z2 / (2.0 * nf)) / denom;
    let half = (Z_95_TWO_SIDED * ((p * (1.0 - p) + z2 / (4.0 * nf)) / nf).sqrt()) / denom;
    let lo = (center - half).clamp(0.0, 1.0);
    let hi = (center + half).clamp(0.0, 1.0);
    Some((lo, hi))
}

/// Computes zero-failure 95% one-sided upper bound adjusted by gauge recall lower bound c: (3 / n) / c.
/// @outputs: 2
/// @domain: n [1..1000000000]
/// @domain: recall_lower [0.0001..1.0]
pub fn stat_rule_of_three_upper_bound(n: u64, recall_lower: f64) -> Option<f64> {
    if n < 1 || n > 1_000_000_000 || !recall_lower.is_finite() || recall_lower < 0.0001 || recall_lower > 1.0 {
        return None;
    }
    let raw = (3.0 / (n as f64)) / recall_lower;
    Some(raw.clamp(0.0, 1.0))
}

/// Computes Bayesian positive predictive value (C07 base-rate conversion).
/// @outputs: 2
/// @domain: base_rate [0.0..1.0]
/// @domain: recall [0.0..1.0]
/// @domain: false_positive_rate [0.0..1.0]
pub fn stat_bayesian_ppv(base_rate: f64, recall: f64, false_positive_rate: f64) -> Option<f64> {
    if !base_rate.is_finite()
        || !recall.is_finite()
        || !false_positive_rate.is_finite()
        || base_rate < 0.0
        || base_rate > 1.0
        || recall < 0.0
        || recall > 1.0
        || false_positive_rate < 0.0
        || false_positive_rate > 1.0
    {
        return None;
    }
    let tp = base_rate * recall;
    let denom = tp + (1.0 - base_rate) * false_positive_rate;
    if denom <= 0.0 {
        return None;
    }
    Some((tp / denom).clamp(0.0, 1.0))
}

/// Computes ln(Gamma(z)) via 9-term Lanczos approximation (g = 7, accuracy < 1e-13 for z >= 0.0001).
/// @outputs: 2
/// @domain: z [0.0001..2000000.0]
pub fn stat_ln_gamma(z: f64) -> Option<f64> {
    if !z.is_finite() || z < 0.0001 || z > 2_000_000.0 {
        return None;
    }
    const COEFFS: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    let mut x = COEFFS[0];
    let zm1 = z - 1.0;
    let mut i = 1usize;
    while i < COEFFS.len() {
        x += COEFFS[i] / (zm1 + (i as f64));
        i += 1;
    }
    let t = zm1 + 7.5;
    let half_ln_2pi = 0.5 * (2.0 * std::f64::consts::PI).ln();
    Some(half_ln_2pi + (zm1 + 0.5) * t.ln() - t + x.ln())
}

/// Evaluates Lentz continued fraction for regularized incomplete beta I_x(a, b).
/// @outputs: 2
/// @domain: x [0.0..1.0]
/// @domain: a [0.0001..1000000.0]
/// @domain: b [0.0001..1000000.0]
fn beta_continued_fraction(x: f64, a: f64, b: f64) -> Option<f64> {
    if !x.is_finite()
        || !a.is_finite()
        || !b.is_finite()
        || x < 0.0
        || x > 1.0
        || a < 0.0001
        || a > 1_000_000.0
        || b < 0.0001
        || b > 1_000_000.0
    {
        return None;
    }
    const TINY: f64 = 1.0e-30;
    let mut c = 1.0f64;
    let mut d = 1.0 - (a + b) * x / (a + 1.0);
    if d.abs() < TINY {
        d = TINY;
    }
    d = 1.0 / d;
    let mut h = d;
    let mut m = 1usize;
    while m <= MAX_BETA_CF_ITER {
        let mf = m as f64;
        let m2 = 2.0 * mf;
        let num_even = mf * (b - mf) * x / ((a + m2 - 1.0) * (a + m2));
        d = 1.0 + num_even * d;
        c = 1.0 + num_even / c;
        d = if d.abs() < TINY { 1.0 / TINY } else { 1.0 / d };
        c = if c.abs() < TINY { TINY } else { c };
        h *= d * c;
        let num_odd = -(a + mf) * (a + b + mf) * x / ((a + m2) * (a + m2 + 1.0));
        d = 1.0 + num_odd * d;
        c = 1.0 + num_odd / c;
        d = if d.abs() < TINY { 1.0 / TINY } else { 1.0 / d };
        c = if c.abs() < TINY { TINY } else { c };
        let delta = d * c;
        h *= delta;
        if (delta - 1.0).abs() < 1.0e-13 {
            return Some(h);
        }
        m += 1;
    }
    None
}

/// Computes regularized incomplete beta CDF I_x(alpha, beta) = P(X <= x) for X ~ Beta(alpha, beta).
/// @outputs: 5
/// @domain: x [0.0..1.0]
/// @domain: alpha [0.0001..1000000.0]
/// @domain: beta [0.0001..1000000.0]
pub fn stat_beta_cdf_regularized(x: f64, alpha: f64, beta: f64) -> Option<f64> {
    if !x.is_finite() || !alpha.is_finite() || !beta.is_finite()
        || x < 0.0 || x > 1.0
        || alpha < 0.0001 || alpha > 1_000_000.0
        || beta < 0.0001 || beta > 1_000_000.0
    {
        return None;
    }
    if x == 0.0 {
        return Some(0.0);
    }
    if x == 1.0 {
        return Some(1.0);
    }
    let ln_b = stat_ln_gamma(alpha)? + stat_ln_gamma(beta)? - stat_ln_gamma(alpha + beta)?;
    let front = (alpha * x.ln() + beta * (1.0 - x).ln() - ln_b).exp();
    let threshold = (alpha + 1.0) / (alpha + beta + 2.0);
    if x < threshold {
        let cf = beta_continued_fraction(x, alpha, beta)?;
        return Some((front * cf / alpha).clamp(0.0, 1.0));
    }
    let cf = beta_continued_fraction(1.0 - x, beta, alpha)?;
    Some((1.0 - front * cf / beta).clamp(0.0, 1.0))
}

/// Evaluates STAT-GATE Q4/Q5 Bayesian trial-to-trusted upgrade across 3 priors: Beta(1,1), Jeffreys(0.5,0.5), Beta(2,2).
/// Returns (conservative_pass, prob_uniform, prob_jeffreys, prob_beta22) where each prob = P(p >= p0).
/// @outputs: 2
/// @domain: successes [0..100000]
/// @domain: failures [0..100000]
/// @domain: p0 [0.0001..0.9999]
/// @domain: confidence [0.5..0.9999]
pub fn stat_bayesian_trial_upgrade(
    successes: u32,
    failures: u32,
    p0: f64,
    confidence: f64,
) -> Option<(bool, f64, f64, f64)> {
    if successes > 100_000
        || failures > 100_000
        || !p0.is_finite()
        || !confidence.is_finite()
        || p0 < 0.0001
        || p0 > 0.9999
        || confidence < 0.5
        || confidence > 0.9999
    {
        return None;
    }
    let s = successes as f64;
    let f = failures as f64;
    let p_uniform = 1.0 - stat_beta_cdf_regularized(p0, 1.0 + s, 1.0 + f)?;
    let p_jeffreys = 1.0 - stat_beta_cdf_regularized(p0, 0.5 + s, 0.5 + f)?;
    let p_beta22 = 1.0 - stat_beta_cdf_regularized(p0, 2.0 + s, 2.0 + f)?;
    let pass = p_uniform >= confidence && p_jeffreys >= confidence && p_beta22 >= confidence;
    Some((pass, p_uniform, p_jeffreys, p_beta22))
}

/// Applies step-down Holm-Bonferroni multiple-comparison correction to p-values.
/// @outputs: 2
/// @domain: alpha [0.0001..0.5]
pub fn stat_holm_bonferroni_reject(p_values: &[f64], alpha: f64) -> Option<Vec<bool>> {
    if p_values.is_empty() || p_values.len() > MAX_SAMPLE_LEN || !alpha.is_finite() || alpha < 0.0001 || alpha > 0.5 {
        return None;
    }
    let mut indexed: Vec<(usize, f64)> = Vec::with_capacity(p_values.len());
    for (idx, &p) in p_values.iter().enumerate() {
        if !p.is_finite() || p < 0.0 || p > 1.0 {
            return None;
        }
        indexed.push((idx, p));
    }
    indexed.sort_by(|a, b| a.1.total_cmp(&b.1));
    let m = indexed.len();
    let mut rejected = vec![false; m];
    for (rank, &(orig_idx, p)) in indexed.iter().enumerate() {
        let thresh = alpha / ((m - rank) as f64);
        if p <= thresh {
            rejected[orig_idx] = true;
        } else {
            break;
        }
    }
    Some(rejected)
}

/// Computes median of an already-sorted non-empty slice of finite f64 values.
/// @outputs: 2
fn sorted_median(sorted: &[f64]) -> Option<f64> {
    let len = sorted.len();
    if len == 0 {
        return None;
    }
    let mid = len / 2;
    if len % 2 == 1 {
        return Some(sorted[mid]);
    }
    Some(0.5 * (sorted[mid - 1] + sorted[mid]))
}

/// Computes sample median and median absolute deviation (MAD) for STAT-GATE C14/C15 OUT-v1.
/// @outputs: 2
pub fn stat_median_and_mad(values: &[f64]) -> Option<(f64, f64)> {
    if values.is_empty() || values.len() > MAX_SAMPLE_LEN {
        return None;
    }
    let mut sorted = Vec::with_capacity(values.len());
    for &v in values {
        if !v.is_finite() {
            return None;
        }
        sorted.push(v);
    }
    sorted.sort_by(|a, b| a.total_cmp(b));
    let med = sorted_median(&sorted)?;
    let mut devs: Vec<f64> = sorted.iter().map(|&v| (v - med).abs()).collect();
    devs.sort_by(|a, b| a.total_cmp(b));
    let mad = sorted_median(&devs)?;
    Some((med, mad))
}

/// Computes OUT-v1 robust z-score: (x - median) / (1.4826 * mad).
/// @outputs: 2
/// @domain: x [-1000000000.0..1000000000.0]
/// @domain: median [-1000000000.0..1000000000.0]
/// @domain: mad [0.000000001..1000000000.0]
pub fn stat_robust_z_score(x: f64, median: f64, mad: f64) -> Option<f64> {
    if !x.is_finite()
        || !median.is_finite()
        || !mad.is_finite()
        || x.abs() > 1_000_000_000.0
        || median.abs() > 1_000_000_000.0
        || mad < 1.0e-9
        || mad > 1_000_000_000.0
    {
        return None;
    }
    Some((x - median) / (MAD_TO_SIGMA * mad))
}

/// Computes required paired task count: ceil(((z_alpha + z_beta) * sd / delta)^2).
/// @outputs: 2
/// @domain: sd [0.0001..100000.0]
/// @domain: delta [0.0001..100000.0]
/// @domain: z_alpha [0.5..6.0]
/// @domain: z_beta [0.1..6.0]
pub fn stat_required_paired_sample_size(sd: f64, delta: f64, z_alpha: f64, z_beta: f64) -> Option<u32> {
    if !sd.is_finite()
        || !delta.is_finite()
        || !z_alpha.is_finite()
        || !z_beta.is_finite()
        || sd < 0.0001
        || sd > 100_000.0
        || delta < 0.0001
        || delta > 100_000.0
        || z_alpha < 0.5
        || z_alpha > 6.0
        || z_beta < 0.1
        || z_beta > 6.0
    {
        return None;
    }
    let ratio = (z_alpha + z_beta) * sd / delta;
    let raw = (ratio * ratio).ceil();
    if !raw.is_finite() || raw < 1.0 || raw > 100_000_000.0 {
        return None;
    }
    Some(raw as u32)
}

/// Computes Shewhart p-chart control limits (LCL clamped at 0.0, UCL clamped at 1.0) for k_sigma (3.0 or 4.0).
/// @outputs: 2
/// @domain: p_bar [0.0..1.0]
/// @domain: n_i [1..1000000000]
/// @domain: k_sigma [1.0..6.0]
pub fn stat_p_chart_limits(p_bar: f64, n_i: u64, k_sigma: f64) -> Option<(f64, f64)> {
    if !p_bar.is_finite()
        || !k_sigma.is_finite()
        || p_bar < 0.0
        || p_bar > 1.0
        || n_i < 1
        || n_i > 1_000_000_000
        || k_sigma < 1.0
        || k_sigma > 6.0
    {
        return None;
    }
    let se = (p_bar * (1.0 - p_bar) / (n_i as f64)).sqrt();
    let lcl = (p_bar - k_sigma * se).max(0.0);
    let ucl = (p_bar + k_sigma * se).min(1.0);
    Some((lcl, ucl))
}

/// Deterministic Xorshift64 PRNG step for reproducible permutation/bootstrap (L-4).
/// @outputs: 2
/// @domain: state [1..18446744073709551615]
pub fn stat_xorshift64_next(state: u64) -> u64 {
    if state == 0 {
        return 0;
    }
    let mut x = state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}

/// Computes equal-weighted mean across distinct strata, with optional sign-flip permutation.
/// @outputs: 1
fn stratified_sign_flip_mean(diffs: &[f64], strata: &[u16], mut rng: Option<&mut u64>) -> f64 {
    let mut keys: [u16; 64] = [0; 64];
    let mut sums: [f64; 64] = [0.0; 64];
    let mut counts: [usize; 64] = [0; 64];
    let mut n_strata = 0usize;
    for (i, &d) in diffs.iter().enumerate() {
        let sign = match rng.as_mut() {
            Some(r) => {
                **r = stat_xorshift64_next(**r);
                if (**r & 1) == 0 { 1.0 } else { -1.0 }
            }
            None => 1.0,
        };
        let s = strata[i];
        let pos = keys[..n_strata].iter().position(|&k| k == s).unwrap_or(n_strata);
        if pos == n_strata && n_strata < 64 {
            keys[n_strata] = s;
            n_strata += 1;
        }
        let slot = pos.min(63);
        sums[slot] += sign * d;
        counts[slot] += 1;
    }
    let mut total = 0.0f64;
    for idx in 0..n_strata {
        total += sums[idx] / (counts[idx].max(1) as f64);
    }
    total / (n_strata.max(1) as f64)
}

/// Two-sided stratified paired sign-flip permutation p-value (STAT-GATE Q14).
/// @outputs: 2
/// @domain: n_perm [100..200000]
/// @domain: seed [1..18446744073709551615]
pub fn stat_stratified_permutation_p(
    diffs: &[f64],
    strata: &[u16],
    n_perm: u32,
    seed: u64,
) -> Option<f64> {
    if diffs.is_empty()
        || diffs.len() != strata.len()
        || diffs.len() > MAX_SAMPLE_LEN
        || n_perm < 100
        || n_perm > 200_000
        || seed == 0
    {
        return None;
    }
    if diffs.iter().any(|v| !v.is_finite()) {
        return None;
    }
    let obs = stratified_sign_flip_mean(diffs, strata, None).abs();
    let mut rng = seed;
    let mut ge_count = 0u32;
    for _ in 0..n_perm {
        if stratified_sign_flip_mean(diffs, strata, Some(&mut rng)).abs() >= obs * (1.0 - 1.0e-12) {
            ge_count += 1;
        }
    }
    Some(((ge_count as f64) + 1.0) / ((n_perm as f64) + 1.0))
}

/// Helper computing one bootstrap resample mean.
/// @outputs: 1
fn bootstrap_sample_mean(diffs: &[f64], rng: &mut u64) -> f64 {
    let n = diffs.len();
    let mut sum = 0.0f64;
    for _ in 0..n {
        *rng = stat_xorshift64_next(*rng);
        let idx = (*rng as usize) % n;
        sum += diffs[idx];
    }
    sum / (n as f64)
}

/// Paired percentile bootstrap confidence interval (mean, lo, hi) at level 1 - alpha.
/// @outputs: 2
/// @domain: alpha [0.001..0.5]
/// @domain: n_boot [100..200000]
/// @domain: seed [1..18446744073709551615]
pub fn stat_paired_bootstrap_ci(
    diffs: &[f64],
    alpha: f64,
    n_boot: u32,
    seed: u64,
) -> Option<(f64, f64, f64)> {
    if diffs.is_empty()
        || diffs.len() > MAX_SAMPLE_LEN
        || !alpha.is_finite()
        || alpha < 0.001
        || alpha > 0.5
        || n_boot < 100
        || n_boot > 200_000
        || seed == 0
    {
        return None;
    }
    if diffs.iter().any(|v| !v.is_finite()) {
        return None;
    }
    let obs_mean = diffs.iter().sum::<f64>() / (diffs.len() as f64);
    let mut rng = seed;
    let mut means = Vec::with_capacity(n_boot as usize);
    for _ in 0..n_boot {
        means.push(bootstrap_sample_mean(diffs, &mut rng));
    }
    means.sort_by(|a, b| a.total_cmp(b));
    let lo_idx = (((alpha / 2.0) * (means.len() as f64)).floor() as usize).min(means.len() - 1);
    let hi_idx = (((1.0 - alpha / 2.0) * (means.len() as f64)).ceil() as usize).min(means.len() - 1);
    Some((obs_mean, means[lo_idx], means[hi_idx]))
}

/// Helper appending matched forbidden terms from a static word list.
/// @outputs: 1
fn collect_forbidden_hits(haystack: &str, terms: &[&'static str], out: &mut Vec<&'static str>) {
    for &term in terms {
        if haystack.contains(term) {
            out.push(term);
        }
    }
}

/// Checks claim text against STAT-GATE §6 wording rules (level: 0=描述, 1=關聯, 2=因果).
/// @outputs: 2
/// @domain: level [0..2]
pub fn stat_check_wording_level(level: u8, text: &str) -> Vec<&'static str> {
    if level > 2 {
        return vec!["INVALID_LEVEL"];
    }
    const ALWAYS_FORBIDDEN: &[&str] = &["保證", "杜絕", "一定", "所有情況", "擋得住"];
    const CAUSAL_FORBIDDEN: &[&str] = &["導致", "因為", "造成", "使得", "證明", "有效"];
    const COMPARE_FORBIDDEN: &[&str] = &["改善", "變好", "提升", "更快", "更少", "優於"];

    let mut hits = Vec::new();
    collect_forbidden_hits(text, ALWAYS_FORBIDDEN, &mut hits);
    let cleaned = text.replace("因為樣本不足", "").replace("因為樣本數不足", "");
    if level <= 1 {
        collect_forbidden_hits(&cleaned, CAUSAL_FORBIDDEN, &mut hits);
    }
    if level == 0 {
        collect_forbidden_hits(&cleaned, COMPARE_FORBIDDEN, &mut hits);
    }
    hits
}

/// Checks whether a whitespace/delimiter-separated token represents a standalone number (excluding `v2`/`C01`, keeping `1.5`/`95%` as 1).
/// @outputs: 2
fn is_standalone_number_token(tok: &str) -> bool {
    let core = tok.trim_matches(|c: char| matches!(c, ',' | ';' | '(' | ')' | '[' | ']' | '%' | '。' | '，'));
    if core.is_empty() || core.bytes().any(|b| b.is_ascii_alphabetic()) {
        return false;
    }
    core.parse::<f64>().is_ok()
}

/// Counts standalone numeric tokens in a paragraph (without splitting decimals or counting `v2`/`C01`).
/// @outputs: 1
fn count_numeric_tokens(text: &str) -> usize {
    let mut count = 0usize;
    for tok in text.split(|c: char| c.is_whitespace() || matches!(c, '→' | '/' | '：' | ':')).take(MAX_SAMPLE_LEN) {
        if is_standalone_number_token(tok) {
            count += 1;
        }
    }
    count
}

/// Detects STAT-GATE §3 T4 trigger (comparative keyword + numeric comparison cue).
/// @outputs: 2
pub fn stat_detect_t4_trigger(paragraph: &str) -> bool {
    const T4_WORDS: &[&str] = &[
        "變好", "改善", "提升", "增加", "降低", "減少", "上升", "下降",
        "降到", "降至", "升到", "升至", "高於", "低於",
        "提高", "好轉", "優化", "更快", "更少", "更穩", "優於",
        "improve", "reduce", "increase", "higher", "lower", "faster", "better", "fewer",
    ];
    let lower = paragraph.to_ascii_lowercase();
    let has_word = T4_WORDS.iter().any(|&w| lower.contains(w));
    if !has_word {
        return false;
    }
    let has_pair_cue = paragraph.contains("→")
        || paragraph.contains("->")
        || (paragraph.contains("從") && paragraph.contains("到"))
        || (paragraph.contains("前") && paragraph.contains("後"))
        || (paragraph.contains("新") && paragraph.contains("舊"));
    has_pair_cue || count_numeric_tokens(paragraph) >= 2
}
