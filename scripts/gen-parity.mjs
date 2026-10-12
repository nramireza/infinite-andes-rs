// @reference-harness
// Herramienta de REFERENCIA (opcional). El repo Rust es la fuente de verdad.
//
// Regenera los datos del núcleo desde el proyecto JS hermano (por defecto
// ../infinite-andes, ajustable con ANDES_JS_DIR) y sobrescribe:
//   - crates/andes-core/src/palette_data.rs
//   - crates/andes-core/src/fauna_data.rs
//   - crates/andes-core/src/flora_data.rs
//   - crates/andes-core/tests/generated/parity_data.rs (vectores de paridad)
//
// Úsalo solo para refrescar el snapshot desde JS o para el test opcional
// `cargo test --features parity`; NO forma parte del flujo normal. Uso:
//   TZ=UTC node scripts/gen-parity.mjs

import { fileURLToPath } from "node:url";
import { dirname, join, resolve } from "node:path";
import { mkdirSync, writeFileSync, readFileSync, existsSync } from "node:fs";

const HERE = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(HERE, "..");
const JS = resolve(process.env.ANDES_JS_DIR || join(ROOT, "..", "infinite-andes"));

const js = (p) => fileURLToPath(new URL(p, `file://${JS}/`));

// Los vectores de fecha se generan en UTC para que el calendario civil de Rust
// (sin horario de verano) coincida con `Date`.
process.env.TZ = "UTC";

const rng = await import(js("src/rng.js"));
const noise = await import(js("src/noise.js"));
const viewport = await import(js("src/viewport.js"));
const palette = await import(js("src/palette.js"));
const pixel = await import(js("src/pixel.js"));
const clock = await import(js("src/clock.js"));
const sun = await import(js("src/sun.js"));
const seasons = await import(js("src/seasons.js"));
const biomes = await import(js("src/biomes.js"));
const terrain = await import(js("src/terrain.js"));
const sky = await import(js("src/sky.js"));
const weather = await import(js("src/weather.js"));
const fauna = await import(js("src/fauna.js"));
const flora = await import(js("src/flora.js"));
const moments = await import(js("src/moments.js"));
const views = await import(js("src/views.js"));
const references = await import(js("src/references.js"));
const activity = await import(js("src/activity.js"));

// Referencias fotográficas de edición (manifiesto del repo JS; ver docs/18).
const MANIFEST = js("docs/referencias/manifiesto.csv");
const REFS = existsSync(MANIFEST) ? references.parseManifest(readFileSync(MANIFEST, "utf8")) : [];

// Emite una lista de `RefRow` para el campo `ref` de flora/fauna.
function refArr(def) {
  const ids = Array.isArray(def.ref) ? def.ref : def.ref ? [def.ref] : [];
  const rows = ids.flatMap((id) => references.refsFor(REFS, id));
  if (!rows.length) return "&[]";
  const item = (r) =>
    `RefRow { encuadre: ${s(r.encuadre || "")}, fuente: ${s(r.fuente || "")}, autor: ${s(r.autor || "")}, licencia: ${s(r.licencia || "")}, archivo: ${s(r.archivo || "")} }`;
  return `&[${rows.map(item).join(", ")}]`;
}

// ---------------------------------------------------------------- formatters
function f(n) {
  if (typeof n !== "number") throw new Error("no es número: " + n);
  if (Number.isNaN(n)) return "f64::NAN";
  if (n === Infinity) return "f64::INFINITY";
  if (n === -Infinity) return "f64::NEG_INFINITY";
  let s = String(n);
  if (!/[.eE]/.test(s)) s += ".0";
  return s;
}
const s = (str) => JSON.stringify(str);
const rgb = (hex) => "[" + palette.hexToRgb(hex).join(", ") + "]";
const rgbArr = (arr) => "[" + arr.join(", ") + "]";
const optF = (v) => (v == null ? "None" : `Some(${f(v)})`);
const optS = (v) => (v == null ? "None" : `Some(${s(v)})`);

// --------------------------------------------------------- palette_data.rs
const FIELDS = palette.FIELDS;
const fieldIndex = new Map(FIELDS.map((k, i) => [k, i]));

const WEATHER_TARGETS = {
  snow: {
    skyTop: "#8f9aa8", skyMid: "#aab4c0", skyHorizon: "#cfd6dd",
    farL: "#8a94a2", farD: "#78828f", midL: "#8590a0", midD: "#707b8a", nearL: "#7c8894", nearD: "#68737f",
    valleyL: "#8a949c", valleyD: "#767f86", costaL: "#8a949c", costaD: "#767f86",
    sand: "#c8ccd0", sandD: "#a8adb2", sea: "#7f8a94", seaD: "#6c7680", seaHi: "#a8b2ba",
  },
  rain: {
    skyTop: "#4a5560", skyMid: "#5a646e", skyHorizon: "#78828c",
    farL: "#55606a", farD: "#48525b", midL: "#4e585f", midD: "#424b52", nearL: "#454e55", nearD: "#3a4248",
    valleyL: "#46524a", valleyD: "#3a443c", costaL: "#46524a", costaD: "#3a443c",
    sand: "#5a5248", sandD: "#48423a", sea: "#3a4750", seaD: "#2f3a42", seaHi: "#55626a",
  },
  fog: {
    skyTop: "#c9d2da", skyMid: "#cdd5dd", skyHorizon: "#d3dae1",
    farL: "#c2cbd3", farD: "#b8c1c9", midL: "#bcc5cd", midD: "#b2bbc3", nearL: "#b6bfc7", nearD: "#acb5bd",
    valleyL: "#bcc5cd", valleyD: "#b2bbc3", costaL: "#bcc5cd", costaD: "#b2bbc3",
    sand: "#c4cbd1", sandD: "#b8bfc5", sea: "#bcc5cd", seaD: "#b2bbc3", seaHi: "#cdd5db",
  },
  wind: { skyTop: "#3c8fca", skyMid: "#8ec0da", skyHorizon: "#e2dcc4" },
  storm: {
    skyTop: "#2c3642", skyMid: "#3c4854", skyHorizon: "#6a7480",
    farL: "#3c4754", farD: "#303a45", midL: "#38424c", midD: "#2b343d", nearL: "#313a42", nearD: "#262e35",
    valleyL: "#3a443c", valleyD: "#2e3630", costaL: "#3a443c", costaD: "#2e3630",
    sand: "#4a4640", sandD: "#3a3632", sea: "#2c3a44", seaD: "#233038", seaHi: "#4a5a66",
  },
};

const BIOME_KEYS = [
  "farL", "farD", "midL", "midD", "nearL", "nearD",
  "valleyL", "valleyD", "costaL", "costaD",
  "sand", "sandD", "rock", "rockD", "floraL", "floraD",
  "snow", "snowD", "ground", "groundHi",
];

const BLOOM_TINTS = [
  ["sand", "#e6b8c8", 0.6], ["sandD", "#c89aa0", 0.45],
  ["valleyL", "#c8a86a", 0.45], ["valleyD", "#b08a58", 0.4],
  ["costaL", "#c2a878", 0.4], ["costaD", "#9a8060", 0.35],
  ["floraL", "#d98ab0", 0.5], ["floraD", "#a86a90", 0.4],
];

let out = "// Datos del núcleo Rust (fuente; editable a mano). Snapshot inicial derivado del JS con scripts/gen-parity.mjs.\n\n";
out += `pub const NFIELDS: usize = ${FIELDS.length};\n`;
out += `pub const FIELDS: [&str; NFIELDS] = [${FIELDS.map(s).join(", ")}];\n\n`;

out += `pub const KEYS: [(f64, [[u8; 3]; NFIELDS]); ${palette.KEYS.length}] = [\n`;
for (const k of palette.KEYS) {
  const row = FIELDS.map((field) => rgb(k.p[field])).join(", ");
  out += `    (${f(k.t)}, [${row}]),\n`;
}
out += "];\n\n";

const wnames = Object.keys(WEATHER_TARGETS);
out += `pub const WEATHER_TARGETS: [(&str, &[(usize, [u8; 3])]); ${wnames.length}] = [\n`;
for (const name of wnames) {
  const entries = Object.keys(WEATHER_TARGETS[name])
    .map((key) => `(${fieldIndex.get(key)}, ${rgb(WEATHER_TARGETS[name][key])})`)
    .join(", ");
  out += `    (${s(name)}, &[${entries}]),\n`;
}
out += "];\n\n";

out += `pub const BIOME_KEYS: [usize; ${BIOME_KEYS.length}] = [${BIOME_KEYS.map((k) => fieldIndex.get(k)).join(", ")}];\n\n`;

out += `pub const BLOOM_TINTS: [(usize, [u8; 3], f64); ${BLOOM_TINTS.length}] = [\n`;
for (const [key, hex, factor] of BLOOM_TINTS) {
  out += `    (${fieldIndex.get(key)}, ${rgb(hex)}, ${f(factor)}),\n`;
}
out += "];\n";

mkdirSync(join(ROOT, "crates/andes-core/src"), { recursive: true });
writeFileSync(join(ROOT, "crates/andes-core/src/palette_data.rs"), out);

// --------------------------------------------------------- parity_data.rs
let p = "// @generated by scripts/gen-parity.mjs — no editar a mano.\n\n";

const mul = [];
for (const seed of [0, 1, 7, 42, 123456, 4294967295]) {
  const r = rng.mulberry32(seed);
  mul.push(`(${seed >>> 0}u32, [${Array.from({ length: 16 }, () => f(r())).join(", ")}])`);
}
p += `pub const MULBERRY32: &[(u32, [f64; 16])] = &[${mul.join(", ")}];\n\n`;

const hi = [];
const h1 = [];
for (const seed of [0, 3, 9, 123]) {
  for (let n = -25; n <= 25; n += 5) {
    hi.push(`(${n}i32, ${seed}u32, ${rng.hashInt(n, seed)}u32)`);
    h1.push(`(${n}i32, ${seed}u32, ${f(rng.hash1(n, seed))})`);
  }
}
p += `pub const HASH_INT: &[(i32, u32, u32)] = &[${hi.join(", ")}];\n\n`;
p += `pub const HASH1: &[(i32, u32, f64)] = &[${h1.join(", ")}];\n\n`;

const hs = [];
for (const str of ["", "andes", "pewen", "Infinite Andes", "ñandú", "42", "0x1f"]) {
  hs.push(`(${s(str)}, ${rng.hashString(str)}u32)`);
}
p += `pub const HASH_STR: &[(&str, u32)] = &[${hs.join(", ")}];\n\n`;

const sti = [];
for (const v of ["42", "0", "pewen", "", "   ", "3600", "1e3", "3.9"]) {
  sti.push(`(${s(v)}, ${rng.seedToInt(v)}u32)`);
}
p += `pub const SEED_TO_INT: &[(&str, u32)] = &[${sti.join(", ")}];\n\n`;

const vn = [];
for (const seed of [0, 11, 99]) {
  for (const x of [-13.37, -0.5, 0.0, 3.7, 12.34, 100.001]) {
    vn.push(`(${f(x)}, ${seed}u32, ${f(noise.valueNoise1(x, seed))})`);
  }
}
p += `pub const VALUE_NOISE: &[(f64, u32, f64)] = &[${vn.join(", ")}];\n\n`;

const fb = [];
const rd = [];
for (const seed of [0, 5, 99]) {
  for (const x of [-20.0, -3.1, 0.0, 12.34, 44.4]) {
    fb.push(`(${f(x)}, ${seed}u32, 4u32, ${f(2.0)}, ${f(0.5)}, ${f(noise.fbm1(x, seed))})`);
    rd.push(`(${f(x)}, ${seed}u32, 4u32, ${f(noise.ridged1(x, seed))})`);
  }
}
p += `pub const FBM: &[(f64, u32, u32, f64, f64, f64)] = &[${fb.join(", ")}];\n\n`;
p += `pub const RIDGED: &[(f64, u32, u32, f64)] = &[${rd.join(", ")}];\n\n`;

const dith = [];
for (const [x, y, seed] of [[0, 0, 0], [1, 2, 3], [-5, 9, 7], [123, 45, 678], [1000, 2000, 4294967295]]) {
  dith.push(`(${x}i32, ${y}i32, ${seed}u32, ${f(pixel.radixDither(x, y, seed))})`);
}
p += `pub const RADIX_DITHER: &[(i32, i32, u32, f64)] = &[${dith.join(", ")}];\n\n`;

const wid = [];
for (const r of [16 / 9, 21 / 9, 32 / 9, 2.4, 1.0, NaN]) {
  wid.push(`(${f(r)}, ${viewport.widthForRatio(r)}u32)`);
}
p += `pub const WIDTH: &[(f64, u32)] = &[${wid.join(", ")}];\n\n`;

const rfs = [];
for (const v of ["32:9", "21x9", "16/9", "2.4", "2.5", "malo", "", " 32 : 9 ", "0:9", "16:0"]) {
  rfs.push(`(${s(v)}, ${optF(viewport.ratioFromString(v))})`);
}
p += `pub const RATIO_FROM_STRING: &[(&str, Option<f64>)] = &[${rfs.join(", ")}];\n\n`;

const lab = [];
for (const r of [32 / 9, 16 / 9, 21 / 9, 2.4, 1.5]) {
  lab.push(`(${f(r)}, ${s(viewport.labelForRatio(r))})`);
}
p += `pub const LABEL: &[(f64, &str)] = &[${lab.join(", ")}];\n\n`;

const lc = [];
for (const [a, b, t] of [["#000000", "#ffffff", 0], ["#000000", "#ffffff", 1], ["#123456", "#123456", 0.5], ["#ff0000", "#00ff00", 0.37], ["#102030", "#e0d0c0", 0.73]]) {
  lc.push(`(${s(a)}, ${s(b)}, ${f(t)}, ${s(palette.lerpColor(a, b, t))})`);
}
p += `pub const LERP_COLOR: &[(&str, &str, f64, &str)] = &[${lc.join(", ")}];\n\n`;

const sh = [];
for (const [a, amt] of [["#000000", -1], ["#ffffff", 1], ["#000000", 1], ["#808080", 0.25], ["#808080", -0.5]]) {
  sh.push(`(${s(a)}, ${f(amt)}, ${s(palette.shade(a, amt))})`);
}
p += `pub const SHADE: &[(&str, f64, &str)] = &[${sh.join(", ")}];\n\n`;

const gp = [];
const combos = [];
for (const h of [0, 3, 6, 9, 12, 15, 18, 21, 23.9]) combos.push([h, "clear", 1]);
for (const w of ["snow", "rain", "fog", "storm", "wind"]) combos.push([12, w, 1]);
combos.push([12, "rain", 0.5]);
for (const [h, w, strength] of combos) {
  const pal = palette.getPalette(h, w, strength);
  const row = FIELDS.map((field) => rgb(pal[field])).join(", ");
  gp.push(`(${f(h)}, ${s(w)}, ${f(strength)}, [${row}])`);
}
p += `pub const GET_PALETTE: &[(f64, &str, f64, [[u8; 3]; ${FIELDS.length}])] = &[${gp.join(", ")}];\n\n`;

const night = [];
for (const h of [-3, 0, 5, 6, 7, 12, 18, 19, 20, 24, 30]) {
  night.push(`(${f(h)}, ${f(palette.nightAmount(h))})`);
}
p += `pub const NIGHT: &[(f64, f64)] = &[${night.join(", ")}];\n\n`;

// applyBiome con un tinte sintético que cubre todas las claves de bioma más
// claves ajenas (skyTop, trunk) que deben ignorarse.
const tint = {};
for (const k of BIOME_KEYS) tint[k] = palette.shade(palette.getPalette(12, "clear")[k], 0.2);
tint.skyTop = "#ff00ff";
tint.trunk = "#00ff00";
const tintPairs = Object.keys(tint).map((k) => `(${s(k)}, ${s(tint[k])})`).join(", ");
const ab = [];
for (const [amount, bloom] of [[0.6, 0.0], [0.6, 0.5], [0.0, 0.5]]) {
  const pal = palette.applyBiome(palette.getPalette(12, "clear"), tint, amount, bloom);
  const row = FIELDS.map((field) => rgb(pal[field])).join(", ");
  ab.push(`(${f(amount)}, ${f(bloom)}, [${row}])`);
}
p += `pub const BIOME_TINT: &[(&str, &str)] = &[${tintPairs}];\n`;
p += `pub const APPLY_BIOME: &[(f64, f64, [[u8; 3]; ${FIELDS.length}])] = &[${ab.join(", ")}];\n\n`;

const lp = [];
for (const [h, a, sa, b, sb, k] of [[12, "rain", 1, "clear", 0, 0.5], [1, "snow", 1, "clear", 0, 0.25], [12, "storm", 1, "fog", 1, 1], [12, "clear", 1, "wind", 1, 0.75]]) {
  const pal = palette.lerpPalettes(palette.getPalette(h, a, sa), palette.getPalette(h, b, sb), k);
  const row = FIELDS.map((field) => rgb(pal[field])).join(", ");
  lp.push(`(${f(h)}, ${s(a)}, ${f(sa)}, ${s(b)}, ${f(sb)}, ${f(k)}, [${row}])`);
}
p += `pub const LERP_PALETTES: &[(f64, &str, f64, &str, f64, f64, [[u8; 3]; ${FIELDS.length}])] = &[${lp.join(", ")}];\n\n`;

// ------------------------------------------------------------- clock / sun
const dates = [
  [2026, 0, 1, 0, 0, 0, 0], [2026, 0, 15, 20, 30, 0, 0], [2026, 1, 1, 3, 15, 30, 500],
  [2026, 2, 1, 0, 0, 0, 0], [2026, 5, 1, 12, 0, 0, 0], [2026, 8, 1, 0, 0, 0, 0],
  [2026, 11, 1, 0, 0, 0, 0], [2026, 11, 31, 23, 59, 59, 999], [2024, 1, 29, 6, 0, 0, 0],
];
const clk = dates.map(([y, m0, day, h, mi, sec, ms]) => {
  const d = new Date(y, m0, day, h, mi, sec, ms);
  return `(${y}i64, ${m0}u32, ${day}u32, ${h}u32, ${mi}u32, ${sec}u32, ${ms}u32, ${f(clock.localHour(d))}, ${clock.dayOfYear(d)}i64, ${s(clock.seasonForDate(d))}, ${f(clock.seasonPhaseForDate(d))})`;
});
p += `pub const CLOCK: &[(i64, u32, u32, u32, u32, u32, u32, f64, i64, &str, f64)] = &[${clk.join(", ")}];\n\n`;

const moon = [];
for (const [y, doy] of [[2000, 5], [2000, 20], [2026, 0], [2026, 100], [2026, 364], [1999, 300]]) {
  moon.push(`(${y}i64, ${doy}i64, ${f(clock.moonPhaseForDate(y, doy))})`);
}
p += `pub const MOON: &[(i64, i64, f64)] = &[${moon.join(", ")}];\n\n`;

const decl = [];
for (const doy of [0, 81, 172, 266, 355]) decl.push(`(${f(doy)}, ${f(sun.solarDeclination(doy))})`);
p += `pub const SOLAR_DECL: &[(f64, f64)] = &[${decl.join(", ")}];\n\n`;

const stim = [];
for (const [doy, lat, lon, tz] of [[355, -33.45, -70.66, -3], [172, -33.45, -70.66, -4], [81, -33.45, -70.66, -3]]) {
  const t = sun.solarTimes(doy, lat, lon, tz);
  stim.push(`(${f(doy)}, ${f(lat)}, ${f(lon)}, ${f(tz)}, ${f(t.rise)}, ${f(t.set)}, ${f(t.noon)})`);
}
p += `pub const SOLAR_TIMES: &[(f64, f64, f64, f64, f64, f64, f64)] = &[${stim.join(", ")}];\n\n`;

const sclk = [];
for (const [hour, rise, set] of [[6, 7, 18], [7, 7, 18], [12, 7, 18], [18, 7, 18], [18.25, 7, 18], [3, 7, 18], [23, 7, 18], [12, 12, 12]]) {
  sclk.push(`(${f(hour)}, ${f(rise)}, ${f(set)}, ${f(sun.solarClock(hour, rise, set))})`);
}
p += `pub const SOLAR_CLOCK: &[(f64, f64, f64, f64)] = &[${sclk.join(", ")}];\n\n`;

// ---------------------------------------------------------------- seasons
const sstate = [];
for (const [phase, mode] of [[0, "auto"], [0.9, "auto"], [1.0, "auto"], [3.9, "auto"], [4.0, "auto"], [1.3, "otono"], [2.5, "invierno"]]) {
  const st = seasons.seasonState(phase, mode);
  sstate.push(`(${f(phase)}, ${s(mode)}, ${st.index}, ${st.next}, ${f(st.t)})`);
}
p += `pub const SEASON_STATE: &[(f64, &str, usize, usize, f64)] = &[${sstate.join(", ")}];\n\n`;

const ssnow = [];
const sbias = [];
const sglow = [];
for (const phase of [0.0, 0.5, 0.9, 1.4, 2.0, 3.3]) {
  const st = seasons.seasonState(phase, "auto");
  ssnow.push(`(${st.index}, ${st.next}, ${f(st.t)}, ${f(seasons.seasonSnowShift(st))})`);
  const bias = seasons.seasonWeatherBias(st);
  sbias.push(`(${st.index}, ${st.next}, ${f(st.t)}, [${Object.keys(bias).map((k) => f(bias[k])).join(", ")}])`);
  const glow = seasons.seasonGlowTint(st);
  sglow.push(`(${st.index}, ${st.next}, ${f(st.t)}, ${glow ? `Some((${rgbArr(palette.hexToRgb(glow.color))}, ${f(glow.amount)}))` : "None"})`);
}
for (const id of seasons.SEASON_IDS) {
  const st = seasons.seasonState(0, id);
  const glow = seasons.seasonGlowTint(st);
  sglow.push(`(${st.index}, ${st.next}, ${f(st.t)}, ${glow ? `Some((${rgbArr(palette.hexToRgb(glow.color))}, ${f(glow.amount)}))` : "None"})`);
}
p += `pub const SEASON_SNOW: &[(usize, usize, f64, f64)] = &[${ssnow.join(", ")}];\n\n`;
p += `pub const SEASON_BIAS: &[(usize, usize, f64, [f64; 6])] = &[${sbias.join(", ")}];\n\n`;

const aseason = [];
for (const [hour, mode, strength] of [[12, "otono", 0.5], [12, "invierno", 0.5], [6, "primavera", 0.5], [12, "verano", 0.5], [12, "otono", 0.0], [12, "auto", 0.3]]) {
  const pal = seasons.applySeason(palette.getPalette(hour, "clear"), seasons.seasonState(0, mode), strength);
  const row = FIELDS.map((field) => rgb(pal[field])).join(", ");
  aseason.push(`(${f(hour)}, ${s(mode)}, ${f(strength)}, [${row}])`);
}
p += `pub const APPLY_SEASON: &[(f64, &str, f64, [[u8; 3]; ${FIELDS.length}])] = &[${aseason.join(", ")}];\n\n`;
p += `pub const SEASON_GLOW: &[(usize, usize, f64, Option<([u8; 3], f64)>)] = &[${sglow.join(", ")}];\n\n`;

// ----------------------------------------------------------------- biomes
const bwe = [];
const bgeo = [];
const bat = [];
const bbloom = [];
const bres = [];
for (const seed of [662118718, 0, 999]) {
  let x = -4000.0;
  while (x <= 4000.0) {
    const w = biomes.biomeWeights(x, seed);
    bwe.push(`(${f(x)}, ${seed}u32, [${biomes.BIOME_IDS.map((id) => f(w[id])).join(", ")}])`);
    const g = biomes.biomeGeometry(w);
    bgeo.push(`(${f(x)}, ${seed}u32, [${f(g.ampMul)}, ${f(g.snowShift)}, ${f(g.fjord)}])`);
    x += 500.0;
  }
}
for (const mode of [...biomes.BIOME_IDS, "auto"]) {
  for (const x of [0.0, 1234.0, 98765.0]) {
    const b = biomes.biomeAt(x, 662118718, mode);
    const weights = biomes.BIOME_IDS.map((id) => f(b.weights[id])).join(", ");
    const tint = Object.keys(b.tint).map((k) => `(${fieldIndex.get(k)}, ${rgb(b.tint[k])})`).join(", ");
    bat.push(`(${f(x)}, ${s(mode)}, ${f(b.amount)}, ${f(b.bloom)}, [${weights}], &[${tint}])`);
    bbloom.push(`(${f(x)}, 662118718u32, ${s(mode)}, ${f(biomes.bloomAt(x, 662118718, b.weights))})`);
    for (const rm of ["auto", "on", "off"]) {
      bres.push(`(${s(rm)}, [${weights}], ${f(b.bloom)}, ${f(biomes.resolveBloom(rm, b))})`);
    }
  }
}
p += `pub const BIOME_WEIGHTS: &[(f64, u32, [f64; 6])] = &[${bwe.join(", ")}];\n\n`;
p += `pub const BIOME_GEOMETRY: &[(f64, u32, [f64; 3])] = &[${bgeo.join(", ")}];\n\n`;
p += `pub const BIOME_AT: &[(f64, &str, f64, f64, [f64; 6], &[(usize, [u8; 3])])] = &[${bat.join(", ")}];\n\n`;
p += `pub const BLOOM_AT: &[(f64, u32, &str, f64)] = &[${bbloom.join(", ")}];\n\n`;
p += `pub const RESOLVE_BLOOM: &[(&str, [f64; 6], f64, f64)] = &[${bres.join(", ")}];\n\n`;

// ---------------------------------------------------------------- terrain
const TSEED = 662118718;
terrain.setBiomeGeometry(null);
terrain.setFjordStrength(null);
terrain.seedLayers(TSEED);
p += `pub const TERRAIN_SEED: u32 = ${TSEED}u32;\n`;
p += `pub const LAYER_SEEDS: [u32; 6] = [${terrain.LAYERS.map((l) => l._seed).join(", ")}];\n\n`;

const bh = [];
const rh = [];
const ri = [];
for (let li = 0; li < terrain.LAYERS.length; li++) {
  for (const wx of [-1500.0, -37.5, 0.0, 1234.5, 9876.0]) {
    bh.push(`(${li}, ${f(wx)}, ${f(terrain.bankHeight(terrain.LAYERS[li], wx))})`);
    rh.push(`(${li}, ${f(wx)}, ${f(terrain.ridgeHeight(terrain.LAYERS[li], wx))})`);
    ri.push(`(${li}, ${f(wx)}, ${f(terrain.riverInfluence(terrain.LAYERS[li], wx))})`);
  }
}
p += `pub const BANK_HEIGHT: &[(usize, f64, f64)] = &[${bh.join(", ")}];\n\n`;
p += `pub const RIDGE_HEIGHT: &[(usize, f64, f64)] = &[${rh.join(", ")}];\n\n`;
p += `pub const RIVER_INFLUENCE: &[(usize, f64, f64)] = &[${ri.join(", ")}];\n`;

// ------------------------------------------------------------------- sky
const meteor = [];
for (const seed of [12345, 662118718]) {
  for (let t = 0; t < 60; t += 0.25) {
    const m = sky.meteorAt(seed, t);
    const val = m
      ? `Some((${f(m.x)}, ${f(m.y)}, ${f(m.dx)}, ${f(m.dy)}, ${f(m.alpha)}))`
      : "None";
    meteor.push(`(${seed}u32, ${f(t)}, ${val})`);
  }
}
p += `pub const METEOR: &[(u32, f64, Option<(f64, f64, f64, f64, f64)>)] = &[${meteor.join(", ")}];\n\n`;

const cel = [];
for (const [hour, rise, set, moon] of [[0, 6.5, 18.5, 0.5], [6.5, 6.5, 18.5, 0.5], [12, 6.5, 18.5, 0.5], [15, 6.5, 18.5, 0.25], [18.5, 6.5, 18.5, 0.75], [22, 6.5, 18.5, 0.5], [7, 7.2, 18.1, 0.5]]) {
  const s = new sky.Sky(662118718);
  const cc = s.celestial(hour, 960, 270, rise, set, moon);
  const phase = cc.phase === undefined ? "None" : `Some(${f(cc.phase)})`;
  const illum = cc.illum === undefined ? "None" : `Some(${f(cc.illum)})`;
  cel.push(`(${f(hour)}, ${f(rise)}, ${f(set)}, ${f(moon)}, ${cc.isSun}, ${f(cc.u)}, ${f(cc.x)}, ${f(cc.y)}, ${f(cc.r)}, ${f(cc.horizonY)}, ${cc.rising}, ${cc.visible}, ${phase}, ${illum})`);
}
p += `pub const CELESTIAL: &[(f64, f64, f64, f64, bool, f64, f64, f64, f64, f64, bool, bool, Option<f64>, Option<f64>)] = &[${cel.join(", ")}];\n\n`;

const skyInit = [];
for (const seed of [662118718, 123, 0]) {
  const s = new sky.Sky(seed);
  const stars = s.stars.slice(0, 3).map((st) => `(${f(st.x)}, ${f(st.y)}, ${f(st.b)}, ${f(st.p)})`).join(", ");
  const clouds = s.clouds.slice(0, 3).map((c) => `(${f(c.x)}, ${f(c.y)}, ${f(c.w)}, ${f(c.s)})`).join(", ");
  skyInit.push(`(${seed}u32, [${stars}], [${clouds}])`);
}
p += `pub const SKY_INIT: &[(u32, [(f64, f64, f64, f64); 3], [(f64, f64, f64, f64); 3])] = &[${skyInit.join(", ")}];\n\n`;

const wl = [];
for (const t of ["wind", "storm", "snow", "rain", "fog", "clear", "otro"]) wl.push(`(${s(t)}, ${f(weather.windLevelFor(t))})`);
p += `pub const WIND_LEVEL: &[(&str, f64)] = &[${wl.join(", ")}];\n\n`;

const wspawn = [];
for (const type of ["snow", "rain", "storm", "wind"]) {
  const w = new weather.Weather(320, 180);
  w.setImmediate(type);
  const parts = w.particles.slice(0, 3).map((p) => `(${f(p.x)}, ${f(p.y)}, ${f(p.vx || 0)}, ${f(p.vy || 0)}, ${p.s || 0}i32, ${p.len || 0}i32, ${f(p.a)}, ${p.leaf || false}, ${p.color ? rgbArr(palette.hexToRgb(p.color)) : "[0, 0, 0]"})`).join(", ");
  wspawn.push(`(${s(type)}, &[${parts}])`);
}
p += `pub const WEATHER_SPAWN: &[(&str, &[(f64, f64, f64, f64, i32, i32, f64, bool, [u8; 3])])] = &[${wspawn.join(", ")}];\n`;
p += `pub const WEATHER_SIZE: (usize, usize) = (320, 180);\n`;

// ------------------------------------------------- momentos y vistas
const me = moments.momentEvents(TSEED, { x: 0 }, 960).map((e) => `(${s(e.type)}, ${f(e.wx)})`).join(", ");
p += `pub const MOMENT_EVENTS: &[(&str, f64)] = &[${me}];\n\n`;

const vd = [];
for (const q of [
  "seed=andes&hour=12.00&weather=snow&aspect=32:9&biome=norte&moment=condor&clock=fast",
  "x=123.6&season=otono",
  "weather=malo&moment=xyz&clock=slow",
  "hour=-1&x=abc",
  "aspect=2.4&bloom=on",
  "",
]) {
  const v = views.decodeView(q);
  vd.push(`(${s(q)}, ${optS(v.seed)}, ${optF(v.x)}, ${optF(v.hour)}, ${optS(v.weather)}, ${optS(v.aspect)}, ${optS(v.moment)}, ${optS(v.season)}, ${optS(v.biome)}, ${optS(v.bloom)}, ${optS(v.clock)})`);
}
p += `pub const VIEW_DECODE: &[(&str, Option<&str>, Option<f64>, Option<f64>, Option<&str>, Option<&str>, Option<&str>, Option<&str>, Option<&str>, Option<&str>, Option<&str>)] = &[${vd.join(", ")}];\n\n`;

const ve = [];
for (const st of [
  { seed: "andes", hour: 12, weather: "snow", aspect: "32:9", biome: "norte" },
  { x: 123.6, season: "otono" },
  { hour: -1, bloom: "on" },
  { weather: "malo" },
  { moment: "condor", clock: "fast" },
]) {
  ve.push(`(${optS(st.seed)}, ${optF(st.x)}, ${optF(st.hour)}, ${optS(st.weather)}, ${optS(st.aspect)}, ${optS(st.moment)}, ${optS(st.season)}, ${optS(st.biome)}, ${optS(st.bloom)}, ${optS(st.clock)}, ${s(views.encodeView(st))})`);
}
p += `pub const VIEW_ENCODE: &[(Option<&str>, Option<f64>, Option<f64>, Option<&str>, Option<&str>, Option<&str>, Option<&str>, Option<&str>, Option<&str>, Option<&str>, &str)] = &[${ve.join(", ")}];\n\n`;

// ------------------------------------------------- spawns de flora y fauna
const valle = terrain.LAYERS.find((l) => l.name === "valle");const flSpawns = flora.floraSpawns(valle, { x: 300 }, 480, 270, TSEED, null).slice(0, 6);
const fl = flSpawns.map((sp) => `(${f(sp.wx)}, ${f(sp.sx)}, ${sp.gy}i32, ${f(sp.size)}, ${s(sp.type)}, ${s(sp.warm)}, ${sp.depth}i32)`).join(", ");
p += `pub const FLORA_SPAWNS: &[(f64, f64, i32, f64, &str, bool, i32)] = &[${fl}];\n\n`;

const faSpawns = fauna.faunaSpawns(valle, { x: 300 }, 480, 270, TSEED, 12, null, null).slice(0, 6);
const fa = faSpawns
  .map((sp) => {
    const gy = sp.gy == null ? "None" : `Some(${f(sp.gy)})`;
    const fy = sp.flightY == null ? "None" : `Some(${f(sp.flightY)})`;
    return `(${f(sp.wx)}, ${f(sp.sx)}, ${gy}, ${fy}, ${s(sp.type)}, ${f(sp.phase)}, ${f(sp.dir)})`;
  })
  .join(", ");
p += `pub const FAUNA_SPAWNS: &[(f64, f64, Option<f64>, Option<f64>, &str, f64, f64)] = &[${fa}];\n\n`;

// ---------------------------------------------------------- actividad (D-052)
const SIGNAL_KEYS = ["cpu", "ram", "temp", "load"];
const act = [
  null,
  {},
  { cpu: 1, ram: 1 },
  { cpu: 0, ram: 0, temp: 55, load: 0 },
  { temp: 100, load: 4 },
  { cpu: 0.42, ram: 0.71, temp: 63, load: 1.8 },
  { cpu: 2, ram: -1, temp: -500, load: 1e6 },
  { cpu: NaN, ram: "x", temp: "y", load: null },
  { cpu: "0.5", ram: "0.5" },
].map((c) => {
  const exp = activity.activityMultiplier(c);
  if (c == null) return `(None, ${f(exp)})`;
  const vals = SIGNAL_KEYS.map((k) => (k in c ? Number(c[k]) : NaN));
  return `(Some((${vals.map(f).join(", ")})), ${f(exp)})`;
});
p += `pub const ACTIVITY: &[(Option<(f64, f64, f64, f64)>, f64)] = &[${act.join(", ")}];\n\n`;

// ---------------------------------------------------------------- fauna data
const rarityEntries = Object.keys(fauna.RARITY_WEIGHT).map((k) => `(${s(k)}, ${f(fauna.RARITY_WEIGHT[k])})`).join(", ");
let fd = "// Datos del núcleo Rust (fuente; editable a mano). Snapshot inicial derivado del JS con scripts/gen-parity.mjs.\n\n";
fd += `pub const RARITY_WEIGHT: &[(&str, f64)] = &[${rarityEntries}];\n\n`;
fd += `use crate::refs::RefRow;\n\n`;
fd += `pub struct SpeciesData {\n    pub name: &'static str,\n    pub movement: &'static str,\n    pub active: &'static str,\n    pub speed: f64,\n    pub range: f64,\n    pub fps: f64,\n    pub rarity: &'static str,\n    pub tracks: bool,\n    pub bloom: bool,\n    pub chunk_ok: bool,\n    pub river: Option<(f64, f64, u32)>,\n    pub anchor: &'static str,\n    pub common: &'static str,\n    pub sci: &'static str,\n    pub endemism: &'static str,\n    pub iucn: &'static str,\n    pub chile: &'static str,\n    pub notes: &'static str,\n    pub refs: &'static [RefRow],\n    pub zones: &'static [(&'static str, &'static str, f64)],\n    pub palette: &'static [(&'static str, &'static str)],\n    pub frames: &'static [&'static [&'static str]],\n}\n\n`;
const speciesNames = Object.keys(fauna.SPECIES);
fd += `pub const SPECIES: &[SpeciesData; ${speciesNames.length}] = &[\n`;
for (const name of speciesNames) {
  const sp = fauna.SPECIES[name];
  const zones = [];
  for (const biome in sp.zones) for (const layer in sp.zones[biome]) zones.push(`(${s(biome)}, ${s(layer)}, ${f(sp.zones[biome][layer])})`);
  const palettePairs = Object.keys(sp.palette).map((c) => `(${s(c)}, ${s(sp.palette[c])})`).join(", ");
  const frames = sp.frames.map((fr) => `&[${fr.map(s).join(", ")}]`).join(", ");
  const river = sp.placement?.river
    ? `Some((${f(sp.placement.river.chance)}, ${f(sp.placement.river.offset)}, ${sp.placement.river.seed >>> 0}u32))`
    : "None";
  const chunkOk = sp.placement?.chunk !== false;
  const anchor = sp.anchor || (sp.movement === "fly" || sp.movement === "flock" ? "center" : "ground");
  fd += `    SpeciesData {\n`;
  fd += `        name: ${s(name)}, movement: ${s(sp.movement)}, active: ${s(sp.active)}, speed: ${f(sp.speed)}, range: ${f(sp.range)}, fps: ${f(sp.fps)}, rarity: ${s(sp.rarity)},\n`;
  fd += `        tracks: ${!!sp.tracks}, bloom: ${!!sp.bloom}, chunk_ok: ${chunkOk}, river: ${river},\n`;
  fd += `        anchor: ${s(anchor)}, common: ${s(sp.common || "")}, sci: ${s(sp.sci || "")}, endemism: ${s(sp.endemism || "")}, iucn: ${s(sp.iucn || "")}, chile: ${s(sp.chile || "")},\n`;
  fd += `        notes: ${s(sp.notes || "")}, refs: ${refArr(sp)},\n`;
  fd += `        zones: &[${zones.join(", ")}],\n`;
  fd += `        palette: &[${palettePairs}],\n`;
  fd += `        frames: &[${frames}],\n`;
  fd += `    },\n`;
}
fd += "];\n";
writeFileSync(join(ROOT, "crates/andes-core/src/fauna_data.rs"), fd);

// ------------------------------------------------- flora metadata data
const floraNames = Object.keys(flora.FLORA);
let fm = "// Datos del núcleo Rust (fuente; editable a mano). Snapshot inicial derivado del JS con scripts/gen-parity.mjs.\n\n";
fm += "use crate::refs::RefRow;\n\n";
fm += `pub struct FloraMeta {\n    pub name: &'static str,\n    pub common: &'static str,\n    pub sci: &'static str,\n    pub endemism: &'static str,\n    pub height_m: &'static str,\n    pub notes: &'static str,\n    pub refs: &'static [RefRow],\n}\n\n`;
fm += `pub const FLORA_META: &[FloraMeta; ${floraNames.length}] = &[\n`;
for (const name of floraNames) {
  const d = flora.FLORA[name];
  fm += `    FloraMeta { name: ${s(name)}, common: ${s(d.common || "")}, sci: ${s(d.sci || "")}, endemism: ${s(d.endemism || "")}, height_m: ${s(d.heightM || "")}, notes: ${s(d.notes || "")}, refs: ${refArr(d)} },\n`;
}
fm += "];\n";
writeFileSync(join(ROOT, "crates/andes-core/src/flora_data.rs"), fm);

// --------------------------------------------------- paleta maestra (docs/14)
const palKeys = palette.KEYS.slice(0, -1); // la última duplica la medianoche
const hourLabel = (t) => {
  const total = Math.round(t * 24 * 60);
  const h = Math.floor(total / 60) % 24;
  const m = total % 60;
  return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}`;
};
const pl = [];
pl.push(`| Campo | ${palKeys.map((k) => hourLabel(k.t)).join(" | ")} |`);
pl.push(`|-------|${palKeys.map(() => "------").join("|")}|`);
for (const field of FIELDS) pl.push(`| \`${field}\` | ${palKeys.map((k) => k.p[field]).join(" | ")} |`);
const paletteDoc = join(ROOT, "docs", "14-paleta-maestra.md");
if (existsSync(paletteDoc)) {
  const src = readFileSync(paletteDoc, "utf8");
  const begin = "<!-- BEGIN:paleta-tabla";
  const end = "<!-- END:paleta-tabla -->";
  const i = src.indexOf(begin);
  const j = src.indexOf(end);
  if (i >= 0 && j > i) {
    const beginLineEnd = src.indexOf("-->", i) + 3;
    writeFileSync(paletteDoc, `${src.slice(0, i)}${src.slice(i, beginLineEnd)}\n${pl.join("\n")}\n${src.slice(j)}`);
  }
}

mkdirSync(join(ROOT, "crates/andes-core/tests/generated"), { recursive: true });
writeFileSync(join(ROOT, "crates/andes-core/tests/generated/parity_data.rs"), p);
console.log(`JS dir: ${JS}`);
console.log("escrito: crates/andes-core/src/palette_data.rs");
console.log("escrito: crates/andes-core/tests/parity_data.rs");
