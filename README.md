# Infinite Andes (Rust)

Reescritura en Rust del paisaje procedural infinito de los Andes chilenos.
Objetivo: un **fondo de pantalla vivo nativo** y, a futuro, el **navegador (WASM)**,
compartiendo un mismo núcleo determinista.

El proyecto original en JavaScript vive en el repositorio hermano
[`../infinite-andes`](../infinite-andes) y sigue siendo la **fuente de verdad**
mientras dure la migración.

## Estado

Este repositorio es la **fuente de verdad** (D-014): el paisaje nació del proyecto JS, pero aquí
evoluciona hacia una visión propia y mejorada, con estética **pixel-art nítida (sin antialias,
D-016)**. Frontends nativo, fondo de pantalla y WASM completos.

| Módulo | Contenido | Paridad |
|--------|-----------|---------|
| `andes-core::rng` | `mulberry32`, `hashString`, `hashInt`, `hash1`, `seedToInt` | ✅ exacta |
| `andes-core::noise` | `valueNoise1`, `fbm1`, `ridged1` | ✅ exacta |
| `andes-core::math` | `jsRound`, `toUint32`, `toInt32`, `clamp01` | — |
| `andes-core::viewport` | `BASE_H`, presets, `widthForRatio`, `ratioFromString`, `labelForRatio` | ✅ exacta |
| `andes-core::palette` | `getPalette`, `lerpColor`, `shade`, `applyWeather`, `applyBiome`, `nightAmount` | ✅ exacta |
| `andes-core::pixel` | `radixDither` (el resto llega con el rasterizador) | ✅ exacta |
| `andes-core::framebuffer` | lienzo RGBA8 nítido: `fill_rect`, `blend`, `fill_ellipse`, `fill_disc`, `vgradient` | propia |
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

Pendientes (detalle en el [roadmap](docs/11-roadmap.md)): más contenido (estaciones más marcadas,
flora endémica, fauna marina, biomas, momentos), features (atajos, timelapse/GIF, multi-monitor de
mundo continuo), el **modo información al clic** (D-051 JS), probar el fondo Wayland en un
compositor real y ampliar la cobertura de tests a los dibujos.

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

Pendiente: probar el fondo Wayland en un compositor real (wlroots), el modo
información al clic (D-051 JS), multi-monitor de mundo continuo y más contenido
(flora, fauna marina, estaciones más marcadas).

## Cómo trabajar

```bash
cargo test         # unitarios + regresión (autocontenido, sin Node)
cargo run -p andes-native   # ventana con panel
```

### Referencia con JavaScript (opcional)

Este repo es la fuente de verdad (D-014); la paridad con JS es un test de
referencia apagado por defecto:

```bash
cargo test --features parity   # compara contra vectores del JS
```

Los datos del núcleo (`palette_data.rs`, `fauna_data.rs`, `flora_data.rs`) son
**fuente de este repo** (editables). `scripts/gen-parity.mjs` solo los refresca
desde `../infinite-andes` (`ANDES_JS_DIR`) y no forma parte del flujo normal.

> Los vectores de fecha se generan en `TZ=UTC` para que el calendario civil de
> Rust coincida con el `Date` local de JS.

## Determinismo

Nada de aleatoriedad no reproducible: todo deriva de la semilla. Se conserva la
semántica numérica del original (`Math.round`, `ToUint32`, `Math.imul`, `f64`)
donde ayuda a portar, pero el paisaje evoluciona aquí.

## Pixel-art y comparación con el original

La estética es pixel-art **nítido, sin antialias** (D-016): nubes y bordes del
terreno se dibujan a píxel entero. `./scripts/pixel-check.sh` sigue disponible
como diagnóstico manual contra el render JS (Canvas):

```bash
./scripts/pixel-check.sh [seed] [hour] [weather] [biome] [moment] [bloom] [season] [aspect] [x]
```

Reporta píxeles distintos, `|Δ|` medio por canal, máximo e histograma. Como el
antialias es una decisión estética propia, las diferencias en los bordes con
Canvas son esperadas. El criterio visual propio son snapshots PNG
(`cargo run -p andes-native --bin infinite-andes -- andes 12 clear`).

## Documentación

La documentación vive en [`docs/`](docs/README.md) (índice y convenciones). Las
**fichas de especies** de `docs/especies/` y las tablas de
`docs/05-flora.md`/`docs/06-fauna.md` se generan en Rust, igual que la tabla de
la paleta maestra:

```bash
cargo run -p andes-core --bin gen-species
cargo run -p andes-core --bin gen-palette
```

Convenciones de trabajo (comandos, determinismo, publicación) en [`AGENTS.md`](AGENTS.md).

## Estructura

```
crates/
  andes-core/    # simulación + rasterizado, sin dependencias
  andes-app/     # estado compartido por los frontends (esqueleto)
  andes-native/  # ventana eframe/egui + export PNG + bin `wallpaper` (X11 EWMH / Wayland layer-shell)
  andes-web/     # WebAssembly (ABI C + reproductor JS en `web/`)
scripts/
  gen-parity.mjs # referencia: refresca datos/vectores desde el JS (opcional)
  build-web.sh   # compila andes-web (wasm32) y copia el módulo
  pixel-check.sh # diagnóstico de píxeles vs Canvas (Chrome headless)
  pixel-harness/ # capture.html (render JS determinista para capturas)
web/             # index.html + main.js (reproductor del .wasm)
docs/            # documentación del port (ver docs/README.md; fichas en docs/especies/)
```
