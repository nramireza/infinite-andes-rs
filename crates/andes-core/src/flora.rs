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

/// Motas deterministas de hoja dentro de una copa: rompen el relleno plano y
/// sugieren follaje sin antialias.
#[allow(clippy::too_many_arguments)]
fn crown_dapple(
    fb: &mut Framebuffer,
    cx: i32,
    cy: i32,
    rx: i32,
    ry: i32,
    hi: Rgb,
    mid: Rgb,
    sh: Rgb,
    seed: u32,
) {
    if rx < 2 || ry < 2 {
        return;
    }
    let n = ((rx * ry) / 5).clamp(3, 44);
    for i in 0..n {
        let hx = hash1(seed.wrapping_add((i as u32).wrapping_mul(73)) as i32, 0x51b);
        let hy = hash1(
            seed.wrapping_add((i as u32).wrapping_mul(131)) as i32,
            0x51c,
        );
        let dx = r((hx * 2.0 - 1.0) * rx as f64 * 0.86);
        let dy = r((hy * 2.0 - 1.0) * ry as f64 * 0.8);
        let nx = dx as f64 / rx.max(1) as f64;
        let ny = dy as f64 / ry.max(1) as f64;
        if nx * nx + ny * ny > 1.0 {
            continue;
        }
        let c = if (i + seed as i32) % 3 == 0 {
            hi
        } else if i % 2 == 0 {
            sh
        } else {
            mid
        };
        dr(fb, cx + dx, cy + dy, 1, 1, c);
    }
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
    crown_dapple(fb, cx, cy, rx, ry, hi, mid, sh, seed);
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
        // Ramillas laterales: el borde de la conífera no es una línea lisa.
        if half > 2 && i % 2 == 0 {
            let sgn = if hash1(seed.wrapping_add(i as u32 * 11) as i32, 0x2c2) < 0.5 {
                -1
            } else {
                1
            };
            let extra = if i % 4 == 0 { 2 } else { 1 };
            dr(
                fb,
                cx + sgn * (half + 1),
                yy + extra - 1,
                1,
                1,
                if sgn < 0 { hi } else { sh },
            );
        }
        // Acículas: motas oscuras y claras dentro de la masa.
        if half > 3 {
            let n = (half / 3).clamp(1, 4);
            for k in 0..n {
                let hh = hash1(
                    seed.wrapping_add((i as u32).wrapping_mul(37).wrapping_add(k as u32)) as i32,
                    0x2c3,
                );
                let px = cx - half + 2 + r(hh * (half * 2 - 3).max(1) as f64);
                dr(fb, px, yy, 1, 1, if (i + k) % 3 == 0 { hi } else { sh });
            }
        }
    }
}

/// Fronda arqueada de palma proyectada en 2D (`x_scale` modela frondas en escorzo
/// hacia/desde la cámara para poblar el centro de la copa sin muesca en V).
#[allow(clippy::too_many_arguments)]
fn frond(
    fb: &mut Framebuffer,
    x: i32,
    top: i32,
    theta: f64,
    x_scale: f64,
    len: i32,
    mid: Rgb,
    hi: Rgb,
    sh: Rgb,
    lit_on_this_side: bool,
) {
    let dir = if theta < 0.0 { -1.0 } else { 1.0 };
    let mut fx = x as f64;
    let mut fy = top as f64;
    let top_col = if lit_on_this_side { hi } else { mid };
    let bot_col = if lit_on_this_side { mid } else { sh };

    for k in 1..=len {
        let t = k as f64 / len.max(1) as f64;
        // La gravedad curva toda fronda hacia afuera y abajo en la punta (fuente radial).
        let ang = theta + dir * (0.28 * t + 0.68 * t * t);
        fx += ang.sin() * x_scale;
        fy -= ang.cos();
        let px = r(fx);
        let py = r(fy);
        let c = if k <= len * 2 / 3 { top_col } else { mid };
        dr(fb, px, py, 1, 1, c);
        // Folíolos pinnados que cuelgan del raquis.
        if len >= 4 && k > 1 && k < len {
            dr(fb, px, py + 1, 1, 1, bot_col);
            if len >= 9 && k % 2 == 0 && k > 2 {
                dr(fb, px, py + 2, 1, 1, sh);
            }
        }
    }
}

/// Roseta de hojas cortas y rígidas (puya y bultos del cojín de llareta).
fn rosette(fb: &mut Framebuffer, cx: i32, cy: i32, radius: i32, hi: Rgb, mid: Rgb, sh: Rgb) {
    if radius < 1 {
        return;
    }
    let n = (10 + radius).clamp(10, 24) as usize;
    for i in 0..n {
        let ang = i as f64 * std::f64::consts::TAU / n as f64 - std::f64::consts::FRAC_PI_2;
        let ux = ang.cos();
        let uy = ang.sin();
        let hh = hash1(i as i32 * 17 + cx, 0x5e7);
        let len = (radius as f64 * (0.68 + hh * 0.42)).round().max(1.0) as i32;
        for k in 1..=len {
            let c = if k == len {
                hi
            } else if k * 2 >= len {
                mid
            } else {
                sh
            };
            let px = cx + r(ux * k as f64);
            let py = cy + r(uy * k as f64);
            dr(fb, px, py, 1, 1, c);
            // Hoja algo más ancha en rosetas grandes.
            if radius >= 6 && k * 2 < len {
                dr(fb, px + if ux < 0.0 { -1 } else { 1 }, py, 1, 1, sh);
            }
        }
    }
}

/// Vetas y nudos de corteza para troncos ya dibujados.
fn bark_marks(fb: &mut Framebuffer, x: i32, y: i32, w: i32, h: i32, trunk: Rgb, seed: u32) {
    if h < 5 || w < 1 {
        return;
    }
    let dark = shade(trunk, -0.28);
    let light = shade(trunk, 0.12);
    let n = (h / 5).clamp(1, 5);
    for i in 0..n {
        let yy = y + 2 + r((i as f64 + 0.5) * (h - 3) as f64 / n as f64);
        let hh = hash1(seed.wrapping_add((i as u32).wrapping_mul(29)) as i32, 0xba4);
        let xx = x + r(hh * w.max(1) as f64);
        dr(fb, xx, yy, 1, if i % 2 == 0 { 1 } else { 2 }, dark);
        if w > 1 && i % 3 == 0 {
            dr(fb, x + (w - 1 - (i & 1)).max(0), yy + 1, 1, 1, light);
        }
    }
}

/// Cono femenino redondo de la araucaria (y otras coníferas).
fn cone_fruit(fb: &mut Framebuffer, x: i32, y: i32, radius: i32, base: Rgb, hi: Rgb) {
    let rr = radius.max(1);
    for dy in -rr..=rr {
        let span = ((rr * rr - dy * dy).max(0) as f64).sqrt().floor() as i32;
        dr(fb, x - span, y + dy, span * 2 + 1, 1, base);
    }
    dr(fb, x, y - rr, 1, 1, hi);
    if rr > 1 {
        dr(fb, x - 1, y, 1, 1, hi);
        dr(fb, x + 1, y + 1, 1, 1, shade(base, -0.2));
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
        "litre" => draw_litre(fb, x, base_y, size, pal, season, light),
        "boldo" => draw_boldo(fb, x, base_y, size, pal, season, light),
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
    let h = (size * 1.08).round().max(12.0) as i32;
    let top = base_y - h;
    let cx = x + r(sway * 0.5);
    let trunk_w = (size * 0.12).round().clamp(2.0, 5.0) as i32;
    let trunk = get(pal, "trunk");
    let td = shade(trunk, -0.28);
    let tl = shade(trunk, 0.12);
    let (hi, mid, sh) = crown_tone(pal, light);
    let lit = lit_left(light);

    // Tronco recto, grueso y despejado en más de la mitad inferior.
    let apex = top;
    let crown_bottom = base_y - r(h as f64 * 0.54);
    let span = (crown_bottom - apex).max(5);
    let trunk_top = apex + r(span as f64 * 0.35);
    let trunk_len = (base_y - trunk_top).max(4);
    dr(
        fb,
        cx - (trunk_w >> 1),
        trunk_top,
        trunk_w,
        trunk_len,
        trunk,
    );
    dr(fb, cx + (trunk_w >> 1) - 1, trunk_top, 1, trunk_len, td);
    if trunk_w > 2 {
        dr(fb, cx - (trunk_w >> 1), trunk_top, 1, trunk_len, tl);
    }
    // Anillos horizontales de corteza pachycaul, típicos del pehuén.
    let mut by = trunk_top + 3;
    while by < base_y - 2 {
        dr(fb, cx - (trunk_w >> 1), by, trunk_w, 1, td);
        by += 4;
    }
    bark_marks(
        fb,
        cx - (trunk_w >> 1),
        trunk_top,
        trunk_w,
        trunk_len,
        trunk,
        x as u32,
    );
    // Base ensanchada (raíces).
    let flare = (size * 0.12).round().clamp(1.0, 3.0) as i32;
    for k in 0..flare {
        dr(
            fb,
            cx - (trunk_w >> 1) - 1 - k,
            base_y - 1 - k,
            1,
            1 + k,
            td,
        );
        dr(fb, cx + (trunk_w >> 1) + k, base_y - 1 - k, 1, 1 + k, td);
    }

    // Copa en domo aparasolado (paraguas): densa y sin huecos horizontales en el
    // interior, con ramas madres visibles bajo la base y puntas erguidas en el borde.
    let base_half = (size * 0.58).round().max(4.0) as i32;
    let tiers = (span / 3).clamp(3, 6);
    let step = (span as f64 / (tiers - 1).max(1) as f64).ceil() as i32;

    // Ramas madres inferiores que nacen del tronco y sostienen el paraguas.
    if h >= 16 {
        let b_y0 = crown_bottom + (h / 14).clamp(1, 3);
        let b_rx = (base_half as f64 * 0.76).round().max(3.0) as i32;
        for &sgn in &[-1i32, 1] {
            line(
                fb,
                cx + sgn * (trunk_w >> 1),
                b_y0,
                cx + sgn * b_rx,
                crown_bottom,
                td,
            );
            if h >= 28 {
                line(
                    fb,
                    cx + sgn * (trunk_w >> 1),
                    b_y0 - 1,
                    cx + sgn * (b_rx * 2 / 3),
                    crown_bottom,
                    trunk,
                );
            }
        }
    }

    for t in 0..tiers {
        let f = t as f64 / (tiers - 1).max(1) as f64; // 0 = base del paraguas, 1 = ápice
        let yy = crown_bottom - r(f * (span - 2).max(2) as f64);
        // Perfil de paraguas/domo: ancho máximo en el tercio inferior (f ≈ 0.24),
        // redondeado hacia el ápice.
        let prof = if f < 0.24 {
            0.80 + 0.20 * (f / 0.24)
        } else {
            let u = (f - 0.24) / 0.76;
            (1.0 - u * u * 0.86).max(0.16).sqrt()
        };
        let rx = (base_half as f64 * prof).round().max(2.0) as i32;
        // Espesor suficiente para solapar el verticilo con el siguiente en el interior,
        // evitando huecos horizontales de lado a lado.
        let thick = (step + 2).clamp(3, 6);
        let tip_in = (rx - 2).max(2);

        for dy in 0..thick {
            let row_y = yy - dy;
            let x0 = cx - rx;
            let x1 = cx + rx;
            for xx in x0..=x1 {
                let dx = (xx - cx).abs();
                // En los 2 px finales del borde el verticilo se separa y curva hacia arriba;
                // en el resto los pisos se solapan formando un domo continuo.
                if dx >= tip_in && dy >= thick - 1 {
                    continue;
                }
                let lift = if dx >= tip_in { 1 } else { 0 };
                let py = row_y - lift;
                let on_lit_side = if lit { xx <= cx } else { xx >= cx };
                let hh = hash1(xx * 13 + py * 29 + t * 7, 0xa71);
                let col = if dy == 0 {
                    // Sombra en la base de cada verticilo, granulada para dar volumen de gajos.
                    if hh < 0.74 {
                        sh
                    } else {
                        mid
                    }
                } else if dy == 1 && !on_lit_side && hh < 0.55 {
                    sh
                } else if dy >= thick - 2 && (on_lit_side || f > 0.65) {
                    if hh < 0.68 {
                        hi
                    } else {
                        mid
                    }
                } else if on_lit_side && hh < 0.36 {
                    hi
                } else if !on_lit_side && hh < 0.42 {
                    sh
                } else {
                    mid
                };
                dr(fb, xx, py, 1, 1, col);
            }
        }

        // Puntas de rama erguidas y espinosas en el borde del verticilo (candelabro).
        for side in [-1i32, 1] {
            let ex = cx + side * rx;
            let ey = yy - 1;
            let tip_col = if (lit && side < 0) || (!lit && side > 0) {
                hi
            } else {
                mid
            };
            dr(fb, ex + side, ey - 1, 1, 2, mid);
            dr(fb, ex + side * 2, ey - 2, 1, 2, tip_col);
            dr(fb, ex + side, ey + 1, 1, 1, sh);
        }
    }

    // Coronilla del domo redondeada en el centro (sin "orejas") y erizada de puntas cortas.
    let top_rx = (base_half as f64 * 0.40).round().max(2.0) as i32;
    let peak_rx = (top_rx / 2).max(1);
    for dx in -top_rx..=top_rx {
        let on_lit = (lit && dx <= 0) || (!lit && dx >= 0);
        let c = if on_lit { hi } else { mid };
        dr(fb, cx + dx, apex - 1, 1, 3, c);
        if dx.abs() <= peak_rx {
            dr(fb, cx + dx, apex - 2, 1, 1, if on_lit { hi } else { mid });
        }
    }
    dr(fb, cx, apex - 3, 1, 1, hi);

    // Conos femeninos redondos sobre ramas altas.
    if h >= 26 {
        let cone_y = apex + r(span as f64 * 0.38);
        let cone_half = (base_half as f64 * 0.52).round().max(2.0) as i32;
        for (i, &sgn) in [-1i32, 1].iter().enumerate() {
            let yy = cone_y + i as i32 * (span / 5).max(1);
            let cr = if size >= 28.0 { 2 } else { 1 };
            cone_fruit(
                fb,
                cx + sgn * cone_half,
                yy,
                cr,
                get(pal, "sandD"),
                get(pal, "sand"),
            );
        }
    }
}

fn draw_bare_branches(fb: &mut Framebuffer, x: i32, top: i32, h: i32, pal: &Palette) {
    let trunk = get(pal, "trunk");
    let td = shade(trunk, -0.2);
    for (dir, len, wave) in [(-1i32, 0.82f64, 0.5f64), (0, 1.0, 0.0), (1, 0.74, -0.5)] {
        let steps = ((h as f64 * len).round() as i32).max(2);
        let mut bx = x;
        for i in 0..steps {
            let yy = top + steps - i;
            bx = x + r(dir as f64 * (i as f64 * 0.35 + (i % 2) as f64 * wave));
            dr(fb, bx, yy, 1, 1, if i % 3 == 0 { trunk } else { td });
        }
        if dir != 0 && steps > 3 {
            dr(fb, bx + dir, top + r((steps as f64) * 0.4), 1, 1, trunk);
            // Ramillas secundarias.
            let base_y = top + r(steps as f64 * 0.7);
            for k in 1..=3 {
                let yy = base_y - k * (steps / 5).max(1);
                line(fb, bx, yy, bx + dir * k, yy - k, td);
            }
        }
    }
    // Horqueta baja.
    line(fb, x, top + h - 2, x - 2, top + h - 4, td);
    line(fb, x, top + h - 3, x + 2, top + h - 5, td);
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
    let h = (size * 1.05).round().max(9.0) as i32;
    let trunk_h = (size * 0.32).round().max(3.0) as i32;
    let top = base_y - h;
    let trunk = get(pal, "trunk");
    let trunk_w = (size * 0.07).round().clamp(2.0, 4.0) as i32;
    dr(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        trunk,
    );
    dr(
        fb,
        x + (trunk_w >> 1) - 1,
        base_y - trunk_h,
        1,
        trunk_h,
        shade(trunk, -0.25),
    );
    bark_marks(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        trunk,
        x as u32,
    );
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
        // Ramas interiores y motas de hoja a mayor resolución.
        if crown_h >= 12 && i % 4 == 2 {
            let inner = (half / 2).max(1);
            dr(fb, x - inner, yy, inner * 2, 1, shade(mid, -0.18));
        }
        if crown_h >= 10 && hash1(x * 3 + yy * 5, 0xa7f) < 0.4 {
            let px = x + r((hash1(i * 17 + x, 0xa80) - 0.5) * half as f64 * 1.5);
            dr(fb, px, yy, 1, 1, if i % 3 == 0 { hi } else { sh });
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
    let h = (size * 0.72).round().max(5.0) as i32;
    let rx = (size * 0.44).round().max(2.0) as i32;
    let tone = crown_tone(pal, light);
    let lit = lit_left(light);
    // Ramillas en la base.
    for k in 0..3 {
        let dx = (k - 1) * (rx / 2);
        line(
            fb,
            x,
            base_y - 1,
            x + dx,
            base_y - h / 3,
            shade(get(pal, "trunk"), -0.1),
        );
    }
    crown_blob(fb, x, base_y - h / 2, rx, h / 2, 0x0b5, tone, lit);
    if size >= 16.0 {
        crown_blob(
            fb,
            x - rx / 2,
            base_y - h / 2 - h / 4,
            rx * 3 / 4,
            h / 3,
            0x0b6,
            tone,
            lit,
        );
        crown_blob(
            fb,
            x + rx / 2,
            base_y - h / 2 - h / 4,
            rx * 3 / 4,
            h / 3,
            0x0b7,
            tone,
            lit,
        );
    }
}

fn draw_grass(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let (hi, mid, sh) = crown_tone(pal, light);
    let lit = lit_left(light);
    let spacing = if size >= 12.0 { 2 } else { 1 };
    let n = (3 + (size * 0.45) as i32).clamp(3, 8);
    for i in 0..n {
        let gx = x + (i - (n >> 1)) * spacing;
        let h = (size * 0.55).round().max(3.0) as i32 + (i * 7 + x).rem_euclid(2);
        let c = if (gx < x) == lit { hi } else { mid };
        dr(fb, gx, base_y - h, 1, h, c);
        // Punta inclinada de la brizna.
        let tip = if i % 2 == 0 { 1 } else { -1 };
        dr(fb, gx + tip, base_y - h - 1, 1, 1, c);
        if size >= 18.0 && i % 2 == 0 {
            // Espiga clara.
            dr(fb, gx + tip, base_y - h - 2, 1, 1, hi);
            dr(fb, gx, base_y - h - 1, 1, 1, sh);
        }
    }
}

fn draw_crop(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let (hi, mid, _sh) = crown_tone(pal, light);
    let w = (size * 0.95).round().max(5.0) as i32;
    let h = (size * 0.18).round().max(2.0) as i32;
    // Franja de tierra y surcos.
    dr(fb, x - (w >> 1), base_y - h, w, h, get(pal, "valleyD"));
    let soil = shade(get(pal, "valleyD"), -0.12);
    let mut i = 1;
    while i < w {
        dr(fb, x - (w >> 1) + i, base_y - h, 1, h, soil);
        i += 3;
    }
    // Hileras de espigas separadas, con grano claro.
    let step = if size >= 18.0 { 4 } else { 3 };
    let mut i = 0;
    while i < w {
        let gh = 2 + (i * 7 + x).rem_euclid(2) + if size >= 18.0 { 1 } else { 0 };
        let c = if i % 4 == 0 { hi } else { mid };
        dr(fb, x - (w >> 1) + i, base_y - h - gh, 1, gh, c);
        dr(
            fb,
            x - (w >> 1) + i,
            base_y - h - gh - 1,
            1,
            1,
            FLOWER_COLORS[1],
        );
        if step > 3 {
            dr(
                fb,
                x - (w >> 1) + i + 1,
                base_y - h - gh + 1,
                1,
                gh - 1,
                mid,
            );
            dr(fb, x - (w >> 1) + i + 1, base_y - h - gh, 1, 1, hi);
        }
        i += step;
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
    // Grietas y facetas.
    if size >= 16.0 {
        let crack = shade(rkd, -0.2);
        let mut yy = base_y - h + 1;
        let mut xx = x - w / 4;
        while yy < base_y - 1 {
            dr(fb, xx, yy, 1, 1, crack);
            xx += if (yy + x) % 2 == 0 { 1 } else { -1 };
            yy += 1;
        }
        dr(fb, x + w / 3, base_y - h / 2, 1, h / 2, shade(rk, 0.1));
    }
    // Piedritas sueltas al pie.
    dr(fb, x - w / 2 - 2, base_y - 1, 1, 1, rkd);
    dr(fb, x + w / 2 + 1, base_y - 1, 1, 1, rkd);
    if size >= 20.0 {
        dr(fb, x - w / 2 - 3, base_y - 1, 2, 1, rkd);
    }
}

fn draw_cactus(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 1.15).round().max(10.0) as i32;
    let top = base_y - h;
    let w = (size * 0.46).round().max(4.0) as i32;
    let half = w / 2;
    // Verde glauco del copao: la paleta de bosque se apaga hacia el gris de roca.
    let (hi0, mid0, sh0) = crown_tone(pal, light);
    let haze = get(pal, "rock");
    let hi = lerp_color(hi0, haze, 0.55);
    let mid = lerp_color(mid0, haze, 0.50);
    let sh = shade(lerp_color(sh0, haze, 0.35), -0.05);
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
        // Costillas internas.
        if w >= 6 && rr % 2 == 0 {
            dr(fb, x - 1, yy, 1, 1, shade(mid, -0.12));
            dr(fb, x + 1, yy, 1, 1, shade(mid, -0.12));
        }
        // Areolas y espinas.
        if rr % 3 == 0 {
            dr(fb, x - half - 1, yy, 1, 1, sand_d);
            dr(fb, x + half + 1, yy, 1, 1, sand_d);
            if w >= 6 {
                dr(fb, x - half + 2, yy, 1, 1, hi);
            }
        }
    }

    // Brazos en candelabro (1-2) que suben doblados.
    let arm_h = (h as f64 * 0.42).round().max(4.0) as i32;
    let dir = if hash1(x, 0xcac) < 0.5 { -1 } else { 1 };
    let arm_y = top + r(h as f64 * 0.5);
    let ax = x + dir * (half + 1);
    dr(fb, x.min(ax), arm_y, (ax - x).abs() + 1, 2, mid);
    for rr in 0..=arm_h {
        let yy = arm_y - rr;
        let aw = if size >= 20.0 { 2 } else { 1 };
        dr(fb, ax - aw, yy, aw * 2, 1, mid);
        dr(fb, if lit { ax - aw } else { ax + aw - 1 }, yy, 1, 1, hi);
        if rr % 3 == 0 {
            dr(
                fb,
                ax + if dir < 0 { -aw - 1 } else { aw + 1 },
                yy,
                1,
                1,
                sand_d,
            );
        }
    }
    if hash1(x, 0xca5) < 0.7 {
        let a_y = top + r(h as f64 * 0.68);
        let a2 = x + (-dir) * (half + 1);
        dr(fb, x.min(a2), a_y, (a2 - x).abs() + 1, 2, mid);
        let aw = if size >= 20.0 { 2 } else { 1 };
        for rr in 0..=(arm_h / 2).max(2) {
            dr(fb, a2 - aw, a_y - rr, aw * 2, 1, mid);
        }
    }

    // Flor en el ápice: el copao florece blanco-rosado cerca de la punta.
    dr(fb, x, top - 1, 1, 1, [0xf4, 0xf0, 0xe6]);
    dr(fb, x - 1, top - 2, 1, 1, [0xf2, 0xa0, 0xb8]);
    dr(fb, x + 1, top - 2, 1, 1, [0xf2, 0xa0, 0xb8]);
    dr(fb, x, top - 2, 1, 1, [0xf7, 0xd9, 0x70]);
    if size >= 20.0 {
        dr(fb, x - 2, top - 1, 1, 1, [0xf2, 0xa0, 0xb8]);
        dr(fb, x + 2, top - 1, 1, 1, [0xf2, 0xa0, 0xb8]);
    }
}

fn draw_alerce(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 1.12).round().max(12.0) as i32;
    let trunk_h = (size * 0.2).round().max(3.0) as i32;
    let top = base_y - h;
    // Corteza rojiza y fibrosa del alerce.
    let trunk = lerp_color(get(pal, "trunk"), [0x7a, 0x46, 0x38], 0.30);
    let trunk_w = (size * 0.08).round().clamp(2.0, 4.0) as i32;
    dr(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        trunk,
    );
    dr(
        fb,
        x + (trunk_w >> 1) - 1,
        base_y - trunk_h,
        1,
        trunk_h,
        shade(trunk, -0.25),
    );
    bark_marks(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        trunk,
        x as u32,
    );
    let n = h - trunk_h;
    let tone = crown_tone(pal, light);
    let lit = lit_left(light);
    // Copa columnar muy estrecha, no un cono ancho.
    let half_max = (size * 0.21).round().max(2.0) as i32;
    crown_cone(fb, x, top, n, half_max, 0xa1e, tone, lit, true);
    // Ramillas colgantes en los bordes y ramas interiores.
    let (hi, mid, sh) = tone;
    for i in (1..n).step_by(2) {
        let t = (i + 1) as f64 / n as f64;
        let half = (half_max as f64 * t.powf(0.82)).round().max(1.0) as i32;
        let yy = top + i;
        let droop = 1 + (n / 18).clamp(0, 2);
        dr(fb, x - half - 1, yy + 1, 1, droop, sh);
        dr(fb, x + half + 1, yy + 1, 1, droop, sh);
        if i % 4 == 0 {
            let inner = (half / 2).max(1);
            dr(fb, x - inner, yy, 1, 1, if lit { hi } else { sh });
            dr(fb, x + inner, yy, 1, 1, if lit { sh } else { hi });
        }
    }
    dr(fb, x - 1, top - 1, 3, 1, mid);
    dr(fb, x, top - 2, 1, 1, hi);
}

/// Nalca: roseta de hojas orbiculares gigantes de borde dentado sobre pecíolos
/// gruesos ("gran paraguas de hojas", ver `porte`).
fn draw_nalca(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let (hi, mid, sh) = crown_tone(pal, light);
    let lit = lit_left(light);
    let trunk = get(pal, "trunk");
    let n = (3 + (size * 0.16) as i32).clamp(3, 8);
    for i in 0..n {
        let hh = hash1(i * 13 + x, 0xaa11);
        let lx = x + r((hh - 0.5) * size * 1.0);
        let stem_h = (size * (0.3 + hh * 0.4)).round().max(4.0) as i32;
        let lr = (size * (0.36 + hh * 0.16)).round().max(3.0) as i32;
        let top = base_y - stem_h;
        // Pecíolo grueso, con base ensanchada.
        let pw = if size >= 20.0 { 2 } else { 1 };
        dr(fb, lx - pw, top, pw * 2, stem_h, trunk);
        dr(fb, lx + pw - 1, top, 1, stem_h, shade(trunk, -0.2));
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
            // Dientes del borde.
            if size >= 16.0 && dy % 2 == 0 {
                dr(fb, lx - span - 1, yy, 1, 1, sh);
                dr(fb, lx + span + 1, yy, 1, 1, sh);
            }
        }
        // Nervadura central y nervios laterales.
        dr(fb, lx, cy - lr + 1, 1, lr * 2 - 1, sh);
        if lr >= 5 {
            for j in 1..=3 {
                let vy = cy - lr + j * lr / 2;
                let half = (lr as f64
                    * (1.0 - ((vy - cy) as f64 / lr as f64).powi(2))
                        .max(0.0)
                        .sqrt()) as i32;
                dr(fb, lx - half + 1, vy, half - 1, 1, shade(sh, 0.1));
                dr(fb, lx + 1, vy, half - 1, 1, shade(sh, 0.1));
            }
        }
    }
    // Espiga floral rojiza que emerge entre las hojas (nalca en flor).
    let spike_h = (size * 0.9).round().max(5.0) as i32;
    let sx = x + r((hash1(x, 0xaa13) - 0.5) * size * 0.35);
    for j in 0..spike_h {
        let c = if j % 3 == 0 {
            [0x8e, 0x4a, 0x32]
        } else {
            [0x6a, 0x3a, 0x28]
        };
        dr(fb, sx, base_y - j, 1, 1, c);
    }
    for j in (spike_h / 3..spike_h).step_by(2) {
        dr(fb, sx - 1, base_y - j, 1, 1, [0xa8, 0x6a, 0x4a]);
        dr(fb, sx + 1, base_y - j, 1, 1, [0xa8, 0x6a, 0x4a]);
    }
    dr(fb, sx, base_y - spike_h - 1, 1, 1, [0xb8, 0x7a, 0x54]);
}

fn draw_colihue(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let (hi, mid, sh) = crown_tone(pal, light);
    let lit = lit_left(light);
    let n = (5 + (size * 0.3) as i32).clamp(5, 12);
    let cane_w = if size >= 22.0 { 2 } else { 1 };
    for i in 0..n {
        let hh = hash1(i * 17 + x, 0xc01);
        let cxp = x + r((i as f64 - (n as f64 - 1.0) / 2.0) * 0.9);
        let ch = (size * (0.8 + hh * 0.55)).round().max(6.0) as i32;
        let top = base_y - ch;
        // Caña con nudos.
        for k in 0..ch {
            let c = if lit && k % 2 == 0 { hi } else { mid };
            dr(fb, cxp, top + k, cane_w, 1, c);
        }
        let mut k = 2;
        while k < ch {
            dr(fb, cxp, base_y - k, cane_w, 1, sh);
            k += 3;
        }
        // Penacho de hojas en la punta.
        let dir = if lit { -1 } else { 1 };
        for j in 0..(4 + (size * 0.08) as i32) {
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
            if size >= 18.0 && j % 2 == 0 {
                dr(fb, cxp + d * 2, ly, 1, 1, hi);
            }
        }
    }
}

fn draw_palma(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let trunk_h = (size * 0.78).round().max(7.0) as i32;
    let top = base_y - trunk_h;
    // Tronco grueso grisáceo-pardo de la palma chilena (Jubaea chilensis),
    // ligeramente estrechado en el cuello superior y con cicatrices romboidales.
    let trunk = lerp_color(get(pal, "trunk"), get(pal, "rock"), 0.38);
    let td = shade(trunk, -0.22);
    let tl = shade(trunk, 0.14);
    let base_w = (size * 0.13).round().clamp(2.0, 6.0) as i32;
    let neck_w = (base_w - 1).max(2);
    let (hi, mid, sh) = crown_tone(pal, light);
    let lit = lit_left(light);

    for i in 0..trunk_h {
        let yy = top + i;
        let w = if i < trunk_h / 4 && base_w >= 4 {
            neck_w
        } else {
            base_w
        };
        let x0 = x - (w >> 1);
        dr(fb, x0, yy, w, 1, trunk);
        let sh_x = if lit { x0 + w - 1 } else { x0 };
        let hi_x = if lit { x0 } else { x0 + w - 1 };
        dr(fb, sh_x, yy, 1, 1, td);
        if w >= 3 {
            dr(fb, hi_x, yy, 1, 1, tl);
        }
        // Cicatrices romboidales de hojas antiguas.
        if i > 2 && i % 3 == 0 {
            dr(fb, x0, yy, w, 1, td);
            if w >= 3 {
                let cx_mark = x0 + 1 + ((i / 3) % (w - 1).max(1));
                dr(fb, cx_mark, yy - 1, 1, 1, tl);
            }
        }
    }

    // Faldón de frondas secas ocre-pardo colgando bajo la corona.
    let dry_l = lerp_color(get(pal, "sandD"), trunk, 0.35);
    let dry_d = shade(dry_l, -0.24);
    let skirt_h = (size * 0.13).round().clamp(2.0, 5.0) as i32;
    let skirt_rx = (neck_w / 2 + 2).max(2);
    for dx in -skirt_rx..=skirt_rx {
        let len = (skirt_h - dx.abs() / 2).max(1);
        let col = if (lit && dx <= 0) || (!lit && dx >= 0) {
            if dx.abs() % 2 == 0 {
                dry_l
            } else {
                dry_d
            }
        } else {
            dry_d
        };
        dr(fb, x + dx, top + 1, 1, len, col);
    }

    // Penacho radial simétrico en pares de frondas laterales + frondas en escorzo
    // que coronan el domo central sin dejar una muesca en V.
    let rr = (size * 0.44).round().max(4.0) as i32;
    let pairs = (4 + (size * 0.08) as i32).clamp(4, 7);
    for p in 0..pairs {
        let u = (p as f64 + 0.45) / pairs as f64; // ~0.10 .. 0.95
        let len = (rr as f64 * (0.84 + 0.16 * (1.0 - (u - 0.5).powi(2) * 1.4)))
            .round()
            .max(3.0) as i32;
        for &sgn in &[-1.0f64, 1.0] {
            let theta = sgn * u * 1.45;
            let side_lit = (lit && sgn < 0.0) || (!lit && sgn > 0.0);
            frond(fb, x, top, theta, 1.0, len, mid, hi, sh, side_lit);
        }
    }
    // Frondas centrales en escorzo (hacia/desde el observador) que cierran la cúpula apical.
    let top_len = (rr as f64 * 0.92).round().max(3.0) as i32;
    for &(xs, th) in &[(0.28f64, 0.22f64), (0.55, 0.34)] {
        for &sgn in &[-1.0f64, 1.0] {
            let side_lit = (lit && sgn < 0.0) || (!lit && sgn > 0.0);
            frond(fb, x, top, sgn * th, xs, top_len, mid, hi, sh, side_lit);
        }
    }
}

fn draw_coihue(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 1.12).round().max(10.0) as i32;
    let trunk_h = (size * 0.42).round().max(4.0) as i32;
    let top = base_y - h;
    let trunk = get(pal, "trunk");
    let td = shade(trunk, -0.25);
    let tl = shade(trunk, 0.12);
    let crown_h = h - trunk_h;
    let trunk_w = (size * 0.09).round().clamp(2.0, 5.0) as i32;

    // Tronco recto, grueso y rugoso, con horqueta que abre hacia la copa.
    dr(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        trunk,
    );
    dr(fb, x + (trunk_w >> 1) - 1, base_y - trunk_h, 1, trunk_h, td);
    if trunk_w > 2 {
        dr(fb, x - (trunk_w >> 1) + 1, base_y - trunk_h, 1, trunk_h, tl);
    }
    bark_marks(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        trunk,
        x as u32,
    );
    let fork = base_y - trunk_h;
    line(fb, x, fork, x - 3, fork - 2, td);
    line(fb, x + 1, fork, x + 4, fork - 2, td);
    line(fb, x - 3, fork - 2, x - 5, fork - 4, td);
    line(fb, x + 4, fork - 2, x + 6, fork - 4, td);
    if size >= 18.0 {
        line(fb, x - 5, fork - 4, x - 8, fork - 6, td);
        line(fb, x + 6, fork - 4, x + 9, fork - 6, td);
    }

    let tone = crown_tone(pal, light);
    let lit = lit_left(light);
    let rx = (size * 0.52).round().max(2.0) as i32;
    let cy = top + crown_h / 2;
    // Copa ancha: tres lóbulos solapados con ramas internas.
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
    let (_, mid, sh) = tone;
    for k in 0..3 {
        let dx = (k - 1) * (rx / 2);
        let y0 = base_y - trunk_h - 2;
        let y1 = cy - crown_h / 5;
        line(
            fb,
            x + dx / 2,
            y0,
            x + dx,
            y1,
            if k == 1 { shade(mid, -0.2) } else { sh },
        );
    }
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
    let h = (size * 1.18).round().max(9.0) as i32;
    let trunk_h = (size * 0.3).round().max(3.0) as i32;
    let top = base_y - h;
    let trunk = get(pal, "trunk");
    let trunk_w = (size * 0.07).round().clamp(2.0, 4.0) as i32;
    dr(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        trunk,
    );
    dr(
        fb,
        x + (trunk_w >> 1) - 1,
        base_y - trunk_h,
        1,
        trunk_h,
        shade(trunk, -0.25),
    );
    bark_marks(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        trunk,
        x as u32,
    );
    let crown_h = h - trunk_h;
    if season == 2 {
        draw_bare_branches(fb, x, top, crown_h, pal);
        return;
    }
    let (hi, mid, sh) = crown_tone(pal, light);
    let lit = lit_left(light);
    let half_max = (size * 0.32).round().max(2.0) as i32;

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
        if crown_h >= 12 && i % 4 == 2 {
            let inner = (half / 2).max(1);
            dr(fb, x - inner, yy, inner * 2, 1, shade(mid, -0.18));
        }
        if crown_h >= 10 && hash1(x * 5 + yy * 3, 0xb0c) < 0.38 {
            let px = x + r((hash1(i * 19 + x, 0xb0d) - 0.5) * half as f64 * 1.5);
            dr(fb, px, yy, 1, 1, if i % 3 == 0 { sh } else { hi });
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
    let h = (size * 1.15).round().max(9.0) as i32;
    let top = base_y - h;
    let trunk = get(pal, "trunk");
    let (hi, mid, sh) = crown_tone(pal, light);
    let lit = lit_left(light);

    let stem_x = |yy: i32| -> i32 {
        let u = (base_y - yy) as f64;
        x + r((u * 0.36).sin() * 2.2)
    };

    // Tallo trepador sinuoso con guía principal y ramillas de soporte.
    let mut prev = stem_x(base_y);
    for i in 0..=h {
        let yy = base_y - i;
        let cx = stem_x(yy);
        let col = if i % 4 == 0 { trunk } else { sh };
        line(fb, prev, (yy + 1).min(base_y), cx, yy, col);
        if i > 1 && i < h {
            dr(fb, cx + if lit { -1 } else { 1 }, yy, 1, 1, mid);
        }
        prev = cx;
    }

    // Nudos de hojas coriáceas acorazonadas/ovadas a ambos lados del tallo.
    let nodes = (h / 4).clamp(3, 8);
    for i in 0..nodes {
        let ny = top + 2 + i * (h - 3).max(1) / nodes.max(1);
        let sx = stem_x(ny);
        let dir = if i % 2 == 0 { -1 } else { 1 };
        let lcol = if (lit && dir < 0) || (!lit && dir > 0) {
            hi
        } else {
            mid
        };
        dr(fb, sx + dir, ny, 2, 1, lcol);
        dr(fb, sx + dir * 2, ny + 1, 2, 1, sh);
        if size >= 16.0 {
            dr(fb, sx + dir * 2, ny - 1, 2, 1, hi);
            dr(fb, sx - dir, ny + 1, 2, 1, mid);
        }
    }

    if season != 0 && season != 3 {
        return;
    }

    // Campanas rojas colgantes (Flor Nacional): separadas en nudos alternos,
    // con cáliz estrecho arriba y boca acampanada abajo.
    let red_hi = [0xf0, 0x50, 0x64];
    let red_m = [0xd4, 0x20, 0x38];
    let red_d = [0x9e, 0x12, 0x24];
    let tip = [0xf8, 0x98, 0xa4];

    let n_bells = if h < 14 {
        2
    } else if h < 24 {
        3
    } else {
        4
    };
    let top_y = top + 2;
    let bot_y = (base_y - 7).max(top_y + 2);
    let span = (bot_y - top_y).max(2);

    let draw_bell = |fb: &mut Framebuffer, bx: i32, ny: i32, big: bool| {
        let left_c = if lit { red_hi } else { red_d };
        let right_c = if lit { red_d } else { red_hi };
        // Cáliz verde en el ápice de la flor.
        dr(fb, bx, ny - 1, 1, 1, mid);
        // Cuello estrecho de la campana (1 px).
        dr(fb, bx, ny, 1, 1, red_m);
        if big {
            // Tubo simétrico (3 px) y boca acampanada con tres puntas de tépalo.
            for dy in 1..=2 {
                dr(fb, bx - 1, ny + dy, 1, 1, left_c);
                dr(fb, bx, ny + dy, 1, 1, red_m);
                dr(fb, bx + 1, ny + dy, 1, 1, right_c);
            }
            dr(fb, bx - 1, ny + 3, 1, 1, left_c);
            dr(fb, bx, ny + 3, 1, 1, tip);
            dr(fb, bx + 1, ny + 3, 1, 1, right_c);
            // Faldón abierto (puntas laterales + estambre central).
            dr(fb, bx - 2, ny + 4, 1, 1, left_c);
            dr(fb, bx - 1, ny + 4, 1, 1, red_m);
            dr(fb, bx, ny + 4, 1, 1, tip);
            dr(fb, bx + 1, ny + 4, 1, 1, red_d);
            dr(fb, bx + 2, ny + 4, 1, 1, right_c);
        } else {
            dr(fb, bx, ny + 1, 1, 1, left_c);
            dr(fb, bx - 1, ny + 2, 1, 1, left_c);
            dr(fb, bx, ny + 2, 1, 1, red_m);
            dr(fb, bx + 1, ny + 2, 1, 1, right_c);
            dr(fb, bx - 1, ny + 3, 1, 1, red_m);
            dr(fb, bx, ny + 3, 1, 1, tip);
            dr(fb, bx + 1, ny + 3, 1, 1, red_d);
        }
    };

    for i in 0..n_bells {
        let ny = top_y + i * span / (n_bells - 1).max(1);
        let sx = stem_x(ny);
        let dir = if i % 2 == 0 { 1 } else { -1 };
        let dist = if size >= 16.0 { 4 } else { 3 };
        let bx = sx + dir * dist;

        line(fb, sx, ny - 1, bx, ny - 1, sh);
        draw_bell(fb, bx, ny, size >= 16.0);

        // En tallas grandes el nudo lleva un par de campanas («una o dos por nudo»)
        // colgando hacia el mismo lado exterior para no chocar con el tallo.
        if size >= 26.0 && i == 1 {
            let bx2 = bx + dir * 4;
            line(fb, bx, ny - 1, bx2, ny, sh);
            draw_bell(fb, bx2, ny + 1, false);
        }
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
    let h = (size * 0.86).round().max(6.0) as i32;
    let rx = (size * 0.52).round().max(2.0) as i32;
    let tone = crown_tone(pal, light);
    let (hi, _mid, sh) = tone;
    let lit = lit_left(light);
    // Tallos leñosos y espinosos.
    for k in 0..3 {
        let dx = (k - 1) * (rx / 2);
        line(
            fb,
            x,
            base_y - 1,
            x + dx,
            base_y - h / 2,
            shade(get(pal, "trunk"), -0.1),
        );
        if size >= 16.0 {
            dr(fb, x + dx / 2, base_y - h / 3, 1, 1, hi);
        }
    }
    // Espinas trífidas del michay en las puntas de las ramas.
    let thorn = lerp_color(get(pal, "trunk"), get(pal, "sand"), 0.45);
    for k in 0..3 {
        let bx = x + (k - 1) * (rx / 2);
        let by = base_y - h / 3;
        dr(fb, bx, by, 1, 1, thorn);
        dr(fb, bx + 1, by - 1, 1, 1, thorn);
        dr(fb, bx - 1, by - 1, 1, 1, thorn);
        if size >= 18.0 {
            dr(fb, bx, by - 2, 1, 1, thorn);
        }
    }
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
    if season == 1 {
        // Bayas azuladas en otoño.
        for i in 0..(2 + (size * 0.2) as i32) {
            let hh = hash1(i * 61 + x, 0xb017);
            dr(
                fb,
                x + r((hh - 0.5) * rx as f64 * 1.7),
                base_y - h + r(hash1(i, 0xb018) * h as f64),
                1,
                1,
                [0x6a, 0x76, 0xa8],
            );
        }
        return;
    }
    if season != 0 && season != 3 {
        return;
    }
    // Flores naranjas en primavera/verano.
    let n = 4 + (size * 0.35) as i32;
    for i in 0..n {
        let hh = hash1(i * 23 + x, 0xb012);
        let fx = x + r((hh - 0.5) * rx as f64 * 1.9);
        let fy = base_y - h + r(hash1(i, 0xb014) * h as f64);
        dr(fb, fx, fy, 1, 1, [0xe8, 0x91, 0x2a]);
        dr(fb, fx + 1, fy, 1, 1, [0xf6, 0xc0, 0x4a]);
        if size >= 18.0 {
            dr(fb, fx, fy + 1, 1, 1, sh);
        }
    }
}

fn draw_chaura(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 0.78).round().max(5.0) as i32;
    let rx = (size * 0.58).round().max(2.0) as i32;
    let tone = crown_tone(pal, light);
    let (hi, mid, sh) = tone;
    // Tallos rastreros.
    for k in 0..3 {
        let dx = (k - 1) * (rx / 2);
        line(
            fb,
            x,
            base_y - 1,
            x + dx,
            base_y - h / 3,
            shade(get(pal, "trunk"), -0.15),
        );
    }
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
    let n = 4 + (size * 0.3) as i32;
    for i in 0..n {
        let hh = hash1(i * 29 + x, 0x0c5);
        let fx = x + r((hh - 0.5) * rx as f64 * 1.7);
        let fy = base_y - h / 2 + r((hash1(i, 0x0c6) - 0.5) * h as f64);
        dr(fb, fx, fy, 1, 1, [0xf2, 0xd8, 0xe8]);
        dr(fb, fx, fy + 1, 1, 1, [0xd9, 0x7a, 0x9a]);
        if size >= 18.0 {
            dr(fb, fx + 1, fy, 1, 1, if i % 2 == 0 { hi } else { sh });
            dr(fb, fx, fy - 1, 1, 1, mid);
        }
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
    let h = (size * 1.0).round().max(8.0) as i32;
    let trunk_h = (size * 0.34).round().max(3.0) as i32;
    let top = base_y - h;
    // Corteza pálida del quillay.
    let trunk = lerp_color(get(pal, "trunk"), get(pal, "sand"), 0.45);
    let trunk_w = (size * 0.08).round().clamp(2.0, 4.0) as i32;
    dr(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        trunk,
    );
    dr(
        fb,
        x + (trunk_w >> 1) - 1,
        base_y - trunk_h,
        1,
        trunk_h,
        shade(trunk, -0.18),
    );
    bark_marks(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        trunk,
        x as u32,
    );
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
    let rx = (crown_h / 2).max(2);
    // Copa globosa: casi esférica, de borde irregular.
    crown_blob(fb, x, top + crown_h / 2, rx, rx, 0x9111, tone, lit);
    let (hi, mid, _sh) = tone;
    for k in 0..2 {
        let dx = (k * 2 - 1) * (rx / 2);
        line(
            fb,
            x + dx / 2,
            base_y - trunk_h - 2,
            x + dx,
            top + crown_h / 2,
            shade(mid, -0.18),
        );
    }
    if season == 0 || season == 3 {
        let n = 2 + (size * 0.18).floor() as i32;
        for i in 0..n {
            let hh = hash1(i * 37 + x, 0x9114);
            let fy = top + 1 + r(((i + 1) as f64 / (n + 1) as f64) * crown_h as f64);
            let fx = x + r((hh - 0.5) * rx as f64 * 1.6);
            dr(fb, fx, fy, 1, 1, [0xf4, 0xf0, 0xe6]);
            if size >= 18.0 {
                dr(fb, fx + 1, fy, 1, 1, hi);
            }
        }
    }
}

fn draw_manio(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 1.08).round().max(10.0) as i32;
    let trunk_h = (size * 0.18).round().max(3.0) as i32;
    let top = base_y - h;
    let trunk = get(pal, "trunk");
    let trunk_w = (size * 0.07).round().clamp(2.0, 4.0) as i32;
    dr(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        trunk,
    );
    dr(
        fb,
        x + (trunk_w >> 1) - 1,
        base_y - trunk_h,
        1,
        trunk_h,
        shade(trunk, -0.25),
    );
    bark_marks(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        trunk,
        x as u32,
    );
    let n = h - trunk_h;
    let (hi, mid, sh) = crown_tone(pal, light);
    // Tono más oscuro (conífera austral).
    let tone = (shade(hi, -0.06), shade(mid, -0.10), shade(sh, -0.10));
    let lit = lit_left(light);
    let half_max = (size * 0.22).round().max(2.0) as i32;
    crown_cone(fb, x, top, n, half_max, 0x3a17, tone, lit, true);
    // Ramas colgantes: puntitos que cuelgan del borde cada pocas filas.
    let (hi2, mid2, sh2) = tone;
    for i in (1..n).step_by(2) {
        let t = (i + 1) as f64 / n as f64;
        let half = (half_max as f64 * t.powf(1.35)).round().max(1.0) as i32;
        let yy = top + i;
        let droop = 2 + (n / 14).clamp(0, 3);
        dr(fb, x - half - 1, yy + 1, 1, droop, shade(mid2, -0.12));
        dr(fb, x + half + 1, yy + 1, 1, droop, shade(mid2, -0.12));
        if i % 5 == 0 {
            dr(fb, x - half, yy, 1, 1, hi2);
            dr(fb, x + half, yy, 1, 1, sh2);
        }
    }
    dr(fb, x, top - 1, 1, 1, hi2);
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
    let h = (size * 1.06).round().max(10.0) as i32;
    let trunk_h = (size * 0.25).round().max(3.0) as i32;
    let top = base_y - h;
    // Corteza lisa pardo-rojiza con lenticelas claras del canelo.
    let bark = [0x7a, 0x48, 0x3a];
    let bark_d = shade(bark, -0.22);
    let bark_l = shade(bark, 0.16);
    let trunk_w = (size * 0.09).round().clamp(2.0, 4.0) as i32;
    let trunk_top = base_y - trunk_h - 2;
    dr(
        fb,
        x - (trunk_w >> 1),
        trunk_top,
        trunk_w,
        trunk_h + 2,
        bark,
    );
    dr(
        fb,
        x + (trunk_w >> 1) - 1,
        trunk_top,
        1,
        trunk_h + 2,
        bark_d,
    );
    if trunk_w > 2 {
        dr(fb, x - (trunk_w >> 1), trunk_top, 1, trunk_h + 2, bark_l);
    }
    bark_marks(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        bark,
        x as u32,
    );

    let crown_h = (h - trunk_h).max(6);
    let (hi0, mid, sh) = crown_tone(pal, light);
    // El canelo destaca por sus hojas grandes de verde brillante (envés glauco/claro).
    let hi = lerp_color(hi0, get(pal, "snow"), 0.14);
    let lit = lit_left(light);
    let max_rx = (size * 0.40).round().max(3.0) as i32;

    // Silueta piramidal-redondeada (ovada-cónica): base redondeada que alcanza su
    // ancho máximo en el tercio inferior y se afila suavemente hasta el ápice,
    // sin piezas sueltas.
    let span_at = |dy: i32| -> i32 {
        let u = dy as f64 / (crown_h - 1).max(1) as f64; // 0 = ápice, 1 = base de copa
        let prof = if u < 0.68 {
            // Cono convexo suave desde el ápice hasta el tercio inferior.
            (u / 0.68).powf(0.74)
        } else {
            // Base redondeada hacia el tronco.
            let v = (u - 0.68) / 0.32;
            (1.0 - v * v * 0.72).max(0.18).sqrt()
        };
        let mut s = (max_rx as f64 * prof).round() as i32;
        // Pequeñas muescas entre pisos ascendentes de follaje para evitar borde liso.
        if dy > 2
            && dy < crown_h - 2
            && hash1((x as u32).wrapping_add(dy as u32 * 31) as i32, 0xc0a) < 0.25
        {
            s = (s - 1).max(1);
        }
        s
    };

    for dy in 0..crown_h {
        let yy = top + dy;
        let span = span_at(dy);
        let u = dy as f64 / (crown_h - 1).max(1) as f64;
        for dx in -span..=span {
            let xx = x + dx;
            let on_lit = if lit { dx <= 0 } else { dx >= 0 };
            let hh = hash1(xx * 17 + yy * 23, 0xc0b);
            // Gajos ascendentes de hojas brillantes.
            let tier_band = (dy + (dx.abs() / 2)) % 4 == 0;
            let col = if u > 0.84 || (!on_lit && dx.abs() > span / 2 && hh < 0.65) {
                sh
            } else if on_lit && (dx.abs() >= span - 1 || (tier_band && hh < 0.70) || hh < 0.38) {
                hi
            } else if !on_lit && hh < 0.42 {
                sh
            } else {
                mid
            };
            dr(fb, xx, yy, 1, 1, col);
        }
    }

    // Ramas interiores oscuras visibles en la base de la copa en tallas medianas/grandes.
    if size >= 18.0 {
        for &sgn in &[-1i32, 1] {
            line(
                fb,
                x,
                base_y - trunk_h - 1,
                x + sgn * (max_rx / 2).max(1),
                top + crown_h * 2 / 3,
                shade(mid, -0.22),
            );
        }
    }

    // Flores blancas estrelladas en corimbos (primavera/verano), siempre dentro de la copa.
    if season == 0 || season == 3 {
        let petal = [0xf6, 0xf4, 0xec];
        let center = [0xe8, 0xc8, 0x56];
        let n = (2 + (size * 0.18).floor() as i32).clamp(2, 8);
        for i in 0..n {
            let hy = hash1(i * 53 + x * 7, 0xc0e);
            let hx = hash1(i * 97 + x * 13, 0xc0f);
            let dy = 2 + r(hy * (crown_h - 4).max(1) as f64);
            let avail = (span_at(dy) - 1).max(0);
            let fx = x + r((hx * 2.0 - 1.0) * avail as f64);
            let fy = top + dy;
            dr(fb, fx, fy, 1, 1, petal);
            if size >= 18.0 && i % 2 == 0 && avail >= 2 {
                dr(fb, fx + 1, fy, 1, 1, petal);
                dr(fb, fx, fy + 1, 1, 1, center);
            }
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
    let h = (size * 0.9).round().max(8.0) as i32;
    let trunk_h = (size * 0.42).round().max(4.0) as i32;
    let top = base_y - h;
    let bark = [0x8a, 0x4a, 0x3a];
    let trunk_w = (size * 0.08).round().clamp(2.0, 4.0) as i32;
    // Tronco canela, alto y visible (sello del arrayán).
    dr(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        bark,
    );
    dr(
        fb,
        x + (trunk_w >> 1) - 1,
        base_y - trunk_h,
        1,
        trunk_h,
        shade(bark, -0.2),
    );
    if size >= 20.0 {
        dr(
            fb,
            x - (trunk_w >> 1) - 2,
            base_y - 2,
            2,
            2,
            shade(bark, -0.15),
        );
    }
    bark_marks(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        bark,
        x as u32,
    );
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
    let (hi, mid, sh) = tone;
    let lit = lit_left(light);
    let rx = (size * 0.38).round().max(2.0) as i32;
    crown_blob(fb, x, top + crown_h / 2, rx, crown_h / 2, 0xa77a, tone, lit);
    for k in 0..2 {
        let dx = (k * 2 - 1) * (rx / 2);
        line(
            fb,
            x + dx / 2,
            base_y - trunk_h - 1,
            x + dx,
            top + crown_h / 2,
            shade(mid, -0.18),
        );
    }
    if season == 0 || season == 3 {
        let n = 2 + (size * 0.18).floor() as i32;
        for i in 0..n {
            let hh = hash1(i * 43 + x, 0xa77b);
            let fx = x + r((hh - 0.5) * rx as f64 * 1.6);
            let fy = top + 1 + (i % crown_h.max(1));
            dr(fb, fx, fy, 1, 1, [0xf4, 0xf0, 0xe6]);
            if size >= 20.0 {
                dr(fb, fx, fy + 1, 1, 1, if i % 2 == 0 { hi } else { sh });
            }
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
    let h = (size * 0.95).round().max(8.0) as i32;
    let trunk_h = (size * 0.3).round().max(3.0) as i32;
    let top = base_y - h;
    let trunk = get(pal, "trunk");
    let trunk_w = (size * 0.06).round().clamp(2.0, 3.0) as i32;
    dr(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        trunk,
    );
    bark_marks(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        trunk,
        x as u32,
    );
    let crown_h = h - trunk_h;
    if crown_h < 1 {
        return;
    }
    let tone = crown_tone(pal, light);
    let (hi, mid, sh) = tone;
    let lit = lit_left(light);
    let rx = (size * 0.32).round().max(2.0) as i32;
    crown_blob(fb, x, top + crown_h / 2, rx, crown_h / 2, 0x07e0, tone, lit);
    for k in 0..2 {
        let dx = (k * 2 - 1) * (rx / 2);
        line(
            fb,
            x + dx / 3,
            base_y - trunk_h - 1,
            x + dx,
            top + crown_h / 2,
            shade(mid, -0.2),
        );
    }
    if season != 0 && season != 3 {
        return;
    }
    // Racimos de flores escarlata (sello del notro).
    let n = 3 + (size * 0.3).floor() as i32;
    for i in 0..n {
        let hh = hash1(i * 47 + x, 0x07e1);
        let fx = x + r((hh - 0.5) * rx as f64 * 1.9);
        let fy = top + 1 + r(hash1(i * 59 + x, 0x07e2) * (crown_h - 1).max(1) as f64);
        dr(fb, fx, fy, 2, 1, [0xd1, 0x40, 0x1e]);
        dr(fb, fx + (i % 2), fy - 1, 1, 1, [0xf2, 0x6a, 0x2a]);
        if size >= 18.0 {
            dr(fb, fx + 1, fy + 1, 1, 1, sh);
            dr(fb, fx, fy, 1, 1, hi);
        }
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
    let h = (size * 0.9).round().max(7.0) as i32;
    let trunk = get(pal, "trunk");
    let td = shade(trunk, -0.2);
    let (hi, mid, d) = crown_tone(pal, light);
    let spread = (size * 0.55).round().max(3.0) as i32;

    // Base corta y ramas en abanico (matorral, no árbol).
    dr(fb, x - 2, base_y - 3, 4, 3, trunk);
    dr(fb, x + 1, base_y - 3, 1, 3, td);
    let mut arms: Vec<(f64, f64)> = vec![
        (-1.0, 0.85),
        (1.0, 0.85),
        (-0.55, 1.0),
        (0.55, 1.0),
        (0.0, 1.0),
        (-1.0, 0.5),
        (1.0, 0.5),
    ];
    if size >= 18.0 {
        arms.extend_from_slice(&[(-0.78, 0.68), (0.78, 0.68), (-0.3, 0.62), (0.3, 0.62)]);
    }
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
            // Espinas claras.
            if j % 2 == 0 {
                dr(fb, px, py - 1, 1, 1, hi);
            }
        }
    }
    // Follaje disperso.
    for i in 0..(4 + (size * 0.5).floor() as i32) {
        let hh = hash1(i * 53 + x, 0x0e6);
        let fx = x + r((hh - 0.5) * spread as f64 * 1.8);
        let fy = base_y - r((0.35 + hash1(i, 0x0e7) * 0.6) * h as f64);
        let c = if i % 2 == 0 { mid } else { hi };
        dr(fb, fx, fy, 2, 1, c);
        dr(fb, fx + 1, fy + 1, 1, 1, d);
    }
    if season == 0 || season == 3 {
        let n = 3 + (size * 0.4).floor() as i32;
        for i in 0..n {
            let hh = hash1(i * 53 + x, 0x0e8);
            let fx = x + r((hh - 0.5) * spread as f64 * 1.9);
            let fy = base_y - r((0.4 + hh * 0.55) * h as f64);
            dr(fb, fx, fy, 1, 1, FLOWER_COLORS[1]);
            if size >= 18.0 {
                dr(fb, fx + 1, fy, 1, 1, [0xf7, 0xd9, 0x70]);
            }
        }
    }
}

fn draw_litre(
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
    let h = (size * 0.92).round().max(8.0) as i32;
    let trunk_h = (size * 0.22).round().max(3.0) as i32;
    let top = base_y - h;
    let tr = get(pal, "trunk");
    let trunk_w = (size * 0.07).round().clamp(2.0, 4.0) as i32;
    dr(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        tr,
    );
    // Corteza moteada del litre.
    let tdk = shade(tr, -0.3);
    let mut k = 1;
    while k < trunk_h {
        dr(
            fb,
            x - (trunk_w >> 1) + (k % trunk_w.max(1)),
            base_y - k,
            1,
            1,
            tdk,
        );
        k += 2;
    }
    bark_marks(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        tr,
        x as u32,
    );
    let crown_h = h - trunk_h;
    let tone = crown_tone(pal, light);
    let lit = lit_left(light);
    let rx = (size * 0.44).round().max(3.0) as i32;
    crown_blob(fb, x, top + crown_h / 2, rx, crown_h / 2, 0x117, tone, lit);
    // Follaje moteado (hojas claras/oscuras) del litre.
    let (hi, mid, d) = tone;
    let n = 4 + (size * 0.35).floor() as i32;
    for i in 0..n {
        let yy = top + 1 + (i % crown_h.max(1));
        if hash1(i * 7 + x, 0x117) < 0.7 {
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
    for k in 0..2 {
        let dx = (k * 2 - 1) * (rx / 2);
        line(
            fb,
            x + dx / 2,
            base_y - trunk_h - 1,
            x + dx,
            top + crown_h / 2,
            shade(mid, -0.2),
        );
    }
    if season == 0 || season == 3 {
        for i in 0..(1 + (size * 0.1) as i32) {
            let hh = hash1(i * 31 + x, 0x119);
            dr(
                fb,
                x + r((hh - 0.5) * rx as f64 * 1.4),
                top + 1 + r(hash1(i, 0x11a) * (crown_h - 1).max(1) as f64),
                1,
                1,
                [0xf2, 0xf0, 0xdc],
            );
        }
    }
}

fn draw_boldo(
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
    let h = (size * 0.92).round().max(8.0) as i32;
    let trunk_h = (size * 0.3).round().max(3.0) as i32;
    let top = base_y - h;
    let tr = get(pal, "trunk");
    let trunk_w = (size * 0.07).round().clamp(2.0, 4.0) as i32;
    dr(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        tr,
    );
    dr(fb, x, base_y - trunk_h, 1, trunk_h, shade(tr, -0.25));
    bark_marks(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        tr,
        x as u32,
    );
    let crown_h = h - trunk_h;
    let (hi, mid, d) = crown_tone(pal, light);
    // Follaje grisáceo y aromático (sello del boldo).
    let grey = get(pal, "rock");
    let tone = (lerp_color(hi, grey, 0.25), lerp_color(mid, grey, 0.3), d);
    let lit = lit_left(light);
    let rx = (size * 0.44).round().max(3.0) as i32;
    crown_blob(fb, x, top + crown_h / 2, rx, crown_h / 2, 0xb01, tone, lit);
    let (hi2, mid2, _) = tone;
    for k in 0..2 {
        let dx = (k * 2 - 1) * (rx / 2);
        line(
            fb,
            x + dx / 2,
            base_y - trunk_h - 1,
            x + dx,
            top + crown_h / 2,
            shade(mid2, -0.2),
        );
    }
    for i in 0..(3 + (size * 0.3).floor() as i32) {
        let yy = top + 1 + (i % crown_h.max(1));
        if hash1(i * 3 + x, 0xb01) < 0.7 {
            dr(
                fb,
                x + r((hash1(i, 0xb02) - 0.5) * rx as f64 * 1.3),
                yy,
                1,
                1,
                if i % 3 == 0 { hi2 } else { d },
            );
        }
    }
    // Flores blancas pequeñas en primavera/verano.
    if season == 0 || season == 3 {
        let n = 2 + (size * 0.22).floor() as i32;
        for i in 0..n {
            let hh = hash1(i * 67 + x, 0xb03);
            let fx = x + r((hh - 0.5) * rx as f64 * 1.5);
            let fy = top + 1 + r(hash1(i * 71 + x, 0xb04) * (crown_h - 1).max(1) as f64);
            dr(fb, fx, fy, 1, 1, [0xf4, 0xf0, 0xe6]);
            if size >= 18.0 {
                dr(fb, fx + 1, fy, 1, 1, [0xf7, 0xd9, 0x70]);
            }
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
    let h = (size * 1.0).round().max(9.0) as i32;
    let trunk_h = (size * 0.28).round().max(3.0) as i32;
    let top = base_y - h;
    // Tronco oscuro y grueso del peumo.
    let bark = shade(get(pal, "trunk"), -0.25);
    let trunk_w = (size * 0.09).round().clamp(2.0, 5.0) as i32;
    dr(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        bark,
    );
    dr(
        fb,
        x + (trunk_w >> 1) - 1,
        base_y - trunk_h,
        1,
        trunk_h,
        shade(bark, -0.2),
    );
    bark_marks(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        bark,
        x as u32,
    );
    let crown_h = h - trunk_h;
    let (hi, mid, d) = crown_tone(pal, light);
    // Siempreverde denso: copa ancha y oscura.
    let tone = (shade(hi, -0.05), shade(mid, -0.08), d);
    let lit = lit_left(light);
    let rx = (size * 0.48).round().max(3.0) as i32;
    crown_blob(fb, x, top + crown_h / 2, rx, crown_h / 2, 0xbe0, tone, lit);
    let (_, mid2, sh2) = tone;
    for k in 0..3 {
        let dx = (k - 1) * (rx / 2);
        line(
            fb,
            x + dx / 3,
            base_y - trunk_h - 1,
            x + dx,
            top + crown_h / 2,
            shade(mid2, -0.2),
        );
        dr(fb, x + dx, top + crown_h / 3, 1, 1, sh2);
    }
    // Frutos rojos en otoño/invierno.
    if season == 1 || season == 2 {
        let n = 3 + (size * 0.3).floor() as i32;
        for i in 0..n {
            let hh = hash1(i * 53 + x, 0xbe1);
            let fy = top + 1 + r(hash1(i, 0xbe2) * (crown_h - 1).max(1) as f64);
            let fx = x + r((hh - 0.5) * rx as f64 * 1.6);
            dr(fb, fx, fy, 1, 1, [0xc0, 0x3a, 0x3a]);
            if size >= 18.0 {
                dr(fb, fx + 1, fy + 1, 1, 1, [0x8e, 0x24, 0x2a]);
            }
        }
    }
}

fn draw_maiten(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 1.05).round().max(9.0) as i32;
    let trunk_h = (size * 0.3).round().max(3.0) as i32;
    let top = base_y - h;
    let trunk = get(pal, "trunk");
    let trunk_w = (size * 0.07).round().clamp(2.0, 4.0) as i32;
    dr(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        trunk,
    );
    dr(
        fb,
        x + (trunk_w >> 1) - 1,
        base_y - trunk_h,
        1,
        trunk_h,
        shade(trunk, -0.22),
    );
    bark_marks(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        trunk,
        x as u32,
    );
    let crown_h = h - trunk_h;
    let tone = crown_tone(pal, light);
    let (hi, mid, _) = tone;
    let lit = lit_left(light);
    let rx = (size * 0.44).round().max(3.0) as i32;
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
    let mut strands = vec![-1.0f64, 1.0, -0.6, 0.6, -0.2, 0.2];
    if size >= 20.0 {
        strands.extend_from_slice(&[-0.8, 0.8, -0.4, 0.4]);
    }
    for (i, k) in strands.iter().enumerate() {
        let lx = x + r(*k * rx as f64);
        let len = ((crown_h as f64) * 0.78 - (i as f64) * 0.6).max(2.0) as i32;
        let c = if i % 2 == 0 { mid } else { hi };
        let mut x0 = lx;
        for j in 0..len {
            dr(fb, x0, top + dome + j, 1, 1, c);
            if j % 3 == 2 {
                x0 += if *k < 0.0 { 1 } else { -1 };
            }
            if j % 4 == 1 {
                dr(
                    fb,
                    x0 + if *k < 0.0 { 1 } else { -1 },
                    top + dome + j,
                    1,
                    1,
                    mid,
                );
            }
        }
    }
}

fn draw_cipres(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 1.2).round().max(12.0) as i32;
    let trunk_h = (size * 0.16).round().max(2.0) as i32;
    let top = base_y - h;
    let trunk = get(pal, "trunk");
    let trunk_w = (size * 0.06).round().clamp(2.0, 4.0) as i32;
    dr(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        trunk,
    );
    bark_marks(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        trunk,
        x as u32,
    );
    let n = h - trunk_h;
    let half_max = (size * 0.17).round().max(2.0) as i32;
    // Conífera columnar de la precordillera.
    let tone = crown_tone(pal, light);
    let lit = lit_left(light);
    crown_cone(fb, x, top, n, half_max, 0xc1a, tone, lit, false);
    // Columna interior algo más oscura y bordes con ramillas cortas.
    let (hi, mid, sh) = tone;
    for i in (2..n).step_by(4) {
        let t = (i + 1) as f64 / n as f64;
        let half = (half_max as f64 * t.powf(0.82)).round().max(1.0) as i32;
        let yy = top + i;
        dr(fb, x - half - 1, yy, 1, 1, sh);
        dr(fb, x + half + 1, yy, 1, 1, sh);
        if i % 8 == 0 {
            dr(fb, x, yy, 1, 1, shade(mid, -0.16));
        }
    }
    dr(fb, x - 1, top - 1, 3, 1, mid);
    dr(fb, x, top - 2, 1, 1, hi);
}

fn draw_puya(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let (hi, mid, sh) = crown_tone(pal, light);
    // Roseta basal de hojas rígidas y espinosas.
    let r0 = (size * 0.5).round().max(3.0) as i32;
    rosette(fb, x, base_y - 1, r0, hi, mid, sh);
    // Vara floral alta con candelabro de flores.
    let sh_h = (size * 1.05).round().max(7.0) as i32;
    let top = base_y - sh_h;
    dr(fb, x, top, 1, sh_h, mid);
    if size >= 18.0 {
        dr(fb, x + 1, top, 1, sh_h, shade(mid, -0.15));
    }
    // Inflorescencia alta y densa: espiga verde-amarilla, no un candelabro abierto.
    let spike_h = (sh_h as f64 * 0.52).round().max(4.0) as i32;
    let spike_base = base_y - spike_h;
    for j in 0..spike_h {
        let yy = spike_base - j;
        let t = j as f64 / spike_h as f64;
        let half = (1.0 + t * size * 0.16).round().max(1.0) as i32;
        dr(
            fb,
            x - half,
            yy,
            half * 2 + 1,
            1,
            if j % 2 == 0 { mid } else { hi },
        );
        for side in [-1i32, 1] {
            let fx = x + side * (half + 1);
            let fc = if (j + if side > 0 { 1 } else { 0 }) % 2 == 0 {
                FLOWER_COLORS[1]
            } else {
                [0xe8, 0xe0, 0x9a]
            };
            dr(fb, fx, yy, 1, 1, fc);
        }
        if size >= 18.0 && j % 3 == 0 {
            dr(fb, x - half - 2, yy, 1, 1, [0xf7, 0xd9, 0x70]);
            dr(fb, x + half + 2, yy, 1, 1, [0xf7, 0xd9, 0x70]);
        }
    }
    for k in (0..sh_h).step_by(3) {
        dr(fb, x, top + k, 1, 1, FLOWER_COLORS[0]);
    }
    dr(fb, x, top - 1, 1, 1, FLOWER_COLORS[1]);
    if size >= 20.0 {
        dr(fb, x - 1, top - 2, 1, 1, FLOWER_COLORS[0]);
        dr(fb, x + 1, top - 2, 1, 1, FLOWER_COLORS[0]);
    }
}

fn draw_llareta(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let (hi, mid, sh) = crown_tone(pal, light);
    let sun = get(pal, "sun");
    // Cojín verde-amarillento de la puna.
    let hi2 = lerp_color(hi, sun, 0.25);
    let mid2 = lerp_color(mid, sun, 0.25);
    let w = (size * 1.7).round().max(6.0) as i32;
    let h = (size * 0.85).round().max(4.0) as i32;
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
    let n = (5 + (size * 1.0) as i32).clamp(5, 28);
    for i in 0..n {
        let hh = hash1(i * 19 + x, 0x11a);
        let bx = x + r((hh - 0.5) * w as f64 * 0.82);
        let byy = base_y - r(hash1(i, 0x11b) * h as f64);
        let c = if i % 3 == 0 {
            sh
        } else if i % 2 == 0 {
            hi2
        } else {
            mid2
        };
        dr(fb, bx, byy, 1, 1, c);
        if size >= 18.0 {
            dr(fb, bx + 1, byy, 1, 1, if i % 4 == 0 { hi2 } else { sh });
            dr(fb, bx, byy - 1, 1, 1, hi2);
        }
    }
    // Sombra de la base.
    dr(fb, x - w / 4, base_y - 1, w / 2, 1, shade(mid2, -0.18));
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
    let (hi, mid, sh) = crown_tone(pal, light);
    let n = (2 + (size * 0.6) as i32).clamp(2, 7);
    for i in 0..n {
        let sx = x + (i - (n >> 1)) * 2;
        let h = (size * 0.95).round().max(4.0) as i32 + (i % 2);
        dr(fb, sx, base_y - h, 1, h, mid);
        dr(fb, sx, base_y - h, 1, h / 2, hi);
        // Hoja linear a media altura.
        if size >= 16.0 && h > 8 {
            let ly = base_y - h / 2;
            dr(fb, sx + if i % 2 == 0 { 1 } else { -1 }, ly, 2, 1, sh);
        }
        // Umbela: flores amarillas agrupadas en la punta.
        let lean = r(sway * (2.0 + i as f64));
        let fx = sx + lean;
        let fy = base_y - h - 1;
        dr(fb, fx, fy, 1, 1, [0xf7, 0xd9, 0x70]);
        dr(fb, fx - 1, fy, 1, 1, FLOWER_COLORS[1]);
        dr(fb, fx + 1, fy, 1, 1, FLOWER_COLORS[1]);
        dr(fb, fx, fy - 1, 1, 1, [0xf7, 0xd9, 0x70]);
        if size >= 18.0 {
            dr(fb, fx - 1, fy - 1, 1, 1, [0xf7, 0xd9, 0x70]);
            dr(fb, fx + 1, fy - 1, 1, 1, [0xf7, 0xd9, 0x70]);
            dr(fb, fx, fy + 1, 1, 1, sh);
        }
    }
}

fn draw_junquillo(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, light: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let (hi, mid, sh) = crown_tone(pal, light);
    let lit = lit_left(light);
    let spacing = if size >= 18.0 { 2 } else { 1 };
    let n = (4 + (size * 0.45) as i32).clamp(4, 10);
    for i in 0..n {
        let gx = x + (i - (n >> 1)) * spacing;
        let h = (size * 1.05).round().max(4.0) as i32 + ((i * 3 + x) % 3);
        let c = if (gx < x) == lit { hi } else { mid };
        dr(fb, gx, base_y - h, 1, h, c);
        // Punta y espiga clara.
        dr(
            fb,
            gx,
            base_y - h - 1,
            1,
            1,
            if i % 2 == 0 { hi } else { sh },
        );
        if size >= 18.0 && i % 2 == 0 {
            dr(fb, gx, base_y - h - 2, 1, 1, sh);
            dr(
                fb,
                gx + if i % 4 == 0 { 1 } else { -1 },
                base_y - h - 1,
                1,
                1,
                mid,
            );
        }
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
    let w = size * 2.8;
    let n = (6 + (size * 0.9) as i32).clamp(6, 20);
    let dim = night.clamp(0.0, 1.0) * 0.85;
    let flora_d = get(pal, "floraD");
    let flora_l = get(pal, "floraL");
    let sky_top = get(pal, "skyTop");
    for i in 0..n {
        let hh = hash1(i * 31 + x, 0xf10e);
        let back = if hash1(i * 53 + x, 0xf10f) < 0.45 {
            1
        } else {
            0
        };
        let fx = x + r((hh - 0.5) * w + sway) - back;
        let stem_h = 2 + (hh * (4.0 + size * 0.15)) as i32;
        let base =
            FLOWER_COLORS[(hash1(i * 7 + x, 0xf1a) * FLOWER_COLORS.len() as f64).floor() as usize];
        let col = if dim > 0.0 {
            lerp_color(base, sky_top, dim)
        } else {
            base
        };
        let yy = base_y - back;
        dr(fb, fx, yy - stem_h, 1, stem_h, flora_d);
        // Hoja linear en el tallo.
        if size >= 14.0 && i % 2 == 0 {
            dr(
                fb,
                fx + if i % 4 == 0 { 1 } else { -1 },
                yy - stem_h / 2,
                2,
                1,
                flora_l,
            );
        }
        // Corola de 5 pétalos, con centro y volumen a mayor tamaño.
        dr(fb, fx, yy - stem_h - 1, 1, 1, col);
        dr(fb, fx - 1, yy - stem_h - 1, 1, 1, col);
        dr(fb, fx + 1, yy - stem_h - 1, 1, 1, col);
        dr(fb, fx - 1, yy - stem_h, 1, 1, col);
        dr(fb, fx + 1, yy - stem_h, 1, 1, col);
        dr(fb, fx, yy - stem_h - 2, 1, 1, [0xf7, 0xd9, 0x70]);
        if size >= 18.0 {
            dr(fb, fx - 2, yy - stem_h - 1, 1, 1, shade(col, -0.12));
            dr(fb, fx + 2, yy - stem_h - 1, 1, 1, shade(col, -0.12));
            dr(fb, fx, yy - stem_h - 3, 1, 1, shade(col, 0.12));
            dr(fb, fx - 1, yy - stem_h - 2, 1, 1, [0xf7, 0xd9, 0x70]);
            dr(fb, fx + 1, yy - stem_h - 2, 1, 1, [0xf7, 0xd9, 0x70]);
        }
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
                for base in [8.0, 18.0, 30.0] {
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
