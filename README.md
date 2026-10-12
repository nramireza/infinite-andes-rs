# Infinite Andes (Rust)

Reescritura en Rust del paisaje procedural infinito de los Andes chilenos.
Objetivo: un **fondo de pantalla vivo nativo** y, a futuro, el **navegador (WASM)**,
compartiendo un mismo núcleo determinista.

El proyecto original en JavaScript vive en el repositorio hermano
[`../infinite-andes`](../infinite-andes) y sigue siendo la **fuente de verdad**
mientras dure la migración.

## Estado

Fases 1–4 del [roadmap](docs/11-roadmap.md) completadas: núcleo matemático y rasterizador con
**paridad exacta** respecto a JS, frontends (nativo, fondo y WASM) y documentación.

| Módulo | Contenido | Paridad |
|--------|-----------|---------|
| `andes-core::rng` | `mulberry32`, `hashString`, `hashInt`, `hash1`, `seedToInt` | ✅ exacta |
| `andes-core::noise` | `valueNoise1`, `fbm1`, `ridged1` | ✅ exacta |
| `andes-core::math` | `jsRound`, `toUint32`, `toInt32`, `clamp01` | — |
| `andes-core::viewport` | `BASE_H`, presets, `widthForRatio`, `ratioFromString`, `labelForRatio` | ✅ exacta |
| `andes-core::palette` | `getPalette`, `lerpColor`, `shade`, `applyWeather`, `applyBiome`, `nightAmount` | ✅ exacta |
| `andes-core::pixel` | `radixDither` (el resto llega con el rasterizador) | ✅ exacta |
| `andes-core::framebuffer` | lienzo RGBA8, `fill_rect`, `fill_rect_f` (cobertura fraccionaria), `blend`, `fill_ellipse`, `vgradient` | razonable |
| `andes-core::clock` | `localHour`, `dayOfYear`, fase de estación, fase lunar | ✅ exacta (calendario UTC) |
| `andes-core::sun` | `solarDeclination`, `solarTimes`, `solarClock` | ✅ exacta |
| `andes-core::seasons` | `seasonState`, tinte, sesgo de clima, `applySeason`, `seasonGlowTint` | ✅ exacta |
| `andes-core::biomes` | pesos, geometría, tinte y floración (pools incluidos) | ✅ exacta |
| `andes-core::terrain` | `LAYERS`, alturas, influencia de ríos/fiordos, dibujo de capas y mar | ✅ alturas exactas |
| `andes-core::sky` | gradiente, Vía Láctea, estrellas, aurora, astros y nubes | ✅ init/astros exactos |
| `andes-core::weather` | partículas, niebla, relámpagos y crossfade de climas | ✅ spawn exacto |
| `andes-core::flora` | registro, pools, colocación por chunk y 22 dibujos | ✅ spawns exactos |
| `andes-core::fauna` | registro (21 especies), pools, actividad y sprites | ✅ spawns exactos |
| `andes-core::moments` | 18sep, Leo Rey, Kung Leo, cóndor, bandada, manada | ✅ eventos exactos |
| `andes-core::views` | `encodeView`/`decodeView`, favoritos y parser JSON | ✅ exacta |
| `andes-core::activity` | `activityMultiplier` (densidad de fauna según actividad del equipo) | ✅ exacta |
| `andes-core::scene` | orquestador: cámara, ciclo día/noche, clima, orden de render, reloj real | completo |

Pendientes transversales (detalle en el [roadmap](docs/11-roadmap.md)): refinar el
**antialias de las nubes** para acercar la paridad de píxeles, el **modo información al clic**
(D-051 JS), portar utilidades restantes de `pixel.js`, probar el fondo Wayland en un compositor
real y ampliar la cobertura de tests a los dibujos.

El binario nativo ya renderiza la escena completa (cielo + clima + terreno + flora
+ fauna + momentos) y ofrece una ventana con panel de control:

```bash
cargo run -p andes-native                        # ventana con panel (egui)
cargo run -p andes-native wall                   # fondo de pantalla (pantalla completa, sin panel)
cargo run -p andes-native andes 12 clear condor  # sin ventana: PPM a /tmp
```

En la ventana: semilla, relación de aspecto, **reloj real** (hora local del
equipo + sol de Chile) o rápido determinista, auto-scroll, clima, bioma, momento,
estación, floración, **actividad del equipo** (densidad de fauna, off por defecto),
ajuste cubrir/contener, **Exportar PNG**, **Exportar tira** y modo fondo.

### Fondo de pantalla del escritorio

```bash
cargo run --bin wallpaper -- andes          # reloj real
cargo run --bin wallpaper -- andes 12 clear # hora fija
```

El binario detecta la sesión automáticamente:

- **X11**: ventana sin bordes a pantalla completa con `_NET_WM_WINDOW_TYPE_DESKTOP`
  (EWMH), reafirmada tras el mapeo para quedar **detrás de los iconos**.
- **Wayland**: usa `wlr-layer-shell` (capa `background`) con software rendering
  (`wl_shm`, Argb8888) vía `smithay-client-toolkit`. *(Compila; no probado en este
  entorno, que no tiene compositor Wayland.)*

`Esc` para salir (X11); `kill` en Wayland.

### WebAssembly (navegador)

```bash
./scripts/build-web.sh                 # compila andes-web y copia el .wasm
python3 -m http.server 8000 --directory web
# abrir http://localhost:8000/?seed=andes&aspect=32:9
```

Sin `wasm-bindgen`: `andes-web` exporta un ABI C (`ia_new`, `ia_render`, …) y
`web/main.js` coloca el framebuffer en un `<canvas>`. Parámetros por URL: `seed`,
`hour`, `weather`, `aspect`, `moment`, `season`, `biome`, `bloom`, `clock`
(`real|fast`), `fit` (`cover|contain`). Requiere el target
`wasm32-unknown-unknown` (`rustup target add wasm32-unknown-unknown`).

Los datos de fauna (`fauna_data.rs`) se **generan** desde `fauna.js`; el registro
de flora está portado a mano. `andes-core` no tiene dependencias; solo los
frontends (`andes-native` usa `eframe`/`image`) las necesitan.

Pendiente: probar el fondo Wayland en un compositor real (wlroots), afinar el
antialias de las nubes para acercar aún más la paridad de píxeles y el modo
información al clic (D-051 JS).

## Cómo trabajar

```bash
cargo test         # unitarios + paridad JS
cargo run -p andes-native   # renderiza el terreno a /tmp/infinite-andes-terrain.ppm
```

### Paridad con JavaScript

Los tests de paridad (`crates/andes-core/tests/parity.rs`) se comparan contra
vectores generados ejecutando **el código JS real** del repositorio hermano:

```bash
TZ=UTC node scripts/gen-parity.mjs
```

El script lee `../infinite-andes` (configurable con `ANDES_JS_DIR`) y regenera:

- `crates/andes-core/src/palette_data.rs` — datos de paleta, clima y bioma.
- `crates/andes-core/tests/generated/parity_data.rs` — vectores de paridad.

**No editar esos archivos a mano.** Si cambia el JS, regenerar y volver a
`cargo test`: cualquier divergencia rompe la compilación de los tests.

> Los vectores de fecha se generan en `TZ=UTC` para que el calendario civil de
> Rust coincida con el `Date` local de JS.

## Determinismo

La regla de oro del proyecto original se mantiene: nada de aleatoriedad no
reproducible. Todo deriva de la semilla. La migración replica la semántica exacta
de JS (`Math.round`, `ToUint32`, `Math.imul`, aritmética `f64`) para que el
paisaje sea idéntico al de la versión web.

## Paridad de píxeles vs Canvas

El rasterizador Rust se compara contra el render real de JS (Canvas) píxel a
píxel:

```bash
./scripts/pixel-check.sh [seed] [hour] [weather] [biome] [moment] [bloom] [season] [aspect] [x]
```

El arnés renderiza la misma escena en Chrome headless (vía
`scripts/pixel-harness/capture.html`, que importa el `src/` del repo JS) y con
`andes-core` (`bin/pixelcheck`), y reporta: píxeles distintos, `|Δ|` medio por
canal (cielo/suelo), máximo e histograma. Escribe `/tmp/ia-rust.ppm`,
`/tmp/ia-js.png`, `/tmp/ia-diff.png` y `/tmp/ia-diff8.png`.

Resultado actual (varias escenas): `|Δ|` medio ≈ **0.25**, máximo ≈ **38**, y
**<0.5%** de píxeles con `Δ≥8` (bordes de nube, por el antialias analítico de
Canvas frente a nuestro supermuestreo). El resto son diferencias de redondeo
de ±1 en el degradado del cielo y el compositing alfa.

## Documentación

La documentación del port vive en [`docs/`](docs/README.md) (índice y convenciones) y refleja el
diseño del proyecto original adaptado a Rust: arquitectura, paridad, mundo, cielo, clima,
flora/fauna, estaciones y modos de visualización. Las **fichas de especies** de `docs/especies/` y
las tablas de `docs/05-flora.md`/`docs/06-fauna.md` se generan en Rust:

```bash
cargo run -p andes-core --bin gen-species
```

Convenciones de trabajo (comandos, determinismo, paridad, publicación) en [`AGENTS.md`](AGENTS.md).

## Estructura

```
crates/
  andes-core/    # simulación + rasterizado, sin dependencias
  andes-app/     # estado compartido por los frontends (esqueleto)
  andes-native/  # ventana eframe/egui + export PNG + bin `wallpaper` (X11 EWMH / Wayland layer-shell)
  andes-web/     # WebAssembly (ABI C + reproductor JS en `web/`)
scripts/
  gen-parity.mjs # genera datos y vectores desde el JS (+ paleta maestra en docs/14)
  build-web.sh   # compila andes-web (wasm32) y copia el módulo
  pixel-check.sh # arnés de paridad de píxeles vs Canvas (Chrome headless)
  pixel-harness/ # capture.html (render JS determinista para capturas)
web/             # index.html + main.js (reproductor del .wasm)
docs/            # documentación del port (ver docs/README.md; fichas en docs/especies/)
```
