//! Núcleo determinista de Infinite Andes: simulación y rasterizado del paisaje.
//!
//! Puerto exacto de los módulos puros de `src/` del proyecto JS. La paridad con
//! JavaScript se verifica en `tests/parity.rs` contra vectores generados por
//! `scripts/gen-parity.mjs`.
//!
//! Regla de oro: nada de aleatoriedad no reproducible. Todo deriva de la semilla.

pub mod activity;
pub mod biomes;
pub mod clock;
pub mod fauna;
pub mod flora;
pub mod flora_data;
pub mod framebuffer;
pub mod math;
pub mod moments;
pub mod noise;
pub mod palette;
pub mod pixel;
pub mod refs;
pub mod rng;
pub mod scene;
pub mod seasons;
pub mod sky;
pub mod sun;
pub mod terrain;
pub mod viewport;
pub mod views;
pub mod weather;
