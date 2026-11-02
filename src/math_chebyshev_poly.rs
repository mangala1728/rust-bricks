// ## Map
// Role:   Compute first‑kind Chebyshev polynomials and evaluate Chebyshev series.
// Inputs: n: u32, x: f64, coeffs: &[f64]
// Outputs: Option<f64>
// Invariants: x ∈ [-1.0, 1.0], coeffs non‑empty, all coefficients finite.

/// Compute the first‑kind Chebyshev polynomial Tₙ(x).
///
/// @outputs: 1
/// @domain: n [0..=u32::MAX], x [-1.0..=1.0]
///
/// Returns `None` if `x` is not finite or lies outside `[-1, 1]`.
pub fn math_chebyshev_t(n: u32, x: f64) -> Option<f64> {
    // Guard: x must be finite and inside the domain.
    if !x.is_finite() || x < -1.0 || x > 1.0 {
        return None;
    }

    // Base cases.
    if n == 0 {
        return Some(1.0);
    }
    if n == 1 {
        return Some(x);
    }

    // Recurrence: Tₖ₊₁ = 2·x·Tₖ – Tₖ₋₁
    let mut t_prev = 1.0; // T₀
    let mut t_curr = x;   // T₁
    // Loop depth is 1 → satisfies nesting ≤ 2.
    for _ in 2..=n {
        let t_next = 2.0 * x * t_curr - t_prev;
        t_prev = t_curr;
        t_curr = t_next;
    }
    Some(t_curr)
}

/// Evaluate a Chebyshev series Σ cₖ·Tₖ(x) using the Clenshaw recurrence.
///
/// @outputs: 1
/// @domain: coeffs.len() [1..=usize::MAX], x [-1.0..=1.0]
///
/// Returns `None` if the slice is empty, `x` is out of domain,
/// or any coefficient is not finite.
pub fn math_chebyshev_clenshaw(coeffs: &[f64], x: f64) -> Option<f64> {
    // Guard: non‑empty, finite x, domain, and finite coefficients.
    if coeffs.is_empty()
        || !x.is_finite()
        || x < -1.0
        || x > 1.0
        || coeffs.iter().any(|&c| !c.is_finite())
    {
        return None;
    }

    // Single‑coefficient shortcut.
    if coeffs.len() == 1 {
        return Some(coeffs[0]);
    }

    // Clenshaw recurrence.
    // b_{k+1} and b_{k+2} start at 0.
    let mut b_k1 = 0.0; // b_{k+1}
    let mut b_k2 = 0.0; // b_{k+2}
    // Iterate backwards over c₁ … c_{n}.
    for &c in coeffs[1..].iter().rev() {
        let b_k = 2.0 * x * b_k1 - b_k2 + c;
        b_k2 = b_k1;
        b_k1 = b_k;
    }
    // Final combination.
    let sum = coeffs[0] + x * b_k1 - b_k2;
    Some(sum)
}
