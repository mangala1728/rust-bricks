use rust_bricks::astro_tithi_special_detector::*;

#[test]
fn test_kshaya_tithi() {
    assert_eq!(
        astro_tithi_special_detector(1200, 1800, 1000, 2000),
        Some(1)
    );
}

#[test]
fn test_vriddhi_tithi() {
    assert_eq!(
        astro_tithi_special_detector(900, 2100, 1000, 2000),
        Some(2)
    );
}

#[test]
fn test_normal_tithi() {
    assert_eq!(
        astro_tithi_special_detector(800, 1500, 1000, 2000),
        Some(0)
    );
}

#[test]
fn test_invalid_tithi_order() {
    assert!(astro_tithi_special_detector(2000, 1000, 500, 1500).is_none());
}

#[test]
fn test_invalid_sunrise_order() {
    assert!(astro_tithi_special_detector(1000, 2000, 1500, 1000).is_none());
}
