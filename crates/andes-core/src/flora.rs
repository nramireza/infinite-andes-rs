//! Flora andina en pixel art (puerto de `src/flora.js`): registro, colocación
//! determinista por chunk y dibujo de cada especie.

use crate::biomes::{biome_flora_pool, resolve_bloom};
use crate::framebuffer::Framebuffer;
use crate::math::js_round;
use crate::palette::{get, lerp_color, shade, Palette, Rgb};
use crate::rng::{hash1, hash_int, Mulberry32};
use crate::terrain::{Layer, Terrain};

const FLOWER_COLORS: [Rgb; 4] = [
    [0xe0, 0x5a, 0x9a],
    [0xf2, 0xc1, 0x4e],
    [0xf4, 0xf0, 0xe6],
    [0x9a, 0x6a, 0xd0],
];

/// Entrada del registro de flora. `zones` son `(bioma, capa, peso)`.
pub struct FloraEntry {
    pub name: &'static str,
    pub kind: &'static str,
    pub height: f64,
    pub bloom_only: bool,
    pub zones: &'static [(&'static str, &'static str, f64)],
}

macro_rules! zone {
    ($($b:literal => $($l:literal : $w:expr),+);* $(;)?) => {{
        &[$( $( ($b, $l, $w as f64) ),+ ),*]
    }};
}

pub const FLORA: &[FloraEntry] = &[
    FloraEntry {
        name: "araucaria",
        kind: "especie",
        height: 1.9,
        bloom_only: false,
        zones: zone! { "centro" => "precordillera":1, "valle":1, "costa":1; "sur" => "precordillera":1, "valle":1, "costa":1 },
    },
    FloraEntry {
        name: "lenga",
        kind: "especie",
        height: 1.3,
        bloom_only: false,
        zones: zone! { "centro" => "costa":2; "sur" => "precordillera":1, "valle":1, "costa":1; "patagonia" => "precordillera":1, "valle":1, "costa":1; "austral" => "valle":1 },
    },
    FloraEntry {
        name: "copihue",
        kind: "especie",
        height: 1.1,
        bloom_only: false,
        zones: zone! { "centro" => "costa":1; "sur" => "valle":1, "costa":1 },
    },
    FloraEntry {
        name: "cactus",
        kind: "especie",
        height: 1.2,
        bloom_only: false,
        zones: zone! { "altiplano" => "precordillera":1, "valle":1, "costa":1; "norte" => "precordillera":1, "valle":1, "costa":1 },
    },
    FloraEntry {
        name: "alerce",
        kind: "especie",
        height: 1.9,
        bloom_only: false,
        zones: zone! { "sur" => "precordillera":1, "costa":1; "austral" => "precordillera":1, "costa":1 },
    },
    FloraEntry {
        name: "nalca",
        kind: "especie",
        height: 0.9,
        bloom_only: false,
        zones: zone! { "sur" => "valle":1, "costa":1; "austral" => "valle":1, "costa":1 },
    },
    FloraEntry {
        name: "colihue",
        kind: "especie",
        height: 1.1,
        bloom_only: false,
        zones: zone! { "sur" => "valle":1, "costa":1; "patagonia" => "valle":1, "costa":1; "austral" => "valle":1, "costa":1 },
    },
    FloraEntry {
        name: "palma",
        kind: "especie",
        height: 1.4,
        bloom_only: false,
        zones: zone! { "centro" => "valle":1, "costa":1 },
    },
    FloraEntry {
        name: "coihue",
        kind: "especie",
        height: 1.7,
        bloom_only: false,
        zones: zone! { "centro" => "valle":1, "costa":1; "sur" => "precordillera":1, "valle":1, "costa":1; "patagonia" => "precordillera":1, "costa":1; "austral" => "precordillera":1, "valle":1, "costa":1 },
    },
    FloraEntry {
        name: "roble",
        kind: "especie",
        height: 1.5,
        bloom_only: false,
        zones: zone! { "centro" => "valle":1, "costa":1; "sur" => "valle":1, "costa":1 },
    },
    FloraEntry {
        name: "michay",
        kind: "especie",
        height: 0.7,
        bloom_only: false,
        zones: zone! { "centro" => "valle":1, "costa":1; "sur" => "precordillera":1, "valle":1, "costa":1; "patagonia" => "precordillera":1, "valle":1, "costa":1 },
    },
    FloraEntry {
        name: "chaura",
        kind: "especie",
        height: 0.6,
        bloom_only: false,
        zones: zone! { "centro" => "costa":1; "sur" => "costa":1; "patagonia" => "valle":1, "costa":1; "austral" => "costa":1 },
    },
    FloraEntry {
        name: "quillay",
        kind: "especie",
        height: 1.0,
        bloom_only: false,
        zones: zone! { "centro" => "precordillera":1, "valle":1, "costa":1; "sur" => "valle":1, "costa":1 },
    },
    FloraEntry {
        name: "manio",
        kind: "especie",
        height: 1.4,
        bloom_only: false,
        zones: zone! { "sur" => "valle":1, "costa":1; "patagonia" => "valle":1, "costa":1; "austral" => "precordillera":1, "valle":1, "costa":1 },
    },
    FloraEntry {
        name: "canelo",
        kind: "especie",
        height: 1.3,
        bloom_only: false,
        zones: zone! { "sur" => "valle":1, "costa":1; "patagonia" => "costa":1; "austral" => "precordillera":1, "valle":1, "costa":1 },
    },
    FloraEntry {
        name: "arrayan",
        kind: "especie",
        height: 1.0,
        bloom_only: false,
        zones: zone! { "sur" => "valle":1, "costa":1; "patagonia" => "valle":1, "costa":1; "austral" => "valle":1, "costa":1 },
    },
    FloraEntry {
        name: "notro",
        kind: "especie",
        height: 0.9,
        bloom_only: false,
        zones: zone! { "sur" => "valle":1, "costa":1; "patagonia" => "valle":1, "costa":1; "austral" => "valle":1, "costa":1 },
    },
    FloraEntry {
        name: "bush",
        kind: "generico",
        height: 0.7,
        bloom_only: false,
        zones: zone! {
            "altiplano" => "precordillera":1, "valle":1, "costa":1;
            "norte" => "precordillera":1, "valle":1, "costa":1;
            "centro" => "valle":1, "costa":1;
            "sur" => "precordillera":1, "valle":1, "costa":1;
            "patagonia" => "precordillera":1, "valle":1, "costa":1;
            "austral" => "precordillera":1, "valle":1, "costa":1
        },
    },
    FloraEntry {
        name: "crop",
        kind: "generico",
        height: 0.8,
        bloom_only: false,
        zones: zone! { "norte" => "valle":1; "centro" => "valle":2 },
    },
    FloraEntry {
        name: "grass",
        kind: "generico",
        height: 0.5,
        bloom_only: false,
        zones: zone! {
            "altiplano" => "precordillera":1, "valle":2, "costa":1, "playa":1;
            "norte" => "valle":1, "playa":1;
            "centro" => "valle":1, "playa":2;
            "sur" => "playa":1;
            "patagonia" => "valle":1, "playa":1;
            "austral" => "playa":1
        },
    },
    FloraEntry {
        name: "rock",
        kind: "generico",
        height: 0.7,
        bloom_only: false,
        zones: zone! {
            "altiplano" => "precordillera":1, "valle":1, "costa":1, "playa":1;
            "norte" => "precordillera":1, "valle":1, "costa":1, "playa":1;
            "centro" => "playa":1;
            "sur" => "playa":1;
            "patagonia" => "precordillera":1, "playa":1;
            "austral" => "playa":1
        },
    },
    FloraEntry {
        name: "espino",
        kind: "especie",
        height: 0.8,
        bloom_only: false,
        zones: zone! { "norte" => "valle":1, "costa":1; "centro" => "valle":2, "costa":2 },
    },
    FloraEntry {
        name: "litre",
        kind: "especie",
        height: 0.75,
        bloom_only: false,
        zones: zone! { "centro" => "valle":2, "costa":2 },
    },
    FloraEntry {
        name: "boldo",
        kind: "especie",
        height: 0.85,
        bloom_only: false,
        zones: zone! { "centro" => "precordillera":1, "costa":2 },
    },
    FloraEntry {
        name: "peumo",
        kind: "especie",
        height: 1.0,
        bloom_only: false,
        zones: zone! { "centro" => "valle":1, "costa":1; "sur" => "valle":1, "costa":1 },
    },
    FloraEntry {
        name: "maiten",
        kind: "especie",
        height: 1.1,
        bloom_only: false,
        zones: zone! { "centro" => "valle":1; "sur" => "valle":1 },
    },
    FloraEntry {
        name: "cipres",
        kind: "especie",
        height: 1.6,
        bloom_only: false,
        zones: zone! { "centro" => "precordillera":1; "sur" => "precordillera":1 },
    },
    FloraEntry {
        name: "puya",
        kind: "especie",
        height: 0.9,
        bloom_only: false,
        zones: zone! { "altiplano" => "precordillera":2, "costa":1; "norte" => "precordillera":1, "costa":1 },
    },
    FloraEntry {
        name: "llareta",
        kind: "especie",
        height: 0.4,
        bloom_only: false,
        zones: zone! { "altiplano" => "precordillera":2 },
    },
    FloraEntry {
        name: "amancay",
        kind: "especie",
        height: 0.6,
        bloom_only: false,
        zones: zone! { "sur" => "valle":1, "costa":1; "patagonia" => "valle":1 },
    },
    FloraEntry {
        name: "junquillo",
        kind: "especie",
        height: 0.7,
        bloom_only: false,
        zones: zone! { "centro" => "valle":1, "costa":1; "sur" => "valle":1, "costa":1; "patagonia" => "valle":1, "costa":1; "austral" => "valle":1, "costa":1 },
    },
    FloraEntry {
        name: "flower",
        kind: "efecto",
        height: 1.0,
        bloom_only: true,
        zones: &[],
    },
];

pub fn flora_entry(name: &str) -> Option<&'static FloraEntry> {
    FLORA.iter().find(|e| e.name == name)
}

fn height_for(name: &str) -> f64 {
    flora_entry(name).map(|e| e.height).unwrap_or(1.0)
}

#[inline]
fn r(x: f64) -> i32 {
    js_round(x) as i32
}

fn dr(fb: &mut Framebuffer, x: i32, y: i32, w: i32, h: i32, c: Rgb) {
    fb.fill_rect(x, y, w, h, c, 1.0);
}

/// Línea de 1 px (Bresenham) para troncos, ramas y tallos.
fn line(fb: &mut Framebuffer, x0: i32, y0: i32, x1: i32, y1: i32, c: Rgb) {
    let (mut x0, mut y0) = (x0, y0);
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        dr(fb, x0, y0, 1, 1, c);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            if x0 == x1 {
                break;
            }
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            if y0 == y1 {
                break;
            }
            err += dx;
            y0 += sy;
        }
    }
}

/// Lado iluminado: `true` si el sol está a la izquierda (o cenit/móvil).
#[inline]
fn lit_left(light: f64) -> bool {
    light <= 0.0
}

/// Trío de tonos de copa (luz, base, sombra) para el lado iluminado `light`.
#[inline]
fn crown_tone(pal: &Palette, light: f64) -> (Rgb, Rgb, Rgb) {
    let l = get(pal, "floraL");
    let d = get(pal, "floraD");
    let sun = get(pal, "sun");
    let k = light.abs().min(1.0);
    (lerp_color(l, sun, 0.20 * k), d, shade(d, -0.24))
}

/// Copa redondeada e irregular con sombra direccional (3 tonos), por filas.
#[allow(clippy::too_many_arguments)]
fn crown_blob(
    fb: &mut Framebuffer,
    cx: i32,
    cy: i32,
    rx: i32,
    ry: i32,
    seed: u32,
    tone: (Rgb, Rgb, Rgb),
    left_lit: bool,
) {
    if rx < 1 || ry < 1 {
        return;
    }
    let (hi, mid, sh) = tone;
    for dy in -ry..=ry {
        let t = dy as f64 / ry as f64;
        let mut span = (rx as f64 * (1.0 - t * t).max(0.0).sqrt()).round() as i32;
        if hash1(
            seed.wrapping_add((dy as u32).wrapping_mul(2_654_435_761)) as i32,
            0x51a,
        ) < 0.30
        {
            span -= 1;
        }
        if span < 1 {
            continue;
        }
        let yy = cy + dy;
        dr(fb, cx - span, yy, span * 2 + 1, 1, mid);
        if dy * 2 > ry {
            dr(fb, cx - span, yy, span * 2 + 1, 1, sh);
        } else if left_lit {
            dr(fb, cx + 1, yy, span, 1, sh);
        } else {
            dr(fb, cx - span, yy, span, 1, sh);
        }
        if dy * 3 < -ry {
            dr(fb, cx - span + 1, yy, (span * 2 - 1).max(1), 1, hi);
        } else if dy <= 0 {
            if left_lit {
                dr(fb, cx - span + 1, yy, span, 1, hi);
            } else {
                dr(fb, cx + 1, yy, span, 1, hi);
            }
        }
    }
}

/// Copa cónica (coníferas y portes piramidales), de `top` hacia abajo.
#[allow(clippy::too_many_arguments)]
fn crown_cone(
    fb: &mut Framebuffer,
    cx: i32,
    top: i32,
    h: i32,
    half_max: i32,
    seed: u32,
    tone: (Rgb, Rgb, Rgb),
    left_lit: bool,
    concave: bool,
) {
    if h < 1 || half_max < 1 {
        return;
    }
    let (hi, mid, sh) = tone;
    for i in 0..h {
        let t = (i + 1) as f64 / h as f64;
        let prof = if concave { t.powf(1.35) } else { t.powf(0.82) };
        let mut half = (half_max as f64 * prof).round().max(1.0) as i32;
        if hash1(seed.wrapping_add(i as u32 * 7) as i32, 0x2c1) < 0.30 {
            half = (half - 1).max(1);
        }
        let yy = top + i;
        dr(fb, cx - half, yy, half * 2 + 1, 1, mid);
        if left_lit {
            dr(fb, cx + half / 2, yy, half - half / 2, 1, sh);
        } else {
            dr(fb, cx - half, yy, half - half / 2, 1, sh);
        }
        if i % 3 == 0 {
            if left_lit {
                dr(fb, cx - half + 1, yy, half, 1, hi);
            } else {
                dr(fb, cx + 1, yy, half, 1, hi);
            }
        }
    }
}

/// Fronda arqueada de palma: nace con ángulo `theta` que crece, así cae en la punta.
#[allow(clippy::too_many_arguments)]
fn frond(fb: &mut Framebuffer, x: i32, top: i32, theta: f64, len: i32, mid: Rgb, hi: Rgb, sh: Rgb) {
    for k in 1..=len {
        let t = k as f64 / len.max(1) as f64;
        let ang = theta + t * 0.9;
        let fx = x + r(ang.sin() * k as f64);
        let fy = top - r(ang.cos() * k as f64);
        let c = if k % 3 == 0 { hi } else { mid };
        dr(fb, fx, fy, 1, 1, c);
        if k % 2 == 0 {
            dr(fb, fx + if theta < 0.0 { 1 } else { -1 }, fy, 1, 1, sh);
        }
    }
}

/// Roseta de hojas cortas y rígidas (puya y bultos del cojín de llareta).
fn rosette(fb: &mut Framebuffer, cx: i32, cy: i32, radius: i32, hi: Rgb, mid: Rgb, sh: Rgb) {
    if radius < 1 {
        return;
    }
    let dirs = [
        (0.0f64, -1.0f64),
        (0.6, -0.9),
        (-0.6, -0.9),
        (1.0, -0.4),
        (-1.0, -0.4),
        (1.0, 0.3),
        (-1.0, 0.3),
        (0.7, 0.7),
        (-0.7, 0.7),
        (0.0, 0.9),
    ];
    for (i, (ux, uy)) in dirs.iter().enumerate() {
        let len = (radius as f64 * (0.8 + (i % 3) as f64 * 0.15))
            .round()
            .max(1.0) as i32;
        for k in 1..=len {
            let c = if k == len {
                hi
            } else if k * 2 >= len {
                mid
            } else {
                sh
            };
            dr(fb, cx + r(ux * k as f64), cy + r(uy * k as f64), 1, 1, c);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw_plant(
    fb: &mut Framebuffer,
    x: f64,
    base_y: f64,
    size: f64,
    type_: &str,
    pal: &Palette,
    sway: f64,
    warm: bool,
    season: i32,
    night: f64,
    light: f64,
) {
    match type_ {
        "araucaria" => draw_araucaria(fb, x, base_y, size, pal, sway, light),
        "lenga" => draw_lenga(fb, x, base_y, size, pal, warm, season, light),
        "bush" => draw_bush(fb, x, base_y, size, pal, light),
        "grass" => draw_grass(fb, x, base_y, size, pal, light),
        "crop" => draw_crop(fb, x, base_y, size, pal, light),
        "rock" => draw_rock(fb, x, base_y, size, pal, light),
        "flower" => draw_flower(fb, x, base_y, size, pal, sway, night),
        "cactus" => draw_cactus(fb, x, base_y, size, pal, light),
        "alerce" => draw_alerce(fb, x, base_y, size, pal, light),
        "nalca" => draw_nalca(fb, x, base_y, size, pal, light),
        "colihue" => draw_colihue(fb, x, base_y, size, pal, light),
        "palma" => draw_palma(fb, x, base_y, size, pal, light),
        "coihue" => draw_coihue(fb, x, base_y, size, pal, light),
        "roble" => draw_roble(fb, x, base_y, size, pal, season, light),
        "copihue" => draw_copihue(fb, x, base_y, size, pal, season, light),
        "michay" => draw_michay(fb, x, base_y, size, pal, season, light),
        "chaura" => draw_chaura(fb, x, base_y, size, pal, light),
        "quillay" => draw_quillay(fb, x, base_y, size, pal, season, light),
        "manio" => draw_manio(fb, x, base_y, size, pal, light),
        "canelo" => draw_canelo(fb, x, base_y, size, pal, season, light),
        "arrayan" => draw_arrayan(fb, x, base_y, size, pal, season, light),
        "notro" => draw_notro(fb, x, base_y, size, pal, season, light),
        "espino" => draw_espino(fb, x, base_y, size, pal, season, light),
        "litre" => draw_litre(fb, x, base_y, size, pal, light),
        "boldo" => draw_boldo(fb, x, base_y, size, pal, light),
        "peumo" => draw_peumo(fb, x, base_y, size, pal, season, light),
        "maiten" => draw_maiten(fb, x, base_y, size, pal, light),
        "cipres" => draw_cipres(fb, x, base_y, size, pal, light),
        "puya" => draw_puya(fb, x, base_y, size, pal, light),
        "llareta" => draw_llareta(fb, x, base_y, size, pal, light),
        "amancay" => draw_amancay(fb, x, base_y, size, pal, sway, light),
        "junquillo" => draw_junquillo(fb, x, base_y, size, pal, light),
        _ => draw_grass(fb, x, base_y, size, pal, light),
    }
}

/// Araucaria: tronco recto y grueso que ramifica a varios metros del suelo y copa
/// en **pisos/verticilos** de ramas horizontales (pagoda), base ancha y ápice
/// redondeado (ver `porte` en `flora_data.rs`).
fn draw_araucaria(
    fb: &mut Framebuffer,
    x: f64,
    base_y: f64,
    size: f64,
    pal: &Palette,
    sway: f64,
    light: f64,
) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 1.05).round().max(9.0) as i32;
    let top = base_y - h;
    let cx = x + r(sway * 0.5);
    let trunk_w = (size * 0.10).round().max(1.0) as i32;
    let trunk = get(pal, "trunk");
    let td = shade(trunk, -0.25);
    let (hi, mid, sh) = crown_tone(pal, light);
    let lit = lit_left(light);

    // Tronco recto y grueso, visible hasta bien entrada la copa.
    dr(fb, cx - (trunk_w >> 1), top, trunk_w, h, trunk);
    dr(fb, cx + (trunk_w >> 1) - 1, top, 1, h, td);

    // Copa en pisos apilados: base ancha, ápice redondeado.
    let apex = top + (h as f64 * 0.04) as i32;
    let crown_bottom = base_y - (h as f64 * 0.4) as i32;
    let tiers = ((h as f64 * 0.16).round() as i32).clamp(3, 6);
    let base_half = (size * 0.5).round().max(3.0) as i32;
    let span = (crown_bottom - apex).max(1);

    for t in 0..tiers {
        let f = t as f64 / (tiers - 1).max(1) as f64; // 0 = ápice, 1 = base
        let yy = apex + r(f * span as f64);
        let half = (base_half as f64 * (0.32 + 0.68 * f)).round().max(1.0) as i32;
        // Banda del piso (ramas), más ancha abajo; el tronco asoma entre pisos.
        dr(fb, cx - half, yy, half * 2 + 1, 1, mid);
        dr(fb, cx - half + 1, yy + 1, (half * 2 - 1).max(1), 1, sh);
        // Luz sobre el piso, en el lado iluminado.
        if lit {
            dr(fb, cx - half + 1, yy - 1, half, 1, hi);
        } else {
            dr(fb, cx + 1, yy - 1, half, 1, hi);
        }
        // Puntas que caen (hábito araucaria).
        let droop = 1 + t;
        for &sgn in &[-1i32, 1] {
            dr(fb, cx + sgn * half, yy + 1, 1, droop, mid);
            dr(fb, cx + sgn * (half - 1), yy + 1, 1, droop.min(2), sh);
        }
        // Textura espinosa dispersa.
        for k in 1..half {
            if hash1(cx * 7 + yy * 13 + k, 0xa7a) < 0.22 {
                dr(fb, cx - k, yy, 1, 1, sh);
                dr(fb, cx + k, yy + 1, 1, 1, sh);
            }
        }
    }
    // Ápice redondeado.
    dr(fb, cx - 1, apex - 1, 3, 2, mid);
    dr(fb, cx, apex - 2, 1, 1, hi);

    // Conos femeninos redondos en las puntas bajas (detalle).
    if size >= 10.0 {
        for &sgn in &[-1i32, 1] {
            let by = crown_bottom - 1;
            dr(fb, cx + sgn * (base_half - 1), by, 2, 2, get(pal, "sandD"));
        }
    }
}

fn draw_bare_branches(fb: &mut Framebuffer, x: i32, top: i32, h: i32, pal: &Palette) {
    let trunk = get(pal, "trunk");
    for (dir, len, wave) in [(-1i32, 0.8f64, 0.5f64), (0, 1.0, 0.0), (1, 0.7, -0.5)] {
        let steps = ((h as f64 * len).round() as i32).max(2);
        let mut bx = x;
        for i in 0..steps {
            let yy = top + steps - i;
            bx = x + r(dir as f64 * (i as f64 * 0.35 + (i % 2) as f64 * wave));
            dr(fb, bx, yy, 1, 1, trunk);
        }
        if dir != 0 && steps > 3 {
            dr(fb, bx + dir, top + r((steps as f64) * 0.4), 1, 1, trunk);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_lenga(
    fb: &mut Framebuffer,
    x: f64,
    base_y: f64,
    size: f64,
    pal: &Palette,
    _warm: bool,
    season: i32,
    light: f64,
) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 1.0).round().max(6.0) as i32;
    let trunk_h = (size * 0.32).round().max(2.0) as i32;
    let top = base_y - h;
    let trunk = get(pal, "trunk");
    dr(fb, x - 1, base_y - trunk_h, 2, trunk_h, trunk);
    let crown_h = h - trunk_h;
    if season == 2 {
        draw_bare_branches(fb, x, top, crown_h, pal);
        return;
    }
    let (hi, mid, sh) = crown_tone(pal, light);
    let lit = lit_left(light);
    let half_max = (size * 0.46).round().max(2.0) as i32;

    // Copa piramidal: angosta arriba, ancha abajo (a diferencia del resto).
    for i in 0..crown_h {
        if season == 1 && hash1(x * 7 + i + 0x1e, 0xa7e) < 0.30 {
            continue;
        }
        let t = (i + 1) as f64 / crown_h as f64;
        let mut half = (half_max as f64 * t.powf(0.8)).round().max(1.0) as i32;
        if hash1(x * 7 + i, 0x1e9) < 0.28 {
            half = (half - 1).max(1);
        }
        let yy = top + i;
        dr(fb, x - half, yy, half * 2 + 1, 1, mid);
        if lit {
            dr(fb, x + half / 2, yy, half - half / 2, 1, sh);
        } else {
            dr(fb, x - half, yy, half - half / 2, 1, sh);
        }
        if i % 2 == 0 {
            if lit {
                dr(fb, x - half + 1, yy, half, 1, hi);
            } else {
                dr(fb, x + 1, yy, half, 1, hi);
            }
        }
    }
    dr(fb, x - 1, top - 1, 2, 1, hi);
    if season == 3 {
        dr(fb, x - 2, top, 1, 1, hi);
        dr(fb, x + 1, top + 1, 1, 1, hi);
    }
}

fn draw_bush(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 0.6).round().max(3.0) as i32;
    let rx = (size * 0.42).round().max(2.0) as i32;
    let tone = crown_tone(pal, light);
    crown_blob(
        fb,
        x,
        base_y - h / 2,
        rx,
        h / 2,
        0x0b5,
        tone,
        lit_left(light),
    );
}

fn draw_grass(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let (hi, mid, _sh) = crown_tone(pal, light);
    let lit = lit_left(light);
    let n = 3 + (size * 0.4).floor() as i32;
    for i in 0..n {
        let gx = x + i - (n >> 1);
        let h = (size * 0.5).round().max(2.0) as i32 + (i * 7 + x).rem_euclid(2);
        let c = if (gx < x) == lit { hi } else { mid };
        dr(fb, gx, base_y - h, 1, h, c);
        // Punta inclinada de la brizna.
        dr(
            fb,
            gx + if i % 2 == 0 { 1 } else { -1 },
            base_y - h - 1,
            1,
            1,
            c,
        );
    }
}

fn draw_crop(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let (hi, mid, _sh) = crown_tone(pal, light);
    let w = (size * 0.9).round().max(4.0) as i32;
    let h = (size * 0.3).round().max(1.0) as i32;
    // Franja de tierra.
    dr(fb, x - (w >> 1), base_y - h, w, h, get(pal, "valleyD"));
    // Hileras de espigas con punta clara.
    let mut i = 0;
    while i < w {
        let gh = 1 + (i * 7 + x).rem_euclid(2);
        let c = if i % 4 == 0 { hi } else { mid };
        dr(fb, x - (w >> 1) + i, base_y - h - gh, 1, gh, c);
        i += 2;
    }
}

fn draw_rock(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let rk = get(pal, "rock");
    let rkd = get(pal, "rockD");
    let hi = shade(rk, 0.2);
    let lit = lit_left(light);
    let w = (size * 0.6).round().max(3.0) as i32;
    let h = (size * 0.34).round().max(2.0) as i32;
    let profile = [0.4, 0.7, 0.95, 1.0, 0.85, 0.6];
    for rr in 0..h {
        let f = profile[(rr as usize).min(profile.len() - 1)];
        let half = ((w as f64 / 2.0) * f).round().max(1.0) as i32;
        let yy = base_y - h + rr;
        dr(
            fb,
            x - half,
            yy,
            half * 2,
            1,
            if rr == 0 { rk } else { rkd },
        );
        if rr > 0 && rr + 1 < h {
            if lit {
                dr(fb, x - half + 1, yy, (half - 1).max(1), 1, hi);
            } else {
                dr(fb, x + 1, yy, (half - 1).max(1), 1, hi);
            }
        }
    }
    // Piedritas sueltas al pie.
    dr(fb, x - w / 2 - 2, base_y - 1, 1, 1, rkd);
    dr(fb, x + w / 2 + 1, base_y - 1, 1, 1, rkd);
}

fn draw_cactus(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 1.1).round().max(7.0) as i32;
    let top = base_y - h;
    let w = (size * 0.44).round().max(3.0) as i32;
    let half = w / 2;
    let (hi, mid, sh) = crown_tone(pal, light);
    let lit = lit_left(light);
    let sand_d = get(pal, "sandD");

    // Columna gruesa con costillas: luz a un lado, surco al otro.
    for rr in 0..h {
        let yy = top + rr;
        dr(fb, x - half, yy, w, 1, mid);
        if lit {
            dr(fb, x - half + 1, yy, 1, 1, hi);
            dr(fb, x + half - 1, yy, 1, 1, sh);
        } else {
            dr(fb, x + half - 1, yy, 1, 1, hi);
            dr(fb, x - half + 1, yy, 1, 1, sh);
        }
    }

    // Brazos en candelabro (1-2) que suben doblados.
    let arm_h = (h as f64 * 0.4).round().max(3.0) as i32;
    let dir = if hash1(x, 0xcac) < 0.5 { -1 } else { 1 };
    let arm_y = top + r(h as f64 * 0.5);
    let ax = x + dir * (half + 1);
    dr(fb, x.min(ax), arm_y, (ax - x).abs() + 1, 1, mid);
    for rr in 0..=arm_h {
        let yy = arm_y - rr;
        dr(fb, ax - 1, yy, 2, 1, mid);
        dr(fb, if lit { ax - 1 } else { ax }, yy, 1, 1, hi);
    }
    if hash1(x, 0xca5) < 0.6 {
        let a_y = top + r(h as f64 * 0.68);
        let a2 = x + (-dir) * (half + 1);
        dr(fb, x.min(a2), a_y, (a2 - x).abs() + 1, 1, mid);
        for rr in 0..=(arm_h / 2).max(1) {
            dr(fb, a2 - 1, a_y - rr, 2, 1, mid);
        }
    }

    // Espinas dispersas en los bordes.
    for frac in [0.2, 0.45, 0.7, 0.9] {
        let y = top + r(h as f64 * frac);
        dr(fb, x - half - 1, y, 1, 1, sand_d);
        dr(fb, x + half + 1, y + 1, 1, 1, sand_d);
    }

    // Flor en el ápice (el copao florece cerca de la punta).
    dr(fb, x, top - 1, 1, 1, FLOWER_COLORS[3]);
    dr(fb, x - 1, top - 2, 1, 1, FLOWER_COLORS[0]);
    dr(fb, x + 1, top - 2, 1, 1, FLOWER_COLORS[0]);
    dr(fb, x, top - 2, 1, 1, [0xf7, 0xd9, 0x70]);
}

fn draw_alerce(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 1.05).round().max(8.0) as i32;
    let trunk_h = (size * 0.18).round().max(2.0) as i32;
    let top = base_y - h;
    let trunk = get(pal, "trunk");
    dr(fb, x - 1, base_y - trunk_h, 2, trunk_h, trunk);
    let n = h - trunk_h;
    let tone = crown_tone(pal, light);
    let lit = lit_left(light);
    let half_max = (size * 0.3).round().max(2.0) as i32;
    crown_cone(fb, x, top, n, half_max, 0xa1e, tone, lit, false);
    // Ramillas colgantes en los bordes.
    let (_, mid, sh) = tone;
    for i in (1..n).step_by(3) {
        let t = (i + 1) as f64 / n as f64;
        let half = (half_max as f64 * t.powf(0.82)).round().max(1.0) as i32;
        let yy = top + i;
        dr(fb, x - half - 1, yy + 1, 1, 1, sh);
        dr(fb, x + half + 1, yy + 1, 1, 1, sh);
    }
    dr(fb, x - 1, top - 1, 2, 1, mid);
}

/// Nalca: roseta de hojas orbiculares gigantes de borde dentado sobre pecíolos
/// gruesos ("gran paraguas de hojas", ver `porte`).
fn draw_nalca(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let (hi, mid, sh) = crown_tone(pal, light);
    let lit = lit_left(light);
    let trunk = get(pal, "trunk");
    let n = 3 + (size * 0.12).floor() as i32;
    for i in 0..n {
        let hh = hash1(i * 13 + x, 0xaa11);
        let lx = x + r((hh - 0.5) * size * 0.9);
        let stem_h = (size * (0.3 + hh * 0.35)).round().max(3.0) as i32;
        let lr = (size * (0.34 + hh * 0.14)).round().max(2.0) as i32;
        let top = base_y - stem_h;
        // Pecíolo grueso.
        dr(fb, lx - 1, top, 2, stem_h, trunk);
        // Hoja orbicular de borde dentado, centrada sobre el pecíolo.
        let cy = top - lr;
        for dy in -lr..=lr {
            let t = dy as f64 / lr as f64;
            let mut span = (lr as f64 * (1.0 - t * t).max(0.0).sqrt()).round() as i32;
            if hash1(i * 31 + lx + dy, 0xaa12) < 0.3 {
                span -= 1;
            }
            if span < 1 {
                continue;
            }
            let yy = cy + dy;
            dr(fb, lx - span, yy, span * 2 + 1, 1, mid);
            if dy > lr / 3 {
                dr(fb, lx - span, yy, span * 2 + 1, 1, sh);
            } else if dy <= 0 {
                if lit {
                    dr(fb, lx - span + 1, yy, span, 1, hi);
                } else {
                    dr(fb, lx + 1, yy, span, 1, hi);
                }
            }
        }
        // Nervadura central.
        dr(fb, lx, cy - lr + 1, 1, lr * 2 - 1, sh);
    }
}

fn draw_colihue(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let (hi, mid, sh) = crown_tone(pal, light);
    let lit = lit_left(light);
    let n = 5 + (size * 0.25).floor() as i32;
    for i in 0..n {
        let hh = hash1(i * 17 + x, 0xc01);
        let cxp = x + r((i as f64 - (n as f64 - 1.0) / 2.0) * 0.8);
        let ch = (size * (0.8 + hh * 0.5)).round().max(5.0) as i32;
        let top = base_y - ch;
        // Caña con nudos.
        for k in 0..ch {
            let c = if lit && k % 2 == 0 { hi } else { mid };
            dr(fb, cxp, top + k, 1, 1, c);
        }
        let mut k = 2;
        while k < ch {
            dr(fb, cxp, base_y - k, 1, 1, sh);
            k += 3;
        }
        // Penacho de hojas en la punta.
        let dir = if lit { -1 } else { 1 };
        for j in 0..4 {
            let ly = top + j;
            let c = if j % 2 == 0 { hi } else { mid };
            dr(fb, cxp + dir * (1 + j / 2), ly, 1, 1, c);
            dr(fb, cxp - dir * (1 + j / 2), ly + (j % 2), 1, 1, c);
        }
        // Hojas laterales.
        let leaves = 3 + (hh * 3.0).floor() as i32;
        let step = (ch as f64 / leaves as f64).round().max(2.0) as i32;
        for j in 0..leaves {
            let ly = top + 2 + j * step;
            let d = if (j + i) % 2 != 0 { 1 } else { -1 };
            dr(fb, cxp + d, ly, 1, 1, mid);
            dr(fb, cxp + d * 2, ly + 1, 1, 1, sh);
        }
    }
}

fn draw_palma(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let trunk_h = (size * 0.5).round().max(4.0) as i32;
    let top = base_y - trunk_h;
    let trunk = get(pal, "trunk");
    let td = shade(trunk, -0.22);
    let (hi, mid, sh) = crown_tone(pal, light);
    dr(fb, x - 1, top, 2, trunk_h, trunk);
    // Anillos del tronco.
    let mut i = 3;
    while i < trunk_h {
        dr(fb, x - 1, top + i, 2, 1, td);
        i += 3;
    }
    // Penacho de frondas que arquean y caen.
    let rr = (size * 0.6).round().max(4.0) as i32;
    let n = 9;
    for j in 0..n {
        let f = (j as f64 - (n as f64 - 1.0) / 2.0) / ((n as f64 - 1.0) / 2.0); // -1..1
        let theta = f * 1.15;
        let len = (rr as f64 * (1.0 - 0.12 * f.abs())).round().max(2.0) as i32;
        frond(fb, x, top, theta, len, mid, hi, sh);
    }
    // Cogollo.
    dr(fb, x - 1, top - 1, 3, 1, hi);
    dr(fb, x, top - 2, 1, 1, hi);
}

fn draw_coihue(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 1.1).round().max(6.0) as i32;
    let trunk_h = (size * 0.42).round().max(3.0) as i32;
    let top = base_y - h;
    let trunk = get(pal, "trunk");
    let td = shade(trunk, -0.25);
    let crown_h = h - trunk_h;

    // Tronco recto con horqueta que abre hacia la copa.
    dr(fb, x - 1, base_y - trunk_h, 2, trunk_h, trunk);
    let fork = base_y - trunk_h;
    line(fb, x, fork, x - 3, fork - 2, td);
    line(fb, x + 1, fork, x + 4, fork - 2, td);
    line(fb, x - 3, fork - 2, x - 5, fork - 4, td);
    line(fb, x + 4, fork - 2, x + 6, fork - 4, td);

    let tone = crown_tone(pal, light);
    let lit = lit_left(light);
    let rx = (size * 0.5).round().max(2.0) as i32;
    let cy = top + crown_h / 2;
    // Copa ancha: tres lóbulos solapados.
    crown_blob(fb, x, cy, rx, crown_h / 2, 0xc01, tone, lit);
    crown_blob(
        fb,
        x - rx * 2 / 3,
        top + crown_h / 3,
        rx * 3 / 4,
        crown_h / 3,
        0xc02,
        tone,
        lit,
    );
    crown_blob(
        fb,
        x + rx * 2 / 3,
        top + crown_h / 3,
        rx * 3 / 4,
        crown_h / 3,
        0xc03,
        tone,
        lit,
    );
}

fn draw_roble(
    fb: &mut Framebuffer,
    x: f64,
    base_y: f64,
    size: f64,
    pal: &Palette,
    season: i32,
    light: f64,
) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 1.15).round().max(6.0) as i32;
    let trunk_h = (size * 0.3).round().max(2.0) as i32;
    let top = base_y - h;
    let trunk = get(pal, "trunk");
    dr(fb, x - 1, base_y - trunk_h, 2, trunk_h, trunk);
    let crown_h = h - trunk_h;
    if season == 2 {
        draw_bare_branches(fb, x, top, crown_h, pal);
        return;
    }
    let (hi, mid, sh) = crown_tone(pal, light);
    let lit = lit_left(light);
    let half_max = (size * 0.3).round().max(2.0) as i32;

    // Copa estrecha y erguida (huso), más angosta que el coihue.
    for i in 0..crown_h {
        if season == 1 && hash1(x * 11 + i + 0x2f, 0xb3e) < 0.3 {
            continue;
        }
        let t = i as f64 / crown_h as f64;
        let mut half = (half_max as f64 * ((0.3 + 0.7 * t) * std::f64::consts::PI).sin())
            .round()
            .max(1.0) as i32;
        if hash1(x * 13 + i, 0xb0b) < 0.3 {
            half = (half - 1).max(1);
        }
        let yy = top + i;
        dr(fb, x - half, yy, half * 2 + 1, 1, mid);
        if lit {
            dr(fb, x + 1, yy, half, 1, sh);
        } else {
            dr(fb, x - half, yy, half, 1, sh);
        }
        if i % 2 == 0 {
            if lit {
                dr(fb, x - half + 1, yy, half, 1, hi);
            } else {
                dr(fb, x + 1, yy, half, 1, hi);
            }
        }
    }
    dr(fb, x - 1, top - 1, 2, 1, hi);
    if season == 3 {
        dr(fb, x - 2, top, 1, 1, hi);
        dr(fb, x + 1, top + 1, 1, 1, hi);
    }
}

fn draw_copihue(
    fb: &mut Framebuffer,
    x: f64,
    base_y: f64,
    size: f64,
    pal: &Palette,
    season: i32,
    light: f64,
) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 1.05).round().max(6.0) as i32;
    let top = base_y - h;
    let trunk = get(pal, "trunk");
    let (hi, _, d) = crown_tone(pal, light);

    // Tallo trepador sinuoso.
    let mut prev = x;
    for i in 0..h {
        let yy = base_y - i;
        let off = r((i as f64 * 0.5).sin() * 2.0);
        let cx = x + off;
        line(fb, prev, yy + 1, cx, yy, if i % 3 == 0 { trunk } else { d });
        prev = cx;
    }
    // Hojas a distintos niveles.
    let leaves = 1 + (size * 0.15) as i32;
    for i in 0..leaves {
        let ly = top + (i + 1) * h / (leaves + 1);
        let off = r(((base_y - ly) as f64 * 0.5).sin() * 2.0);
        let lx = x + off;
        dr(fb, lx + if i % 2 == 0 { 1 } else { -1 }, ly, 2, 1, hi);
    }
    if season != 0 && season != 3 {
        return;
    }
    // Campanas rojas colgantes (sello del copihue).
    let n = 2 + (size * 0.2).floor() as i32;
    for i in 0..n {
        let yy = top + r(((i + 1) as f64 / (n + 1) as f64) * h as f64);
        let off = r(((base_y - yy) as f64 * 0.5).sin() * 2.0);
        let bx = x + off;
        dr(fb, bx + 1, yy, 1, 1, [0xe0, 0x50, 0x70]);
        dr(fb, bx, yy + 1, 3, 1, [0xd9, 0x4f, 0x6a]);
        dr(fb, bx, yy + 2, 3, 1, [0xd9, 0x4f, 0x6a]);
        dr(fb, bx + 1, yy + 3, 1, 1, [0xf2, 0xa0, 0xb4]);
    }
}

fn draw_michay(
    fb: &mut Framebuffer,
    x: f64,
    base_y: f64,
    size: f64,
    pal: &Palette,
    season: i32,
    light: f64,
) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 0.8).round().max(4.0) as i32;
    let rx = (size * 0.5).round().max(2.0) as i32;
    let tone = crown_tone(pal, light);
    let lit = lit_left(light);
    // Arbusto denso: tres lóbulos bajos y apiñados.
    crown_blob(fb, x, base_y - h / 2, rx, h / 2, 0xb011, tone, lit);
    crown_blob(
        fb,
        x - rx / 2,
        base_y - h / 2 - h / 4,
        rx * 3 / 4,
        h / 3,
        0xb015,
        tone,
        lit,
    );
    crown_blob(
        fb,
        x + rx / 2,
        base_y - h / 2 - h / 4,
        rx * 3 / 4,
        h / 3,
        0xb016,
        tone,
        lit,
    );
    if season != 0 && season != 3 {
        return;
    }
    // Flores naranjas en primavera/verano.
    let n = 3 + (size * 0.3).floor() as i32;
    for i in 0..n {
        let hh = hash1(i * 23 + x, 0xb012);
        let fx = x + r((hh - 0.5) * rx as f64 * 1.9);
        let fy = base_y - h + r(hash1(i, 0xb014) * h as f64);
        dr(fb, fx, fy, 1, 1, [0xe8, 0x91, 0x2a]);
        dr(fb, fx + 1, fy, 1, 1, [0xf6, 0xc0, 0x4a]);
    }
}

fn draw_chaura(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 0.72).round().max(3.0) as i32;
    let rx = (size * 0.55).round().max(2.0) as i32;
    let tone = crown_tone(pal, light);
    crown_blob(
        fb,
        x,
        base_y - h / 2,
        rx,
        h / 2,
        0x0c4,
        tone,
        lit_left(light),
    );
    // Bayas blanco-rosadas (perenne).
    let n = 3 + (size * 0.2).floor() as i32;
    for i in 0..n {
        let hh = hash1(i * 29 + x, 0x0c5);
        let fx = x + r((hh - 0.5) * rx as f64 * 1.7);
        let fy = base_y - h / 2 + r((hash1(i, 0x0c6) - 0.5) * h as f64);
        dr(fb, fx, fy, 1, 1, [0xf2, 0xd8, 0xe8]);
        dr(fb, fx, fy + 1, 1, 1, [0xd9, 0x7a, 0x9a]);
    }
}

fn draw_quillay(
    fb: &mut Framebuffer,
    x: f64,
    base_y: f64,
    size: f64,
    pal: &Palette,
    season: i32,
    light: f64,
) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 0.95).round().max(5.0) as i32;
    let trunk_h = (size * 0.34).round().max(2.0) as i32;
    let top = base_y - h;
    // Corteza pálida del quillay.
    let trunk = lerp_color(get(pal, "trunk"), get(pal, "sand"), 0.45);
    dr(fb, x - 1, base_y - trunk_h, 2, trunk_h, trunk);
    line(
        fb,
        x,
        base_y - trunk_h,
        x - 3,
        base_y - trunk_h - 2,
        shade(trunk, -0.2),
    );
    line(
        fb,
        x + 1,
        base_y - trunk_h,
        x + 3,
        base_y - trunk_h - 2,
        shade(trunk, -0.2),
    );
    let crown_h = h - trunk_h;
    let tone = crown_tone(pal, light);
    let lit = lit_left(light);
    let rx = (crown_h / 2).max(1);
    // Copa globosa: casi esférica, de borde irregular.
    crown_blob(fb, x, top + crown_h / 2, rx, rx, 0x9111, tone, lit);
    if season == 0 || season == 3 {
        let n = 1 + (size * 0.12).floor() as i32;
        for i in 0..n {
            let hh = hash1(i * 37 + x, 0x9114);
            let fy = top + 1 + r(((i + 1) as f64 / (n + 1) as f64) * crown_h as f64);
            dr(
                fb,
                x + r((hh - 0.5) * rx as f64 * 1.6),
                fy,
                1,
                1,
                [0xf4, 0xf0, 0xe6],
            );
        }
    }
}

fn draw_manio(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 1.0).round().max(7.0) as i32;
    let trunk_h = (size * 0.16).round().max(2.0) as i32;
    let top = base_y - h;
    let trunk = get(pal, "trunk");
    dr(fb, x - 1, base_y - trunk_h, 2, trunk_h, trunk);
    let n = h - trunk_h;
    let (hi, mid, sh) = crown_tone(pal, light);
    // Tono más oscuro (conífera austral).
    let tone = (shade(hi, -0.06), shade(mid, -0.10), shade(sh, -0.10));
    let lit = lit_left(light);
    let half_max = (size * 0.26).round().max(2.0) as i32;
    crown_cone(fb, x, top, n, half_max, 0x3a17, tone, lit, true);
    // Ramas colgantes: puntitos que cuelgan del borde cada pocas filas.
    for i in (1..n).step_by(3) {
        let t = (i + 1) as f64 / n as f64;
        let half = (half_max as f64 * t.powf(1.35)).round().max(1.0) as i32;
        let yy = top + i;
        dr(fb, x - half - 1, yy + 1, 1, 2, shade(mid, -0.12));
        dr(fb, x + half + 1, yy + 1, 1, 2, shade(mid, -0.12));
    }
}

fn draw_canelo(
    fb: &mut Framebuffer,
    x: f64,
    base_y: f64,
    size: f64,
    pal: &Palette,
    season: i32,
    light: f64,
) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 0.95).round().max(6.0) as i32;
    let trunk_h = (size * 0.26).round().max(2.0) as i32;
    let top = base_y - h;
    // Corteza rojiza del canelo.
    let bark = [0x7a, 0x46, 0x38];
    dr(fb, x - 1, base_y - trunk_h, 2, trunk_h, bark);
    let crown_h = h - trunk_h;
    let tone = crown_tone(pal, light);
    let lit = lit_left(light);
    let rx = (size * 0.34).round().max(2.0) as i32;
    let ry = crown_h / 2;
    crown_blob(fb, x, top + ry, rx, ry, 0xc0e, tone, lit);
    // Ápice puntiagudo: la copa densa y erguida del canelo.
    dr(fb, x - 1, top - 2, 2, 2, get(pal, "floraD"));
    dr(fb, x, top - 3, 1, 1, get(pal, "floraL"));
    if season == 0 || season == 3 {
        let n = 1 + (size * 0.12).floor() as i32;
        for i in 0..n {
            let hh = hash1(i * 41 + x, 0xc0e);
            dr(
                fb,
                x + r((hh - 0.5) * rx as f64 * 1.7),
                top + 1 + r(hh * (crown_h - 1).max(1) as f64),
                1,
                1,
                [0xf6, 0xf2, 0xe6],
            );
        }
    }
}

fn draw_arrayan(
    fb: &mut Framebuffer,
    x: f64,
    base_y: f64,
    size: f64,
    pal: &Palette,
    season: i32,
    light: f64,
) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 0.82).round().max(5.0) as i32;
    let trunk_h = (size * 0.42).round().max(3.0) as i32;
    let top = base_y - h;
    let bark = [0x8a, 0x4a, 0x3a];
    // Tronco canela, alto y visible (sello del arrayán).
    dr(fb, x - 1, base_y - trunk_h, 2, trunk_h, bark);
    line(
        fb,
        x,
        base_y - trunk_h,
        x - 2,
        base_y - trunk_h - 1,
        shade(bark, -0.2),
    );
    line(
        fb,
        x + 1,
        base_y - trunk_h,
        x + 2,
        base_y - trunk_h - 1,
        shade(bark, -0.2),
    );
    let crown_h = h - trunk_h;
    if crown_h < 1 {
        return;
    }
    let tone = crown_tone(pal, light);
    let lit = lit_left(light);
    let rx = (size * 0.34).round().max(2.0) as i32;
    crown_blob(fb, x, top + crown_h / 2, rx, crown_h / 2, 0xa77a, tone, lit);
    if season == 0 || season == 3 {
        let n = 1 + (size * 0.12).floor() as i32;
        for i in 0..n {
            let hh = hash1(i * 43 + x, 0xa77b);
            dr(
                fb,
                x + r((hh - 0.5) * rx as f64 * 1.6),
                top + 1 + (i % crown_h.max(1)),
                1,
                1,
                [0xf4, 0xf0, 0xe6],
            );
        }
    }
}

fn draw_notro(
    fb: &mut Framebuffer,
    x: f64,
    base_y: f64,
    size: f64,
    pal: &Palette,
    season: i32,
    light: f64,
) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 0.9).round().max(5.0) as i32;
    let trunk_h = (size * 0.3).round().max(2.0) as i32;
    let top = base_y - h;
    dr(fb, x - 1, base_y - trunk_h, 2, trunk_h, get(pal, "trunk"));
    let crown_h = h - trunk_h;
    if crown_h < 1 {
        return;
    }
    let tone = crown_tone(pal, light);
    let lit = lit_left(light);
    let rx = (size * 0.3).round().max(2.0) as i32;
    crown_blob(fb, x, top + crown_h / 2, rx, crown_h / 2, 0x07e0, tone, lit);
    if season != 0 && season != 3 {
        return;
    }
    // Racimos de flores escarlata (sello del notro).
    let n = 2 + (size * 0.2).floor() as i32;
    for i in 0..n {
        let hh = hash1(i * 47 + x, 0x07e1);
        let fx = x + r((hh - 0.5) * rx as f64 * 1.9);
        let fy = top + 1 + r(hash1(i * 59 + x, 0x07e2) * (crown_h - 1).max(1) as f64);
        dr(fb, fx, fy, 2, 1, [0xd1, 0x40, 0x1e]);
        dr(fb, fx + (i % 2), fy - 1, 1, 1, [0xf2, 0x6a, 0x2a]);
    }
}

fn draw_espino(
    fb: &mut Framebuffer,
    x: f64,
    base_y: f64,
    size: f64,
    pal: &Palette,
    season: i32,
    light: f64,
) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 0.85).round().max(5.0) as i32;
    let trunk = get(pal, "trunk");
    let td = shade(trunk, -0.2);
    let (hi, mid, d) = crown_tone(pal, light);
    let spread = (size * 0.5).round().max(3.0) as i32;

    // Base corta y ramas en abanico (matorral, no árbol).
    dr(fb, x - 1, base_y - 2, 2, 2, trunk);
    let arms = [
        (-1.0f64, 0.85f64),
        (1.0, 0.85),
        (-0.55, 1.0),
        (0.55, 1.0),
        (0.0, 1.0),
        (-1.0, 0.5),
        (1.0, 0.5),
    ];
    for (k, (dx, dy)) in arms.iter().enumerate() {
        let ex = x + r(*dx * spread as f64);
        let ey = base_y - r(*dy * h as f64);
        line(
            fb,
            x,
            base_y - 1,
            ex,
            ey,
            if k % 2 == 0 { trunk } else { td },
        );
        for j in 0..=(1 + (k % 3)) {
            let t = (j + 1) as f64 / 4.0;
            let px = x + r((ex - x) as f64 * t);
            let py = base_y - r((base_y - ey) as f64 * t);
            dr(fb, px + 1, py, 1, 1, td);
            if j % 2 == 0 {
                dr(fb, px - 1, py, 1, 1, td);
            }
        }
    }
    // Follaje disperso.
    for i in 0..(3 + (size * 0.4).floor() as i32) {
        let hh = hash1(i * 53 + x, 0x0e6);
        let fx = x + r((hh - 0.5) * spread as f64 * 1.8);
        let fy = base_y - r((0.35 + hash1(i, 0x0e7) * 0.6) * h as f64);
        let c = if i % 2 == 0 { mid } else { hi };
        dr(fb, fx, fy, 2, 1, c);
        dr(fb, fx + 1, fy + 1, 1, 1, d);
    }
    if season == 0 || season == 3 {
        let n = 2 + (size * 0.3).floor() as i32;
        for i in 0..n {
            let hh = hash1(i * 53 + x, 0x0e8);
            dr(
                fb,
                x + r((hh - 0.5) * spread as f64 * 1.9),
                base_y - r((0.4 + hh * 0.55) * h as f64),
                1,
                1,
                FLOWER_COLORS[1],
            );
        }
    }
}

fn draw_litre(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 0.88).round().max(5.0) as i32;
    let trunk_h = (size * 0.22).round().max(2.0) as i32;
    let top = base_y - h;
    let tr = get(pal, "trunk");
    dr(fb, x - 1, base_y - trunk_h, 2, trunk_h, tr);
    // Corteza moteada del litre.
    let tdk = shade(tr, -0.3);
    let mut k = 1;
    while k < trunk_h {
        dr(fb, x - 1 + (k % 2), base_y - k, 1, 1, tdk);
        k += 2;
    }
    let crown_h = h - trunk_h;
    let tone = crown_tone(pal, light);
    let lit = lit_left(light);
    let rx = (size * 0.4).round().max(3.0) as i32;
    crown_blob(fb, x, top + crown_h / 2, rx, crown_h / 2, 0x117, tone, lit);
    // Follaje moteado (hojas claras/oscuras) del litre.
    let (hi, _, d) = tone;
    let n = 2 + (size * 0.3).floor() as i32;
    for i in 0..n {
        let yy = top + 1 + (i % crown_h.max(1));
        if hash1(i * 7 + x, 0x117) < 0.6 {
            dr(
                fb,
                x + r((hash1(i, 0x118) - 0.5) * rx as f64 * 1.2),
                yy,
                1,
                1,
                if i % 2 == 0 { hi } else { d },
            );
        }
    }
}

fn draw_boldo(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 0.88).round().max(5.0) as i32;
    let trunk_h = (size * 0.3).round().max(2.0) as i32;
    let top = base_y - h;
    let tr = get(pal, "trunk");
    dr(fb, x - 1, base_y - trunk_h, 2, trunk_h, tr);
    dr(fb, x, base_y - trunk_h, 1, trunk_h, shade(tr, -0.25));
    let crown_h = h - trunk_h;
    let (hi, mid, d) = crown_tone(pal, light);
    // Follaje grisáceo y aromático (sello del boldo).
    let grey = get(pal, "rock");
    let tone = (lerp_color(hi, grey, 0.25), lerp_color(mid, grey, 0.3), d);
    let lit = lit_left(light);
    let rx = (size * 0.4).round().max(3.0) as i32;
    crown_blob(fb, x, top + crown_h / 2, rx, crown_h / 2, 0xb01, tone, lit);
    for i in 0..(2 + (size * 0.25).floor() as i32) {
        let yy = top + 1 + (i % crown_h.max(1));
        if hash1(i * 3 + x, 0xb01) < 0.6 {
            dr(
                fb,
                x + r((hash1(i, 0xb02) - 0.5) * rx as f64 * 1.3),
                yy,
                1,
                1,
                d,
            );
        }
    }
}

fn draw_peumo(
    fb: &mut Framebuffer,
    x: f64,
    base_y: f64,
    size: f64,
    pal: &Palette,
    season: i32,
    light: f64,
) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 0.95).round().max(6.0) as i32;
    let trunk_h = (size * 0.28).round().max(2.0) as i32;
    let top = base_y - h;
    // Tronco oscuro y grueso del peumo.
    let bark = shade(get(pal, "trunk"), -0.25);
    dr(fb, x - 1, base_y - trunk_h, 2, trunk_h, bark);
    let crown_h = h - trunk_h;
    let (hi, mid, d) = crown_tone(pal, light);
    // Siempreverde denso: copa ancha y oscura.
    let tone = (shade(hi, -0.05), shade(mid, -0.08), d);
    let lit = lit_left(light);
    let rx = (size * 0.46).round().max(3.0) as i32;
    crown_blob(fb, x, top + crown_h / 2, rx, crown_h / 2, 0xbe0, tone, lit);
    // Frutos rojos en otoño/invierno.
    if season == 1 || season == 2 {
        let n = 2 + (size * 0.2).floor() as i32;
        for i in 0..n {
            let hh = hash1(i * 53 + x, 0xbe1);
            let fy = top + 1 + r(hash1(i, 0xbe2) * (crown_h - 1).max(1) as f64);
            dr(
                fb,
                x + r((hh - 0.5) * rx as f64 * 1.6),
                fy,
                1,
                1,
                [0xc0, 0x3a, 0x3a],
            );
        }
    }
}

fn draw_maiten(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 1.0).round().max(6.0) as i32;
    let trunk_h = (size * 0.3).round().max(3.0) as i32;
    let top = base_y - h;
    let trunk = get(pal, "trunk");
    dr(fb, x - 1, base_y - trunk_h, 2, trunk_h, trunk);
    let crown_h = h - trunk_h;
    let tone = crown_tone(pal, light);
    let (hi, mid, _) = tone;
    let lit = lit_left(light);
    let rx = (size * 0.42).round().max(3.0) as i32;
    let dome = (crown_h / 2).max(1);

    // Cúpula.
    crown_blob(
        fb,
        x,
        top + dome - dome / 2,
        rx,
        (dome / 2).max(1),
        0xaa17,
        tone,
        lit,
    );
    // Ramas lloronas que caen de la copa.
    for (i, k) in [-1.0f64, 1.0, -0.6, 0.6, -0.2, 0.2].iter().enumerate() {
        let lx = x + r(*k * rx as f64);
        let len = ((crown_h as f64) * 0.75 - (i as f64) * 0.8).max(2.0) as i32;
        let c = if i % 2 == 0 { mid } else { hi };
        let mut x0 = lx;
        for j in 0..len {
            dr(fb, x0, top + dome + j, 1, 1, c);
            if j % 3 == 2 {
                x0 += if *k < 0.0 { 1 } else { -1 };
            }
        }
    }
}

fn draw_cipres(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 1.15).round().max(9.0) as i32;
    let trunk_h = (size * 0.14).round().max(2.0) as i32;
    let top = base_y - h;
    dr(fb, x - 1, base_y - trunk_h, 2, trunk_h, get(pal, "trunk"));
    let n = h - trunk_h;
    let half_max = (size * 0.22).round().max(2.0) as i32;
    // Conífera columnar de la precordillera.
    let tone = crown_tone(pal, light);
    crown_cone(fb, x, top, n, half_max, 0xc1a, tone, lit_left(light), false);
    dr(fb, x - 1, top - 1, 2, 1, get(pal, "floraL"));
}

fn draw_puya(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let (hi, mid, sh) = crown_tone(pal, light);
    // Roseta basal de hojas rígidas y espinosas.
    let r0 = (size * 0.45).round().max(2.0) as i32;
    rosette(fb, x, base_y - 1, r0, hi, mid, sh);
    // Vara floral alta con candelabro de flores.
    let sh_h = (size * 0.95).round().max(5.0) as i32;
    let top = base_y - sh_h;
    dr(fb, x, top, 1, sh_h, mid);
    let nbr = 3 + (size * 0.15).floor() as i32;
    for j in 0..nbr {
        let t = (j + 1) as f64 / (nbr + 1) as f64;
        let by = top + r(t * sh_h as f64);
        let bl = ((size * 0.35) * (1.0 - t * 0.6)).round().max(1.0) as i32;
        for k in 1..=bl {
            let yy = by - k / 2;
            dr(fb, x + k, yy, 1, 1, mid);
            dr(fb, x - k, yy, 1, 1, mid);
        }
        dr(fb, x + bl, by - bl / 2, 1, 1, FLOWER_COLORS[1]);
        dr(fb, x - bl, by - bl / 2, 1, 1, FLOWER_COLORS[0]);
    }
    for k in (0..sh_h).step_by(3) {
        dr(fb, x, top + k, 1, 1, FLOWER_COLORS[0]);
    }
    dr(fb, x, top - 1, 1, 1, FLOWER_COLORS[1]);
}

fn draw_llareta(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let (hi, mid, sh) = crown_tone(pal, light);
    let sun = get(pal, "sun");
    // Cojín verde-amarillento de la puna.
    let hi2 = lerp_color(hi, sun, 0.25);
    let mid2 = lerp_color(mid, sun, 0.25);
    let w = (size * 1.6).round().max(4.0) as i32;
    let h = (size * 0.8).round().max(3.0) as i32;
    for rr in 0..h {
        let t = rr as f64 / (h - 1).max(1) as f64;
        let half = (w as f64 * 0.5 * (1.0 - (1.0 - t).powi(2)).max(0.0).sqrt())
            .round()
            .max(1.0) as i32;
        let yy = base_y - h + rr;
        dr(fb, x - half, yy, half * 2 + 1, 1, mid2);
        if rr * 2 < h {
            dr(fb, x - half, yy, half * 2 + 1, 1, hi2);
        }
    }
    // Textura: bultos de rosetas apiñadas.
    let n = 3 + (size * 0.8) as i32;
    for i in 0..n {
        let hh = hash1(i * 19 + x, 0x11a);
        let bx = x + r((hh - 0.5) * w as f64 * 0.8);
        let byy = base_y - r(hash1(i, 0x11b) * h as f64);
        dr(fb, bx, byy, 1, 1, if i % 2 == 0 { hi2 } else { sh });
    }
}

fn draw_amancay(
    fb: &mut Framebuffer,
    x: f64,
    base_y: f64,
    size: f64,
    pal: &Palette,
    sway: f64,
    light: f64,
) {
    let x = r(x);
    let base_y = r(base_y);
    let (hi, mid, _sh) = crown_tone(pal, light);
    let n = 2 + (size * 0.5).floor() as i32;
    for i in 0..n {
        let sx = x + i - (n >> 1);
        let h = (size * 0.9).round().max(3.0) as i32 + (i % 2);
        dr(fb, sx, base_y - h, 1, h, mid);
        dr(fb, sx, base_y - h, 1, h / 2, hi);
        // Umbela: flores amarillas agrupadas en la punta.
        let lean = r(sway * (2.0 + i as f64));
        let fx = sx + lean;
        let fy = base_y - h - 1;
        dr(fb, fx, fy, 1, 1, [0xf7, 0xd9, 0x70]);
        dr(fb, fx - 1, fy, 1, 1, FLOWER_COLORS[1]);
        dr(fb, fx + 1, fy, 1, 1, FLOWER_COLORS[1]);
        dr(fb, fx, fy - 1, 1, 1, [0xf7, 0xd9, 0x70]);
    }
}

fn draw_junquillo(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let (hi, mid, sh) = crown_tone(pal, light);
    let lit = lit_left(light);
    let n = 4 + (size * 0.8).floor() as i32;
    for i in 0..n {
        let gx = x + i - (n >> 1);
        let h = (size * 1.0).round().max(3.0) as i32 + ((i * 3 + x) % 2);
        let c = if (gx < x) == lit { hi } else { mid };
        dr(fb, gx, base_y - h, 1, h, c);
        dr(
            fb,
            gx,
            base_y - h - 1,
            1,
            1,
            if i % 2 == 0 { hi } else { sh },
        );
    }
}

fn draw_flower(
    fb: &mut Framebuffer,
    x: f64,
    base_y: f64,
    size: f64,
    pal: &Palette,
    sway: f64,
    night: f64,
) {
    let x = r(x);
    let base_y = r(base_y);
    let w = size * 2.6;
    let n = 6 + (size * 0.7).floor() as i32;
    let dim = night.clamp(0.0, 1.0) * 0.85;
    let flora_d = get(pal, "floraD");
    let sky_top = get(pal, "skyTop");
    for i in 0..n {
        let hh = hash1(i * 31 + x, 0xf10e);
        let back = if hash1(i * 53 + x, 0xf10f) < 0.45 {
            1
        } else {
            0
        };
        let fx = x + r((hh - 0.5) * w + sway) - back;
        let stem_h = 1 + (hh * 4.0).floor() as i32;
        let base =
            FLOWER_COLORS[(hash1(i * 7 + x, 0xf1a) * FLOWER_COLORS.len() as f64).floor() as usize];
        let col = if dim > 0.0 {
            lerp_color(base, sky_top, dim)
        } else {
            base
        };
        let yy = base_y - back;
        dr(fb, fx, yy - stem_h, 1, stem_h, flora_d);
        // Corola de 5 pétalos.
        dr(fb, fx, yy - stem_h - 1, 1, 1, col);
        dr(fb, fx - 1, yy - stem_h - 1, 1, 1, col);
        dr(fb, fx + 1, yy - stem_h - 1, 1, 1, col);
        dr(fb, fx - 1, yy - stem_h, 1, 1, col);
        dr(fb, fx + 1, yy - stem_h, 1, 1, col);
        dr(fb, fx, yy - stem_h - 2, 1, 1, [0xf7, 0xd9, 0x70]);
    }
}

/// Pools de centro derivados de las zonas (fallback sin `poolAt`).
pub fn centro_flora_pool(layer_name: &str) -> Vec<(&'static str, f64)> {
    let mut out = Vec::new();
    for e in FLORA {
        for &(biome, layer, w) in e.zones {
            if biome == "centro" && layer == layer_name {
                out.push((e.name, w));
            }
        }
    }
    out
}

const DEFAULT_FLORA_POOL: [(&str, f64); 1] = [("grass", 1.0)];

fn pick_type(pool: &[(&str, f64)], rng: &mut Mulberry32) -> &'static str {
    let total: f64 = pool.iter().map(|(_, w)| *w).sum();
    if total <= 0.0 {
        return intern(pool[0].0);
    }
    let mut r = rng.next_f64() * total;
    for (t, w) in pool {
        r -= *w;
        if r < 0.0 {
            return intern(t);
        }
    }
    intern(pool[pool.len() - 1].0)
}

/// Devuelve el `&'static str` del registro a partir de un nombre.
fn intern(name: &str) -> &'static str {
    FLORA
        .iter()
        .find(|e| e.name == name)
        .map(|e| e.name)
        .unwrap_or("grass")
}

pub struct FloralSpawn {
    pub wx: f64,
    pub sx: f64,
    pub gy: i32,
    pub size: f64,
    pub type_: &'static str,
    pub warm: bool,
    pub depth: i32,
}

#[allow(clippy::too_many_arguments)]
pub fn flora_spawns<F>(
    layer: &Layer,
    terrain: &Terrain,
    layer_idx: usize,
    camera_x: f64,
    w: usize,
    h: usize,
    seed: u32,
    pool_at: F,
) -> Vec<FloralSpawn>
where
    F: Fn(f64) -> Vec<(&'static str, f64)>,
{
    let Some(f) = layer.flora else {
        return Vec::new();
    };
    let p = layer.parallax;
    let chunk_w = f.chunk_w;
    let start_c = ((camera_x * p - 80.0) / chunk_w).floor() as i64;
    let end_c = ((camera_x * p + w as f64 + 80.0) / chunk_w).floor() as i64;
    let mut out = Vec::new();
    let layer_seed = layer.seed;
    for c in start_c..=end_c {
        let mut rng = Mulberry32::new(hash_int(
            c as i32,
            seed.wrapping_add(layer_seed.wrapping_mul(7)),
        ));
        let count = (rng.next_f64() * f.max_per).floor() as i32
            + if rng.next_f64() < f.min_chance { 1 } else { 0 };
        // El pool se resuelve una vez por chunk: el bioma varía despacio y
        // construirlo por spawn obliga a recalcular pesos y recorrer el registro.
        let pool = pool_at(c as f64 * chunk_w);
        let list = if !pool.is_empty() {
            pool
        } else {
            centro_flora_pool(layer.name)
        };
        for _ in 0..count {
            let wx = (c as f64 + rng.next_f64()) * chunk_w;
            let sx = wx - camera_x * p;
            if sx < -48.0 || sx > w as f64 + 48.0 {
                continue;
            }
            if terrain.river_influence(layer_idx, wx) > 0.25 {
                continue;
            }
            let gy = r(terrain.bank_height(layer_idx, wx));
            if gy > h as i32 + 4 {
                continue;
            }
            let base = f.min_size + rng.next_f64() * (f.max_size - f.min_size);
            let type_ = if !list.is_empty() {
                pick_type(&list, &mut rng)
            } else {
                pick_type(&DEFAULT_FLORA_POOL, &mut rng)
            };
            let size = base * height_for(type_);
            let warm = rng.next_f64() < 0.4;
            let mut depth = 0;
            if type_ == "flower" {
                let d = hash1((wx * 0.5).floor() as i32, 0xf100);
                depth = r(d * (layer.amp * 1.3));
            }
            out.push(FloralSpawn {
                wx,
                sx,
                gy,
                size,
                type_,
                warm,
                depth,
            });
        }
    }
    out
}

#[allow(clippy::too_many_arguments)]
pub fn place_flora(
    fb: &mut Framebuffer,
    layer: &Layer,
    terrain: &Terrain,
    layer_idx: usize,
    pal: &Palette,
    camera_x: f64,
    w: usize,
    h: usize,
    seed: u32,
    t_sec: f64,
    biome_mode: &str,
    bloom_mode: &str,
    season: i32,
    night: f64,
    light: f64,
) {
    if layer.flora.is_none() {
        return;
    }
    let mut pal2 = *pal;
    if layer.darken != 0.0 {
        for (key, amt) in [
            ("floraL", layer.darken),
            ("floraD", layer.darken),
            ("trunk", layer.darken),
            ("rock", layer.darken),
            ("rockD", layer.darken),
            ("sun", layer.darken * 0.5),
            ("sunGlow", layer.darken * 0.5),
        ] {
            if let Some(idx) = crate::palette::field_index(key) {
                pal2[idx] = shade(pal2[idx], -amt);
            }
        }
    }
    let night_amt = if night.is_finite() {
        night.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let pool_at = |wx: f64| -> Vec<(&'static str, f64)> {
        let (weights, bloom) = crate::biomes::biome_weights_bloom(wx, seed, biome_mode);
        let bloom = resolve_bloom(bloom_mode, &weights, bloom);
        biome_flora_pool(layer.name, &weights, bloom)
    };
    for s in flora_spawns(layer, terrain, layer_idx, camera_x, w, h, seed, pool_at) {
        let sway = (t_sec * 0.9 + s.wx * 0.05).sin();
        let gy = if s.depth != 0 {
            (s.gy + s.depth).min(h as i32 - 2)
        } else {
            s.gy
        };
        draw_plant(
            fb, s.sx, gy as f64, s.size, s.type_, &pal2, sway, s.warm, season, night_amt, light,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::get_palette;

    const SEED: u32 = 662118718;

    #[test]
    fn registry_has_all_drawers() {
        for e in FLORA {
            // `draw_plant` cae a grass si el tipo no existe; comprobamos cobertura
            // llamando a cada tipo sin pánico.
            assert!(!e.name.is_empty());
        }
    }

    #[test]
    fn every_drawer_paints() {
        // Cada especie debe pintar algo, en las cuatro estaciones y a dos tallas.
        let pal = get_palette(12.0, "clear", 1.0);
        for e in FLORA {
            for season in 0..4 {
                for base in [6.0, 14.0] {
                    let mut fb = Framebuffer::new(200, 120);
                    draw_plant(
                        &mut fb,
                        100.0,
                        100.0,
                        base * e.height,
                        e.name,
                        &pal,
                        0.0,
                        false,
                        season,
                        0.0,
                        -0.6,
                    );
                    assert!(
                        fb.px.iter().any(|p| *p != 0),
                        "{} (estación {season}, talla {base}) no dibujó",
                        e.name
                    );
                }
            }
        }
    }

    #[test]
    fn spawns_are_deterministic() {
        let terrain = Terrain::new(SEED);
        let layer = &crate::terrain::LAYERS[2]; // valle
        let a = flora_spawns(layer, &terrain, 2, 300.0, 480, 270, SEED, |_| Vec::new());
        let b = flora_spawns(layer, &terrain, 2, 300.0, 480, 270, SEED, |_| Vec::new());
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(b.iter()) {
            assert_eq!(x.wx, y.wx);
            assert_eq!(x.type_, y.type_);
        }
    }

    #[test]
    fn place_flora_draws() {
        let terrain = Terrain::new(SEED);
        let pal = get_palette(12.0, "clear", 1.0);
        let mut fb = Framebuffer::new(480, 270);
        for idx in [1, 2, 3, 4] {
            place_flora(
                &mut fb,
                &crate::terrain::LAYERS[idx],
                &terrain,
                idx,
                &pal,
                300.0,
                480,
                270,
                SEED,
                0.0,
                "auto",
                "auto",
                0,
                0.0,
                0.0,
            );
        }
        assert!(fb.px.iter().any(|p| *p != 0));
    }
}
