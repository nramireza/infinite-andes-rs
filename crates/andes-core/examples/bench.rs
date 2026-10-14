use std::time::Instant;

use andes_core::framebuffer::Framebuffer;
use andes_core::rng::seed_to_int;
use andes_core::scene::Scene;

fn main() {
    if std::env::args().nth(1).as_deref() == Some("fill") {
        let n = 5760;
        for (label, alpha) in [("blend 0.85", 0.85), ("opaque 1.00", 1.0)] {
            let mut fb = Framebuffer::new(960, 270);
            let t0 = Instant::now();
            for k in 0..n {
                let x = k % 960;
                fb.fill_rect(x, 130, 1, 140, [40, 60, 80], alpha);
            }
            let el = t0.elapsed().as_secs_f64();
            println!(
                "col {label}: {:.3} ms ({:.4} ms/col)",
                el * 1000.0,
                el * 1000.0 / n as f64
            );
        }
        let mut fb = Framebuffer::new(960, 270);
        let t0 = Instant::now();
        for _ in 0..151 {
            fb.fill_rect(0, 0, 960, 1, [40, 60, 80], 0.85);
        }
        let el = t0.elapsed().as_secs_f64();
        println!("row 960 blended: {:.3} ms", el * 1000.0);
        return;
    }
    let w: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(960);
    let h: usize = 270;
    let weather = std::env::args().nth(2).unwrap_or_else(|| "clear".into());
    let weather_static: &'static str = Box::leak(weather.clone().into_boxed_str());
    let frames: usize = 300;

    let mut scene = Scene::new(seed_to_int("andes"), w, h);
    scene.set_weather_type(weather_static);
    let mut fb = Framebuffer::new(w, h);

    for _ in 0..40 {
        scene.update(1.0 / 60.0);
        scene.render(&mut fb);
    }

    let t0 = Instant::now();
    for _ in 0..frames {
        scene.update(1.0 / 60.0);
        scene.render(&mut fb);
    }
    let el = t0.elapsed().as_secs_f64();
    println!(
        "{w}x{h} {weather}: {:.3} ms/frame ({:.0} fps)",
        el * 1000.0 / frames as f64,
        frames as f64 / el
    );
}
