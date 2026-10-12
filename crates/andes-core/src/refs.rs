//! Fila de referencia fotográfica (provenance) compartida por flora y fauna.
//!
//! Las referencias son material de edición (nunca se cargan en runtime). Los datos los emite
//! `scripts/gen-parity.mjs` desde el manifiesto del proyecto JS; ver `docs/18-referencias.md`.

pub struct RefRow {
    pub encuadre: &'static str,
    pub fuente: &'static str,
    pub autor: &'static str,
    pub licencia: &'static str,
    pub archivo: &'static str,
}
