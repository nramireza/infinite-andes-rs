//! Fauna endémica en pixel art (puerto de `src/fauna.js`): registro generado,
//! colocación determinista por chunk/actividad y dibujo de sprites.

include!("fauna_data.rs");

use crate::biomes::{biome_fauna_pool, bloom_chance_mul, resolve_bloom};
use crate::framebuffer::Framebuffer;
use crate::math::js_round;
use crate::palette::{get, shade, Palette, Rgb};
use crate::rng::{hash1, hash_int, Mulberry32};
use crate::terrain::{Layer, Terrain};

#[inline]
fn r(x: f64) -> i32 {
    js_round(x) as i32
}

pub fn species(name: &str) -> Option<&'static SpeciesData> {
    SPECIES.iter().find(|s| s.name == name)
}

fn rarity_weight(rarity: &str) -> f64 {
    RARITY_WEIGHT
        .iter()
        .find(|(k, _)| *k == rarity)
        .map(|(_, w)| *w)
        .unwrap_or(1.0)
}

pub fn species_weight(type_: &str) -> f64 {
    species(type_)
        .map(|s| rarity_weight(s.rarity))
        .unwrap_or(0.0)
}

pub fn is_active(sp: &SpeciesData, hour: f64) -> bool {
    let h = ((hour % 24.0) + 24.0) % 24.0;
    match sp.active {
        "day" => (7.0..=19.0).contains(&h),
        "night" => h >= 20.0 || h <= 6.0,
        "dusk" => (17.0..=21.0).contains(&h) || (5.0..=8.0).contains(&h),
        _ => true,
    }
}

fn entry_weight(type_: &str, extra: f64) -> f64 {
    species_weight(type_) * extra
}

fn pick_weighted(list: &[(&'static str, f64)], rng: &mut Mulberry32) -> &'static str {
    let total: f64 = list.iter().map(|(t, w)| entry_weight(t, *w)).sum();
    if total <= 0.0 {
        return list[0].0;
    }
    let mut r = rng.next_f64() * total;
    for (t, w) in list {
        r -= entry_weight(t, *w);
        if r < 0.0 {
            return t;
        }
    }
    list[list.len() - 1].0
}

fn sprite_colors(sp: &SpeciesData, pal: &Palette) -> Vec<(char, Rgb)> {
    sp.palette
        .iter()
        .map(|(ch, key)| {
            let col = if let Some(hex) = key.strip_prefix('#') {
                crate::palette::hex_to_rgb(hex)
            } else {
                get(pal, key)
            };
            (ch.chars().next().unwrap(), col)
        })
        .collect()
}

fn color_of(colors: &[(char, Rgb)], ch: char) -> Option<Rgb> {
    colors.iter().find(|(c, _)| *c == ch).map(|(_, col)| *col)
}

/// Contorno de los sprites: 1 px oscuro que los despega del fondo.
const OUTLINE: Rgb = [0x16, 0x16, 0x1c];

fn draw_matrix(
    fb: &mut Framebuffer,
    rows: &[&str],
    colors: &[(char, Rgb)],
    left: i32,
    top: i32,
    flip: bool,
) {
    let h = rows.len() as i32;
    let w = rows
        .iter()
        .map(|row| row.chars().count())
        .max()
        .unwrap_or(0) as i32;
    if w == 0 || h == 0 {
        return;
    }
    let cell = |rx: i32, ry: i32| -> Option<Rgb> {
        if rx < 0 || ry < 0 || ry >= h {
            return None;
        }
        let ch = rows[ry as usize].chars().nth(rx as usize)?;
        if ch == '.' || ch == ' ' {
            None
        } else {
            color_of(colors, ch)
        }
    };
    // Primero el contorno: celdas vacías con **dos o más** vecinos ocupados, una
    // celda más allá del borde. Así se contornea el bulto pero no se engordan los
    // apéndices de 1 px (cuellos, patas, colas).
    for ry in -1..=h {
        for rx in -1..=w {
            if cell(rx, ry).is_some() {
                continue;
            }
            let n = (cell(rx - 1, ry).is_some() as u8)
                + (cell(rx + 1, ry).is_some() as u8)
                + (cell(rx, ry - 1).is_some() as u8)
                + (cell(rx, ry + 1).is_some() as u8);
            if n < 2 {
                continue;
            }
            let dx = if flip { w - 1 - rx } else { rx };
            fb.fill_rect(left + dx, top + ry, 1, 1, OUTLINE, 1.0);
        }
    }
    // Luego el sprite por encima.
    for (ry, row) in rows.iter().enumerate() {
        for (rx, ch) in row.chars().enumerate() {
            if ch == '.' || ch == ' ' {
                continue;
            }
            let Some(col) = color_of(colors, ch) else {
                continue;
            };
            let dx = if flip { w - 1 - rx as i32 } else { rx as i32 };
            fb.fill_rect(left + dx, top + ry as i32, 1, 1, col, 1.0);
        }
    }
}

/// Dibuja un frame aislado de una especie (galerías de diagnóstico). Devuelve
/// `false` si la especie no existe en el registro.
pub fn draw_species_preview(
    fb: &mut Framebuffer,
    name: &str,
    left: i32,
    top: i32,
    frame: usize,
    flip: bool,
    pal: &Palette,
) -> bool {
    let Some(sp) = species(name) else {
        return false;
    };
    let colors = sprite_colors(sp, pal);
    let n = sp.frames.len().max(1);
    let rows = sp.frames.get(frame % n).copied().unwrap_or(sp.frames[0]);
    draw_matrix(fb, rows, &colors, left, top, flip);
    true
}

/// Tamaño (ancho, alto) de la matriz base de una especie, en píxeles.
pub fn sprite_size(name: &str) -> Option<(i32, i32)> {
    let sp = species(name)?;
    let rows = sp.frames.first()?;
    let w = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0) as i32;
    Some((w, rows.len() as i32))
}

pub struct FaunaSpawn {
    pub wx: f64,
    pub sx: f64,
    pub gy: Option<f64>,
    pub flight_y: Option<f64>,
    pub type_: &'static str,
    pub phase: f64,
    pub dir: f64,
}

pub fn centro_fauna_pool(layer_name: &str) -> Vec<(&'static str, f64)> {
    let mut out = Vec::new();
    for sp in SPECIES {
        for &(biome, layer, w) in sp.zones {
            if biome == "centro" && layer == layer_name {
                out.push((sp.name, w));
            }
        }
    }
    out
}

const DEFAULT_FAUNA_POOL: [(&str, f64); 1] = [("condor", 1.0)];

#[allow(clippy::too_many_arguments)]
pub fn fauna_spawns<F, C>(
    layer: &Layer,
    terrain: &Terrain,
    layer_idx: usize,
    camera_x: f64,
    w: usize,
    h: usize,
    seed: u32,
    hour: f64,
    pool_at: F,
    chance_at: C,
) -> Vec<FaunaSpawn>
where
    F: Fn(f64) -> Vec<(&'static str, f64)>,
    C: Fn(f64) -> f64,
{
    let Some(f) = layer.fauna else {
        return Vec::new();
    };
    let p = layer.parallax;
    let chunk_w = f.chunk_w;
    let chunk_seed = seed
        .wrapping_add(layer.seed.wrapping_mul(13))
        .wrapping_add(40503);
    let start_c = ((camera_x * p - 120.0) / chunk_w).floor() as i64;
    let end_c = ((camera_x * p + w as f64 + 120.0) / chunk_w).floor() as i64;
    let mut out = Vec::new();

    for c in start_c..=end_c {
        let wx0 = c as f64 * chunk_w;
        let chance_mul = chance_at(wx0);
        let mut rng = Mulberry32::new(hash_int(c as i32, chunk_seed));
        if rng.next_f64() > f.chance * chance_mul {
            continue;
        }
        let pool = pool_at(wx0);
        let list = if !pool.is_empty() {
            pool
        } else {
            centro_fauna_pool(layer.name)
        };
        let mut generic: Vec<(&str, f64)> = if !list.is_empty() {
            list
        } else {
            DEFAULT_FAUNA_POOL.to_vec()
        };
        generic.retain(|(t, _)| species(t).map(|s| s.chunk_ok).unwrap_or(false));
        if generic.is_empty() {
            continue;
        }
        let type_ = pick_weighted(&generic, &mut rng);
        let Some(sp) = species(type_) else { continue };

        let wx = (c as f64 + rng.next_f64()) * chunk_w;
        let sx = wx - camera_x * p;
        let phase = rng.next_f64() * std::f64::consts::PI * 2.0;
        let dir = if rng.next_f64() < 0.5 { -1.0 } else { 1.0 };
        if sx < -60.0 || sx > w as f64 + 60.0 {
            continue;
        }

        let mut gy = None;
        let mut flight_y = None;
        if sp.movement == "fly" || sp.movement == "flock" {
            flight_y = Some(
                layer.base_y
                    - layer.amp * 0.55
                    - hash1(c as i32, chunk_seed.wrapping_add(77)) * layer.amp * 0.22,
            );
        } else {
            if terrain.river_influence(layer_idx, wx) > 0.25 {
                continue;
            }
            let g = terrain.bank_height(layer_idx, wx);
            if g > h as f64 + 4.0 {
                continue;
            }
            gy = Some(g);
        }
        if !is_active(sp, hour) {
            continue;
        }
        out.push(FaunaSpawn {
            wx,
            sx,
            gy,
            flight_y,
            type_,
            phase,
            dir,
        });
    }

    if let Some(rivers) = layer.rivers {
        for sp in SPECIES {
            let Some((chance, offset, rs_seed)) = sp.river else {
                continue;
            };
            for xc in terrain.river_event_centers(layer_idx, camera_x, w) {
                let mut rng = Mulberry32::new(hash_int(
                    xc.floor() as i32,
                    chunk_seed.wrapping_add(rs_seed),
                ));
                if rng.next_f64() > chance {
                    continue;
                }
                let side = if rng.next_f64() < 0.5 { -1.0 } else { 1.0 };
                let wx = xc + side * rivers.width * offset;
                let sx = wx - camera_x * p;
                if sx < -60.0 || sx > w as f64 + 60.0 {
                    continue;
                }
                let pool = pool_at(wx);
                let list = if !pool.is_empty() {
                    pool
                } else {
                    centro_fauna_pool(layer.name)
                };
                if !list.iter().any(|(t, _)| *t == sp.name) {
                    continue;
                }
                if !is_active(sp, hour) {
                    continue;
                }
                let gy = terrain.bank_height(layer_idx, wx);
                if gy > h as f64 + 4.0 {
                    continue;
                }
                let phase = rng.next_f64() * std::f64::consts::PI * 2.0;
                let dir = if rng.next_f64() < 0.5 { -1.0 } else { 1.0 };
                out.push(FaunaSpawn {
                    wx,
                    sx,
                    gy: Some(gy),
                    flight_y: None,
                    type_: sp.name,
                    phase,
                    dir,
                });
            }
        }
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn draw_tracks(
    fb: &mut Framebuffer,
    layer: &Layer,
    pal: &Palette,
    camera_x: f64,
    w: usize,
    s: &FaunaSpawn,
    t_sec: f64,
    terrain: &Terrain,
    layer_idx: usize,
) {
    let sp = species(s.type_).unwrap();
    let p = layer.parallax;
    let osc = (t_sec * sp.speed + s.phase).sin();
    let vel = (t_sec * sp.speed + s.phase).cos();
    let dir = if vel >= 0.0 { 1.0 } else { -1.0 };
    let wx = s.wx + osc * sp.range;
    let col = shade(get(pal, "sandD"), -0.3);
    for i in 1..=3 {
        let bx = wx - dir * i as f64 * 3.0;
        let sx = r(bx - camera_x * p);
        if sx < 0 || sx >= w as i32 {
            continue;
        }
        let by = r(terrain.bank_height(layer_idx, bx));
        let a = 0.4 * (1.0 - i as f64 / 4.0);
        fb.fill_rect(sx, by - 1, 1, 1, col, a);
        fb.fill_rect(sx + if i % 2 != 0 { 1 } else { -1 }, by - 1, 1, 1, col, a);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_fauna(
    fb: &mut Framebuffer,
    layer: &Layer,
    pal: &Palette,
    camera_x: f64,
    s: &FaunaSpawn,
    t_sec: f64,
    terrain: &Terrain,
    layer_idx: usize,
) {
    let sp = species(s.type_).unwrap();
    let p = layer.parallax;
    let osc = (t_sec * sp.speed + s.phase).sin();
    let wx = s.wx + osc * sp.range;
    let left = r(wx - camera_x * p);
    let colors = sprite_colors(sp, pal);
    let base = sp.frames[0];
    let w = base
        .iter()
        .map(|row| row.chars().count())
        .max()
        .unwrap_or(0) as f64;
    let h = base.len() as f64;

    let top = if sp.movement == "fly" || sp.movement == "flock" {
        r(
            s.flight_y.unwrap_or(0.0) + (t_sec * sp.speed * 0.7 + s.phase * 1.7).sin() * 3.0
                - h / 2.0,
        )
    } else {
        let mut ground = terrain.bank_height(layer_idx, wx);
        if sp.movement == "hop" {
            ground -= (t_sec * sp.speed * 2.0 + s.phase).sin().abs() * 2.0;
        }
        if sp.movement == "swim" {
            ground += (t_sec * sp.speed * 1.3 + s.phase).sin() * 1.5;
        }
        r(ground - h)
    };

    if sp.tracks && sp.movement == "walk" {
        draw_tracks(fb, layer, pal, camera_x, fb.w, s, t_sec, terrain, layer_idx);
    }

    let n_frames = sp.frames.len() as f64;
    let frame_pos = (t_sec * sp.fps + s.phase / (std::f64::consts::PI * 2.0)).floor();
    let fi = (((frame_pos % n_frames) + n_frames) % n_frames) as usize;
    let rows = sp.frames.get(fi).copied().unwrap_or(sp.frames[0]);
    let lx = r(left as f64 - w / 2.0);
    draw_matrix(fb, rows, &colors, lx, top, s.dir < 0.0);

    if sp.movement == "flock" {
        for i in 1..=2 {
            let dx = r(i as f64 * 7.0 * s.dir);
            let dy = r((t_sec * sp.speed + s.phase + i as f64 * 1.3).sin() * 1.5) - i * 2;
            draw_matrix(fb, rows, &colors, lx + dx, top + dy, s.dir < 0.0);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn place_fauna(
    fb: &mut Framebuffer,
    layer: &Layer,
    terrain: &Terrain,
    layer_idx: usize,
    pal: &Palette,
    camera_x: f64,
    w: usize,
    h: usize,
    seed: u32,
    hour: f64,
    t_sec: f64,
    biome_mode: &str,
    bloom_mode: &str,
    activity_mul: f64,
) {
    if layer.fauna.is_none() {
        return;
    }
    let pool_at = |wx: f64| -> Vec<(&'static str, f64)> {
        let (weights, bloom) = crate::biomes::biome_weights_bloom(wx, seed, biome_mode);
        let bloom = resolve_bloom(bloom_mode, &weights, bloom);
        biome_fauna_pool(layer.name, &weights, bloom)
    };
    let chance_at = |wx: f64| -> f64 {
        let (weights, bloom) = crate::biomes::biome_weights_bloom(wx, seed, biome_mode);
        let bloom = resolve_bloom(bloom_mode, &weights, bloom);
        bloom_chance_mul(bloom) * activity_mul
    };
    for s in fauna_spawns(
        layer, terrain, layer_idx, camera_x, w, h, seed, hour, pool_at, chance_at,
    ) {
        draw_fauna(fb, layer, pal, camera_x, &s, t_sec, terrain, layer_idx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::get_palette;

    const SEED: u32 = 662118718;

    #[test]
    fn activity_bands() {
        let condor = species("condor").unwrap();
        let guina = species("guina").unwrap();
        let pudu = species("pudu").unwrap();
        assert!(is_active(condor, 12.0));
        assert!(!is_active(condor, 2.0));
        assert!(is_active(guina, 23.0));
        assert!(!is_active(guina, 12.0));
        assert!(is_active(pudu, 18.0));
        assert!(is_active(pudu, 8.0));
        assert!(!is_active(pudu, 12.0));
    }

    #[test]
    fn spawns_are_deterministic() {
        let terrain = Terrain::new(SEED);
        let layer = &crate::terrain::LAYERS[2];
        let a = fauna_spawns(
            layer,
            &terrain,
            2,
            300.0,
            480,
            270,
            SEED,
            12.0,
            |_| Vec::new(),
            |_| 1.0,
        );
        let b = fauna_spawns(
            layer,
            &terrain,
            2,
            300.0,
            480,
            270,
            SEED,
            12.0,
            |_| Vec::new(),
            |_| 1.0,
        );
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(b.iter()) {
            assert_eq!(x.type_, y.type_);
        }
    }

    #[test]
    fn place_fauna_draws() {
        let terrain = Terrain::new(SEED);
        let pal = get_palette(12.0, "clear", 1.0);
        let mut fb = Framebuffer::new(480, 270);
        for idx in 0..6 {
            place_fauna(
                &mut fb,
                &crate::terrain::LAYERS[idx],
                &terrain,
                idx,
                &pal,
                300.0,
                480,
                270,
                SEED,
                12.0,
                0.0,
                "auto",
                "auto",
                1.0,
            );
        }
        // no exige dibujo (la fauna es rara), solo que no falle.
        let _ = fb.px;
    }
}
