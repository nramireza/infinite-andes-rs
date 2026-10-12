//! Paletas del paisaje: interpolación por hora + modificadores de clima y bioma.
//! Los datos viven en `palette_data.rs`, que es **fuente de este repo** (snapshot
//! inicial derivado del proyecto JS).

#![allow(clippy::type_complexity)]

use crate::math::{clamp01, js_round};

include!("palette_data.rs");

pub type Rgb = [u8; 3];
/// Paleta completa, en el orden de `FIELDS`.
pub type Palette = [Rgb; NFIELDS];

pub fn field_index(name: &str) -> Option<usize> {
    FIELDS.iter().position(|f| *f == name)
}

/// Acceso por nombre (devuelve negro si la clave no existe).
pub fn get(pal: &Palette, name: &str) -> Rgb {
    field_index(name).map(|i| pal[i]).unwrap_or([0, 0, 0])
}

pub fn hex_to_rgb(hex: &str) -> Rgb {
    let h = hex.trim_start_matches('#');
    if h.len() < 6 {
        return [0, 0, 0];
    }
    let ch = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).unwrap_or(0);
    [ch(0), ch(2), ch(4)]
}

pub fn rgb_to_hex(c: Rgb) -> String {
    format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}

#[inline]
fn clamp_byte(v: f64) -> u8 {
    js_round(v).clamp(0.0, 255.0) as u8
}

pub fn lerp_color(a: Rgb, b: Rgb, t: f64) -> Rgb {
    let mut out = [0u8; 3];
    for i in 0..3 {
        out[i] = clamp_byte(a[i] as f64 + (b[i] as f64 - a[i] as f64) * t);
    }
    out
}

pub fn shade(c: Rgb, amt: f64) -> Rgb {
    let mut out = [0u8; 3];
    for i in 0..3 {
        let v = if amt >= 0.0 {
            c[i] as f64 + (255.0 - c[i] as f64) * amt
        } else {
            c[i] as f64 * (1.0 + amt)
        };
        out[i] = clamp_byte(v);
    }
    out
}

fn interp_keys(time: f64) -> Palette {
    let t = ((time % 1.0) + 1.0) % 1.0;
    let n = KEYS.len();
    let mut i = 0usize;
    while i < n - 2 && t >= KEYS[i + 1].0 {
        i += 1;
    }
    let (at, ap) = &KEYS[i];
    let (bt, bp) = &KEYS[i + 1];
    let span = if *bt - *at != 0.0 { *bt - *at } else { 1.0 };
    let k = clamp01((t - *at) / span);
    let mut out = [[0u8; 3]; NFIELDS];
    for (f, slot) in out.iter_mut().enumerate() {
        *slot = lerp_color(ap[f], bp[f], k);
    }
    out
}

pub fn apply_weather(pal: &Palette, weather: &str, strength: f64) -> Palette {
    let st = clamp01(strength);
    let Some((_, targets)) = WEATHER_TARGETS.iter().find(|(name, _)| *name == weather) else {
        return *pal;
    };
    let mut out = *pal;
    for (idx, color) in targets.iter() {
        let k = if weather == "fog" {
            st * 0.8
        } else if weather == "wind" {
            st * 0.35
        } else {
            st * 0.7
        };
        out[*idx] = lerp_color(out[*idx], *color, k);
    }
    if weather == "snow" {
        if let Some(i) = field_index("snow") {
            out[i] = [255, 255, 255];
        }
    }
    if weather == "rain" {
        if let Some(i) = field_index("water") {
            out[i] = lerp_color(out[i], hex_to_rgb("#3a4750"), st * 0.5);
        }
    }
    out
}

/// Aplica el tinte de bioma (`amount` 0..1) y el rubor de la floración (`bloom` 0..1).
pub fn apply_biome(pal: &Palette, tint: &[(usize, Rgb)], amount: f64, bloom: f64) -> Palette {
    let a = clamp01(amount);
    let b = clamp01(bloom);
    if (tint.is_empty() || a <= 0.0) && b <= 0.0 {
        return *pal;
    }
    let mut out = *pal;
    if a > 0.0 {
        for (idx, color) in tint {
            if !BIOME_KEYS.contains(idx) {
                continue;
            }
            out[*idx] = lerp_color(out[*idx], *color, a);
        }
    }
    if b > 0.0 {
        for (idx, color, factor) in BLOOM_TINTS.iter() {
            out[*idx] = lerp_color(out[*idx], *color, b * factor);
        }
    }
    out
}

pub fn get_palette(hour24: f64, weather: &str, strength: f64) -> Palette {
    let base = interp_keys(hour24 / 24.0);
    apply_weather(&base, weather, strength)
}

/// Mezcla dos paletas clima a clima (crossfade): `k` interpola de `from` a `to`.
pub fn lerp_palettes(from: &Palette, to: &Palette, k: f64) -> Palette {
    let t = clamp01(k);
    let mut out = *to;
    for (i, slot) in out.iter_mut().enumerate() {
        *slot = lerp_color(from[i], to[i], t);
    }
    out
}

/// Cuánto "de noche" está (0 = pleno día, 1 = noche cerrada).
pub fn night_amount(hour24: f64) -> f64 {
    let h = ((hour24 % 24.0) + 24.0) % 24.0;
    if (7.0..=18.0).contains(&h) {
        return 0.0;
    }
    if h > 18.0 && h < 20.0 {
        return (h - 18.0) / 2.0;
    }
    if h > 5.0 && h < 7.0 {
        return 1.0 - (h - 5.0) / 2.0;
    }
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lerp_color_respects_extremes() {
        assert_eq!(
            rgb_to_hex(lerp_color(
                hex_to_rgb("#000000"),
                hex_to_rgb("#ffffff"),
                0.0
            )),
            "#000000"
        );
        assert_eq!(
            rgb_to_hex(lerp_color(
                hex_to_rgb("#000000"),
                hex_to_rgb("#ffffff"),
                1.0
            )),
            "#ffffff"
        );
        assert_eq!(
            rgb_to_hex(lerp_color(
                hex_to_rgb("#123456"),
                hex_to_rgb("#123456"),
                0.5
            )),
            "#123456"
        );
    }

    #[test]
    fn shade_saturates_without_overflow() {
        assert_eq!(rgb_to_hex(shade(hex_to_rgb("#000000"), -1.0)), "#000000");
        assert_eq!(rgb_to_hex(shade(hex_to_rgb("#ffffff"), 1.0)), "#ffffff");
        assert_eq!(rgb_to_hex(shade(hex_to_rgb("#000000"), 1.0)), "#ffffff");
    }

    #[test]
    fn hex_rgb_round_trip() {
        assert_eq!(hex_to_rgb("#ff8000"), [255, 128, 0]);
        assert_eq!(rgb_to_hex([255, 128, 0]), "#ff8000");
    }

    #[test]
    fn get_palette_has_all_keys_at_several_hours() {
        for h in [0.0, 6.0, 12.0, 18.0, 23.9] {
            let pal = get_palette(h, "clear", 1.0);
            for name in ["skyTop", "skyMid", "sea", "snow", "floraL", "trunk", "rock"] {
                assert!(field_index(name).is_some(), "falta {name}");
                let _ = get(&pal, name);
            }
        }
    }

    #[test]
    fn weather_modifies_palette() {
        let clear = get_palette(12.0, "clear", 1.0);
        assert_eq!(
            get(&get_palette(12.0, "snow", 1.0), "snow"),
            [255, 255, 255]
        );
        assert_ne!(
            get(&get_palette(12.0, "rain", 1.0), "sea"),
            get(&clear, "sea")
        );
        assert_ne!(
            get(&get_palette(12.0, "storm", 1.0), "sea"),
            get(&clear, "sea")
        );
        assert_ne!(
            get(&get_palette(12.0, "storm", 1.0), "skyTop"),
            get(&get_palette(12.0, "rain", 1.0), "skyTop")
        );
    }

    #[test]
    fn night_amount_goes_from_zero_to_one() {
        assert_eq!(night_amount(12.0), 0.0);
        assert_eq!(night_amount(0.0), 1.0);
        assert_eq!(night_amount(6.0), 0.5);
        assert_eq!(night_amount(19.0), 0.5);
        for h in [-3.0, 0.0, 5.0, 8.0, 15.0, 19.0, 24.0, 30.0] {
            let v = night_amount(h);
            assert!((0.0..=1.0).contains(&v), "nightAmount({h}) = {v}");
        }
    }
}
