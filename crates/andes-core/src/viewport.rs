//! Dimensiones del lienzo: altura fija y relación de aspecto configurable.
//! (puerto exacto de `src/viewport.js`).

use crate::math::js_round;

pub const BASE_H: u32 = 270;
pub const DEFAULT_ASPECT: &str = "32:9";

pub struct AspectPreset {
    pub id: &'static str,
    pub ratio: f64,
}

pub const ASPECT_PRESETS: [AspectPreset; 3] = [
    AspectPreset {
        id: "16:9",
        ratio: 16.0 / 9.0,
    },
    AspectPreset {
        id: "21:9",
        ratio: 21.0 / 9.0,
    },
    AspectPreset {
        id: "32:9",
        ratio: 32.0 / 9.0,
    },
];

pub fn width_for_ratio(ratio: f64) -> u32 {
    let r = if ratio.is_finite() && ratio > 0.0 {
        ratio
    } else {
        32.0 / 9.0
    };
    let w = js_round(BASE_H as f64 * r);
    if w < 1.0 {
        1
    } else {
        w as u32
    }
}

/// Acepta `"32:9"`, `"21x9"`, `"16/9"` o un decimal y devuelve la relación.
pub fn ratio_from_string(s: &str) -> Option<f64> {
    let str = s.trim();
    if let Some((a, b)) = parse_ratio(str) {
        if a > 0.0 && b > 0.0 {
            return Some(a / b);
        }
        return None;
    }
    match str.parse::<f64>() {
        Ok(n) if n.is_finite() && n > 0.0 => Some(n),
        _ => None,
    }
}

/// `^(\d+(\.\d+)?)\s*[:xX/]\s*(\d+(\.\d+)?)$`
fn parse_ratio(s: &str) -> Option<(f64, f64)> {
    let bytes = s.as_bytes();
    let mut i = 0;
    let a = parse_number(bytes, &mut i)?;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() || !matches!(bytes[i], b':' | b'x' | b'X' | b'/') {
        return None;
    }
    i += 1;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    let b = parse_number(bytes, &mut i)?;
    if i != bytes.len() {
        return None;
    }
    Some((a, b))
}

fn parse_number(bytes: &[u8], i: &mut usize) -> Option<f64> {
    let start = *i;
    while *i < bytes.len() && bytes[*i].is_ascii_digit() {
        *i += 1;
    }
    if *i == start {
        return None;
    }
    if *i < bytes.len() && bytes[*i] == b'.' {
        *i += 1;
        let frac = *i;
        while *i < bytes.len() && bytes[*i].is_ascii_digit() {
            *i += 1;
        }
        if *i == frac {
            return None;
        }
    }
    std::str::from_utf8(&bytes[start..*i])
        .ok()?
        .parse::<f64>()
        .ok()
}

/// Etiqueta canónica para la URL: preset conocido o relación decimal.
pub fn label_for_ratio(ratio: f64) -> String {
    for p in &ASPECT_PRESETS {
        if (p.ratio - ratio).abs() < 0.0001 {
            return p.id.to_string();
        }
    }
    format!("{ratio:.3}:1")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn width_for_ratio_derives_width_from_fixed_height() {
        assert_eq!(width_for_ratio(16.0 / 9.0), 480);
        assert_eq!(width_for_ratio(21.0 / 9.0), 630);
        assert_eq!(width_for_ratio(32.0 / 9.0), 960);
        assert_eq!(width_for_ratio(f64::NAN), 960);
    }

    #[test]
    fn presets_cover_canonical_aspects() {
        assert_eq!(DEFAULT_ASPECT, "32:9");
        let ids: Vec<&str> = ASPECT_PRESETS.iter().map(|p| p.id).collect();
        assert_eq!(ids, ["16:9", "21:9", "32:9"]);
        assert_eq!(
            width_for_ratio(ratio_from_string(DEFAULT_ASPECT).unwrap()),
            960
        );
        assert_eq!(BASE_H, 270);
    }

    #[test]
    fn ratio_from_string_accepts_separators_and_decimals() {
        assert!((ratio_from_string("32:9").unwrap() - 32.0 / 9.0).abs() < 1e-9);
        assert!((ratio_from_string("21x9").unwrap() - 21.0 / 9.0).abs() < 1e-9);
        assert!((ratio_from_string("16/9").unwrap() - 16.0 / 9.0).abs() < 1e-9);
        assert_eq!(ratio_from_string("2.4"), Some(2.4));
        assert_eq!(ratio_from_string("malo"), None);
        assert_eq!(ratio_from_string(""), None);
    }

    #[test]
    fn label_for_ratio_recognizes_presets_and_decimals() {
        assert_eq!(label_for_ratio(32.0 / 9.0), "32:9");
        assert_eq!(label_for_ratio(16.0 / 9.0), "16:9");
        assert_eq!(label_for_ratio(2.4), "2.400:1");
    }
}
