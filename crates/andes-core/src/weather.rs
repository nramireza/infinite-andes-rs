//! Clima: partículas (nieve/lluvia/viento), niebla y tormenta eléctrica.
//! (puerto de `src/weather.js`).

use crate::framebuffer::Framebuffer;
use crate::math::js_round;
use crate::palette::{get, Palette, Rgb};
use crate::rng::Mulberry32;

const COUNTS: [(&str, usize); 6] = [
    ("snow", 230),
    ("rain", 260),
    ("wind", 90),
    ("fog", 0),
    ("clear", 0),
    ("storm", 170),
];
const LEAF: [Rgb; 4] = [
    [0x8a, 0x5a, 0x2a],
    [0xb0, 0x7a, 0x3a],
    [0x6a, 0x7a, 0x3a],
    [0x9a, 0x7a, 0x2a],
];
const RAIN: Rgb = [0xc8, 0xdc, 0xea];

#[derive(Clone, Copy, Debug)]
pub struct Particle {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub s: i32,
    pub len: i32,
    pub a: f64,
    pub leaf: bool,
    pub color: Rgb,
}

impl Particle {
    const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        vx: 0.0,
        vy: 0.0,
        s: 0,
        len: 0,
        a: 0.0,
        leaf: false,
        color: [0, 0, 0],
    };
}

pub fn wind_level_for(type_: &str) -> f64 {
    match type_ {
        "wind" => 1.0,
        "storm" => 0.7,
        "snow" => 0.4,
        "rain" => 0.25,
        "fog" => 0.1,
        _ => 0.05,
    }
}

fn count_for(type_: &str) -> usize {
    COUNTS
        .iter()
        .find(|(t, _)| *t == type_)
        .map(|(_, n)| *n)
        .unwrap_or(0)
}

pub struct Weather {
    pub type_: &'static str,
    pub strength: f64,
    pub transition: &'static str,
    pub prev_type: Option<&'static str>,
    pub prev_strength: f64,
    pub prev_particles: Vec<Particle>,
    pub particles: Vec<Particle>,
    pub fog_phase: f64,
    pub light_timer: f64,
    pub light_flash: f64,
    pub light_x: f64,
    pub light_bolt: Option<Vec<(i32, i32)>>,
    pub w: usize,
    pub h: usize,
    rng: Mulberry32,
}

impl Weather {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            type_: "clear",
            strength: 0.0,
            transition: "in",
            prev_type: None,
            prev_strength: 0.0,
            prev_particles: Vec::new(),
            particles: Vec::new(),
            fog_phase: 0.0,
            light_timer: 1.5,
            light_flash: 0.0,
            light_x: 0.0,
            light_bolt: None,
            w,
            h,
            rng: Mulberry32::new(12345),
        }
    }

    pub fn set_immediate(&mut self, type_: &'static str, w: usize, h: usize) {
        self.w = w;
        self.h = h;
        if type_ == self.type_ && self.prev_type.is_none() {
            self.build();
            self.strength = 0.0;
            self.transition = "in";
            return;
        }
        self.swap(type_);
    }

    pub fn request(&mut self, type_: &'static str) {
        if type_ == self.type_ && self.prev_type.is_none() && self.transition == "in" {
            return;
        }
        self.swap(type_);
    }

    fn swap(&mut self, type_: &'static str) {
        self.prev_type = Some(self.type_);
        self.prev_strength = self.strength;
        self.prev_particles = std::mem::take(&mut self.particles);
        self.type_ = type_;
        self.build();
        self.strength = 0.0;
        self.transition = "in";
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        self.w = w;
        self.h = h;
        self.build();
    }

    fn build(&mut self) {
        let n = count_for(self.type_);
        let mut ps = Vec::with_capacity(n);
        for _ in 0..n {
            ps.push(self.spawn(true));
        }
        self.particles = ps;
    }

    fn spawn(&mut self, anywhere: bool) -> Particle {
        let w = self.w as f64;
        let h = self.h as f64;
        let mut p = Particle::ZERO;
        match self.type_ {
            "snow" => {
                p.x = self.rng.next_f64() * w;
                p.y = if anywhere {
                    self.rng.next_f64() * h
                } else {
                    -4.0
                };
                p.vy = 10.0 + self.rng.next_f64() * 22.0;
                p.vx = (self.rng.next_f64() - 0.5) * 8.0;
                p.s = if self.rng.next_f64() < 0.35 { 2 } else { 1 };
                p.a = 0.5 + self.rng.next_f64() * 0.5;
            }
            "rain" => {
                p.x = self.rng.next_f64() * w;
                p.y = if anywhere {
                    self.rng.next_f64() * h
                } else {
                    -6.0
                };
                p.vy = 150.0 + self.rng.next_f64() * 120.0;
                p.len = 3 + (self.rng.next_f64() * 4.0).floor() as i32;
                p.a = 0.35 + self.rng.next_f64() * 0.4;
            }
            "storm" => {
                p.x = self.rng.next_f64() * w;
                p.y = if anywhere {
                    self.rng.next_f64() * h
                } else {
                    -8.0
                };
                p.vy = 200.0 + self.rng.next_f64() * 160.0;
                p.len = 4 + (self.rng.next_f64() * 4.0).floor() as i32;
                p.a = 0.45 + self.rng.next_f64() * 0.4;
            }
            _ => {
                p.x = if anywhere {
                    self.rng.next_f64() * w
                } else {
                    -10.0
                };
                p.y = self.rng.next_f64() * h;
                p.vx = 40.0 + self.rng.next_f64() * 90.0;
                p.vy = (self.rng.next_f64() - 0.5) * 12.0;
                p.len = 2 + (self.rng.next_f64() * 4.0).floor() as i32;
                p.leaf = self.rng.next_f64() < 0.3;
                p.color = LEAF[(self.rng.next_f64() * LEAF.len() as f64).floor() as usize];
                p.a = 0.3 + self.rng.next_f64() * 0.5;
            }
        }
        p
    }

    pub fn wind_level(&self) -> f64 {
        wind_level_for(self.type_)
    }

    pub fn effective_wind(&self) -> f64 {
        let mut w = wind_level_for(self.type_) * self.strength;
        if let Some(prev) = self.prev_type {
            w += wind_level_for(prev) * self.prev_strength;
        }
        w.min(1.0)
    }

    pub fn update(&mut self, dt: f64) {
        let k = (dt * 1.6).min(1.0);
        let target = if self.type_ == "clear" { 0.0 } else { 1.0 };
        self.strength += (target - self.strength) * k;
        if self.prev_type.is_some() {
            self.prev_strength += (0.0 - self.prev_strength) * k;
            if self.prev_strength <= 0.02 {
                self.prev_strength = 0.0;
                self.prev_type = None;
                self.prev_particles = Vec::new();
            }
        }
        self.fog_phase += dt * (6.0 + self.wind_level() * 26.0);
        if self.strength >= 0.01 {
            let t = self.type_;
            step_particles(t, &mut self.particles, &mut self.rng, self.w, self.h, dt);
        }
        if let Some(prev) = self.prev_type {
            if self.prev_strength >= 0.01 {
                step_particles(
                    prev,
                    &mut self.prev_particles,
                    &mut self.rng,
                    self.w,
                    self.h,
                    dt,
                );
            }
        }
        if self.type_ == "storm" || self.prev_type == Some("storm") {
            self.update_lightning(dt);
        }
    }

    fn update_lightning(&mut self, dt: f64) {
        self.light_flash = (self.light_flash - dt * 1.2).max(0.0);
        self.light_timer -= dt;
        if self.light_timer <= 0.0 {
            self.light_timer = 1.5 + self.rng.next_f64() * 5.0;
            self.light_flash = 0.18 + self.rng.next_f64() * 0.16;
            self.light_x = self.w as f64 * (0.15 + self.rng.next_f64() * 0.7);
            self.light_bolt = Some(self.make_bolt());
        }
    }

    fn make_bolt(&mut self) -> Vec<(i32, i32)> {
        let mut rows = Vec::new();
        let end = js_round(self.h as f64 * 0.66) as i32;
        let mut x = self.light_x;
        for y in 0..=end {
            let r = self.rng.next_f64();
            if r < 0.2 {
                x += if r < 0.1 { -2.0 } else { 2.0 };
            } else if r < 0.45 {
                x += if r < 0.275 { -1.0 } else { 1.0 };
            }
            rows.push((js_round(x) as i32, y));
            if y > 4 && self.rng.next_f64() < 0.05 {
                rows.push((js_round(x) as i32 + 3, y));
            }
        }
        rows
    }

    pub fn draw(&self, fb: &mut Framebuffer, pal: &Palette) {
        self.draw_particles(fb, self.type_, &self.particles, self.strength, pal);
        if let Some(prev) = self.prev_type {
            self.draw_particles(fb, prev, &self.prev_particles, self.prev_strength, pal);
        }
    }

    fn draw_particles(
        &self,
        fb: &mut Framebuffer,
        type_: &str,
        particles: &[Particle],
        s: f64,
        pal: &Palette,
    ) {
        if s < 0.01 || type_ == "clear" {
            return;
        }
        if type_ == "fog" {
            let fog = get(pal, "fog");
            fb.fill_rect(0, 0, fb.w as i32, fb.h as i32, fog, 0.16 * s);
            for i in 0..5 {
                let y = (fb.h as f64 * 0.45
                    + i as f64 * 26.0
                    + (self.fog_phase * 0.02 + i as f64).sin() * 5.0)
                    % (fb.h as f64 + 40.0)
                    - 20.0;
                fb.fill_rect(0, js_round(y) as i32, fb.w as i32, 14, fog, 0.10 * s);
            }
            return;
        }
        let wind = wind_level_for(type_);
        let white: Rgb = [255, 255, 255];
        for p in particles {
            let a = p.a * s;
            if type_ == "snow" {
                fb.fill_rect(
                    js_round(p.x) as i32,
                    js_round(p.y) as i32,
                    p.s,
                    p.s,
                    white,
                    a,
                );
            } else if type_ == "rain" || type_ == "storm" {
                let slant = wind * 1.6;
                for i in 0..p.len {
                    fb.fill_rect(
                        js_round(p.x - i as f64 * slant) as i32,
                        js_round(p.y + i as f64) as i32,
                        1,
                        1,
                        RAIN,
                        a,
                    );
                }
            } else if p.leaf {
                fb.fill_rect(js_round(p.x) as i32, js_round(p.y) as i32, 2, 1, p.color, a);
            } else {
                fb.fill_rect(
                    js_round(p.x) as i32,
                    js_round(p.y) as i32,
                    p.len,
                    1,
                    [220, 230, 210],
                    a * 0.6,
                );
            }
        }
    }

    pub fn draw_lightning(&self, fb: &mut Framebuffer, pal: &Palette) {
        let Some(bolt) = &self.light_bolt else { return };
        if self.light_flash <= 0.0 {
            return;
        }
        let a = self.light_flash.min(1.0);
        let cloud_hi = get(pal, "cloudHi");
        for &(x, y) in bolt {
            fb.fill_rect(x - 1, y, 3, 1, cloud_hi, 0.5 * a);
        }
        let white: Rgb = [255, 255, 255];
        for &(x, y) in bolt {
            fb.fill_rect(x, y, 1, 1, white, (a * 3.0).min(1.0));
        }
        fb.fill_rect(0, 0, fb.w as i32, fb.h as i32, white, 0.09 * a);
    }
}

fn step_particles(
    type_: &str,
    arr: &mut [Particle],
    rng: &mut Mulberry32,
    w: usize,
    h: usize,
    dt: f64,
) {
    if type_ == "fog" || type_ == "clear" {
        return;
    }
    let wind = wind_level_for(type_);
    let wf = w as f64;
    let hf = h as f64;
    for p in arr.iter_mut() {
        match type_ {
            "snow" => {
                p.y += p.vy * dt;
                p.x += (p.vx + wind * 26.0) * dt;
                if p.y > hf + 4.0 {
                    p.y = -4.0;
                    p.x = rng.next_f64() * wf;
                }
            }
            "rain" | "storm" => {
                p.y += p.vy * dt;
                p.x -= wind * 30.0 * dt;
                if p.y > hf + 6.0 {
                    p.y = -6.0;
                    p.x = rng.next_f64() * wf;
                }
            }
            _ => {
                p.x += p.vx * dt;
                p.y += p.vy * dt;
                if p.x > wf + 12.0 {
                    p.x = -12.0;
                    p.y = rng.next_f64() * hf;
                }
            }
        }
        if p.x < -20.0 {
            p.x += wf + 40.0;
        }
        if p.x > wf + 20.0 {
            p.x -= wf + 40.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::get_palette;

    #[test]
    fn wind_levels_are_ordered() {
        assert_eq!(wind_level_for("wind"), 1.0);
        assert!(wind_level_for("storm") > wind_level_for("rain"));
    }

    #[test]
    fn crossfade_keeps_two_climates() {
        let mut w = Weather::new(320, 180);
        w.set_immediate("snow", 320, 180);
        for _ in 0..240 {
            w.update(0.05);
        }
        assert!(w.strength > 0.95);
        w.request("rain");
        assert_eq!(w.type_, "rain");
        assert_eq!(w.prev_type, Some("snow"));
        assert!(w.prev_strength > 0.95);
        let mut guard = 0;
        while w.prev_type.is_some() && guard < 10000 {
            w.update(0.016);
            guard += 1;
        }
        assert!(w.prev_type.is_none());
        assert!(w.strength > 0.9);
    }

    #[test]
    fn request_same_climate_is_ignored() {
        let mut w = Weather::new(320, 180);
        w.set_immediate("rain", 320, 180);
        for _ in 0..240 {
            w.update(0.05);
        }
        w.request("rain");
        assert!(w.prev_type.is_none());
        assert!(w.strength > 0.9);
    }

    #[test]
    fn storm_makes_lightning() {
        let mut w = Weather::new(320, 180);
        w.set_immediate("storm", 320, 180);
        w.update(0.016);
        assert!(!w.particles.is_empty());
        let mut guard = 0;
        while w.light_flash <= 0.0 && guard < 2000 {
            w.update(0.1);
            guard += 1;
        }
        assert!(w.light_flash > 0.0);
        assert!(w.light_bolt.as_ref().unwrap().len() > 5);
        let mut fb = Framebuffer::new(320, 180);
        let pal = get_palette(15.0, "storm", 1.0);
        w.draw_lightning(&mut fb, &pal);
        assert!(fb.px.iter().any(|b| *b != 0));
    }

    #[test]
    fn render_is_deterministic() {
        let pal = get_palette(12.0, "snow", 1.0);
        let mut a = Weather::new(320, 180);
        a.set_immediate("snow", 320, 180);
        let mut b = Weather::new(320, 180);
        b.set_immediate("snow", 320, 180);
        let mut fa = Framebuffer::new(320, 180);
        let mut fb = Framebuffer::new(320, 180);
        a.draw(&mut fa, &pal);
        b.draw(&mut fb, &pal);
        assert_eq!(fa.px, fb.px);
    }
}
