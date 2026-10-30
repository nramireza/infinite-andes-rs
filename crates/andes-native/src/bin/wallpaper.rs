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
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
#[cfg(target_os = "linux")]
use winit::platform::x11::{WindowAttributesExtX11, WindowType};
#[cfg(not(target_os = "linux"))]
use winit::window::Fullscreen;
use winit::window::{Window, WindowId, WindowLevel};

const HEIGHT: usize = 270;
/// El fondo es un paneo lento; 10 fps basta y mantiene bajo el uso de CPU.
const FRAME_MS: u64 = 100;

#[derive(Clone, Copy)]
struct DesktopGeometry {
    x: i32,
    y: i32,
    w: u32,
    h: u32,
}

fn iso(seed: &str) -> u32 {
    seed_to_int(seed)
}

/// Unión de todos los monitores conectados, en coordenadas globales.
fn desktop_geometry(el: &ActiveEventLoop) -> DesktopGeometry {
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;
    for monitor in el.available_monitors() {
        let p = monitor.position();
        let s = monitor.size();
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        max_x = max_x.max(p.x + s.width as i32);
        max_y = max_y.max(p.y + s.height as i32);
    }
    if min_x == i32::MAX {
        return DesktopGeometry {
            x: 0,
            y: 0,
            w: 3840,
            h: 1080,
        };
    }
    DesktopGeometry {
        x: min_x,
        y: min_y,
        w: (max_x - min_x) as u32,
        h: (max_y - min_y) as u32,
    }
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
    // Con escalado entero (4x en 3840x1080) cada fila origen se repite varias
    // veces; se expande una sola vez y se copia a las filas destino.
    let mut expanded = vec![0u32; w];
    let mut last_sy = usize::MAX;
    for (y, row_out) in out.chunks_mut(w).enumerate().take(h) {
        let sy = (y * fb.h / h).min(fb.h - 1);
        if sy != last_sy {
            let src = &fb.px[sy * fb.w * 4..];
            for (x, dst) in expanded.iter_mut().enumerate() {
                let i = cols[x] * 4;
                *dst = ((src[i] as u32) << 16) | ((src[i + 1] as u32) << 8) | (src[i + 2] as u32);
            }
            last_sy = sy;
        }
        row_out.copy_from_slice(&expanded);
    }
}

#[cfg(target_os = "linux")]
fn x11_window_id(window: &Window) -> Option<u32> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    match window.window_handle().map(|h| h.as_raw()) {
        Ok(RawWindowHandle::Xlib(h)) => Some(h.window as u32),
        Ok(RawWindowHandle::Xcb(h)) => Some(h.window.get()),
        _ => None,
    }
}

/// Reafirma geometría y apila la ventana al fondo. Necesario porque winit mapea
/// la ventana antes de que el servidor X la coloque en la posición final.
#[cfg(target_os = "linux")]
fn pin_background(conn: &x11rb::rust_connection::RustConnection, win: u32, geo: DesktopGeometry) {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::{ConfigureWindowAux, ConnectionExt as _, StackMode};
    use x11rb::wrapper::ConnectionExt as _;
    let geometry = ConfigureWindowAux::new()
        .x(geo.x)
        .y(geo.y)
        .width(geo.w)
        .height(geo.h)
        .border_width(0);
    let _ = conn.configure_window(win, &geometry);
    let _ = conn.configure_window(win, &ConfigureWindowAux::new().stack_mode(StackMode::BELOW));
    let _ = conn.flush();
    // Espera a que el servidor procese el cambio antes de seguir.
    let _ = conn.sync();
}

struct App {
    hour: Option<f64>,
    window: Option<Arc<Window>>,
    context: Option<Context<Arc<Window>>>,
    surface: Option<Surface<Arc<Window>, Arc<Window>>>,
    scene: Scene,
    fb: Framebuffer,
    geometry: DesktopGeometry,
    #[cfg(target_os = "linux")]
    x11: Option<(x11rb::rust_connection::RustConnection, u32)>,
    last: Instant,
    last_pin: Instant,
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
            None => {
                let (now, tz) = local_now();
                scene.set_clock_fast(false);
                scene.set_now(now, tz);
                // Deja hora y estación sincronizadas ya desde el primer fotograma.
                scene.update(0.0);
            }
        }
        Self {
            hour,
            window: None,
            context: None,
            surface: None,
            scene,
            fb: Framebuffer::new(w, HEIGHT),
            geometry: DesktopGeometry {
                x: 0,
                y: 0,
                w: 3840,
                h: 1080,
            },
            #[cfg(target_os = "linux")]
            x11: None,
            last: Instant::now(),
            last_pin: Instant::now(),
        }
    }

    fn pin(&self) {
        #[cfg(target_os = "linux")]
        if let Some((conn, win)) = &self.x11 {
            pin_background(conn, *win, self.geometry);
        }
    }

    fn redraw(&mut self, _el: &ActiveEventLoop) {
        let (Some(window), Some(surface)) = (self.window.as_ref(), self.surface.as_mut()) else {
            return;
        };
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
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let geo = desktop_geometry(el);
        self.geometry = geo;
        let ratio = geo.w as f64 / geo.h.max(1) as f64;
        let fb_w = width_for_ratio(ratio) as usize;
        if fb_w != self.fb.w {
            self.scene.resize(fb_w, HEIGHT);
            self.fb = Framebuffer::new(fb_w, HEIGHT);
        }
        let attrs = Window::default_attributes()
            .with_title("Infinite Andes")
            .with_decorations(false)
            .with_window_level(WindowLevel::AlwaysOnBottom)
            .with_position(PhysicalPosition::new(geo.x, geo.y))
            .with_inner_size(PhysicalSize::new(geo.w, geo.h));
        #[cfg(target_os = "linux")]
        let attrs = attrs
            .with_override_redirect(true)
            .with_x11_window_type(vec![WindowType::Desktop]);
        #[cfg(not(target_os = "linux"))]
        let attrs = attrs.with_fullscreen(Some(Fullscreen::Borderless(None)));
        let Ok(window) = el.create_window(attrs) else {
            return;
        };
        let window = Arc::new(window);
        let _ = window.set_cursor_hittest(false);
        #[cfg(target_os = "linux")]
        if let Some(win) = x11_window_id(&window) {
            match x11rb::connect(None) {
                Ok((conn, _screen)) => {
                    pin_background(&conn, win, geo);
                    self.x11 = Some((conn, win));
                }
                Err(e) => eprintln!("[wall] no se pudo conectar a X11: {e}"),
            }
        }
        self.last_pin = Instant::now();
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
        if self.last_pin.elapsed() >= std::time::Duration::from_secs(1) {
            self.pin();
            self.last_pin = Instant::now();
        }
        let next = self.last + std::time::Duration::from_millis(FRAME_MS);
        let now = Instant::now();
        if now >= next {
            if let Some(w) = &self.window {
                w.request_redraw();
            }
        } else {
            el.set_control_flow(ControlFlow::WaitUntil(next));
        }
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
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::new(&seed, hour, weather, moment);
    event_loop.run_app(&mut app)?;
    Ok(())
}
