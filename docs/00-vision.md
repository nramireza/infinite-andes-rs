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

El proyecto original en JavaScript ([`../../infinite-andes`](../../infinite-andes)) es la **fuente
de verdad** mientras dure la migración:

- El **diseño** (geografía, paleta, flora, fauna, clima, estaciones) es el del original. Este repo
  no rediseña: **replica**.
- La **paridad** se persigue de forma **exacta** en lo matemático y **de píxel** en el
  rasterizado ([09 · Arquitectura](09-arquitectura.md), [11 · Roadmap](11-roadmap.md)).
- Los **datos** que el original declara en código (paleta, fauna) se **generan** desde el JS con
  `scripts/gen-parity.mjs`; no se editan a mano.

## Inspiración

- [{Shan, Shui}\*](https://github.com/LingDong-/shan-shui-inf) de Lingdong Huang: paisaje
  procedural infinito, composición en capas y *scroll* sin fin.
- El proyecto original Infinite Andes (JS): misma geografía, paleta y especies de Chile.
- Los grabados y pinturas de paisaje andino y la tradición del naturalismo chileno.

## Objetivos

- **Fondo de pantalla nativo real**: una ventana sin bordes que vive detrás de los iconos del
  escritorio (X11 vía EWMH, Wayland vía `wlr-layer-shell`), ligera y sin dependencias en el núcleo.
- **Núcleo determinista y portable**: `andes-core` sin dependencias, testeable en Node-like
  (Rust) y compilable a `wasm32-unknown-unknown`.
- **Paridad exacta** con JS: la misma semilla reproduce el mismo paisaje, píxel a píxel en lo
  posible.
- Representar el **perfil oeste→este** de Chile central: Mar · Playa · Cordillera de la Costa ·
  Valle central · Precordillera · Cordillera de los Andes.
- **Sincronía con Chile**: por defecto la hora, la estación y el amanecer/atardecer reales
  (decisión de diseño D-027 del repo JS).

## No-objetivos

- Gameplay, puntaje, niveles ni personaje jugable.
- **Audio**: es una pieza solo visual.
- Rediseñar el paisaje: se porta el del proyecto original.
- Dependencias en el núcleo: solo los **frontends** (`eframe`/`egui`, `image`, `chrono`,
  `smithay-client-toolkit`) pueden depender de terceros.

## Público

Personas que disfrutan de arte generativo, pixel art y paisajes; sin instrucciones ni interacción.

## Preguntas abiertas

- [ ] Refinar el antialias de las nubes para acercar aún más la paridad de píxeles con Canvas.
- [ ] Probar el fondo Wayland en un compositor real (wlroots).
- [x] Núcleo matemático con paridad exacta (Fase 1, ver [11 · Roadmap](11-roadmap.md)).
