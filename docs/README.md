# Documentación — Infinite Andes (Rust)

Índice y convenciones de la documentación del **gemelo Rust** de Infinite Andes.

Este repositorio es la **fuente de verdad** (D-014) del paisaje en Rust; el proyecto original en
JavaScript ([`../../infinite-andes`](../../infinite-andes)) es **referencia e inspiración**. La
documentación describe el paisaje propio de este repo: la arquitectura Rust, la estética pixel-art,
el mundo (mundo, flora, fauna, estaciones…) y los **frontends** (nativo y WASM). La paridad con JS
es un test de referencia opcional (D-015).

## Índice

| # | Documento | Contenido |
|---|-----------|-----------|
| 00 | [Visión](00-vision.md) | Qué es, relación con el proyecto JS, objetivos y no-objetivos |
| 01 | [Experiencia](01-experiencia.md) | Exploración sin gameplay, recorrido y controles |
| 02 | [Mundo](02-mundo.md) | Geografía, 6 capas, parallax, biomas, escala |
| 03 | [Cielo y astros](03-cielo-astros.md) | Día/noche, sol y luna, aurora, estrellas, nubes |
| 04 | [Clima](04-clima.md) | Estados, transiciones, duración, intensidad |
| 05 | [Flora](05-flora.md) | Especies vegetales y su especificación |
| 06 | [Fauna](06-fauna.md) | Especies endémicas, comportamiento y sprites |
| 07 | [Ríos](07-rios.md) | Diseño e integración de los ríos |
| 08 | [Arte pixel](08-arte-pixel.md) | Resolución, paleta, framebuffer, sprites |
| 09 | [Arquitectura](09-arquitectura.md) | Crates, módulos, pipeline de render, determinismo y paridad |
| 10 | [UI y exportación](10-ui-y-export.md) | Panel nativo, export PNG, ABI web, URLs |
| 11 | [Roadmap](11-roadmap.md) | Fases de migración y estado de paridad |
| 12 | [Decisiones](12-decisiones.md) | Registro de decisiones (ADR ligera) del port |
| 13 | [Glosario](13-glosario.md) | Términos técnicos y especies |
| 14 | [Paleta maestra](14-paleta-maestra.md) | Tabla de colores por hora (generada) |
| 15 | [Estaciones](15-estaciones.md) | Ciclo estacional, tinte, nieve y clima |
| 16 | [Modos de visualización](16-modos-de-visualizacion.md) | Fondo de pantalla nativo, kiosco web, WASM |
| 17 | [Reactividad del sistema](17-reactividad-sistema.md) | Densidad de fauna según actividad del equipo |
| 18 | [Referencias](18-referencias.md) | Fotos reales PD/CC0 como guía de sprites |

Plantillas en [`templates/`](templates/). Fichas de flora y fauna en [`especies/`](especies/) (generadas con `cargo run -p andes-core --bin gen-species`).

## Convenciones

- Cada documento empieza con una cita de metadatos:

  ```markdown
  > Estado: borrador · Actualizado: 2026-10-12
  ```

  - **Estado**: `borrador` (se está escribiendo) · `en progreso` (parcial) · `estable` (consolidado).
- Títulos en español, frases concisas, tablas para datos repetibles.
- Los **placeholders** se marcan con `TODO:` en una lista.
- Las especies se documentan con nombre común, nombre científico y si son endémicas de Chile.
- Las decisiones de **diseño y arquitectura** se registran en `12-decisiones.md` (las del repo JS
  se citan como `D-XXX (JS)`).
- Referencias al código con la ruta (`crates/andes-core/src/terrain.rs`) y, cuando aporte,
  `archivo.rs:línea`.

## Cómo contribuir a la documentación

1. Elige el doc correspondiente (o crea uno nuevo numerado si es un tema nuevo).
2. Usa la plantilla adecuada de `templates/` para especímenes, features o decisiones.
3. Actualiza el índice de este archivo y el de [`../README.md`](../README.md).
4. Si tocas `flora.rs`/`fauna.rs`, regenera las fichas con `cargo run -p andes-core --bin gen-species`.
