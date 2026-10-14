# 08 · Arte pixel

> Estado: estable · Actualizado: 2026-10-12

## Resolución y escalado

- **Altura interna fija: 270 px** (`BASE_H`). El ancho sale de la **relación de aspecto**
  (`width_for_ratio`): 16:9 → 480, 21:9 → 630 y **32:9 → 960 (por defecto)**. Constantes y utilidades
  en [`viewport.rs`](../crates/andes-core/src/viewport.rs).
- La relación se elige en el panel nativo (**Relación**) o con `?aspect=` en la web (p. ej.
  `?aspect=21:9` o `?aspect=2.4`); se redimensiona en caliente (`Scene::resize`).
- El escalado a pantalla es del **frontend**: el nativo dibuja el framebuffer en la ventana egui y
  la web coloca los píxeles en un `<canvas>` con `image-rendering: pixelated`.
- Todo el dibujo es con coordenadas enteras y `fill_rect` de 1 px para evitar bordes borrosos.

## Framebuffer

El rasterizador propio vive en [`framebuffer.rs`](../crates/andes-core/src/framebuffer.rs): un búfer
**RGBA8** con primitivas sin dependencias:

| Método | Uso |
|--------|-----|
| `set` / `blend` | Píxel opaco / mezcla alfa |
| `fill_rect` | Rectángulo de píxeles enteros (sprites, bandas) |
| `fill_ellipse` / `fill_disc` | Elipses y discos rasterizados a píxel entero (**sin antialias**, nubes y astros) |
| `vgradient` | Gradiente vertical (cielo) |

Es la pieza que sustituye al `CanvasRenderingContext2D` del original y lo que hace el núcleo
testeable sin navegador. Todo se dibuja **nítido** (D-016): no hay cobertura fraccionaria.

## Paleta

- Centralizada en [`palette.rs`](../crates/andes-core/src/palette.rs) como **claves por franja
  horaria** interpoladas por hora (`get_palette(hour, weather, strength)`).
- Los **datos** (claves, targets de clima, tintes de bioma) viven en `palette_data.rs`, **generados**
  desde `palette.js` con `scripts/gen-parity.mjs`.
- La **tabla completa por hora** está en [14 · Paleta maestra](14-paleta-maestra.md), generada con
  `scripts/gen-parity.mjs`.
- Campos de la paleta: cielo (`skyTop/skyMid/skyHorizon`), astros (`sun/sunGlow/star`), nubes,
  montañas (`farL/farD`, `midL/midD`, `nearL/nearD`), valle (`valleyL/valleyD`), costa
  (`costaL/costaD`), arena (`sand/sandD`), mar (`sea/seaD/seaHi`), nieve (`snow/snowD`), agua,
  flora (`floraL/floraD`), tronco, roca (`rock/rockD`) y niebla (`fog`).
- El clima modifica la paleta con `apply_weather`; el bioma con `apply_biome`.
- **Regla**: no usar hex "a mano" en el dibujo; siempre leer de `get_palette(hour, weather, strength)`.

## Capas de color y perspectiva atmosférica

- Las capas lejanas usan tonos más azulados/desaturados (`alpha` < 1) para dar profundidad.
- La nieve se superpone en las cumbres según `snow_frac`.
- Las vetas de roca se dibujan mezcladas hacia el tono de la capa para no "saltar".

## Dithering y textura

- [`pixel.rs`](../crates/andes-core/src/pixel.rs) expone `radix_dither` (paridad exacta con JS); el
  resto de utilidades de pixel-art del original llegan con el rasterizador (`framebuffer.rs`).
- Texturas sutiles: vetas de roca, franjas de campos, línea de marea, dither por hash.

## Sprites

- Los sprites son **matrices de píxeles en código** (filas de caracteres + mapa de paleta) y se
  dibujan con `fill_rect` en coordenadas enteras.
- La flora es **procedural** (formas por bucles) en `flora.rs`; cada `draw_*` dibuja el hábito real
  de la planta (copa de paraguas de la araucaria, frondas de la palma, candelabro del copao, roseta
  de la nalca…) y el borde de las copas usa `hash1` para una silueta irregular determinista
  (D-046, D-048 JS).
- La fauna (`fauna.rs`) usa matrices de caracteres mapeadas a una clave de `get_palette` (responde a
  hora y clima) o a un hex literal (p. ej. el flamenco o el pingüino). Ver D-010 (JS).
- Los **momentos** (`moments.rs`: Leo Rey, Kung Leo) usan el mismo enfoque de matriz y una **fuente
  3×5** propia (`draw_text`) para el destello "MORTAL KUMBIA".

### Guía de estilo para sprites nuevos

- **Silueta primero:** la forma debe leerse como la especie real antes que el color. Cada especie
  declara `wiki` (Wikipedia) y `porte` (silueta) en su registro; el dibujo debe ser **fiel al
  `porte`** ([18 · Referencias](18-referencias.md), D-024).
- **Tamaño por especie:** escala relativa; la fauna con su matriz (ver [06 · Fauna](06-fauna.md)) y
  la flora con un **multiplicador `height`** sobre el rango de la capa (ver [05 · Flora](05-flora.md)).
- **Anclaje**: `ground`/pies o `center` (voladores).
- **Contorno:** 1 px de un tono oscuro para despegarlo del fondo.
- **Luz:** un tono claro en el lomo/pecho; evita más de 3–4 tonos por sprite. La **flora** recibe
  además una dirección de luz (`light`, −1…+1) derivada de la posición del sol/luna, y aclara el
  lado iluminado de cada copa (D-023).
- **Paleta:** cada carácter debe existir en `palette` (clave de `get_palette` o `#hex`).
- **Frames:** 2 bastan; el vaivén lateral es senoidal sobre `t_sec` (determinista).

### Galería de diagnóstico

Para inspeccionar una especie **aislada** (sin el paisaje) existe la galería
([`gallery.rs`](../crates/andes-core/src/gallery.rs)):

```bash
cargo run -p andes-native --bin gallery -- [hour] [weather] [season] [scale]
# genera gallery-flora.png y gallery-fauna.png en el directorio actual
```

- **Flora**: una celda por entrada de `FLORA`, con **tres tallas** por porte (`FLORA_SIZES`).
- **Fauna**: una celda por especie, con **todos los frames** y su versión **volteada**.
- Fondo a cuadros y rótulo (fuente 3×5 común en [`font.rs`](../crates/andes-core/src/font.rs));
  el binario la escala por **vecino más cercano** (×4 por defecto) para juzgar píxeles (D-022).
- La composición es **pura** en `andes-core` (sin I/O) y tiene tests; el binario solo exporta.


## Animación

- Animaciones por funciones del tiempo (`t_sec`): oleaje del mar, brillo del agua, bamboleo de la
  flora, penachos de volcán, parpadeo de estrellas, partículas de clima.
- Los animales usan **frames discretos** (2) con *frame rate* fijo por especie.

## Comparación con el original

`./scripts/pixel-check.sh` (Chrome headless) sigue disponible como **diagnóstico manual** contra el
render JS, pero ya no es un criterio: Rust es pixel-art **sin antialias** (D-016), así que las
diferencias en los bordes de nube son esperadas. El criterio propio son snapshots PNG
(`cargo run -p andes-native --bin infinite-andes -- ...`). Ver [11 · Roadmap](11-roadmap.md).

## Pendiente

- Portar a `pixel.rs` las utilidades restantes si el rasterizador las necesita.
- Más variedad de siluetas de nube (hoy son la misma forma a distintas escalas).
- **Reescribir las siluetas de flora** (hoy muchas comparten la misma copa) y afinar los sprites de
  fauna (contorno, luz, frames), iterando con la galería de diagnóstico (D-022).
