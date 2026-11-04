// 本檔由 cloud_farmer 依 specs/nmea0183_gprmc.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::nmea0183_gprmc::*;

#[test]
fn golden_vectors_from_spec() {
    assert!(nmea_validate_checksum("$GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W*6A"), "spec golden vector #1");
    assert!(!nmea_validate_checksum("$GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W*6B"), "spec golden vector #2");
    assert!(!nmea_validate_checksum("GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W*6A"), "spec golden vector #3");
    assert!(!nmea_validate_checksum("$GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W"), "spec golden vector #4");
    let (lat_ne, lon_ne) = nmea_parse_lat_lon("4807.038", 'N', "01131.000", 'E').unwrap();
    assert!((lat_ne - 48.1173).abs() < 1e-4, "spec golden vector #5");
    assert!((lon_ne - 11.516667).abs() < 1e-4, "spec golden vector #6");
    let (lat_sw, lon_sw) = nmea_parse_lat_lon("4807.038", 'S', "01131.000", 'W').unwrap();
    assert!((lat_sw + 48.1173).abs() < 1e-4, "spec golden vector #7");
    assert!((lon_sw + 11.516667).abs() < 1e-4, "spec golden vector #8");
    assert!(nmea_parse_lat_lon("4807.038", 'X', "01131.000", 'E').is_none(), "spec golden vector #9");
    assert!(nmea_parse_lat_lon("4807.038", 'N', "01131.000", 'X').is_none(), "spec golden vector #10");
}
