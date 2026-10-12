//! Genera la tabla de la paleta maestra (`docs/14-paleta-maestra.md`) a partir de
//! los datos del núcleo (`palette_data.rs`), sin depender de Node.
//!
//! Uso: `cargo run -p andes-core --bin gen-palette` (desde el repo).

use std::fs;
use std::path::PathBuf;

use andes_core::palette::{FIELDS, KEYS};

const BEGIN: &str = "<!-- BEGIN:paleta-tabla";
const END: &str = "<!-- END:paleta-tabla -->";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn hour_label(t: f64) -> String {
    let total = (t * 24.0 * 60.0).round() as i64;
    let h = (total / 60).rem_euclid(24);
    let m = total % 60;
    format!("{h:02}:{m:02}")
}

fn table() -> String {
    // La última clave duplica la medianoche.
    let keys = &KEYS[..KEYS.len() - 1];
    let mut out = String::new();
    out.push_str("| Campo |");
    for k in keys {
        out.push_str(&format!(" {} |", hour_label(k.0)));
    }
    out.push('\n');
    out.push_str("|-------|");
    for _ in keys {
        out.push_str("------|");
    }
    out.push('\n');
    for (i, field) in FIELDS.iter().enumerate() {
        out.push_str(&format!("| `{field}` |"));
        for k in keys {
            let c = k.1[i];
            out.push_str(&format!(" #{:02x}{:02x}{:02x} |", c[0], c[1], c[2]));
        }
        out.push('\n');
    }
    out.trim_end().to_string()
}

fn main() {
    let path = root().join("docs/14-paleta-maestra.md");
    let src = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("no se pudo leer {}: {e}", path.display());
            std::process::exit(1);
        }
    };
    let Some(begin) = src.find(BEGIN) else {
        eprintln!("falta el marcador {BEGIN}");
        std::process::exit(1);
    };
    let Some(end) = src.find(END) else {
        eprintln!("falta el marcador {END}");
        std::process::exit(1);
    };
    if end <= begin {
        eprintln!("marcadores desordenados");
        std::process::exit(1);
    }
    let begin_line_end = src[begin..].find("-->").map(|i| begin + i + 3).unwrap();
    let out = format!("{}\n{}\n{}", &src[..begin_line_end], table(), &src[end..]);
    match fs::write(&path, out) {
        Ok(()) => println!("actualizado: {}", path.display()),
        Err(e) => {
            eprintln!("no se pudo escribir {}: {e}", path.display());
            std::process::exit(1);
        }
    }
}
