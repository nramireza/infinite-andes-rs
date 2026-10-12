//! Frontend WebAssembly: expone la escena por ABI C para un reproductor JS
//! mínimo (`web/main.js`). Sin `wasm-bindgen`: se exportan funciones `#[no_mangle]`
//! y el JS lee el framebuffer directamente de la memoria lineal.

// Frontera FFI: los punteros los gestiona el reproductor JS, no Rust.
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use andes_core::framebuffer::Framebuffer;
use andes_core::scene::Scene;

pub struct WebScene {
    scene: Scene,
    fb: Framebuffer,
}

const SCRATCH_LEN: usize = 256;
static mut SCRATCH: [u8; SCRATCH_LEN] = [0; SCRATCH_LEN];

#[inline]
fn s<'a>(p: *const u8, len: usize) -> &'a str {
    if p.is_null() || len == 0 {
        return "";
    }
    unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(p, len)) }
}

#[no_mangle]
pub extern "C" fn ia_new(seed: u32, w: u32, h: u32) -> *mut WebScene {
    let (w, h) = (w.max(1) as usize, h.max(1) as usize);
    let mut scene = Scene::new(seed, w, h);
    scene.camera_x = 0.0;
    Box::into_raw(Box::new(WebScene {
        scene,
        fb: Framebuffer::new(w, h),
    }))
}

#[no_mangle]
pub extern "C" fn ia_free(p: *mut WebScene) {
    if !p.is_null() {
        unsafe { drop(Box::from_raw(p)) };
    }
}

#[no_mangle]
pub extern "C" fn ia_update(p: *mut WebScene, dt: f64) {
    let ws = unsafe { &mut *p };
    ws.scene.update(dt);
}

#[no_mangle]
pub extern "C" fn ia_render(p: *mut WebScene) {
    let ws = unsafe { &mut *p };
    ws.scene.render(&mut ws.fb);
}

#[no_mangle]
pub extern "C" fn ia_pixels(p: *mut WebScene) -> *const u8 {
    unsafe { (*p).fb.px.as_ptr() }
}

#[no_mangle]
pub extern "C" fn ia_pixel_len(p: *mut WebScene) -> usize {
    unsafe { (*p).fb.px.len() }
}

#[no_mangle]
pub extern "C" fn ia_width(p: *mut WebScene) -> u32 {
    unsafe { (*p).fb.w as u32 }
}

#[no_mangle]
pub extern "C" fn ia_height(p: *mut WebScene) -> u32 {
    unsafe { (*p).fb.h as u32 }
}

#[no_mangle]
pub extern "C" fn ia_scratch() -> *mut u8 {
    std::ptr::addr_of_mut!(SCRATCH) as *mut u8
}

#[no_mangle]
pub extern "C" fn ia_scratch_len() -> u32 {
    SCRATCH_LEN as u32
}

#[no_mangle]
pub extern "C" fn ia_set_weather(p: *mut WebScene, sptr: *const u8, len: usize) {
    let v = s(sptr, len);
    let t = match v {
        "snow" => "snow",
        "rain" => "rain",
        "fog" => "fog",
        "wind" => "wind",
        "storm" => "storm",
        "clear" => "clear",
        _ => "auto",
    };
    unsafe { (*p).scene.set_weather_type(t) };
}

#[no_mangle]
pub extern "C" fn ia_set_biome(p: *mut WebScene, sptr: *const u8, len: usize) {
    unsafe { (*p).scene.set_biome(s(sptr, len)) };
}

#[no_mangle]
pub extern "C" fn ia_set_moment(p: *mut WebScene, sptr: *const u8, len: usize) {
    unsafe { (*p).scene.set_moment(s(sptr, len)) };
}

#[no_mangle]
pub extern "C" fn ia_set_season(p: *mut WebScene, sptr: *const u8, len: usize) {
    unsafe { (*p).scene.set_season(s(sptr, len)) };
}

#[no_mangle]
pub extern "C" fn ia_set_bloom(p: *mut WebScene, sptr: *const u8, len: usize) {
    unsafe { (*p).scene.set_bloom(s(sptr, len)) };
}

#[no_mangle]
pub extern "C" fn ia_set_hour(p: *mut WebScene, hour: f64) {
    let ws = unsafe { &mut *p };
    ws.scene.hour = hour;
    ws.scene.time_auto = false;
}

#[no_mangle]
pub extern "C" fn ia_set_autoscroll(p: *mut WebScene, on: u32) {
    unsafe { (*p).scene.auto_scroll = on != 0 };
}

#[no_mangle]
pub extern "C" fn ia_set_clock_fast(p: *mut WebScene, fast: u32) {
    unsafe { (*p).scene.set_clock_fast(fast != 0) };
}

/// Fija la hora local (reloj real). `y` año, `m0` mes 0-based, etc.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn ia_set_now(
    p: *mut WebScene,
    y: i64,
    m0: u32,
    d: u32,
    h: u32,
    mi: u32,
    sec: u32,
    ms: u32,
    tz: f64,
) {
    use andes_core::clock::DateTime;
    unsafe {
        (*p).scene
            .set_now(DateTime::new(y, m0, d, h, mi, sec, ms), tz)
    };
}
