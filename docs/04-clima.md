# 04 · Clima

> Estado: estable · Actualizado: 2026-10-12

Implementado en [`crates/andes-core/src/weather.rs`](../crates/andes-core/src/weather.rs) (partículas,
niebla y relámpagos) y [`palette.rs`](../crates/andes-core/src/palette.rs) (tintes de color). La
lógica de cambio automático está en `Scene::update`. El **spawn de partículas** tiene paridad exacta
con JS ([11 · Roadmap](11-roadmap.md)).

## Estados

| Estado | Clave | Efecto |
|--------|-------|--------|
| Despejado | `clear` | Sin partículas; colores base |
| Nieve | `snow` | Partículas blancas que caen con deriva; cielo y montañas desaturados |
| Lluvia | `rain` | Trazos inclinados por el viento; escena oscurecida y desaturada |
| Niebla | `fog` | Capas horizontales translúcidas + tinte gris; reduce contraste |
| Viento | `wind` | Ráfagas y hojas; colores algo más secos |
| Tormenta | `storm` | Lluvia densa y rápida, cielo muy oscuro y **relámpagos** |

`wind_level_for(type)` da el viento de cada clima; `Weather::effective_wind()` cuenta también el
clima saliente durante el crossfade. El viento afecta la velocidad de las partículas, la inclinación
de la precipitación y la agitación del mar (D-029 JS). La tormenta es un evento puntual:
`draw_lightning` dibuja un rayo entre el cielo y el terreno con un flash breve, con temporizador
determinista (1.5–6.5 s).

## Transiciones

Los cambios de clima usan **crossfade**: el clima saliente se desvanece mientras el entrante sube
**en paralelo** (`prev_type`/`prev_strength` en `Weather`, D-029 JS). Ambos avanzan a la misma tasa,
de modo que la suma de intensidades se mantiene ≈ 1 durante la transición (≈ 2 s). La paleta se
compone mezclando las dos paletas de clima (`lerp_palettes` en `Scene::render`) y las partículas de
ambos estados se dibujan a la vez.

## Automático

Con el clima en **Automático** (`weather_auto`), cada **30–60 s** se elige un estado distinto del
actual. El sorteo es **ponderado por la estación** (`pick_season_weather`, ver
[15 · Estaciones](15-estaciones.md)): invierno favorece nieve/lluvia/tormenta y verano el despejado.
Con el reloj real la estación es la de la fecha actual, así que el clima acompaña la época real de
Chile. El usuario puede **forzar** un clima (`set_weather_type`), lo que desactiva el modo automático.

## Interacción con la paleta

`apply_weather(pal, weather, strength)` mezcla los colores hacia los tonos del clima según la
intensidad (`strength`), de modo que la nieve "blanquea" y la lluvia o la tormenta "ensombrecen"
toda la escena (cielo, montañas, valle, costa, arena, mar).

## Parámetros

- **Nativo**: selector **Clima** del panel (automático o forzado).
- **Web**: `?weather=clear|snow|rain|fog|wind|storm` deja el clima fijo (útil para capturas);
  `ia_set_weather` en el ABI C.

## Pendiente

- Ninguna pendiente: el crossfade, el viento y la tormenta están portados (D-029 JS). Los **spawns**
  de partículas son exactos; el **dibujo** se compara aparte en el arnés de píxeles.
