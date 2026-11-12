use rust_bricks::astro_panchanga_yoga_karana::*;

#[test]
fn test_golden_vectors() {
    // 日月同在 0 毫度：Yoga 1 (Vishkambha), Karana 1 (Bava, 第 1 Tithi 前半)
    assert_eq!(astro_panchanga_yoga_karana(0, 0), Some((1, 1)));

    // 月亮 142.5 度 (142500 milli-deg -> 8550 arcmin), 太陽 30.0 度 (30000 milli-deg -> 1800 arcmin)
    // Sum = 10350 arcmin -> 10350 / 800 = 12 -> Yoga 13 (Dhriti)
    // Delta = 112500 milli-deg -> 112500 / 6000 = 18 -> Karana 19 (Bhadra / Vishti)
    assert_eq!(astro_panchanga_yoga_karana(142_500, 30_000), Some((13, 19)));

    // 越界拒絕：大於 360000 毫度
    assert!(astro_panchanga_yoga_karana(360_001, 10_000).is_none());
    assert!(astro_panchanga_yoga_karana(10_000, 360_001).is_none());
}

#[test]
fn test_boundary_values() {
    // Upper domain boundary check
    assert_eq!(astro_panchanga_yoga_karana(360_000, 360_000), Some((1, 1)));

    // Maximum Yoga (27) and Karana (60) boundaries
    assert_eq!(astro_panchanga_yoga_karana(359_999, 0), Some((27, 60)));
}
