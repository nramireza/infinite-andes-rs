//! Exporta las hojas de diagnóstico de flora y fauna a PNG (escaladas nearest),
//! para inspeccionar siluetas y color fuera del paisaje.
//!
//! Uso: `cargo run -p andes-native --bin gallery -- [hour] [weather] [season] [scale]`
//! Genera `gallery-flora.png` y `gallery-fauna.png` en el directorio actual.

use andes_core::framebuffer::Framebuffer;
use andes_core::gallery;
use andes_core::palette::get_palette;

fn fb_to_image(fb: &Framebuffer) -> image::RgbaImage {
    let mut img = image::RgbaImage::new(fb.w as u32, fb.h as u32);
    for y in 0..fb.h {
        for x in 0..fb.w {
            let i = (y * fb.w + x) * 4;
            img.put_pixel(
                x as u32,
                y as u32,
                image::Rgba([fb.px[i], fb.px[i + 1], fb.px[i + 2], 255]),
            );
        }
    }
    img
}

fn save(fb: &Framebuffer, factor: i32, name: &str) {
    let up = gallery::scale_nearest(fb, factor);
    match fb_to_image(&up).save(name) {
        Ok(()) => println!("  {name}  ({}×{})", up.w, up.h),
        Err(e) => eprintln!("  error al guardar {name}: {e}"),
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let hour: f64 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(12.0);
    let weather = a.get(2).cloned().unwrap_or_else(|| "clear".into());
    let season: i32 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);
    let factor: i32 = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(4);

    let pal = get_palette(hour, &weather, 1.0);
    println!("Galería (hora {hour}, {weather}, estación {season}, ×{factor})");
    save(
        &gallery::flora_sheet(&pal, season, 4),
        factor,
        "gallery-flora.png",
    );
    save(&gallery::fauna_sheet(&pal, 3), factor, "gallery-fauna.png");
}
