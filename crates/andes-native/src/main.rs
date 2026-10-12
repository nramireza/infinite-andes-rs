//! Frontend nativo: ventana con panel de control (egui), reloj real, modo
//! fondo de pantalla y export PNG.
//!
//! - `infinite-andes`            → ventana con panel
//! - `infinite-andes wall`       → pantalla completa sin paneles (fondo de pantalla)
//! - `infinite-andes <seed> ...` → sin ventana, renderiza a PPM (capturas/CI)

use std::time::Instant;

use andes_core::clock::DateTime;
use andes_core::framebuffer::Framebuffer;
use andes_core::rng::seed_to_int;
use andes_core::scene::Scene;
use andes_core::viewport::{width_for_ratio, ASPECT_PRESETS};

const ASPECTS: [(&str, f64); 3] = [
    ("16:9", 16.0 / 9.0),
    ("21:9", 21.0 / 9.0),
    ("32:9", 32.0 / 9.0),
];
const WEATHERS: [&str; 7] = ["auto", "clear", "snow", "rain", "fog", "wind", "storm"];
const BIOMES: [&str; 7] = [
    "auto",
    "altiplano",
    "norte",
    "centro",
    "sur",
    "patagonia",
    "austral",
];
const MOMENTS: [&str; 8] = [
    "auto", "none", "18sep", "leorey", "kungleo", "condor", "bandada", "manada",
];
const SEASONS: [&str; 5] = ["auto", "verano", "otono", "invierno", "primavera"];
const BLOOMS: [&str; 3] = ["auto", "on", "off"];
const HEIGHT: usize = 270;

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

fn forced(label: &str) -> &'static str {
    match label {
        "18sep" => "18sep",
        "leorey" => "leorey",
        "kungleo" => "kungleo",
        "condor" => "condor",
        "bandada" => "bandada",
        "manada" => "manada",
        "none" => "none",
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

fn export_png(fb: &Framebuffer, seed: &str, hour: f64) {
    let mut img = image::RgbaImage::new(fb.w as u32, fb.h as u32);
    for (i, px) in fb.px.chunks(4).enumerate() {
        img.put_pixel(
            (i as u32) % fb.w as u32,
            (i as u32) / fb.w as u32,
            image::Rgba([px[0], px[1], px[2], 255]),
        );
    }
    let hh = hour.floor() as i32;
    let mm = (hour.fract() * 60.0).floor() as i32;
    let name = format!("infinite-andes_{seed}_{hh:02}{mm:02}.png");
    match img.save(&name) {
        Ok(()) => println!("exportado {name}"),
        Err(e) => eprintln!("no se pudo exportar: {e}"),
    }
}

fn local_now() -> (DateTime, f64) {
    use chrono::{Datelike, Local, Timelike};
    let now = Local::now();
    let dt = DateTime::new(
        now.year() as i64,
        now.month0(),
        now.day(),
        now.hour(),
        now.minute(),
        now.second(),
        now.timestamp_subsec_millis(),
    );
    (dt, now.offset().local_minus_utc() as f64 / 3600.0)
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let wall = args.iter().any(|a| a == "wall" || a == "--wall");
    if !wall && args.len() > 1 {
        return headless(&args);
    }
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 420.0])
            .with_title("Infinite Andes")
            .with_fullscreen(wall)
            .with_decorations(!wall),
        ..Default::default()
    };
    eframe::run_native(
        "Infinite Andes",
        options,
        Box::new(move |_cc| Ok(Box::new(App::new(wall)))),
    )
}

fn headless(args: &[String]) -> eframe::Result<()> {
    let seed = args.get(1).cloned().unwrap_or_else(|| "andes".into());
    let hour: f64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(8.0);
    let weather = args.get(3).map(|s| pmap(s)).unwrap_or("clear");
    let moment = args.get(4).map(|s| forced(s)).unwrap_or("auto");
    let biome = args.get(5).map(|s| s.as_str()).unwrap_or("auto");
    let aspect = args
        .get(6)
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(ASPECT_PRESETS[2].ratio);
    let season = args.get(7).map(|s| s.as_str()).unwrap_or("auto");
    let bloom = args.get(8).map(|s| s.as_str()).unwrap_or("auto");

    let w = width_for_ratio(aspect) as usize;
    let mut scene = Scene::new(seed_to_int(&seed), w, HEIGHT);
    scene.hour = hour;
    scene.time_auto = false;
    scene.auto_scroll = false;
    scene.camera_x = 0.0;
    scene.set_weather_type(weather);
    scene.set_moment(moment);
    scene.set_biome(biome);
    scene.set_season(season);
    scene.set_bloom(bloom);

    let mut fb = Framebuffer::new(w, HEIGHT);
    scene.render(&mut fb);
    let path = "/tmp/infinite-andes.ppm";
    match write_ppm(path, &fb) {
        Ok(()) => println!("infinite-andes · seed={seed} · {w}x{HEIGHT} · hour={hour} · weather={weather} · {path}"),
        Err(e) => eprintln!("no se pudo escribir {path}: {e}"),
    }
    std::process::exit(0);
}

struct App {
    scene: Scene,
    fb: Framebuffer,
    tex: Option<eframe::egui::TextureHandle>,
    seed_text: String,
    aspect_idx: usize,
    hour: f64,
    clock_fast: bool,
    auto_scroll: bool,
    weather: usize,
    biome: usize,
    moment: usize,
    season: usize,
    bloom: usize,
    fit_cover: bool,
    wall: bool,
    last: Instant,
    fps_acc: f64,
    fps_frames: u32,
    fps: f64,
}

impl App {
    fn new(wall: bool) -> Self {
        let (w, h) = (width_for_ratio(ASPECTS[2].1) as usize, HEIGHT);
        let seed = "andes";
        let mut scene = Scene::new(seed_to_int(seed), w, h);
        scene.camera_x = 0.0;
        scene.set_clock_fast(false);
        Self {
            scene,
            fb: Framebuffer::new(w, h),
            tex: None,
            seed_text: seed.to_string(),
            aspect_idx: 2,
            hour: 8.0,
            clock_fast: false,
            auto_scroll: true,
            weather: 0,
            biome: 0,
            moment: 0,
            season: 0,
            bloom: 0,
            fit_cover: wall,
            wall,
            last: Instant::now(),
            fps_acc: 0.0,
            fps_frames: 0,
            fps: 0.0,
        }
    }

    fn set_aspect(&mut self, idx: usize) {
        self.aspect_idx = idx;
        let w = width_for_ratio(ASPECTS[idx].1) as usize;
        self.scene.resize(w, HEIGHT);
        self.fb.resize(w, HEIGHT);
        self.tex = None;
    }

    fn reset_seed(&mut self, seed: &str) {
        let (w, h) = (self.fb.w, self.fb.h);
        let mut scene = Scene::new(seed_to_int(seed), w, h);
        scene.camera_x = 0.0;
        scene.hour = self.hour;
        scene.auto_scroll = self.auto_scroll;
        scene.set_clock_fast(self.clock_fast);
        scene.set_weather_type(pmap(WEATHERS[self.weather]));
        scene.set_biome(BIOMES[self.biome]);
        scene.set_moment(forced(MOMENTS[self.moment]));
        scene.set_season(SEASONS[self.season]);
        scene.set_bloom(BLOOMS[self.bloom]);
        self.scene = scene;
    }

    fn ui_controls(&mut self, ui: &mut eframe::egui::Ui) {
        ui.heading("Infinite Andes");
        ui.label(format!("{:.0} fps", self.fps));
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Semilla");
            ui.add(eframe::egui::TextEdit::singleline(&mut self.seed_text).desired_width(120.0));
        });
        ui.horizontal(|ui| {
            if ui.button("Aplicar").clicked() {
                let s = self.seed_text.clone();
                self.reset_seed(&s);
            }
            if ui.button("Aleatoria").clicked() {
                let n = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.subsec_nanos())
                    .unwrap_or(1);
                let s = format!("semilla-{n}");
                self.seed_text = s.clone();
                self.reset_seed(&s);
            }
        });

        ui.separator();
        ui.label("Relación de aspecto");
        let cur = ASPECTS[self.aspect_idx].0;
        eframe::egui::ComboBox::from_id_source("aspect")
            .selected_text(cur)
            .show_ui(ui, |ui| {
                for (i, (label, _)) in ASPECTS.iter().enumerate() {
                    if ui.selectable_label(i == self.aspect_idx, *label).clicked() {
                        self.set_aspect(i);
                    }
                }
            });

        ui.separator();
        ui.checkbox(&mut self.clock_fast, "Reloj rápido determinista");
        self.scene.set_clock_fast(self.clock_fast);
        if self.clock_fast {
            ui.add(
                eframe::egui::Slider::new(&mut self.hour, 0.0..=24.0)
                    .fixed_decimals(2)
                    .text("hora"),
            );
            self.scene.hour = self.hour;
            self.scene.time_auto = false;
        } else {
            ui.label(format!(
                "Hora local: {:02}:{:02}",
                self.hour.floor(),
                (self.hour.fract() * 60.0).floor()
            ));
        }
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.auto_scroll, "Auto-scroll");
            ui.checkbox(&mut self.fit_cover, "Cubrir (recortar)");
        });
        self.scene.auto_scroll = self.auto_scroll;

        ui.separator();
        combo(
            ui,
            "weather",
            "Clima",
            &WEATHERS,
            &mut self.weather,
            |a, v| a.set_weather_type(pmap(v)),
            &mut self.scene,
        );
        combo(
            ui,
            "biome",
            "Bioma",
            &BIOMES,
            &mut self.biome,
            |a, v| a.set_biome(v),
            &mut self.scene,
        );
        combo(
            ui,
            "moment",
            "Momento",
            &MOMENTS,
            &mut self.moment,
            |a, v| a.set_moment(forced(v)),
            &mut self.scene,
        );
        combo(
            ui,
            "season",
            "Estación",
            &SEASONS,
            &mut self.season,
            |a, v| a.set_season(v),
            &mut self.scene,
        );
        combo(
            ui,
            "bloom",
            "Floración",
            &BLOOMS,
            &mut self.bloom,
            |a, v| a.set_bloom(v),
            &mut self.scene,
        );

        ui.separator();
        if ui.button("Exportar PNG").clicked() {
            export_png(&self.fb, &self.seed_text, self.hour);
        }
        if ui
            .button(if self.wall {
                "Salir del modo fondo"
            } else {
                "Modo fondo (pantalla completa)"
            })
            .clicked()
        {
            self.wall = !self.wall;
            let ctx = ui.ctx().clone();
            ctx.send_viewport_cmd(eframe::egui::ViewportCommand::Fullscreen(self.wall));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn combo(
    ui: &mut eframe::egui::Ui,
    id: &str,
    label: &str,
    options: &[&str],
    selected: &mut usize,
    apply: impl Fn(&mut Scene, &str),
    scene: &mut Scene,
) {
    ui.horizontal(|ui| {
        ui.label(label);
        let cur = options[*selected];
        eframe::egui::ComboBox::from_id_source(id)
            .selected_text(cur)
            .show_ui(ui, |ui| {
                for (i, opt) in options.iter().enumerate() {
                    if ui.selectable_label(i == *selected, *opt).clicked() {
                        *selected = i;
                        apply(scene, opt);
                    }
                }
            });
    });
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        let dt = (now - self.last).as_secs_f64().min(0.1);
        self.last = now;
        self.fps_acc += dt;
        self.fps_frames += 1;
        if self.fps_acc >= 0.5 {
            self.fps = self.fps_frames as f64 / self.fps_acc;
            self.fps_acc = 0.0;
            self.fps_frames = 0;
        }

        let (dt_now, tz) = local_now();
        self.scene.set_now(dt_now, tz);
        self.scene.set_clock_fast(self.clock_fast);
        self.scene.update(dt);
        if !self.clock_fast {
            self.hour = self.scene.hour;
        }
        self.scene.render(&mut self.fb);

        let img =
            eframe::egui::ColorImage::from_rgba_unmultiplied([self.fb.w, self.fb.h], &self.fb.px);
        match &mut self.tex {
            Some(t) => t.set(img, eframe::egui::TextureOptions::NEAREST),
            None => {
                self.tex =
                    Some(ctx.load_texture("scene", img, eframe::egui::TextureOptions::NEAREST));
            }
        }

        if !self.wall {
            eframe::egui::SidePanel::right("controls")
                .resizable(false)
                .default_width(250.0)
                .show(ctx, |ui| {
                    self.ui_controls(ui);
                });
        }

        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.max_rect();
            if let Some(t) = &self.tex {
                let tex_size = eframe::egui::vec2(self.fb.w as f32, self.fb.h as f32);
                let sx = rect.width() / tex_size.x;
                let sy = rect.height() / tex_size.y;
                let scale = if self.fit_cover {
                    sx.max(sy)
                } else {
                    sx.min(sy)
                };
                let size = tex_size * scale;
                let img_rect = eframe::egui::Rect::from_center_size(rect.center(), size);
                let uv = eframe::egui::Rect::from_min_max(
                    eframe::egui::pos2(0.0, 0.0),
                    eframe::egui::pos2(1.0, 1.0),
                );
                ui.painter()
                    .image(t.id(), img_rect, uv, eframe::egui::Color32::WHITE);
            }
        });

        ctx.request_repaint();
    }
}
