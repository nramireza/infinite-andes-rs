# 13 · Glosario

> Estado: estable · Actualizado: 2026-10-12

## Términos técnicos

| Término | Significado |
|---------|-------------|
| **Parallax** | Desplazamiento relativo de capas a distinta velocidad para dar profundidad |
| **fBm** (fractal Brownian motion) | Suma de octavas de ruido; genera terreno suave y detallado |
| **Ruido ridged** | Variante de ruido con crestas afiladas; ideal para montañas |
| **PRNG sembrado** | Generador pseudoaleatorio reproducible a partir de una semilla (`Mulberry32`) |
| **`hash1`** / **`hash_int`** | Hash determinista de un entero a [0,1) / a entero de 32 bits |
| **Chunk** | Bloque de mundo (p. ej. 56 px) usado para decidir flora/fauna por hash |
| **Pixel art** | Estética de píxeles de baja resolución, aquí 270 px de alto (32:9 = 960×270) escalado |
| **`Framebuffer`** | Lienzo RGBA8 propio que sustituye al Canvas del original |
| **`SEA_Y`** | Línea del mar en la resolución interna (`terrain.rs`, valor 242) |
| **`ridge_height`** | Altura de la silueta de una capa en un `world_x` dado (incluye tallado de río) |
| **`bank_height`** | Altura de la capa sin el tallado del río (nivel del banco) |
| **Dithering** | Mezcla de píxeles para simular tonos intermedios (`radix_dither`) |
| **Paridad** | Fidelidad al proyecto JS: exacta (valores) o de píxel (render) |
| **Crate** | Unidad de compilación de Rust (`andes-core`, `andes-native`, …) |
| **ABI C** | Interfaz de funciones `extern "C"` que expone `andes-web` al reproductor JS |

## Nomenclatura de módulos

| JS (`src/`) | Rust (`crates/andes-core/src/`) |
|-------------|---------------------------------|
| `rng.js` | `rng.rs` |
| `noise.js` | `noise.rs` |
| `pixel.js` | `pixel.rs` (+ `framebuffer.rs`) |
| `palette.js` | `palette.rs` (+ `palette_data.rs` generado) |
| `terrain.js` | `terrain.rs` |
| `flora.js` | `flora.rs` |
| `fauna.js` | `fauna.rs` (+ `fauna_data.rs` generado) |
| `sky.js` / `weather.js` | `sky.rs` / `weather.rs` |
| `seasons.js` / `biomes.js` | `seasons.rs` / `biomes.rs` |
| `clock.js` / `sun.js` | `clock.rs` / `sun.rs` |
| `scene.js` / `views.js` | `scene.rs` / `views.rs` |

## Flora (nombres comunes y científicos)

| Común | Científico | Nota |
|-------|-----------|------|
| Araucaria / Pehuén | *Araucaria araucana* | Endémica; árbol emblema |
| Lenga | *Nothofagus pumilio* | Bosque caducifolio andino |
| Ñire | *Nothofagus antarctica* | Pariente de la lenga, más austral |
| Coihue | *Nothofagus dombeyi* | Bosque de la Costa |
| Copihue | *Lapageria rosea* | Flor nacional de Chile |
| Nalca / Pangue | *Gunnera tinctoria* | Hojas gigantes ribereñas |
| Palma chilena | *Jubaea chilensis* | Endémica, en peligro |
| Alerce / Lahual | *Fitzroya cupressoides* | Longevo |
| Copao | *Eulychnia* spp. | Cactus columnar |
| Colihue / Quila | *Chusquea* spp. | Cañaveral (bambú nativo) |
| Quillay | *Quillaja saponaria* | Esclerófilo endémico de Chile central |
| Mañío | *Podocarpus* spp. | Conífera austral de bosque húmedo |
| Canelo | *Drimys winteri* | Árbol sagrado mapuche |
| Arrayán | *Luma apiculata* | Tronco canela rojizo |
| Notro | *Embothrium coccineum* | Ramilletes de flores rojas |

## Fauna (nombres comunes y científicos)

| Común | Científico | Nota |
|-------|-----------|------|
| Cóndor | *Vultur gryphus* | Ave símbolo de los Andes |
| Pudú | *Pudu puda* | Uno de los ciervos más pequeños |
| Güiña | *Leopardus guigna* | Felino nativo, esquivo |
| Huemul | *Hippocamelus bisulcus* | Ciervo andino, en peligro |
| Puma | *Puma concolor* | Depredador tope |
| Zorro culpeo | *Lycalopex culpaeus* | Zorro andino |
| Zorro chilla | *Lycalopex griseus* | Zorro gris |
| Guanaco | *Lama guanicoe* | Camélido silvestre |
| Vicuña | *Vicugna vicugna* | Camélido de altiplano |
| Chingue | *Conepatus chinga* | Mofeta andina |
| Monito del monte | *Dromiciops gliroides* | Marsupial, "fósil viviente" |
| Rana de Darwin | *Rhinoderma darwinii* | Anfibio endémico |
| Choroy | *Enicognathus leptorhynchus* | Loro endémico de Chile |
| Cachaña | *Enicognathus ferrugineus* | Loro austral |
| Flamenco chileno | *Phoenicopterus chilensis* | Ave de lagunas |
| Chungungo | *Lontra felina* | Nutria marina |
| Chinchilla de cola larga | *Chinchilla lanigera* | Roedor endémico |
| Pingüino de Humboldt | *Spheniscus humboldti* | Pingüino del Pacífico sur |
| Choique | *Rhea pennata* | Ñandú de la estepa |
| Chucao | *Scelorchilus rubecula* | Ave de sotobosque |
| Huillín | *Lontra provocax* | Nutria de río del sur |

> Verificar nombres y estatus de endemismo/conservación contra el proyecto original
> ([05 · Flora](05-flora.md) y [06 · Fauna](06-fauna.md)).
