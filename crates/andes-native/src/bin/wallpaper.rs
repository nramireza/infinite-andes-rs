//! Fondo de pantalla nativo: ventana sin bordes a pantalla completa marcada como
//! "desktop" (EWMH `_NET_WM_WINDOW_TYPE_DESKTOP` en X11) o `wlr-layer-shell`
//! (capa background) en Wayland, para quedar tras los iconos del escritorio.
//!
//! Uso: `infinite-andes-wall [seed] [hour] [weather] [moment]`
//! Con `hour` presente usa el reloj rápido; sin él, el reloj real local.

#[path = "wallpaper/wayland.rs"]
mod wayland;

use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Instant;

use andes_core::clock::DateTime;
use andes_core::framebuffer::Framebuffer;
use andes_core::rng::seed_to_int;
use andes_core::scene::Scene;
use andes_core::viewport::{width_for_ratio, ASPECT_PRESETS};
use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Fullscreen, Window, WindowId};

const HEIGHT: usize = 270;

fn iso(seed: &str) -> u32 {
    seed_to_int(seed)
}

fn local_now() -> (DateTime, f64) {
    use chrono::{Datelike, Local, Timelike};
    let now = Local::now();
    (
        DateTime::new(
            now.year() as i64,
            now.month0(),
            now.day(),
            now.hour(),
            now.minute(),
            now.second(),
            now.timestamp_subsec_millis(),
        ),
        now.offset().local_minus_utc() as f64 / 3600.0,
    )
}

fn blit(fb: &Framebuffer, out: &mut [u32], w: usize, h: usize) {
    if fb.w == 0 || fb.h == 0 || w == 0 || h == 0 {
        return;
    }
    // LUT de columnas: la división se hace una vez por columna, no por píxel.
    let cols: Vec<usize> = (0..w).map(|x| (x * fb.w / w).min(fb.w - 1)).collect();
    for (y, row_out) in out.chunks_mut(w).enumerate().take(h) {
        let sy = (y * fb.h / h).min(fb.h - 1);
        let src = &fb.px[sy * fb.w * 4..];
        for (x, dst) in row_out.iter_mut().enumerate() {
            let i = cols[x] * 4;
            *dst = ((src[i] as u32) << 16) | ((src[i + 1] as u32) << 8) | (src[i + 2] as u32);
        }
    }
}

#[cfg(target_os = "linux")]
fn set_desktop_hint(window: &Window) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let win = match window.window_handle().map(|h| h.as_raw()) {
        Ok(RawWindowHandle::Xlib(h)) => h.window as u32,
        Ok(RawWindowHandle::Xcb(h)) => h.window.get(),
        _ => return,
    };
    let Ok((conn, _screen)) = x11rb::connect(None) else {
        return;
    };
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::{AtomEnum, ConnectionExt as _, PropMode};
    use x11rb::wrapper::ConnectionExt as _;
    let atom = conn
        .intern_atom(false, b"_NET_WM_WINDOW_TYPE")
        .ok()
        .and_then(|c| c.reply().ok())
        .map(|r| r.atom);
    let desktop = conn
        .intern_atom(false, b"_NET_WM_WINDOW_TYPE_DESKTOP")
        .ok()
        .and_then(|c| c.reply().ok())
        .map(|r| r.atom);
    if let (Some(atom), Some(desktop)) = (atom, desktop) {
        let _ = conn.change_property32(PropMode::REPLACE, win, atom, AtomEnum::ATOM, &[desktop]);
        let _ = conn.flush();
    }
}

#[cfg(not(target_os = "linux"))]
fn set_desktop_hint(_window: &Window) {}

struct App {
    hour: Option<f64>,
    window: Option<Arc<Window>>,
    context: Option<Context<Arc<Window>>>,
    surface: Option<Surface<Arc<Window>, Arc<Window>>>,
    scene: Scene,
    fb: Framebuffer,
    last: Instant,
    hint_frames: u32,
}

impl App {
    fn new(seed_str: &str, hour: Option<f64>, weather: &'static str, moment: &'static str) -> Self {
        let w = width_for_ratio(ASPECT_PRESETS[2].ratio) as usize;
        let seed = iso(seed_str);
        let mut scene = Scene::new(seed, w, HEIGHT);
        scene.camera_x = 0.0;
        scene.set_weather_type(weather);
        scene.set_moment(moment);
        match hour {
            Some(h) => {
                scene.set_clock_fast(true);
                scene.hour = h;
                scene.time_auto = false;
            }
            None => scene.set_clock_fast(false),
        }
        Self {
            hour,
            window: None,
            context: None,
            surface: None,
            scene,
            fb: Framebuffer::new(w, HEIGHT),
            last: Instant::now(),
            hint_frames: 0,
        }
    }

    fn redraw(&mut self, el: &ActiveEventLoop) {
        let (Some(window), Some(surface)) = (self.window.as_ref(), self.surface.as_mut()) else {
            return;
        };
        if self.hint_frames < 180 {
            set_desktop_hint(window);
            self.hint_frames += 1;
        }
        let size = window.inner_size();
        let (Some(w), Some(h)) = (
            NonZeroU32::new(size.width.max(1)),
            NonZeroU32::new(size.height.max(1)),
        ) else {
            return;
        };
        let _ = surface.resize(w, h);

        let now = Instant::now();
        let dt = (now - self.last).as_secs_f64().min(0.1);
        self.last = now;
        if self.hour.is_none() {
            let (dt_now, tz) = local_now();
            self.scene.set_now(dt_now, tz);
        }
        self.scene.update(dt);
        self.scene.render(&mut self.fb);

        if let Ok(mut buffer) = surface.buffer_mut() {
            blit(&self.fb, &mut buffer, w.get() as usize, h.get() as usize);
            let _ = buffer.present();
        }
        el.set_control_flow(ControlFlow::WaitUntil(
            now + std::time::Duration::from_millis(33),
        ));
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("Infinite Andes")
            .with_decorations(false)
            .with_fullscreen(Some(Fullscreen::Borderless(None)));
        let Ok(window) = el.create_window(attrs) else {
            return;
        };
        let window = Arc::new(window);
        set_desktop_hint(&window);
        let Ok(context) = Context::new(window.clone()) else {
            return;
        };
        let Ok(surface) = Surface::new(&context, window.clone()) else {
            return;
        };
        self.context = Some(context);
        self.surface = Some(surface);
        self.window = Some(window);
        self.last = Instant::now();
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                use winit::keyboard::{Key, NamedKey};
                if event.state.is_pressed()
                    && matches!(event.logical_key, Key::Named(NamedKey::Escape))
                {
                    el.exit();
                }
            }
            WindowEvent::RedrawRequested => self.redraw(el),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
        let _ = el;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let seed = args.get(1).cloned().unwrap_or_else(|| "andes".into());
    let hour = args.get(2).and_then(|s| s.parse::<f64>().ok());
    let weather = match args.get(3).map(|s| s.as_str()) {
        Some("snow") => "snow",
        Some("rain") => "rain",
        Some("fog") => "fog",
        Some("wind") => "wind",
        Some("storm") => "storm",
        Some("clear") => "clear",
        _ => "auto",
    };
    let moment = match args.get(4).map(|s| s.as_str()) {
        Some("condor") => "condor",
        Some("manada") => "manada",
        Some("bandada") => "bandada",
        Some("leorey") => "leorey",
        Some("kungleo") => "kungleo",
        Some("18sep") => "18sep",
        Some("none") => "none",
        _ => "auto",
    };
    if std::env::var_os("WAYLAND_DISPLAY")
        .map(|v| !v.is_empty())
        .unwrap_or(false)
    {
        match wayland::run(&seed, hour, weather, moment) {
            Ok(()) => return Ok(()),
            Err(e) => {
                eprintln!("[wall] wayland no disponible ({e}); usando X11/winit");
                std::env::remove_var("WAYLAND_DISPLAY");
            }
        }
    }

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new(&seed, hour, weather, moment);
    event_loop.run_app(&mut app)?;
    Ok(())
}
