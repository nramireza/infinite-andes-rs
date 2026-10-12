# 17 · Reactividad del sistema

> Estado: en progreso · Actualizado: 2026-10-12

Idea: que la **densidad de población** de la fauna reaccione a la **actividad real del equipo** (CPU,
RAM, temperatura o *load average*) de forma opcional. En el proyecto original se leen mediante un
**puente local opcional** (D-052 JS); aquí se documenta el **hook del núcleo** y lo que falta del
frontend.

## Principio: no romper el determinismo

La base del paisaje sigue siendo **determinista por semilla**: la *identidad* y la *posición* de cada
candidato se deciden por hash de chunk ([09 · Arquitectura](09-arquitectura.md)). La actividad
**solo pondera la densidad**, igual que la hora filtra la actividad de cada especie: nunca decide
*qué* animal ni *dónde*.

- Sin la actividad activa → el paisaje es idéntico y los tests no cambian.
- El multiplicador entra en `Scene::render` al llamar a `place_fauna(..., activity_mul)`, el mismo
  camino que usa el desierto florido (`chance × (1 + 0.6·bloom)`).

## Hook en el núcleo (implementado)

- `Scene::activity_mul` (por defecto `1.0`) y `Scene::set_activity_mul(mul)`; el valor se acota a
  `> 0` finito y se propaga a `place_fauna`.
- El núcleo **no lee sensores**: solo consume un multiplicador, de modo que sigue sin dependencias y
  determinista. La lectura del sistema es responsabilidad del frontend.

## Frontend nativo (implementado)

En el original, un proceso local sin dependencias expone un JSON por HTTP
(`{ cpu, ram, temp, load }`). El port **lee los sensores del SO directamente** en un hilo de fondo
(`crates/andes-native/src/activity.rs`), sin red ni dependencias:

| Señal | Origen (Linux) | Efecto |
|-------|----------------|--------|
| `cpu` | delta de `/proc/stat` | + (más actividad general) |
| `ram` | `/proc/meminfo` (`MemAvailable`/`MemTotal`) | + (más bandadas) |
| temperatura | máx. de `/sys/class/thermal/thermal_zone*/temp` | − (menos actividad) |
| `load` | `/proc/loadavg` (1 min) | − (menos especies esquivas) |

El sondeo (cada 5 s) publica la última muestra por canal; `poll_activity` la convierte con
`activity_multiplier` y llama a `set_activity_mul`. El switch del panel está **off por defecto**
(al apagarlo no se lee ningún sensor y el multiplicador vuelve a 1) y el HUD muestra el estado
(`act …` · `act <m>` · `act sin sensores`).

El mapeo (D-052 JS) produce un **multiplicador global** acotado a `[0.5, 1.8]`. Mientras no haya
sensores disponibles se usa `1.0` (paisaje idéntico).

## Criterios de aceptación

- [x] Hook `activity_mul` en el núcleo, con multiplicador 1 por defecto.
- [x] Puente de sensores del SO en el frontend nativo y switch en el panel (off por defecto).
- [x] Indicador visible del estado (equivalente al `act <m>` del HUD original).
- [x] El multiplicador no altera identidad ni posición, solo la densidad.
- [ ] Persistir la preferencia del switch (el nativo no tiene `localStorage`).

## Referencias

- D-052, D-055 (repo JS) · D-011 (port) · [06 · Fauna](06-fauna.md) · [09 · Arquitectura](09-arquitectura.md).
