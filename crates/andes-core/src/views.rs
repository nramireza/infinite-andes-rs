//! Vistas: serialización del estado de la escena para enlaces y favoritos
//! (puerto de `src/views.js`). Todo puro, sin DOM.

use crate::viewport::ratio_from_string;

pub const VIEW_KEYS: [&str; 10] = [
    "seed", "x", "hour", "weather", "aspect", "moment", "season", "biome", "bloom", "clock",
];
pub const VIEW_WEATHERS: [&str; 7] = ["auto", "clear", "snow", "rain", "fog", "wind", "storm"];
pub const VIEW_MOMENTS: [&str; 8] = [
    "auto", "none", "18sep", "leorey", "kungleo", "condor", "bandada", "manada",
];
pub const VIEW_SEASONS: [&str; 5] = ["auto", "verano", "otono", "invierno", "primavera"];
pub const VIEW_BIOMES: [&str; 7] = [
    "auto",
    "altiplano",
    "norte",
    "centro",
    "sur",
    "patagonia",
    "austral",
];
pub const VIEW_BLOOMS: [&str; 3] = ["auto", "on", "off"];
pub const VIEW_CLOCKS: [&str; 2] = ["real", "fast"];

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ViewState {
    pub seed: Option<String>,
    pub x: Option<f64>,
    pub hour: Option<f64>,
    pub weather: Option<String>,
    pub aspect: Option<String>,
    pub moment: Option<String>,
    pub season: Option<String>,
    pub biome: Option<String>,
    pub bloom: Option<String>,
    pub clock: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Favorite {
    pub name: String,
    pub query: String,
}

fn clamp_hour(h: f64) -> f64 {
    ((h % 24.0) + 24.0) % 24.0
}

pub fn decode_view(query: &str) -> ViewState {
    let q = query.strip_prefix('?').unwrap_or(query);
    let pairs = parse_query(q);
    let get = |k: &str| pairs.iter().find(|(pk, _)| pk == k).map(|(_, v)| v.clone());
    let mut view = ViewState::default();

    if let Some(seed) = get("seed") {
        if !seed.is_empty() {
            view.seed = Some(seed);
        }
    }
    if let Some(x) = get("x")
        .and_then(|v| v.parse::<f64>().ok())
        .filter(|v| v.is_finite())
    {
        view.x = Some(x.round());
    }
    if let Some(hour) = get("hour")
        .and_then(|v| v.parse::<f64>().ok())
        .filter(|v| v.is_finite())
    {
        view.hour = Some(clamp_hour(hour));
    }
    if let Some(w) = get("weather").filter(|w| VIEW_WEATHERS.contains(&w.as_str())) {
        view.weather = Some(w);
    }
    if let Some(a) = get("aspect").filter(|a| ratio_from_string(a).is_some()) {
        view.aspect = Some(a.trim().to_string());
    }
    if let Some(m) = get("moment").filter(|m| VIEW_MOMENTS.contains(&m.as_str())) {
        view.moment = Some(m);
    }
    if let Some(s) = get("season").filter(|s| VIEW_SEASONS.contains(&s.as_str())) {
        view.season = Some(s);
    }
    if let Some(b) = get("biome").filter(|b| VIEW_BIOMES.contains(&b.as_str())) {
        view.biome = Some(b);
    }
    if let Some(b) = get("bloom").filter(|b| VIEW_BLOOMS.contains(&b.as_str())) {
        view.bloom = Some(b);
    }
    if let Some(c) = get("clock").filter(|c| VIEW_CLOCKS.contains(&c.as_str())) {
        view.clock = Some(c);
    }
    view
}

pub fn encode_view(view: &ViewState) -> String {
    let mut p: Vec<(String, String)> = Vec::new();
    if let Some(seed) = &view.seed {
        if !seed.is_empty() {
            p.push(("seed".into(), seed.clone()));
        }
    }
    if let Some(x) = view.x.filter(|x| x.is_finite()) {
        p.push(("x".into(), format!("{}", x.round() as i64)));
    }
    if let Some(hour) = view.hour.filter(|h| h.is_finite()) {
        p.push(("hour".into(), format!("{:.2}", clamp_hour(hour))));
    }
    if let Some(w) = view
        .weather
        .as_deref()
        .filter(|w| VIEW_WEATHERS.contains(w))
    {
        p.push(("weather".into(), w.into()));
    }
    if let Some(a) = view
        .aspect
        .as_deref()
        .filter(|a| ratio_from_string(a).is_some())
    {
        p.push(("aspect".into(), a.trim().into()));
    }
    if let Some(m) = view.moment.as_deref().filter(|m| VIEW_MOMENTS.contains(m)) {
        p.push(("moment".into(), m.into()));
    }
    if let Some(s) = view.season.as_deref().filter(|s| VIEW_SEASONS.contains(s)) {
        p.push(("season".into(), s.into()));
    }
    if let Some(b) = view.biome.as_deref().filter(|b| VIEW_BIOMES.contains(b)) {
        p.push(("biome".into(), b.into()));
    }
    if let Some(b) = view.bloom.as_deref().filter(|b| VIEW_BLOOMS.contains(b)) {
        p.push(("bloom".into(), b.into()));
    }
    if let Some(c) = view.clock.as_deref().filter(|c| VIEW_CLOCKS.contains(c)) {
        p.push(("clock".into(), c.into()));
    }
    p.iter()
        .map(|(k, v)| format!("{k}={}", url_encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}

pub fn add_view(list: &[Favorite], view: &ViewState, name: &str) -> Vec<Favorite> {
    let entry = Favorite {
        name: name.trim().to_string(),
        query: encode_view(view),
    };
    let mut out: Vec<Favorite> = list
        .iter()
        .filter(|e| e.name != entry.name)
        .cloned()
        .collect();
    out.push(entry);
    out
}

pub fn remove_view(list: &[Favorite], name: &str) -> Vec<Favorite> {
    list.iter().filter(|e| e.name != name).cloned().collect()
}

/// JSON de localStorage -> lista válida (tolera datos corruptos).
pub fn parse_views(raw: &str) -> Vec<Favorite> {
    if raw.is_empty() {
        return Vec::new();
    }
    let mut p = JsonParser {
        b: raw.as_bytes(),
        i: 0,
    };
    let Some(v) = p.parse() else {
        return Vec::new();
    };
    let Json::Arr(items) = v else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for it in items {
        if let Json::Obj(fields) = it {
            let name = fields
                .iter()
                .find(|(k, _)| k == "name")
                .and_then(|(_, v)| as_str(v));
            let query = fields
                .iter()
                .find(|(k, _)| k == "query")
                .and_then(|(_, v)| as_str(v));
            if let (Some(n), Some(q)) = (name, query) {
                out.push(Favorite { name: n, query: q });
            }
        }
    }
    out
}

fn as_str(v: &Json) -> Option<String> {
    if let Json::Str(s) = v {
        Some(s.clone())
    } else {
        None
    }
}

fn parse_query(q: &str) -> Vec<(String, String)> {
    q.split('&')
        .filter(|p| !p.is_empty())
        .map(|part| {
            let (k, v) = part.split_once('=').unwrap_or((part, ""));
            (url_decode(k), url_decode(v))
        })
        .collect()
}

fn url_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < b.len() => {
                let hex = std::str::from_utf8(&b[i + 1..i + 3])
                    .ok()
                    .and_then(|h| u8::from_str_radix(h, 16).ok());
                match hex {
                    Some(v) => {
                        out.push(v);
                        i += 3;
                    }
                    None => {
                        out.push(b'%');
                        i += 1;
                    }
                }
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'*' | b'-' | b'.' | b'_' => {
                out.push(b as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

// ------------------------------------------------------- mini parser JSON
#[allow(dead_code)]
enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

struct JsonParser<'a> {
    b: &'a [u8],
    i: usize,
}

impl JsonParser<'_> {
    fn parse(&mut self) -> Option<Json> {
        self.ws();
        self.value()
    }

    fn ws(&mut self) {
        while self.i < self.b.len() && self.b[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }

    fn value(&mut self) -> Option<Json> {
        self.ws();
        match *self.b.get(self.i)? {
            b'{' => self.obj(),
            b'[' => self.arr(),
            b'"' => self.string().map(Json::Str),
            b't' => self.lit("true").map(|_| Json::Bool(true)),
            b'f' => self.lit("false").map(|_| Json::Bool(false)),
            b'n' => self.lit("null").map(|_| Json::Null),
            _ => self.number(),
        }
    }

    fn lit(&mut self, s: &str) -> Option<()> {
        if self.b[self.i..].starts_with(s.as_bytes()) {
            self.i += s.len();
            Some(())
        } else {
            None
        }
    }

    fn obj(&mut self) -> Option<Json> {
        self.i += 1;
        let mut fields = Vec::new();
        self.ws();
        if self.b.get(self.i) == Some(&b'}') {
            self.i += 1;
            return Some(Json::Obj(fields));
        }
        loop {
            self.ws();
            let key = self.string()?;
            self.ws();
            if self.b.get(self.i) != Some(&b':') {
                return None;
            }
            self.i += 1;
            let val = self.value()?;
            fields.push((key, val));
            self.ws();
            match self.b.get(self.i) {
                Some(b',') => self.i += 1,
                Some(b'}') => {
                    self.i += 1;
                    return Some(Json::Obj(fields));
                }
                _ => return None,
            }
        }
    }

    fn arr(&mut self) -> Option<Json> {
        self.i += 1;
        let mut items = Vec::new();
        self.ws();
        if self.b.get(self.i) == Some(&b']') {
            self.i += 1;
            return Some(Json::Arr(items));
        }
        loop {
            items.push(self.value()?);
            self.ws();
            match self.b.get(self.i) {
                Some(b',') => self.i += 1,
                Some(b']') => {
                    self.i += 1;
                    return Some(Json::Arr(items));
                }
                _ => return None,
            }
        }
    }

    fn string(&mut self) -> Option<String> {
        if self.b.get(self.i) != Some(&b'"') {
            return None;
        }
        self.i += 1;
        let mut out = String::new();
        while let Some(&c) = self.b.get(self.i) {
            match c {
                b'"' => {
                    self.i += 1;
                    return Some(out);
                }
                b'\\' => {
                    self.i += 1;
                    match self.b.get(self.i) {
                        Some(b'"') => out.push('"'),
                        Some(b'\\') => out.push('\\'),
                        Some(b'/') => out.push('/'),
                        Some(b'n') => out.push('\n'),
                        Some(b't') => out.push('\t'),
                        Some(b'r') => out.push('\r'),
                        Some(b'u') => {
                            let hex = std::str::from_utf8(&self.b[self.i + 1..self.i + 5]).ok()?;
                            let cp = u16::from_str_radix(hex, 16).ok()?;
                            out.push(char::from_u32(cp as u32)?);
                            self.i += 4;
                        }
                        _ => return None,
                    }
                    self.i += 1;
                }
                _ => {
                    out.push(c as char);
                    self.i += 1;
                }
            }
        }
        None
    }

    fn number(&mut self) -> Option<Json> {
        let start = self.i;
        while self.i < self.b.len()
            && matches!(
                self.b[self.i],
                b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9'
            )
        {
            self.i += 1;
        }
        std::str::from_utf8(&self.b[start..self.i])
            .ok()?
            .parse::<f64>()
            .ok()
            .map(Json::Num)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let v = decode_view("seed=andes&hour=12.00&weather=snow&aspect=32:9&biome=norte");
        assert_eq!(v.seed.as_deref(), Some("andes"));
        assert_eq!(v.hour, Some(12.0));
        assert_eq!(v.weather.as_deref(), Some("snow"));
        assert_eq!(v.biome.as_deref(), Some("norte"));
        let q = encode_view(&v);
        let v2 = decode_view(&q);
        assert_eq!(v, v2);
    }

    #[test]
    fn rejects_invalid() {
        let v = decode_view("weather=malo&moment=xyz&clock=slow");
        assert_eq!(v.weather, None);
        assert_eq!(v.moment, None);
        assert_eq!(v.clock, None);
    }

    #[test]
    fn favorites() {
        let v = decode_view("seed=andes");
        let list = add_view(&[], &v, "casa");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "casa");
        let list = remove_view(&list, "casa");
        assert!(list.is_empty());
    }

    #[test]
    fn parse_views_tolerates_garbage() {
        assert!(parse_views("").is_empty());
        assert!(parse_views("no-json").is_empty());
        let raw = r#"[{"name":"a","query":"seed=x"},{"name":2,"query":"q"}]"#;
        let out = parse_views(raw);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].name, "a");
    }
}
