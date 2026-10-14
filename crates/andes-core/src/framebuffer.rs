//! Lienzo de píxeles RGBA8 en memoria. Sustituye al `CanvasRenderingContext2D`
//! para el rasterizado determinista del paisaje.
//!
//! Todo se dibuja en coordenadas enteras y sin antialias. El canal alfa se
//! compone en espacio directo (`src·a + dst·(1-a)`), sin premultiplicar.

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
        self.px.fill(0);
    }

    #[inline]
    fn idx(&self, x: usize, y: usize) -> usize {
        (y * self.w + x) * 4
    }

    /// Opacidad `a` (0..1) a un alfa entero `0..=255` con redondeo a la mitad
    /// hacia arriba.
    #[inline]
    fn alpha8(a: f64) -> u32 {
        (a.clamp(0.0, 1.0) * 255.0 + 0.5) as u32
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

    /// Compone un píxel ya recortado con alfa entero `0..=255` (source-over).
    #[inline]
    fn blend_at(&mut self, i: usize, c: Rgb, a8: u32) {
        if a8 >= 255 {
            self.px[i] = c[0];
            self.px[i + 1] = c[1];
            self.px[i + 2] = c[2];
        } else {
            let inv = 255 - a8;
            for (ch, &src) in c.iter().enumerate() {
                let dst = self.px[i + ch] as u32 * inv;
                self.px[i + ch] = ((src as u32 * a8 + dst + 127) / 255) as u8;
            }
        }
        self.px[i + 3] = 255;
    }

    /// Compone `c` con opacidad `a` sobre el píxel existente (source-over).
    #[inline]
    pub fn blend(&mut self, x: i32, y: i32, c: Rgb, a: f64) {
        if a <= 0.0 || x < 0 || y < 0 || x >= self.w as i32 || y >= self.h as i32 {
            return;
        }
        let a8 = Self::alpha8(a);
        let i = self.idx(x as usize, y as usize);
        self.blend_at(i, c, a8);
    }

    /// Rectángulo de color `c` con opacidad `a`, recortado al lienzo.
    ///
    /// Es el camino caliente del rasterizado (columnas de 1 px de ancho por la
    /// altura del terreno). Se compone con enteros sobre el `slice` de cada fila.
    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: Rgb, a: f64) {
        if w <= 0 || h <= 0 || a <= 0.0 {
            return;
        }
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + w).min(self.w as i32);
        let y1 = (y + h).min(self.h as i32);
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        let a8 = Self::alpha8(a);
        if a8 == 0 {
            return;
        }
        let cols = (x1 - x0) as usize;
        let stride = self.w * 4;
        let inv = 255 - a8;
        let (sr, sg, sb) = (c[0] as u32 * a8, c[1] as u32 * a8, c[2] as u32 * a8);

        // Columnas de 1 px: el patrón caliente del terreno. Se recorre con paso
        // de fila sin trocear el slice en cada fila.
        if cols == 1 {
            let mut i = self.idx(x0 as usize, y0 as usize);
            let end = self.idx(x0 as usize, y1 as usize);
            while i < end {
                if a8 >= 255 {
                    self.px[i] = c[0];
                    self.px[i + 1] = c[1];
                    self.px[i + 2] = c[2];
                } else {
                    self.px[i] = ((sr + self.px[i] as u32 * inv + 127) / 255) as u8;
                    self.px[i + 1] = ((sg + self.px[i + 1] as u32 * inv + 127) / 255) as u8;
                    self.px[i + 2] = ((sb + self.px[i + 2] as u32 * inv + 127) / 255) as u8;
                }
                self.px[i + 3] = 255;
                i += stride;
            }
            return;
        }

        let span = cols * 4;
        for yy in y0..y1 {
            let base = self.idx(x0 as usize, yy as usize);
            let row = &mut self.px[base..base + span];
            if a8 >= 255 {
                for px in row.as_chunks_mut::<4>().0 {
                    px[0] = c[0];
                    px[1] = c[1];
                    px[2] = c[2];
                    px[3] = 255;
                }
            } else {
                for px in row.as_chunks_mut::<4>().0 {
                    px[0] = ((sr + px[0] as u32 * inv + 127) / 255) as u8;
                    px[1] = ((sg + px[1] as u32 * inv + 127) / 255) as u8;
                    px[2] = ((sb + px[2] as u32 * inv + 127) / 255) as u8;
                    px[3] = 255;
                }
            }
        }
    }

    /// Elipse rellena rasterizada a píxeles (sin antialias), al estilo pixel-art:
    /// para cada fila entera se calcula la semianchura y se pinta con `fill_rect`.
    pub fn fill_ellipse(&mut self, cx: f64, cy: f64, rx: f64, ry: f64, c: Rgb, a: f64) {
        if a <= 0.0 || rx <= 0.0 || ry <= 0.0 {
            return;
        }
        let y0 = (cy - ry).ceil() as i32;
        let y1 = (cy + ry).floor() as i32;
        for y in y0..=y1 {
            let dy = (y as f64 + 0.5 - cy) / ry;
            let t = 1.0 - dy * dy;
            if t <= 0.0 {
                continue;
            }
            let span = rx * t.sqrt();
            let x0 = (cx - span).round() as i32;
            let x1 = (cx + span).round() as i32;
            if x1 >= x0 {
                self.fill_rect(x0, y, x1 - x0 + 1, 1, c, a);
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
    fn fill_ellipse_is_crisp() {
        let mut fb = Framebuffer::new(20, 20);
        fb.fill_ellipse(10.0, 10.0, 5.0, 5.0, [255, 255, 255], 1.0);
        assert_eq!(
            &fb.px[(10 * 20 + 10) * 4..(10 * 20 + 10) * 4 + 3],
            &[255, 255, 255]
        );
        assert_eq!(&fb.px[0..3], &[0, 0, 0]);
        for px in fb.px.chunks(4) {
            assert!(px[0] == 0 || px[0] == 255, "cobertura parcial en pixel-art");
        }
    }

    #[test]
    fn blend_halfway() {
        let mut fb = Framebuffer::new(1, 1);
        fb.set(0, 0, [0, 0, 0]);
        fb.blend(0, 0, [255, 255, 255], 0.5);
        assert_eq!(fb.px[0], 128);
    }
}
