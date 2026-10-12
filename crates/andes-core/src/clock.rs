//! Reloj real: hora local, día del año, estación y fase lunar.
//! (puerto de `src/clock.js`). En vez de `Date` se reciben las componentes; el
//! paso a número de día usa el calendario civil (equivalente al `Date` local en
//! una zona sin horario de verano, como UTC).

use crate::seasons::SEASON_IDS;

/// Componentes de fecha/hora local (mes 0-based, como `Date`).
#[derive(Clone, Copy, Debug)]
pub struct DateTime {
    pub year: i64,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub millis: u32,
}

impl DateTime {
    pub const fn new(
        year: i64,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        millis: u32,
    ) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            millis,
        }
    }

    /// Fracción de día (0..24) con minutos, segundos y milisegundos.
    pub fn local_hour(&self) -> f64 {
        self.hour as f64
            + self.minute as f64 / 60.0
            + self.second as f64 / 3600.0
            + self.millis as f64 / 3_600_000.0
    }

    /// Día del año (0 = 1 de enero).
    pub fn day_of_year(&self) -> i64 {
        day_number(self.year, self.month, self.day) - day_number(self.year, 0, 1)
    }
}

/// Número de días desde 1970-01-01 (algoritmo civil de Hinnant).
pub fn day_number(y: i64, m0: u32, d: u32) -> i64 {
    let y = if m0 <= 1 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let m = m0 as i64 + 1;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Verano dic–feb, otoño mar–may, invierno jun–ago, primavera sep–nov.
pub fn season_for_date(month0: u32) -> &'static str {
    if month0 == 11 || month0 <= 1 {
        "verano"
    } else if month0 <= 4 {
        "otono"
    } else if month0 <= 7 {
        "invierno"
    } else {
        "primavera"
    }
}

fn season_start(year: i64, id: &str) -> (i64, u32, u32) {
    match id {
        "verano" => (year, 11, 1),
        "otono" => (year, 2, 1),
        "invierno" => (year, 5, 1),
        _ => (year, 8, 1),
    }
}

/// Fase continua 0..4 alineada con `SEASON_IDS` (verano=0).
pub fn season_phase_for_date(dt: &DateTime) -> f64 {
    let id = season_for_date(dt.month);
    let index = SEASON_IDS.iter().position(|s| *s == id).unwrap() as f64;

    let mut start_year = dt.year;
    if id == "verano" && dt.month <= 1 {
        start_year -= 1;
    }
    let (sy, sm, sd) = season_start(start_year, id);
    let next_id = SEASON_IDS[((index as usize) + 1) % 4];
    let next_year = if id == "verano" {
        start_year + 1
    } else {
        start_year
    };
    let (ny, nm, nd) = season_start(next_year, next_id);

    let tod = (dt.hour as f64 * 3600.0
        + dt.minute as f64 * 60.0
        + dt.second as f64
        + dt.millis as f64 / 1000.0)
        / 86_400.0;
    let date = day_number(dt.year, dt.month, dt.day) as f64 + tod;
    let start = day_number(sy, sm, sd) as f64;
    let next = day_number(ny, nm, nd) as f64;
    let progress = (date - start) / (next - start);
    index + progress.clamp(0.0, 1.0)
}

/// Ciclo sinódico aproximado (días).
pub const MOON_CYCLE: f64 = 29.530588;

fn leap_years_before(y: i64) -> i64 {
    let y1 = y - 1;
    y1.div_euclid(4) - y1.div_euclid(100) + y1.div_euclid(400)
}

/// Días desde el 1 de enero de 2000 (0 = ese día).
pub fn days_since_2000(year: i64, day_of_year: i64) -> i64 {
    (year - 2000) * 365 + (leap_years_before(year) - leap_years_before(2000)) + day_of_year
}

/// Fase `[0,1)` para días desde la época de luna nueva (2000-01-06).
pub fn moon_phase_from_days(days: f64) -> f64 {
    let d = ((days - 5.0) % MOON_CYCLE + MOON_CYCLE) % MOON_CYCLE;
    d / MOON_CYCLE
}

pub fn moon_phase_for_date(year: i64, day_of_year: i64) -> f64 {
    moon_phase_from_days(days_since_2000(year, day_of_year) as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_hour_has_fraction() {
        assert_eq!(DateTime::new(2026, 0, 15, 20, 30, 0, 0).local_hour(), 20.5);
        assert_eq!(DateTime::new(2026, 0, 15, 0, 0, 0, 0).local_hour(), 0.0);
        assert_eq!(DateTime::new(2026, 0, 15, 23, 15, 0, 0).local_hour(), 23.25);
    }

    #[test]
    fn day_of_year_from_jan_first() {
        assert_eq!(DateTime::new(2026, 0, 1, 0, 0, 0, 0).day_of_year(), 0);
        assert_eq!(DateTime::new(2026, 1, 1, 0, 0, 0, 0).day_of_year(), 31);
        assert_eq!(DateTime::new(2026, 11, 31, 0, 0, 0, 0).day_of_year(), 364);
    }

    #[test]
    fn season_uses_southern_hemisphere() {
        assert_eq!(season_for_date(0), "verano");
        assert_eq!(season_for_date(11), "verano");
        assert_eq!(season_for_date(3), "otono");
        assert_eq!(season_for_date(6), "invierno");
        assert_eq!(season_for_date(9), "primavera");
    }

    #[test]
    fn season_phase_is_continuous_and_exact_at_starts() {
        assert_eq!(
            season_phase_for_date(&DateTime::new(2026, 11, 1, 0, 0, 0, 0)),
            0.0
        );
        assert_eq!(
            season_phase_for_date(&DateTime::new(2026, 2, 1, 0, 0, 0, 0)),
            1.0
        );
        assert_eq!(
            season_phase_for_date(&DateTime::new(2026, 5, 1, 0, 0, 0, 0)),
            2.0
        );
        assert_eq!(
            season_phase_for_date(&DateTime::new(2026, 8, 1, 0, 0, 0, 0)),
            3.0
        );
        let mid = season_phase_for_date(&DateTime::new(2026, 0, 15, 0, 0, 0, 0));
        assert!(
            mid > 0.3 && mid < 0.7,
            "fase de verano fuera de rango: {mid}"
        );
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn moon_phase_is_deterministic_bounded_and_cyclic() {
        assert!(MOON_CYCLE > 29.0 && MOON_CYCLE < 30.0);
        let nueva = moon_phase_for_date(2000, 5);
        assert!(
            nueva < 0.02 || nueva > 0.98,
            "se esperaba luna nueva: {nueva}"
        );
        let llena = moon_phase_for_date(2000, 20);
        assert!(
            (llena - 0.5).abs() < 0.03,
            "se esperaba luna llena: {llena}"
        );
        let a = moon_phase_for_date(2000, 5);
        let b = moon_phase_from_days(days_since_2000(2000, 5) as f64 + MOON_CYCLE);
        assert!((a - b).abs() < 0.01);
        for d in [-30, 0, 100, 364] {
            let p = moon_phase_for_date(2026, d);
            assert!((0.0..1.0).contains(&p), "fase fuera de rango: {p}");
        }
    }
}
