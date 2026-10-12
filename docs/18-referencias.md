# 18 · Referencias

> Estado: en progreso · Actualizado: 2026-10-12

Uso de **imágenes fotográficas reales como guía** para dibujar los sprites de flora y fauna con más
fidelidad. Las referencias son **material de edición**: nunca se cargan en tiempo de ejecución (el
sprite sigue siendo una matriz de píxeles en código, D-010 JS) y **no se versionan en el
repositorio**.

En el **port a Rust** las referencias son **propiedad del proyecto original**: el manifiesto y el
buscador viven en el repo JS ([`../../infinite-andes/docs/18-referencias.md`](../../infinite-andes/docs/18-referencias.md)),
y este repo las hereda al portar los sprites.

## Reglas

1. **Licencia:** solo **dominio público / CC0** (uso libre, sin obligación de atribución).
2. **Manifiesto versionado** con la procedencia de cada imagen; el binario queda fuera.
3. **Sin runtime:** ninguna imagen entra al render ni a los tests.
4. **Fidelidad primero:** la silueta y la proporción mandan sobre el color.

## Estado en el port

- La **metadata de identidad** de flora y fauna (incluido el campo `ref`, si aplica) se **genera**
  desde el JS y se muestra en las fichas de [`especies/`](especies/README.md).
- **Pendiente** decidir si el port mantiene su propio manifiesto/carpeta `refs/` o si reutiliza el
  del repo JS tal cual. Por ahora no se versionan imágenes en este repositorio.
- Los sprites portados son **idénticos** a los del original (matrices generadas o portadas a mano),
  así que heredan las referencias ya curadas allí.

## Flujo (heredado)

1. Elegir 2 fuentes **PD/CC0** representativas (idealmente perfil y detalle de cabeza/silueta).
2. Registrar la fila en el manifiesto (fuente, autor, licencia, encuadre).
3. Descargar la imagen a `refs/` (ignorada por git) con el nombre indicado.
4. Traducir a matriz: silueta y proporción sobre la foto; luego color con las claves de la paleta.
5. Regenerar fichas con `cargo run -p andes-core --bin gen-species` si se tocó el registro.

## Criterios de aceptación

- [x] Ninguna imagen se carga en tiempo de ejecución (solo matrices en código).
- [ ] Manifiesto propio del port (o decisión explícita de reutilizar el del repo JS).
- [ ] Las fichas generadas muestran el campo `ref` cuando la especie lo declare.

## Referencias

- D-053, D-047, D-048 (repo JS) · [08 · Arte pixel](08-arte-pixel.md) ·
  [05 · Flora](05-flora.md) · [06 · Fauna](06-fauna.md).
