# 09 · Arquitectura

> Estado: estable · Actualizado: 2026-10-13

## Principios técnicos

- **Núcleo sin dependencias**: `andes-core` no depende de terceros (solo `std`); es portable a
  `wasm32-unknown-unknown` y testeable con `cargo test`.
- **Determinismo por semilla**: nada de aleatoriedad no reproducible para generar paisaje; la misma
  semilla reproduce el mismo paisaje.
- **Pixel-art nítido**: el rasterizador dibuja a píxel entero, sin antialias (D-016).
- **JS como referencia**: se conserva la semántica (`Math.round`, `ToUint32`, `Math.imul`, aritmética
  `f64`) donde ayuda a portar, pero la paridad exacta ya no es un requisito (D-014/D-015).
- **Baja resolución + escalado pixelado**: altura fija 270 px y ancho por relación de aspecto. Ver
  [08 · Arte pixel](08-arte-pixel.md).
- **Todo derivado de funciones**: el mundo no se almacena; la altura es una función de `x`.

## Workspace de crates

```
crates/
  andes-core/    # simulación + rasterizado, sin dependencias
  andes-app/     # estado compartido por los frontends (esqueleto)
  andes-native/  # ventana eframe/egui + export PNG + bin `wallpaper` (X11 EWMH / Wayland layer-shell)
  andes-web/     # WebAssembly (ABI C + reproductor JS en web/)
```

| Crate | Dependencias | Responsabilidad |
|-------|--------------|-----------------|
| `andes-core` | ninguna | Todo el núcleo determinista y el rasterizador |
| `andes-app` | `andes-core` | Estado de la app compartido (esqueleto) |
| `andes-native` | `eframe`/`egui`, `image`, `chrono`, `smithay-client-toolkit` | Ventana, panel, export PNG, fondo de pantalla |
| `andes-web` | `andes-core` | ABI C para el reproductor JS (`web/main.js`) |

## Módulos del núcleo (`crates/andes-core/src/`)

| Módulo Rust | Equivalente JS | Responsabilidad |
|-------------|----------------|-----------------|
| `rng.rs` | `rng.js` | `Mulberry32`, `hash_string`, `hash_int`, `hash1`, `seed_to_int` |
| `noise.rs` | `noise.js` | `value_noise1`, `fbm1`, `ridged1` |
| `math.rs` | — | `js_round`, `to_uint32`, `to_int32`, `clamp01` |
| `viewport.rs` | `viewport.js` | `BASE_H`, presets, `width_for_ratio`, `ratio_from_string`, `label_for_ratio` |
| `palette.rs` (+`palette_data.rs`) | `palette.js` | `get_palette`, `lerp_color`, `shade`, `apply_weather`, `apply_biome`, `night_amount` |
| `pixel.rs` | `pixel.js` | `radix_dither` (el resto llega con el rasterizador) |
| `framebuffer.rs` | (Canvas) | Lienzo RGBA8: `fill_rect`, `fill_rect_f`, `blend`, `fill_ellipse`, `vgradient` |
| `clock.rs` | `clock.js` | `local_hour`, `day_of_year`, estación, fase lunar |
| `sun.rs` | `sun.js` | `solar_declination`, `solar_times`, `solar_clock` |
| `seasons.rs` | `seasons.js` | `season_state`, tinte, sesgo de clima, `apply_season`, `season_glow_tint` |
| `biomes.rs` | `biomes.js` | pesos, geometría, tinte, floración y pools |
| `terrain.rs` | `terrain.js` | `LAYERS`, alturas, ríos/fiordos, dibujo de capas y mar |
| `sky.rs` | `sky.js` | gradiente, Vía Láctea, estrellas, aurora, astros y nubes |
| `weather.rs` | `weather.js` | partículas, niebla, relámpagos y crossfade |
| `flora.rs` | `flora.js` | registro `FLORA`, pools, colocación y dibujos |
| `fauna.rs` (+`fauna_data.rs`) | `fauna.js` | registro `SPECIES`, pools, actividad y sprites |
| `moments.rs` | `moments.js` | 18sep, Leo Rey, Kung Leo, cóndor, bandada, manada |
| `views.rs` | `views.js` | `decode_view`/`encode_view`, favoritos |
| `activity.rs` | `activity.js` | `activity_multiplier`: densidad de fauna según actividad |
| `scene.rs` | `scene.js` | orquestador: cámara, día/noche, clima, orden de render, reloj real |

## Pipeline de render (`Scene::render`)

1. Calcular la paleta clima a clima: `get_palette` y, si hay crossfade, mezclarla con la del clima
   saliente (`lerp_palettes`); luego `apply_season` y `apply_biome`. Calcular `biome_at` y
   `resolve_bloom`.
2. Calcular el estado celeste (`Sky::celestial`) y la fase lunar (`Scene::moon_phase`).
3. `Sky::draw(...)`: gradiente + Vía Láctea + estrellas + aurora + resplandor (con tinte estacional
   y fase lunar).
4. `Sky::draw_body(...)`: el astro **al fondo** (lo tapan nubes y terreno).
5. `Sky::draw_clouds(...)`: nubes por delante del astro y por detrás del terreno.
6. `moment_sky(...)`: ambiente de momento sobre el cielo (p. ej. 18-sep).
7. `Weather::draw_lightning(...)`: rayo de tormenta, por delante del cielo y detrás del terreno.
8. **Capas de atrás hacia adelante** (`LAYERS`):
   - Si la capa es `sea`: `draw_sea` (con `effective_wind` del clima y la coordenada `x` del astro).
   - Si no: `Terrain::draw_layer` (incluye canales de río) + `place_flora`.
   - `place_fauna` en todas (incluido el mar, para aves y mamíferos marinos).
9. `moment_ground(...)`: personajes de momento en primer plano.
10. `Weather::draw(...)`: partículas y niebla (de ambos climas durante el crossfade).

## Determinismo y semilla

- `seed_to_int` convierte la semilla (texto o número) a entero de 32 bits.
- `seed_layers(seed)` (`terrain.rs`) mezcla la semilla en cada capa (`layer_seeds`); **debe
  llamarse antes de dibujar** (lo hace `Scene::new` y `Terrain::seed_layers`).
- La flora y la fauna combinan la semilla del usuario con la de la capa y siembran su PRNG con
  `hash_int` (**entero** de 32 bits) por chunk. Ver D-011 (JS).
- Los **momentos** (`moments.rs`) también son deterministas: un `hash_int` por bloque de `SPACING` px
  decide si hay momento y cuál. Ver D-014 (JS).
- Los ríos usan la semilla de la capa.

## Parallax

Cada capa tiene `parallax` (0 = fondo, 1 = frente). Para una columna de pantalla `sx`:
`world_x = camera_x * parallax + sx`. La altura se evalúa en `world_x`. Las capas delanteras se
dibujan después y ocultan a las traseras.

## Formato de una capa (`LAYERS`)

| Campo | Significado |
|-------|-------------|
| `name` | Identificador (`andes`, `precordillera`, `valle`, `costa`, `playa`, `mar`) |
| `parallax` | Velocidad relativa de la cámara |
| `base_y`, `amp` | Centro y amplitud vertical de la silueta |
| `freq` | Frecuencia del ruido de altura |
| `seed` | Semilla base de la capa (se mezcla con la del usuario) |
| `rugged` | Peso del ruido ridged (0..1) |
| `snow_frac` | Umbral de nieve (≥1.4 = sin nieve) |
| `light_key`, `dark_key` | Claves de paleta para el borde y el cuerpo |
| `alpha` | Opacidad (perspectiva atmosférica) |
| `volcano` | `Some(Volcano { spacing, height })` (solo Andes) |
| `rocky` | Vetas de roca (Andes) |
| `fields` | Textura de campos (Valle) |
| `beach` | Línea de marea (Playa) |
| `sea` | La capa es el mar (dibujo propio con `draw_sea`) |
| `rivers` / `fjords` | `Option<Channel>` (Valle, Costa) |
| `darken` | Oscurecimiento base de la capa |
| `flora` | `Option<FloraCfg>` (espacial) |
| `fauna` | `Option<FaunaCfg>` (espacial) |

## Registro de especies (`FLORA` / `SPECIES`)

- `FLORA` (`flora.rs`): una entrada por tipo dibujable
  (`kind`, `height`, `bloom_only`, `zones`, + metadata de identidad).
- `SPECIES` (`fauna_data.rs`, **fuente**): `movement`, `active`, `speed`, `range`, `fps`, `rarity`,
  `tracks`, `bloom`, `chunk_ok`, `river`, `zones`, `palette`, `frames` (+ metadata de identidad).

**Zonas (pools).** `zones[bioma][capa]` da el peso con que la especie aparece en ese bioma y capa.
`biome_flora_pool`/`biome_fauna_pool` (`biomes.rs`) construyen el pool efectivo a partir de las zonas.
`river` coloca la especie en el borde del cauce y `bloom` la suma al desierto florido.

**Metadatos del núcleo.** `palette_data.rs`, `fauna_data.rs` y la metadata de identidad de
flora/fauna son **fuente de este repo** (D-014). `scripts/gen-parity.mjs` los puede refrescar desde
el JS (herramienta de referencia, no flujo normal).

## Referencia con JavaScript (opcional)

### 1. Paridad matemática (`cargo test --features parity`)

`crates/andes-core/tests/parity.rs` compara contra vectores generados del JS
(`scripts/gen-parity.mjs`). Está detrás de la feature `parity` (D-015): no corre en `cargo test` ni
bloquea la evolución de Rust. Cubre `rng`, `noise`, `pixel`, `viewport`, `palette`, `clock`, `sun`,
`seasons`, `biomes`, alturas de `terrain`, `meteor`, `celestial`, clima, spawns, momentos y vistas.

### 2. Paridad de píxeles (`scripts/pixel-check.sh`, manual)

Renderiza la misma escena en Chrome headless (vía `scripts/pixel-harness/capture.html`) y con
`andes-core` (`bin/pixelcheck`), y reporta píxeles distintos, `|Δ|` medio, máximo e histograma. Como
Rust es pixel-art sin antialias (D-016), las diferencias en bordes son esperadas; es un diagnóstico,
no un criterio.

### Estado

Ver el [11 · Roadmap](11-roadmap.md) y el [`README`](../README.md). La matemática y los spawns siguen
la semántica del JS; el rasterizado es **propio** (nítido) y puede divergir a propósito.

## Tests

`cargo test` ejecuta los tests unitarios (dentro de cada módulo) y la regresión propia; los tests de
paridad son opcionales (`--features parity`). Los tests de render usan un `Framebuffer` real (no hace
falta navegador), así que el pipeline completo se puede ejercitar sin entorno gráfico:

```bash
cargo test
cargo run -p andes-native --bin infinite-andes -- andes 12 clear   # render sin ventana a /tmp (PPM)
```

## Rendimiento

El rasterizado es el camino caliente. `Framebuffer` compone con **enteros** (alfa a `0..=255`),
escribe columnas de 1 px con paso de fila y trocea las filas anchas; el cielo se pinta con un
`fill_rect` por fila. El terreno cachea por columna (`Column` `Copy`, `snow_shift` incluido),
precalcula las alturas del cauce por columna y resuelve los pools de flora/fauna por chunk. El
`blit` de los fondos usa una LUT de columnas.

Medición: `cargo run --release -p andes-core --example bench` (~7 ms/fotograma a 960×270, 60 fps en
ventana y fondo). El perfil `dev` va optimizado para que `cargo run` (debug) también sea fluido. Ver
D-021.

## Limitaciones conocidas

- Un elemento que cruce varias capas con distinto parallax no puede quedar perfectamente pegado a
  todas; en los ríos se resuelve manteniendo el cauce dentro de una sola capa (ver
  [07 · Ríos](07-rios.md)).
- El coste de dibujar canales evalúa ruido por columna de cauce; aceptable con pocos ríos por
  pantalla.
- El antialias de las nubes difiere del de Canvas (supermuestreo vs analítico); pendiente de refinar.
