//! Genera las fichas de especies (`docs/especies/`) y las tablas de 05/06 a partir
//! de los registros `FLORA` y `SPECIES` (fuente única), más la metadata generada
//! desde `flora_data.rs`/`fauna_data.rs`.
//!
//! Uso: `cargo run -p andes-core --bin gen-species` (desde el repo).
//! Equivalente Rust de `scripts/species.mjs` del proyecto JS.

use std::fs;
use std::path::PathBuf;

use andes_core::fauna::{RARITY_WEIGHT, SPECIES};
use andes_core::flora::FLORA;
use andes_core::flora_data::FLORA_META;
use andes_core::refs::RefRow;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn today() -> String {
    if let Ok(d) = std::env::var("GEN_SPECIES_DATE") {
        return d;
    }
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let (y, m, d) = civil_from_days(secs.div_euclid(86400));
    format!("{y:04}-{m:02}-{d:02}")
}

// Algoritmo civil-from-days de Howard Hinnant (sin dependencias).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn biome_name(b: &str) -> &str {
    match b {
        "altiplano" => "Altiplano",
        "norte" => "Norte",
        "centro" => "Centro",
        "sur" => "Sur",
        "patagonia" => "Patagonia",
        "austral" => "Austral",
        other => other,
    }
}

fn layer_name(l: &str) -> &str {
    match l {
        "andes" => "Andes",
        "precordillera" => "Precordillera",
        "valle" => "Valle",
        "costa" => "Costa",
        "playa" => "Playa",
        "mar" => "Mar",
        other => other,
    }
}

fn active_name(a: &str) -> &str {
    match a {
        "day" => "día",
        "night" => "noche",
        "dusk" => "crepúsculo",
        other => other,
    }
}

fn rarity_name(r: &str) -> &str {
    match r {
        "abundante" => "abundante",
        "comun" => "común",
        "poco-comun" => "poco-común",
        "rara" => "rara",
        "muy-rara" => "muy-rara",
        other => other,
    }
}

fn num(x: f64) -> String {
    if x.fract() == 0.0 {
        format!("{}", x as i64)
    } else {
        format!("{x}")
    }
}

fn sci(s: &str) -> String {
    if s.is_empty() {
        "—".into()
    } else {
        format!("*{s}*")
    }
}

fn zone_label(zones: &[(&str, &str, f64)]) -> String {
    if zones.is_empty() {
        return "—".into();
    }
    let mut biomes: Vec<&str> = Vec::new();
    let mut layers: Vec<&str> = Vec::new();
    for (b, l, _) in zones {
        if !biomes.contains(b) {
            biomes.push(b);
        }
        if !layers.contains(l) {
            layers.push(l);
        }
    }
    let bs = biomes
        .iter()
        .map(|b| biome_name(b))
        .collect::<Vec<_>>()
        .join(", ");
    let ls = layers
        .iter()
        .map(|l| layer_name(l).to_lowercase())
        .collect::<Vec<_>>()
        .join(", ");
    format!("{bs} ({ls})")
}

fn layers_of(zones: &[(&str, &str, f64)]) -> String {
    let mut layers: Vec<&str> = Vec::new();
    for (_, l, _) in zones {
        if !layers.contains(l) {
            layers.push(l);
        }
    }
    let out = layers
        .iter()
        .map(|l| layer_name(l))
        .collect::<Vec<_>>()
        .join(", ");
    if out.is_empty() {
        "—".into()
    } else {
        out
    }
}

fn refs_section(refs: &[RefRow]) -> String {
    if refs.is_empty() {
        return String::new();
    }
    let mut lines = vec![
        "## Referencias fotográficas".to_string(),
        String::new(),
        "| Encuadre | Fuente | Autor | Licencia | Archivo |".to_string(),
        "|----------|--------|-------|----------|---------|".to_string(),
    ];
    for r in refs {
        let enc = if r.encuadre.is_empty() {
            "—"
        } else {
            r.encuadre
        };
        let autor = if r.autor.is_empty() { "—" } else { r.autor };
        lines.push(format!(
            "| {enc} | {} | {autor} | {} | `{}` |",
            r.fuente, r.licencia, r.archivo
        ));
    }
    lines.push(String::new());
    format!("{}\n", lines.join("\n"))
}

fn meta_for(name: &str) -> &'static andes_core::flora_data::FloraMeta {
    FLORA_META
        .iter()
        .find(|m| m.name == name)
        .expect("toda entrada de FLORA debe tener metadata en flora_data.rs")
}

// ---------------------------------------------------------------- fichas

fn flora_ficha(name: &str) -> String {
    let e = FLORA.iter().find(|f| f.name == name).unwrap();
    let d = meta_for(name);
    format!(
        "# {common} (`{name}`)\n\n\
         > Ficha generada con `cargo run -p andes-core --bin gen-species` · Actualizado: {today}\n\n\
         | Campo | Valor |\n\
         |-------|-------|\n\
         | Nombre común | {common} |\n\
         | Nombre científico | {sci} |\n\
         | Endémica de Chile | {endemism} |\n\
         | Tipo en código | `{name}` (flora) |\n\
         | Zona / capa | {zone} |\n\
         | Altura relativa (juego) | ×{height} |\n\
         | Altura real típica | {height_m} |\n\n\
         ## Notas\n\n\
         {notes}\n\n\
         {refs}## Referencias\n\n\
         - [05 · Flora](../../05-flora.md)\n\
         - [`crates/andes-core/src/flora.rs`](../../../crates/andes-core/src/flora.rs)\n",
        common = if d.common.is_empty() { name } else { d.common },
        sci = sci(d.sci),
        endemism = if d.endemism.is_empty() { "—" } else { d.endemism },
        zone = zone_label(e.zones),
        height = num(e.height),
        height_m = if d.height_m.is_empty() { "—" } else { d.height_m },
        notes = if d.notes.is_empty() { "—" } else { d.notes },
        refs = refs_section(d.refs),
        today = today(),
    )
}

fn fauna_ficha(sp: &andes_core::fauna::SpeciesData) -> String {
    let base = sp.frames[0];
    let w = base.iter().map(|r| r.chars().count()).max().unwrap_or(0);
    let anchor = if sp.anchor == "ground" {
        "pies"
    } else {
        "centro"
    };
    let placement = if sp.river.is_some() {
        "borde del cauce"
    } else {
        "por chunk"
    };
    format!(
        "# {common} (`{name}`)\n\n\
         > Ficha generada con `cargo run -p andes-core --bin gen-species` · Actualizado: {today}\n\n\
         | Campo | Valor |\n\
         |-------|-------|\n\
         | Nombre común | {common} |\n\
         | Nombre científico | {sci} |\n\
         | Endémica de Chile | {endemism} |\n\
         | Estado UICN | {iucn} |\n\
         | Tipo en código | `{name}` (fauna) |\n\
         | Movimiento | `{movement}` |\n\
         | Actividad | {active} |\n\
         | Rareza (juego) | {rarity} |\n\
         | Anclaje | {anchor} |\n\
         | Colocación | {placement} |\n\
         | Sprite | {w}×{h} px · {frames} frames |\n\n\
         ## Notas\n\n\
         {notes}\n\n\
         {refs}## Referencias\n\n\
         - [06 · Fauna](../../06-fauna.md)\n\
         - [`crates/andes-core/src/fauna.rs`](../../../crates/andes-core/src/fauna.rs)\n",
        common = sp.common,
        name = sp.name,
        sci = sci(sp.sci),
        endemism = if sp.endemism.is_empty() { "—" } else { sp.endemism },
        iucn = if sp.iucn.is_empty() { "—" } else { sp.iucn },
        movement = sp.movement,
        active = active_name(sp.active),
        rarity = rarity_name(sp.rarity),
        h = base.len(),
        frames = sp.frames.len(),
        notes = if sp.notes.is_empty() { "—" } else { sp.notes },
        refs = refs_section(sp.refs),
        today = today(),
    )
}

// ------------------------------------------------------------------ tablas

fn flora_table() -> String {
    let mut rows = vec![
        "| Nombre común | Nombre científico | Endémica | Tipo en código | Zona/capa | Altura real | Notas |".to_string(),
        "|--------------|-------------------|----------|----------------|-----------|-------------|-------|".to_string(),
    ];
    for e in FLORA {
        let d = meta_for(e.name);
        rows.push(format!(
            "| {} | {} | {} | `{}` | {} | {} | {} |",
            if d.common.is_empty() {
                e.name
            } else {
                d.common
            },
            sci(d.sci),
            if d.endemism.is_empty() {
                "—"
            } else {
                d.endemism
            },
            e.name,
            zone_label(e.zones),
            if d.height_m.is_empty() {
                "—"
            } else {
                d.height_m
            },
            if d.notes.is_empty() { "—" } else { d.notes },
        ));
    }
    rows.join("\n")
}

fn fauna_implementado_table() -> String {
    let mut rows = vec![
        "| Tipo en código | Nombre común | Capa(s) | Movimiento | Actividad |".to_string(),
        "|----------------|--------------|---------|------------|-----------|".to_string(),
    ];
    for sp in SPECIES {
        let extra = if sp.river.is_some() {
            " (borde del cauce)"
        } else {
            ""
        };
        rows.push(format!(
            "| `{}` | {} | {}{extra} | `{}` | {} |",
            sp.name,
            sp.common,
            layers_of(sp.zones),
            sp.movement,
            active_name(sp.active),
        ));
    }
    rows.join("\n")
}

fn fauna_densidad_table() -> String {
    let mut entries: Vec<&andes_core::fauna::SpeciesData> = SPECIES.iter().collect();
    entries.sort_by(|a, b| {
        let wa = weight_of(a.rarity);
        let wb = weight_of(b.rarity);
        wb.partial_cmp(&wa)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.common.cmp(b.common))
    });
    entries
        .iter()
        .map(|sp| {
            format!(
                "| {} | {} | {} | {} | {} |",
                sp.common,
                if sp.iucn.is_empty() { "—" } else { sp.iucn },
                if sp.chile.is_empty() { "—" } else { sp.chile },
                rarity_name(sp.rarity),
                num(weight_of(sp.rarity)),
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn weight_of(rarity: &str) -> f64 {
    RARITY_WEIGHT
        .iter()
        .find(|(k, _)| *k == rarity)
        .map(|(_, w)| *w)
        .unwrap_or(1.0)
}

fn fauna_objetivo_table() -> String {
    let mut rows = vec![
        "| Nombre común | Nombre científico | Endémica | Capa/zona | Actividad | Comportamiento |"
            .to_string(),
        "|--------------|-------------------|----------|-----------|-----------|----------------|"
            .to_string(),
    ];
    for sp in SPECIES {
        rows.push(format!(
            "| {} | {} | {} | {} | {} | {} |",
            sp.common,
            sci(sp.sci),
            if sp.endemism.is_empty() {
                "—"
            } else {
                sp.endemism
            },
            zone_label(sp.zones),
            active_name(sp.active),
            if sp.notes.is_empty() { "—" } else { sp.notes },
        ));
    }
    rows.join("\n")
}

// --------------------------------------------------------------- inyección

fn inject(file: &PathBuf, tag: &str, content: &str) {
    let src = fs::read_to_string(file).unwrap_or_else(|_| panic!("no se pudo leer {file:?}"));
    let begin = format!("<!-- BEGIN:{tag}");
    let end = format!("<!-- END:{tag} -->");
    let i = src
        .find(&begin)
        .unwrap_or_else(|| panic!("marcador {tag} no hallado en {file:?}"));
    let j = src
        .find(&end)
        .unwrap_or_else(|| panic!("fin de marcador {tag} no hallado en {file:?}"));
    assert!(i < j, "marcadores {tag} invertidos en {file:?}");
    let begin_line_end = src[i..].find("-->").map(|k| i + k + 3).unwrap();
    let result = format!(
        "{}{}\n{}\n{}",
        &src[..i],
        &src[i..begin_line_end],
        content,
        &src[j..]
    );
    fs::write(file, result).unwrap();
}

fn main() {
    let root = root();
    let today = today();

    let out = root.join("docs/especies");
    let _ = fs::remove_dir_all(&out);
    fs::create_dir_all(&out).unwrap();

    let flora_codes: Vec<&str> = FLORA
        .iter()
        .filter(|e| e.kind == "especie")
        .map(|e| e.name)
        .collect();

    for name in &flora_codes {
        fs::write(out.join(format!("flora-{name}.md")), flora_ficha(name)).unwrap();
    }
    for sp in SPECIES {
        fs::write(out.join(format!("fauna-{}.md", sp.name)), fauna_ficha(sp)).unwrap();
    }

    let mut index = vec![
        "# Especies".to_string(),
        String::new(),
        format!("> Fichas generadas con `cargo run -p andes-core --bin gen-species` · Actualizado: {today}"),
        String::new(),
        "Una ficha por especie (flora y fauna), generada desde los registros `FLORA` y `SPECIES`".to_string(),
        "(fuente única de identidad, zonas y metadata). Plantilla en [`../templates/especimen.md`](../templates/especimen.md).".to_string(),
        String::new(),
        "## Flora".to_string(),
        String::new(),
    ];
    for name in &flora_codes {
        let d = meta_for(name);
        index.push(format!(
            "- [{}](flora-{name}.md)",
            if d.common.is_empty() { name } else { d.common }
        ));
    }
    index.push(String::new());
    index.push("## Fauna".to_string());
    index.push(String::new());
    for sp in SPECIES {
        index.push(format!("- [{}](fauna-{}.md)", sp.common, sp.name));
    }
    index.push(String::new());
    fs::write(out.join("README.md"), index.join("\n")).unwrap();

    inject(
        &root.join("docs/05-flora.md"),
        "flora-tabla",
        &flora_table(),
    );
    inject(
        &root.join("docs/06-fauna.md"),
        "fauna-implementado",
        &fauna_implementado_table(),
    );
    inject(
        &root.join("docs/06-fauna.md"),
        "fauna-densidad",
        &fauna_densidad_table(),
    );
    inject(
        &root.join("docs/06-fauna.md"),
        "fauna-objetivo",
        &fauna_objetivo_table(),
    );

    println!(
        "Especies: {} flora + {} fauna (fichas y tablas de 05/06)",
        flora_codes.len(),
        SPECIES.len()
    );
    let _ = today;
}
