//! Insert Part: the standard parts library (screws, bolts, nuts, washers, set screws, pins,
//! bearings; in the Gear / Pulley dialog gears, worms, cycloidal discs and pulleys). A search
//! finds a part by its size, name or dimensions ("608", "M6", "m2 helical", "8x22"); otherwise
//! the categories are pictures to pick from, then the family, series (bearings), size and
//! length, the options the family takes (bore, helix angle, worm starts), the size's table
//! values and a picture of the part. Pictures are built off the UI thread: a helical gear or a
//! worm takes seconds.
//! Picking a hole (its wall or rim) seats the part in it with a rigid joint; without one the part
//! goes at the origin. The live preview shows it placed.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::ui_kit::{choice_row, list_picker, picture_tile, text_row, tile_grid};
use egui::vec2;
use serde_json::{Value, json};
use solidcraft_engine::{Sel, Session};

use crate::SolveApp;
use crate::dialogs::{Kind, combo, form_row};
use crate::selection::{EDGES, FACES, SelInput};
use crate::theme::Tokens;

/// One family of the library.
#[derive(Clone, Debug, PartialEq)]
pub struct Family {
    pub id: String,
    pub name: String,
    pub standard: String,
    /// "fasteners" or "gears_pulleys" (the Fastener and Gear / Pulley dialogs each list one).
    pub group: String,
    /// The category it is listed under (`Category::id`).
    pub category: String,
    /// Size names and their table rows.
    pub sizes: Vec<(String, Value)>,
    pub lengths: Option<Vec<f64>>,
    /// What it takes besides size and length: "bore", "helix_angle", "starts", "drive", "thread".
    pub options: Vec<String>,
    /// The drives a screw comes with, (id, name), the first its default.
    pub drives: Vec<(String, String)>,
}

/// The library as `parts.library` gives it.
pub fn families(v: &Value) -> Vec<Family> {
    v.get("families")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|f| {
            let id = f.get("family")?.as_str()?.to_string();
            let sizes = f
                .get("sizes")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|s| Some((s.get("size")?.as_str().map(str::to_string).or_else(|| s.get("size").map(Value::to_string))?, s.clone())))
                .collect();
            Some(Family {
                name: f.get("name").and_then(Value::as_str).unwrap_or(&id).to_string(),
                standard: f.get("standard").and_then(Value::as_str).unwrap_or("").to_string(),
                group: f.get("group").and_then(Value::as_str).unwrap_or("fasteners").to_string(),
                category: f.get("category").and_then(Value::as_str).unwrap_or("").to_string(),
                sizes,
                lengths: f.get("lengths").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_f64).collect()),
                options: f
                    .get("options")
                    .and_then(Value::as_array)
                    .map_or(Vec::new(), |a| a.iter().filter_map(|o| o.as_str().map(str::to_string)).collect()),
                drives: f.get("drives").and_then(Value::as_array).map_or(Vec::new(), |a| {
                    a.iter().filter_map(|d| Some((d.get("drive")?.as_str()?.to_string(), d.get("name")?.as_str()?.to_string()))).collect()
                }),
                id,
            })
        })
        .collect()
}

/// A category of the library: its name and the part that pictures it (insert parameters).
#[derive(Clone, Debug, PartialEq)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub sample: Value,
}

/// The categories one dialog lists, as `parts.library` gives them.
pub fn categories(v: &Value, gears: bool) -> Vec<Category> {
    v.get("categories")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|c| (c.get("group").and_then(Value::as_str) == Some("gears_pulleys")) == gears)
        .filter_map(|c| {
            Some(Category {
                id: c.get("category")?.as_str()?.to_string(),
                name: c.get("name").and_then(Value::as_str).unwrap_or("").to_string(),
                sample: c.get("sample")?.clone(),
            })
        })
        .collect()
}

/// A search result: a size of a family (indices into the dialog's library) and its line.
#[derive(Clone, Debug, PartialEq)]
pub struct Hit {
    pub family: usize,
    pub size: usize,
    pub label: String,
}

/// The most search results listed.
const MAX_HITS: usize = 40;

#[derive(Clone, Debug)]
pub struct Pt {
    pub family: usize,
    pub size: usize,
    /// Index into the family's lengths (none: the default length).
    pub length: Option<usize>,
    pub name: String,
    /// The library, read once.
    pub lib: Option<Vec<Family>>,
    pub cats: Vec<Category>,
    /// The Gear / Pulley dialog (its families only, a bore field, "Width" for the length).
    pub gears: bool,
    /// The bore typed in (mm); empty: the size's default.
    pub bore: String,
    /// A helical gear's helix angle typed in (degrees or an angle); empty: the default.
    pub helix: String,
    /// A worm wheel's worm starts typed in; empty: one.
    pub starts: String,
    /// The series the size list shows (bearings: "62"…); empty: all.
    pub series: String,
    /// A screw's drive (its id); empty: the family's first.
    pub drive: String,
    /// The thread modelled (cut into the part, slower) rather than cosmetic.
    pub modeled: bool,
    /// What is typed in the search field, and what it found (for `hits_for`).
    pub query: String,
    pub hits: Vec<Hit>,
    pub hits_for: String,
}

impl Pt {
    fn new(gears: bool) -> Self {
        Pt {
            family: 0,
            size: 0,
            length: None,
            name: String::new(),
            lib: None,
            cats: Vec::new(),
            gears,
            bore: String::new(),
            helix: String::new(),
            starts: String::new(),
            series: String::new(),
            drive: String::new(),
            modeled: false,
            query: String::new(),
            hits: Vec::new(),
            hits_for: String::new(),
        }
    }

    /// Go to a family's size, as a new pick: its length, bore and starts back to the defaults.
    fn pick(&mut self, family: usize, size: usize) {
        self.family = family;
        self.size = size;
        self.length = None;
        self.series.clear();
        self.bore.clear();
        self.starts.clear();
        self.drive.clear();
    }
}

/// The dialog for Insert Part.
pub fn start(_app: &SolveApp, id: &str) -> Option<(Kind, Vec<SelInput>)> {
    matches!(id, "parts.fastener" | "parts.insert" | "parts.gear").then(|| {
        let gears = id == "parts.gear";
        (Kind::Part(Pt::new(gears)), vec![SelInput::new("Hole (optional)", FACES | EDGES, false)])
    })
}

/// The families one dialog lists.
fn of_group(lib: Vec<Family>, gears: bool) -> Vec<Family> {
    lib.into_iter().filter(|f| (f.group == "gears_pulleys") == gears).collect()
}

/// The size's name as a list shows it: teeth as z20, a worm's starts, a disc's lobes.
fn size_label(row: &Value, size: &str) -> String {
    if row.get("teeth").is_some() {
        format!("z{size}")
    } else if row.get("starts").is_some() {
        format!("{size}-start")
    } else if row.get("lobes").is_some() {
        format!("{size} lobes")
    } else {
        size.to_string()
    }
}

/// A bearing's envelope, bore × outside × width ("8x22x7"); empty for other parts.
fn envelope(row: &Value) -> String {
    match (row.get("bore").and_then(Value::as_f64), row.get("outer").and_then(Value::as_f64), row.get("width").and_then(Value::as_f64)) {
        (Some(d), Some(od), Some(b)) => format!("{d}x{od}x{b}"),
        _ => String::new(),
    }
}

/// The module or pin circle a family id ends in, as typed: `helical_gear_m1_5` → "m1.5".
fn family_alias(id: &str) -> String {
    for mark in ["_m", "_d"] {
        if let Some(i) = id.rfind(mark) {
            return format!("{}{}", &mark[1..], id[i + 2..].replace('_', "."));
        }
    }
    String::new()
}

/// The library's sizes matching every word of `query` (case-insensitive) in the size, a
/// bearing's envelope, or the family's name, standard, category and module; best first: a size
/// typed exactly, then the sizes it begins, then envelopes, then names ("62" lists the 62xx
/// bearings before the ISO 4762 screws).
pub fn search(lib: &[Family], cats: &[Category], query: &str) -> Vec<Hit> {
    let words: Vec<String> = query.split_whitespace().map(|w| w.to_lowercase().replace('×', "x")).collect();
    if words.is_empty() {
        return Vec::new();
    }
    let mut found: Vec<(u32, Hit)> = Vec::new();
    for (fi, f) in lib.iter().enumerate() {
        let category = cats.iter().find(|c| c.id == f.category).map_or("", |c| c.name.as_str());
        let family = format!("{} {} {} {} {}", f.name, f.standard, f.id.replace('_', " "), category, family_alias(&f.id)).to_lowercase();
        for (si, (size, row)) in f.sizes.iter().enumerate() {
            let (shown, dims) = (size_label(row, size), envelope(row));
            let (size_l, shown_l) = (size.to_lowercase(), shown.to_lowercase());
            let mut score = 0;
            for w in &words {
                let s = if size_l == *w || shown_l == *w {
                    8
                } else if size_l.starts_with(w.as_str()) || shown_l.starts_with(w.as_str()) {
                    4
                } else if dims.contains(w.as_str()) || size_l.contains(w.as_str()) {
                    2
                } else if family.contains(w.as_str()) {
                    1
                } else {
                    0
                };
                if s == 0 {
                    score = 0;
                    break;
                }
                score += s;
            }
            if score == 0 {
                continue;
            }
            let label = if dims.is_empty() { format!("{shown} · {}", f.name) } else { format!("{shown} · {} {}", f.name, dims.replace('x', "×")) };
            found.push((score, Hit { family: fi, size: si, label }));
        }
    }
    // Stable: equal scores keep the library's order.
    found.sort_by_key(|f| std::cmp::Reverse(f.0));
    found.into_iter().take(MAX_HITS).map(|(_, h)| h).collect()
}

/// The insert command's parameters for the picks.
pub fn insert_params(lib: &[Family], k: &Pt, hole: Option<&Sel>) -> Result<Value, String> {
    let f = lib.get(k.family).ok_or("the parts library is empty")?;
    let (size, _) = f.sizes.get(k.size).ok_or("pick a size")?;
    let mut p = json!({"family": f.id, "size": size});
    if let Some(l) = k.length.and_then(|i| f.lengths.as_ref().and_then(|ls| ls.get(i))) {
        p["length"] = json!(l);
    }
    match hole {
        Some(Sel::Face { point, .. } | Sel::Edge { point, .. }) => p["at"] = json!([point.x, point.y, point.z]),
        _ => {
            p["point"] = json!([0.0, 0.0, 0.0]);
            p["direction"] = json!([0.0, 0.0, 1.0]);
        }
    }
    if !k.name.trim().is_empty() {
        p["name"] = json!(k.name.trim());
    }
    let takes = |o: &str| f.options.iter().any(|x| x == o);
    if takes("bore") && !k.bore.trim().is_empty() {
        let b: f64 = k.bore.trim().trim_end_matches("mm").trim().replace(',', ".").parse().map_err(|_| "the bore must be a number (mm)")?;
        p["bore"] = json!(b);
    }
    if takes("helix_angle") && !k.helix.trim().is_empty() {
        // A plain number is degrees; anything else goes as an angle expression ("20 deg").
        let h = k.helix.trim().trim_end_matches('°').trim().replace(',', ".");
        p["helix_angle"] = h.parse::<f64>().map_or_else(|_| json!(h), |d| json!(d));
    }
    if takes("starts") && !k.starts.trim().is_empty() {
        let n: u32 = k.starts.trim().parse().ok().filter(|n| (1..=4).contains(n)).ok_or("the worm's starts must be 1…4")?;
        p["starts"] = json!(n);
    }
    if takes("drive") && f.drives.iter().any(|d| d.0 == k.drive) {
        p["drive"] = json!(k.drive);
    }
    if takes("thread") && k.modeled {
        p["thread"] = json!("modeled");
    }
    Ok(p)
}

/// The part alone at the origin, built in a session of its own (for its picture).
fn part_alone(p: &Value) -> Option<Session> {
    let mut s = Session::default();
    let mut q = p.clone();
    if let Value::Object(m) = &mut q {
        m.remove("at");
        m.remove("name");
        m.insert("point".into(), json!([0.0, 0.0, 0.0]));
        m.insert("direction".into(), json!([0.0, 0.0, 1.0]));
    }
    s.execute("parts.insert", &q).ok()?;
    Some(s)
}

/// Part pictures by their insert parameters, kept while the app runs: the parts are built off
/// the UI thread where there is one and pictured on it.
#[derive(Clone, Default)]
struct Pictures(Arc<Mutex<Shelf>>);

#[derive(Default)]
struct Shelf {
    shown: HashMap<String, Option<egui::TextureHandle>>,
    building: HashSet<String>,
    built: Vec<(String, Option<Session>)>,
}

/// A part's picture as far as it has got.
enum Shown {
    Ready(egui::TextureHandle),
    Building,
    Missing,
}

impl Shown {
    fn texture(&self) -> Option<&egui::TextureHandle> {
        match self {
            Shown::Ready(t) => Some(t),
            _ => None,
        }
    }
}

/// The picture of the part `params` inserts; the first ask starts building it, and the frame is
/// repainted when it is done.
fn picture(ctx: &egui::Context, params: &Value) -> Shown {
    let mut p = params.clone();
    if let Value::Object(m) = &mut p {
        m.remove("at");
        m.remove("name");
    }
    let key = p.to_string();
    let pictures = ctx.data_mut(|d| d.get_temp_mut_or_default::<Pictures>(egui::Id::new("sc_part_pictures")).clone());
    {
        let Ok(mut shelf) = pictures.0.lock() else { return Shown::Missing };
        for (k, s) in std::mem::take(&mut shelf.built) {
            let tex = s
                .as_ref()
                .and_then(crate::home::thumbnail)
                .map(|img| ctx.load_texture(format!("sc_part_picture_{k}"), img, egui::TextureOptions::LINEAR));
            shelf.building.remove(&k);
            shelf.shown.insert(k, tex);
        }
        if let Some(t) = shelf.shown.get(&key) {
            return t.clone().map_or(Shown::Missing, Shown::Ready);
        }
        if !shelf.building.insert(key.clone()) {
            return Shown::Building;
        }
    }
    build(ctx, pictures, key, p);
    Shown::Building
}

#[cfg(not(target_arch = "wasm32"))]
fn build(ctx: &egui::Context, pictures: Pictures, key: String, params: Value) {
    let ctx = ctx.clone();
    std::thread::spawn(move || {
        let s = part_alone(&params);
        if let Ok(mut shelf) = pictures.0.lock() {
            shelf.built.push((key, s));
        }
        ctx.request_repaint();
    });
}

/// In a browser there is no thread to build on: built now, pictured next frame.
#[cfg(target_arch = "wasm32")]
fn build(ctx: &egui::Context, pictures: Pictures, key: String, params: Value) {
    let s = part_alone(&params);
    if let Ok(mut shelf) = pictures.0.lock() {
        shelf.built.push((key, s));
    }
    ctx.request_repaint();
}

/// The library and its categories, read once per dialog.
fn loaded(app: &mut SolveApp, k: &mut Pt) -> Vec<Family> {
    if let Some(l) = &k.lib {
        return l.clone();
    }
    let v = app.session.execute("parts.library", &json!({})).unwrap_or(Value::Null);
    let lib = of_group(families(&v), k.gears);
    k.cats = categories(&v, k.gears);
    k.lib = Some(lib.clone());
    lib
}

/// The dialog's rows.
pub fn rows(app: &mut SolveApp, ui: &mut egui::Ui, k: &mut Pt, inputs: &[SelInput]) -> bool {
    let t = Tokens::get();
    let lib = loaded(app, k);
    if lib.is_empty() {
        ui.vertical(|ui| {
            crate::ui_kit::text(ui, crate::i18n::tr_label("the parts library is not available"), crate::ui_kit::Text::Note("parts_unavailable"));
        });
        return false;
    }
    text_row(ui, "Search", &mut k.query, if k.gears { "m2, helical, GT2 20…" } else { "608, M6, 8x22…" });
    if !k.query.trim().is_empty() {
        // While searching, the results stand in for the rest: picking one goes back to it.
        if k.hits_for != k.query {
            k.hits = search(&lib, &k.cats, &k.query);
            k.hits_for = k.query.clone();
        }
        let mut picked = None;
        ui.vertical(|ui| {
            if k.hits.is_empty() {
                crate::ui_kit::text(ui, crate::i18n::tr_label("nothing in the library matches"), crate::ui_kit::Text::Note("parts_no_match"));
            }
            for (i, h) in k.hits.iter().enumerate() {
                if list_picker(ui, &format!("part_hit_{i}"), &h.label, false).clicked() {
                    picked = Some((h.family, h.size));
                }
            }
        });
        let Some((family, size)) = picked else { return false };
        k.pick(family, size);
        k.query.clear();
    }
    // The categories, as pictures.
    let here = lib.get(k.family).map(|f| f.category.clone()).unwrap_or_default();
    let cats: Vec<(usize, Category)> = k.cats.clone().into_iter().enumerate().collect();
    let mut chosen = None;
    ui.vertical(|ui| {
        tile_grid(ui, &cats, |ui, (i, c), size| {
            let mut sample = c.sample.clone();
            sample["point"] = json!([0.0, 0.0, 0.0]);
            sample["direction"] = json!([0.0, 0.0, 1.0]);
            let shown = picture(ui.ctx(), &sample);
            if picture_tile(ui, &c.id, shown.texture(), crate::i18n::tr_label(&c.name), c.id == here, size).clicked() {
                chosen = Some(*i);
            }
        });
    });
    if let Some(c) = chosen.and_then(|i| k.cats.get(i)).cloned() {
        // Its sample's family and size, else the category's first family.
        let sample_family = c.sample.get("family").and_then(Value::as_str).unwrap_or("");
        let family = lib.iter().position(|f| f.id == sample_family).or_else(|| lib.iter().position(|f| f.category == c.id)).unwrap_or(0);
        let sample_size = c.sample.get("size").and_then(Value::as_str).unwrap_or("");
        let size = lib.get(family).and_then(|f| f.sizes.iter().position(|s| s.0 == sample_size)).unwrap_or(0);
        k.pick(family, size);
    }
    // The families of its category.
    let Some(f) = lib.get(k.family).cloned() else { return false };
    let members: Vec<usize> = (0..lib.len()).filter(|i| lib.get(*i).is_some_and(|g| g.category == f.category)).collect();
    if members.len() > 1 {
        let names: Vec<String> = members
            .iter()
            .filter_map(|i| lib.get(*i))
            .map(|g| if g.standard.is_empty() { g.name.clone() } else { format!("{} ({})", g.name, g.standard) })
            .collect();
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        let mut at = members.iter().position(|i| *i == k.family).unwrap_or(0);
        choice_row(ui, "Type", "pt_family", &mut at, &refs);
        if let Some(i) = members.get(at).copied().filter(|i| *i != k.family) {
            k.pick(i, 0);
        }
    }
    let Some(f) = lib.get(k.family) else { return false };
    // A series (bearings) narrows the sizes.
    let mut series: Vec<String> = Vec::new();
    for (_, row) in &f.sizes {
        if let Some(s) = row.get("series").and_then(Value::as_str).filter(|s| !series.iter().any(|x| x == s)) {
            series.push(s.to_string());
        }
    }
    if !series.is_empty() {
        let mut labels = vec![crate::i18n::tr_label("All").to_string()];
        labels.extend(series.iter().cloned());
        let refs: Vec<&str> = labels.iter().map(String::as_str).collect();
        let mut at = series.iter().position(|s| *s == k.series).map_or(0, |i| i + 1);
        choice_row(ui, "Series", "pt_series", &mut at, &refs);
        k.series = at.checked_sub(1).and_then(|i| series.get(i)).cloned().unwrap_or_default();
    }
    let shown: Vec<usize> = (0..f.sizes.len())
        .filter(|i| k.series.is_empty() || f.sizes.get(*i).and_then(|s| s.1.get("series")).and_then(Value::as_str) == Some(&k.series))
        .collect();
    if !shown.contains(&k.size) {
        k.size = shown.first().copied().unwrap_or(0);
    }
    let sizes: Vec<String> = shown
        .iter()
        .filter_map(|i| f.sizes.get(*i))
        .map(|(s, row)| {
            let dims = envelope(row);
            if dims.is_empty() { size_label(row, s) } else { format!("{s}  ({})", dims.replace('x', "×")) }
        })
        .collect();
    let refs: Vec<&str> = sizes.iter().map(String::as_str).collect();
    let mut at = shown.iter().position(|i| *i == k.size).unwrap_or(0);
    choice_row(ui, "Size", "pt_size", &mut at, &refs);
    k.size = shown.get(at).copied().unwrap_or(k.size);
    if let Some(ls) = &f.lengths {
        let mut labels = vec!["Default".to_string()];
        labels.extend(ls.iter().map(|l| crate::prefs::show_mm(app, *l, 1)));
        let refs: Vec<&str> = labels.iter().map(String::as_str).collect();
        let mut i = k.length.map_or(0, |x| x + 1);
        form_row(ui, if k.gears { "Width" } else { "Length" }, |ui| {
            combo(ui, "pt_len", &refs, &mut i);
            k.length = i.checked_sub(1);
        });
    }
    let takes = |o: &str| f.options.iter().any(|x| x == o);
    if takes("bore") {
        text_row(ui, "Bore", &mut k.bore, "automatic");
    }
    if takes("helix_angle") {
        text_row(ui, "Helix angle", &mut k.helix, "20°");
    }
    if takes("starts") {
        text_row(ui, "Starts", &mut k.starts, "1");
    }
    if takes("drive") {
        let names: Vec<&str> = f.drives.iter().map(|d| crate::i18n::tr_label(&d.1)).collect();
        let mut at = f.drives.iter().position(|d| d.0 == k.drive).unwrap_or(0);
        choice_row(ui, "Drive", "pt_drive", &mut at, &names);
        k.drive = f.drives.get(at).map(|d| d.0.clone()).unwrap_or_default();
    }
    if takes("thread") {
        let kinds = [crate::i18n::tr_label("Cosmetic"), crate::i18n::tr_label("Modeled (slower)")];
        let mut at = usize::from(k.modeled);
        choice_row(ui, "Thread", "pt_thread", &mut at, &kinds);
        k.modeled = at == 1;
    }
    text_row(ui, "Name", &mut k.name, "automatic");
    // The part (its picture follows every field that shapes it), and its table values.
    let shown = insert_params(&lib, k, None).ok().map_or(Shown::Missing, |p| picture(ui.ctx(), &p));
    ui.vertical(|ui| match shown {
        Shown::Ready(tx) => {
            ui.add(egui::Image::new(&tx).fit_to_exact_size(vec2(180.0, 112.0)).corner_radius(t.radius_sm));
        }
        Shown::Building => {
            crate::ui_kit::text(ui, crate::i18n::tr_label("building the picture…"), crate::ui_kit::Text::Note("parts_picture"));
        }
        Shown::Missing => {
            crate::ui_kit::text(ui, crate::i18n::tr_label("no picture"), crate::ui_kit::Text::Note("parts_picture"));
        }
    });
    if let Some((_, row)) = f.sizes.get(k.size)
        && let Some(o) = row.as_object()
    {
        for (key, v) in o.iter().filter(|(key, _)| key.as_str() != "size") {
            form_row(ui, &key.replace('_', " "), |ui| {
                crate::ui_kit::text(
                    ui,
                    &match (key.as_str(), v.as_f64()) {
                        // Counts and angles, not lengths.
                        ("teeth" | "starts" | "lobes" | "pins" | "ratio", Some(x)) => format!("{x}"),
                        ("helix_angle" | "lead_angle", Some(x)) => format!("{x:.1}°"),
                        (_, Some(x)) => crate::prefs::show_mm(app, x, 1),
                        _ => v.as_str().map_or_else(|| v.to_string(), str::to_string),
                    },
                    crate::ui_kit::Text::Cell,
                );
            });
        }
    }
    if inputs.first().is_some_and(|i| i.items.is_empty()) {
        ui.vertical(|ui| {
            crate::ui_kit::text(
                ui,
                crate::i18n::tr_label("pick a hole to seat it, or OK places it at the origin"),
                crate::ui_kit::Text::Note("parts_placement"),
            );
        });
    }
    false
}

/// The commands OK runs.
pub fn commands(app: &SolveApp, k: &Pt, inputs: &[SelInput]) -> Result<Vec<(String, Value)>, String> {
    let lib = match &k.lib {
        Some(l) => l.clone(),
        None => app.session.scratch().execute("parts.library", &json!({})).map(|v| of_group(families(&v), k.gears)).unwrap_or_default(),
    };
    let p = insert_params(&lib, k, inputs.first().and_then(|i| i.items.first()))?;
    // The toolbar's command when the engine has it (same parameters).
    let tool = if k.gears { "parts.gear" } else { "parts.fastener" };
    let id = if solidcraft_engine::find_command(tool).is_some() { tool } else { "parts.insert" };
    Ok(vec![(id.into(), p)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use solidcraft_engine::geom::Vec3;

    fn lib() -> Vec<Family> {
        families(&json!({"families": [
            {"family": "socket_head_cap_screw", "name": "Socket Head Cap Screw", "standard": "ISO 4762",
             "sizes": [{"size": "M5", "head_diameter": 8.5}, {"size": "M6", "head_diameter": 10}], "lengths": [10, 16, 20]},
            {"family": "bearing", "name": "Deep Groove Ball Bearing", "standard": "", "sizes": [{"size": "608", "bore": 8}], "lengths": null},
        ]}))
    }

    #[test]
    fn library_rows_parse() {
        let l = lib();
        assert_eq!(l.len(), 2);
        assert_eq!(l[0].sizes[1].0, "M6");
        assert_eq!(l[0].lengths.as_deref(), Some(&[10.0, 16.0, 20.0][..]));
        assert!(l[1].lengths.is_none());
    }

    /// A hole pick seats the part (`at`); without one it goes to the origin along Z.
    #[test]
    fn insert_params_for_picks() {
        let l = lib();
        let k = Pt { size: 1, length: Some(2), ..Pt::new(false) };
        let hole = Sel::Face { body: "Body1".into(), index: 3, point: Vec3::new(5.0, 0.0, 10.0) };
        let p = insert_params(&l, &k, Some(&hole)).unwrap();
        assert_eq!(p, json!({"family": "socket_head_cap_screw", "size": "M6", "length": 20.0, "at": [5.0, 0.0, 10.0]}));
        let k = Pt { family: 1, name: "Front bearing".into(), ..Pt::new(false) };
        let p = insert_params(&l, &k, None).unwrap();
        assert_eq!(p["point"], json!([0.0, 0.0, 0.0]));
        assert_eq!(p["name"], "Front bearing");
        assert!(p.get("length").is_none());
        assert!(insert_params(&[], &k, None).is_err());
    }

    /// A family's options decide which typed fields go to the command: a helical gear takes a
    /// bore and a helix angle (degrees, or an angle), a worm wheel the worm's starts; a field the
    /// family does not take is left out.
    #[test]
    fn gear_options_reach_the_command() {
        let l = families(&json!({"families": [
            {"family": "helical_gear_m1", "name": "Helical gear", "sizes": [{"size": "20"}], "lengths": [10], "options": ["bore", "helix_angle"]},
            {"family": "worm_wheel_m1", "name": "Worm wheel", "sizes": [{"size": "30"}], "lengths": [10], "options": ["bore", "starts"]},
            {"family": "ring_gear_m1", "name": "Ring gear", "sizes": [{"size": "60"}], "lengths": [10], "options": []},
        ]}));
        let k = Pt { bore: "6".into(), helix: "-15".into(), starts: "2".into(), ..Pt::new(true) };
        let p = insert_params(&l, &k, None).unwrap();
        assert_eq!((p["bore"].clone(), p["helix_angle"].clone(), p.get("starts")), (json!(6.0), json!(-15.0), None));
        let p = insert_params(&l, &Pt { helix: "0.3 rad".into(), ..k.clone() }, None).unwrap();
        assert_eq!(p["helix_angle"], "0.3 rad");
        let p = insert_params(&l, &Pt { family: 1, ..k.clone() }, None).unwrap();
        assert_eq!((p["starts"].clone(), p.get("helix_angle")), (json!(2), None));
        let p = insert_params(&l, &Pt { family: 2, ..k.clone() }, None).unwrap();
        assert!(p.get("bore").is_none() && p.get("starts").is_none(), "{p}");
        assert!(insert_params(&l, &Pt { family: 1, starts: "7".into(), ..k }, None).is_err());
        // A screw: its drive when it has a choice, and a modelled thread.
        let l = families(&json!({"families": [
            {"family": "pan_head_screw", "name": "Pan head screw", "sizes": [{"size": "M4"}], "lengths": [10],
             "options": ["drive", "thread"], "drives": [{"drive": "cross", "name": "Cross recess (PH)"}, {"drive": "hexalobular", "name": "Hexalobular (TX)"}]},
            {"family": "washer", "name": "Washer", "sizes": [{"size": "M4"}], "options": []},
        ]}));
        assert_eq!(l.first().map(|f| f.drives.len()), Some(2));
        let k = Pt { drive: "hexalobular".into(), modeled: true, ..Pt::new(false) };
        let p = insert_params(&l, &k, None).unwrap();
        assert_eq!((p["drive"].clone(), p["thread"].clone()), (json!("hexalobular"), json!("modeled")));
        let p = insert_params(&l, &Pt { family: 1, ..k }, None).unwrap();
        assert!(p.get("drive").is_none() && p.get("thread").is_none(), "{p}");
    }

    /// A search matches every word in the name, standard, category, module, size or a bearing's
    /// envelope; a size typed exactly comes first.
    #[test]
    fn search_finds_sizes_by_name_module_and_envelope() {
        let v = json!({"families": [
            {"family": "bearing", "name": "Deep groove ball bearing", "standard": "ISO 15", "category": "bearings",
             "sizes": [{"size": "6200", "bore": 10, "outer": 30, "width": 9}, {"size": "608", "series": "60", "bore": 8, "outer": 22, "width": 7}]},
            {"family": "helical_gear_m1_5", "name": "Helical gear, normal module 1.5", "category": "helical_gears", "group": "gears_pulleys",
             "sizes": [{"size": "20", "teeth": 20}, {"size": "24", "teeth": 24}]},
        ], "categories": [
            {"category": "bearings", "name": "Bearings", "group": "fasteners", "sample": {"family": "bearing", "size": "608"}},
            {"category": "helical_gears", "name": "Helical gears", "group": "gears_pulleys", "sample": {"family": "helical_gear_m1_5", "size": "20"}},
        ]});
        let (lib, cats) = (families(&v), categories(&v, false));
        assert_eq!(cats.len(), 1);
        assert_eq!(categories(&v, true).first().map(|c| c.name.as_str()), Some("Helical gears"));
        let hits = search(&lib, &cats, "608");
        assert_eq!(hits.iter().map(|h| (h.family, h.size)).collect::<Vec<_>>(), [(0, 1)]);
        assert_eq!(hits.first().map(|h| h.label.as_str()), Some("608 · Deep groove ball bearing 8×22×7"));
        // "20" is in 6200 too: the gear of exactly 20 teeth comes first.
        let hits = search(&lib, &cats, "20");
        assert_eq!(hits.iter().map(|h| (h.family, h.size)).collect::<Vec<_>>(), [(1, 0), (0, 0)]);
        // "62" begins 6200, and is only in the standard's name for the gear family: the bearing first.
        let mut screw = lib.get(1).cloned().unwrap();
        screw.standard = "ISO 4762".into();
        let hits = search(&[screw, lib[0].clone()], &cats, "62");
        assert_eq!(hits.first().map(|h| (h.family, h.size)), Some((1, 0)));
        assert_eq!(search(&lib, &cats, "8X22").len(), 1);
        let hits = search(&lib, &categories(&v, true), "M1.5 helical z24");
        assert_eq!(hits.iter().map(|h| (h.family, h.size)).collect::<Vec<_>>(), [(1, 1)]);
        assert!(search(&lib, &cats, "worm").is_empty() && search(&lib, &cats, "  ").is_empty());
    }
}
