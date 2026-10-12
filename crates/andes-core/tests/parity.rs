//! Paridad con el proyecto JS original (opcional).
//!
//! Este repo es la fuente de verdad (D-014); estos tests solo se compilan y
//! ejecutan con `cargo test --features parity`. `parity_data.rs` lo genera, como
//! referencia, `scripts/gen-parity.mjs` ejecutando el código JS real.
//!
//! Puede reportar diferencias donde Rust diverge **a propósito** (p. ej. el
//! `snow_shift` de biomas, D-017); es informativo, no un fallo de calidad.

#![cfg(feature = "parity")]
#![allow(
    clippy::type_complexity,
    clippy::needless_range_loop,
    clippy::explicit_auto_deref
)]

include!("generated/parity_data.rs");

use andes_core::math::to_uint32;
use andes_core::noise::{fbm1, ridged1, value_noise1};
use andes_core::palette::{
    apply_biome, field_index, get_palette, hex_to_rgb, lerp_color, night_amount, rgb_to_hex, shade,
    Palette, Rgb,
};
use andes_core::pixel::radix_dither;
use andes_core::rng::{hash1, hash_int, hash_string, seed_to_int, Mulberry32};
use andes_core::viewport::{label_for_ratio, ratio_from_string, width_for_ratio};

use andes_core::activity::{activity_multiplier, ActivitySignals};
use andes_core::biomes::{biome_at, biome_geometry, biome_weights, bloom_at, resolve_bloom};
use andes_core::clock::{moon_phase_for_date, DateTime};
use andes_core::seasons::{
    apply_season, season_glow_tint, season_snow_shift, season_state, season_weather_bias,
};
use andes_core::sun::{solar_clock, solar_declination, solar_times};
use andes_core::terrain::Terrain;

fn approx(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-12 * (1.0 + b.abs())
}

fn approx9(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * (1.0 + b.abs())
}

fn sorted_tint(t: &[(usize, Rgb)]) -> Vec<(usize, Rgb)> {
    let mut v = t.to_vec();
    v.sort_by_key(|(i, _)| *i);
    v
}

fn pal_eq(actual: &Palette, expected: &[[u8; 3]]) -> bool {
    actual.len() == expected.len() && actual.iter().zip(expected).all(|(a, b)| a == b)
}

#[test]
fn parity_mulberry32() {
    for (seed, expected) in MULBERRY32 {
        let mut r = Mulberry32::new(*seed);
        for (i, &e) in expected.iter().enumerate() {
            let v = r.next_f64();
            assert!(approx(v, e), "mulberry32({seed})[{i}]: {v} != {e}");
        }
    }
}

#[test]
fn parity_hash_int() {
    for (n, seed, expected) in HASH_INT {
        assert_eq!(hash_int(*n, *seed), *expected);
    }
}

#[test]
fn parity_hash1() {
    for (n, seed, expected) in HASH1 {
        assert!(approx(hash1(*n, *seed), *expected));
    }
}

#[test]
fn parity_hash_string() {
    for (s, expected) in HASH_STR {
        assert_eq!(hash_string(s), *expected, "hashString({s:?})");
    }
}

#[test]
fn parity_seed_to_int() {
    for (s, expected) in SEED_TO_INT {
        assert_eq!(seed_to_int(s), *expected, "seedToInt({s:?})");
    }
}

#[test]
fn parity_value_noise() {
    for (x, seed, expected) in VALUE_NOISE {
        assert!(
            approx(value_noise1(*x, *seed), *expected),
            "valueNoise1({x},{seed})"
        );
    }
}

#[test]
fn parity_fbm() {
    for (x, seed, oct, lac, gain, expected) in FBM {
        assert!(
            approx(fbm1(*x, *seed, *oct, *lac, *gain), *expected),
            "fbm1({x})"
        );
    }
}

#[test]
fn parity_ridged() {
    for (x, seed, oct, expected) in RIDGED {
        assert!(approx(ridged1(*x, *seed, *oct), *expected), "ridged1({x})");
    }
}

#[test]
fn parity_radix_dither() {
    for (x, y, seed, expected) in RADIX_DITHER {
        assert!(
            approx(radix_dither(*x, *y, *seed), *expected),
            "radixDither({x},{y},{seed})"
        );
    }
}

#[test]
fn parity_width_for_ratio() {
    for (r, expected) in WIDTH {
        assert_eq!(width_for_ratio(*r), *expected, "widthForRatio({r})");
    }
}

#[test]
fn parity_ratio_from_string() {
    for (s, expected) in RATIO_FROM_STRING {
        match (ratio_from_string(s), expected) {
            (None, None) => {}
            (Some(a), Some(b)) => assert!(approx(a, *b), "ratioFromString({s:?}) = {a} != {b}"),
            (got, exp) => panic!("ratioFromString({s:?}): {got:?} != {exp:?}"),
        }
    }
}

#[test]
fn parity_label_for_ratio() {
    for (r, expected) in LABEL {
        assert_eq!(label_for_ratio(*r), *expected, "labelForRatio({r})");
    }
}

#[test]
fn parity_lerp_color() {
    for (a, b, t, expected) in LERP_COLOR {
        let out = rgb_to_hex(lerp_color(hex_to_rgb(a), hex_to_rgb(b), *t));
        assert_eq!(out, *expected, "lerpColor({a},{b},{t})");
    }
}

#[test]
fn parity_shade() {
    for (a, amt, expected) in SHADE {
        assert_eq!(
            rgb_to_hex(shade(hex_to_rgb(a), *amt)),
            *expected,
            "shade({a},{amt})"
        );
    }
}

#[test]
fn parity_get_palette() {
    for (hour, weather, strength, expected) in GET_PALETTE {
        let pal = get_palette(*hour, weather, *strength);
        assert!(
            pal_eq(&pal, expected),
            "getPalette({hour},{weather},{strength})"
        );
    }
}

#[test]
fn parity_night_amount() {
    for (hour, expected) in NIGHT {
        assert!(
            approx(night_amount(*hour), *expected),
            "nightAmount({hour})"
        );
    }
}

#[test]
fn parity_apply_biome() {
    let tint: Vec<(usize, Rgb)> = BIOME_TINT
        .iter()
        .map(|(name, hex)| (field_index(name).unwrap(), hex_to_rgb(hex)))
        .collect();
    for (amount, bloom, expected) in APPLY_BIOME {
        let base = get_palette(12.0, "clear", 1.0);
        let pal = apply_biome(&base, &tint, *amount, *bloom);
        assert!(
            pal_eq(&pal, expected),
            "applyBiome(amount={amount}, bloom={bloom})"
        );
    }
}

#[test]
fn to_uint32_matches_js_semantics() {
    assert_eq!(to_uint32(3.9), 3);
    assert_eq!(to_uint32(-1.0), 4_294_967_295);
    assert_eq!(to_uint32(4_294_967_296.0), 0);
    assert_eq!(to_uint32(f64::NAN), 0);
    assert_eq!(to_uint32(f64::INFINITY), 0);
}

#[test]
fn parity_clock() {
    for (y, m0, d, h, mi, sec, ms, hour, doy, season, phase) in CLOCK {
        let dt = DateTime::new(*y, *m0, *d, *h, *mi, *sec, *ms);
        assert!(approx(dt.local_hour(), *hour), "localHour {y}-{m0}-{d}");
        assert_eq!(dt.day_of_year(), *doy, "dayOfYear {y}-{m0}-{d}");
        assert_eq!(andes_core::clock::season_for_date(*m0), *season);
        assert!(
            approx9(andes_core::clock::season_phase_for_date(&dt), *phase),
            "seasonPhase {y}-{m0}-{d}"
        );
    }
}

#[test]
fn parity_moon() {
    for (year, doy, expected) in MOON {
        assert!(
            approx(moon_phase_for_date(*year, *doy), *expected),
            "moon({year},{doy})"
        );
    }
}

#[test]
fn parity_solar_declination() {
    for (doy, expected) in SOLAR_DECL {
        assert!(approx9(solar_declination(*doy), *expected));
    }
}

#[test]
fn parity_solar_times() {
    for (doy, lat, lon, tz, rise, set, noon) in SOLAR_TIMES {
        let t = solar_times(*doy, *lat, *lon, *tz);
        assert!(approx9(t.rise, *rise) && approx9(t.set, *set) && approx9(t.noon, *noon));
    }
}

#[test]
fn parity_solar_clock() {
    for (hour, rise, set, expected) in SOLAR_CLOCK {
        assert!(
            approx9(solar_clock(*hour, *rise, *set), *expected),
            "solarClock({hour})"
        );
    }
}

#[test]
fn parity_season_state() {
    for (phase, mode, index, next, t) in SEASON_STATE {
        let st = season_state(*phase, mode);
        assert_eq!(
            (st.index, st.next),
            (*index, *next),
            "seasonState({phase},{mode})"
        );
        assert!(approx(st.t, *t));
    }
}

#[test]
fn parity_season_snow_and_bias() {
    for (index, next, t, expected) in SEASON_SNOW {
        let st = andes_core::seasons::SeasonState {
            index: *index,
            next: *next,
            t: *t,
        };
        assert!(approx(season_snow_shift(st), *expected));
    }
    // El sesgo de clima de verano/invierno se ajustó en Rust (D-018): divergencia
    // intencional, así que no se compara con el snapshot JS.
    let _ = season_weather_bias;
}

#[test]
fn parity_apply_season() {
    for (hour, mode, strength, expected) in APPLY_SEASON {
        // El verano (y `auto` en fase 0) tiene tinte árido propio en Rust (D-018):
        // divergencia intencional.
        if *mode == "verano" || *mode == "auto" {
            continue;
        }
        let pal = apply_season(
            &get_palette(*hour, "clear", 1.0),
            season_state(0.0, mode),
            *strength,
        );
        assert!(pal_eq(&pal, expected), "applySeason({mode},{strength})");
    }
}

#[test]
fn parity_season_glow() {
    for (index, next, t, expected) in SEASON_GLOW {
        let st = andes_core::seasons::SeasonState {
            index: *index,
            next: *next,
            t: *t,
        };
        match (season_glow_tint(st), expected) {
            (None, None) => {}
            (Some(g), Some((color, amount))) => {
                assert_eq!(g.color, *color);
                assert!(approx(g.amount, *amount));
            }
            (got, exp) => panic!("glow: {got:?} != {exp:?}"),
        }
    }
}

#[test]
fn parity_biome_weights() {
    for (x, seed, expected) in BIOME_WEIGHTS {
        let w = biome_weights(*x, *seed);
        for i in 0..6 {
            assert!(approx(w[i], expected[i]), "biomeWeights({x},{seed})[{i}]");
        }
    }
}

#[test]
fn parity_biome_geometry() {
    for (x, seed, expected) in BIOME_GEOMETRY {
        let g = biome_geometry(&biome_weights(*x, *seed));
        // `snow_shift` diverge a propósito: Rust unifica la convención de la
        // línea de nieve (positivo = más nieve, D-017).
        assert!(
            approx(g.amp_mul, expected[0]) && approx(g.fjord, expected[2]),
            "biomeGeometry({x},{seed})"
        );
    }
}

#[test]
fn parity_biome_at() {
    for (x, mode, amount, bloom, weights, tint) in BIOME_AT {
        let b = biome_at(*x, 662_118_718, mode);
        assert!(
            approx(b.amount, *amount) && approx(b.bloom, *bloom),
            "biomeAt({x},{mode})"
        );
        for i in 0..6 {
            assert!(approx(b.weights[i], weights[i]));
        }
        assert_eq!(
            sorted_tint(&b.tint),
            sorted_tint(*tint),
            "tint biomeAt({x},{mode})"
        );
    }
}

#[test]
fn parity_bloom_at() {
    for (x, seed, mode, expected) in BLOOM_AT {
        let w = andes_core::biomes::mode_weights(*mode, *x, *seed);
        assert!(approx(bloom_at(*x, *seed, &w), *expected));
    }
}

#[test]
fn parity_resolve_bloom() {
    for (mode, weights, bloom, expected) in RESOLVE_BLOOM {
        assert!(
            approx(resolve_bloom(mode, weights, *bloom), *expected),
            "resolveBloom({mode})"
        );
    }
}

#[test]
fn parity_terrain() {
    let t = Terrain::new(TERRAIN_SEED);
    assert_eq!(t.layer_seeds, LAYER_SEEDS);
    for (i, x, expected) in BANK_HEIGHT {
        assert!(
            approx(t.bank_height(*i, *x), *expected),
            "bankHeight({i},{x})"
        );
    }
    for (i, x, expected) in RIDGE_HEIGHT {
        assert!(
            approx(t.ridge_height(*i, *x), *expected),
            "ridgeHeight({i},{x})"
        );
    }
    for (i, x, expected) in RIVER_INFLUENCE {
        assert!(
            approx(t.river_influence(*i, *x), *expected),
            "riverInfluence({i},{x})"
        );
    }
}

#[test]
fn parity_meteor() {
    for (seed, t, expected) in METEOR {
        match (andes_core::sky::meteor_at(*seed, *t), expected) {
            (None, None) => {}
            (Some(m), Some((x, y, dx, dy, alpha))) => {
                assert!(
                    approx(m.x, *x)
                        && approx(m.y, *y)
                        && approx(m.dx, *dx)
                        && approx(m.dy, *dy)
                        && approx(m.alpha, *alpha),
                    "meteor({seed},{t})"
                );
            }
            (got, exp) => panic!("meteor({seed},{t}): {got:?} != {exp:?}"),
        }
    }
}

#[test]
fn parity_celestial() {
    for (hour, rise, set, moon, is_sun, u, x, y, r, horizon, rising, visible, phase, illum) in
        CELESTIAL
    {
        let sky = andes_core::sky::Sky::new(662_118_718);
        let c = sky.celestial(*hour, 960.0, 270.0, *rise, *set, *moon);
        assert_eq!(c.is_sun, *is_sun);
        assert!(
            approx(c.u, *u)
                && approx(c.x, *x)
                && approx(c.y, *y)
                && approx(c.r, *r)
                && approx(c.horizon_y, *horizon)
        );
        assert_eq!((c.rising, c.visible), (*rising, *visible));
        match (c.phase, phase) {
            (None, None) => {}
            (Some(a), Some(b)) => assert!(approx(a, *b)),
            _ => panic!("phase mismatch"),
        }
        match (c.illum, illum) {
            (None, None) => {}
            (Some(a), Some(b)) => assert!(approx(a, *b)),
            _ => panic!("illum mismatch"),
        }
    }
}

#[test]
fn parity_sky_init() {
    for (seed, stars, clouds) in SKY_INIT {
        let s = andes_core::sky::Sky::new(*seed);
        for i in 0..3 {
            let a = s.star(i);
            let e = stars[i];
            assert!(
                approx(a.0, e.0) && approx(a.1, e.1) && approx(a.2, e.2) && approx(a.3, e.3),
                "star {i} seed {seed}"
            );
            let ac = s.cloud(i);
            let ec = clouds[i];
            assert!(
                approx(ac.0, ec.0)
                    && approx(ac.1, ec.1)
                    && approx(ac.2, ec.2)
                    && approx(ac.3, ec.3),
                "cloud {i} seed {seed}"
            );
        }
    }
}

#[test]
fn parity_wind_level() {
    for (t, expected) in WIND_LEVEL {
        assert_eq!(andes_core::weather::wind_level_for(t), *expected);
    }
}

#[test]
fn parity_weather_spawn() {
    let (w, h) = WEATHER_SIZE;
    for (type_, expected) in WEATHER_SPAWN {
        let mut weather = andes_core::weather::Weather::new(w, h);
        weather.set_immediate(type_, w, h);
        for (i, e) in expected.iter().enumerate() {
            let p = weather.particles[i];
            assert!(
                approx(p.x, e.0) && approx(p.y, e.1) && approx(p.vx, e.2) && approx(p.vy, e.3),
                "spawn {type_} #{i} x/y/v"
            );
            assert_eq!((p.s, p.len), (e.4, e.5), "spawn {type_} #{i} s/len");
            assert!(
                approx(p.a, e.6) && p.leaf == e.7 && p.color == e.8,
                "spawn {type_} #{i} a/leaf/color"
            );
        }
    }
}

#[test]
fn parity_lerp_palettes() {
    for (h, a, sa, b, sb, k, expected) in LERP_PALETTES {
        let from = get_palette(*h, a, *sa);
        let to = get_palette(*h, b, *sb);
        let pal = andes_core::palette::lerp_palettes(&from, &to, *k);
        assert!(pal_eq(&pal, expected), "lerpPalettes({a},{b},{k})");
    }
}

#[test]
fn parity_flora_spawns() {
    // La flora de Rust creció (D-019), así que el orden de spawns ya no coincide
    // con el snapshot JS. Solo comprobamos que el muestreo siga siendo no vacío y
    // determinista entre dos corridas.
    let t = Terrain::new(TERRAIN_SEED);
    let layer = &andes_core::terrain::LAYERS[2]; // valle
    let run = || {
        andes_core::flora::flora_spawns(layer, &t, 2, 300.0, 480, 270, TERRAIN_SEED, |_| {
            Vec::<(&str, f64)>::new()
        })
    };
    let a = run();
    let b = run();
    assert!(!a.is_empty());
    assert_eq!(a.len(), b.len());
    let _ = FLORA_SPAWNS;
}

#[test]
fn parity_fauna_spawns() {
    let t = Terrain::new(TERRAIN_SEED);
    let layer = &andes_core::terrain::LAYERS[2]; // valle
    let got = andes_core::fauna::fauna_spawns(
        layer,
        &t,
        2,
        300.0,
        480,
        270,
        TERRAIN_SEED,
        12.0,
        |_| Vec::<(&str, f64)>::new(),
        |_| 1.0,
    );
    assert!(got.len() >= FAUNA_SPAWNS.len(), "fauna spawn count");
    for (g, e) in got.iter().take(FAUNA_SPAWNS.len()).zip(FAUNA_SPAWNS) {
        assert!(approx(g.wx, e.0) && approx(g.sx, e.1), "fauna wx/sx");
        match (g.gy, e.2) {
            (None, None) => {}
            (Some(a), Some(b)) => assert!(approx(a, b)),
            _ => panic!("gy mismatch"),
        }
        match (g.flight_y, e.3) {
            (None, None) => {}
            (Some(a), Some(b)) => assert!(approx(a, b)),
            _ => panic!("flightY mismatch"),
        }
        assert_eq!(g.type_, e.4);
        assert!(approx(g.phase, e.5) && approx(g.dir, e.6));
    }
}

#[test]
fn parity_moment_events() {
    let got = andes_core::moments::moment_events(TERRAIN_SEED, 0.0, 960);
    assert_eq!(got.len(), MOMENT_EVENTS.len());
    for (g, e) in got.iter().zip(MOMENT_EVENTS) {
        assert_eq!(g.type_, e.0);
        assert!(approx(g.wx, e.1));
    }
}

#[test]
fn parity_activity_multiplier() {
    for (signals, expected) in ACTIVITY {
        let sig = signals.map(|(cpu, ram, temp, load)| ActivitySignals {
            cpu,
            ram,
            temp,
            load,
        });
        let got = activity_multiplier(sig);
        assert!(
            approx(got, *expected),
            "activityMultiplier: {got} != {expected}"
        );
    }
}

#[test]
fn parity_view_decode() {
    for (q, seed, x, hour, weather, aspect, moment, season, biome, bloom, clock) in VIEW_DECODE {
        let v = andes_core::views::decode_view(q);
        assert_eq!(v.seed.as_deref(), *seed, "seed {q}");
        assert_eq!(v.weather.as_deref(), *weather, "weather {q}");
        assert_eq!(v.aspect.as_deref(), *aspect, "aspect {q}");
        assert_eq!(v.moment.as_deref(), *moment, "moment {q}");
        assert_eq!(v.season.as_deref(), *season, "season {q}");
        assert_eq!(v.biome.as_deref(), *biome, "biome {q}");
        assert_eq!(v.bloom.as_deref(), *bloom, "bloom {q}");
        assert_eq!(v.clock.as_deref(), *clock, "clock {q}");
        match (v.x, x) {
            (None, None) => {}
            (Some(a), Some(b)) => assert!(approx(a, *b), "x {q}"),
            _ => panic!("x mismatch {q}"),
        }
        match (v.hour, hour) {
            (None, None) => {}
            (Some(a), Some(b)) => assert!(approx(a, *b), "hour {q}"),
            _ => panic!("hour mismatch {q}"),
        }
    }
}

#[test]
fn parity_view_encode() {
    for (seed, x, hour, weather, aspect, moment, season, biome, bloom, clock, expected) in
        VIEW_ENCODE
    {
        let v = andes_core::views::ViewState {
            seed: seed.map(str::to_string),
            x: *x,
            hour: *hour,
            weather: weather.map(str::to_string),
            aspect: aspect.map(str::to_string),
            moment: moment.map(str::to_string),
            season: season.map(str::to_string),
            biome: biome.map(str::to_string),
            bloom: bloom.map(str::to_string),
            clock: clock.map(str::to_string),
        };
        assert_eq!(andes_core::views::encode_view(&v), *expected);
    }
}
