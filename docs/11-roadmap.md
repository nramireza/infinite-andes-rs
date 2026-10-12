# 11 · Roadmap

> Estado: en progreso · Actualizado: 2026-10-12

El port avanza **por fases**, siempre con el repo JS como fuente de verdad. El objetivo de cada fase
es igualar el comportamiento del original y verificarlo (paridad exacta o de píxel según el caso)
antes de pasar a la siguiente.

## Fase 1 — Núcleo matemático ✅ (completada)

Portado con **paridad exacta** respecto a JS:

| Módulo | Contenido | Paridad |
|--------|-----------|---------|
| `rng` | `mulberry32`, `hashString`, `hashInt`, `hash1`, `seedToInt` | ✅ exacta |
| `noise` | `valueNoise1`, `fbm1`, `ridged1` | ✅ exacta |
| `math` | `jsRound`, `toUint32`, `toInt32`, `clamp01` | — |
| `viewport` | `BASE_H`, presets, `widthForRatio`, `ratioFromString`, `labelForRatio` | ✅ exacta |
| `palette` | `getPalette`, `lerpColor`, `shade`, `applyWeather`, `applyBiome`, `nightAmount` | ✅ exacta |
| `pixel` | `radixDither` | ✅ exacta |

## Fase 2 — Rasterizador y simulación ✅ (completada, en pulido)

| Módulo | Contenido | Paridad |
|--------|-----------|---------|
| `framebuffer` | lienzo RGBA8, `fill_rect`, `fill_rect_f`, `blend`, `fill_ellipse`, `vgradient` | razonable |
| `clock` | hora local, día del año, estación, fase lunar | ✅ exacta (calendario) |
| `sun` | `solarDeclination`, `solarTimes`, `solarClock` | ✅ exacta |
| `seasons` | `seasonState`, tinte, sesgo de clima, `applySeason`, `seasonGlowTint` | ✅ exacta |
| `biomes` | pesos, geometría, tinte y floración (pools incluidos) | ✅ exacta |
| `terrain` | `LAYERS`, alturas, influencia de ríos/fiordos, dibujo de capas y mar | ✅ alturas exactas |
| `sky` | gradiente, Vía Láctea, estrellas, aurora, astros y nubes | ✅ init/astros exactos |
| `weather` | partículas, niebla, relámpagos y crossfade | ✅ spawn exacto |
| `flora` | registro, pools, colocación por chunk y 22 dibujos | ✅ spawns exactos |
| `fauna` | registro (21 especies), pools, actividad y sprites | ✅ spawns exactos |
| `moments` | 18sep, Leo Rey, Kung Leo, cóndor, bandada, manada | ✅ eventos exactos |
| `views` | `encodeView`/`decodeView`, favoritos y parser | ✅ exacta |
| `activity` | `activityMultiplier`: densidad de fauna según actividad del equipo | ✅ exacta |
| `scene` | orquestador completo (cámara, día/noche, clima, orden de render, reloj real) | completo |

## Fase 3 — Frontends ✅ (completada)

- [x] **Nativo**: ventana `eframe`/`egui` con panel, reloj real, export PNG
      ([10 · UI y exportación](10-ui-y-export.md)).
- [x] **Fondo de pantalla**: `wallpaper` con X11 EWMH y Wayland `wlr-layer-shell`
      ([16 · Modos de visualización](16-modos-de-visualizacion.md)).
- [x] **WebAssembly**: `andes-web` (ABI C) + reproductor `web/main.js`
      ([10 · UI y exportación](10-ui-y-export.md)).

## Fase 4 — Documentación y datos ✅ (completada)

- [x] Documentación `00`–`18` adaptada al port.
- [x] Fichas de especies generadas **en Rust** con `gen-species` ([05](05-flora.md), [06](06-fauna.md)).
- [x] Paleta maestra generada con `gen-parity.mjs` ([14](14-paleta-maestra.md)).
- [x] `AGENTS.md` del repo Rust y enlace al índice desde el `README`.

## Pendientes transversales

- [x] **Reactividad del sistema** en el frontend nativo: sensores del SO + switch y HUD
      ([17 · Reactividad del sistema](17-reactividad-sistema.md)).
- [x] **Export de tira larga** en PNG (D-040 JS) en el panel nativo
      ([10 · UI y exportación](10-ui-y-export.md)).
- [~] **Antialias de las nubes**: borde con supermuestreo dirigido (mejora la paridad; resta afinar).
- [ ] **Modo información al clic** sobre flora y fauna (D-051 JS).
- [ ] Portar utilidades restantes de `pixel.js` si el rasterizador las necesita.
- [ ] Probar el fondo Wayland en un compositor real (wlroots).
- [ ] Cobertura de tests Rust ampliada a los dibujos (más allá de spawns y alturas).

## Backlog / ideas

- [ ] Igualar la UI web con el panel del original (vistas favoritas, copiar enlace, kiosco).
- [ ] Persistir la preferencia del switch de actividad en el frontend nativo (no hay `localStorage`).
- [ ] Referencias fotográficas PD/CC0 para el redibujo de sprites ([18 · Referencias](18-referencias.md)).
