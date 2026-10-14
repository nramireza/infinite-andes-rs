//! Cielo: gradiente, estrellas, aurora austral, astros (sol/luna) y nubes.
//! (puerto de `src/sky.js`).

use std::f64::consts::PI;

use crate::framebuffer::Framebuffer;
use crate::math::{js_round, to_uint32};
use crate::palette::{get, lerp_color, shade, Palette, Rgb};
use crate::rng::{hash1, Mulberry32};

pub const RISE: f64 = 6.5;
pub const SET: f64 = 18.5;
const BACK_HORIZON: f64 = 118.0;
const FRONT_HORIZON: f64 = 212.0;
const METEOR_PERIOD: f64 = 5.5;
const METEOR_DUR: f64 = 0.7;

#[derive(Clone, Copy, Debug)]
pub struct Meteor {
    pub x: f64,
    pub y: f64,
    pub dx: f64,
    pub dy: f64,
    pub alpha: f64,
}

/// Estrella fugaz determinista para un instante; `None` si no hay ninguna.
pub fn meteor_at(seed: u32, t_sec: f64) -> Option<Meteor> {
    if !t_sec.is_finite() || t_sec < 0.0 {
        return None;
    }
    let s = seed ^ 0x7e70;
    let block = (t_sec / METEOR_PERIOD).floor() as i32;
    if hash1(block, s) > 0.55 {
        return None;
    }
    let t0 = t_sec - block as f64 * METEOR_PERIOD;
    if t0 > METEOR_DUR {
        return None;
    }
    let p = t0 / METEOR_DUR;
    let dir = if hash1(block, s.wrapping_add(1)) < 0.5 {
        -1.0
    } else {
        1.0
    };
    let x0 = 0.12 + hash1(block, s.wrapping_add(2)) * 0.76;
    let y0 = 0.04 + hash1(block, s.wrapping_add(3)) * 0.34;
    let dx = dir * (0.05 + hash1(block, s.wrapping_add(4)) * 0.07);
    let dy = 0.03 + hash1(block, s.wrapping_add(5)) * 0.04;
    Some(Meteor {
        x: x0 + dx * p,
        y: y0 + dy * p,
        dx,
        dy,
        alpha: (p * PI).sin(),
    })
}

#[derive(Clone, Copy)]
struct Star {
    x: f64,
    y: f64,
    b: f64,
    p: f64,
}

#[derive(Clone, Copy)]
struct Cloud {
    x: f64,
    y: f64,
    w: f64,
    s: f64,
}

/// Estado de un astro. `depth` 0 = fondo (tras los Andes), 1 = frente (mar).
#[derive(Clone, Copy, Debug)]
pub struct Celestial {
    pub is_sun: bool,
    pub u: f64,
    pub x: f64,
    pub y: f64,
    pub r: f64,
    pub depth: f64,
    pub horizon_y: f64,
    pub rising: bool,
    pub visible: bool,
    pub phase: Option<f64>,
    pub illum: Option<f64>,
}

pub struct Sky {
    pub seed: u32,
    stars: Vec<Star>,
    clouds: Vec<Cloud>,
}

impl Sky {
    pub fn new(seed: u32) -> Self {
        // `mulberry32(hash1(...))`: en JS el seed se trunca a 32 bits y el hash
        // está en [0,1), así que `>>> 0` da 0 (estrellas iguales por semilla).
        let star_seed = to_uint32(hash1(1, seed ^ 0x51ed));
        let mut rng = Mulberry32::new(star_seed);
        let mut stars = Vec::with_capacity(110);
        for _ in 0..110 {
            let x = rng.next_f64();
            let y = rng.next_f64() * 0.62;
            let b = 0.3 + rng.next_f64() * 0.7;
            let p = rng.next_f64() * PI * 2.0;
            stars.push(Star { x, y, b, p });
        }
        let cloud_seed = to_uint32(hash1(2, seed ^ 0x77aa));
        let mut r2 = Mulberry32::new(cloud_seed);
        let mut clouds = Vec::with_capacity(14);
        for _ in 0..14 {
            let x = r2.next_f64() * 4000.0;
            let y = 18.0 + r2.next_f64() * 90.0;
            let w = 34.0 + r2.next_f64() * 90.0;
            let s = 0.5 + r2.next_f64() * 0.7;
            clouds.push(Cloud { x, y, w, s });
        }
        Self {
            seed,
            stars,
            clouds,
        }
    }

    /// Acceso a una estrella por índice `(x, y, brillo, fase)` (para paridad).
    pub fn star(&self, i: usize) -> (f64, f64, f64, f64) {
        let s = self.stars[i];
        (s.x, s.y, s.b, s.p)
    }

    /// Acceso a una nube por índice `(x, y, ancho, velocidad)` (para paridad).
    pub fn cloud(&self, i: usize) -> (f64, f64, f64, f64) {
        let c = self.clouds[i];
        (c.x, c.y, c.w, c.s)
    }

    pub fn celestial(
        &self,
        hour24: f64,
        w: f64,
        h: f64,
        rise: f64,
        set: f64,
        moon_phase: f64,
    ) -> Celestial {
        let hh = ((hour24 % 24.0) + 24.0) % 24.0;
        let (u, is_sun) = if hh >= rise && hh < set {
            ((hh - rise) / (set - rise), true)
        } else {
            let nh = if hh >= set { hh - set } else { hh + 24.0 - set };
            (nh / (24.0 - set + rise), false)
        };
        let horizon_y = BACK_HORIZON + u * (FRONT_HORIZON - BACK_HORIZON);
        let a = 245.0;
        let y = horizon_y - a * (u * PI).sin();
        let x = w * 0.5;
        let r = 3.0 + u * 4.0;
        let rising = u < 0.5;
        let visible = rising && y > -26.0 && y < h + 26.0;
        let mut cel = Celestial {
            is_sun,
            u,
            x,
            y,
            r,
            depth: u,
            horizon_y,
            rising,
            visible,
            phase: None,
            illum: None,
        };
        if !is_sun {
            let phase = ((moon_phase % 1.0) + 1.0) % 1.0;
            cel.phase = Some(phase);
            cel.illum = Some(1.0 - 2.0 * (phase - 0.5).abs());
        }
        cel
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        fb: &mut Framebuffer,
        pal: &Palette,
        night_amt: f64,
        t_sec: f64,
        cel: Option<&Celestial>,
        glow_tint: Option<(Rgb, f64)>,
        moon_dim: f64,
    ) {
        let w = fb.w;
        let h = fb.h;
        let horizon = js_round(h as f64 * 0.56) as i32;
        let sky_top = get(pal, "skyTop");
        let sky_mid = get(pal, "skyMid");
        let sky_horizon = get(pal, "skyHorizon");
        for y in 0..horizon {
            let t = (y as f64 + 0.5) / horizon as f64;
            let c = if t <= 0.62 {
                lerp_color(sky_top, sky_mid, t / 0.62)
            } else {
                lerp_color(sky_mid, sky_horizon, (t - 0.62) / 0.38)
            };
            fb.fill_rect(0, y, w as i32, 1, c, 1.0);
        }
        fb.fill_rect(0, horizon, w as i32, h as i32 - horizon, sky_horizon, 1.0);

        if night_amt > 0.15 {
            self.draw_milky_way(fb, pal, night_amt);
        }
        if night_amt > 0.02 {
            self.draw_stars(fb, pal, night_amt, t_sec, moon_dim);
        }
        if night_amt > 0.35 {
            self.draw_aurora(fb, night_amt, t_sec);
        }
        self.draw_shooting_stars(fb, pal, night_amt, t_sec);
        if let Some(cel) = cel {
            self.draw_glow(fb, pal, cel, glow_tint);
        }
    }

    pub fn draw_shooting_stars(
        &self,
        fb: &mut Framebuffer,
        pal: &Palette,
        night_amt: f64,
        t_sec: f64,
    ) {
        if night_amt <= 0.25 {
            return;
        }
        let Some(m) = meteor_at(self.seed, t_sec) else {
            return;
        };
        let span = fb.h as f64 * 0.56;
        let hx = m.x * fb.w as f64;
        let hy = m.y * span;
        let tx = hx - m.dx * fb.w as f64;
        let ty = hy - m.dy * span;
        let steps = 9;
        let snow = get(pal, "snow");
        let star = get(pal, "star");
        for i in 0..steps {
            let t = i as f64 / steps as f64;
            let x = js_round(hx + (tx - hx) * t) as i32;
            let y = js_round(hy + (ty - hy) * t) as i32;
            if x < 0 || x >= fb.w as i32 || y < 0 || y as f64 > span {
                continue;
            }
            let c = if i == 0 { snow } else { star };
            fb.blend(x, y, c, m.alpha * (1.0 - t) * night_amt);
        }
    }

    pub fn draw_glow(
        &self,
        fb: &mut Framebuffer,
        pal: &Palette,
        cel: &Celestial,
        tint: Option<(Rgb, f64)>,
    ) {
        if !cel.rising {
            return;
        }
        let dy = (cel.y - cel.horizon_y).abs();
        let low = (1.0 - dy / 90.0).max(0.0);
        if low <= 0.02 {
            return;
        }
        let h = fb.h as f64;
        let gy = cel.y.max(-10.0).min(h);
        let moon_dim = if cel.is_sun {
            1.0
        } else {
            0.35 + 0.65 * cel.illum.unwrap_or(1.0)
        };
        let base = (if cel.is_sun { 0.9 } else { 0.42 }) * moon_dim;
        let sun_glow = get(pal, "sunGlow");
        let glow_col = match tint {
            Some((color, amount)) if amount > 0.0 => lerp_color(sun_glow, color, amount),
            _ => sun_glow,
        };
        let layers = [
            (64.0, base * 0.05),
            (38.0, base * 0.1),
            (22.0, base * 0.2),
            (12.0, base * 0.34),
        ];
        for (r, a) in layers {
            fb.fill_disc(cel.x, gy, r, glow_col, a * low);
        }
    }

    pub fn draw_body(&self, fb: &mut Framebuffer, cel: &Celestial, pal: &Palette) {
        if !cel.visible {
            return;
        }
        let r = js_round(cel.r);
        fb.fill_disc(cel.x, cel.y, r + 4.0, get(pal, "sunGlow"), 0.22);
        let cx = js_round(cel.x) as i32;
        let cy = js_round(cel.y) as i32;
        if !cel.is_sun {
            let phase = cel.phase.unwrap_or(0.5);
            let illum = 1.0 - 2.0 * (phase - 0.5).abs();
            let sun = get(pal, "sun");
            if illum > 0.97 {
                fb.fill_disc(cx as f64, cy as f64, r, sun, 1.0);
            } else {
                let moon_dark = shade(sun, -0.55);
                fb.fill_disc(cx as f64, cy as f64, r, moon_dark, 1.0);
                let waxing = phase < 0.5;
                let ri = r as i32;
                for dy in -ri..=ri {
                    let half = ((r * r - (dy as f64) * (dy as f64)).max(0.0))
                        .sqrt()
                        .floor();
                    let width = js_round(2.0 * illum * half);
                    if width <= 0.0 {
                        continue;
                    }
                    let x0 = if waxing { half - width + 1.0 } else { -half };
                    fb.fill_rect(cx + x0 as i32, cy + dy, width as i32, 1, sun, 1.0);
                }
                if illum > 0.25 {
                    self.moon_craters(fb, cx, cy, pal, r, illum, waxing);
                }
            }
        } else {
            let sun = get(pal, "sun");
            fb.fill_disc(cx as f64, cy as f64, r, sun, 1.0);
            fb.fill_rect(cx - 2, cy - 2, 4, 4, get(pal, "sunGlow"), 1.0);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn moon_craters(
        &self,
        fb: &mut Framebuffer,
        cx: i32,
        cy: i32,
        pal: &Palette,
        r: f64,
        illum: f64,
        waxing: bool,
    ) {
        let sun_glow = get(pal, "sunGlow");
        let lit_range = |dy: i32| -> Option<(f64, f64)> {
            let half = ((r * r - (dy as f64) * (dy as f64)).max(0.0))
                .sqrt()
                .floor();
            let width = js_round(2.0 * illum * half);
            if width <= 0.0 {
                return None;
            }
            let x0 = if waxing { half - width + 1.0 } else { -half };
            Some((x0, x0 + width))
        };
        let inside = |dy: i32, x0: f64, w: f64| -> bool {
            match lit_range(dy) {
                Some((lo, hi)) => x0 >= lo && x0 + w <= hi,
                None => false,
            }
        };
        if inside(-1, -2.0, 1.0) {
            fb.fill_rect(cx - 2, cy - 1, 1, 1, sun_glow, 1.0);
        }
        if inside(2, 1.0, 2.0) {
            fb.fill_rect(cx + 1, cy + 2, 2, 1, sun_glow, 1.0);
        }
        if inside(3, -1.0, 1.0) {
            fb.fill_rect(cx - 1, cy + 3, 1, 1, sun_glow, 1.0);
        }
    }

    fn draw_milky_way(&self, fb: &mut Framebuffer, pal: &Palette, night_amt: f64) {
        let w = fb.w;
        let span = fb.h as f64 * 0.56;
        let s = self.seed ^ 0x9e37;
        let y_top = span * 0.60;
        let y_bot = span * 0.08;
        let star = get(pal, "star");
        let cloud = get(pal, "cloud");
        let layers = [
            (0.0, 0.075, star),
            (0.09, 0.05, star),
            (-0.09, 0.045, cloud),
        ];
        for (li, (off, a, col)) in layers.iter().enumerate() {
            let mut x = 0i32;
            while x < w as i32 {
                let t = x as f64 / w as f64;
                let axis = y_top + (y_bot - y_top) * t + (t * PI).sin() * span * 0.05;
                let cy = axis + off * span;
                let xs = x as u32;
                if hash1(xs as i32, s.wrapping_add(3000).wrapping_add(li as u32)) < 0.42 {
                    x += 2;
                    continue;
                }
                let h = 2
                    + (hash1(xs as i32, s.wrapping_add(6000).wrapping_add(li as u32)) * 3.0).floor()
                        as i32;
                let alpha = a
                    * night_amt
                    * (0.45 + hash1(xs as i32, s.wrapping_add(9000).wrapping_add(li as u32)) * 0.9);
                fb.fill_rect(x, js_round(cy) as i32, 2, h, *col, alpha);
                x += 2;
            }
        }
        let mut x = 0i32;
        while x < w as i32 {
            if hash1(x, s.wrapping_add(12000)) < 0.82 {
                x += 5;
                continue;
            }
            let t = x as f64 / w as f64;
            let axis = y_top + (y_bot - y_top) * t + (t * PI).sin() * span * 0.05;
            let y = axis + (hash1(x, s.wrapping_add(13000)) - 0.5) * span * 0.28;
            fb.fill_rect(x, js_round(y) as i32, 1, 1, star, 0.5 * night_amt);
            x += 5;
        }
    }

    fn draw_stars(
        &self,
        fb: &mut Framebuffer,
        pal: &Palette,
        night_amt: f64,
        t_sec: f64,
        moon_dim: f64,
    ) {
        let span = fb.h as f64 * 0.56;
        let dim = 1.0 - 0.5 * moon_dim.min(1.0);
        let star = get(pal, "star");
        for s in &self.stars {
            let tw = 0.55 + 0.45 * (t_sec * 1.6 + s.p).sin();
            let a = s.b * tw * night_amt * dim;
            if a <= 0.04 {
                continue;
            }
            fb.blend(
                js_round(s.x * fb.w as f64) as i32,
                js_round(s.y * span) as i32,
                star,
                a,
            );
        }
    }

    fn draw_aurora(&self, fb: &mut Framebuffer, night_amt: f64, t_sec: f64) {
        let w = fb.w;
        let bands: [(f64, f64, Rgb, f64); 3] = [
            (14.0, 10.0, [64, 255, 170], 0.10),
            (26.0, 13.0, [120, 220, 255], 0.07),
            (40.0, 9.0, [150, 110, 255], 0.08),
        ];
        for (y_base, amp, col, alpha) in bands {
            for x in 0..w {
                let n = (x as f64 * 0.03 + t_sec * 0.4 + y_base).sin() * amp;
                let top = y_base + n;
                let h = 26.0 + (x as f64 * 0.02 + t_sec * 0.3).sin() * 12.0;
                fb.fill_rect(
                    x as i32,
                    js_round(top) as i32,
                    1,
                    js_round(h) as i32,
                    col,
                    alpha * night_amt,
                );
            }
        }
    }

    pub fn draw_clouds(&self, fb: &mut Framebuffer, pal: &Palette, camera_x: f64, t_sec: f64) {
        let w = fb.w as f64;
        let period = w + 600.0;
        for c in &self.clouds {
            let mut sx = (c.x - camera_x * 0.04 - t_sec * 3.0 * c.s) % period;
            if sx < -300.0 {
                sx += period;
            }
            if sx > w + 150.0 {
                continue;
            }
            self.puff(fb, sx, c.y, c.w, get(pal, "cloud"), get(pal, "cloudHi"));
        }
    }

    fn puff(&self, fb: &mut Framebuffer, x: f64, y: f64, w: f64, col: Rgb, hi: Rgb) {
        let parts = [
            (0.0, 0.0, w * 0.5, 6.0),
            (-w * 0.22, 1.0, w * 0.42, 5.0),
            (w * 0.22, 2.0, w * 0.38, 4.0),
        ];
        for (dx, dy, ww, hh) in parts {
            fb.fill_ellipse(js_round(x + dx), js_round(y + dy), ww, hh, col, 1.0);
        }
        fb.fill_ellipse(
            js_round(x - w * 0.1),
            js_round(y - 2.0),
            w * 0.3,
            3.0,
            hi,
            1.0,
        );
    }
}

// (fin de la implementación)

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::get_palette;

    const SEED: u32 = 662118718;

    #[test]
    fn meteor_is_deterministic_bounded_and_occasional() {
        let mut found = 0;
        let mut t = 0.0;
        while t < 60.0 {
            let a = meteor_at(12345, t);
            assert_eq!(a.is_some(), meteor_at(12345, t).is_some());
            if let Some(m) = a {
                found += 1;
                assert!(m.x > 0.0 && m.x < 1.4);
                assert!(m.y > 0.0 && m.y < 0.6);
                assert!(m.alpha > 0.0 && m.alpha <= 1.0);
            }
            t += 0.1;
        }
        assert!(found > 0 && found < 600);
    }

    #[test]
    fn milky_way_is_deterministic_per_seed() {
        let pal = get_palette(1.0, "clear", 1.0);
        let mut a = Framebuffer::new(480, 270);
        let mut b = Framebuffer::new(480, 270);
        Sky::new(SEED).draw(&mut a, &pal, 1.0, 0.0, None, None, 0.0);
        Sky::new(SEED).draw(&mut b, &pal, 1.0, 0.0, None, None, 0.0);
        assert_eq!(a.px, b.px);
        let mut c = Framebuffer::new(480, 270);
        Sky::new(123).draw(&mut c, &pal, 1.0, 0.0, None, None, 0.0);
        assert_ne!(a.px, c.px);
    }

    #[test]
    fn moon_phase_lit_side() {
        let sky = Sky::new(SEED);
        let pal = get_palette(1.0, "clear", 1.0);
        let cel = Celestial {
            is_sun: false,
            u: 0.1,
            x: 240.0,
            y: 60.0,
            r: 5.0,
            depth: 0.1,
            horizon_y: 118.0,
            rising: true,
            visible: true,
            phase: Some(0.5),
            illum: Some(1.0),
        };
        let mut fb = Framebuffer::new(480, 270);
        sky.draw_body(&mut fb, &cel, &pal);
        let sun = get(&pal, "sun");
        let lit = fb
            .px
            .chunks(4)
            .filter(|p| p[0] == sun[0] && p[1] == sun[1] && p[2] == sun[2])
            .count();
        assert!(lit > 5, "luna llena iluminada ({lit})");
    }
}
