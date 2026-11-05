// ## Map
// Role: Solar zenith and elevation angle computation from day-of-year and UTC time
// Inputs: day_of_year: u16, utc_hour: f64, lat_deg: f64, lon_deg: f64
// Outputs: Option<(f64, f64)> (zenith_deg, elevation_deg)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 2
/// @domain: day_of_year [1..=366]
/// @domain: utc_hour [0.0..=24.0]
/// @domain: lat_deg [-90.0..=90.0]
/// @domain: lon_deg [-180.0..=180.0]
///
/// Compute solar zenith and elevation angles for a given day of year,
/// UTC hour, latitude and longitude. Returns `None` if any input is out of
/// its allowed domain.
///
/// The algorithm follows the Cooper approximation for solar declination
/// and uses local solar time to obtain the hour angle.
pub fn astro_solar_zenith_and_elevation(
    day_of_year: u16,
    utc_hour: f64,
    lat_deg: f64,
    lon_deg: f64,
) -> Option<(f64, f64)> {
    // ---- domain guards -------------------------------------------------------
    if !(1..=366).contains(&day_of_year) {
        return None;
    }
    if !utc_hour.is_finite() || utc_hour < 0.0 || utc_hour > 24.0 {
        return None;
    }
    if lat_deg.abs() > 90.0 || lon_deg.abs() > 180.0 {
        return None;
    }

    // ---- solar declination (Cooper equation) ---------------------------------
    let b = 2.0 * std::f64::consts::PI * ((284 + day_of_year as i32) as f64) / 365.0;
    let declination_rad = (23.45_f64).to_radians() * b.sin();

    // ---- local solar time and hour angle ------------------------------------
    let local_solar_time = utc_hour + lon_deg / 15.0;
    let hour_angle_rad = ((local_solar_time - 12.0) * 15.0).to_radians();

    // ---- zenith angle --------------------------------------------------------
    let lat_rad = lat_deg.to_radians();
    let cos_zenith = lat_rad.sin() * declination_rad.sin()
        + lat_rad.cos() * declination_rad.cos() * hour_angle_rad.cos();
    let cos_z_clamped = cos_zenith.clamp(-1.0, 1.0);
    let zenith_deg = cos_z_clamped.acos().to_degrees();

    // ---- elevation angle ------------------------------------------------------
    let elevation_deg = 90.0 - zenith_deg;

    Some((zenith_deg, elevation_deg))
}
