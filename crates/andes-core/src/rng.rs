//! PRNG determinista + hashing por chunk (puerto exacto de `src/rng.js`).

use crate::math::to_uint32;

/// `mulberry32`: PRNG de 32 bits reproducible.
pub struct Mulberry32 {
    a: u32,
}

impl Mulberry32 {
    pub fn new(seed: u32) -> Self {
        Self { a: seed }
    }

    #[inline]
    pub fn next_f64(&mut self) -> f64 {
        self.a = self.a.wrapping_add(0x6d2b79f5);
        let mut t = (self.a ^ (self.a >> 15)).wrapping_mul(1 | self.a);
        t = (t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t))) ^ t;
        ((t ^ (t >> 14)) as f64) / 4_294_967_296.0
    }
}

/// FNV-1a sobre las unidades de código UTF-16 (igual que `charCodeAt`).
pub fn hash_string(s: &str) -> u32 {
    let mut h: u32 = 2_166_136_261;
    for unit in s.encode_utf16() {
        h ^= unit as u32;
        h = h.wrapping_mul(16_777_619);
    }
    h
}

/// Hash determinista de un entero -> entero de 32 bits (para sembrar PRNGs).
#[inline]
pub fn hash_int(n: i32, seed: u32) -> u32 {
    let mut h = (n as u32) ^ seed.wrapping_mul(0x9e37_79b1);
    h = (h ^ (h >> 16)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    h ^ (h >> 16)
}

/// Hash determinista de un entero -> `[0,1)`. Independiente del orden.
#[inline]
pub fn hash1(n: i32, seed: u32) -> f64 {
    hash_int(n, seed) as f64 / 4_294_967_296.0
}

/// Semilla legible -> entero de 32 bits.
///
/// Reproduce `seedToInt`: número (si el texto es numérico finito) o hash FNV-1a
/// del texto recortado. Se ignoran las formas exóticas que `Number()` acepta y
/// `f64::parse` no (p. ej. `0x1f`); no se usan como semillas.
pub fn seed_to_int(seed: &str) -> u32 {
    let s = seed.trim();
    if s.is_empty() {
        return 0;
    }
    match s.parse::<f64>() {
        Ok(n) if n.is_finite() => to_uint32(n),
        _ => hash_string(s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mulberry32_is_reproducible() {
        let mut a = Mulberry32::new(42);
        let mut b = Mulberry32::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_f64(), b.next_f64());
        }
    }

    #[test]
    fn mulberry32_stays_in_unit_interval() {
        let mut r = Mulberry32::new(7);
        for _ in 0..1000 {
            let v = r.next_f64();
            assert!((0.0..1.0).contains(&v), "fuera de rango: {v}");
        }
    }

    #[test]
    fn mulberry32_differs_by_seed() {
        assert_ne!(Mulberry32::new(1).next_f64(), Mulberry32::new(2).next_f64());
    }

    #[test]
    fn hash1_is_deterministic_and_in_range() {
        assert_eq!(hash1(123, 9), hash1(123, 9));
        assert_eq!(hash1(5, 0), hash1(5, 0));
        for i in -50..50 {
            let v = hash1(i, 3);
            assert!((0.0..1.0).contains(&v), "fuera de rango: {v}");
        }
    }

    #[test]
    fn hash_int_seeds_distinct_prngs() {
        for i in -20..20 {
            let _ = hash_int(i, 123);
        }
        assert_ne!(hash_int(1, 0), hash_int(2, 0));
        assert_ne!(
            Mulberry32::new(hash_int(1, 0)).next_f64(),
            Mulberry32::new(hash_int(2, 0)).next_f64()
        );
    }

    #[test]
    fn hash_string_is_stable() {
        assert_eq!(hash_string("andes"), hash_string("andes"));
        assert_ne!(hash_string("andes"), hash_string("pewen"));
    }

    #[test]
    fn seed_to_int_normalizes_text_number_and_empty() {
        assert_eq!(seed_to_int("42"), 42);
        assert_eq!(seed_to_int("pewen"), hash_string("pewen"));
        assert_eq!(seed_to_int(""), 0);
        assert_eq!(seed_to_int("  "), 0);
    }
}
