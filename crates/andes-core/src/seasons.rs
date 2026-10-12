//! Estaciones: tinte global, línea de nieve y sesgo de clima (puerto de
//! `src/seasons.js`).

use crate::palette::{field_index, lerp_color, Palette, Rgb};

pub const SEASON_IDS: [&str; 4] = ["verano", "otono", "invierno", "primavera"];
pub const SEASON_DURATION: f64 = 120.0;
pub const SEASON_STRENGTH: f64 = 0.5;
const HOLD: f64 = 0.6;

/// Orden de las claves de clima (el mismo de `seasons.js`).
pub const WEATHERS: [&str; 6] = ["clear", "snow", "rain", "fog", "wind", "storm"];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SeasonState {
    pub index: usize,
    pub next: usize,
    pub t: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct Glow {
    pub color: Rgb,
    pub amount: f64,
}

const fn hex(hex: u32) -> Rgb {
    [(hex >> 16) as u8, (hex >> 8) as u8, hex as u8]
}

const SEASON_KEYS: [&str; 12] = [
    "floraL",
    "floraD",
    "costaL",
    "costaD",
    "valleyL",
    "valleyD",
    "sand",
    "sandD",
    "skyHorizon",
    "skyMid",
    "snow",
    "snowD",
];

struct Season {
    tint: &'static [(&'static str, Rgb)],
    snow_shift: f64,
    weather_bias: [f64; 6],
    glow: Option<Glow>,
}

const fn g(hexv: u32, amount: f64) -> Glow {
    Glow {
        color: hex(hexv),
        amount,
    }
}

const SEASONS: [Season; 4] = [
    Season {
        tint: &[],
        snow_shift: -0.18,
        weather_bias: [2.2, 0.2, 0.5, 0.6, 1.0, 0.15],
        glow: None,
    },
    Season {
        tint: &[
            ("floraL", hex(0x8a6a2e)),
            ("floraD", hex(0x5e4420)),
            ("costaL", hex(0x8a6a2e)),
            ("costaD", hex(0x5e4420)),
            ("valleyL", hex(0x8a7a34)),
            ("valleyD", hex(0x5e5226)),
            ("sand", hex(0xdcb884)),
            ("sandD", hex(0xb8935e)),
            ("skyHorizon", hex(0xe6c08a)),
        ],
        snow_shift: 0.12,
        weather_bias: [1.0, 0.6, 0.9, 1.0, 1.8, 0.6],
        glow: Some(g(0xff9a4a, 0.45)),
    },
    Season {
        tint: &[
            ("floraL", hex(0x3a5a4a)),
            ("floraD", hex(0x26403a)),
            ("costaL", hex(0x38564a)),
            ("costaD", hex(0x26403a)),
            ("valleyL", hex(0x4a6a52)),
            ("valleyD", hex(0x33503c)),
            ("sand", hex(0xb8bec4)),
            ("sandD", hex(0x98a0a8)),
            ("skyHorizon", hex(0xcfd8e0)),
        ],
        snow_shift: 0.32,
        weather_bias: [0.8, 2.2, 1.4, 1.2, 1.2, 1.0],
        glow: Some(g(0xb8c8f0, 0.4)),
    },
    Season {
        tint: &[
            ("floraL", hex(0x4a9a44)),
            ("floraD", hex(0x2f6a30)),
            ("costaL", hex(0x489a44)),
            ("costaD", hex(0x2f6a30)),
            ("valleyL", hex(0x6aa848)),
            ("valleyD", hex(0x48803a)),
            ("sand", hex(0xe2c896)),
            ("sandD", hex(0xc0a070)),
            ("skyHorizon", hex(0xd6ecf2)),
        ],
        snow_shift: -0.05,
        weather_bias: [1.6, 0.3, 0.8, 0.7, 1.2, 0.3],
        glow: Some(g(0xffe6b0, 0.35)),
    },
];

fn season_index(mode: &str) -> Option<usize> {
    SEASON_IDS.iter().position(|s| *s == mode)
}

fn smoothstep(a: f64, b: f64, x: f64) -> f64 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn season_state(phase: f64, mode: &str) -> SeasonState {
    if let Some(i) = season_index(mode) {
        return SeasonState {
            index: i,
            next: i,
            t: 0.0,
        };
    }
    let p = ((phase % 4.0) + 4.0) % 4.0;
    let index = p.floor() as usize;
    let t = smoothstep(HOLD, 1.0, p - index as f64);
    SeasonState {
        index,
        next: (index + 1) % 4,
        t,
    }
}

fn blend_num(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Desplazamiento de la línea de nieve (se suma al del bioma).
pub fn season_snow_shift(state: SeasonState) -> f64 {
    let a = SEASONS[state.index].snow_shift;
    let b = SEASONS[state.next].snow_shift;
    blend_num(a, b, state.t)
}

/// Pesos de clima de la estación (mezclados entre actual y siguiente).
pub fn season_weather_bias(state: SeasonState) -> [f64; 6] {
    let a = SEASONS[state.index].weather_bias;
    let b = SEASONS[state.next].weather_bias;
    let mut out = [0.0; 6];
    for (o, (x, y)) in out.iter_mut().zip(a.iter().zip(b.iter())) {
        *o = blend_num(*x, *y, state.t);
    }
    out
}

/// Tinte mezclado entre estación actual y siguiente (solo claves de estación).
fn season_tint(state: SeasonState) -> Vec<(usize, Rgb)> {
    let a = SEASONS[state.index].tint;
    let b = SEASONS[state.next].tint;
    let mut out = Vec::new();
    for &(key, color) in a {
        if !SEASON_KEYS.contains(&key) {
            continue;
        }
        let target = if state.t > 0.0 {
            match b.iter().find(|(k, _)| *k == key) {
                Some((_, c)) => lerp_color(color, *c, state.t),
                None => color,
            }
        } else {
            color
        };
        out.push((field_index(key).unwrap(), target));
    }
    out
}

/// Resplandor estacional mezclado; `None` = base (verano).
pub fn season_glow_tint(state: SeasonState) -> Option<Glow> {
    let a = SEASONS[state.index].glow;
    let b = SEASONS[state.next].glow;
    match (a, b) {
        (None, None) => None,
        (None, Some(b)) => Some(Glow {
            color: b.color,
            amount: b.amount * state.t,
        }),
        (Some(a), None) => Some(Glow {
            color: a.color,
            amount: a.amount * (1.0 - state.t),
        }),
        (Some(a), Some(b)) => Some(Glow {
            color: if state.t > 0.0 {
                lerp_color(a.color, b.color, state.t)
            } else {
                a.color
            },
            amount: a.amount + (b.amount - a.amount) * state.t,
        }),
    }
}

/// Aplica el tinte de estación a una paleta. `strength` en 0..1.
pub fn apply_season(pal: &Palette, state: SeasonState, strength: f64) -> Palette {
    let s = strength.clamp(0.0, 1.0);
    let mut out = *pal;
    if s <= 0.0 {
        return out;
    }
    for (idx, tint) in season_tint(state) {
        out[idx] = lerp_color(out[idx], tint, s);
    }
    out
}

/// Elige un clima según el sesgo de estación. `rnd` devuelve `[0,1)`.
pub fn pick_season_weather<F: FnMut() -> f64>(state: SeasonState, mut rnd: F) -> &'static str {
    let bias = season_weather_bias(state);
    let total: f64 = bias.iter().sum();
    if total <= 0.0 {
        return WEATHERS[0];
    }
    let mut r = rnd() * total;
    for (i, k) in WEATHERS.iter().enumerate() {
        r -= bias[i];
        if r < 0.0 {
            return k;
        }
    }
    WEATHERS[WEATHERS.len() - 1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::get_palette;

    #[test]
    fn season_state_holds_and_crossfades() {
        let fijo = season_state(1.3, "otono");
        assert_eq!((fijo.index, fijo.next, fijo.t), (1, 1, 0.0));
        assert_eq!(
            season_state(0.0, "auto"),
            SeasonState {
                index: 0,
                next: 1,
                t: 0.0
            }
        );
        assert_eq!(season_state(0.9, "auto").index, 0);
        assert!(season_state(0.9, "auto").t > 0.0);
        assert_eq!(season_state(1.0, "auto").index, 1);
        assert_eq!(season_state(1.0, "auto").t, 0.0);
        assert_eq!(season_state(3.9, "auto").index, 3);
        assert_eq!(season_state(3.9, "auto").next, 0);
        assert_eq!(season_state(4.0, "auto").index, 0);
    }

    #[test]
    fn winter_lowers_snow_line() {
        let inv = season_snow_shift(season_state(0.0, "invierno"));
        let ver = season_snow_shift(season_state(0.0, "verano"));
        assert!(inv > 0.0 && ver < 0.0 && inv > ver);
    }

    #[test]
    fn winter_bias_favors_snow() {
        let inv = season_weather_bias(season_state(0.0, "invierno"));
        let ver = season_weather_bias(season_state(0.0, "verano"));
        let i_snow = WEATHERS.iter().position(|w| *w == "snow").unwrap();
        let i_clear = WEATHERS.iter().position(|w| *w == "clear").unwrap();
        assert!(inv.iter().all(|v| *v >= 0.0));
        assert!(inv[i_snow] > ver[i_snow]);
        assert!(ver[i_clear] > inv[i_clear]);
    }

    #[test]
    fn every_season_has_storm_bias() {
        let i_storm = WEATHERS.iter().position(|w| *w == "storm").unwrap();
        for id in SEASON_IDS {
            assert!(season_weather_bias(season_state(0.0, id))[i_storm] > 0.0);
        }
    }

    #[test]
    fn pick_season_weather_is_deterministic_and_biased() {
        let st = season_state(0.0, "invierno");
        assert_eq!(
            pick_season_weather(st, || 0.1),
            pick_season_weather(st, || 0.1)
        );
        let mut snow = 0;
        let mut r = 0.0;
        for _ in 0..1000 {
            r = (r + 0.37) % 1.0;
            if pick_season_weather(st, || r) == "snow" {
                snow += 1;
            }
        }
        assert!(snow > 250, "se esperaba bastante nieve (fue {snow}/1000)");
    }

    #[test]
    fn apply_season_only_touches_season_keys() {
        let base = get_palette(12.0, "clear", 1.0);
        let out = apply_season(&base, season_state(0.0, "otono"), 0.5);
        for (i, (before, after)) in base.iter().zip(out.iter()).enumerate() {
            if before != after {
                assert!(SEASON_KEYS.contains(&crate::palette::FIELDS[i]));
            }
        }
    }

    #[test]
    fn apply_season_zero_or_summer_keeps_palette() {
        let base = get_palette(12.0, "clear", 1.0);
        assert_eq!(apply_season(&base, season_state(0.0, "otono"), 0.0), base);
        assert_eq!(apply_season(&base, season_state(0.0, "verano"), 0.5), base);
    }

    #[test]
    fn glow_is_none_in_summer() {
        assert!(season_glow_tint(season_state(0.0, "verano")).is_none());
        assert!(season_glow_tint(season_state(0.0, "otono")).unwrap().amount > 0.0);
        let mezcla = season_glow_tint(season_state(0.9, "auto")).unwrap();
        assert!(mezcla.amount > 0.0);
    }
}
