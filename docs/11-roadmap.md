# 11 · Roadmap

> Estado: en progreso · Actualizado: 2026-10-12

El proyecto avanzó **por fases** portando el original JS con paridad. Desde D-014/D-015, **este repo
es la fuente de verdad** y la paridad con JS es un test de referencia opcional
(`cargo test --features parity`); el paisaje evoluciona hacia una visión propia (D-016, estética
pixel-art nítida).

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
- [x] Paleta maestra generada **en Rust** con `gen-palette` ([14](14-paleta-maestra.md)).
- [x] `AGENTS.md` del repo Rust y enlace al índice desde el `README`.

## Fase 5 — Reorientación y pixel-art ✅ (completada)

- [x] **Rust canónico**: paridad como test opcional (D-014/D-015).
- [x] **Estética pixel-art nítida**: nubes y bordes de terreno sin antialias (D-016).
- [x] **Reorientación**: docs y ADRs alineados con la fuente de verdad de este repo.

## Fase 6 — Contenido ✅/en curso

- [x] **Línea de nieve estacional coherente** (D-017): el invierno ya no se ve árido.
- [x] **Estaciones más marcadas** (D-018): verano árido, más contraste y clima.
- [x] **Flora endémica nueva** (D-019): espino, litre, boldo, peumo, maitén, ciprés, puya, llareta,
      amancay y junquillo.
- [x] **Fauna marina y aves** (D-020): lobo marino, delfín chileno, gaviota, albatros y pelícano;
      `chungungo` y `pinguino` más frecuentes.
- [x] **Galería de diagnóstico** de flora y fauna (D-022): `cargo run -p andes-native --bin gallery`.
- [x] **Siluetas de flora propias** y luz direccional (D-023).
- [x] **Referencia Wikipedia + porte** por especie y araucaria fiel a su porte (D-024).
- [x] Revisión de siluetas de flora por `porte` (lote 1: araucaria, nalca, quillay, michay, chaura, boldo…).
- [x] Fauna por `porte` (lote 2): contorno de 1 px automático y redibujo de aves y mamíferos marinos.
- [ ] Calidad de dibujo: afinar el resto de sprites de fauna (cánidos, felinos, camélidos).
- [x] Completar fotos `refs` (Commons/iNaturalist CC0) para todas las especies reales.
- [ ] Momentos/sorpresas nuevas (eclipse, migraciones…).
- [ ] Biomas nuevos (islas, humedales, fiordos más marcados).

## Fase 7 — Features ✅/en curso

- [x] Export de tira larga en PNG ([10](10-ui-y-export.md)).
- [x] Actividad del sistema con sensores del SO ([17](17-reactividad-sistema.md)).
- [ ] Atajos de teclado y ocultar panel.
- [ ] Timelapse/GIF (rango de horas).
- [ ] **Multi-monitor de mundo continuo**: lo generado en el monitor derecho persiste al pasar al
      izquierdo (mismo mundo, `camera_x` compartido).

## Pendientes transversales

- [ ] **Modo información al clic** sobre flora y fauna (D-051 JS).
- [ ] Portar utilidades restantes de `pixel.js` si el rasterizador las necesita.
- [ ] Probar el fondo Wayland en un compositor real (wlroots).
- [ ] Cobertura de tests Rust ampliada a los dibujos (más allá de spawns y alturas).
- [ ] Rendimiento: límite de FPS configurable, cache de capas, perfilado.

## Backlog / ideas

- [ ] Igualar la UI web con el panel del original (vistas favoritas, copiar enlace, kiosco).
- [ ] Persistir la preferencia del switch de actividad en el frontend nativo (no hay `localStorage`).
- [ ] Referencias fotográficas PD/CC0 para el redibujo de sprites ([18 · Referencias](18-referencias.md)).
