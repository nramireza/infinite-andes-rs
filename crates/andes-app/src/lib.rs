//! Estado de la aplicación compartido por los frontends nativo y web.
//!
//! Fase 1: esqueleto. Se llenará al portar `scene.js` (cámara, reloj, clima).

/// Ancho de lienzo por defecto (32:9) — marcador de la dependencia con el núcleo.
pub fn placeholder() -> u32 {
    andes_core::viewport::width_for_ratio(andes_core::viewport::ASPECT_PRESETS[2].ratio)
}
