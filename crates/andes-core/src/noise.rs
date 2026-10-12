//! Ruido de valor 1D con interpolación suave + fBm (puerto de `src/noise.js`).

use crate::math::to_int32;
use crate::rng::hash1;

#[inline]
fn smooth(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

pub fn value_noise1(x: f64, seed: u32) -> f64 {
    let i = x.floor();
    let f = x - i;
    let a = hash1(to_int32(i), seed);
    let b = hash1(to_int32(i + 1.0), seed);
    a + (b - a) * smooth(f)
}

pub fn fbm1(x: f64, seed: u32, octaves: u32, lacunarity: f64, gain: f64) -> f64 {
    let mut amp = 0.5;
    let mut freq = 1.0;
    let mut sum = 0.0;
    let mut norm = 0.0;
    for o in 0..octaves {
        let s = seed.wrapping_add(o.wrapping_mul(1013));
        sum += amp * value_noise1(x * freq, s);
        norm += amp;
        amp *= gain;
        freq *= lacunarity;
    }
    sum / norm
}

/// Ruido "ridged": crestas afiladas, ideal para montañas.
pub fn ridged1(x: f64, seed: u32, octaves: u32) -> f64 {
    let mut amp = 0.5;
    let mut freq = 1.0;
    let mut sum = 0.0;
    let mut norm = 0.0;
    for o in 0..octaves {
        let s = seed.wrapping_add(o.wrapping_mul(7919));
        let n = value_noise1(x * freq, s);
        sum += amp * (1.0 - (n * 2.0 - 1.0).abs());
        norm += amp;
        amp *= 0.5;
        freq *= 2.1;
    }
    sum / norm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_noise1_is_deterministic_and_continuous() {
        assert_eq!(value_noise1(3.7, 11), value_noise1(3.7, 11));
        let a = value_noise1(3.7, 11);
        let b = value_noise1(3.7 + 0.001, 11);
        assert!((a - b).abs() < 0.01, "salto grande: {}", (a - b).abs());
    }

    #[test]
    fn fbm1_and_ridged1_stay_in_unit_interval() {
        let mut x = -20.0;
        while x < 20.0 {
            let f = fbm1(x, 5, 4, 2.0, 0.5);
            let r = ridged1(x, 5, 3);
            assert!((0.0..=1.0).contains(&f), "fbm fuera de rango: {f}");
            assert!((0.0..=1.0).contains(&r), "ridged fuera de rango: {r}");
            x += 0.13;
        }
    }

    #[test]
    fn fbm1_and_ridged1_are_deterministic() {
        assert_eq!(fbm1(12.34, 99, 4, 2.0, 0.5), fbm1(12.34, 99, 4, 2.0, 0.5));
        assert_eq!(ridged1(12.34, 99, 4), ridged1(12.34, 99, 4));
    }
}
