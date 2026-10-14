//! Hojas de diagnóstico ("galería"): componen en un `Framebuffer` cada tipo de
//! flora y cada especie de fauna **aislados**, con fondo a cuadros y rótulo, para
//! inspeccionar silueta y color sin depender del paisaje completo.
//!
//! Son puras (sin I/O): el frontend las escala y exporta (ver el binario
//! `gallery` de `andes-native`). La rejilla se genera en varias tallas para
//! juzgar cómo se lee cada especie en el rango real de la capa.

use crate::fauna;
use crate::flora;
use crate::font;
use crate::framebuffer::Framebuffer;
use crate::palette::{get, shade, Palette, Rgb};

const CHECK: i32 = 8;
const FLORA_SIZES: [f64; 3] = [6.0, 10.0, 15.0];

/// Rellena con un tablero de cuadros `c0`/`c1` de `CHECK` px.
fn checker(fb: &mut Framebuffer, x0: i32, y0: i32, w: i32, h: i32, c0: Rgb, c1: Rgb) {
    for yy in (0..h).step_by(CHECK as usize) {
        for xx in (0..w).step_by(CHECK as usize) {
            let c = if (xx / CHECK + yy / CHECK) % 2 == 0 {
                c0
            } else {
                c1
            };
            fb.fill_rect(
                x0 + xx,
                y0 + yy,
                CHECK.min(w - xx),
                CHECK.min(h - yy),
                c,
                1.0,
            );
        }
    }
}

fn grid_fb(count: usize, cols: usize, cw: i32, ch: i32) -> Framebuffer {
    let cols = cols.max(1);
    let rows = count.div_ceil(cols);
    Framebuffer::new(
        (cols as i32 * cw) as usize,
        (rows as i32 * ch).max(1) as usize,
    )
}

/// Hoja de flora: una celda por entrada de `FLORA`, con tres tallas por porte.
pub fn flora_sheet(pal: &Palette, season: i32, cols: usize) -> Framebuffer {
    let cols = cols.max(1);
    let (cw, ch) = (140, 56);
    let mut fb = grid_fb(flora::FLORA.len(), cols, cw, ch);
    let base = get(pal, "skyMid");
    let (c0, c1) = (shade(base, -0.05), shade(base, -0.22));
    let label = shade(base, -0.65);
    let ground = shade(get(pal, "valleyD"), -0.1);
    let (fw, fh) = (fb.w as i32, fb.h as i32);
    checker(&mut fb, 0, 0, fw, fh, c0, c1);

    for (i, e) in flora::FLORA.iter().enumerate() {
        let cx = (i % cols) as i32 * cw;
        let cy = (i / cols) as i32 * ch;
        let gy = cy + ch - 14;
        fb.fill_rect(cx, gy + 1, cw, 1, ground, 0.6);
        for (j, sb) in FLORA_SIZES.iter().enumerate() {
            let sx = cx + 8 + j as i32 * 44;
            flora::draw_plant(
                &mut fb,
                sx as f64,
                gy as f64,
                sb * e.height,
                e.name,
                pal,
                0.0,
                false,
                season,
                0.0,
                -0.6,
            );
        }
        font::draw_text(&mut fb, e.name, cx + 3, cy + ch - 7, label, 1);
    }
    fb
}

/// Hoja de fauna: una celda por especie; arriba los frames, abajo volteados.
pub fn fauna_sheet(pal: &Palette, cols: usize) -> Framebuffer {
    let cols = cols.max(1);
    let (cw, ch) = (96, 52);
    let mut fb = grid_fb(fauna::SPECIES.len(), cols, cw, ch);
    let base = get(pal, "skyMid");
    let (c0, c1) = (shade(base, -0.05), shade(base, -0.22));
    let label = shade(base, -0.65);
    let (fw, fh) = (fb.w as i32, fb.h as i32);
    checker(&mut fb, 0, 0, fw, fh, c0, c1);

    for (i, sp) in fauna::SPECIES.iter().enumerate() {
        let cx = (i % cols) as i32 * cw;
        let cy = (i / cols) as i32 * ch;
        let (w, h) = fauna::sprite_size(sp.name).unwrap_or((1, 1));
        let nf = sp.frames.len().clamp(1, 4);
        for f in 0..nf {
            let left = cx + 6 + f as i32 * (w + 3);
            fauna::draw_species_preview(&mut fb, sp.name, left, cy + 6, f, false, pal);
            fauna::draw_species_preview(&mut fb, sp.name, left, cy + 9 + h, f, true, pal);
        }
        font::draw_text(&mut fb, sp.name, cx + 3, cy + ch - 7, label, 1);
    }
    fb
}

/// Escala por vecino más cercano (sin interpolar), para inspección de píxeles.
pub fn scale_nearest(fb: &Framebuffer, factor: i32) -> Framebuffer {
    let f = factor.max(1) as usize;
    let mut out = Framebuffer::new(fb.w * f, fb.h * f);
    for y in 0..fb.h {
        for x in 0..fb.w {
            let i = (y * fb.w + x) * 4;
            let c = [fb.px[i], fb.px[i + 1], fb.px[i + 2]];
            for dy in 0..f {
                for dx in 0..f {
                    out.set((x * f + dx) as i32, (y * f + dy) as i32, c);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::get_palette;

    #[test]
    fn flora_sheet_draws_and_fits() {
        let pal = get_palette(12.0, "clear", 1.0);
        for season in [0, 2] {
            let fb = flora_sheet(&pal, season, 4);
            assert!(fb.px.iter().any(|p| *p != 0));
        }
    }

    #[test]
    fn fauna_sheet_draws() {
        let pal = get_palette(12.0, "clear", 1.0);
        let fb = fauna_sheet(&pal, 3);
        assert!(fb.px.iter().any(|p| *p != 0));
    }

    #[test]
    fn scale_nearest_multiplies_pixels() {
        let mut fb = Framebuffer::new(2, 2);
        fb.set(0, 0, [10, 20, 30]);
        let up = scale_nearest(&fb, 3);
        assert_eq!((up.w, up.h), (6, 6));
        for (x, y) in [(0, 0), (2, 0), (0, 2), (2, 2)] {
            let i = (y * up.w + x) * 4;
            assert_eq!(&up.px[i..i + 3], &[10, 20, 30], "píxel ({x},{y})");
        }
    }
}
