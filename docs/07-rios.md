# 07 · Ríos

> Estado: estable · Actualizado: 2026-10-12

Implementado en [`crates/andes-core/src/terrain.rs`](../crates/andes-core/src/terrain.rs)
(`river_influence`, `river_event_centers`, `ridge_height`, el tallado y `draw_layer`/`draw_sea`). Las
**alturas** (incluida la influencia de ríos y fiordos) tienen paridad exacta con JS
([11 · Roadmap](11-roadmap.md)).

## Requisito

Los ríos deben verse **integrados al terreno** en el que aparecen, no como un elemento pegado
encima: moverse **a la misma velocidad que su capa** (heredar el parallax) y que el **contorno
generado del terreno sea el contorno del río**.

Además: **no debe haber ríos en la playa ni en el mar**. Los ríos van de la cordillera hacia abajo,
por el valle y la Costa.

## Enfoque actual (Modelo A · quebrada vertical)

El río es un **cauce tallado dentro de la generación de la capa**, leído como una quebrada que baja
desde la cordillera hacia el nivel de la capa siguiente:

- **Config por capa** (`Layer::rivers`, tipo `Channel`): presente en **Valle**
  (`spacing 1100`, `width 3.2`, `depth 7`, `wfreq 4.2`) y **Costa**
  (`spacing 1500`, `width 2.1`, `depth 5`, `wfreq 4.5`).
- **Muesca de entrada**: la silueta usa `ridge_height` (que incluye el tallado del río) y
  `bank_height` da el nivel del banco. En las columnas con río, la silueta se hunde, de modo que el
  borde generado forma el cauce. Al ser parte de la función de altura, **hereda el parallax**.
- **Canal centrado en la muesca**: el agua se dibuja centrada en el mismo punto que talla la muesca
  (`river_event_centers`), con un **meandro sutil** en profundidad: el centro se desplaza hasta
  0.55·`width` por lado dentro de la holgura libre, arranca centrado en el nacimiento y usa un seno
  determinista por evento (D-039 JS).
- **Conicidad y pendiente real**: el ancho por profundidad **nace como un punto** y se ensancha al
  bajar, con una ondulación leve (`wfreq`). El nacimiento y el recorrido se derivan de la **altura
  real del terreno en el centro** (`ridge_height`), así el cauce sigue la pendiente del relieve.
- **Nacimiento orgánico**: el agua brota unas filas **más abajo de la punta de la muesca** (desfase
  determinista por evento, `hash1`) con un **pequeño salto** brillante (`pal.seaHi`) en la primera
  fila (D-032 JS).
- **Integración**: el agua se recorta a `ridge_height` (no flota sobre el valle/cielo), lleva
  **bancos** oscuros a los lados y un brillo de borde. Color derivado de la paleta
  (`lerp_color(sea, seaHi, 0.3)`).
- **Oclusión natural**: la capa siguiente (más al frente) tapa la desembocadura.
- **Flora**: `flora_spawns` usa `bank_height` y omite las columnas con `river_influence > 0.25`.
- **Fauna del cauce**: la rana de Darwin y el huillín (`SpeciesData::river`) se colocan en el
  **borde del cauce** (`ev.xc ± offset`), no por chunk (D-022 JS).
- **Fiordos** (D-042 JS): el mismo tallado se reutiliza con el config `Layer::fjords` (valle y Costa),
  activo solo con el bioma **austral**; así los canales densos no afectan al resto del paisaje (ver
  [02 · Mundo](02-mundo.md)).

## Estado

Resuelto el problema principal: al fijar el centro del canal a la muesca, la silueta tallada y el
agua coinciden. El **meandro sutil** y el **cauce que sigue la pendiente real** están portados
(D-039 JS). La **influencia de ríos** (`river_influence`) es exacta; el **dibujo** se compara en el
arnés de píxeles.

## Opciones evaluadas

| Opción | Descripción | Estado |
|--------|-------------|--------|
| Tallado por capa, quebrada vertical (actual) | Muesca + canal centrado en la capa, hereda parallax | En uso |
| Meandro en profundidad | El canal se desplaza en x según la profundidad | En uso en versión **sutil** (≤0.55·width) |
| Río por tramos | Un tramo por capa con parallax propio, unión oculta | Descartado por costuras |
| Río único | Una sola velocidad para todo el río | Descartado: se despega de todas las capas |

## Próximos pasos

- Sin pendientes abiertos de ríos en el núcleo. El registro de diseño completo está en
  `12-decisiones.md` del repo JS (D-032, D-039).
