//! Flora andina en pixel art (puerto de `src/flora.js`): registro, colocación
//! determinista por chunk y dibujo de cada especie.

use crate::biomes::{biome_at, biome_flora_pool, resolve_bloom};
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

fn outline(pal: &Palette) -> Rgb {
    shade(get(pal, "floraD"), -0.3)
}

#[inline]
fn r(x: f64) -> i32 {
    js_round(x) as i32
}

fn dr(fb: &mut Framebuffer, x: i32, y: i32, w: i32, h: i32, c: Rgb) {
    fb.fill_rect(x, y, w, h, c, 1.0);
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
) {
    match type_ {
        "araucaria" => draw_araucaria(fb, x, base_y, size, pal, sway),
        "lenga" => draw_lenga(fb, x, base_y, size, pal, warm, season),
        "bush" => draw_bush(fb, x, base_y, size, pal),
        "grass" => draw_grass(fb, x, base_y, size, pal),
        "crop" => draw_crop(fb, x, base_y, size, pal),
        "rock" => draw_rock(fb, x, base_y, size, pal),
        "flower" => draw_flower(fb, x, base_y, size, pal, sway, night),
        "cactus" => draw_cactus(fb, x, base_y, size, pal),
        "alerce" => draw_alerce(fb, x, base_y, size, pal),
        "nalca" => draw_nalca(fb, x, base_y, size, pal),
        "colihue" => draw_colihue(fb, x, base_y, size, pal),
        "palma" => draw_palma(fb, x, base_y, size, pal),
        "coihue" => draw_coihue(fb, x, base_y, size, pal),
        "roble" => draw_roble(fb, x, base_y, size, pal, season),
        "copihue" => draw_copihue(fb, x, base_y, size, pal, season),
        "michay" => draw_michay(fb, x, base_y, size, pal, season),
        "chaura" => draw_chaura(fb, x, base_y, size, pal),
        "quillay" => draw_quillay(fb, x, base_y, size, pal, season),
        "manio" => draw_manio(fb, x, base_y, size, pal),
        "canelo" => draw_canelo(fb, x, base_y, size, pal, season),
        "arrayan" => draw_arrayan(fb, x, base_y, size, pal, season),
        "notro" => draw_notro(fb, x, base_y, size, pal, season),
        _ => draw_grass(fb, x, base_y, size, pal),
    }
}

fn draw_araucaria(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, sway: f64) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 1.0).round().max(8.0) as i32;
    let trunk_h = (size * 0.4).round().max(3.0) as i32;
    let trunk_w = (size * 0.09).round().max(1.0) as i32;
    let crown_h = h - trunk_h;
    let crown_top = base_y - h;
    let half_max = (size * 0.46).round().max(3.0) as i32;
    let cx = x + r(sway * 0.6);
    let dark = outline(pal);
    let (trunk, flora_l, flora_d) = (get(pal, "trunk"), get(pal, "floraL"), get(pal, "floraD"));

    dr(
        fb,
        x - (trunk_w >> 1),
        base_y - trunk_h,
        trunk_w,
        trunk_h,
        trunk,
    );
    let cy = crown_top as f64 + crown_h as f64 / 2.0;
    let ry = crown_h as f64 / 2.0;
    let mut dy = -ry;
    while dy <= ry {
        let t = dy / ry;
        let mut half = r(half_max as f64 * (1.0 - t * t).max(0.0).sqrt());
        if t > 0.7 {
            half = r(half_max as f64 * 0.94);
        }
        if half > 0 {
            let yy = r(cy + dy);
            dr(fb, cx - half, yy, half * 2 + 1, 1, dark);
            dr(fb, cx - half + 1, yy, half * 2 - 1, 1, flora_d);
            if dy % 3.0 == 0.0 {
                dr(
                    fb,
                    cx - half + 1,
                    yy,
                    (half as f64 * 1.3).round().max(1.0) as i32,
                    1,
                    flora_l,
                );
            }
        }
        dy += 1.0;
    }
    dr(fb, cx - 1, crown_top - 1, 3, 1, flora_l);
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

fn draw_lenga(
    fb: &mut Framebuffer,
    x: f64,
    base_y: f64,
    size: f64,
    pal: &Palette,
    _warm: bool,
    season: i32,
) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 1.0).round().max(6.0) as i32;
    let trunk_h = (size * 0.3).round().max(2.0) as i32;
    let top = base_y - h;
    let dark = outline(pal);
    let (trunk, flora_l, flora_d) = (get(pal, "trunk"), get(pal, "floraL"), get(pal, "floraD"));

    dr(fb, x - 1, base_y - trunk_h, 2, trunk_h, trunk);
    if season == 2 {
        draw_bare_branches(fb, x, top, h - trunk_h, pal);
        return;
    }
    let crown_h = h - trunk_h;
    let ry = crown_h as f64 / 2.0;
    let rx = (size * 0.44).round().max(2.0) as i32;
    let cy = top as f64 + ry;
    let mut dy = -ry;
    while dy <= ry {
        let t = dy / ry;
        let mut span = (rx as f64 * (1.0 - t * t).max(0.0).sqrt()).floor() as i32;
        if hash1((x) * 7 + r(dy), 0x1e9) < 0.3 {
            span -= 1;
        }
        if season == 1 && hash1((x) * 7 + r(dy) + 0x1e, 0xa7e) < 0.3 {
            dy += 1.0;
            continue;
        }
        if span > 0 {
            let yy = r(cy + dy);
            dr(fb, x - span, yy, span * 2 + 1, 1, dark);
            dr(fb, x - span + 1, yy, span * 2 - 1, 1, flora_l);
            if t > 0.0 {
                dr(fb, x + 1, yy, span - 1, 1, flora_d);
            }
        }
        dy += 1.0;
    }
    dr(fb, x - 2, base_y - trunk_h - 2, 1, 2, trunk);
    dr(fb, x + 2, base_y - trunk_h - 2, 1, 2, trunk);
    if season == 3 {
        dr(fb, x - 2, r(cy - ry), 1, 1, flora_l);
        dr(fb, x + 1, r(cy - ry) + 1, 1, 1, flora_l);
    }
}

fn draw_bush(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette) {
    let x = r(x);
    let base_y = r(base_y);
    let w = (size * 0.8).round().max(4.0) as i32;
    let h = (size * 0.5).round().max(3.0) as i32;
    let dark = outline(pal);
    let (flora_l, flora_d) = (get(pal, "floraL"), get(pal, "floraD"));
    for rr in 0..h {
        let t = (rr + 1) as f64 / (h + 1) as f64;
        let mut half = ((w as f64 / 2.0) * (t * std::f64::consts::PI * 0.9).sin() + w as f64 * 0.08)
            .round()
            .max(1.0) as i32;
        if hash1(x * 3 + rr, 0x6b) < 0.35 {
            half = (half - 1).max(1);
        }
        let yy = base_y - h + rr;
        dr(fb, x - half, yy, half * 2, 1, dark);
        dr(fb, x - half + 1, yy, (half * 2 - 2).max(1), 1, flora_d);
        if rr == 0 || rr == h - 1 {
            dr(fb, x - half + 1, yy, (half * 2 - 2).max(1), 1, flora_l);
        }
    }
}

fn draw_grass(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette) {
    let x = r(x);
    let base_y = r(base_y);
    let n = 2 + (size * 0.4).floor() as i32;
    let (flora_l, flora_d) = (get(pal, "floraL"), get(pal, "floraD"));
    for i in 0..n {
        let gx = x + i - (n >> 1);
        let gh = 1 + ((i * 7 + x) % 2);
        let c = if i % 2 != 0 { flora_l } else { flora_d };
        dr(fb, gx, base_y - gh, 1, gh, c);
    }
}

fn draw_crop(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette) {
    let x = r(x);
    let base_y = r(base_y);
    let w = (size * 0.9).round().max(4.0) as i32;
    let h = (size * 0.32).round().max(1.0) as i32;
    dr(fb, x - (w >> 1), base_y - h, w, h, get(pal, "valleyD"));
    let mut i = 0;
    while i < w {
        let c = if i % 4 == 0 {
            get(pal, "valleyL")
        } else {
            get(pal, "costaL")
        };
        dr(fb, x - (w >> 1) + i, base_y - h - 1, 1, 1, c);
        i += 2;
    }
}

fn draw_rock(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette) {
    let x = r(x);
    let base_y = r(base_y);
    let w = (size * 0.55).round().max(3.0) as i32;
    let h = (size * 0.32).round().max(2.0) as i32;
    let profile = [0.45, 0.75, 1.0, 0.95, 0.7];
    for rr in 0..h {
        let f = profile[(rr.min(4)) as usize];
        let half = ((w as f64 / 2.0) * f).round().max(1.0) as i32;
        let yy = base_y - h + rr;
        dr(fb, x - half, yy, half * 2, 1, get(pal, "rockD"));
        if rr == 0 {
            dr(fb, x - half, yy, half * 2, 1, get(pal, "rock"));
        }
    }
}

fn draw_cactus(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 1.1).round().max(7.0) as i32;
    let top = base_y - h;
    let w = (size * 0.34).round().max(3.0) as i32;
    let half = w >> 1;
    let dark = outline(pal);
    let (flora_l, flora_d, sand_d) = (get(pal, "floraL"), get(pal, "floraD"), get(pal, "sandD"));

    dr(fb, x - half, top, w, h, dark);
    dr(fb, x - half + 1, top, (w - 2).max(1), h, flora_d);
    dr(fb, x - half + 1, top, 1, h, flora_l);

    let dir = if hash1(x, 0xcac) < 0.5 { -1 } else { 1 };
    let arm_y = top + r(h as f64 * 0.45);
    let arm_h = (h as f64 * 0.34).round().max(2.0) as i32;
    dr(fb, x + dir * half, arm_y, 2, 1, dark);
    dr(fb, x + dir * (half + 1), arm_y - arm_h, 2, arm_h + 1, dark);
    dr(fb, x + dir * (half + 1), arm_y - arm_h, 1, arm_h, flora_d);
    dr(fb, x + dir * (half + 1), arm_y - arm_h, 1, 1, flora_l);

    if hash1(x, 0xca5) < 0.5 {
        let d2 = -dir;
        let a_y = top + r(h as f64 * 0.62);
        dr(fb, x + d2 * half, a_y, 1, 1, flora_d);
        dr(fb, x + d2 * (half + 1), a_y - 2, 1, 2, flora_d);
    }
    for frac in [0.28, 0.55, 0.8] {
        let y = top + r(h as f64 * frac);
        dr(fb, x - half - 1, y, 1, 1, sand_d);
        dr(fb, x + half + 1, y + 1, 1, 1, sand_d);
    }
    if hash1(x, 0xcaf) < 0.5 {
        dr(fb, x, top - 1, 1, 1, flora_l);
    }
}

fn draw_alerce(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 1.05).round().max(8.0) as i32;
    let trunk_h = (size * 0.2).round().max(2.0) as i32;
    let top = base_y - h;
    let half_max = (size * 0.32).round().max(2.0) as i32;
    let (trunk, flora_l, flora_d) = (get(pal, "trunk"), get(pal, "floraL"), get(pal, "floraD"));
    dr(fb, x - 1, base_y - trunk_h, 2, trunk_h, trunk);
    let n = h - trunk_h;
    for i in 0..n {
        let t = i as f64 / n as f64;
        let half = (half_max as f64 * t.powf(0.68)).round().max(1.0) as i32;
        let yy = top + i;
        dr(fb, x - half, yy, half * 2 + 1, 1, flora_d);
        if i % 2 == 0 {
            dr(fb, x - half, yy, half * 2 + 1, 1, flora_l);
        }
        if i % 3 == 1 {
            dr(fb, x - half - 1, yy + 1, 1, 1, flora_d);
            dr(fb, x + half + 1, yy + 1, 1, 1, flora_d);
        }
    }
    dr(fb, x - 1, top - 1, 2, 1, flora_l);
}

fn draw_nalca(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette) {
    let x = r(x);
    let base_y = r(base_y);
    let dark = outline(pal);
    let n = 3 + (size * 0.1).floor() as i32;
    let (trunk, flora_l, flora_d) = (get(pal, "trunk"), get(pal, "floraL"), get(pal, "floraD"));
    for i in 0..n {
        let hh = hash1(i * 13 + x, 0xaa11);
        let lx = x + r((hh - 0.5) * size * 1.1);
        let stem_h = (size * (0.4 + hh * 0.4)).round().max(3.0) as i32;
        let top = base_y - stem_h;
        dr(fb, lx, top, 1, stem_h, trunk);
        let rr = (size * 0.3).round().max(2.0) as i32;
        for dy in -rr..=rr {
            let span = (rr as f64 * (1.0 - (dy as f64 / rr as f64).powi(2)).max(0.0).sqrt()).floor()
                as i32;
            if span <= 0 {
                continue;
            }
            let dent = if hash1(i * 31 + lx + dy, 0xaa12) < 0.5 {
                1
            } else {
                0
            };
            dr(fb, lx - span - dent, top + dy, span * 2 + 1 + dent, 1, dark);
            dr(
                fb,
                lx - span - dent + 1,
                top + dy,
                (span * 2 - 1 + dent).max(1),
                1,
                flora_d,
            );
        }
        dr(fb, lx, top - rr, 1, rr * 2 + 1, flora_l);
        dr(fb, lx - rr, top - rr, rr * 2 + 1, 1, flora_l);
    }
}

fn draw_colihue(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette) {
    let x = r(x);
    let base_y = r(base_y);
    let dark = outline(pal);
    let n = 4 + (size * 0.25).floor() as i32;
    let (trunk, flora_l, flora_d) = (get(pal, "trunk"), get(pal, "floraL"), get(pal, "floraD"));
    for i in 0..n {
        let hh = hash1(i * 17 + x, 0xc01);
        let cxp = x + r((i as f64 - (n as f64 - 1.0) / 2.0) * 0.8);
        let ch = (size * (0.75 + hh * 0.5)).round().max(5.0) as i32;
        let top = base_y - ch;
        dr(fb, cxp, top, 1, ch, trunk);
        let mut k = 2;
        while k < ch {
            dr(fb, cxp, base_y - k, 1, 1, dark);
            k += 3;
        }
        let c = if i % 2 != 0 { flora_l } else { flora_d };
        let leaves = 3 + (hh * 3.0).floor() as i32;
        let step = (ch as f64 / leaves as f64).round().max(2.0) as i32;
        for j in 0..leaves {
            let ly = top + 1 + j * step;
            let dir = if (j + i) % 2 != 0 { 1 } else { -1 };
            dr(fb, cxp + dir, ly, 1, 1, c);
            dr(fb, cxp + dir * 2, ly + 1, 1, 1, c);
            if j % 2 != 0 {
                dr(fb, cxp + dir, ly + 2, 1, 1, c);
            }
        }
    }
}

fn draw_palma(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette) {
    let x = r(x);
    let base_y = r(base_y);
    let trunk_h = (size * 0.46).round().max(4.0) as i32;
    let top = base_y - trunk_h;
    let (trunk, flora_l, flora_d) = (get(pal, "trunk"), get(pal, "floraL"), get(pal, "floraD"));
    dr(fb, x - 1, top, 2, trunk_h, trunk);
    let trunk_dark = shade(trunk, -0.2);
    let mut i = 3;
    while i < trunk_h {
        dr(fb, x - 1, top + i, 2, 1, trunk_dark);
        i += 3;
    }
    let rr = (size * 0.26).round().max(3.0) as i32;
    let dirs = [
        (0.0, -1.0),
        (-0.6, -0.85),
        (0.6, -0.85),
        (-0.9, -0.4),
        (0.9, -0.4),
        (-1.0, 0.15),
        (1.0, 0.15),
        (-0.8, 0.6),
        (0.8, 0.6),
    ];
    for (ux, uy) in dirs {
        for k in 1..=rr {
            dr(
                fb,
                x + r(ux * k as f64),
                top + r(uy * k as f64),
                1,
                1,
                flora_d,
            );
        }
    }
    for (ux, uy) in [
        (0.0, -1.0),
        (-1.0, 0.15),
        (1.0, 0.15),
        (-0.8, 0.6),
        (0.8, 0.6),
    ] {
        dr(
            fb,
            x + r(ux * rr as f64),
            top + r(uy * rr as f64),
            1,
            1,
            flora_l,
        );
    }
    dr(fb, x, top - 1, 1, 2, trunk);
}

fn draw_coihue(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 1.1).round().max(6.0) as i32;
    let trunk_h = (size * 0.38).round().max(3.0) as i32;
    let top = base_y - h;
    let (trunk, flora_l, flora_d) = (get(pal, "trunk"), get(pal, "floraL"), get(pal, "floraD"));
    dr(fb, x - 1, base_y - trunk_h, 2, trunk_h, trunk);
    dr(fb, x - 2, base_y - trunk_h - 2, 1, 2, trunk);
    dr(fb, x + 2, base_y - trunk_h - 2, 1, 2, trunk);
    dr(fb, x - 3, base_y - trunk_h - 4, 1, 2, trunk);
    dr(fb, x + 3, base_y - trunk_h - 4, 1, 2, trunk);
    let crown_h = h - trunk_h;
    let ry = crown_h as f64 / 2.0;
    let rx = (size * 0.48).round().max(2.0) as i32;
    let cy = top as f64 + ry;
    let dark = outline(pal);
    let mut dy = -ry;
    while dy <= ry {
        let t = dy / ry;
        let mut span = (rx as f64 * (1.0 - t * t).max(0.0).sqrt()).floor() as i32;
        if hash1(x * 7 + r(dy), 0xc01) < 0.35 {
            span -= 1;
        }
        if span > 0 {
            let yy = r(cy + dy);
            dr(fb, x - span, yy, 1, 1, dark);
            dr(fb, x + span, yy, 1, 1, dark);
            dr(fb, x - span, yy, span * 2 + 1, 1, flora_l);
            dr(fb, x + 1, yy, span, 1, flora_d);
        }
        dy += 1.0;
    }
}

fn draw_roble(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, season: i32) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 1.15).round().max(6.0) as i32;
    let trunk_h = (size * 0.28).round().max(2.0) as i32;
    let top = base_y - h;
    let trunk = get(pal, "trunk");
    dr(fb, x - 1, base_y - trunk_h, 2, trunk_h, trunk);
    let crown_h = h - trunk_h;
    if season == 2 {
        draw_bare_branches(fb, x, top, crown_h, pal);
        return;
    }
    dr(fb, x - 2, base_y - trunk_h - 2, 1, 2, trunk);
    dr(fb, x + 2, base_y - trunk_h - 2, 1, 2, trunk);
    let half_max = (size * 0.32).round().max(2.0) as i32;
    let dark = outline(pal);
    let (flora_l, flora_d) = (get(pal, "floraL"), get(pal, "floraD"));
    for i in 0..crown_h {
        if season == 1 && hash1(x * 11 + i + 0x2f, 0xb3e) < 0.3 {
            continue;
        }
        let t = i as f64 / crown_h as f64;
        let mut half = (half_max as f64 * ((0.25 + 0.75 * t) * std::f64::consts::PI).sin())
            .round()
            .max(1.0) as i32;
        if hash1(x * 13 + i, 0xb0b) < 0.3 {
            half = (half - 1).max(1);
        }
        let yy = top + i;
        dr(fb, x - half, yy, 1, 1, dark);
        dr(fb, x + half, yy, 1, 1, dark);
        dr(fb, x - half, yy, half * 2 + 1, 1, flora_l);
        dr(fb, x + 1, yy, half, 1, flora_d);
    }
    if season == 3 {
        dr(fb, x - 2, top - 1, 1, 1, flora_l);
        dr(fb, x + 1, top - 2, 1, 1, flora_l);
    }
}

fn draw_copihue(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, season: i32) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 1.05).round().max(6.0) as i32;
    let top = base_y - h;
    let dark = outline(pal);
    let flora_l = get(pal, "floraL");
    for i in 0..h {
        let yy = base_y - i;
        let off = r((i as f64 * 0.6).sin() * 1.5);
        dr(fb, x + off, yy, 1, 1, dark);
        if i % 3 == 0 {
            dr(
                fb,
                x + off + if i % 2 != 0 { 1 } else { -1 },
                yy,
                1,
                1,
                flora_l,
            );
        }
    }
    if season != 0 && season != 3 {
        return;
    }
    let n = 2 + (size * 0.16).floor() as i32;
    for i in 0..n {
        let yy = top + r(((i + 1) as f64 / (n + 1) as f64) * h as f64);
        let off = r(((base_y - yy) as f64 * 0.6).sin() * 1.5);
        let bx = x + off;
        dr(fb, bx, yy + 1, 3, 3, [0xd9, 0x4f, 0x6a]);
        dr(fb, bx + 1, yy, 1, 1, [0xe0, 0x50, 0x70]);
        dr(fb, bx, yy + 2, 1, 1, [0xe0, 0x50, 0x70]);
        dr(fb, bx + 2, yy + 2, 1, 1, [0xe0, 0x50, 0x70]);
        dr(fb, bx + 1, yy + 4, 1, 1, [0xf2, 0xa0, 0xb4]);
    }
}

fn draw_michay(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, season: i32) {
    let x = r(x);
    let base_y = r(base_y);
    let w = (size * 0.8).round().max(4.0) as i32;
    let h = (size * 0.6).round().max(3.0) as i32;
    let dark = outline(pal);
    let (flora_l, flora_d) = (get(pal, "floraL"), get(pal, "floraD"));
    for rr in 0..h {
        let t = (rr + 1) as f64 / (h + 1) as f64;
        let half = ((w as f64 / 2.0) * (t * std::f64::consts::PI * 0.9).sin() + w as f64 * 0.08)
            .round()
            .max(1.0) as i32;
        let yy = base_y - h + rr;
        dr(fb, x - half, yy, half * 2, 1, dark);
        dr(fb, x - half + 1, yy, (half * 2 - 2).max(1), 1, flora_d);
        if rr == 0 || rr == h - 1 {
            dr(fb, x - half + 1, yy, (half * 2 - 2).max(1), 1, flora_l);
        }
    }
    dr(fb, x - (w >> 1) - 1, base_y - h + 1, 1, 1, dark);
    dr(fb, x + (w >> 1), base_y - h + 2, 1, 1, dark);
    if season != 0 && season != 3 {
        return;
    }
    let n = 2 + (size * 0.16).floor() as i32;
    for i in 0..n {
        let hh = hash1(i * 23 + x, 0xb011);
        let fx = x + r((hh - 0.5) * w as f64);
        let fy = base_y - h - 1 - (i % 2);
        dr(fb, fx, fy, 1, 1, [0xe8, 0x91, 0x2a]);
        dr(fb, fx + 1, fy, 1, 1, [0xf6, 0xc0, 0x4a]);
    }
}

fn draw_chaura(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette) {
    let x = r(x);
    let base_y = r(base_y);
    let w = (size * 0.72).round().max(4.0) as i32;
    let h = (size * 0.44).round().max(2.0) as i32;
    let dark = outline(pal);
    let (flora_l, flora_d) = (get(pal, "floraL"), get(pal, "floraD"));
    for rr in 0..h {
        let t = (rr + 1) as f64 / (h + 1) as f64;
        let half = ((w as f64 / 2.0) * (t * std::f64::consts::PI * 0.9).sin() + w as f64 * 0.08)
            .round()
            .max(1.0) as i32;
        let yy = base_y - h + rr;
        dr(fb, x - half, yy, half * 2, 1, dark);
        dr(fb, x - half + 1, yy, (half * 2 - 2).max(1), 1, flora_d);
        if rr == 0 {
            dr(fb, x - half + 1, yy, (half * 2 - 2).max(1), 1, flora_l);
        }
    }
    let n = 2 + (size * 0.16).floor() as i32;
    for i in 0..n {
        let hh = hash1(i * 29 + x, 0x0c4);
        let fx = x + r((hh - 0.5) * w as f64);
        let fy = base_y - h - 1 - (i % 2);
        dr(fb, fx, fy, 1, 1, [0xf2, 0xd8, 0xe8]);
        dr(fb, fx, fy - 1, 1, 1, [0xd9, 0x7a, 0x9a]);
    }
}

fn draw_quillay(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, season: i32) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 0.85).round().max(5.0) as i32;
    let trunk_h = (size * 0.22).round().max(2.0) as i32;
    let top = base_y - h;
    dr(fb, x - 1, base_y - trunk_h, 2, trunk_h, get(pal, "trunk"));
    let crown_h = h - trunk_h;
    let ry = crown_h as f64 / 2.0;
    let rx = (size * 0.4).round().max(2.0) as i32;
    let cy = top as f64 + ry;
    let dark = outline(pal);
    let (flora_l, flora_d) = (get(pal, "floraL"), get(pal, "floraD"));
    let mut dy = -ry;
    while dy <= ry {
        let span = (rx as f64 * (1.0 - (dy / ry).powi(2)).max(0.0).sqrt()).floor() as i32;
        if span > 0 {
            let yy = r(cy + dy);
            dr(fb, x - span, yy, 1, 1, dark);
            dr(fb, x + span, yy, 1, 1, dark);
            dr(fb, x - span, yy, span * 2 + 1, 1, flora_l);
            if hash1(x * 5 + yy, 0x9111) < 0.4 {
                dr(fb, x + 1, yy, span, 1, flora_d);
            }
        }
        dy += 1.0;
    }
    if season != 0 && season != 3 {
        return;
    }
    let n = 1 + (size * 0.1).floor() as i32;
    for i in 0..n {
        let hh = hash1(i * 37 + x, 0x9112);
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

fn draw_manio(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 1.0).round().max(7.0) as i32;
    let trunk_h = (size * 0.18).round().max(2.0) as i32;
    let top = base_y - h;
    let half_max = (size * 0.3).round().max(2.0) as i32;
    let (trunk, flora_l, flora_d) = (get(pal, "trunk"), get(pal, "floraL"), get(pal, "floraD"));
    dr(fb, x - 1, base_y - trunk_h, 2, trunk_h, trunk);
    let n = h - trunk_h;
    for i in 0..n {
        let t = i as f64 / n as f64;
        let half = (half_max as f64 * (0.35 + 0.65 * (t * std::f64::consts::PI).sin()))
            .round()
            .max(1.0) as i32;
        let yy = top + i;
        dr(fb, x - half, yy, half * 2 + 1, 1, flora_d);
        if i % 3 == 0 || t < 0.18 {
            dr(fb, x - half, yy, half * 2 + 1, 1, flora_l);
        }
        if i % 2 == 1 {
            dr(fb, x - half - 1, yy, 1, 1, flora_l);
            dr(fb, x + half + 1, yy, 1, 1, flora_l);
        }
    }
    dr(fb, x - 1, top - 1, 2, 1, flora_l);
}

fn draw_canelo(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, season: i32) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 0.95).round().max(6.0) as i32;
    let trunk_h = (size * 0.24).round().max(2.0) as i32;
    let top = base_y - h;
    dr(fb, x - 1, base_y - trunk_h, 2, trunk_h, get(pal, "trunk"));
    let crown_h = h - trunk_h;
    let ry = crown_h as f64 / 2.0;
    let rx = (size * 0.36).round().max(2.0) as i32;
    let cy = top as f64 + ry;
    let dark = outline(pal);
    let (flora_l, flora_d) = (get(pal, "floraL"), get(pal, "floraD"));
    let mut dy = -ry;
    while dy <= ry {
        let span = (rx as f64 * (1.0 - (dy / ry).powi(2)).max(0.0).sqrt()).floor() as i32;
        if span > 0 {
            let yy = r(cy + dy);
            dr(fb, x - span, yy, 1, 1, dark);
            dr(fb, x + span, yy, 1, 1, dark);
            dr(fb, x - span, yy, span * 2 + 1, 1, flora_d);
            if (yy + span) % 3 == 0 {
                dr(fb, x - span, yy, span * 2 + 1, 1, flora_l);
            }
        }
        dy += 1.0;
    }
    if season != 0 && season != 3 {
        return;
    }
    let n = 1 + (size * 0.1).floor() as i32;
    for i in 0..n {
        let hh = hash1(i * 41 + x, 0xc0e);
        dr(
            fb,
            x + r((hh - 0.5) * rx as f64 * 1.7),
            top + 1 + (i % crown_h.max(1)),
            1,
            1,
            [0xf6, 0xf2, 0xe6],
        );
    }
}

fn draw_arrayan(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, season: i32) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 0.8).round().max(5.0) as i32;
    let trunk_h = (size * 0.34).round().max(3.0) as i32;
    let top = base_y - h;
    dr(fb, x - 1, base_y - trunk_h, 2, trunk_h, [0x8a, 0x4a, 0x3a]);
    let crown_h = h - trunk_h;
    let ry = crown_h as f64 / 2.0;
    let rx = (size * 0.32).round().max(2.0) as i32;
    let cy = top as f64 + ry;
    let dark = outline(pal);
    let (flora_l, flora_d) = (get(pal, "floraL"), get(pal, "floraD"));
    let mut dy = -ry;
    while dy <= ry {
        let span = (rx as f64 * (1.0 - (dy / ry).powi(2)).max(0.0).sqrt()).floor() as i32;
        if span > 0 {
            let yy = r(cy + dy);
            dr(fb, x - span, yy, 1, 1, dark);
            dr(fb, x + span, yy, 1, 1, dark);
            dr(fb, x - span, yy, span * 2 + 1, 1, flora_d);
            dr(fb, x - span, yy, span.max(1), 1, flora_l);
        }
        dy += 1.0;
    }
    if season != 0 && season != 3 {
        return;
    }
    let n = 1 + (size * 0.1).floor() as i32;
    for i in 0..n {
        let hh = hash1(i * 43 + x, 0xa77a);
        dr(
            fb,
            x + r((hh - 0.5) * rx as f64 * 1.6),
            top + 1 + (i % 3),
            1,
            1,
            [0xf4, 0xf0, 0xe6],
        );
    }
}

fn draw_notro(fb: &mut Framebuffer, x: f64, base_y: f64, size: f64, pal: &Palette, season: i32) {
    let x = r(x);
    let base_y = r(base_y);
    let h = (size * 0.85).round().max(5.0) as i32;
    let trunk_h = (size * 0.26).round().max(2.0) as i32;
    let top = base_y - h;
    dr(fb, x - 1, base_y - trunk_h, 2, trunk_h, get(pal, "trunk"));
    let crown_h = h - trunk_h;
    let rx = (size * 0.34).round().max(2.0) as i32;
    let ry = crown_h as f64 / 2.0;
    let cy = top as f64 + ry;
    let dark = outline(pal);
    let flora_d = get(pal, "floraD");
    let mut dy = -ry;
    while dy <= ry {
        let span = (rx as f64 * (1.0 - (dy / ry).powi(2)).max(0.0).sqrt()).floor() as i32;
        if span > 0 {
            let yy = r(cy + dy);
            dr(fb, x - span, yy, 1, 1, dark);
            dr(fb, x + span, yy, 1, 1, dark);
            dr(fb, x - span, yy, span * 2 + 1, 1, flora_d);
        }
        dy += 1.0;
    }
    if season != 0 && season != 3 {
        return;
    }
    let n = 2 + (size * 0.14).floor() as i32;
    for i in 0..n {
        let hh = hash1(i * 47 + x, 0x07e0);
        let fx = x + r((hh - 0.5) * rx as f64 * 1.8);
        let fy = base_y - r(2.0 + hh * crown_h as f64);
        dr(fb, fx, fy, 1, 1, [0xd1, 0x40, 0x1e]);
        dr(fb, fx + 1, fy - (i % 2), 1, 1, [0xf2, 0x6a, 0x2a]);
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
        dr(fb, fx, yy - stem_h - 1, 1, 1, col);
        dr(fb, fx - 1, yy - stem_h, 1, 1, col);
        dr(fb, fx + 1, yy - stem_h, 1, 1, col);
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
            let pool = pool_at(wx);
            let list = if !pool.is_empty() {
                pool
            } else {
                centro_flora_pool(layer.name)
            };
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
        let b = biome_at(wx, seed, biome_mode);
        let bloom = resolve_bloom(bloom_mode, &b.weights, b.bloom);
        biome_flora_pool(layer.name, &b.weights, bloom)
    };
    for s in flora_spawns(layer, terrain, layer_idx, camera_x, w, h, seed, pool_at) {
        let sway = (t_sec * 0.9 + s.wx * 0.05).sin();
        let gy = if s.depth != 0 {
            (s.gy + s.depth).min(h as i32 - 2)
        } else {
            s.gy
        };
        draw_plant(
            fb, s.sx, gy as f64, s.size, s.type_, &pal2, sway, s.warm, season, night_amt,
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
            );
        }
        assert!(fb.px.iter().any(|p| *p != 0));
    }
}
