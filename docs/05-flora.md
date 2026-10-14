# 05 · Flora

> Estado: estable · Actualizado: 2026-10-12

Implementado en [`crates/andes-core/src/flora.rs`](../crates/andes-core/src/flora.rs) (registro,
dibujo y colocación) usando la paleta de [`palette.rs`](../crates/andes-core/src/palette.rs). La flora
se dibuja por capa con `place_flora(fb, layer, …)`; cada capa define en `LAYERS` su `FloraCfg`
(`chunk_w`, `min_size`, `max_size`, `min_chance`, `max_per`; lo espacial) y el registro `FLORA`
define las especies y sus `zones` (los pools por bioma y capa). Los **spawns** tienen paridad exacta
con JS ([11 · Roadmap](11-roadmap.md)).

El **dibujo** de cada tipo vive en funciones `draw_*` (el equivalente a los `DRAWERS`), con silueta,
escala y contorno oscuro de 1 px (D-056 JS).

## Especies representadas (actualmente)

<!-- BEGIN:flora-tabla (generado con `cargo run -p andes-core --bin gen-species`) -->
| Nombre común | Nombre científico | Endémica | Tipo en código | Zona/capa | Altura real | Notas |
|--------------|-------------------|----------|----------------|-----------|-------------|-------|
| Araucaria / Pehuén | *Araucaria araucana* | Sí (Chile/Argentina) | `araucaria` | Centro, Sur (precordillera, valle, costa) | 30–40 m | Árbol emblema; silueta de paraguas; la copa se bambolea. |
| Lenga / Ñire | *Nothofagus pumilio / N. antarctica* | No (Patagonia) | `lenga` | Centro, Sur, Patagonia, Austral (costa, precordillera, valle) | 15–25 m | Caducifolio: pierde hojas en otoño, queda desnudo en invierno y brota en primavera. |
| Copihue | *Lapageria rosea* | Sí (Chile) | `copihue` | Centro, Sur (costa, valle) | trepadora (hasta ~10 m) | Enredadera y flor nacional; campanas rojas en primavera/verano. |
| Copao / Cactus columnar | *Eulychnia spp.* | No | `cactus` | Altiplano, Norte (precordillera, valle, costa) | 3–7 m | Columna con brazos y espinas. |
| Alerce / Lahual | *Fitzroya cupressoides* | Sí (Chile/Argentina) | `alerce` | Sur, Austral (precordillera, costa) | 40–45 m | Conífera alta y estrecha; en peligro. |
| Nalca / Pangue | *Gunnera tinctoria* | Sí (Chile/Argentina) | `nalca` | Sur, Austral (valle, costa) | 1,5–3 m (hojas) | Hojas gigantes junto al agua. |
| Colihue / Quila | *Chusquea spp.* | No | `colihue` | Sur, Patagonia, Austral (valle, costa) | 3–6 m | Cañaverales (bambú nativo). |
| Palma chilena | *Jubaea chilensis* | Sí (Chile) | `palma` | Centro (valle, costa) | 15–20 m | Tronco esbelto y frondas; en peligro. |
| Coihue | *Nothofagus dombeyi* | No (Patagonia) | `coihue` | Centro, Sur, Patagonia, Austral (valle, costa, precordillera) | 35–45 m | Copa ancha y redondeada; tronco recto. |
| Roble | *Nothofagus obliqua* | Sí (Chile/Argentina) | `roble` | Centro, Sur (valle, costa) | 25–35 m | Caducifolio; copa estrecha y erguida; pierde hojas en otoño. |
| Michay | *Berberis darwinii* | No (Patagonia) | `michay` | Centro, Sur, Patagonia (valle, costa, precordillera) | 1–3 m | Arbusto espinoso; flores naranjas en primavera/verano. |
| Chaura | *Gaultheria mucronata* | No (Patagonia) | `chaura` | Centro, Sur, Patagonia, Austral (costa, valle) | 0,5–1,5 m | Arbusto achaparrado con bayas blanco-rosadas; perenne. |
| Quillay | *Quillaja saponaria* | Sí (Chile) | `quillay` | Centro, Sur (precordillera, valle, costa) | 10–15 m | Esclerófilo; flores blancas en primavera/verano. |
| Mañío | *Podocarpus spp.* | No (Patagonia) | `manio` | Sur, Patagonia, Austral (valle, costa, precordillera) | 10–20 m | Conífera austral oscura y estrecha; perenne. |
| Canelo | *Drimys winteri* | No (Chile/Argentina) | `canelo` | Sur, Patagonia, Austral (valle, costa, precordillera) | 15–20 m | Siempreverde de copa densa; flor blanca; árbol sagrado mapuche. |
| Arrayán | *Luma apiculata* | No (Chile/Argentina) | `arrayan` | Sur, Patagonia, Austral (valle, costa) | 10–15 m | Tronco canela rojizo y copa menuda; flor blanca. |
| Notro / Ciruelillo | *Embothrium coccineum* | No (Chile/Argentina) | `notro` | Sur, Patagonia, Austral (valle, costa) | 5–10 m | Ramilletes de flores rojas. |
| Arbusto genérico | — | — | `bush` | Altiplano, Norte, Centro, Sur, Patagonia, Austral (precordillera, valle, costa) | — | Bulto verde redondeado. |
| Cultivos / campos | — | — | `crop` | Norte, Centro (valle) | — | Hileras de cultivo; refuerza el valle agrícola. |
| Pasto / duna | — | — | `grass` | Altiplano, Norte, Centro, Sur, Patagonia, Austral (precordillera, valle, costa, playa) | — | Matas pequeñas de pasto. |
| Rocas | — | — | `rock` | Altiplano, Norte, Centro, Sur, Patagonia, Austral (precordillera, valle, costa, playa) | — | Pedreros sueltos. |
| Espino | *Acacia caven* | No (Chile/Argentina) | `espino` | Norte, Centro (valle, costa) | 2–4 m | Matorral espinoso del valle y la costa en clima seco; flores amarillas. |
| Litre | *Lithraea caustica* | Sí (Chile) | `litre` | Centro (valle, costa) | 3–8 m | Esclerófilo de copa redonda; savia irritante. |
| Boldo | *Peumus boldus* | Sí (Chile) | `boldo` | Centro (precordillera, costa) | 5–12 m | Copa redondeada y aromática; hojas medicinales. |
| Peumo | *Cryptocarya alba* | Sí (Chile) | `peumo` | Centro, Sur (valle, costa) | 10–18 m | Siempreverde denso del bosque esclerófilo. |
| Maitén | *Maytenus boaria* | No (Chile/Argentina) | `maiten` | Centro, Sur (valle) | 6–15 m | Ramas colgantes que le dan un aire llorón. |
| Ciprés de la cordillera | *Austrocedrus chilensis* | No (Chile/Argentina) | `cipres` | Centro, Sur (precordillera) | 10–20 m | Conífera columnar de la precordillera. |
| Puya | *Puya chilensis* | Sí (Chile) | `puya` | Altiplano, Norte (precordillera, costa) | 1–3 m (con el tallo) | Roseta espinosa del Norte y el altiplano. |
| Llareta | *Azorella compacta* | No (Chile/Perú/Bolivia) | `llareta` | Altiplano (precordillera) | 0,3–0,5 m | Cojín compacto de la puna; crecimiento lentísimo. |
| Amancay | *Alstroemeria aurea* | No (Chile/Argentina) | `amancay` | Sur, Patagonia (valle, costa) | 0,5–1 m | Flores amarillas de los claros del sur. |
| Junquillo | *Juncus spp.* | No | `junquillo` | Centro, Sur, Patagonia, Austral (valle, costa) | 0,5–1 m | Juncos de humedales y bordes de agua. |
| Flor del desierto | — | — | `flower` | — | — | Parche del desierto florido; entra solo con la floración. |
<!-- END:flora-tabla -->

> La vegetación no crece dentro del cauce: `place_flora` omite las columnas con
> `river_influence > 0.25` y usa `bank_height` para sentarse en el banco.

## Estaciones

Además del tinte de `apply_season`, las especies caducifolias tienen **variantes estructurales por
estación** (D-031 JS, [15 · Estaciones](15-estaciones.md)); `place_flora` recibe el índice de
estación y solo cambia el **dibujo** (el spawn es idéntico):

- **Lenga y roble** (caducifolios): en **otoño** pierden hojas (huecos deterministas en la copa,
  `hash1` por planta); en **invierno** quedan **desnudos** (solo ramas); en **primavera** muestran
  **brotes** claros en la punta.
- **Copihue y michay**: las campanas/flores aparecen en **primavera y verano**.
- **Quillay**: las motas blancas de flor aparecen en **primavera y verano**.
- La **chaura** es perenne y conserva sus bayas todo el año.

## Biomas

El bioma ([`biomes.rs`](../crates/andes-core/src/biomes.rs)) **pondera** los `types` de cada capa; el
pool efectivo se construye con `biome_flora_pool`. El **norte árido no tiene araucaria**; el **sur**
suma alerce, nalca, colihue, coihue, roble, copihue y michay; el **centro** aporta el quillay
esclerófilo y el **sur/Patagonia** el mañío; el **austral** (fiordos) suma canelo, arrayán y notro.
El tipo `flower` solo aparece con la floración del norte (`bloom_at`); se dibuja como manto amplio de
tallos con corola (`draw_flower`) que se **reparte hacia dentro de la banda visible** de la capa.

## Altura relativa (porte por especie)

El tamaño de cada planta no es fijo por tipo: cada capa define un rango (`min_size`–`max_size` en
`terrain.rs`) y **cada especie lo multiplica por su `height`** (D-049 JS). Así la perspectiva por
parallax se conserva y, dentro de una capa, los árboles duplican a los arbustos.

| Tipo | `height` | Altura real | Tipo | `height` | Altura real |
|------|----------|-------------|------|----------|-------------|
| Araucaria | ×1.9 | 30–40 m | Quillay | ×1.0 | 10–15 m |
| Alerce | ×1.9 | 40–45 m | Arrayán | ×1.0 | 10–15 m |
| Coihue | ×1.7 | 35–45 m | Notro | ×0.9 | 5–10 m |
| Roble | ×1.5 | 25–35 m | Nalca | ×0.9 | 1,5–3 m (hojas) |
| Palma | ×1.4 | 15–20 m | Colihue | ×1.1 | 3–6 m |
| Mañío | ×1.4 | 10–20 m | Copihue | ×1.1 | trepadora |
| Lenga | ×1.3 | 15–25 m | Copao | ×1.2 | 3–7 m |
| Canelo | ×1.3 | 15–20 m | Michay | ×0.7 | 1–3 m |
| Chaura | ×0.6 | 0,5–1,5 m | Arbusto genérico | ×0.7 | — |
| Cultivo | ×0.8 | — | Pasto / duna | ×0.5 | — |
| Rocas | ×0.7 | — | Flor del desierto | ×1.0 | — |

En `flora_spawns` el `rng` se consume igual que antes (solo cambia el `size`, no la posición ni el
tipo), de modo que el paisaje sigue siendo reproducible.

## Silueta y hábito

Las funciones `draw_*` priorizan el **hábito real** de cada planta sobre el color (D-048 JS). La
silueta es lo primero: debe reconocerse **sin color**. Cada árbol/arbusto tiene porte propio y ya no
comparte la misma "copa genérica" (D-023):

- **Araucaria**: tronco alto y recto con **copa de paraguas** y ramas punzantes por pisos.
- **Alerce**: tronco recto visible y copa cónica estrecha con ramas colgantes.
- **Lenga**: copa **piramidal** (angosta arriba, ancha abajo), distinta del resto de Nothofagus.
- **Coihue**: **copa ancha de tres lóbulos** sobre tronco recto con **horqueta**.
- **Roble**: copa **estrecha y erguida** en huso; caducifolio (desnudo en invierno).
- **Mañío**: **conífera austral oscura** con ramas colgantes (más estrecha que el alerce).
- **Ciprés**: **columna cónica** de la precordillera.
- **Canelo**: copa **densa y alta** con **ápice puntiagudo** y flores blancas.
- **Quillay**: **copa ancha y lobulada** sobre horqueta; flores blancas.
- **Arrayán**: **tronco canela** alto y visible y copa menuda; flor blanca.
- **Notro**: porte esbelto con **racimos de flores escarlata**.
- **Litre, boldo y peumo**: copas redondas diferenciadas por textura y tono (moteado, claro,
  siempreverde oscuro).
- **Espino**: **matorral** de ramas en abanico con **espinas** y flores amarillas (no un árbol).
- **Michay y chaura**: arbustos bajos y compactos con flores naranjas / bayas rosadas.
- **Maitén**: copa con **ramas lloronas** que caen.
- **Copihue**: **enredadera** de tallo sinuoso con **campanas rojas colgantes**.
- **Palma chilena**: tronco con anillos y **frondas pinnadas que arquean y caen** (copa redondeada).
- **Copao / cactus**: **columna gruesa con costillas**, 1–2 **brazos en candelabro** que suben,
  espinas y **flor apical**.
- **Nalca**: roseta de **hojas orbiculares gigantes** de borde dentado sobre pecíolos gruesos.
- **Colihue**: cañas con **nudos** y **penacho** de hojas en la punta.
- **Puya**: **roseta espinosa** basal con **varas florales altas** en candelabro.
- **Llareta**: **cojín hemisférico** verde-amarillento con bultos de rosetas.
- **Amancay**: varillas con **umbela** de flores amarillas; **junquillo**: macolla de juncos.
- **Bush / grass / crop / rock / flower**: genéricos (mata, briznas, hileras, piedras, flor del
  desierto).

Los dibujos se apoyan en primitivas compartidas en `flora.rs` —`line` (troncos y ramas), `crown_blob`
(copa redondeada irregular), `crown_cone` (coníferas/portes piramidales), `frond` (fronda arqueada) y
`rosette` (estrella de hojas)— y **todas** reciben una **dirección de luz** (`light`, −1 izquierda …
+1 derecha) derivada de la posición del sol/luna en pantalla, de modo que el lado iluminado se aclara
y el sombrío se oscurece ([08 · Arte pixel](08-arte-pixel.md), D-023). El borde irregular sigue
saliendo de `hash1` (determinista).

Para ver cada tipo aislado (tres tallas, sobre fondo a cuadros) usa la **galería de diagnóstico**:
`cargo run -p andes-native --bin gallery` ([08 · Arte pixel](08-arte-pixel.md), D-022).

## Referencias y porte

Cada especie real de `flora_data.rs` declara `wiki` (artículo de **Wikipedia**, referencia estable) y
`porte` (descripción fiel de la **silueta/hábito**, que es la especificación de dibujo). Las fichas de
[`especies/`](especies/README.md) muestran ambas en las secciones *Descripción / porte* y
*Referencia*. La silueta manda sobre el color: el dibujo debe reconocerse sin color y coincidir con el
`porte` ([18 · Referencias](18-referencias.md), D-024).

La **araucaria** ilustra el criterio: su `porte` (ver la ficha) describe tronco recto que ramifica a
varios metros y copa en **pisos/verticilos** de ramas horizontales tipo pagoda, no una copa
redondeada.

## Fichas de especies

Cada especie tiene su ficha en [`especies/`](especies/README.md), generada con
`cargo run -p andes-core --bin gen-species` (nombre científico, endemismo, zona, notas, porte y
referencia).

## Cómo añadir una especie

1. Añade la especie al registro `FLORA` en `flora.rs` (`kind`, `height`, `zones`; `zones[bioma][capa]`
   es su peso en el pool) y su metadata en `flora_data.rs` (`common`, `sci`, `endemism`, `height_m`,
   `wiki`, `porte`, `notes`, `refs`).
2. Añade su caso a `draw_plant` (misma clave que el tipo) dibujando el `porte` y, si aplica, el color
   a `palette.rs`.
3. Ejecuta `cargo run -p andes-core --bin gen-species` para regenerar esta tabla, la de
   [06 · Fauna](06-fauna.md) y su ficha en [`especies/`](especies/README.md).
4. Documenta en [`12-decisiones.md`](12-decisiones.md) si cambia el diseño del port.
