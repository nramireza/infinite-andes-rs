# 15 · Estaciones del año

> Estado: estable · Actualizado: 2026-10-12

El paisaje tiene un **ciclo estacional** que tiñe la paleta, mueve la **línea de nieve** y sesga el
**clima**. Implementado en [`crates/andes-core/src/seasons.rs`](../crates/andes-core/src/seasons.rs) y
compuesto en `Scene::render` (D-019 JS). Tiene **paridad exacta** con JS.

## Las cuatro estaciones

| Estación | Carácter | `snow_shift` | Clima favorecido |
|----------|----------|--------------|------------------|
| **verano** | base actual, cálido y despejado | −0.18 (nieve más alta) | despejado |
| **otoño** | ámbar/rojo en flora y Costa, horizonte cálido | +0.12 | viento, lluvia |
| **invierno** | frío y desaturado | +0.32 (nieve más baja) | nieve, lluvia, niebla, tormenta |
| **primavera** | verdes frescos | −0.05 | despejado |

- `season_snow_shift` se **suma** al desplazamiento del bioma ([02 · Mundo](02-mundo.md)); solo actúa
  en capas que ya tienen nieve. El verano **no tiñe**: es la referencia base.
- `SEASON_IDS` fija el orden (`verano`, `otono`, `invierno`, `primavera`).

## Ciclo y mezcla

- **Real (por defecto)**: la fase sale de la **fecha del equipo** (`season_phase_for_date`,
  [`clock.rs`](../crates/andes-core/src/clock.rs)), con el **hemisferio sur** como referencia. No
  avanza por tiempo de ejecución.
- **Ciclo rápido** (reloj rápido): ciclo temporal lento, **`SEASON_DURATION = 120 s`** por estación
  (año ≈ 8 min). La fase avanza con `season_speed = 1/SEASON_DURATION` y envuelve cada 4.
- **Fijo**: `set_season("verano"|"otono"|"invierno"|"primavera")` o el selector **Estación**.
- **Mezcla**: `season_state(phase, mode)` devuelve estación actual, siguiente y una mezcla `t` con
  **meseta + crossfade**, de modo que hay estaciones claras y transiciones suaves, sin saltos.

## Composición de la paleta

Orden por fotograma en `Scene::render`:

```
get_palette(hora, clima)  →  apply_season(estación)  →  apply_biome(bioma)
```

- `apply_season(pal, state, strength)` interpola claves de atmósfera y terreno/flora
  (`floraL/D`, `costaL/D`, `valleyL/D`, `sand/D`, `skyHorizon`). No toca astros ni mar.
- El **resplandor del astro** se tiñe aparte con `season_glow_tint` (cálido en otoño/primavera, frío
  en invierno), aplicado en el cielo sin pasar por la paleta global.
- La fuerza del tinte se **atenúa de noche** (`strength · (1 − 0.85·night_amount)`), para no lavar el
  cielo nocturno (`SEASON_STRENGTH = 0.5`).
- El **bioma** se aplica al final: manda en lo regional.

## Clima por estación

`pick_season_weather(state, rnd)` sortea el clima automático con los pesos `season_weather_bias` de la
estación mezclados entre actual y siguiente (invierno favorece nieve/lluvia/tormenta; verano,
despejado). Ver [04 · Clima](04-clima.md).

## Parámetros

| Parámetro | Valores | Descripción |
|-----------|---------|-------------|
| `season`  | `auto` (def.), `verano`, `otono`, `invierno`, `primavera` | Estación fija o según reloj/ciclo |
| `clock`   | `real` (def.), `fast` | Estación y hora reales, o ciclo rápido |

`ia_set_season` / `ia_set_clock_fast` en el ABI web; selector **Estación** y **Reloj** en el panel
nativo.

## Pendientes

- Ninguna del núcleo: las variantes de flora por estación (caída de hojas, brotes, flores) y la
  duración del día por estación están portadas (D-027, D-031 JS; ver [05 · Flora](05-flora.md)).
