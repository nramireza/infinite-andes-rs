# 12 · Decisiones

> Estado: estable · Actualizado: 2026-10-12

Registro de decisiones propias del **port a Rust**. Las decisiones de **diseño** del paisaje
(geografía, paleta, flora, fauna, estaciones, momentos…) son las del **proyecto original** y viven en
el `docs/12-decisiones.md` del repo JS (`D-001`…`D-058`), que sigue siendo la referencia canónica.
Aquí solo se registran las decisiones que atañen a **este repositorio** (stack Rust, crates,
frontends, generación de datos).

Para añadir una, copia [`templates/decision.md`](templates/decision.md) y agrega una entrada con el
siguiente número.

## D-001 · Reescritura en Rust con el JS como fuente de verdad

- **Fecha:** 2026-10-12
- **Estado:** aceptada
- **Decisión:** reescribir Infinite Andes en Rust, manteniendo el proyecto JS como **fuente de
  verdad** del diseño y de los datos durante la migración. La paridad se persigue de forma **exacta**
  en lo matemático y **de píxel** en el rasterizado.
- **Motivo:** obtener un **fondo de pantalla nativo** real y un camino a WASM sin renunciar al
  paisaje ya validado.
- **Alternativas:** mantener solo JS (descartado: no hay fondo nativo); rediseñar en Rust
  (descartado: se perdería la paridad y el trabajo del original).

## D-002 · Workspace de cuatro crates

- **Fecha:** 2026-10-12
- **Estado:** aceptada
- **Decisión:** organizar el repo en `andes-core` (sin dependencias), `andes-app` (estado
  compartido), `andes-native` (ventana/fondo/export) y `andes-web` (WASM).
- **Motivo:** separar el núcleo portable y testeable de los frontends que sí pueden depender de
  terceros.

## D-003 · Rasterizador propio (`Framebuffer` RGBA8)

- **Fecha:** 2026-10-12
- **Estado:** aceptada
- **Decisión:** sustituir `CanvasRenderingContext2D` por un `Framebuffer` RGBA8 propio con
  `fill_rect`, `fill_rect_f`, `blend`, `fill_ellipse` y `vgradient`.
- **Motivo:** independencia de dependencias y testabilidad del render completo en `cargo test`.
- **Consecuencia:** el antialias de las nubes no coincide al 100 % con el de Canvas (pendiente).

## D-004 · Datos generados desde JS (no editar a mano)

- **Fecha:** 2026-10-12
- **Estado:** aceptada
- **Decisión:** `palette_data.rs`, `fauna_data.rs` y la metadata de identidad de flora/fauna se
  **generan** con `scripts/gen-parity.mjs` leyendo el JS; no se editan a mano.
- **Motivo:** una sola fuente de verdad y divergencias imposibles de compilar.
- **Consecuencia:** si cambia el JS, regenerar y volver a `cargo test`.

## D-005 · Paridad verificada por vectores y por píxeles

- **Fecha:** 2026-10-12
- **Estado:** aceptada
- **Decisión:** verificar con (1) vectores generados ejecutando el **código JS real**
  (`tests/parity.rs` + `gen-parity.mjs`) y (2) comparación de píxeles contra Chrome headless
  (`scripts/pixel-check.sh`).
- **Motivo:** la fidelidad al original es un requisito, no una aspiración.
- **Consecuencia:** resultado actual `|Δ|` medio ≈ 0.25, <0.5 % de píxeles con `Δ≥8`.

## D-006 · Frontend nativo con `eframe`/`egui`

- **Fecha:** 2026-10-12
- **Estado:** aceptada
- **Decisión:** la ventana y el panel de control usan `eframe`/`egui`; el export PNG usa `image`; la
  hora real se obtiene con `chrono` en el frontend y se inyecta con `Scene::set_now`.
- **Motivo:** UI inmediata y multiplataforma sin ensuciar el núcleo.
- **Consecuencia:** solo `andes-native` arrastra dependencias de terceros.

## D-007 · Fondo de pantalla nativo (X11 / Wayland)

- **Fecha:** 2026-10-12
- **Estado:** aceptada
- **Decisión:** el binario `wallpaper` detecta la sesión: **X11** usa EWMH
  (`_NET_WM_WINDOW_TYPE_DESKTOP`) y **Wayland** usa `wlr-layer-shell` (capa `background`) con
  `wl_shm` vía `smithay-client-toolkit`.
- **Motivo:** integrarse con el escritorio como fondo real, no como ventana normal.
- **Alternativas:** depender de herramientas externas (`swww`, `hyprpaper`); descartado para no
  imponer un compositor concreto.

## D-008 · WebAssembly sin `wasm-bindgen`

- **Fecha:** 2026-10-12
- **Estado:** aceptada
- **Decisión:** `andes-web` exporta un **ABI C** (`ia_new`, `ia_render`, …) y `web/main.js` lee el
  framebuffer directamente de la memoria lineal.
- **Motivo:** evitar la cadena de herramientas de `wasm-bindgen` y mantener el build simple.
- **Consecuencia:** el reproductor JS gestiona punteros y el paso de cadenas (scratch fijo).

## D-009 · Fichas de especies generadas en Rust

- **Fecha:** 2026-10-12
- **Estado:** aceptada
- **Decisión:** las fichas de `docs/especies/` y las tablas de 05/06 se generan con
  `cargo run -p andes-core --bin gen-species`, leyendo los registros del núcleo.
- **Motivo:** replicar `npm run species` (D-045 JS) sin depender de Node en el flujo Rust.
- **Consecuencia:** la metadata de identidad debe generarse desde el JS hacia el núcleo (D-004).

## D-010 · Paleta maestra generada desde `gen-parity.mjs`

- **Fecha:** 2026-10-12
- **Estado:** aceptada
- **Decisión:** la tabla de [14 · Paleta maestra](14-paleta-maestra.md) se emite desde
  `scripts/gen-parity.mjs` (que ya lee la paleta JS), en vez de un binario Rust aparte.
- **Motivo:** menos código; los datos de paleta ya se generan en ese mismo script.

## D-011 · Reactividad del sistema con sensores del SO (frontend nativo)

- **Fecha:** 2026-10-12
- **Estado:** aceptada
- **Decisión:** portar `activityMultiplier` al núcleo (`activity.rs`, sin dependencias) y, en el
  frontend nativo, leer la actividad del equipo **directamente de `/proc` y `/sys`** en un hilo de
  fondo (CPU, RAM, temperatura y `load`), en vez del puente HTTP del original (D-052 JS).
- **Motivo:** el nativo no puede reusar el puente de navegador; los sensores del SO evitan una
  dependencia de red y mantienen `andes-core` sin dependencias. El switch es **off por defecto**, así
  que el paisaje y los dorados no cambian mientras está apagado.
- **Alternativas:** añadir un cliente HTTP (`ureq`) para hablar con el puente de `activity.js`
  (descartado: dependencia y proceso externo innecesarios en el nativo).
- **Consecuencia:** la preferencia del switch aún no se persiste (el nativo no tiene `localStorage`);
  documentado en [17](17-reactividad-sistema.md).

## D-012 · Export de tira larga en el frontend nativo

- **Fecha:** 2026-10-12
- **Estado:** aceptada
- **Decisión:** portar `exportStrip` (D-040 JS) al nativo: redimensionar la escena de forma temporal
  a `w·n`, renderizar una sola vez en un lienzo ancho y restaurar el tamaño, guardando
  `infinite-andes_<seed>_strip<n>_x<x0>.png`.
- **Motivo:** replicar el export del original sin costuras (cielo y parallax continúan) y sin
  reescribir el render.
- **Consecuencia:** el ancho de la tira se acota a 1–40 pantallas; el uso de memoria crece con `n`.

## D-013 · Antialias de nubes con supermuestreo dirigido al borde

- **Fecha:** 2026-10-12
- **Estado:** aceptada
- **Decisión:** en `Framebuffer::fill_ellipse`, resolver interior/exterior de forma exacta por
  distancia normalizada y supermuestrear solo la banda de borde (`≈1 px`), con `N = 16`.
- **Motivo:** mejora la cobertura del antialias de Canvas a bajo coste (el interior no muestrea) y
  acelera las elipses grandes del terreno.
- **Consecuencia:** la paridad de píxeles de las nubes mejora de forma marginal; sigue pendiente
  afinar la diferencia de AA de Canvas (D-003).
