# AGENTS.md — Guía para asistentes (opencode)

Este archivo describe cómo trabajar en **Infinite Andes (Rust)** de forma consistente.

## Qué es el proyecto

Reescritura en **Rust** del paisaje procedural infinito de los Andes chilenos en pixel art. Objetivo:
un **fondo de pantalla vivo nativo** y, a futuro, el **navegador (WASM)**, compartiendo un mismo
núcleo determinista.

**Este repositorio es la fuente de verdad** (D-014). El proyecto original en JavaScript vive en el
repositorio hermano [`../infinite-andes`](../infinite-andes) y es **referencia e inspiración**: de
ahí salió el paisaje, pero aquí evoluciona hacia una visión propia y mejorada. La documentación está
en [`docs/`](docs/README.md).

## Cómo compilar, ejecutar y verificar

```bash
cargo test                      # unitarios + regresión (obligatorio antes de publicar)
cargo run -p andes-native       # ventana con panel (egui)
cargo run -p andes-native wall  # fondo de pantalla (pantalla completa, sin panel)
cargo run -p andes-native andes 12 clear condor   # sin ventana: PPM a /tmp
cargo run --bin wallpaper -- andes                # fondo X11/Wayland
```

- **Sin linter configurado por el repo**, pero usa `cargo fmt` y `cargo clippy` si están disponibles.
- **Web (WASM)**: `./scripts/build-web.sh` y sirve `web/` por HTTP.

### Referencia JS (opcional)

El proyecto JS es una **referencia**, no un requisito. Dos arneses manuales:

1. **Paridad matemática** (test opcional, apagado por defecto):

   ```bash
   cargo test --features parity
   ```

   Compara contra vectores generados del JS. Los datos del núcleo (`palette_data.rs`,
   `fauna_data.rs`, `flora_data.rs`) son **fuente de este repo** (editables); `gen-parity.mjs` es una
   herramienta de referencia que los **sobrescribe** y no forma parte del flujo normal.

2. **Paridad de píxeles** (diagnóstico manual):

   ```bash
   ./scripts/pixel-check.sh [seed] [hour] [weather] [biome] [moment] [bloom] [season] [aspect] [x]
   ```

   Reporta píxeles distintos, `|Δ|` medio/máximo e histograma. Como Rust usa pixel-art **sin
   antialias** (D-016), las diferencias con Canvas son esperadas y no son un fallo. El criterio
   visual propio son snapshots PNG (`cargo run -p andes-native --bin infinite-andes -- ...`).

### Documentación generada

- **Fichas de especies y tablas de 05/06**: `cargo run -p andes-core --bin gen-species`. No editar a
  mano `docs/especies/` ni las tablas entre `<!-- BEGIN:… -->` / `<!-- END:… -->`.
- **Paleta maestra (docs/14)**: `cargo run -p andes-core --bin gen-palette` (sin Node).

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
- **Pixel-art nítido**: sin antialias (D-016). Dibujar con `fill_rect`/primitivas cuantizadas a píxel
  entero; `js_round` para redondeos.

## Arquitectura (resumen)

Pipeline por fotograma en `crates/andes-core/src/scene.rs` (`Scene::render`): `sky` → capas de
`terrain` (con `flora` por capa) → `fauna` → `momentos` → clima. El orden y los detalles están en
[`docs/09-arquitectura.md`](docs/09-arquitectura.md).

`terrain.rs` es el corazón: define `LAYERS` (parallax, `base_y`, `amp`, `freq`, `seed`, nieve,
volcanes, ríos/fiordos) y las alturas (`ridge_height`, `bank_height`, `river_influence`).

## Al cambiar el diseño

1. El **diseño del paisaje** se decide aquí. Actualiza el doc correspondiente de
   [`docs/`](docs/README.md).
2. Registra la decisión en [`docs/12-decisiones.md`](docs/12-decisiones.md).
3. Añade una entrada en [`CHANGELOG.md`](CHANGELOG.md) (si existe).
4. Publica los cambios en GitHub (ver más abajo).

## Publicación en GitHub

El repositorio es `origin` (`git@github.com:nramireza/infinite-andes-rs.git`), rama `main`.

Al terminar una tanda:

1. Verifica: `cargo test`, `cargo fmt` y `cargo clippy`. Si algo falla, no se publica.
2. Regenera docs si aplica: `cargo run -p andes-core --bin gen-species` y
   `cargo run -p andes-core --bin gen-palette`.
3. Actualiza docs, `docs/12-decisiones.md` y sube la versión en `Cargo.toml`.
4. Commitea con un mensaje conciso siguiendo el historial y sube: `git push origin main`.

Nunca se publican cambios a medio verificar, ni secretos ni credenciales.

## Plantillas

Para nuevas features, decisiones o fichas, copia
[`docs/templates/`](docs/templates/) (`feature.md`, `decision.md`, `specimen.md`).

## Alcance / no-objetivos

- **No hay gameplay** (sin puntaje, niveles ni personaje jugable).
- **Sin audio**: es una pieza solo visual, pensada como **fondo de pantalla vivo**.
- No romper el determinismo ni la pixelación nítida (sin antialias).
- El diseño se decide en este repo; el JS es referencia, no obligación.
