// ## Map
// Role: Integration test for RK4 scalar & 2D numerical ODE solver
// Inputs: none (uses public API)
// Outputs: test assertions
// Invariants: golden values verified against analytical solutions (tolerance 1e-6)

use rust_bricks::math_rk4_ode::*;

/// Verify the scalar RK4 step against the analytical solution of
/// dy/dt = -y  (exponential decay).  The golden value after one step
/// with dt = 0.1 is 0.9048375 (tolerance 1e‑6).
#[test]
fn test_rk4_step_1d_exponential_decay() {
    let t0 = 0.0_f64;
    let y0 = 1.0_f64;
    let dt = 0.1_f64;

    // derivative: f(t, y) = -y
    let result = rk4_step_1d(t0, y0, dt, |_, y| -y).expect("dt within domain"); // @safety-proof

    let golden = 0.9048375_f64;
    assert!((result - golden).abs() < 1e-6, "RK4 result deviates from golden value");
}

#[test]
fn test_rk4_step_2d_harmonic_oscillator() {
    // dx/dt = v, dv/dt = -x (analytic: x(t) = cos(t))
    let (x1, v1) = rk4_step_2d(0.0, (1.0, 0.0), 0.1, |_, (x, v)| (v, -x)).expect("dt within domain"); // @safety-proof
    assert!((x1 - 0.995004).abs() < 1e-4);
    assert!((v1 - (-0.099833)).abs() < 1e-4);
}
