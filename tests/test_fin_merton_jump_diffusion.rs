// ## Map
// Role: integration test for Merton Jump Diffusion Poisson weight & jump parameters
// Inputs: none (uses public API)
// Outputs: test assertions
// Invariants: zero external deps, deterministic golden values

use rust_bricks::fin_merton_jump_diffusion::*;

const EPS: f64 = 1e-12;

/// Helper for approximate equality
fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < EPS
}

#[test]
fn poisson_weight_basic_cases() {
    // λ = 0 → only n = 0 has probability 1
    let w0 = fin_merton_poisson_weight(0.0, 1.0, 0).expect("should be Some");
    assert!(approx_eq(w0, 1.0));

    let w1 = fin_merton_poisson_weight(0.0, 1.0, 1).expect("should be Some");
    assert!(approx_eq(w1, 0.0));
}

#[test]
fn poisson_weight_invalid_inputs() {
    // negative intensity
    assert!(fin_merton_poisson_weight(-1.0, 1.0, 0).is_none());
    // n exceeds bound
    assert!(fin_merton_poisson_weight(1.0, 1.0, 31).is_none());
    // non‑positive time
    assert!(fin_merton_poisson_weight(1.0, 0.0, 0).is_none());
}

#[test]
fn jump_parameters_basic_cases() {
    // μ_j = 0, δ = 0 → κ = 0, λ' = λ
    let (kappa, lambda_prime) = fin_merton_jump_parameters(2.0, 0.0, 0.0)
        .expect("should be Some");
    assert!(approx_eq(kappa, 0.0));
    assert!(approx_eq(lambda_prime, 2.0));
}

#[test]
fn jump_parameters_invalid_inputs() {
    // negative λ
    assert!(fin_merton_jump_parameters(-0.5, 0.0, 0.0).is_none());
    // negative δ
    assert!(fin_merton_jump_parameters(1.0, 0.0, -0.1).is_none());
    // non‑finite μ_j
    assert!(fin_merton_jump_parameters(1.0, f64::INFINITY, 0.0).is_none());
}
