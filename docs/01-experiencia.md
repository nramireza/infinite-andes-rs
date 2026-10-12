# 01 · Experiencia

> Estado: en progreso · Actualizado: 2026-10-12

## Concepto

El paisaje se recorre **sin objetivo**: se contempla y se explora. El usuario decide el ritmo —
parar cuando una composición le gusta, o dejar que el paisaje se desplace solo.

En Rust hay **dos formas de verlo**: la **ventana nativa** con panel ([`andes-native`](../crates/andes-native))
y el **modo fondo de pantalla** a pantalla completa, sin panel
([16 · Modos de visualización](16-modos-de-visualizacion.md)).

## Formas de recorrerlo

- **Manual**: `◀`/`▶` en el panel nativo (desplazamiento por saltos).
- **Auto-scroll**: desplazamiento continuo a velocidad ajustable (`auto_scroll`, `scroll_speed`).
- **Cambio de paisaje**: nueva semilla genera otro mundo.
- **Enlace/vista exacta** (web): parámetros de URL decodificados por `views.rs`
  ([10 · UI y exportación](10-ui-y-export.md)).

## Qué se puede observar

- Amanecer naranjo con el sol detrás de la cordillera.
- Mediodía con el sol fuera de pantalla (pasa por encima).
- Atardecer cálido sin sol visible (queda tras la cámara).
- Noche con estrellas y aurora austral.
- Clima cambiando solo: despejado, nieve, lluvia, niebla, viento y tormenta.
- Ríos que bajan por el valle y un portezuelo en la cordillera de la Costa.
- Volcanes con penacho en los Andes.

## Controles

### Ventana nativa (panel egui, [`crates/andes-native/src/main.rs`](../crates/andes-native/src/main.rs))

| Control | Función |
|---------|---------|
| Semilla | Escribir una semilla (texto o número) y regenerar |
| Relación | 16:9, 21:9 o 32:9 (por defecto) |
| Auto-scroll | Recorrido automático |
| Reloj | Real (hora local del equipo + sol de Chile) o rápido determinista |
| Clima | Automático o forzado (despejado, nieve, lluvia, niebla, viento, tormenta) |
| Estación | Automática (ciclo) o fija (verano, otoño, invierno, primavera) |
| Momento | Raro automático o forzado (18-sep, Leo Rey, Kung Leo, cóndor, bandada, manada) |
| Región | Procedural, altiplano, norte, centro, sur, Patagonia o Austral (fiordos) |
| Floración | Desierto florido automático, forzado o desactivado |
| Ajuste | Cubrir (`cover`) o contener (`contain`) |
| Exportar PNG | Descargar la vista actual |
| Fondo | Entra en modo fondo de pantalla (pantalla completa, sin panel) |

### Línea de comandos (`andes-native`)

```bash
cargo run -p andes-native                        # ventana con panel
cargo run -p andes-native wall                   # fondo de pantalla (sin panel)
cargo run -p andes-native andes 12 clear condor  # sin ventana: PPM a /tmp
```

### Web (WASM)

`andes-web` expone un ABI C que `web/main.js` reproduce en un `<canvas>`; los controles vienen por
**parámetros de URL** (`seed`, `hour`, `weather`, `aspect`, `moment`, `season`, `biome`, `bloom`,
`clock`, `fit`) decodificados con el mismo `views.rs` del núcleo
([10 · UI y exportación](10-ui-y-export.md)).

## Principios de sensación

- **Tranquilidad**: ritmos lentos, sin urgencia ni interrupciones.
- **Legibilidad**: las capas deben leerse como geografía, no como manchas.
- **Vida sutil**: el movimiento (agua, clima, penachos, pasto) evita que se sienta estático.
- **Coherencia**: hora y clima afectan todo (cielo, montañas, mar, agua, flora).
- **Sincronía con Chile**: por defecto la hora, la estación y el amanecer/atardecer siguen la
  realidad (D-027 del repo JS); con el reloj rápido vuelve el ciclo de demostración.
