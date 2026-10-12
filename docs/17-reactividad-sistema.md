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

## Frontend (pendiente)

En el original, un proceso local sin dependencias expone un JSON por HTTP:

```json
{ "cpu": 0.42, "ram": 0.71, "temp": 63, "load": 1.8 }
```

El mapeo (D-052 JS) produce un **multiplicador global** acotado a `[0.5, 1.8]`:

| Señal | Efecto |
|-------|--------|
| `cpu` alta | + (más actividad general) |
| `ram` alta | + (más bandadas) |
| Temperatura alta | − (menos actividad) |
| `load` alto | − (menos especies esquivas) |

En Rust **falta** implementar el puente (p. ej. `sysinfo` o lectura de `loadavg`/`/sys` en Linux) y
el control en el panel nativo. Mientras tanto, `set_activity_mul` permite inyectar el valor desde
cualquier frontend o test.

## Criterios de aceptación

- [x] Hook `activity_mul` en el núcleo, con multiplicador 1 por defecto.
- [ ] Puente local en el frontend nativo y control (switch) en el panel.
- [ ] Indicador visible del estado (equivalente al `act <m>` del HUD original).
- [x] El multiplicador no altera identidad ni posición, solo la densidad.

## Referencias

- D-052, D-055 (repo JS) · [06 · Fauna](06-fauna.md) · [09 · Arquitectura](09-arquitectura.md).
