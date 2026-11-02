use rust_bricks::fin_kelly_criterion::*;

#[test]
fn golden_vectors() {
    // Vector 1 – full Kelly, win_prob=0.60, odds_b=1.0
    let f1 = match kelly_optimal_fraction(0.60, 1.0, 1.0) {
        Some(v) => v,
        None => {
            // @safety-proof – input is within the documented domain
            panic!("valid input produced None");
        }
    };
    assert!((f1 - 0.20).abs() < 1e-4);

    // Vector 2 – half Kelly
    let f2 = match kelly_optimal_fraction(0.60, 1.0, 0.5) {
        Some(v) => v,
        None => {
            // @safety-proof – input is within the documented domain
            panic!("valid input produced None");
        }
    };
    assert!((f2 - 0.10).abs() < 1e-4);

    // Vector 3 – negative expectation → zero position
    let f3 = match kelly_optimal_fraction(0.40, 1.0, 1.0) {
        Some(v) => v,
        None => {
            // @safety-proof – input is within the documented domain
            panic!("valid input produced None");
        }
    };
    assert!((f3 - 0.0).abs() < 1e-12);

    // Out‑of‑domain arguments must yield None
    assert!(kelly_optimal_fraction(0.0, 1.0, 1.0).is_none());
    assert!(kelly_optimal_fraction(0.5, 0.0, 1.0).is_none());
    assert!(kelly_optimal_fraction(0.5, 1.0, 0.0).is_none());
}
