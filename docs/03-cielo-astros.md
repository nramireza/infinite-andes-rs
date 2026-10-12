# 03 · Cielo y astros

> Estado: estable · Actualizado: 2026-10-12

Implementado en [`crates/andes-core/src/sky.rs`](../crates/andes-core/src/sky.rs) y la paleta en
[`crates/andes-core/src/palette.rs`](../crates/andes-core/src/palette.rs). La **inicialización y
los astros** tienen paridad exacta con JS ([11 · Roadmap](11-roadmap.md)).

## Gradiente del cielo

Gradiente vertical de tres tonos que cambia con la hora: `skyTop` → `skyMid` → `skyHorizon`
(`Framebuffer::vgradient`). El horizonte visual está a `H * 0.56` (~151 px). Los colores salen de las
claves de `palette.rs` interpoladas por hora del día.

## Orden del cielo

De atrás hacia adelante: **gradiente → Vía Láctea → estrellas → aurora → resplandor → sol/luna →
nubes → capas de terreno → clima**. Las nubes se pintan **después del astro** (lo tapan) y **antes
del terreno** (las montañas tapan a las nubes).

## Astros

El sol y la luna **nacen tras los Andes** (fondo), **salen de pantalla** a mediodía/medianoche,
y en el ocaso quedan **tras la cámara** (no se dibujan). Constantes en `sky.rs`:

- `RISE = 6.5`, `SET = 18.5` (ventana del sol; la noche es el resto).
- `Sky::celestial(hour, w, h, rise, set, moon)` devuelve un `Celestial` con la posición, radio,
  horizonte, visibilidad y fase.
- El cuerpo se dibuja **al fondo** (`Sky::draw_body`), justo tras el resplandor y **antes de las
  nubes**: el terreno lo oculta al alba y las nubes pueden taparlo.
- **Solo se dibuja al salir**: al ocaso no hay disco solar ni lunar visible.

### Fases de la luna

La luna tiene **fase real** (D-030 JS): `moon_phase_for_date`/`moon_phase_from_days`
([`clock.rs`](../crates/andes-core/src/clock.rs), ciclo sinódico `MOON_CYCLE = 29.530588`) dan la
fase en [0,1) — 0 nueva, 0.5 llena. Con el reloj real sale de la **fecha del equipo**; con el reloj
rápido, de los **días simulados** (`Scene::day_num`, avanza al envolver la hora). En `draw_body`:

- **Llena**: disco completo en `pal.sun` con cráteres.
- **Creciente/menguante**: disco oscuro (`shade(pal.sun, −0.55)`) con la porción iluminada según la
  fase — el **terminador** recorta por fila; creciente ilumina la derecha, menguante la izquierda.
  Los **cráteres** solo se dibujan sobre la cara iluminada.
- El **resplandor lunar** se atenúa con la fase (`illum`), y la luna llena **apaga las estrellas
  más débiles** (`draw` recibe `moon_dim`).

### Resplandor

`Sky::draw_glow` pinta un resplandor cálido/frío cuando el astro está cerca del horizonte, pero
**solo al salir**. El tono se **tiñe según la estación** (`season_glow_tint`,
[`seasons.rs`](../crates/andes-core/src/seasons.rs)): cálido en otoño y primavera, frío en invierno;
el verano es la base sin tinte. El tinte se aplica solo al resplandor, no a la clave `sunGlow`
global.

## Estrellas

Campo de ~110 estrellas deterministas por semilla (`Sky::new`, accesibles con `Sky::star(i)`),
visibles con `night_amount > 0.02`, con parpadeo suave. Aparecen al caer la noche y se apagan al
amanecer.

## Vía Láctea

Banda de polvo estelar inclinada determinista por semilla, visible con `night_amount > 0.15`. Se
compone de tres capas tenues (`pal.star`/`pal.cloud`) más polvo brillante disperso, y queda **por
detrás de las estrellas**.

## Estrellas fugaces

`meteor_at(seed, t_sec)` (pura, exportada) define **estrellas fugaces** en bloques de tiempo raros
(~5.5 s, p≈0.55) con un trazo de ~0.7 s; `Sky::draw_shooting_stars` lo dibuja solo con
`nightAmount > 0.25`, tras las estrellas y la aurora. La posición va en fracciones de ancho/alto del
cielo, así no depende de `W`/`H`. Ver D-021 (JS).

## Reflejo del astro en el mar

`draw_sea` ([`terrain.rs`](../crates/andes-core/src/terrain.rs)) recibe la **coordenada `x` del
astro** (`cel.x`) y pinta una columna de brillo con destellos, teñida con `sunGlow`/`sun`. Es
determinista y se anima con `t_sec`.

## Aurora austral

Bandas verticales semitransparentes (verde/cian/violeta) cerca de la parte superior del cielo,
visibles con `night_amount > 0.35`. Guiño al hemisferio sur.

## Nubes

Nubes de puffs elípticos que derivan lentamente con parallax bajo (`camera_x * 0.04`) y se tiñen
según la hora y el clima. Se dibujan **por delante de las estrellas y del sol/luna** y **por detrás
del terreno**. `Sky::cloud(i)` expone su estado inicial (paridad exacta de init).

## Pendiente

- Refinar el **antialias de las nubes** para acercar aún más la paridad de píxeles con Canvas
  (nuestro supermuestreo frente al antialias analítico de Canvas); es la mayor fuente de diferencias
  ([11 · Roadmap](11-roadmap.md)).
