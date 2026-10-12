//! Momentos raros y deterministas (puerto de `src/moments.js`): 18sep, Leo Rey,
//! Kung Leo, cóndor, bandada y manada.

use crate::framebuffer::Framebuffer;
use crate::math::js_round;
use crate::palette::{get, hex_to_rgb, Palette, Rgb};
use crate::rng::{hash1, hash_int, Mulberry32};
use crate::terrain::{Terrain, LAYERS};

#[inline]
fn r(x: f64) -> i32 {
    js_round(x) as i32
}

pub const MOMENT_IDS: [&str; 6] = ["18sep", "leorey", "kungleo", "condor", "bandada", "manada"];
const SPACING: f64 = 9000.0;
const CHANCE: f64 = 0.22;
const TRICOLOR: [Rgb; 3] = [[0x00, 0x39, 0xa6], [0xff, 0xff, 0xff], [0xd5, 0x2b, 0x1e]];

fn costa_index() -> Option<usize> {
    LAYERS.iter().position(|l| l.name == "costa")
}

#[derive(Clone, Copy, Debug)]
pub struct MomentEvent {
    pub type_: &'static str,
    pub wx: f64,
}

pub fn moment_events(seed: u32, camera_x: f64, w: usize) -> Vec<MomentEvent> {
    let chunk_seed = seed ^ 0x1e0e;
    let c0 = ((camera_x - 240.0) / SPACING).floor() as i64;
    let c1 = ((camera_x + w as f64 + 240.0) / SPACING).floor() as i64;
    let mut out = Vec::new();
    for c in c0..=c1 {
        let mut rng = Mulberry32::new(hash_int(c as i32, chunk_seed));
        if rng.next_f64() > CHANCE {
            continue;
        }
        let type_ = MOMENT_IDS[(rng.next_f64() * MOMENT_IDS.len() as f64).floor() as usize];
        let wx = (c as f64 + 0.2 + rng.next_f64() * 0.6) * SPACING;
        out.push(MomentEvent { type_, wx });
    }
    out
}

fn events_for(mode: &str, seed: u32, camera_x: f64, w: usize) -> Vec<MomentEvent> {
    if mode == "auto" {
        return moment_events(seed, camera_x, w);
    }
    if !MOMENT_IDS.contains(&mode) {
        return Vec::new();
    }
    vec![MomentEvent {
        type_: intern(mode),
        wx: camera_x + w as f64 * 0.5,
    }]
}

fn intern(name: &str) -> &'static str {
    MOMENT_IDS
        .iter()
        .find(|m| **m == name)
        .copied()
        .unwrap_or("18sep")
}

fn patriotic_sky(fb: &mut Framebuffer, w: usize, h: usize, t_sec: f64) {
    let red = [0xd5, 0x2b, 0x1e];
    let white = [0xff, 0xff, 0xff];
    fb.fill_rect(0, 0, w as i32, r(h as f64 * 0.26), red, 0.1);
    fb.fill_rect(
        0,
        r(h as f64 * 0.26),
        w as i32,
        r(h as f64 * 0.16),
        white,
        0.06,
    );
    for i in 0..70 {
        let hh = hash1(i, 0x18ce);
        let drift = 8.0 + hh * 26.0;
        let x = ((i as f64 * 149.0 + t_sec * drift) % (w as f64 + 24.0)) - 12.0;
        let y = (hash1(i, 0x5e9) * h as f64 * 0.55 + t_sec * (6.0 + hh * 14.0)) % (h as f64 * 0.62);
        fb.fill_rect(r(x), r(y), 2, 3, TRICOLOR[i as usize % 3], 0.7);
    }
}

const LEO_FRAMES: [&[&str]; 2] = [
    &[
        "..h.h.h..",
        ".hhhhhh..",
        "hhhhhhh..",
        ".sssss...",
        ".skkss...",
        "..sss....",
        "..ggg....",
        ".ggggg...",
        "ggggggg..",
        ".gg.gg...",
        "..g.g....",
        "..g.g....",
    ],
    &[
        "...h.h...",
        "..hhhhh..",
        "..hhhhhh.",
        "...sssss.",
        "..sskks..",
        "...sss...",
        "...ggg...",
        "..gggggg.",
        "..ggggg..",
        "..gg.gg..",
        "..g...g..",
        "..g...g..",
    ],
];

const KUNG_FRAMES: [&[&str]; 2] = [
    &[
        "..mmmm...",
        ".mmmmm...",
        "..rrr....",
        ".sssss...",
        ".skkss...",
        "..sss....",
        "..ggg..m.",
        ".ggggg.m.",
        "gg.ggg.m.",
        ".g.g.g...",
        "..g.g....",
        "..g.g....",
    ],
    &[
        "..mmmm...",
        ".mmmmm...",
        "..rrr....",
        "...sssss.",
        "..sskks..",
        "...sss...",
        ".m..ggg..",
        ".m.ggggg.",
        ".mggg.gg.",
        "...g.g...",
        "..g...g..",
        "..g...g..",
    ],
];

const CONDOR_FRAMES: [&[&str]; 2] = [
    &[
        "...d.......d...",
        "..ddd.....ddd..",
        ".dddddddddddddd",
        "ddddddddddddddd",
        "...d.kddddk.d..",
        ".....dddddd....",
        "......dddd.....",
    ],
    &[
        "....dd...dd....",
        "...dddddddddd..",
        "..dddddddddddd.",
        "..dddddddddddd.",
        "....kddddddk...",
        "......dddd.....",
        "......d..d.....",
    ],
];

const GUANACO_FRAMES: [&[&str]; 2] = [
    &[
        "...bb...", "..bbbb..", ".bbbbbb.", "bbbbbbh.", ".b..b...", ".l..l...",
    ],
    &[
        "...bb...", "..bbbb..", ".bbbbbb.", "bbbbbbh.", "..b..b..", ".l...l..",
    ],
];

fn palette_map(pairs: &[(&str, &str)], pal: &Palette) -> Vec<(char, Rgb)> {
    pairs
        .iter()
        .map(|(ch, key)| {
            let col = if let Some(hex) = key.strip_prefix('#') {
                hex_to_rgb(hex)
            } else {
                get(pal, key)
            };
            (ch.chars().next().unwrap(), col)
        })
        .collect()
}

fn draw_matrix(
    fb: &mut Framebuffer,
    rows: &[&str],
    colors: &[(char, Rgb)],
    left: i32,
    top: i32,
    flip: bool,
) {
    let w = rows
        .iter()
        .map(|row| row.chars().count())
        .max()
        .unwrap_or(0) as i32;
    for (ry, row) in rows.iter().enumerate() {
        for (rx, ch) in row.chars().enumerate() {
            if ch == '.' || ch == ' ' {
                continue;
            }
            let Some((_, col)) = colors.iter().find(|(c, _)| *c == ch) else {
                continue;
            };
            let dx = if flip { w - 1 - rx as i32 } else { rx as i32 };
            fb.fill_rect(left + dx, top + ry as i32, 1, 1, *col, 1.0);
        }
    }
}

fn glyph(ch: char) -> Option<[&'static str; 5]> {
    Some(match ch {
        'A' => ["###", "#.#", "###", "#.#", "#.#"],
        'B' => ["##.", "#.#", "##.", "#.#", "##."],
        'I' => ["###", ".#.", ".#.", ".#.", "###"],
        'K' => ["#.#", "#.#", "##.", "#.#", "#.#"],
        'L' => ["#..", "#..", "#..", "#..", "###"],
        'M' => ["#.#", "###", "###", "#.#", "#.#"],
        'O' => ["###", "#.#", "#.#", "#.#", "###"],
        'R' => ["##.", "#.#", "##.", "#.#", "#.#"],
        'T' => ["###", ".#.", ".#.", ".#.", ".#."],
        'U' => ["#.#", "#.#", "#.#", "#.#", "###"],
        ' ' => ["...", "...", "...", "...", "..."],
        _ => return None,
    })
}

pub fn draw_text(fb: &mut Framebuffer, text: &str, left: i32, top: i32, color: Rgb, scale: i32) {
    let mut cx = left;
    for raw in text.to_uppercase().chars() {
        if let Some(g) = glyph(raw) {
            for (ry, row) in g.iter().enumerate() {
                for (rx, c) in row.chars().enumerate() {
                    if c != '#' {
                        continue;
                    }
                    fb.fill_rect(
                        cx + rx as i32 * scale,
                        top + ry as i32 * scale,
                        scale,
                        scale,
                        color,
                        1.0,
                    );
                }
            }
        }
        cx += 4 * scale;
    }
}

fn draw_character(
    fb: &mut Framebuffer,
    rows: &[&str],
    pairs: &[(&str, &str)],
    pal: &Palette,
    left: i32,
    top: i32,
    flip: bool,
) {
    let colors = palette_map(pairs, pal);
    draw_matrix(fb, rows, &colors, left, top, flip);
}

fn draw_leo(
    fb: &mut Framebuffer,
    pal: &Palette,
    left: i32,
    top: i32,
    phase: f64,
    dir: f64,
    t_sec: f64,
) {
    let sway = r((t_sec * 3.0 + phase).sin() * 1.0);
    let n = LEO_FRAMES.len() as f64;
    let fi = (((t_sec * 3.0 + phase).floor() % n) + n) % n;
    draw_character(
        fb,
        LEO_FRAMES[fi as usize],
        LEO_PAL,
        pal,
        left + sway,
        top,
        dir < 0.0,
    );
    let star = get(pal, "star");
    for i in 0..3 {
        let p = (t_sec * 0.6 + i as f64 / 3.0 + phase * 0.1) % 1.0;
        let nx = left as f64
            + if dir < 0.0 {
                -4.0 - i as f64 * 2.0
            } else {
                9.0 + i as f64 * 2.0
            };
        let ny = top - r(p * 16.0);
        let a = 0.85 * (1.0 - p);
        fb.fill_rect(r(nx), ny, 1, 1, star, a);
        fb.fill_rect(r(nx), ny - 1, 1, 1, star, a);
        fb.fill_rect(r(nx) + 1, ny - 2, 1, 1, star, a);
    }
}

fn draw_kung_leo(
    fb: &mut Framebuffer,
    pal: &Palette,
    left: i32,
    top: i32,
    phase: f64,
    dir: f64,
    t_sec: f64,
) {
    let n = KUNG_FRAMES.len() as f64;
    let fi = (((t_sec * 6.0 + phase).floor() % n) + n) % n;
    draw_character(
        fb,
        KUNG_FRAMES[fi as usize],
        KUNG_PAL,
        pal,
        left,
        top,
        dir < 0.0,
    );
    let reach = 16.0 + (t_sec * 2.0 + phase).sin() * 8.0;
    let hx = r(left as f64 + if dir < 0.0 { -reach } else { 9.0 + reach });
    let hy = r(top as f64 - 4.0 + (t_sec * 3.0 + phase).sin() * 2.0);
    fb.fill_rect(hx, hy, 1, 1, [0xc0, 0xc6, 0xd0], 1.0);
    fb.fill_rect(hx + 1, hy, 1, 1, [0xe2, 0xe6, 0xee], 1.0);
    fb.fill_rect(hx + 2, hy, 1, 1, [0xc0, 0xc6, 0xd0], 1.0);
    if (t_sec * 2.4 + phase).sin() > 0.1 {
        let text = "MORTAL KUMBIA";
        let w = text.chars().count() as i32 * 4 - 1;
        draw_text(
            fb,
            text,
            r(left as f64 - w as f64 / 2.0 + 4.0),
            r(top as f64 - 22.0),
            get(pal, "sun"),
            1,
        );
    }
}

fn condor_frame(t: f64, phase: f64, i: i32) -> usize {
    let v = (t * 2.2 + phase + i as f64 * 0.5).floor() % 2.0;
    (((v + 2.0) % 2.0) as usize).min(1)
}

fn draw_condor_flight(
    fb: &mut Framebuffer,
    pal: &Palette,
    ev: MomentEvent,
    camera_x: f64,
    w: usize,
    t_sec: f64,
    layer_idx: usize,
) {
    let layer = &LAYERS[layer_idx];
    let base = ev.wx - camera_x * layer.parallax;
    if base < -80.0 || base > w as f64 + 80.0 {
        return;
    }
    let s = hash1(r(ev.wx), 0x3e0) * std::f64::consts::PI * 2.0;
    let x = base + (t_sec * 0.22 + s).sin() * w as f64 * 0.32;
    let y = 26.0 + (t_sec * 0.6 + s).sin() * 6.0;
    draw_character(
        fb,
        CONDOR_FRAMES[condor_frame(t_sec, s, 0)],
        CONDOR_PAL,
        pal,
        r(x),
        r(y),
        false,
    );
    for i in 1..=2 {
        draw_character(
            fb,
            CONDOR_FRAMES[condor_frame(t_sec, s, i)],
            CONDOR_PAL,
            pal,
            r(x - i as f64 * 11.0),
            r(y - 3.0 - i as f64 * 2.0),
            false,
        );
    }
}

fn draw_bandada(
    fb: &mut Framebuffer,
    pal: &Palette,
    ev: MomentEvent,
    camera_x: f64,
    w: usize,
    t_sec: f64,
    layer_idx: usize,
) {
    let layer = &LAYERS[layer_idx];
    let base = ev.wx - camera_x * layer.parallax;
    if base < -100.0 || base > w as f64 + 100.0 {
        return;
    }
    let s = hash1(r(ev.wx), 0x4e0) * std::f64::consts::PI * 2.0;
    let x = base + (t_sec * 0.18 + s).sin() * w as f64 * 0.3;
    let y = 38.0 + (t_sec * 0.5 + s).sin() * 5.0;
    let trunk = get(pal, "trunk");
    for i in 0..7 {
        let side = if i % 2 != 0 { 1.0 } else { -1.0 };
        let k = ((i + 1) / 2) as f64;
        let bx = r(x + side * k * 5.0);
        let by = r(y + k * 3.0);
        let flap = if (t_sec * 6.0 + i as f64).sin() > 0.0 {
            0
        } else {
            1
        };
        fb.fill_rect(bx, by, 1, 1, trunk, 1.0);
        fb.fill_rect(bx - 1, by + 1 + flap, 1, 1, trunk, 1.0);
        fb.fill_rect(bx + 1, by + 1 + flap, 1, 1, trunk, 1.0);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_manada(
    fb: &mut Framebuffer,
    pal: &Palette,
    ev: MomentEvent,
    camera_x: f64,
    w: usize,
    t_sec: f64,
    terrain: &Terrain,
    layer_idx: usize,
) {
    let layer = &LAYERS[layer_idx];
    let s = hash1(r(ev.wx), 0x5e0) * std::f64::consts::PI * 2.0;
    for i in 0..3 {
        let wx = ev.wx + (i as f64 - 1.0) * 9.0;
        let left = r(wx - camera_x * layer.parallax);
        if left < -20 || left > w as i32 + 20 {
            continue;
        }
        let top =
            r(terrain.bank_height(layer_idx, wx) - 6.0 + (t_sec * 4.0 + i as f64).sin() * 0.5);
        let fi = (((t_sec * 4.0 + s + i as f64 * 0.3).floor() % 2.0) + 2.0) % 2.0;
        draw_character(
            fb,
            GUANACO_FRAMES[fi as usize],
            GUANACO_PAL,
            pal,
            left,
            top,
            false,
        );
    }
}

const LEO_PAL: &[(&str, &str)] = &[
    ("h", "#1a1a22"),
    ("s", "#d9a066"),
    ("g", "#f2c14e"),
    ("k", "#141419"),
];
const KUNG_PAL: &[(&str, &str)] = &[
    ("h", "#1a1a22"),
    ("s", "#d9a066"),
    ("g", "#f2c14e"),
    ("r", "#c62828"),
    ("m", "#c0c6d0"),
];
const CONDOR_PAL: &[(&str, &str)] = &[("d", "trunk"), ("k", "snow")];
const GUANACO_PAL: &[(&str, &str)] = &[("b", "trunk"), ("l", "sand"), ("h", "trunk")];

#[allow(clippy::too_many_arguments)]
pub fn moment_sky(
    fb: &mut Framebuffer,
    pal: &Palette,
    camera_x: f64,
    w: usize,
    h: usize,
    seed: u32,
    t_sec: f64,
    mode: &str,
) {
    if mode == "none" {
        return;
    }
    let evs = events_for(mode, seed, camera_x, w);
    let Some(ci) = costa_index() else { return };
    for ev in evs {
        match ev.type_ {
            "18sep" => patriotic_sky(fb, w, h, t_sec),
            "condor" => draw_condor_flight(fb, pal, ev, camera_x, w, t_sec, ci),
            "bandada" => draw_bandada(fb, pal, ev, camera_x, w, t_sec, ci),
            _ => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn moment_ground(
    fb: &mut Framebuffer,
    pal: &Palette,
    camera_x: f64,
    w: usize,
    h: usize,
    seed: u32,
    t_sec: f64,
    mode: &str,
    terrain: &Terrain,
) {
    let _ = h;
    if mode == "none" {
        return;
    }
    let Some(ci) = costa_index() else { return };
    let layer = &LAYERS[ci];
    let p = layer.parallax;
    for ev in events_for(mode, seed, camera_x, w) {
        if ev.type_ == "manada" {
            draw_manada(fb, pal, ev, camera_x, w, t_sec, terrain, ci);
            continue;
        }
        if ev.type_ != "leorey" && ev.type_ != "kungleo" {
            continue;
        }
        let left = r(ev.wx - camera_x * p);
        if left < -40 || left > w as i32 + 40 {
            continue;
        }
        let ground = terrain.bank_height(ci, ev.wx);
        let top = r(ground - 12.0);
        let phase = hash1(r(ev.wx), 0x1e0) * std::f64::consts::PI * 2.0;
        let dir = if hash1(r(ev.wx), 0x2e0) < 0.5 {
            -1.0
        } else {
            1.0
        };
        if ev.type_ == "leorey" {
            draw_leo(fb, pal, left, top, phase, dir, t_sec);
        } else {
            draw_kung_leo(fb, pal, left, top, phase, dir, t_sec);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::get_palette;

    const SEED: u32 = 662118718;

    #[test]
    fn events_are_deterministic() {
        assert_eq!(
            moment_events(SEED, 0.0, 960).len(),
            moment_events(SEED, 0.0, 960).len()
        );
    }

    #[test]
    fn none_draws_nothing() {
        let pal = get_palette(12.0, "clear", 1.0);
        let mut fb = Framebuffer::new(960, 270);
        let t = Terrain::new(SEED);
        moment_sky(&mut fb, &pal, 0.0, 960, 270, SEED, 2.0, "none");
        moment_ground(&mut fb, &pal, 0.0, 960, 270, SEED, 2.0, "none", &t);
        assert!(fb.px.iter().all(|p| *p == 0));
    }

    #[test]
    fn forced_moments_draw_deterministically() {
        let pal = get_palette(12.0, "clear", 1.0);
        let t = Terrain::new(SEED);
        for mode in ["leorey", "kungleo", "18sep", "condor", "bandada", "manada"] {
            let mut a = Framebuffer::new(960, 270);
            let mut b = Framebuffer::new(960, 270);
            moment_sky(&mut a, &pal, 0.0, 960, 270, SEED, 2.0, mode);
            moment_ground(&mut a, &pal, 0.0, 960, 270, SEED, 2.0, mode, &t);
            moment_sky(&mut b, &pal, 0.0, 960, 270, SEED, 2.0, mode);
            moment_ground(&mut b, &pal, 0.0, 960, 270, SEED, 2.0, mode, &t);
            assert_eq!(a.px, b.px, "{mode} no es determinista");
            assert!(a.px.iter().any(|p| *p != 0), "{mode} no dibujó");
        }
    }
}
