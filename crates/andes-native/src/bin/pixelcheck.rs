//! Compara el rasterizador Rust con la captura de Canvas (JS) píxel a píxel.
//!
//! Uso:
//!   pixelcheck <js.png> <seed> <hour> <weather> <biome> <moment> <bloom> <season> <aspect> <x>
//!
//! Renderiza la misma escena con `andes-core`, compara contra el PNG capturado
//! por Chrome y reporta estadísticas de diferencia. Escribe `/tmp/ia-rust.ppm`
//! y `/tmp/ia-diff.png`.

use andes_core::framebuffer::Framebuffer;
use andes_core::rng::seed_to_int;
use andes_core::scene::Scene;
use andes_core::viewport::{ratio_from_string, width_for_ratio, ASPECT_PRESETS};

fn pmap(label: &str) -> &'static str {
    match label {
        "clear" => "clear",
        "snow" => "snow",
        "rain" => "rain",
        "fog" => "fog",
        "wind" => "wind",
        "storm" => "storm",
        _ => "auto",
    }
}

fn write_ppm(path: &str, fb: &Framebuffer) -> std::io::Result<()> {
    let mut out = Vec::with_capacity(15 + fb.w * fb.h * 3);
    out.extend_from_slice(format!("P6\n{} {}\n255\n", fb.w, fb.h).as_bytes());
    for px in fb.px.chunks(4) {
        out.extend_from_slice(&px[0..3]);
    }
    std::fs::write(path, out)
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let js_path = a.get(1).cloned().unwrap_or_else(|| "/tmp/ia-js.png".into());
    let seed = a.get(2).cloned().unwrap_or_else(|| "andes".into());
    let hour: f64 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(12.0);
    let weather = a.get(4).map(|s| pmap(s)).unwrap_or("clear");
    let biome = a.get(5).map(|s| s.as_str()).unwrap_or("auto");
    let moment = a.get(6).map(|s| s.as_str()).unwrap_or("none");
    let bloom = a.get(7).map(|s| s.as_str()).unwrap_or("auto");
    let season = a.get(8).map(|s| s.as_str()).unwrap_or("auto");
    let aspect = a
        .get(9)
        .and_then(|s| ratio_from_string(s))
        .unwrap_or(ASPECT_PRESETS[2].ratio);
    let x: f64 = a.get(10).and_then(|s| s.parse().ok()).unwrap_or(0.0);

    let w = width_for_ratio(aspect) as usize;
    let h = 270usize;
    let mut scene = Scene::new(seed_to_int(&seed), w, h);
    scene.auto_scroll = false;
    scene.time_auto = false;
    scene.weather_auto = false;
    scene.hour = hour;
    scene.t_sec = 0.0;
    scene.camera_x = x;
    scene.set_biome(biome);
    scene.set_moment(moment);
    scene.set_bloom(bloom);
    scene.set_season(season);
    scene.set_weather_type(weather);

    let mut fb = Framebuffer::new(w, h);
    scene.render(&mut fb);
    let _ = write_ppm("/tmp/ia-rust.ppm", &fb);

    let js = match image::open(&js_path) {
        Ok(img) => img.to_rgba8(),
        Err(e) => {
            eprintln!("no se pudo abrir {js_path}: {e}");
            std::process::exit(2);
        }
    };
    let (jw, jh) = (js.width() as usize, js.height() as usize);
    if jw != w || jh != h {
        eprintln!("dimensión distinta: JS {jw}x{jh} vs Rust {w}x{h}");
    }
    let cw = w.min(jw);
    let ch = h.min(jh);

    let mut diff = image::RgbaImage::new(cw as u32, ch as u32);
    let mut diff8 = image::RgbaImage::new(cw as u32, ch as u32);
    let mut differing = 0u64;
    let mut sum = 0u64;
    let mut max = 0u32;
    let mut hist = [0u64; 9]; // d = 0..7 exactos, 8 = >=8
    let horizon = (h as f64 * 0.56).round() as usize;
    let mut sum_sky = 0u64;
    let mut n_sky = 0u64;
    let mut sum_gnd = 0u64;
    let mut n_gnd = 0u64;
    let total = (cw * ch) as u64;
    for y in 0..ch {
        for xx in 0..cw {
            let i = (y * w + xx) * 4;
            let r = fb.px[i] as i32;
            let g = fb.px[i + 1] as i32;
            let b = fb.px[i + 2] as i32;
            let jp = js.get_pixel(xx as u32, y as u32);
            let (jr, jg, jb) = (jp[0] as i32, jp[1] as i32, jp[2] as i32);
            let d = (r - jr).abs().max((g - jg).abs()).max((b - jb).abs());
            let dsum = ((r - jr).abs() + (g - jg).abs() + (b - jb).abs()) as u64;
            sum += dsum;
            if y < horizon {
                sum_sky += dsum;
                n_sky += 3;
            } else {
                sum_gnd += dsum;
                n_gnd += 3;
            }
            max = max.max(d as u32);
            if d > 0 {
                differing += 1;
            }
            hist[(d as usize).min(8)] += 1;
            let c = if d == 0 {
                [0, 0, 0, 255]
            } else if d < 8 {
                [40, 40, 0, 255]
            } else if d < 32 {
                [120, 60, 0, 255]
            } else {
                [255, 0, 0, 255]
            };
            diff.put_pixel(xx as u32, y as u32, image::Rgba(c));
            let c8 = if d >= 8 {
                [255, 40, 40, 255]
            } else {
                [0, 0, 0, 255]
            };
            diff8.put_pixel(xx as u32, y as u32, image::Rgba(c8));
        }
    }
    let _ = diff.save("/tmp/ia-diff.png");
    let _ = diff8.save("/tmp/ia-diff8.png");

    let pct = 100.0 * differing as f64 / total as f64;
    let mean = sum as f64 / (total as f64 * 3.0);
    let mean_sky = sum_sky as f64 / n_sky.max(1) as f64;
    let mean_gnd = sum_gnd as f64 / n_gnd.max(1) as f64;
    println!("escena: seed={seed} hour={hour} weather={weather} biome={biome} moment={moment} bloom={bloom} season={season} aspect={aspect} x={x}");
    println!("  tamaño {w}x{h} · píxeles {total}");
    println!("  distintos: {differing} ({pct:.3}%)");
    println!(
        "  |Δ| medio/canal: {mean:.4} (cielo {mean_sky:.4} · suelo {mean_gnd:.4}) · máx: {max}"
    );
    println!("  histograma Δ (d=0..7, 8=≥8): {hist:?}");
    let big: u64 = hist[8..].iter().sum();
    let pct_big = 100.0 * big as f64 / total as f64;
    let ok = max <= 64 && mean < 1.0 && pct_big < 0.5;
    println!("  píxeles con Δ≥8: {big} ({pct_big:.3}%)");
    println!(
        "  veredicto: {}",
        if ok {
            "OK (paridad aproximada)"
        } else {
            "REVISAR (diferencias notables)"
        }
    );
    println!("  rust=/tmp/ia-rust.ppm diff=/tmp/ia-diff.png js={js_path}");
}
