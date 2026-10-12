# 16 · Modos de visualización

> Estado: en progreso · Actualizado: 2026-10-12

Formas de dejar el paisaje corriendo. En Rust, el eje central es el **fondo de pantalla nativo**;
el kiosco/PWA del navegador es el equivalente del proyecto original.

## Ventana normal (implementado)

El binario principal de [`andes-native`](../crates/andes-native) abre una ventana con panel; sirve
para explorar y para exportar PNG ([10 · UI y exportación](10-ui-y-export.md)).

```bash
cargo run -p andes-native
```

## Modo fondo en la ventana (implementado)

`andes-native wall` abre una ventana **sin bordes a pantalla completa**, sin panel. Es la forma
sencilla de dejar el paisaje en pantalla dentro del propio proceso egui.

```bash
cargo run -p andes-native wall
```

## Fondo de pantalla del escritorio (implementado)

Binario dedicado `wallpaper`, que detecta la sesión automáticamente:

```bash
cargo run --bin wallpaper -- andes          # reloj real
cargo run --bin wallpaper -- andes 12 clear # hora fija
cargo run --bin wallpaper -- andes          # X11: Esc para salir
```

| Sesión | Integración | Estado |
|--------|-------------|--------|
| **X11** | Ventana sin bordes a pantalla completa con `_NET_WM_WINDOW_TYPE_DESKTOP` (EWMH), reafirmada tras el mapeo para quedar **detrás de los iconos** | En uso |
| **Wayland** | `wlr-layer-shell` (capa `background`) con software rendering (`wl_shm`, Argb8888) vía `smithay-client-toolkit` | Compila; **pendiente** de probar en un compositor real (wlroots) |

- Salir: `Esc` en X11; `kill` en Wayland.
- El binario vive en [`crates/andes-native/src/bin/wallpaper.rs`](../crates/andes-native/src/bin/wallpaper.rs)
  (con el backend Wayland en `wallpaper/wayland.rs`).

## Web (WASM) (implementado)

El crate [`andes-web`](../crates/andes-web) compila el núcleo a `wasm32-unknown-unknown` y lo expone
por ABI C; [`web/main.js`](../web/main.js) lo reproduce en un `<canvas>`:

```bash
./scripts/build-web.sh
python3 -m http.server 8000 --directory web
# abrir http://localhost:8000/?seed=andes&aspect=32:9
```

Parámetros de URL equivalentes al kiosco del original: `fit` (`cover`/`contain`), `clock`
(`real|fast`), etc. ([10 · UI y exportación](10-ui-y-export.md)).

## Live wallpaper PWA (fuera de alcance en Rust)

El proyecto original ofrece una **PWA instalable** con *service worker* (offline) y **Wake Lock**
(D-050 JS). En Rust ese frente **no se persigue**: el objetivo nativo es el binario `wallpaper` y el
camino web es WASM. Se documenta como **capa externa opcional**: cualquier herramienta que muestre
web puede apuntar al reproductor WASM.

## Criterios de aceptación

- [x] Ventana normal y modo `wall` (pantalla completa) en `andes-native`.
- [x] Fondo X11 detrás de los iconos (EWMH).
- [ ] Fondo Wayland probado en un compositor `wlr-layer-shell` real.
- [x] Reproductor WASM funcional con parámetros de URL.
