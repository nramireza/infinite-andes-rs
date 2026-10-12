# 10 · UI y exportación

> Estado: estable · Actualizado: 2026-10-12

## Frontend nativo

El crate [`andes-native`](../crates/andes-native) abre una ventana `eframe`/`egui` con un panel de
control. Binarios/argumentos ([`src/main.rs`](../crates/andes-native/src/main.rs)):

- `infinite-andes` → ventana con panel.
- `infinite-andes wall` → pantalla completa sin panel (fondo de pantalla).
- `infinite-andes <seed> <hour> <weather> [moment]` → **sin ventana**, renderiza a PPM en `/tmp`
  (capturas/CI).

### Panel de control

| Elemento | Acción |
|----------|--------|
| Semilla | Escribir una semilla y regenerar |
| Relación | 16:9 · 21:9 · 32:9 (`ASPECTS`) |
| Auto-scroll | Recorrido automático |
| Reloj | Real (`clock=real`, hora local + sol de Chile) o rápido determinista |
| Clima | `auto`, `clear`, `snow`, `rain`, `fog`, `wind`, `storm` |
| Estación | `auto`, `verano`, `otono`, `invierno`, `primavera` |
| Momento | `auto`, `none`, `18sep`, `leorey`, `kungleo`, `condor`, `bandada`, `manada` |
| Región | `auto`, `altiplano`, `norte`, `centro`, `sur`, `patagonia`, `austral` |
| Floración | `auto`, `on`, `off` |
| Ajuste | Cubrir (`cover`) / contener (`contain`) |
| Actividad | Densidad de fauna según la actividad real del equipo (`off` por defecto; ver [17](17-reactividad-sistema.md)) |
| Exportar PNG | Descargar la vista actual (`export_png`) |
| Exportar tira | Tira larga de varias pantallas hacia la derecha (1–40, def. 8) |
| Fondo | Modo fondo de pantalla (pantalla completa) |

## Estado de la escena (`Scene`)

[`crates/andes-core/src/scene.rs`](../crates/andes-core/src/scene.rs):

- `camera_x` — posición horizontal del mundo.
- `auto_scroll`, `scroll_speed` — recorrido automático.
- `hour` (0–24), `time_auto`, `time_speed` — ciclo solar.
- `clock_fast`, `now`, `tz_hours`, `lat`, `longitude` — reloj y cálculo solar.
- `weather_auto`, `weather_timer`, `weather` — clima y transiciones.
- `season`/`season_phase` — estación fija o ciclo.
- `moment`, `biome_mode`, `bloom_mode` — momento raro, región y floración.
- `seed`, `t_sec`, `day_num` — semilla, tiempo y días simulados.
- `activity_mul` — multiplicador de densidad por actividad ([17](17-reactividad-sistema.md)).

## Exportación PNG

`export_png(fb, seed, hour)` (nativo) vuelca el `Framebuffer` a un `image::RgbaImage` y guarda
`infinite-andes_<seed>_<HHMM>.png` con la vista actual (al tamaño interno vigente, p. ej. 960×270 a
32:9). El render *headless* escribe además un **PPM** a `/tmp` para inspección y CI.

`export_strip(scene, seed, tiles)` (D-040 JS) renderiza una **tira larga** de `tiles` pantallas
hacia la derecha: redimensiona la escena temporalmente a `w·n`, renderiza una sola vez en un lienzo
ancho (cielo y parallax continúan sin costuras como en el original) y restaura el tamaño. Guarda
`infinite-andes_<seed>_strip<n>_x<x0>.png`.

## Web (WASM) y URLs

[`andes-web`](../crates/andes-web) exporta un **ABI C** (sin `wasm-bindgen`) que
[`web/main.js`](../web/main.js) reproduce en un `<canvas>`. Funciones principales:

| Símbolo | Acción |
|---------|--------|
| `ia_new(seed, w, h)` / `ia_free` | Crear/liberar la escena |
| `ia_update(dt)` / `ia_render()` | Avanzar y renderizar |
| `ia_pixels` / `ia_pixel_len` / `ia_width` / `ia_height` | Acceso al framebuffer en memoria lineal |
| `ia_set_weather` / `ia_set_biome` / `ia_set_moment` / `ia_set_season` / `ia_set_bloom` | Forzar estado |
| `ia_set_hour` | Hora fija (desactiva el tiempo automático) |
| `ia_set_autoscroll` | Auto-scroll on/off |
| `ia_set_clock_fast` | Reloj real o rápido |
| `ia_set_now(y, m0, d, h, mi, s, ms, tz)` | Fija la hora local (reloj real) |

Parámetros por URL (decodificados con `views.rs`, el mismo módulo puro que el original):

| Parámetro | Efecto |
|-----------|--------|
| `seed` | Semilla (texto o número) |
| `x` | Posición de mundo (desactiva auto-scroll) |
| `hour` | Hora fija (desactiva auto) |
| `weather` | Clima fijo |
| `aspect` | Relación (`32:9`, `2.4`, …) |
| `moment` | Momento raro |
| `season` | Estación fija o `auto` |
| `biome` | Región fija o `auto` |
| `bloom` | Floración (`auto`/`on`/`off`) |
| `clock` | Reloj `real` (def.) o `fast` |
| `fit` | `cover` (def.) o `contain` |

`decode_view`/`encode_view` ([`views.rs`](../crates/andes-core/src/views.rs)) tienen paridad exacta
con JS, incluidos los favoritos (`add_view`/`remove_view`/`parse_views`).

## Reloj real y sol

Con `clock_fast = false` la escena sigue la **hora local del equipo** y la **estación del hemisferio
sur** de la fecha (`clock.rs`); el amanecer/atardecer se calculan para Chile central
(`sun.rs`) y reasignan la curva de la paleta y la posición del astro
(`Scene::palette_hour`). El frontend nativo obtiene la hora con `chrono` y la inyecta con
`set_now`; así el **núcleo no depende de terceros** (D-009 del port).

## Modo fondo de pantalla

- **Nativo**: `andes-native wall` (pantalla completa) o el binario `wallpaper`
  ([16 · Modos de visualización](16-modos-de-visualizacion.md)).
- **Web**: kiosco/`fit` en el reproductor JS (equivalente a `?ui=0&fit=cover` del original).

## Pendiente

- Paridad de UI web con el panel del original (vistas favoritas, botón copiar enlace).
- Refinar el antialias de las nubes ([08 · Arte pixel](08-arte-pixel.md)).
- Persistir la preferencia del switch de actividad (el nativo no tiene `localStorage`).
