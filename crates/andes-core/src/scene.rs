//! Orquestador de la escena: cámara, ciclo día/noche, clima y orden de render
//! (puerto parcial de `src/scene.js`).

use crate::biomes::{biome_at, resolve_bloom};
use crate::clock::{moon_phase_for_date, moon_phase_from_days, season_phase_for_date, DateTime};
use crate::fauna::place_fauna;
use crate::flora::place_flora;
use crate::framebuffer::Framebuffer;
use crate::moments::{moment_ground, moment_sky};
use crate::palette::{apply_biome, get_palette, lerp_palettes, night_amount};
use crate::rng::{seed_to_int, Mulberry32};
use crate::seasons::{
    apply_season, pick_season_weather, season_glow_tint, season_snow_shift, season_state,
    SeasonState, SEASON_DURATION, SEASON_STRENGTH,
};
use crate::sky::{Sky, RISE, SET};
use crate::sun::{solar_clock, solar_times};
use crate::terrain::{draw_sea, Terrain, LAYERS};
use crate::weather::Weather;

pub struct Scene {
    pub w: usize,
    pub h: usize,
    pub seed: u32,
    pub camera_x: f64,
    pub auto_scroll: bool,
    pub scroll_speed: f64,
    pub hour: f64,
    pub time_auto: bool,
    pub time_speed: f64,
    pub sky: Sky,
    pub weather: Weather,
    pub terrain: Terrain,
    pub moment: String,
    pub biome_mode: String,
    pub bloom_mode: String,
    pub season: String,
    pub season_phase: f64,
    pub season_speed: f64,
    pub activity_mul: f64,
    pub day_num: i64,
    pub t_sec: f64,
    pub lat: f64,
    pub longitude: f64,
    pub clock_fast: bool,
    pub now: Option<DateTime>,
    pub tz_hours: f64,
    pub weather_auto: bool,
    pub weather_timer: f64,
    weather_rng: Mulberry32,
}

impl Scene {
    pub fn new(seed: u32, w: usize, h: usize) -> Self {
        let mut terrain = Terrain::new(seed);
        terrain.set_biome_mode(Some("auto"));
        let mut scene = Self {
            w,
            h,
            seed,
            camera_x: 0.0,
            auto_scroll: true,
            scroll_speed: 30.0,
            hour: 8.0,
            time_auto: true,
            time_speed: 0.125,
            sky: Sky::new(seed),
            weather: Weather::new(w, h),
            terrain,
            moment: "auto".into(),
            biome_mode: "auto".into(),
            bloom_mode: "auto".into(),
            season: "auto".into(),
            season_phase: 0.0,
            season_speed: 1.0 / SEASON_DURATION,
            activity_mul: 1.0,
            day_num: 0,
            t_sec: 0.0,
            lat: -33.45,
            longitude: -70.66,
            clock_fast: true,
            now: None,
            tz_hours: 0.0,
            weather_auto: true,
            weather_timer: 12.0,
            weather_rng: Mulberry32::new(seed ^ 0x5eed),
        };
        scene.sync_snow_shift();
        scene
    }

    pub fn new_from_seed_str(seed: &str, w: usize, h: usize) -> Self {
        Self::new(seed_to_int(seed), w, h)
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        self.w = w;
        self.h = h;
        self.weather.resize(w, h);
    }

    fn season_state_now(&self) -> SeasonState {
        season_state(self.season_phase, &self.season)
    }

    fn sync_snow_shift(&mut self) {
        self.terrain.snow_shift_extra = season_snow_shift(self.season_state_now());
    }

    pub fn set_biome(&mut self, mode: &str) {
        self.biome_mode = mode.to_string();
        self.terrain.set_biome_mode(Some(mode));
    }

    pub fn set_bloom(&mut self, mode: &str) {
        self.bloom_mode = mode.to_string();
    }

    pub fn set_moment(&mut self, mode: &str) {
        self.moment = mode.to_string();
    }

    pub fn set_season(&mut self, mode: &str) {
        self.season = mode.to_string();
        self.sync_snow_shift();
    }

    pub fn set_clock_fast(&mut self, fast: bool) {
        self.clock_fast = fast;
    }

    /// Fija la hora local del equipo (reloj real) y el desfase horario.
    pub fn set_now(&mut self, now: DateTime, tz_hours: f64) {
        self.now = Some(now);
        self.tz_hours = tz_hours;
    }

    pub fn set_activity_mul(&mut self, mul: f64) {
        self.activity_mul = if mul.is_finite() && mul > 0.0 {
            mul
        } else {
            1.0
        };
    }

    pub fn set_weather_type(&mut self, type_: &'static str) {
        if type_ == "auto" {
            self.weather_auto = true;
            self.weather_timer = 0.5;
            return;
        }
        self.weather_auto = false;
        self.weather.set_immediate(type_, self.w, self.h);
    }

    pub fn moon_phase(&self) -> f64 {
        if self.clock_fast {
            moon_phase_from_days(self.day_num as f64)
        } else if let Some(now) = self.now {
            moon_phase_for_date(now.year, now.day_of_year())
        } else {
            moon_phase_from_days(self.day_num as f64)
        }
    }

    pub fn update(&mut self, dt: f64) {
        self.t_sec += dt;
        if self.auto_scroll {
            self.camera_x += self.scroll_speed * dt;
        }
        if self.clock_fast {
            if self.time_auto {
                self.hour += self.time_speed * dt;
                if self.hour >= 24.0 {
                    self.hour -= 24.0;
                    self.day_num += 1;
                }
            }
            if self.season == "auto" {
                self.season_phase = (self.season_phase + self.season_speed * dt) % 4.0;
                self.sync_snow_shift();
            }
        } else if let Some(now) = self.now {
            self.hour = now.local_hour();
            if self.season == "auto" {
                self.season_phase = season_phase_for_date(&now);
                self.sync_snow_shift();
            }
        }
        if self.weather_auto {
            self.weather_timer -= dt;
            if self.weather_timer <= 0.0 {
                let st = self.season_state_now();
                let mut pick = pick_season_weather(st, || self.weather_rng.next_f64());
                if pick == self.weather.type_ {
                    pick = if pick == "clear" { "wind" } else { "clear" };
                }
                self.weather.request(pick);
                self.weather_timer = 30.0 + self.weather_rng.next_f64() * 30.0;
            }
        }
        self.weather.update(dt);
    }

    pub fn palette_hour(&self) -> f64 {
        if !self.clock_fast {
            if let Some(now) = self.now {
                let s = solar_times(
                    now.day_of_year() as f64,
                    self.lat,
                    self.longitude,
                    self.tz_hours,
                );
                return solar_clock(self.hour, s.rise, s.set);
            }
        }
        self.hour
    }

    pub fn render(&mut self, fb: &mut Framebuffer) {
        let pal_hour = self.palette_hour();
        let night = night_amount(pal_hour);
        let season = self.season_state_now();
        let center = self.camera_x + self.w as f64 * 0.5;
        let biome = biome_at(center, self.seed, &self.biome_mode);
        let bloom = resolve_bloom(&self.bloom_mode, &biome.weights, biome.bloom);

        let wth = &self.weather;
        let mut pal = get_palette(pal_hour, wth.type_, wth.strength);
        if let Some(prev) = wth.prev_type {
            let prev_pal = get_palette(pal_hour, prev, wth.prev_strength);
            let total = wth.strength + wth.prev_strength;
            let k = if total < 0.001 {
                1.0
            } else {
                wth.strength / total
            };
            pal = lerp_palettes(&prev_pal, &pal, k);
        }
        pal = apply_season(&pal, season, SEASON_STRENGTH * (1.0 - 0.85 * night));
        let night_biome = 1.0 - 0.85 * night;
        pal = apply_biome(
            &pal,
            &biome.tint,
            biome.amount * night_biome,
            bloom * night_biome,
        );

        let moon = self.moon_phase();
        let cel = self
            .sky
            .celestial(pal_hour, self.w as f64, self.h as f64, RISE, SET, moon);
        let glow = season_glow_tint(season).map(|g| (g.color, g.amount));
        let moon_dim = if !cel.is_sun && cel.visible {
            cel.illum.unwrap_or(0.0)
        } else {
            0.0
        };

        fb.clear();
        self.sky
            .draw(fb, &pal, night, self.t_sec, Some(&cel), glow, moon_dim);
        self.sky.draw_body(fb, &cel, &pal);
        self.sky.draw_clouds(fb, &pal, self.camera_x, self.t_sec);
        moment_sky(
            fb,
            &pal,
            self.camera_x,
            self.w,
            self.h,
            self.seed,
            self.t_sec,
            &self.moment,
        );
        self.weather.draw_lightning(fb, &pal);

        for (i, layer) in LAYERS.iter().enumerate() {
            if layer.sea {
                let wind = self.weather.effective_wind();
                draw_sea(
                    fb,
                    &pal,
                    self.camera_x,
                    self.w,
                    self.h,
                    self.t_sec,
                    Some(cel.x),
                    wind,
                );
            } else {
                self.terrain
                    .draw_layer(i, fb, &pal, self.camera_x, self.w, self.h);
                place_flora(
                    fb,
                    layer,
                    &self.terrain,
                    i,
                    &pal,
                    self.camera_x,
                    self.w,
                    self.h,
                    self.seed,
                    self.t_sec,
                    &self.biome_mode,
                    &self.bloom_mode,
                    season.index as i32,
                    night,
                );
            }
            place_fauna(
                fb,
                layer,
                &self.terrain,
                i,
                &pal,
                self.camera_x,
                self.w,
                self.h,
                self.seed,
                self.hour,
                self.t_sec,
                &self.biome_mode,
                &self.bloom_mode,
                self.activity_mul,
            );
        }
        moment_ground(
            fb,
            &pal,
            self.camera_x,
            self.w,
            self.h,
            self.seed,
            self.t_sec,
            &self.moment,
            &self.terrain,
        );
        self.weather.draw(fb, &pal);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_renders_and_is_deterministic() {
        let mut a = Scene::new(seed_to_int("andes"), 480, 270);
        let mut b = Scene::new(seed_to_int("andes"), 480, 270);
        a.set_weather_type("clear");
        b.set_weather_type("clear");
        let mut fa = Framebuffer::new(480, 270);
        let mut fb = Framebuffer::new(480, 270);
        a.render(&mut fa);
        b.render(&mut fb);
        assert_eq!(fa.px, fb.px);
        assert!(fa.px.iter().any(|p| *p != 0));
    }

    #[test]
    fn day_and_night_differ() {
        let mut s = Scene::new(seed_to_int("andes"), 480, 270);
        s.set_weather_type("clear");
        let mut day = Framebuffer::new(480, 270);
        s.hour = 12.0;
        s.render(&mut day);
        let mut night = Framebuffer::new(480, 270);
        s.hour = 1.0;
        s.render(&mut night);
        assert_ne!(day.px, night.px);
    }

    #[test]
    fn scene_advances_camera_and_time() {
        let mut s = Scene::new(seed_to_int("andes"), 480, 270);
        let x0 = s.camera_x;
        let h0 = s.hour;
        s.update(1.0);
        assert!(s.camera_x > x0);
        assert!(s.hour > h0);
    }

    #[test]
    fn real_clock_sets_hour_and_season() {
        let mut s = Scene::new(seed_to_int("andes"), 480, 270);
        s.set_clock_fast(false);
        s.set_now(DateTime::new(2026, 0, 15, 14, 30, 0, 0), -3.0);
        s.update(0.1);
        assert!((s.hour - 14.5).abs() < 1e-9, "hour real: {}", s.hour);
        assert!(
            (0.0..1.0).contains(&s.season_phase),
            "fase de verano: {}",
            s.season_phase
        );
        let ph = s.palette_hour();
        assert!((0.0..24.0).contains(&ph), "palette_hour: {ph}");
    }
}
