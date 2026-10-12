# AGENTS.md — Guía para asistentes (opencode)

Este archivo describe cómo trabajar en **Infinite Andes (Rust)** de forma consistente.

## Qué es el proyecto

Reescritura en **Rust** del paisaje procedural infinito de los Andes chilenos en pixel art. Objetivo:
un **fondo de pantalla vivo nativo** y, a futuro, el **navegador (WASM)**, compartiendo un mismo
núcleo determinista.

El proyecto original en JavaScript vive en el repositorio hermano [`../infinite-andes`](../infinite-andes)
y sigue siendo la **fuente de verdad** durante la migración. Toda la documentación del port está en
[`docs/`](docs/README.md).

## Cómo compilar, ejecutar y verificar

```bash
cargo test                      # unitarios + paridad JS (obligatorio antes de publicar)
cargo run -p andes-native       # ventana con panel (egui)
cargo run -p andes-native wall  # fondo de pantalla (pantalla completa, sin panel)
cargo run -p andes-native andes 12 clear condor   # sin ventana: PPM a /tmp
cargo run --bin wallpaper -- andes                # fondo X11/Wayland
```

- **Sin linter configurado por el repo**, pero usa `cargo fmt` y `cargo clippy` si están disponibles.
- **Web (WASM)**: `./scripts/build-web.sh` y sirve `web/` por HTTP.

### Paridad con JavaScript

La fuente de verdad es el repo JS hermano (por defecto `../infinite-andes`, ajustable con
`ANDES_JS_DIR`). Hay dos niveles:

1. **Paridad matemática** — vectores generados ejecutando el código JS real:

   ```bash
   TZ=UTC node scripts/gen-parity.mjs   # regenera palette_data.rs, fauna_data.rs, flora_data.rs y parity_data.rs
   cargo test
   ```

   **No editar** `palette_data.rs`, `fauna_data.rs`, `flora_data.rs` ni `tests/generated/parity_data.rs`
   a mano. Si cambia el JS, regenerar y volver a `cargo test`; cualquier divergencia rompe la
   compilación de los tests.

2. **Paridad de píxeles** — comparación contra el render real de JS (Canvas):

   ```bash
   ./scripts/pixel-check.sh [seed] [hour] [weather] [biome] [moment] [bloom] [season] [aspect] [x]
   ```

   Reporta píxeles distintos, `|Δ|` medio/máximo e histograma. Escribe `/tmp/ia-rust.ppm`,
   `/tmp/ia-js.png`, `/tmp/ia-diff.png` y `/tmp/ia-diff8.png`. Estado actual: `|Δ|` medio ≈ 0.25,
   <0.5 % de píxeles con `Δ≥8` (bordes de nube).

### Documentación generada

- **Fichas de especies y tablas de 05/06**: `cargo run -p andes-core --bin gen-species`. No editar a
  mano `docs/especies/` ni las tablas entre `<!-- BEGIN:… -->` / `<!-- END:… -->`.
- **Paleta maestra (docs/14)**: se inyecta al correr `node scripts/gen-parity.mjs`.

## Convenciones de código

- **Rust 2021**, workspace de cuatro crates. `andes-core` **no tiene dependencias** (solo `std`);
  los frontends (`andes-native`, `andes-web`) sí pueden tenerlas.
- **Idioma**: nombres de dominio en español donde el original los usa (`valle`, `costa`, `playa`,
  `mar`); el código y esta guía en español.
- **Sin comentarios innecesarios**; los que hay explican *por qué*, no *qué*, y están en español.
- **Determinismo ante todo**: nada de aleatoriedad no reproducible para generar paisaje. Usar
  `Mulberry32`/`hash_int` de `rng.rs`. El paisaje debe reproducirse a partir de la semilla.
- **Semilla**: al crear o regenerar el terreno, `seed_layers(seed)` debe llamarse antes de dibujar.
- **Render** en un `Framebuffer` RGBA8 de **altura 270 px** y ancho por relación de aspecto
  (`viewport.rs`; 32:9 → 960×270 por defecto). Dibujar en coordenadas enteras (`fill_rect`).
- **Colores**: no escribir hex "a mano" en el dibujo del terreno/cielo; usar las claves de
  `get_palette(hour, weather, strength)`.
- **Paridad**: replicar la semántica de JS (`js_round`, `to_uint32`, aritmética `f64`).

## Arquitectura (resumen)

Pipeline por fotograma en `crates/andes-core/src/scene.rs` (`Scene::render`): `sky` → capas de
`terrain` (con `flora` por capa) → `fauna` → `momentos` → clima. El orden y los detalles están en
[`docs/09-arquitectura.md`](docs/09-arquitectura.md).

`terrain.rs` es el corazón: define `LAYERS` (parallax, `base_y`, `amp`, `freq`, `seed`, nieve,
volcanes, ríos/fiordos) y las alturas (`ridge_height`, `bank_height`, `river_influence`).

## Al cambiar el diseño

1. El **diseño del paisaje** se decide en el repo JS; aquí se **porta**. Actualiza el doc
   correspondiente de [`docs/`](docs/README.md).
2. Registra la decisión **del port** en [`docs/12-decisiones.md`](docs/12-decisiones.md); las del
   diseño van al repo JS.
3. Añade una entrada en [`CHANGELOG.md`](CHANGELOG.md) (si existe).
4. Publica los cambios en GitHub (ver más abajo).

## Publicación en GitHub

El repositorio es `origin` (`git@github.com:nramireza/infinite-andes-rs.git`), rama `main`.

Al terminar una tanda:

1. Verifica: `TZ=UTC node scripts/gen-parity.mjs` (si cambió el JS) y `cargo test`. Si algo falla,
   no se publica.
2. Regenera docs si aplica: `cargo run -p andes-core --bin gen-species`.
3. Actualiza docs, `docs/12-decisiones.md` y sube la versión en `Cargo.toml`.
4. Commitea con un mensaje conciso siguiendo el historial y sube: `git push origin main`.

Nunca se publican cambios a medio verificar, ni secretos ni credenciales.

## Plantillas

Para nuevas features, decisiones o fichas, copia
[`docs/templates/`](docs/templates/) (`feature.md`, `decision.md`, `specimen.md`).

## Alcance / no-objetivos

- **No hay gameplay** (sin puntaje, niveles ni personaje jugable).
- **Sin audio**: es una pieza solo visual, pensada como **fondo de pantalla vivo**.
- No romper el determinismo ni la pixelación nítida.
- No rediseñar el paisaje: el original JS manda.
