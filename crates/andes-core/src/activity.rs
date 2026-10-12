//! Reactividad del sistema: convierte la actividad real del equipo en un
//! multiplicador global de densidad de fauna (puerto de `src/activity.js`).
//!
//! No decide la identidad ni la posición de ningún animal (eso sigue siendo
//! determinista por semilla): solo pondera la probabilidad de aparición, igual
//! que la hora o la floración. El mapeo es puro y sin dependencias.

pub const ACTIVITY_MIN: f64 = 0.5;
pub const ACTIVITY_MAX: f64 = 1.8;

const WEIGHT_CPU: f64 = 0.55;
const WEIGHT_RAM: f64 = 0.25;
const WEIGHT_HEAT: f64 = 0.3;
const WEIGHT_LOAD: f64 = 0.2;
const TEMP_BASE: f64 = 55.0;
const TEMP_SPAN: f64 = 45.0;
const LOAD_SPAN: f64 = 4.0;

/// Señales del equipo. Un valor no finito (`f64::NAN`) equivale a "ausente":
/// `cpu`/`ram`/`load` caen a 0 y `temp` a `TEMP_BASE` (sin penalización térmica).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ActivitySignals {
    pub cpu: f64,
    pub ram: f64,
    pub temp: f64,
    pub load: f64,
}

fn num(v: f64, fallback: f64) -> f64 {
    if v.is_finite() {
        v
    } else {
        fallback
    }
}

fn clamp01(v: f64) -> f64 {
    v.clamp(0.0, 1.0)
}

/// Señales → multiplicador acotado a `[ACTIVITY_MIN, ACTIVITY_MAX]`. Sin
/// señales (`None`) el resultado es `1.0`: el render actual queda intacto.
pub fn activity_multiplier(signals: Option<ActivitySignals>) -> f64 {
    let Some(sig) = signals else {
        return 1.0;
    };
    let cpu = clamp01(num(sig.cpu, 0.0));
    let ram = clamp01(num(sig.ram, 0.0));
    let heat = clamp01((num(sig.temp, TEMP_BASE) - TEMP_BASE) / TEMP_SPAN);
    let load = clamp01(num(sig.load, 0.0) / LOAD_SPAN);
    let raw = 1.0 + WEIGHT_CPU * cpu + WEIGHT_RAM * ram - WEIGHT_HEAT * heat - WEIGHT_LOAD * load;
    raw.clamp(ACTIVITY_MIN, ACTIVITY_MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_signals_is_neutral() {
        assert_eq!(activity_multiplier(None), 1.0);
    }

    #[test]
    fn missing_signals_are_neutral() {
        let s = ActivitySignals {
            cpu: f64::NAN,
            ram: f64::NAN,
            temp: f64::NAN,
            load: f64::NAN,
        };
        assert_eq!(activity_multiplier(Some(s)), 1.0);
    }

    #[test]
    fn full_cpu_and_ram_raise_multiplier() {
        let s = ActivitySignals {
            cpu: 1.0,
            ram: 1.0,
            temp: TEMP_BASE,
            load: 0.0,
        };
        assert!((activity_multiplier(Some(s)) - 1.8).abs() < 1e-12);
    }

    #[test]
    fn heat_and_load_lower_multiplier() {
        let s = ActivitySignals {
            cpu: 0.0,
            ram: 0.0,
            temp: TEMP_BASE + TEMP_SPAN,
            load: LOAD_SPAN,
        };
        assert!((activity_multiplier(Some(s)) - ACTIVITY_MIN).abs() < 1e-12);
    }

    #[test]
    fn clamps_out_of_range_inputs() {
        let s = ActivitySignals {
            cpu: 5.0,
            ram: -3.0,
            temp: -100.0,
            load: 1e6,
        };
        let m = activity_multiplier(Some(s));
        assert!((ACTIVITY_MIN..=ACTIVITY_MAX).contains(&m));
    }
}
