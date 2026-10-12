//! Utilidades de dibujo pixel-art. Fase 1: solo la parte determinista que no
//! depende del lienzo (`radixDither`). El resto (`bakeSprite`, primitivas)
//! llegará con el rasterizador.

use crate::math::to_uint32;

/// Ruido de dithering determinista en `[0,1)`.
///
/// Reproduce la aritmética en `f64` de JS (los productos pueden superar 2^53;
/// hay que redondear a doble **antes** de `ToUint32`, no con enteros exactos).
pub fn radix_dither(x: i32, y: i32, seed: u32) -> f64 {
    let n =
        (x as f64) * 374_761_393.0 + (y as f64) * 668_265_263.0 + (seed as f64) * 2_246_822_519.0;
    let n = to_uint32(n);
    let mut h = n ^ (n >> 13);
    h = h.wrapping_mul(1_274_126_177);
    ((h ^ (h >> 16)) as f64) / 4_294_967_296.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radix_dither_is_deterministic_and_in_range() {
        assert_eq!(radix_dither(1, 2, 3), radix_dither(1, 2, 3));
        for (x, y) in [(0, 0), (5, 9), (123, 45), (-7, -7)] {
            let v = radix_dither(x, y, 42);
            assert!((0.0..1.0).contains(&v), "fuera de rango: {v}");
        }
    }
}
