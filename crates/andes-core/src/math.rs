//! Utilidades numéricas que replican la semántica exacta de JavaScript.

/// `Math.round`: redondea la mitad hacia +infinito (no hacia el más cercano par).
///
/// La implementación nativa `f64::round` redondea la mitad alejándose de cero,
/// lo que diverge de JS para los negativos (`Math.round(-0.5) === -0`).
#[inline]
pub fn js_round(x: f64) -> f64 {
    (x + 0.5).floor()
}

/// `ToUint32` de ECMAScript: trunca hacia cero, aplica módulo 2^32 y trata
/// `NaN`, `±0` e `±Inf` como `0`.
#[inline]
pub fn to_uint32(n: f64) -> u32 {
    if !n.is_finite() || n == 0.0 {
        return 0;
    }
    let m = n.trunc() % 4_294_967_296.0;
    let m = if m < 0.0 { m + 4_294_967_296.0 } else { m };
    m as u32
}

/// `ToInt32` de ECMAScript: mismo patrón de bits que `ToUint32`, reinterpretado.
#[inline]
pub fn to_int32(n: f64) -> i32 {
    to_uint32(n) as i32
}

/// `Math.max(0, Math.min(1, x))` (propaga `NaN`, como JS).
#[inline]
pub fn clamp01(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}
