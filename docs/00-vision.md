# 00 · Visión

> Estado: en progreso · Actualizado: 2026-10-12

## Qué es

**Infinite Andes (Rust)** es la reescritura en **Rust** del paisaje procedural infinito de los
Andes chilenos en **pixel art**. El objetivo es un **fondo de pantalla vivo nativo** y, a futuro,
el **navegador (WebAssembly)**, compartiendo un mismo **núcleo determinista**
([`crates/andes-core`](../crates/andes-core)).

No es un juego: es una **pieza generativa explorable**, como una pintura continua que se despliega,
pensada para quedar corriendo en pantalla sin interacción.

## Relación con el proyecto original

Este repositorio es la **fuente de verdad** (D-014). El proyecto original en JavaScript
([`../../infinite-andes`](../../infinite-andes)) es **referencia e inspiración**:

- El paisaje nació allí (geografía, paleta, flora, fauna, clima, estaciones), pero aquí evoluciona
  hacia una **visión propia y mejorada**: se puede rediseñar y ampliar.
- La **paridad** con JS dejó de ser un requisito: queda como **test de referencia opcional**
  (`cargo test --features parity`, D-015).
- Los **datos** del núcleo (`palette_data.rs`, `fauna_data.rs`, `flora_data.rs`) son fuente de este
  repo y se editan aquí; `scripts/gen-parity.mjs` solo los refresca desde el JS si hace falta.

## Inspiración

- [{Shan, Shui}\*](https://github.com/LingDong-/shan-shui-inf) de Lingdong Huang: paisaje
  procedural infinito, composición en capas y *scroll* sin fin.
- El proyecto original Infinite Andes (JS): misma geografía, paleta y especies de Chile.
- Los grabados y pinturas de paisaje andino y la tradición del naturalismo chileno.

## Objetivos

- **Fondo de pantalla nativo real**: una ventana sin bordes que vive detrás de los iconos del
  escritorio (X11 vía EWMH, Wayland vía `wlr-layer-shell`), ligera y sin dependencias en el núcleo.
- **Núcleo determinista y portable**: `andes-core` sin dependencias, testeable sin navegador y
  compilable a `wasm32-unknown-unknown`.
- **Determinismo propio**: la misma semilla reproduce el mismo paisaje. La estética es pixel-art
  **nítido, sin antialias** (D-016).
- Representar el **perfil oeste→este** de Chile central: Mar · Playa · Cordillera de la Costa ·
  Valle central · Precordillera · Cordillera de los Andes.
- **Sincronía con Chile**: por defecto la hora, la estación y el amanecer/atardecer reales
  (decisión de diseño D-027 del repo JS).

## No-objetivos

- Gameplay, puntaje, niveles ni personaje jugable.
- **Audio**: es una pieza solo visual.
- Antialias: la imagen se mantiene nítida pixel-art (D-016).
- Dependencias en el núcleo: solo los **frontends** (`eframe`/`egui`, `image`, `chrono`,
  `smithay-client-toolkit`) pueden depender de terceros.

## Público

Personas que disfrutan de arte generativo, pixel art y paisajes; sin instrucciones ni interacción.

## Preguntas abiertas

- [ ] Probar el fondo Wayland en un compositor real (wlroots).
- [ ] Multi-monitor con **mundo continuo** (el contenido generado a la derecha persiste al pasar al
      monitor izquierdo).
- [x] Núcleo determinista y rasterizador pixel-art (sin antialias) del port.
