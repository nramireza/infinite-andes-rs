# 06 · Fauna

> Estado: estable · Actualizado: 2026-10-12

Implementado en [`crates/andes-core/src/fauna.rs`](../crates/andes-core/src/fauna.rs) (comportamiento,
spawn y dibujo) y [`fauna_data.rs`](../crates/andes-core/src/fauna_data.rs) (**registro generado**
desde `fauna.js` con `scripts/gen-parity.mjs`). Los sprites son **matrices de píxeles en código**,
con spawn determinista por *chunk* y actividad según la hora. Los **spawns** tienen paridad exacta
con JS ([11 · Roadmap](11-roadmap.md)).

## Implementado

<!-- BEGIN:fauna-implementado (generado con `cargo run -p andes-core --bin gen-species`) -->
| Tipo en código | Nombre común | Capa(s) | Movimiento | Actividad |
|----------------|--------------|---------|------------|-----------|
| `condor` | Cóndor | Andes | `fly` | día |
| `huemul` | Huemul | Precordillera, Valle | `walk` | día |
| `pudu` | Pudú | Valle, Costa, Precordillera | `walk` | crepúsculo |
| `guina` | Güiña / Kodkod | Valle, Costa | `hop` | noche |
| `puma` | Puma | Precordillera, Valle | `walk` | crepúsculo |
| `culpeo` | Zorro culpeo | Precordillera, Valle, Costa | `walk` | día |
| `chilla` | Zorro chilla | Costa, Playa, Valle | `walk` | crepúsculo |
| `guanaco` | Guanaco | Precordillera, Valle | `walk` | día |
| `vicuna` | Vicuña | Precordillera, Valle | `walk` | día |
| `chingue` | Chingue | Valle, Costa | `walk` | noche |
| `monito` | Monito del monte | Costa | `hop` | noche |
| `chinchilla` | Chinchilla de cola larga | Andes, Precordillera | `hop` | noche |
| `choroy` | Choroy | Costa | `flock` | día |
| `cachana` | Cachaña | Costa | `flock` | día |
| `flamenco` | Flamenco chileno | Playa | `walk` | día |
| `chungungo` | Chungungo | Mar | `swim` | día |
| `pinguino` | Pingüino de Humboldt | Mar | `swim` | día |
| `rana` | Rana de Darwin | Costa, Valle (borde del cauce) | `sit` | día |
| `choique` | Choique / Ñandú petizo | Precordillera, Valle | `walk` | día |
| `chucao` | Chucao | Costa | `hop` | día |
| `huillin` | Huillín | Valle, Costa (borde del cauce) | `swim` | día |
| `lobo-marino` | Lobo marino común | Mar | `swim` | día |
| `delfin` | Delfín chileno | Mar | `swim` | día |
| `gaviota` | Gaviota dominicana | Playa, Mar | `fly` | día |
| `albatros` | Albatros de ceja negra | Mar | `fly` | día |
| `pelicano` | Pelícano | Mar, Playa | `fly` | día |
<!-- END:fauna-implementado -->

### Especificación de sprites

Anchura y frames por tipo; el mapa de paleta mezcla claves de `get_palette` con algún **hex literal**
(flamenco, pingüino) para tonos que la paleta no cubre.

| Tipo | Tamaño (px) | Frames | Anclaje | Silueta / color |
|------|-------------|--------|---------|-----------------|
| `condor` | 16 × 7 | 2 | centro | alas anchas con primarias "digitadas", collar `snow`, cabeza chica |
| `huemul` | 10 × 9 | 2 | pies | cuerpo robusto, cuello corto, astas bifurcadas (`trunk`/`sandD`) |
| `pudu` | 8 × 7 | 2 | pies | ciervo diminuto, hocico corto |
| `guina` | 9 × 5 | 2 | pies | felino pequeño y redondo, vientre `sandD` |
| `puma` | 14 × 6 | 2 | pies | perfil largo, cola tendida, vientre `sand` |
| `culpeo` | 11 × 6 | 2 | pies | zorro de cola poblada `#c96a3a` |
| `chilla` | 9 × 6 | 2 | pies | zorro gris, cola más corta |
| `guanaco` | 10 × 14 | 2 | pies | cuello y patas largos (`sandD`/`sand`) |
| `vicuna` | 8 × 12 | 2 | pies | camélido esbelto y erguido |
| `chingue` | 10 × 6 | 2 | pies | rayas `snow`, cola blanca |
| `monito` | 9 × 6 | 2 | pies | marsupial diminuto |
| `chinchilla` | 9 × 6 | 2 | pies | roedor redondo, orejas grandes, cola `snowD` |
| `choroy` / `cachana` | 12 × 6 | 2 | centro | loros con cola larga, verde/rojizo |
| `flamenco` | 7 × 15 | 2 | pies | cuello en S, patas finas, pico curvo `#f2a0b8` |
| `pinguino` | 7 × 10 | 2 | pies | erguido, aletas, pico naranja |
| `chungungo` | 12 × 5 | 2 | pies | mustélido alargado nadando |
| `huillin` | 12 × 5 | 2 | pies | nutria de río alargada |
| `choique` | 9 × 15 | 2 | pies | ñandú: cuello y patas largos |
| `chucao` | 8 × 7 | 2 | pies | pecho rojizo `#c0502a`, cola corta |
| `rana` | 6 × 4 | 2 | pies | triangular, hocico puntiagudo |

- Los caracteres de la matriz se mapean a **claves de `get_palette(hour, weather)`** (o a un hex
  literal), así que la fauna responde a la hora y al clima sin reescribir colores. El mapa por
  especie está en `SpeciesData::palette`.
- El dibujo es `fill_rect` de 1 px en **coordenadas enteras**, con volteo horizontal según la
  dirección; no depende de ningún canvas del sistema (testeable en `cargo test`).
- Animación discreta a *frame rate* fijo por especie (`fps`), sin aleatoriedad no determinista.

### Comportamiento

- **Spawn por chunk**: para cada bloque de `chunk_w` px de la capa, un `hash_int` decide si hay
  fauna, de qué especie (ponderada por la lista) y en qué posición. La identidad del candidato
  **no depende de la hora**; la hora solo **filtra** si la especie está activa (`is_active`), de modo
  que volver a la misma posición y hora reproduce los mismos animales (D-010 JS).
- **Movimiento**: `fly` y `flock` trazan una trayectoria senoidal en el cielo de su capa; las
  bandadas (`flock`) dibujan dos compañeros en formación; `walk`/`hop`/`swim` siguen `bank_height` y
  evitan el cauce (`river_influence > 0.25`). `swim` añade un vaivén sobre el agua; `sit` (rana de
  Darwin) queda **estática**. Las especies ligadas al cauce (`SpeciesData::river`: rana y huillín) se
  colocan aparte, en el **borde del río** de valle y Costa, con `chance`, `offset` y `seed` propios.
- **Despawn**: al salir de pantalla se descarta; el hash lo regenera idéntico al volver.
- **Densidad y rareza**: cada especie declara una clase (`rarity`) y el sorteo por chunk es
  **ponderado** (`pick_weighted`), de modo que lo abundante aparece más seguido (D-015 JS).
- **Mar y playa**: la fauna marina (`swim`) se dibuja **sobre el mar**; la de playa (chilla,
  flamenco), sobre la arena.

### Densidad según rareza real

`chance` por capa fija cuánta fauna hay en el ambiente; la `rarity` de cada especie fija **cuál** (y
con qué frecuencia relativa). Pesos en `RARITY_WEIGHT` (`fauna_data.rs`).

| Especie | UICN global | Chile | Clase | Peso |
|---------|-------------|-------|-------|------|
<!-- BEGIN:fauna-densidad (generado con `cargo run -p andes-core --bin gen-species`) -->
| Chingue | LC | común | abundante | 6 |
| Zorro chilla | LC | común | abundante | 6 |
| Zorro culpeo | LC | común | abundante | 6 |
| Cachaña | LC | común | común | 3 |
| Choroy | LC | endémica, estable | común | 3 |
| Chucao | LC | común, sotobosque | común | 3 |
| Cóndor | NT | amplia | común | 3 |
| Gaviota dominicana | LC | costera | común | 3 |
| Güiña / Kodkod | LC | común, esquiva | común | 3 |
| Pingüino de Humboldt | VU | colonias | común | 3 |
| Albatros de ceja negra | LC | oceánico | poco-común | 1.2 |
| Choique / Ñandú petizo | NT | decreciente | poco-común | 1.2 |
| Flamenco chileno | NT | localizado | poco-común | 1.2 |
| Guanaco | LC | VU centro/norte | poco-común | 1.2 |
| Lobo marino común | LC | costero | poco-común | 1.2 |
| Monito del monte | NT | localizado | poco-común | 1.2 |
| Pelícano | NT | costero | poco-común | 1.2 |
| Pudú | NT | decreciente | poco-común | 1.2 |
| Puma | LC (NT nacional) | NT nacional | poco-común | 1.2 |
| Vicuña | LC | localizada | poco-común | 1.2 |
| Chungungo | EN | costero | rara | 0.5 |
| Delfín chileno | NT | endémico | rara | 0.5 |
| Huillín | EN | ríos del sur | rara | 0.5 |
| Chinchilla de cola larga | EN | casi extinta | muy-rara | 0.12 |
| Huemul | EN | ~1.000 ind. | muy-rara | 0.12 |
| Rana de Darwin | EN | endémica, en declive | muy-rara | 0.12 |
<!-- END:fauna-densidad -->

- **Chance por capa**: andes 0.25, precordillera 0.3, valle 0.5, costa 0.55, playa 0.3, mar 0.35.
- Fuentes: listados de la UICN (versiones 2016–2025), clasificación nacional del MMA y
  BirdLife/GBIF. Es una **aproximación artística** coherente con la del proyecto original.

### Biomas y floración

El bioma ([`biomes.rs`](../crates/andes-core/src/biomes.rs)) **pondera** las especies de cada capa con
`biome_fauna_pool`: el altiplano trae vicuña, guanaco, chinchilla y flamenco; el norte, guanaco,
vicuña, culpeo y flamenco; el sur, pudú, monito, choroy, huemul, choique, chucao, huillín y la rana
de Darwin (junto al cauce); la Patagonia, guanaco, choique, puma y huemul; el austral suma huillín y
chungungo. Durante el **desierto florido** (`bloom_at`) sube la densidad (`chance × (1 + 0.6·bloom)`)
y se refuerzan aves y zorros (`BLOOM_FAUNA`).

## Especies objetivo (endémicas / nativas de Chile)

<!-- BEGIN:fauna-objetivo (generado con `cargo run -p andes-core --bin gen-species`) -->
| Nombre común | Nombre científico | Endémica | Capa/zona | Actividad | Comportamiento |
|--------------|-------------------|----------|-----------|-----------|----------------|
| Cóndor | *Vultur gryphus* | No (Andes) | Altiplano, Norte, Centro, Sur, Patagonia, Austral (andes) | día | Ave símbolo de los Andes; planea lento con aleteo ocasional. |
| Huemul | *Hippocamelus bisulcus* | Sí (Chile/Argentina) | Centro, Sur, Patagonia, Austral (precordillera, valle) | día | Ciervo andino en peligro; pastorea y deja huellas. |
| Pudú | *Pudu puda* | Sí (Chile/Argentina) | Centro, Sur, Patagonia, Austral (valle, costa, precordillera) | crepúsculo | Uno de los ciervos más pequeños; camina entre arbustos. |
| Güiña / Kodkod | *Leopardus guigna* | Sí (Chile/Argentina) | Centro, Sur, Patagonia, Austral (valle, costa) | noche | Felino esquivo; se desplaza a saltos de noche. |
| Puma | *Puma concolor* | No | Sur, Patagonia, Austral (precordillera, valle) | crepúsculo | Depredador tope; raro y solitario; deja huellas. |
| Zorro culpeo | *Lycalopex culpaeus* | No (Sudamérica) | Altiplano, Norte, Centro, Sur, Patagonia (precordillera, valle, costa) | día | Zorro andino de cola rojiza; trota de día. |
| Zorro chilla | *Lycalopex griseus* | No | Altiplano, Norte, Centro, Sur, Patagonia, Austral (costa, playa, valle) | crepúsculo | Zorro gris, más pequeño; activo al crepúsculo. |
| Guanaco | *Lama guanicoe* | No | Altiplano, Norte, Centro, Patagonia, Austral (precordillera, valle) | día | Camélido silvestre; tropillas en la estepa; deja huellas. |
| Vicuña | *Vicugna vicugna* | No | Altiplano, Norte, Centro (precordillera, valle) | día | Camélido del altiplano; grupos muy ágiles. |
| Chingue | *Conepatus chinga* | No | Altiplano, Norte, Centro, Sur (valle, costa) | noche | Mofeta de hocico al suelo; lento, de noche. |
| Monito del monte | *Dromiciops gliroides* | Sí (Chile/Argentina) | Centro, Sur (costa) | noche | Marsupial, fósil viviente; diminuto, en el colihue. |
| Chinchilla de cola larga | *Chinchilla lanigera* | Sí (Chile) | Altiplano, Norte, Centro, Sur, Patagonia, Austral (andes, precordillera) | noche | Roedor ágil de roqueríos andinos; casi extinto. |
| Choroy | *Enicognathus leptorhynchus* | Sí (Chile) | Centro, Sur (costa) | día | Loro endémico; bandada ruidosa del bosque de la Costa. |
| Cachaña | *Enicognathus ferrugineus* | No (Patagonia) | Centro, Sur (costa) | día | Loro austral; vuela en bandadas. |
| Flamenco chileno | *Phoenicopterus chilensis* | No | Altiplano, Norte, Centro, Sur, Patagonia, Austral (playa) | día | Filtra en lagunas y marismas costeras. |
| Chungungo | *Lontra felina* | Sí (Chile/Perú) | Altiplano, Norte, Centro, Sur, Patagonia, Austral (mar) | día | Nutria marina; nada y se asoma entre roqueríos. |
| Pingüino de Humboldt | *Spheniscus humboldti* | No | Altiplano, Norte, Centro, Sur, Patagonia, Austral (mar) | día | Pingüino del Pacífico sur; nada y emerge. |
| Rana de Darwin | *Rhinoderma darwinii* | Sí (Chile/Argentina) | Centro, Sur, Patagonia, Austral (costa, valle) | día | Anfibio endémico estático en el borde del cauce. |
| Choique / Ñandú petizo | *Rhea pennata* | No | Sur, Patagonia (precordillera, valle) | día | Ñandú de la estepa; camina y deja huellas. |
| Chucao | *Scelorchilus rubecula* | No (Chile/Argentina) | Sur, Patagonia, Austral (costa) | día | Ave de sotobosque; pecho rojizo. |
| Huillín | *Lontra provocax* | No (Chile/Argentina) | Centro, Sur, Patagonia, Austral (valle, costa) | día | Nutria de río del sur; nada en el borde del cauce. |
| Lobo marino común | *Otaria flavescens* | No | Altiplano, Norte, Centro, Sur, Patagonia, Austral (mar) | día | Macho con melena; se asolea en roqueríos y nada cerca de la costa. |
| Delfín chileno | *Cephalorhynchus eutropia* | Sí (Chile) | Altiplano, Norte, Centro, Sur, Patagonia, Austral (mar) | día | Delfín endémico, pequeño y sin pico; salta en grupos. |
| Gaviota dominicana | *Larus dominicanus* | No | Altiplano, Norte, Centro, Sur, Patagonia, Austral (playa, mar) | día | Ave costera abundante; planea sobre la playa y el mar. |
| Albatros de ceja negra | *Thalassarche melanophris* | No | Altiplano, Norte, Centro, Sur, Patagonia, Austral (mar) | día | Planea con las alas rígidas sobre el oleaje. |
| Pelícano | *Pelecanus thagus* | No (Chile/Perú) | Altiplano, Norte, Centro, Sur, Patagonia, Austral (mar, playa) | día | Vuela en fila bordeando la costa; pico largo. |
<!-- END:fauna-objetivo -->

> Los nombres científicos y la condición de endemismo son una aproximación: varias especies son
> "nativas" más que endémicas de Chile. El estado de conservación (UICN/MMA) se detalla en la tabla
> de densidad de arriba.

## Modelo de datos de una especie

Cada entrada de `SPECIES` (`fauna_data.rs`) es un **registro** con los campos de
comportamiento (`movement`, `active`, `speed`/`range`, `fps`, `rarity`), el sprite (`palette` y
`frames`), las **zonas** (`zones[bioma][capa]`, fuente única de los pools) y la colocación
(`river`, `chunk_ok`, `bloom`). La **metadata de identidad** (`common`, `sci`, `endemism`, `iucn`,
`notes`, `wiki`, `porte`, `refs`) es **fuente de este repositorio** (editable a mano) y la consume el
binario `gen-species`: `wiki` es el artículo de Wikipedia (referencia estable) y `porte` la
descripción fiel de la silueta que guía el sprite ([18 · Referencias](18-referencias.md), D-024). La
flora sigue el mismo esquema en `FLORA`. Ver [09 · Arquitectura](09-arquitectura.md).

## Cómo añadir una especie

La fauna se **genera** desde `fauna.js` del repo JS con `scripts/gen-parity.mjs`; para editarla se
toca el proyecto original y se regenera. El flujo en este repo:

```bash
# en el repo JS: editar src/fauna.js
TZ=UTC node scripts/gen-parity.mjs          # regenera fauna_data.rs (+ metadata)
cargo test                                  # valida paridad y sprites
cargo run -p andes-core --bin gen-species   # regenera docs/especies y las tablas de 05/06
```

## Pendientes

- Paridad de **init/astros** exacta y **spawns** exactos; queda refinar el **dibujo** (fuentes de
  diferencia de píxel).
- Afinar los sprites (contorno y luz normalizados, más frames, mayor tamaño) sobre la **galería de
  diagnóstico** (`cargo run -p andes-native --bin gallery`, D-022).
