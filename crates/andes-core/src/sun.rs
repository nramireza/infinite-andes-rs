//! Sol real: amanecer/atardecer por día del año y latitud, y reasignación de la
//! hora real a la curva de la paleta (puerto de `src/sun.js`).

use std::f64::consts::PI;

const DEG: f64 = PI / 180.0;

/// Declinación solar aproximada (grados) para un día del año.
pub fn solar_declination(day_of_year: f64) -> f64 {
    -23.44 * ((2.0 * PI * ((day_of_year + 10.0) / 365.25)).cos())
}

fn equation_of_time(day_of_year: f64) -> f64 {
    let b = 2.0 * PI * ((day_of_year - 81.0) / 364.0);
    9.87 * (2.0 * b).sin() - 7.53 * b.cos() - 1.5 * b.sin()
}

#[derive(Clone, Copy, Debug)]
pub struct SolarTimes {
    pub rise: f64,
    pub set: f64,
    pub noon: f64,
}

/// Salida/puesta del sol en hora local. `tz_hours` al oeste es negativo (Chile).
pub fn solar_times(day_of_year: f64, lat: f64, longitude: f64, tz_hours: f64) -> SolarTimes {
    let decl = solar_declination(day_of_year) * DEG;
    let phi = lat * DEG;
    let cos_h = (-phi.tan() * decl.tan()).clamp(-1.0, 1.0);
    let half_day = cos_h.acos() / DEG / 15.0;
    let noon = 12.0 - longitude / 15.0 + tz_hours + equation_of_time(day_of_year) / 60.0;
    SolarTimes {
        rise: noon - half_day,
        set: noon + half_day,
        noon,
    }
}

/// Anclas de la curva de paleta/cielo.
pub const PAL_RISE: f64 = 6.0;
pub const PAL_SET: f64 = 18.25;

/// Reasigna la hora real a la curva fija: amanecer→PAL_RISE, atardecer→PAL_SET.
pub fn solar_clock(hour: f64, rise: f64, set: f64) -> f64 {
    let span = set - rise;
    if span.is_nan() || span <= 0.1 {
        return hour;
    }
    if hour >= rise && hour <= set {
        return PAL_RISE + (hour - rise) * (PAL_SET - PAL_RISE) / span;
    }
    let h = if hour >= set { hour } else { hour + 24.0 };
    let night_span = rise + 24.0 - set;
    let mapped = PAL_SET + (h - set) * (PAL_RISE + 24.0 - PAL_SET) / night_span;
    mapped % 24.0
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAT: f64 = -33.45;
    const LON: f64 = -70.66;

    #[test]
    fn declination_reaches_solstices() {
        assert!((solar_declination(355.0) + 23.44).abs() < 0.6);
        assert!((solar_declination(172.0) - 23.44).abs() < 0.6);
    }

    #[test]
    fn austral_summer_is_long_winter_short() {
        let verano = solar_times(355.0, LAT, LON, -3.0);
        let invierno = solar_times(172.0, LAT, LON, -4.0);
        assert!(verano.rise < 7.0);
        assert!(verano.set > 20.0);
        assert!(invierno.rise > 7.0);
        assert!(invierno.set < 18.0);
        assert!(verano.set - verano.rise > invierno.set - invierno.rise);
    }

    #[test]
    fn solar_clock_is_anchored_and_monotone() {
        let t = solar_times(355.0, LAT, LON, -3.0);
        assert!((solar_clock(t.rise, t.rise, t.set) - PAL_RISE).abs() < 1e-9);
        assert!((solar_clock(t.set, t.rise, t.set) - PAL_SET).abs() < 1e-9);
        let mut prev = solar_clock(t.rise, t.rise, t.set);
        let mut h = t.rise + 0.25;
        while h <= t.set {
            let v = solar_clock(h, t.rise, t.set);
            assert!(v >= prev);
            prev = v;
            h += 0.25;
        }
        assert!(
            (solar_clock(t.set - 0.01, t.rise, t.set) - solar_clock(t.set + 0.01, t.rise, t.set))
                .abs()
                < 0.1
        );
    }

    #[test]
    fn solar_clock_survives_invalid_span() {
        assert_eq!(solar_clock(12.0, 12.0, 12.0), 12.0);
    }
}
