//! Biomas: pesos, geometría, tinte y floración (puerto parcial de `src/biomes.js`).
//!
//! Fase 2: se portan los pesos, la geometría, el tinte y la floración. Los pools
//! de flora/fauna (que dependen de los registros `FLORA`/`SPECIES`) llegan con
//! esos módulos.

use crate::noise::fbm1;
use crate::palette::{field_index, lerp_color, Rgb};
use crate::rng::hash1;

pub const BIOME_IDS: [&str; 6] = [
    "altiplano",
    "norte",
    "centro",
    "sur",
    "patagonia",
    "austral",
];

const BIOME_FREQ: f64 = 0.00015;
const BLOOM_BLOCK: f64 = 60000.0;
const BLOOM_CHANCE: f64 = 0.28;

#[derive(Clone, Copy)]
pub struct Geometry {
    pub amp_mul: f64,
    /// Desplazamiento de la línea de nieve: **positivo = más nieve** (línea más
    /// baja), igual que en las estaciones.
    pub snow_shift: f64,
    pub fjord: f64,
}

struct Biome {
    palette: &'static [(&'static str, Rgb)],
    geometry: Geometry,
}

const fn hex(v: u32) -> Rgb {
    [(v >> 16) as u8, (v >> 8) as u8, v as u8]
}

const fn geo(amp: f64, snow: f64, fjord: f64) -> Geometry {
    Geometry {
        amp_mul: amp,
        snow_shift: snow,
        fjord,
    }
}

const BIOMES: [Biome; 6] = [
    Biome {
        palette: &[
            ("valleyL", hex(0x9a9668)),
            ("valleyD", hex(0x6e6c4a)),
            ("costaL", hex(0x8e8a60)),
            ("costaD", hex(0x646044)),
            ("floraL", hex(0x7e8654)),
            ("floraD", hex(0x565c38)),
            ("sand", hex(0xe2d6ae)),
            ("sandD", hex(0xb8ac84)),
            ("rock", hex(0x8f8378)),
            ("rockD", hex(0x6b6058)),
            ("snow", hex(0xffffff)),
            ("snowD", hex(0xe4eaf0)),
        ],
        geometry: geo(1.12, 0.1, 0.0),
    },
    Biome {
        palette: &[
            ("valleyL", hex(0xa8a05a)),
            ("valleyD", hex(0x7a7440)),
            ("costaL", hex(0x9a9450)),
            ("costaD", hex(0x6e6a38)),
            ("floraL", hex(0x8a8a4a)),
            ("floraD", hex(0x5e5e30)),
            ("sand", hex(0xe8d29a)),
            ("sandD", hex(0xc8ae72)),
            ("rock", hex(0x8a7a6a)),
            ("rockD", hex(0x6a5a4e)),
            ("snow", hex(0xfff6e6)),
            ("snowD", hex(0xe0d2b8)),
        ],
        geometry: geo(0.92, -0.3, 0.0),
    },
    Biome {
        palette: &[],
        geometry: geo(1.0, 0.0, 0.0),
    },
    Biome {
        palette: &[
            ("valleyL", hex(0x3f7a3a)),
            ("valleyD", hex(0x2c5a2a)),
            ("costaL", hex(0x2f7a3a)),
            ("costaD", hex(0x1f5a28)),
            ("floraL", hex(0x2a6a34)),
            ("floraD", hex(0x1c4a24)),
            ("sand", hex(0xd8c090)),
            ("sandD", hex(0xb0986e)),
            ("rock", hex(0x5f6a70)),
            ("rockD", hex(0x48525a)),
            ("snow", hex(0xffffff)),
            ("snowD", hex(0xd8e6f0)),
        ],
        geometry: geo(1.06, 0.3, 0.0),
    },
    Biome {
        palette: &[
            ("valleyL", hex(0x8a8a5e)),
            ("valleyD", hex(0x62623e)),
            ("costaL", hex(0x727a52)),
            ("costaD", hex(0x525a3a)),
            ("floraL", hex(0x6a7a48)),
            ("floraD", hex(0x485838)),
            ("sand", hex(0xd8cba0)),
            ("sandD", hex(0xb0a37c)),
            ("rock", hex(0x6b7078)),
            ("rockD", hex(0x4e545c)),
            ("snow", hex(0xffffff)),
            ("snowD", hex(0xdbe8f2)),
        ],
        geometry: geo(1.02, 0.45, 0.0),
    },
    Biome {
        palette: &[
            ("valleyL", hex(0x3d6b52)),
            ("valleyD", hex(0x2a4a3a)),
            ("costaL", hex(0x356048)),
            ("costaD", hex(0x244234)),
            ("floraL", hex(0x2f6a46)),
            ("floraD", hex(0x1e4a32)),
            ("sand", hex(0xb8c4bc)),
            ("sandD", hex(0x8ea098)),
            ("rock", hex(0x5a6a6e)),
            ("rockD", hex(0x414f54)),
            ("snow", hex(0xffffff)),
            ("snowD", hex(0xdbe8f2)),
        ],
        geometry: geo(1.08, 0.5, 1.0),
    },
];

fn clamp01(v: f64) -> f64 {
    v.clamp(0.0, 1.0)
}

fn smoothstep(a: f64, b: f64, x: f64) -> f64 {
    let t = clamp01((x - a) / (b - a));
    t * t * (3.0 - 2.0 * t)
}

pub fn biome_index(id: &str) -> Option<usize> {
    BIOME_IDS.iter().position(|b| *b == id)
}

/// Pesos normalizados (suman 1) de cada bioma en una posición de mundo.
pub fn biome_weights(world_x: f64, seed: u32) -> [f64; 6] {
    let u = fbm1(world_x * BIOME_FREQ, seed.wrapping_add(4242), 3, 2.0, 0.5);
    let pos = smoothstep(0.15, 0.85, u) * (BIOME_IDS.len() as f64 - 0.5);
    let mut out = [0.0; 6];
    let mut total = 0.0;
    for (i, o) in out.iter_mut().enumerate() {
        let lo = clamp01((pos - (i as f64 - 0.75)) / 0.5);
        let hi = clamp01(((i as f64 + 0.75) - pos) / 0.5);
        *o = lo * hi;
        total += lo * hi;
    }
    let t = if total != 0.0 { total } else { 1.0 };
    for v in out.iter_mut() {
        *v /= t;
    }
    out
}

/// Pesos según el modo: región fija o procedural (`auto`).
pub fn mode_weights(mode: &str, world_x: f64, seed: u32) -> [f64; 6] {
    if let Some(i) = biome_index(mode) {
        let mut w = [0.0; 6];
        w[i] = 1.0;
        return w;
    }
    biome_weights(world_x, seed)
}

/// Tinte mezclado por pesos (solo biomas con paleta; centro no tiñe).
pub fn tint_for(weights: &[f64; 6]) -> Vec<(usize, Rgb)> {
    let mut acc: Vec<(usize, Rgb, f64)> = Vec::new();
    for (i, &w) in weights.iter().enumerate() {
        if w <= 0.0 {
            continue;
        }
        for &(key, color) in BIOMES[i].palette {
            let idx = field_index(key).unwrap();
            match acc.iter_mut().find(|(k, _, _)| *k == idx) {
                Some((_, c, aw)) => {
                    let nw = *aw + w;
                    *c = lerp_color(*c, color, w / nw);
                    *aw = nw;
                }
                None => acc.push((idx, color, w)),
            }
        }
    }
    acc.into_iter().map(|(k, c, _)| (k, c)).collect()
}

/// Geometría mezclada por pesos: amplitud, nieve y fiordos.
pub fn biome_geometry(weights: &[f64; 6]) -> Geometry {
    let mut amp_mul = 0.0;
    let mut snow_shift = 0.0;
    let mut fjord = 0.0;
    for (i, &w) in weights.iter().enumerate() {
        if w <= 0.0 {
            continue;
        }
        let g = BIOMES[i].geometry;
        amp_mul += w * g.amp_mul;
        snow_shift += w * g.snow_shift;
        fjord += w * g.fjord;
    }
    Geometry {
        amp_mul: if amp_mul != 0.0 { amp_mul } else { 1.0 },
        snow_shift,
        fjord,
    }
}

/// Floración de desierto: bloques raros y extensos, solo con presencia de norte.
pub fn bloom_at(world_x: f64, seed: u32, weights: &[f64; 6]) -> f64 {
    let gate = weights[biome_index("norte").unwrap()];
    if gate <= 0.01 {
        return 0.0;
    }
    let c = (world_x / BLOOM_BLOCK).floor();
    if hash1(c as i32, seed.wrapping_add(0xb100)) > BLOOM_CHANCE {
        return 0.0;
    }
    let u = world_x / BLOOM_BLOCK - c;
    let env = smoothstep(0.0, 0.15, u) * smoothstep(1.0, 0.85, u);
    clamp01(gate * env)
}

/// Resuelve la floración según el modo (`auto`/`on`/`off`).
pub fn resolve_bloom(mode: &str, weights: &[f64; 6], bloom: f64) -> f64 {
    match mode {
        "off" => 0.0,
        "on" => {
            if weights[biome_index("norte").unwrap()] > 0.05 {
                1.0
            } else {
                0.0
            }
        }
        _ => bloom,
    }
}

/// Contexto de bioma en una posición de mundo.
pub struct BiomeContext {
    pub weights: [f64; 6],
    pub tint: Vec<(usize, Rgb)>,
    pub amount: f64,
    pub bloom: f64,
}

pub fn biome_at(world_x: f64, seed: u32, mode: &str) -> BiomeContext {
    let weights = mode_weights(mode, world_x, seed);
    let tint = tint_for(&weights);
    let amount = 1.0 - weights[biome_index("centro").unwrap()];
    let block = (world_x / BLOOM_BLOCK).floor();
    let center = mode_weights(mode, (block + 0.5) * BLOOM_BLOCK, seed);
    let bloom = bloom_at(world_x, seed, &center);
    BiomeContext {
        weights,
        tint,
        amount,
        bloom,
    }
}

/// Peso de la flora de floración dentro del pool.
pub const BLOOM_FLORA_WEIGHT: f64 = 2.5;
const BLOOM_CHANCE_MUL: f64 = 0.6;

/// Multiplicador de densidad de fauna durante la floración.
pub fn bloom_chance_mul(bloom: f64) -> f64 {
    1.0 + BLOOM_CHANCE_MUL * bloom
}

/// Pool de flora ponderado por bioma (desde las zonas del registro `FLORA`).
/// El orden replica el de `biomes.js`: bioma (BIOME_IDS) por fuera, registro por dentro.
pub fn biome_flora_pool(
    layer_name: &str,
    weights: &[f64; 6],
    bloom: f64,
) -> Vec<(&'static str, f64)> {
    let mut out = Vec::new();
    for (i, &w) in weights.iter().enumerate() {
        if w <= 0.0 {
            continue;
        }
        let biome = BIOME_IDS[i];
        for e in crate::flora::FLORA {
            for &(b, layer, zw) in e.zones {
                if b == biome && layer == layer_name {
                    out.push((e.name, w * zw));
                }
            }
        }
    }
    if bloom > 0.0 {
        out.push(("flower", bloom * BLOOM_FLORA_WEIGHT));
    }
    out
}

/// Peso de la fauna de floración dentro del pool.
pub const BLOOM_FAUNA_WEIGHT: f64 = 1.5;

/// Pool de fauna ponderado por bioma (desde las zonas del registro `SPECIES`).
pub fn biome_fauna_pool(
    layer_name: &str,
    weights: &[f64; 6],
    bloom: f64,
) -> Vec<(&'static str, f64)> {
    let mut out = Vec::new();
    for (i, &w) in weights.iter().enumerate() {
        if w <= 0.0 {
            continue;
        }
        let biome = BIOME_IDS[i];
        for sp in crate::fauna::SPECIES {
            for &(b, layer, zw) in sp.zones {
                if b == biome && layer == layer_name {
                    out.push((sp.name, w * zw));
                }
            }
        }
    }
    if bloom > 0.0 {
        for sp in crate::fauna::SPECIES {
            if sp.bloom {
                out.push((sp.name, bloom * BLOOM_FAUNA_WEIGHT));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: u32 = 662118718; // seedToInt("andes")

    #[test]
    fn weights_are_deterministic_and_sum_one() {
        let mut x = -5000.0;
        while x <= 5000.0 {
            let a = biome_weights(x, SEED);
            let b = biome_weights(x, SEED);
            assert_eq!(a, b);
            let total: f64 = a.iter().sum();
            assert!((total - 1.0).abs() < 1e-9, "suma {total} en x={x}");
            assert!(a.iter().all(|v| (0.0..=1.0).contains(v)));
            x += 250.0;
        }
    }

    #[test]
    fn each_biome_dominates_somewhere() {
        let mut max = [0.0f64; 6];
        let mut x = 0.0;
        while x < 300000.0 {
            let w = biome_weights(x, SEED);
            for i in 0..6 {
                max[i] = max[i].max(w[i]);
            }
            x += 250.0;
        }
        for (i, id) in BIOME_IDS.iter().enumerate() {
            assert!(max[i] > 0.9, "{id} nunca domina (max {})", max[i]);
        }
    }

    #[test]
    fn mode_weights_fixes_region() {
        for id in BIOME_IDS {
            let w = mode_weights(id, 1234.0, SEED);
            for (i, other) in BIOME_IDS.iter().enumerate() {
                assert_eq!(w[i], if *other == id { 1.0 } else { 0.0 });
            }
        }
        assert_eq!(
            mode_weights("auto", 1234.0, SEED),
            biome_weights(1234.0, SEED)
        );
    }

    #[test]
    fn centro_has_no_tint_or_bloom() {
        let b = biome_at(1234.0, SEED, "centro");
        assert_eq!(b.weights[biome_index("centro").unwrap()], 1.0);
        assert_eq!(b.amount, 0.0);
        assert!(b.tint.is_empty());
        assert_eq!(b.bloom, 0.0);
    }

    #[test]
    fn geometry_mixes_and_centro_changes_nothing() {
        let c = biome_geometry(&mode_weights("centro", 0.0, SEED));
        assert_eq!((c.amp_mul, c.snow_shift, c.fjord), (1.0, 0.0, 0.0));
        assert!(biome_geometry(&mode_weights("norte", 0.0, SEED)).amp_mul < 1.0);
        assert!(biome_geometry(&mode_weights("sur", 0.0, SEED)).amp_mul > 1.0);
        assert_eq!(
            biome_geometry(&mode_weights("austral", 0.0, SEED)).fjord,
            1.0
        );
    }

    #[test]
    fn bloom_only_where_north() {
        let centro = mode_weights("centro", 0.0, SEED);
        let mut x = 0.0;
        while x < 20000.0 {
            assert_eq!(bloom_at(x, SEED, &centro), 0.0);
            x += 100.0;
        }
        let norte = mode_weights("norte", 0.0, SEED);
        let mut positivos = 0;
        let mut x = 0.0;
        while x < 200000.0 {
            let v = bloom_at(x, SEED, &norte);
            assert!((0.0..=1.0).contains(&v));
            if v > 0.0 {
                positivos += 1;
            }
            x += 100.0;
        }
        assert!(positivos > 0);
    }

    #[test]
    fn resolve_bloom_modes() {
        let norte = mode_weights("norte", 0.0, SEED);
        let centro = mode_weights("centro", 0.0, SEED);
        assert_eq!(resolve_bloom("off", &norte, 0.7), 0.0);
        assert_eq!(resolve_bloom("on", &norte, 0.0), 1.0);
        assert_eq!(resolve_bloom("on", &centro, 0.0), 0.0);
        assert_eq!(resolve_bloom("auto", &norte, 0.7), 0.7);
    }
}
