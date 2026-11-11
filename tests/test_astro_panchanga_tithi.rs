use rust_bricks::astro_panchanga_tithi::*;

#[test]
fn test_golden_vectors() {
    // 基準恆等：日月同度（0度合相，新月/朔 Amavasya 終點，第 1 Tithi 初始）
    assert_eq!(astro_panchanga_tithi(0, 0), Some((1, 0)));

    // 朔月 180 度沖相（滿月/望 Purnima，Delta = 180000 -> Tithi 16, progress 0）
    assert_eq!(astro_panchanga_tithi(180_000, 0), Some((16, 0)));

    // 月亮 142.5 度 (142500)，太陽 30.0 度 (30000) -> Delta = 112500 -> Tithi 10 (Dashami), 3750 bp
    assert_eq!(astro_panchanga_tithi(142_500, 30_000), Some((10, 3750)));

    // 越界拒絕：大於 360000 毫度
    assert!(astro_panchanga_tithi(360_001, 10_000).is_none());
    assert!(astro_panchanga_tithi(10_000, 360_001).is_none());
}

#[test]
fn test_boundary_and_edge_cases() {
    // 邊界值：最大有效度數 360000 毫度與 0 毫度同餘
    assert_eq!(astro_panchanga_tithi(360_000, 360_000), Some((1, 0)));
    assert_eq!(astro_panchanga_tithi(360_000, 0), Some((1, 0)));

    // Tithi 1 的最後一毫度 (11999) -> 9999 bp
    assert_eq!(astro_panchanga_tithi(11_999, 0), Some((1, 9999)));

    // Tithi 2 的第一毫度 (12000) -> 0 bp
    assert_eq!(astro_panchanga_tithi(12_000, 0), Some((2, 0)));

    // 跨越 0 度邊界（月 5 度，日 355 度 -> Delta = 10 度 = 10000 毫度 -> Tithi 1, 8333 bp）
    assert_eq!(astro_panchanga_tithi(5_000, 355_000), Some((1, 8333)));

    // Tithi 30 的最後一毫度 (359999) -> Tithi 30, 9999 bp
    assert_eq!(astro_panchanga_tithi(359_999, 0), Some((30, 9999)));
}
