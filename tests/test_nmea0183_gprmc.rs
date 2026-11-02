use rust_bricks::nmea0183_gprmc::*;

#[test]
fn test_checksum_valid() {
    let sentence = "$GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W*6A";
    assert!(nmea_validate_checksum(sentence));
}

#[test]
fn test_checksum_invalid() {
    // Corrupt checksum (change last digit)
    let sentence = "$GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W*00";
    assert!(!nmea_validate_checksum(sentence));
}

#[test]
fn test_parse_lat_lon_success() {
    let raw_lat = "4807.038";
    let lat_dir = 'N';
    let raw_lon = "01131.000";
    let lon_dir = 'E';
    let result = nmea_parse_lat_lon(raw_lat, lat_dir, raw_lon, lon_dir).unwrap();

    let expected_lat = 48.0 + 7.038 / 60.0;      // 48.1173
    let expected_lon = 11.0 + 31.000 / 60.0;    // 11.516666...

    assert!((result.0 - expected_lat).abs() < 1e-4);
    assert!((result.1 - expected_lon).abs() < 1e-4);
}

#[test]
fn test_parse_lat_lon_invalid_direction() {
    // Invalid latitude direction
    let raw_lat = "4807.038";
    let lat_dir = 'X';
    let raw_lon = "01131.000";
    let lon_dir = 'E';
    assert!(nmea_parse_lat_lon(raw_lat, lat_dir, raw_lon, lon_dir).is_none());

    // Invalid longitude direction
    let lat_dir = 'N';
    let lon_dir = 'Y';
    assert!(nmea_parse_lat_lon(raw_lat, lat_dir, raw_lon, lon_dir).is_none());
}
