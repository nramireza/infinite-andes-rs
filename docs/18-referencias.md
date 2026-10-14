# 18 · Referencias

> Estado: en progreso · Actualizado: 2026-10-14

Uso de **referencias reales como guía** para dibujar la flora y la fauna con fidelidad. Son
**material de edición**: nunca se cargan en tiempo de ejecución (el sprite/la planta se dibuja en
código, D-010 JS, D-016) y **no se versionan imágenes en el repositorio**. En el port a Rust las
referencias son **propiedad de este repositorio** (D-024).

## Referencia canónica: Wikipedia + porte

Cada especie real declara en su registro (`flora_data.rs` / `fauna_data.rs`) dos campos:

| Campo | Qué es |
|-------|--------|
| `wiki` | Enlace al **artículo de Wikipedia** (referencia estable para leer e imágenes). |
| `porte` | **Descripción fiel de la silueta/hábito** en español: es la **especificación de dibujo**. |

- La **silueta manda**: el `porte` describe el porte (proporciones, rasgos distintivos, hábito) y el
  dibujo debe reconocerse **sin color** y coincidir con él (D-023, D-024).
- Los tipos genéricos (`bush`, `crop`, `grass`, `rock`, `flower`) llevan `porte` genérico y `wiki`
  vacío.
- El artículo de Wikipedia debe existir y resolver (se verifica con un `curl` antes de fijarlo;
  se usa el **nombre científico** como título, que Wikipedia redirige a la página correcta).

Las **fotos PD/CC0** (Wikimedia Commons, PhyloPic) siguen en el campo `refs` como procedencia
opcional para la forma; no son obligatorias si el `wiki` + `porte` bastan.

## Reglas

1. **Licencia de fotos:** solo **dominio público / CC0** (uso libre, sin obligación de atribución).
2. **Sin runtime:** ninguna imagen entra al render ni a los tests.
3. **Fidelidad primero:** la silueta y la proporción mandan sobre el color.
4. **Enlaces, no binarios:** el repo guarda enlaces y texto; las imágenes viven fuera.

## Fichas

`cargo run -p andes-core --bin gen-species` genera `docs/especies/*.md` con:

- **## Descripción / porte** — desde `porte`.
- **## Referencia** — enlace a Wikipedia desde `wiki`.
- **## Referencias fotográficas** — tabla de `refs` (si hay).

## Flujo por especie

1. Abrir el artículo de Wikipedia (`wiki`) y leer la sección **Descripción**.
2. Escribir/ajustar el `porte` (silueta, proporciones, rasgos distintivos).
3. Redibujar en `flora.rs`/`fauna.rs` con la **galería de diagnóstico** (D-022) como criterio.
4. Si aplica, añadir una foto PD/CC0 a `refs` (procedencia).
5. Regenerar fichas: `cargo run -p andes-core --bin gen-species`.

## Criterios de aceptación

- [x] Referencia estable `wiki` en las especies reales (flora y fauna).
- [x] Campo `porte` (silueta) en todas las entradas y presente en las fichas.
- [x] Ninguna imagen se carga en runtime ni se versiona.
- [ ] Completar las **fotos** `refs` que faltan (10 flora y `culpeo`/`huillin`).
- [ ] Revisar la fidelidad de todas las siluetas (por lotes, con la galería).

## Referencias

- D-023, D-024 · [05 · Flora](05-flora.md) · [06 · Fauna](06-fauna.md) ·
  [08 · Arte pixel](08-arte-pixel.md).
