//! Lectura de la actividad del equipo desde el sistema (Linux).
//!
//! Puerto nativo del puente de `src/activity.js` (D-052/D-055): el núcleo solo
//! recibe el multiplicador (`andes_core::activity`); aquí se leen `/proc` y
//! `/sys` en un hilo de fondo para no bloquear el render.

use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use andes_core::activity::ActivitySignals;

/// Muestrea CPU (delta de `/proc/stat`), RAM, carga y temperatura.
#[derive(Default)]
pub struct SensorSampler {
    last_total: u64,
    last_idle: u64,
    primed: bool,
}

impl SensorSampler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sample(&mut self) -> Option<ActivitySignals> {
        let cpu = self.cpu();
        let ram = read_ram();
        let load = read_load();
        let temp = read_temp();
        if cpu.is_none() && ram.is_none() && load.is_none() {
            return None;
        }
        Some(ActivitySignals {
            cpu: cpu.unwrap_or(f64::NAN),
            ram: ram.unwrap_or(f64::NAN),
            temp: temp.unwrap_or(f64::NAN),
            load: load.unwrap_or(f64::NAN),
        })
    }

    fn cpu(&mut self) -> Option<f64> {
        let s = fs::read_to_string("/proc/stat").ok()?;
        let line = s.lines().next()?;
        let mut it = line.split_whitespace();
        if it.next()? != "cpu" {
            return None;
        }
        let vals: Vec<u64> = it.filter_map(|v| v.parse().ok()).collect();
        if vals.len() < 4 {
            return None;
        }
        let idle = vals[3] + vals.get(4).copied().unwrap_or(0);
        let total: u64 = vals.iter().sum();
        let busy = if self.primed && total > self.last_total {
            let dt = (total - self.last_total) as f64;
            let di = idle.saturating_sub(self.last_idle) as f64;
            ((dt - di) / dt).clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.last_total = total;
        self.last_idle = idle;
        self.primed = true;
        Some(busy)
    }
}

fn read_ram() -> Option<f64> {
    let s = fs::read_to_string("/proc/meminfo").ok()?;
    let mut total = None;
    let mut avail = None;
    for line in s.lines() {
        if let Some(v) = line.strip_prefix("MemTotal:") {
            total = v
                .split_whitespace()
                .next()
                .and_then(|n| n.parse::<f64>().ok());
        } else if let Some(v) = line.strip_prefix("MemAvailable:") {
            avail = v
                .split_whitespace()
                .next()
                .and_then(|n| n.parse::<f64>().ok());
        }
        if total.is_some() && avail.is_some() {
            break;
        }
    }
    let (total, avail) = (total?, avail?);
    if total <= 0.0 {
        return None;
    }
    Some(((total - avail) / total).clamp(0.0, 1.0))
}

fn read_load() -> Option<f64> {
    let s = fs::read_to_string("/proc/loadavg").ok()?;
    s.split_whitespace().next()?.parse::<f64>().ok()
}

fn read_temp() -> Option<f64> {
    let dir = fs::read_dir("/sys/class/thermal").ok()?;
    let mut max: Option<f64> = None;
    for entry in dir.flatten() {
        let name = entry.file_name();
        if !name.to_string_lossy().starts_with("thermal_zone") {
            continue;
        }
        let Ok(raw) = fs::read_to_string(entry.path().join("temp")) else {
            continue;
        };
        let Ok(milli) = raw.trim().parse::<f64>() else {
            continue;
        };
        let celsius = milli / 1000.0;
        if celsius <= 0.0 || celsius > 200.0 {
            continue;
        }
        max = Some(max.map_or(celsius, |m| m.max(celsius)));
    }
    max
}

/// Sondea los sensores en un hilo de fondo y publica la última muestra.
pub struct ActivityPoller {
    rx: Receiver<Option<ActivitySignals>>,
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl ActivityPoller {
    pub fn start(interval: Duration) -> Self {
        let (tx, rx) = channel();
        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = stop.clone();
        let handle = thread::spawn(move || {
            let mut sampler = SensorSampler::new();
            let _ = sampler.sample();
            while !stop_thread.load(Ordering::Relaxed) {
                let mut waited = Duration::ZERO;
                while waited < interval {
                    if stop_thread.load(Ordering::Relaxed) {
                        return;
                    }
                    thread::sleep(Duration::from_millis(50));
                    waited += Duration::from_millis(50);
                }
                if tx.send(sampler.sample()).is_err() {
                    return;
                }
            }
        });
        Self {
            rx,
            stop,
            handle: Some(handle),
        }
    }

    /// Última muestra disponible (no bloquea): `Some(None)` = sensores caídos.
    pub fn poll(&self) -> Option<Option<ActivitySignals>> {
        self.rx.try_iter().last()
    }
}

impl Drop for ActivityPoller {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use andes_core::activity::activity_multiplier;

    #[test]
    fn sample_never_panics_and_stays_in_range() {
        let mut sampler = SensorSampler::new();
        let _ = sampler.sample(); // prima el delta de CPU
        if let Some(signals) = sampler.sample() {
            let m = activity_multiplier(Some(signals));
            assert!(
                (0.5..=1.8).contains(&m),
                "multiplicador fuera de rango: {m}"
            );
        }
    }
}
