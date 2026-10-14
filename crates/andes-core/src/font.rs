//! Fuente pixel 3×5 para rótulos (momentos y galerías de diagnóstico).
//!
//! Cada glifo es una matriz de 3 columnas por 5 filas de `#`/`.`; se dibuja con
//! `fill_rect` en coordenadas enteras (sin antialias). El avance es de 4 px por
//! carácter (3 + 1 de separación), igual en todos los escalados.

use crate::framebuffer::Framebuffer;
use crate::palette::Rgb;

fn glyph(ch: char) -> Option<[&'static str; 5]> {
    Some(match ch {
        '0' => ["###", "#.#", "#.#", "#.#", "###"],
        '1' => [".#.", "##.", ".#.", ".#.", "###"],
        '2' => ["###", "..#", "###", "#..", "###"],
        '3' => ["###", "..#", "###", "..#", "###"],
        '4' => ["#.#", "#.#", "###", "..#", "..#"],
        '5' => ["###", "#..", "###", "..#", "###"],
        '6' => ["###", "#..", "###", "#.#", "###"],
        '7' => ["###", "..#", "..#", "..#", "..#"],
        '8' => ["###", "#.#", "###", "#.#", "###"],
        '9' => ["###", "#.#", "###", "..#", "###"],
        'A' => ["###", "#.#", "###", "#.#", "#.#"],
        'B' => ["##.", "#.#", "##.", "#.#", "##."],
        'C' => ["###", "#..", "#..", "#..", "###"],
        'D' => ["##.", "#.#", "#.#", "#.#", "##."],
        'E' => ["###", "#..", "##.", "#..", "###"],
        'F' => ["###", "#..", "##.", "#..", "#.."],
        'G' => ["###", "#..", "#.#", "#.#", "###"],
        'H' => ["#.#", "#.#", "###", "#.#", "#.#"],
        'I' => ["###", ".#.", ".#.", ".#.", "###"],
        'J' => ["..#", "..#", "..#", "#.#", "###"],
        'K' => ["#.#", "#.#", "##.", "#.#", "#.#"],
        'L' => ["#..", "#..", "#..", "#..", "###"],
        'M' => ["#.#", "###", "###", "#.#", "#.#"],
        'N' => ["#.#", "###", "###", "###", "#.#"],
        'O' => ["###", "#.#", "#.#", "#.#", "###"],
        'P' => ["###", "#.#", "###", "#..", "#.."],
        'Q' => ["###", "#.#", "#.#", "###", "..#"],
        'R' => ["##.", "#.#", "##.", "#.#", "#.#"],
        'S' => ["###", "#..", "###", "..#", "###"],
        'T' => ["###", ".#.", ".#.", ".#.", ".#."],
        'U' => ["#.#", "#.#", "#.#", "#.#", "###"],
        'V' => ["#.#", "#.#", "#.#", "#.#", ".#."],
        'W' => ["#.#", "#.#", "###", "###", "#.#"],
        'X' => ["#.#", "#.#", ".#.", "#.#", "#.#"],
        'Y' => ["#.#", "#.#", ".#.", ".#.", ".#."],
        'Z' => ["###", "..#", ".#.", "#..", "###"],
        ' ' | '.' => ["...", "...", "...", "...", "..."],
        '-' => ["...", "...", "###", "...", "..."],
        '/' => ["..#", "..#", ".#.", "#..", "#.."],
        _ => return None,
    })
}

/// Ancho en píxeles que ocupará `text` al dibujarse con `scale`.
pub fn text_width(text: &str, scale: i32) -> i32 {
    text.chars().count() as i32 * 4 * scale - scale
}

/// Dibuja `text` (mayúsculas/minúsculas indistintas) en `left`/`top`.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_ascii_labels() {
        for ch in "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 -/.".chars() {
            assert!(glyph(ch).is_some(), "falta el glifo {ch}");
        }
    }

    #[test]
    fn width_matches_advance() {
        assert_eq!(text_width("A", 1), 3);
        assert_eq!(text_width("AB", 1), 7);
        assert_eq!(text_width("AB", 2), 14);
    }
}
