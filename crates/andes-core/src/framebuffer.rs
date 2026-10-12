//! Lienzo de píxeles RGBA8 en memoria. Sustituye al `CanvasRenderingContext2D`
//! para el rasterizado determinista del paisaje.
//!
//! Todo se dibuja en coordenadas enteras y sin antialias. El canal alfa se
//! compone en espacio directo (`src·a + dst·(1-a)`), sin premultiplicar.

use crate::math::js_round;
use crate::palette::Rgb;

pub struct Framebuffer {
    pub w: usize,
    pub h: usize,
    pub px: Vec<u8>,
}

impl Framebuffer {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            px: vec![0; w * h * 4],
        }
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        self.w = w;
        self.h = h;
        self.px = vec![0; w * h * 4];
    }

    pub fn clear(&mut self) {
        for b in self.px.iter_mut() {
            *b = 0;
        }
    }

    #[inline]
    fn idx(&self, x: usize, y: usize) -> usize {
        (y * self.w + x) * 4
    }

    #[inline]
    pub fn set(&mut self, x: i32, y: i32, c: Rgb) {
        if x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 {
            return;
        }
        let i = self.idx(x as usize, y as usize);
        self.px[i] = c[0];
        self.px[i + 1] = c[1];
        self.px[i + 2] = c[2];
        self.px[i + 3] = 255;
    }

    /// Compone `c` con opacidad `a` sobre el píxel existente (source-over).
    #[inline]
    pub fn blend(&mut self, x: i32, y: i32, c: Rgb, a: f64) {
        if a <= 0.0 || x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 {
            return;
        }
        if a >= 1.0 {
            self.set(x, y, c);
            return;
        }
        let i = self.idx(x as usize, y as usize);
        for (ch, dst) in self.px[i..i + 3].iter_mut().enumerate() {
            let d = *dst as f64;
            *dst = js_round(c[ch] as f64 * a + d * (1.0 - a)).clamp(0.0, 255.0) as u8;
        }
        self.px[i + 3] = 255;
    }

    /// Rectángulo de color `c` con opacidad `a`, recortado al lienzo.
    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Rgb, a: f64) {
        if w <= 0 || h <= 0 || a <= 0.0 {
            return;
        }
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + w).min(self.w as i32);
        let y1 = (y + h).min(self.h as i32);
        for yy in y0..y1 {
            for xx in x0..x1 {
                self.blend(xx, yy, c, a);
            }
        }
    }

    /// Rectángulo con bordes fraccionarios: aproxima la cobertura de `fillRect`
    /// de Canvas (la opacidad efectiva es `a` por el área cubierta del píxel).
    pub fn fill_rect_f(&mut self, x: f64, y: f64, w: f64, h: f64, c: Rgb, a: f64) {
        if w <= 0.0 || h <= 0.0 || a <= 0.0 {
            return;
        }
        let x0 = x.max(0.0);
        let y0 = y.max(0.0);
        let x1 = (x + w).min(self.w as f64);
        let y1 = (y + h).min(self.h as f64);
        let px0 = x0.floor() as i32;
        let py0 = y0.floor() as i32;
        let px1 = x1.ceil() as i32;
        let py1 = y1.ceil() as i32;
        for py in py0..py1 {
            let cov_y = ((py + 1) as f64).min(y1) - (py as f64).max(y0);
            if cov_y <= 0.0 {
                continue;
            }
            for px in px0..px1 {
                let cov_x = ((px + 1) as f64).min(x1) - (px as f64).max(x0);
                if cov_x <= 0.0 {
                    continue;
                }
                self.blend(px, py, c, a * cov_x * cov_y);
            }
        }
    }

    /// Elipse rellena con cobertura por supermuestreo (aproxima el antialias de
    /// `ctx.ellipse(...).fill()` de Canvas).
    pub fn fill_ellipse(&mut self, cx: f64, cy: f64, rx: f64, ry: f64, c: Rgb, a: f64) {
        if a <= 0.0 || rx <= 0.0 || ry <= 0.0 {
            return;
        }
        const N: i32 = 8;
        let x0 = (cx - rx).floor().max(0.0) as i32;
        let y0 = (cy - ry).floor().max(0.0) as i32;
        let x1 = ((cx + rx).ceil() as i32).min(self.w as i32);
        let y1 = ((cy + ry).ceil() as i32).min(self.h as i32);
        let inv = 1.0 / (N * N) as f64;
        for py in y0..y1 {
            for px in x0..x1 {
                let mut inside = 0;
                for sy in 0..N {
                    for sx in 0..N {
                        let fx = px as f64 + (sx as f64 + 0.5) / N as f64;
                        let fy = py as f64 + (sy as f64 + 0.5) / N as f64;
                        let dx = (fx - cx) / rx;
                        let dy = (fy - cy) / ry;
                        if dx * dx + dy * dy <= 1.0 {
                            inside += 1;
                        }
                    }
                }
                if inside > 0 {
                    self.blend(px, py, c, a * inside as f64 * inv);
                }
            }
        }
    }

    /// Círculo relleno rasterizado a píxeles (sin antialias), igual que `disc`
    /// de `pixel.js`.
    pub fn fill_disc(&mut self, cx: f64, cy: f64, r: f64, c: Rgb, a: f64) {
        if a <= 0.0 || r < 0.0 {
            return;
        }
        let r2 = r * r;
        let top = -(r.ceil() as i32);
        let bot = r.ceil() as i32;
        for dy in top..=bot {
            let span = ((r2 - (dy as f64) * (dy as f64)).max(0.0)).sqrt().floor();
            self.fill_rect(
                (cx - span).round() as i32,
                (cy + dy as f64).round() as i32,
                span as i32 * 2 + 1,
                1,
                c,
                a,
            );
        }
    }

    /// Degradado vertical de 1 px de ancho (columna) entre dos colores.
    pub fn vgradient(&mut self, x: i32, y: i32, h: i32, top: Rgb, bottom: Rgb, a: f64) {
        if h <= 0 || a <= 0.0 {
            return;
        }
        for i in 0..h {
            let t = if h <= 1 {
                0.0
            } else {
                i as f64 / (h - 1) as f64
            };
            let c = crate::palette::lerp_color(top, bottom, t);
            self.blend(x, y + i, c, a);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_clip() {
        let mut fb = Framebuffer::new(4, 3);
        fb.set(0, 0, [10, 20, 30]);
        fb.set(-1, 0, [1, 2, 3]);
        fb.set(4, 0, [1, 2, 3]);
        assert_eq!(&fb.px[0..3], &[10, 20, 30]);
        assert_eq!(fb.px[3], 255);
    }

    #[test]
    fn fill_rect_clips_bounds() {
        let mut fb = Framebuffer::new(4, 4);
        fb.fill_rect(-2, -2, 3, 3, [255, 0, 0], 1.0);
        assert_eq!(&fb.px[0..3], &[255, 0, 0]);
        assert_eq!(&fb.px[(4 + 1) * 4..(4 + 1) * 4 + 3], &[0, 0, 0]);
    }

    #[test]
    fn blend_halfway() {
        let mut fb = Framebuffer::new(1, 1);
        fb.set(0, 0, [0, 0, 0]);
        fb.blend(0, 0, [255, 255, 255], 0.5);
        assert_eq!(js_round(255.0 * 0.5) as u8, fb.px[0]);
        assert_eq!(fb.px[0], 128);
    }
}
