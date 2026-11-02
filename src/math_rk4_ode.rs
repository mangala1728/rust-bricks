// ## Map: Role=RK4 Solver, Inputs=[t: f64, y: f64 or (f64,f64), dt: f64, derivative_fn], Outputs=[Option<f64>, Option<(f64,f64)>], Invariants=dt ∈ [1.0e-9 .. 1000.0]

/// @outputs: 1
/// @domain: dt [1.0e-9..1000.0]
///
/// Classical fourth‑order Runge‑Kutta single step for a scalar ODE:
///   dy/dt = f(t, y)
///
/// Returns `None` if `dt` is outside the allowed domain.
pub fn rk4_step_1d<F>(t: f64, y: f64, dt: f64, derivative_fn: F) -> Option<f64>
where
    F: Fn(f64, f64) -> f64,
{
    // domain guard
    if dt < 1.0e-9 || dt > 1000.0 {
        return None;
    }

    // k1 … k4
    let k1 = derivative_fn(t, y);
    let k2 = derivative_fn(t + 0.5 * dt, y + 0.5 * dt * k1);
    let k3 = derivative_fn(t + 0.5 * dt, y + 0.5 * dt * k2);
    let k4 = derivative_fn(t + dt, y + dt * k3);

    // RK4 update
    let y_next = y + dt / 6.0 * (k1 + 2.0 * k2 + 2.0 * k3 + k4);
    Some(y_next)
}

/// @outputs: 2
/// @domain: dt [1.0e-9..1000.0]
///
/// RK4 step for a 2‑dimensional state vector `(x, v)`.
/// The derivative function must return a tuple `(dx/dt, dv/dt)`.
///
/// Returns `None` if `dt` is outside the allowed domain.
pub fn rk4_step_2d<F>(t: f64, y: (f64, f64), dt: f64, derivative_fn: F) -> Option<(f64, f64)>
where
    F: Fn(f64, (f64, f64)) -> (f64, f64),
{
    // domain guard
    if dt < 1.0e-9 || dt > 1000.0 {
        return None;
    }

    // k1 … k4 as tuples
    let k1 = derivative_fn(t, y);
    let y_k2 = (y.0 + 0.5 * dt * k1.0, y.1 + 0.5 * dt * k1.1);
    let k2 = derivative_fn(t + 0.5 * dt, y_k2);
    let y_k3 = (y.0 + 0.5 * dt * k2.0, y.1 + 0.5 * dt * k2.1);
    let k3 = derivative_fn(t + 0.5 * dt, y_k3);
    let y_k4 = (y.0 + dt * k3.0, y.1 + dt * k3.1);
    let k4 = derivative_fn(t + dt, y_k4);

    // RK4 update for each component
    let x_next = y.0 + dt / 6.0 * (k1.0 + 2.0 * k2.0 + 2.0 * k3.0 + k4.0);
    let v_next = y.1 + dt / 6.0 * (k1.1 + 2.0 * k2.1 + 2.0 * k3.1 + k4.1);
    Some((x_next, v_next))
}
