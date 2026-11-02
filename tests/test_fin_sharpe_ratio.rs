use rust_bricks::fin_sharpe_ratio::*;

#[test]
fn test_sharpe_ratio_golden() {
    let returns = [0.02_f64, 0.04, 0.06];
    let annual_rf = 0.0_f64;
    let periods_per_year = 12.0_f64;

    let result = fin_sharpe_ratio(&returns, annual_rf, periods_per_year)
        .expect("valid Sharpe calculation");
    let expected = 6.928203230275509; // (2.0 * sqrt(12))
    assert!((result - expected).abs() < 1e-3, "Sharpe = {}, expected {}", result, expected);
}

#[test]
fn test_information_ratio_golden() {
    let port = [0.05_f64, 0.07, 0.06];
    let bench = [0.02_f64, 0.03, 0.04];
    let periods_per_year = 12.0_f64;

    let result = fin_information_ratio(&port, &bench, periods_per_year)
        .expect("valid Information Ratio calculation");
    let expected = 10.392304845413264; // (3.0 * sqrt(12))
    assert!((result - expected).abs() < 1e-3, "IR = {}, expected {}", result, expected);
}
