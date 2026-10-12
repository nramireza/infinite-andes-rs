//! Fondo de pantalla en Wayland mediante `wlr-layer-shell` (capa *background*).
//! Usa software rendering con `wl_shm` (Argb8888) y vuelca el framebuffer del
//! núcleo. Se elige automáticamente cuando `WAYLAND_DISPLAY` está definido.

use std::convert::TryInto;
use std::time::Instant;

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_registry, delegate_shm,
    output::{OutputHandler, OutputState},
    reexports::client::{
        globals::registry_queue_init,
        protocol::{wl_output, wl_shm, wl_surface},
        Connection, QueueHandle,
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};

use andes_core::clock::DateTime;
use andes_core::framebuffer::Framebuffer;
use andes_core::rng::seed_to_int;
use andes_core::scene::Scene;
use andes_core::viewport::{width_for_ratio, ASPECT_PRESETS};

const HEIGHT: usize = 270;
const POOL_BYTES: usize = 4096 * 2160 * 4;

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

/// Escala el framebuffer al tamaño de la superficie y lo escribe como ARGB8888.
fn blit(fb: &Framebuffer, canvas: &mut [u8], width: usize, height: usize) {
    if width == 0 || height == 0 || fb.w == 0 || fb.h == 0 {
        return;
    }
    for (index, chunk) in canvas.chunks_mut(4).enumerate() {
        let x = index % width;
        let y = index / width;
        let sx = (x * fb.w / width).min(fb.w - 1);
        let sy = (y * fb.h / height).min(fb.h - 1);
        let i = (sy * fb.w + sx) * 4;
        let color = 0xFF00_0000u32
            | ((fb.px[i] as u32) << 16)
            | ((fb.px[i + 1] as u32) << 8)
            | (fb.px[i + 2] as u32);
        let array: &mut [u8; 4] = chunk.try_into().unwrap();
        *array = color.to_le_bytes();
    }
}

struct Wall {
    registry_state: RegistryState,
    output_state: OutputState,
    shm: Shm,
    pool: Option<SlotPool>,
    layer: LayerSurface,
    width: u32,
    height: u32,
    first_configure: bool,
    exit: bool,
    scene: Scene,
    fb: Framebuffer,
    hour: Option<f64>,
    last: Instant,
}

impl Wall {
    fn draw(&mut self, qh: &QueueHandle<Self>) {
        let (width, height) = (self.width.max(1) as usize, self.height.max(1) as usize);

        if self.hour.is_none() {
            let (dt_now, tz) = local_now();
            self.scene.set_now(dt_now, tz);
        }
        let now = Instant::now();
        let dt = (now - self.last).as_secs_f64().min(0.1);
        self.last = now;
        self.scene.update(dt);
        self.scene.render(&mut self.fb);

        let stride = width as i32 * 4;
        let pool = self
            .pool
            .get_or_insert_with(|| SlotPool::new(POOL_BYTES, &self.shm).expect("SlotPool"));
        let (buffer, canvas) = match pool.create_buffer(
            width as i32,
            height as i32,
            stride,
            wl_shm::Format::Argb8888,
        ) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[wall/wayland] create_buffer: {e}");
                return;
            }
        };
        blit(&self.fb, canvas, width, height);

        let surface = self.layer.wl_surface();
        surface.damage_buffer(0, 0, width as i32, height as i32);
        surface.frame(qh, surface.clone());
        if let Err(e) = buffer.attach_to(surface) {
            eprintln!("[wall/wayland] attach: {e}");
            return;
        }
        self.layer.commit();
    }
}

impl CompositorHandler for Wall {
    fn scale_factor_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: i32,
    ) {
    }
    fn transform_changed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: wl_output::Transform,
    ) {
    }
    fn frame(&mut self, _: &Connection, qh: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: u32) {
        self.draw(qh);
    }
    fn surface_enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }
    fn surface_leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_surface::WlSurface,
        _: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for Wall {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl LayerShellHandler for Wall {
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) {
        self.exit = true;
    }
    fn configure(
        &mut self,
        _: &Connection,
        qh: &QueueHandle<Self>,
        _: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _: u32,
    ) {
        if configure.new_size.0 > 0 {
            self.width = configure.new_size.0;
        }
        if configure.new_size.1 > 0 {
            self.height = configure.new_size.1;
        }
        if self.first_configure {
            self.first_configure = false;
            self.draw(qh);
        }
    }
}

impl ShmHandler for Wall {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

delegate_compositor!(Wall);
delegate_output!(Wall);
delegate_shm!(Wall);
delegate_layer!(Wall);
delegate_registry!(Wall);

impl ProvidesRegistryState for Wall {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}

pub fn run(
    seed: &str,
    hour: Option<f64>,
    weather: &'static str,
    moment: &'static str,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh)?;
    let layer_shell = LayerShell::bind(&globals, &qh)?;
    let shm = Shm::bind(&globals, &qh)?;

    let w = width_for_ratio(ASPECT_PRESETS[2].ratio) as usize;
    let mut scene = Scene::new(seed_to_int(seed), w, HEIGHT);
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

    let surface = compositor.create_surface(&qh);
    let layer = layer_shell.create_layer_surface(
        &qh,
        surface,
        Layer::Background,
        Some("infinite-andes"),
        None,
    );
    layer.set_anchor(Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT);
    layer.set_exclusive_zone(-1);
    layer.set_keyboard_interactivity(KeyboardInteractivity::None);
    layer.set_size(0, 0);
    layer.commit();

    let mut wall = Wall {
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &qh),
        shm,
        pool: None,
        layer,
        width: w as u32,
        height: HEIGHT as u32,
        first_configure: true,
        exit: false,
        scene,
        fb: Framebuffer::new(w, HEIGHT),
        hour,
        last: Instant::now(),
    };

    while !wall.exit {
        event_queue.blocking_dispatch(&mut wall)?;
    }
    Ok(())
}
