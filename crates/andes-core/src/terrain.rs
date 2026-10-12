//! Geografía de Chile central: 6 capas con parallax, alturas, canales y dibujo
//! sobre un `Framebuffer` (puerto de `src/terrain.js`).

use std::collections::HashMap;

use crate::biomes::{biome_geometry, mode_weights};
use crate::framebuffer::Framebuffer;
use crate::noise::{fbm1, ridged1};
use crate::palette::{get, lerp_color, shade, Palette, Rgb};
use crate::rng::hash1;

pub const SEA_Y: f64 = 242.0;

#[derive(Clone, Copy)]
pub struct Channel {
    pub spacing: f64,
    pub chance: f64,
    pub width: f64,
    pub depth: f64,
    pub wfreq: f64,
}

#[derive(Clone, Copy)]
pub struct Volcano {
    pub spacing: f64,
    pub height: f64,
}

#[derive(Clone, Copy)]
pub struct FloraCfg {
    pub chunk_w: f64,
    pub min_size: f64,
    pub max_size: f64,
    pub min_chance: f64,
    pub max_per: f64,
}

const fn flora(
    chunk_w: f64,
    min_size: f64,
    max_size: f64,
    min_chance: f64,
    max_per: f64,
) -> FloraCfg {
    FloraCfg {
        chunk_w,
        min_size,
        max_size,
        min_chance,
        max_per,
    }
}

#[derive(Clone, Copy)]
pub struct FaunaCfg {
    pub chunk_w: f64,
    pub chance: f64,
}

const fn fauna(chunk_w: f64, chance: f64) -> FaunaCfg {
    FaunaCfg { chunk_w, chance }
}

pub struct Layer {
    pub name: &'static str,
    pub parallax: f64,
    pub base_y: f64,
    pub amp: f64,
    pub freq: f64,
    pub seed: u32,
    pub rugged: f64,
    pub snow_frac: f64,
    pub light_key: &'static str,
    pub dark_key: &'static str,
    pub alpha: f64,
    pub volcano: Option<Volcano>,
    pub rocky: bool,
    pub fields: bool,
    pub beach: bool,
    pub sea: bool,
    pub rivers: Option<Channel>,
    pub fjords: Option<Channel>,
    pub darken: f64,
    pub flora: Option<FloraCfg>,
    pub fauna: Option<FaunaCfg>,
}

const fn ch(spacing: f64, chance: f64, width: f64, depth: f64, wfreq: f64) -> Channel {
    Channel {
        spacing,
        chance,
        width,
        depth,
        wfreq,
    }
}

pub const LAYERS: [Layer; 6] = [
    Layer {
        name: "andes",
        parallax: 0.07,
        base_y: 128.0,
        amp: 92.0,
        freq: 0.0032,
        seed: 101,
        rugged: 0.78,
        snow_frac: 0.22,
        light_key: "farL",
        dark_key: "farD",
        alpha: 0.8,
        volcano: Some(Volcano {
            spacing: 600.0,
            height: 58.0,
        }),
        rocky: true,
        fields: false,
        beach: false,
        sea: false,
        rivers: None,
        fjords: None,
        darken: 0.0,
        flora: None,
        fauna: Some(fauna(220.0, 0.25)),
    },
    Layer {
        name: "precordillera",
        parallax: 0.16,
        base_y: 158.0,
        amp: 50.0,
        freq: 0.0060,
        seed: 211,
        rugged: 0.7,
        snow_frac: 0.94,
        light_key: "midL",
        dark_key: "midD",
        alpha: 0.9,
        volcano: None,
        rocky: false,
        fields: false,
        beach: false,
        sea: false,
        rivers: None,
        fjords: None,
        darken: 0.0,
        flora: Some(flora(84.0, 5.0, 10.0, 0.2, 1.0)),
        fauna: Some(fauna(200.0, 0.3)),
    },
    Layer {
        name: "valle",
        parallax: 0.32,
        base_y: 200.0,
        amp: 20.0,
        freq: 0.012,
        seed: 307,
        rugged: 0.12,
        snow_frac: 1.4,
        light_key: "valleyL",
        dark_key: "valleyD",
        alpha: 0.97,
        volcano: None,
        rocky: false,
        fields: true,
        beach: false,
        sea: false,
        rivers: Some(ch(1100.0, 0.45, 3.2, 7.0, 4.2)),
        fjords: Some(ch(64.0, 0.6, 5.0, 11.0, 5.0)),
        darken: 0.0,
        flora: Some(flora(40.0, 5.0, 12.0, 0.75, 5.0)),
        fauna: Some(fauna(150.0, 0.5)),
    },
    Layer {
        name: "costa",
        parallax: 0.52,
        base_y: 226.0,
        amp: 12.0,
        freq: 0.010,
        seed: 419,
        rugged: 0.05,
        snow_frac: 1.4,
        light_key: "costaL",
        dark_key: "costaD",
        alpha: 1.0,
        volcano: None,
        rocky: false,
        fields: false,
        beach: false,
        sea: false,
        rivers: Some(ch(1500.0, 0.4, 2.1, 5.0, 4.5)),
        fjords: Some(ch(60.0, 0.62, 4.5, 8.0, 5.0)),
        darken: 0.0,
        flora: Some(flora(44.0, 8.0, 18.0, 0.8, 4.0)),
        fauna: Some(fauna(160.0, 0.55)),
    },
    Layer {
        name: "playa",
        parallax: 0.78,
        base_y: 232.0,
        amp: 7.0,
        freq: 0.020,
        seed: 523,
        rugged: 0.1,
        snow_frac: 1.4,
        light_key: "sand",
        dark_key: "sandD",
        alpha: 1.0,
        volcano: None,
        rocky: false,
        fields: false,
        beach: true,
        sea: false,
        rivers: None,
        fjords: None,
        darken: 0.0,
        flora: Some(flora(50.0, 4.0, 8.0, 0.5, 3.0)),
        fauna: Some(fauna(200.0, 0.3)),
    },
    Layer {
        name: "mar",
        parallax: 1.0,
        base_y: SEA_Y,
        amp: 6.0,
        freq: 0.05,
        seed: 631,
        rugged: 0.2,
        snow_frac: 1.4,
        light_key: "sea",
        dark_key: "seaD",
        alpha: 1.0,
        volcano: None,
        rocky: false,
        fields: false,
        beach: false,
        sea: true,
        rivers: None,
        fjords: None,
        darken: 0.0,
        flora: None,
        fauna: Some(fauna(240.0, 0.35)),
    },
];

#[derive(Clone)]
struct Column {
    a: f64,
    y: i64,
    jitter: i64,
    rock: Option<[(i64, bool); 2]>,
}

pub struct Terrain {
    pub seed: u32,
    pub layer_seeds: [u32; 6],
    /// Modo de bioma para la geometría (`None` = sin sampler, como los tests).
    pub biome_mode: Option<String>,
    /// Desplazamiento de nieve por estación (se suma al del bioma).
    pub snow_shift_extra: f64,
    col_cache: HashMap<(usize, i64), Column>,
    field_cache: HashMap<usize, Vec<f64>>,
}

impl Terrain {
    pub fn new(seed: u32) -> Self {
        let mut t = Self {
            seed,
            layer_seeds: [0; 6],
            biome_mode: None,
            snow_shift_extra: 0.0,
            col_cache: HashMap::new(),
            field_cache: HashMap::new(),
        };
        t.seed_layers(seed);
        t
    }

    pub fn seed_layers(&mut self, seed: u32) {
        for (i, l) in LAYERS.iter().enumerate() {
            self.layer_seeds[i] = l.seed.wrapping_add(seed.wrapping_mul(0x9e37_79b1));
        }
        self.clear_caches();
    }

    fn s(&self, layer_idx: usize) -> u32 {
        self.layer_seeds[layer_idx]
    }

    pub fn set_biome_mode(&mut self, mode: Option<&str>) {
        self.biome_mode = mode.map(|m| m.to_string());
        self.clear_caches();
    }

    pub fn clear_caches(&mut self) {
        self.col_cache.clear();
        self.field_cache.clear();
    }

    fn amp_mul_at(&self, wx: f64) -> f64 {
        match &self.biome_mode {
            Some(mode) => biome_geometry(&mode_weights(mode, wx, self.seed)).amp_mul,
            None => 1.0,
        }
    }

    fn snow_shift_at(&self, wx: f64) -> f64 {
        let biome = match &self.biome_mode {
            Some(mode) => biome_geometry(&mode_weights(mode, wx, self.seed)).snow_shift,
            None => 0.0,
        };
        biome + self.snow_shift_extra
    }

    fn fjord_at(&self, wx: f64) -> f64 {
        match &self.biome_mode {
            Some(mode) => biome_geometry(&mode_weights(mode, wx, self.seed)).fjord,
            None => 0.0,
        }
    }

    fn base_noise(&self, layer_idx: usize, layer: &Layer, wx: f64) -> f64 {
        let s = self.s(layer_idx);
        let f = fbm1(wx * layer.freq + s as f64 * 0.37, s, 4, 2.0, 0.5);
        let r = ridged1(wx * layer.freq * 1.7 + s as f64, s.wrapping_add(5), 3);
        let rug = layer.rugged;
        f * (1.0 - rug) + r * rug
    }

    fn volcano_add(&self, layer_idx: usize, layer: &Layer, wx: f64) -> f64 {
        let Some(v) = layer.volcano else { return 0.0 };
        let s = self.s(layer_idx);
        let big = v.spacing;
        let c0 = (wx / big).floor() - 1.0;
        let mut add = 0.0f64;
        for k in 0..3 {
            let c = (c0 + k as f64) as i64;
            if hash1(c as i32, s.wrapping_add(555)) > 0.32 {
                continue;
            }
            let xc = c as f64 * big + hash1(c as i32, s.wrapping_add(556)) * big;
            let rad = big * (0.16 + hash1(c as i32, s.wrapping_add(557)) * 0.12);
            let d = (wx - xc).abs();
            if d > rad {
                continue;
            }
            let peak = v.height * (0.8 + hash1(c as i32, s.wrapping_add(558)) * 0.6);
            add = add.max(peak * (1.0 - d / rad).powf(1.1));
        }
        add
    }

    /// Altura del terreno sin el tallado del canal (nivel de los bancos).
    pub fn bank_height(&self, layer_idx: usize, wx: f64) -> f64 {
        let layer = &LAYERS[layer_idx];
        let n = self.base_noise(layer_idx, layer, wx);
        layer.base_y
            - (n * layer.amp * self.amp_mul_at(wx) + self.volcano_add(layer_idx, layer, wx))
    }

    fn channel_influence(&self, r: Option<&Channel>, wx: f64, seed: u32, salt: u32) -> f64 {
        let Some(r) = r else { return 0.0 };
        let big = r.spacing;
        let c0 = (wx / big).floor() - 1.0;
        let half_w = r.width * 2.2;
        let mut infl = 0.0f64;
        for k in 0..3 {
            let c = (c0 + k as f64) as i64;
            if hash1(c as i32, seed.wrapping_add(salt)) > r.chance {
                continue;
            }
            let xc =
                c as f64 * big + hash1(c as i32, seed.wrapping_add(salt).wrapping_add(1)) * big;
            let d = (wx - xc).abs();
            if d > half_w {
                continue;
            }
            infl = infl.max((1.0 - d / half_w).powf(1.6));
        }
        infl
    }

    /// Influencia `0..1` del cauce en una columna (río y, con austral, fiordos).
    pub fn river_influence(&self, layer_idx: usize, wx: f64) -> f64 {
        let layer = &LAYERS[layer_idx];
        let s = self.s(layer_idx);
        let mut infl = self.channel_influence(layer.rivers.as_ref(), wx, s, 911);
        let st = self.fjord_at(wx);
        if st > 0.0 {
            infl = infl.max(
                self.channel_influence(layer.fjords.as_ref(), wx, s.wrapping_add(7), 411) * st,
            );
        }
        infl
    }

    fn river_carve(&self, layer_idx: usize, layer: &Layer, wx: f64) -> f64 {
        let s = self.s(layer_idx);
        let mut carve = match &layer.rivers {
            Some(r) => r.depth * self.channel_influence(Some(r), wx, s, 911),
            None => 0.0,
        };
        let st = self.fjord_at(wx);
        if st > 0.0 {
            if let Some(f) = &layer.fjords {
                carve = carve.max(
                    f.depth * self.channel_influence(Some(f), wx, s.wrapping_add(7), 411) * st,
                );
            }
        }
        carve
    }

    pub fn ridge_height(&self, layer_idx: usize, wx: f64) -> f64 {
        let layer = &LAYERS[layer_idx];
        self.bank_height(layer_idx, wx) + self.river_carve(layer_idx, layer, wx)
    }

    /// Centros (mundo) de los eventos de río visibles para una cámara.
    pub fn river_event_centers(&self, layer_idx: usize, camera_x: f64, w: usize) -> Vec<f64> {
        self.river_events(layer_idx, camera_x, w)
            .into_iter()
            .map(|e| e.xc)
            .collect()
    }

    fn rock_vein(wx: f64, s: u32, vi: u32, depth: f64) -> (i64, bool) {
        let path = fbm1(
            wx * 0.02 + vi as f64 * 37.0,
            s.wrapping_add(41).wrapping_add(vi * 13),
            2,
            2.0,
            0.5,
        );
        ((path * depth).floor() as i64, path > 0.62)
    }

    fn column_at(&mut self, layer_idx: usize, wx: f64) -> Column {
        let key = wx.round() as i64;
        if let Some(c) = self.col_cache.get(&(layer_idx, key)) {
            return c.clone();
        }
        let layer = &LAYERS[layer_idx];
        let s = self.s(layer_idx);
        let a = layer.amp * self.amp_mul_at(wx);
        let y = self.ridge_height(layer_idx, wx).round() as i64;
        let jitter = if layer.snow_frac < 1.4 {
            (fbm1(wx * 0.22, s.wrapping_add(7), 2, 2.0, 0.5) * 6.0).floor() as i64
        } else {
            0
        };
        let rock = if layer.rocky {
            let depth = a * 0.72;
            Some([
                Self::rock_vein(wx, s, 0, depth),
                Self::rock_vein(wx, s, 1, depth),
            ])
        } else {
            None
        };
        let col = Column { a, y, jitter, rock };
        if self.col_cache.len() >= 4096 {
            self.col_cache.clear();
        }
        self.col_cache.insert((layer_idx, key), col.clone());
        col
    }

    fn field_mask(&mut self, layer_idx: usize, h: usize) -> Vec<f64> {
        if let Some(m) = self.field_cache.get(&layer_idx) {
            if m.len() == h {
                return m.clone();
            }
        }
        let s = self.s(layer_idx);
        let m: Vec<f64> = (0..h)
            .map(|row| {
                fbm1(
                    ((row as f64) * 0.14).floor(),
                    s.wrapping_add(71),
                    2,
                    2.0,
                    0.5,
                )
            })
            .collect();
        self.field_cache.insert(layer_idx, m.clone());
        m
    }

    pub fn draw_layer(
        &mut self,
        layer_idx: usize,
        fb: &mut Framebuffer,
        pal: &Palette,
        camera_x: f64,
        w: usize,
        h: usize,
    ) {
        let layer = &LAYERS[layer_idx];
        let p = layer.parallax;
        let mut light = get(pal, layer.light_key);
        let mut dark = get(pal, layer.dark_key);
        if layer.darken != 0.0 {
            light = shade(light, -layer.darken);
            dark = shade(dark, -layer.darken);
        }
        let snow = get(pal, "snow");
        let snow_d = get(pal, "snowD");
        let s = self.s(layer_idx);
        let rock_a = lerp_color(get(pal, "rock"), dark, 0.55);
        let rock_b = lerp_color(get(pal, "rockD"), dark, 0.6);
        let alpha = layer.alpha;

        let fields = if layer.fields {
            Some(self.field_mask(layer_idx, h))
        } else {
            None
        };

        for sx in 0..w {
            let wx = camera_x * p + sx as f64;
            let col = self.column_at(layer_idx, wx);
            let a = col.a;
            let sf = if layer.snow_frac < 1.4 {
                layer.snow_frac + self.snow_shift_at(wx)
            } else {
                layer.snow_frac
            };
            let snow_thr = a * sf;
            let band = (a * 0.22).max(2.0);
            let mut y = col.y;
            if y > h as i64 {
                y = h as i64;
            }
            let yi = y as i32;
            let sxi = sx as i32;

            fb.fill_rect(sxi, yi, 1, h as i32 - yi, dark, alpha);
            fb.fill_rect_f(sxi as f64, yi as f64, 1.0, band, light, alpha);

            let tx = hash1((wx.floor()) as i32, s);
            if tx > 0.82 {
                fb.fill_rect_f(
                    sxi as f64,
                    yi as f64 + band + (tx * 9.0).floor(),
                    1.0,
                    2.0,
                    light,
                    alpha,
                );
            } else if tx < 0.14 {
                fb.fill_rect_f(sxi as f64, yi as f64 + band + 2.0, 1.0, 3.0, dark, alpha);
            }

            let mut snow_bottom = 0i64;
            if snow_thr > 0.0 && sf < 1.4 {
                let hgt = layer.base_y - y as f64;
                if hgt > snow_thr {
                    let depth = (a * 0.36).min((hgt - snow_thr) * 0.9 + 2.0);
                    let d = ((depth - col.jitter as f64).round() as i64).max(1);
                    fb.fill_rect(sxi, yi, 1, d as i32, snow, alpha);
                    fb.fill_rect(
                        sxi,
                        yi + d as i32,
                        1,
                        1 + (col.jitter as f64 * 0.5).floor() as i32,
                        snow_d,
                        alpha,
                    );
                    snow_bottom = d + 1;
                }
            }

            if layer.rocky {
                let base = y + snow_bottom;
                if let Some(rock) = &col.rock {
                    for (vi, &(off, two)) in rock.iter().enumerate() {
                        let c = if vi == 0 { rock_a } else { rock_b };
                        fb.fill_rect(
                            sxi,
                            (base + off) as i32,
                            1,
                            if two { 2 } else { 1 },
                            c,
                            alpha * 0.75,
                        );
                    }
                }
            }

            if let Some(fields) = &fields {
                let mut row = y + 7;
                while row < h as i64 {
                    let m = fields[row as usize];
                    if m > 0.58 {
                        let c = if m > 0.74 {
                            get(pal, "costaL")
                        } else {
                            get(pal, "valleyL")
                        };
                        fb.fill_rect(sxi, row as i32, 1, 3, c, 0.35);
                    }
                    row += 7;
                }
            }
        }

        if layer.rivers.is_some() || layer.fjords.is_some() {
            self.draw_channel(layer_idx, fb, pal, camera_x, w, h);
        }

        if layer.beach {
            let c = shade(get(pal, "sandD"), -0.12);
            for sx in 0..w {
                let wx = camera_x * p + sx as f64;
                let y =
                    (SEA_Y - 5.0 + (fbm1(wx * 0.06, 321, 2, 2.0, 0.5) - 0.5) * 4.0).round() as i32;
                fb.fill_rect(sx as i32, y, 1, SEA_Y as i32 - y + 1, c, alpha);
            }
        }

        if layer.volcano.is_some() {
            self.draw_plumes(layer_idx, fb, pal, camera_x, w, h);
        }
    }

    fn draw_channel(
        &mut self,
        layer_idx: usize,
        fb: &mut Framebuffer,
        pal: &Palette,
        camera_x: f64,
        w: usize,
        h: usize,
    ) {
        let layer = &LAYERS[layer_idx];
        let p = layer.parallax;
        let water = lerp_color(get(pal, "sea"), get(pal, "seaHi"), 0.3);
        let bank = shade(water, -0.42);
        let sea_hi = get(pal, "seaHi");

        let mut evs = self.river_events(layer_idx, camera_x, w);
        evs.extend(self.fjord_events(layer_idx, camera_x, w));

        for ev in &evs {
            let r = if ev.kind == EventKind::Fjord {
                layer.fjords.as_ref().unwrap()
            } else {
                layer.rivers.as_ref().unwrap()
            };
            let sx0 = ev.xc - camera_x * p;
            let a = layer.amp * self.amp_mul_at(ev.xc);
            let top_ref = layer.base_y - a;
            let bot_ref = layer.base_y + a;
            let y0 = (top_ref.round() as i64).max(0);
            let y1 = (bot_ref.round() as i64).min(h as i64);
            let head_base = self.ridge_height(layer_idx, ev.xc);
            let head_y = head_base + 1.0 + hash1(ev.seed as i32, 0x51e) * 3.0;
            let run = (bot_ref - head_y).max(2.0);
            for y in y0..y1 {
                if (y as f64) < head_y - 1.0 {
                    continue;
                }
                let u = ((y as f64 - head_y) / run).clamp(0.0, 1.0);
                let hw = channel_half(r, ev.seed, u);
                let cx = sx0 + channel_offset_for(r, ev.seed, u);
                let left = (cx - hw).round() as i32;
                let right = (cx + hw).round() as i32;
                if right < -2 || left > w as i32 + 2 {
                    continue;
                }
                let rim = (y as f64 * 0.5 + ev.seed * 8.0).sin() > 0.55;
                let is_head = (y as f64) < head_y.ceil();
                for sx in (left - 1)..=(right + 1) {
                    if sx < 0 || sx >= w as i32 {
                        continue;
                    }
                    let wx = camera_x * p + sx as f64;
                    if (y as f64) < self.ridge_height(layer_idx, wx) {
                        continue;
                    }
                    let col = if is_head {
                        sea_hi
                    } else if sx <= left || sx >= right {
                        bank
                    } else if rim && (sx == left + 1 || sx == right - 1) {
                        sea_hi
                    } else {
                        water
                    };
                    fb.set(sx, y as i32, col);
                }
            }
        }
    }

    fn river_events(&self, layer_idx: usize, camera_x: f64, w: usize) -> Vec<Event> {
        let layer = &LAYERS[layer_idx];
        let Some(r) = &layer.rivers else {
            return Vec::new();
        };
        let p = layer.parallax;
        let s = self.s(layer_idx);
        let big = r.spacing;
        let c0 = ((camera_x * p - 80.0) / big).floor() as i64;
        let c1 = ((camera_x * p + w as f64 + 80.0) / big).floor() as i64;
        let mut out = Vec::new();
        for c in c0..=c1 {
            if hash1(c as i32, s.wrapping_add(911)) > r.chance {
                continue;
            }
            out.push(Event {
                xc: c as f64 * big + hash1(c as i32, s.wrapping_add(912)) * big,
                seed: hash1(c as i32, s.wrapping_add(913)),
                kind: EventKind::River,
            });
        }
        out
    }

    fn fjord_events(&self, layer_idx: usize, camera_x: f64, w: usize) -> Vec<Event> {
        if self.biome_mode.is_none() {
            return Vec::new();
        }
        let layer = &LAYERS[layer_idx];
        let Some(f) = &layer.fjords else {
            return Vec::new();
        };
        let p = layer.parallax;
        let s = self.s(layer_idx).wrapping_add(7);
        let big = f.spacing;
        let c0 = ((camera_x * p - 80.0) / big).floor() as i64;
        let c1 = ((camera_x * p + w as f64 + 80.0) / big).floor() as i64;
        let mut out = Vec::new();
        for c in c0..=c1 {
            if hash1(c as i32, s.wrapping_add(411)) > f.chance {
                continue;
            }
            let xc = c as f64 * big + hash1(c as i32, s.wrapping_add(412)) * big;
            let st = self.fjord_at(xc);
            if st <= 0.05 {
                continue;
            }
            out.push(Event {
                xc,
                seed: hash1(c as i32, s.wrapping_add(413)),
                kind: EventKind::Fjord,
            });
        }
        out
    }

    fn draw_plumes(
        &self,
        layer_idx: usize,
        fb: &mut Framebuffer,
        pal: &Palette,
        camera_x: f64,
        w: usize,
        _h: usize,
    ) {
        let layer = &LAYERS[layer_idx];
        let v = layer.volcano.unwrap();
        let p = layer.parallax;
        let s = self.s(layer_idx);
        let big = v.spacing;
        let start_c = ((camera_x * p - 40.0) / big).floor() as i64 - 1;
        let end_c = ((camera_x * p + w as f64 + 40.0) / big).floor() as i64 + 1;
        let col = get(pal, "cloudHi");
        for c in start_c..=end_c {
            if hash1(c as i32, s.wrapping_add(555)) > 0.32 {
                continue;
            }
            let xc = c as f64 * big + hash1(c as i32, s.wrapping_add(556)) * big;
            let sx = xc - camera_x * p;
            if sx < -60.0 || sx > w as f64 + 60.0 {
                continue;
            }
            let peak_y = self.ridge_height(layer_idx, xc);
            let seed = hash1(c as i32, s.wrapping_add(559));
            for i in 0..6 {
                let t = i as f64 / 6.0;
                let drift = (seed * 10.0 + i as f64 * 0.9).sin() * 4.0 + t * 10.0;
                let px = sx + drift;
                let py = peak_y - 4.0 - i as f64 * 5.0;
                let r = 2.4 + i as f64 * 0.7;
                fb.fill_ellipse(
                    px.round(),
                    py.round(),
                    r,
                    r * 0.8,
                    col,
                    0.22 * (1.0 - t * 0.7),
                );
            }
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum EventKind {
    River,
    Fjord,
}

struct Event {
    xc: f64,
    seed: f64,
    kind: EventKind,
}

fn channel_half(r: &Channel, seed: f64, u: f64) -> f64 {
    let taper = u.max(0.0).powf(0.8) * 1.4;
    let wobble = 0.12 * (u * r.wfreq + seed * 5.0).sin();
    (r.width * (taper + wobble)).max(0.4)
}

fn channel_offset_for(r: &Channel, seed: f64, u: f64) -> f64 {
    let ramp = (u / 0.2).min(1.0);
    r.width * 0.55 * (u * r.wfreq * 1.15 + seed * 6.3).sin() * ramp
}

/// El mar: oleaje, espuma y gradiente de profundidad, con reflejo del astro.
#[allow(clippy::too_many_arguments)]
pub fn draw_sea(
    fb: &mut Framebuffer,
    pal: &Palette,
    camera_x: f64,
    w: usize,
    h: usize,
    t_sec: f64,
    cel_x: Option<f64>,
    wind: f64,
) {
    let p = LAYERS[5].parallax;
    let amp = 1.0 + wind * 0.45;
    let speed = 1.0 + wind * 0.8;
    let sea = get(pal, "sea");
    let sea_d = get(pal, "seaD");
    let sea_hi = get(pal, "seaHi");
    let white: Rgb = [255, 255, 255];
    for sx in 0..w {
        let wx = camera_x * p + sx as f64;
        let wv = fbm1(wx * 0.035, 4321, 3, 2.0, 0.5);
        let top = (SEA_Y + (wv - 0.5) * 8.0 * amp).round() as i32;
        fb.fill_rect(sx as i32, top, 1, h as i32 - top, sea_d, 1.0);
        fb.fill_rect(sx as i32, top, 1, 9, sea, 1.0);
        let crest =
            (wx * 0.11 + t_sec * 1.4 * speed).sin() + (wx * 0.05 - t_sec * 0.9 * speed).sin();
        if crest > 0.9 - wind * 0.3 {
            fb.fill_rect(sx as i32, top, 1, 2, sea_hi, 1.0);
            if crest > 1.5 - wind * 0.45 {
                fb.fill_rect(sx as i32, top, 1, 1, white, 1.0);
            }
        }
        if wv > 0.72 && crest > 0.2 {
            fb.fill_rect(
                sx as i32,
                top + 3 + (wv * 4.0).floor() as i32,
                1,
                1,
                sea_hi,
                1.0,
            );
        }
        if wind > 0.55 && crest > 1.1 {
            fb.fill_rect(sx as i32, top - 1, 1, 1, white, 1.0);
        }
    }
    draw_reflection(fb, pal, w, h, t_sec, cel_x);
}

fn draw_reflection(
    fb: &mut Framebuffer,
    pal: &Palette,
    w: usize,
    h: usize,
    t_sec: f64,
    cel_x: Option<f64>,
) {
    let Some(cx) = cel_x else { return };
    let cx = cx.round();
    let col = lerp_color(get(pal, "sea"), get(pal, "sunGlow"), 0.55);
    let hi = lerp_color(col, get(pal, "sun"), 0.5);
    let max_half = 14.0;
    for y in SEA_Y as i32..h as i32 {
        let t = (y as f64 - SEA_Y) / (h as f64 - SEA_Y);
        let half = (max_half * (0.35 + t * 0.9)).round();
        for sx in (cx - half) as i32..=(cx + half) as i32 {
            if sx < 0 || sx >= w as i32 {
                continue;
            }
            let d = (sx as f64 - cx).abs() / (half + 1.0);
            let wave = (y as f64 * 0.7 + t_sec * 2.2 + sx as f64 * 0.35).sin();
            if wave < 0.35 {
                continue;
            }
            let a = (1.0 - d) * 0.28 * (0.5 + 0.5 * (t_sec * 1.3 + y as f64 * 0.5).sin());
            if a <= 0.02 {
                continue;
            }
            let c = if d < 0.3 { hi } else { col };
            fb.blend(sx, y, c, a);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: u32 = 662118718; // seedToInt("andes")

    #[test]
    fn seed_layers_is_deterministic() {
        let mut t = Terrain::new(SEED);
        let a = t.layer_seeds;
        t.seed_layers(SEED);
        assert_eq!(t.layer_seeds, a);
        t.seed_layers(999);
        assert_ne!(t.layer_seeds, a);
    }

    #[test]
    fn heights_are_deterministic_per_seed() {
        let mut t = Terrain::new(SEED);
        let h1: Vec<f64> = (0..6).map(|i| t.bank_height(i, 1234.5)).collect();
        t.seed_layers(SEED);
        let h2: Vec<f64> = (0..6).map(|i| t.bank_height(i, 1234.5)).collect();
        assert_eq!(h1, h2);
        t.seed_layers(1);
        assert_ne!(
            t.bank_height(0, 9876.0),
            Terrain::new(SEED).bank_height(0, 9876.0)
        );
    }

    #[test]
    fn ridge_never_above_bank() {
        let t = Terrain::new(SEED);
        for i in 0..6 {
            let mut wx = -2000.0;
            while wx < 2000.0 {
                assert!(t.ridge_height(i, wx) >= t.bank_height(i, wx) - 1e-9);
                wx += 17.3;
            }
        }
    }

    #[test]
    fn river_influence_in_range_and_zero_without_rivers() {
        let t = Terrain::new(SEED);
        for (i, layer) in LAYERS.iter().enumerate() {
            let has_rivers = layer.rivers.is_some();
            let mut max = 0.0f64;
            let mut wx = -2000.0;
            while wx < 4000.0 {
                let v = t.river_influence(i, wx);
                assert!((0.0..=1.0).contains(&v), "{}: {v}", LAYERS[i].name);
                max = max.max(v);
                wx += 3.1;
            }
            if !has_rivers {
                assert_eq!(max, 0.0);
            } else {
                assert!(max > 0.0);
            }
        }
    }

    #[test]
    fn column_cache_does_not_change_output() {
        let mut t = Terrain::new(SEED);
        let pal = crate::palette::get_palette(12.0, "clear", 1.0);
        let layer = 0;
        let mut fb1 = Framebuffer::new(480, 270);
        t.draw_layer(layer, &mut fb1, &pal, 0.0, 480, 270);
        let first: Vec<u8> = fb1.px.clone();
        let mut fb2 = Framebuffer::new(480, 270);
        t.draw_layer(layer, &mut fb2, &pal, 0.0, 480, 270);
        assert_eq!(fb2.px, first);
        t.clear_caches();
        let mut fb3 = Framebuffer::new(480, 270);
        t.draw_layer(layer, &mut fb3, &pal, 0.0, 480, 270);
        assert_eq!(fb3.px, first);
    }

    fn render_scene(biome: Option<&str>) -> Vec<u8> {
        let mut t = Terrain::new(SEED);
        t.set_biome_mode(biome);
        let pal = crate::palette::get_palette(12.0, "clear", 1.0);
        let mut fb = Framebuffer::new(960, 270);
        for (i, layer) in LAYERS.iter().enumerate() {
            if layer.sea {
                draw_sea(&mut fb, &pal, 12345.0, 960, 270, 3.0, Some(480.0), 0.2);
            } else {
                t.draw_layer(i, &mut fb, &pal, 12345.0, 960, 270);
            }
        }
        fb.px
    }

    #[test]
    fn render_smoke_all_layers() {
        for biome in [None, Some("austral"), Some("norte")] {
            let px = render_scene(biome);
            let nonblack = px.chunks(4).filter(|p| (p[0] | p[1] | p[2]) != 0).count();
            assert!(
                nonblack > 10000,
                "pocos píxeles pintados ({nonblack}) con {biome:?}"
            );
        }
    }

    #[test]
    fn render_is_deterministic() {
        assert_eq!(render_scene(Some("austral")), render_scene(Some("austral")));
    }
}
