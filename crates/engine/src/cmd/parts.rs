//! Standard parts: ISO metric fasteners, dowel pins and ball bearings, generated in code from
//! the dimensions in the public standards tables (no vendor geometry); involute gears on the
//! ISO 53 basic rack (spur, helical, herringbone, internal ring gears, worm wheels), Archimedean
//! worms, cycloidal reducer discs and timing-belt pulleys (GT2, HTD 3M/5M), generated from their
//! tooth geometry. Each part is inserted as
//! its own component, built by ordinary features (so it stays parametric and editable), with a
//! cosmetic thread where it has one; at a hole it gets a rigid joint that puts its axis on the
//! hole's and seats it on the face.
//!
//! Part frame: the seating plane is z = 0 (the underside of a head, the bottom of a nut or
//! washer, the top of a set screw or pin); the shank runs down −z.

use serde_json::{Value, json};
use solidcraft_geom::Vec3;

use super::CommandSpec;
use crate::params::{bad, num, str_, vec3};
use crate::{EngineError, Result, Session};

pub static COMMANDS: &[CommandSpec] = &[
    CommandSpec::new("parts.library", "Standard Parts", library).icon("library")
        .noundo()
        .params("family? → families (name, standard, group, category, sizes, lengths, options) and categories (name, group, a sample part); with family, each size's dimensions (mm)"),
    CommandSpec::new("parts.fastener", "Insert Fastener", insert).at("SOLID", "INSERT").icon("fastener").params(
        "the Insert Part dialog's command (same as parts.insert): family, size, length?, at?|point?+direction?, name?",
    ),
    CommandSpec::new("parts.gear", "Insert Gear / Pulley", insert).at("SOLID", "INSERT").icon("gear").params(
        "the Insert Gear / Pulley dialog's command (same as parts.insert): family (spur_gear_m… | helical_gear_m… | herringbone_gear_m… | ring_gear_m… | worm_m… | worm_wheel_m… | cycloidal_disc_d… | pulley_…), size (teeth; a worm's starts; a cycloidal disc's lobes), length? (width), bore? (mm), helix_angle?, starts?, at?|point?+direction?, name?",
    ),
    CommandSpec::new("parts.insert", "Insert Part", insert).icon("fastener").params(
        "family (socket_head_cap_screw | hex_bolt | hex_nut | washer | set_screw | dowel_pin | bearing | spur_gear_m0_5 … spur_gear_m3 | helical_gear_m… | herringbone_gear_m… | ring_gear_m… | worm_m… | worm_wheel_m… (modules 0_5, 1, 1_5, 2, 3) | cycloidal_disc_d40 | _d60 | _d80 | _d100 | pulley_gt2 | pulley_htd3m | pulley_htd5m); \
         size (\"M6\", \"608\" for bearings, the number of teeth for gears and pulleys: \"20\"; a worm's number of starts 1…4; a cycloidal disc's number of lobes); \
         length? (mm; screws, bolts, set screws, pins; a gear's or disc's face width, a pulley's belt width, a worm's length; default a standard length near 2.5 d); \
         bore?: mm (gears except ring gears, worms, discs and pulleys; default from the size); \
         helix_angle?: angle (helical and herringbone gears, 5…45 deg, default 20 deg; negative: left hand); \
         starts?: 1…4 (a worm wheel: the starts of the worm it meshes with, default 1); \
         at?: [x,y,z] on a hole's rim or wall (the part seats on the hole's face with a rigid joint) | point?: [x,y,z] with direction?: [x,y,z] (no joint); name?",
    ),
    CommandSpec::new("parts.resize", "Change Part Size", resize_atomic).icon("scale")
        .params("component: id|name of an inserted part; size?; length?; bore? (gears, worms, discs, pulleys); helix_angle?; starts? — rebuilds it (joints keep it seated)"),
];

/// A family: (id, name, standard).
const FAMILIES: &[(&str, &str, &str)] = &[
    ("socket_head_cap_screw", "Socket head cap screw", "ISO 4762 / ISO 14579"),
    ("low_head_cap_screw", "Low head cap screw", "DIN 7984"),
    ("button_head_screw", "Button head screw", "ISO 7380-1"),
    ("flanged_button_head_screw", "Flanged button head screw", "ISO 7380-2"),
    ("countersunk_socket_screw", "Countersunk socket screw", "ISO 10642"),
    ("countersunk_screw", "Countersunk screw", "ISO 2009 / ISO 7046-1 / ISO 14581"),
    ("pan_head_screw", "Pan head screw", "ISO 7045 / ISO 14583"),
    ("pan_head_slotted_screw", "Pan head screw, slotted", "ISO 1580"),
    ("cheese_head_screw", "Cheese head screw", "ISO 1207 / ISO 14580"),
    ("hex_bolt", "Hex head bolt / screw", "ISO 4014 / ISO 4017"),
    ("hex_nut", "Hex nut", "ISO 4032"),
    ("washer", "Plain washer", "ISO 7089"),
    ("set_screw", "Hex socket set screw, flat point", "ISO 4026"),
    ("set_screw_cone", "Hex socket set screw, cone point", "ISO 4027"),
    ("set_screw_dog", "Hex socket set screw, dog point", "ISO 4028"),
    ("set_screw_cup", "Hex socket set screw, cup point", "ISO 4029"),
    ("dowel_pin", "Parallel pin (dowel)", "ISO 8734"),
    ("bearing", "Deep groove ball bearing", "ISO 15, series 68, 69, 60, 62, 63"),
    ("spur_gear_m0_5", "Spur gear, module 0.5", "ISO 53 profile, 20°"),
    ("spur_gear_m1", "Spur gear, module 1", "ISO 53 profile, 20°"),
    ("spur_gear_m1_5", "Spur gear, module 1.5", "ISO 53 profile, 20°"),
    ("spur_gear_m2", "Spur gear, module 2", "ISO 53 profile, 20°"),
    ("spur_gear_m3", "Spur gear, module 3", "ISO 53 profile, 20°"),
    ("helical_gear_m0_5", "Helical gear, normal module 0.5", "ISO 53 normal profile, 20°"),
    ("helical_gear_m1", "Helical gear, normal module 1", "ISO 53 normal profile, 20°"),
    ("helical_gear_m1_5", "Helical gear, normal module 1.5", "ISO 53 normal profile, 20°"),
    ("helical_gear_m2", "Helical gear, normal module 2", "ISO 53 normal profile, 20°"),
    ("helical_gear_m3", "Helical gear, normal module 3", "ISO 53 normal profile, 20°"),
    ("herringbone_gear_m0_5", "Herringbone gear, normal module 0.5", "ISO 53 normal profile, 20°"),
    ("herringbone_gear_m1", "Herringbone gear, normal module 1", "ISO 53 normal profile, 20°"),
    ("herringbone_gear_m1_5", "Herringbone gear, normal module 1.5", "ISO 53 normal profile, 20°"),
    ("herringbone_gear_m2", "Herringbone gear, normal module 2", "ISO 53 normal profile, 20°"),
    ("herringbone_gear_m3", "Herringbone gear, normal module 3", "ISO 53 normal profile, 20°"),
    ("ring_gear_m0_5", "Internal ring gear, module 0.5", "ISO 53 profile, 20°"),
    ("ring_gear_m1", "Internal ring gear, module 1", "ISO 53 profile, 20°"),
    ("ring_gear_m1_5", "Internal ring gear, module 1.5", "ISO 53 profile, 20°"),
    ("ring_gear_m2", "Internal ring gear, module 2", "ISO 53 profile, 20°"),
    ("ring_gear_m3", "Internal ring gear, module 3", "ISO 53 profile, 20°"),
    ("worm_m0_5", "Worm, axial module 0.5", "Archimedean (ZA), 20°"),
    ("worm_m1", "Worm, axial module 1", "Archimedean (ZA), 20°"),
    ("worm_m1_5", "Worm, axial module 1.5", "Archimedean (ZA), 20°"),
    ("worm_m2", "Worm, axial module 2", "Archimedean (ZA), 20°"),
    ("worm_m3", "Worm, axial module 3", "Archimedean (ZA), 20°"),
    ("worm_wheel_m0_5", "Worm wheel, module 0.5", "helical teeth, not throated"),
    ("worm_wheel_m1", "Worm wheel, module 1", "helical teeth, not throated"),
    ("worm_wheel_m1_5", "Worm wheel, module 1.5", "helical teeth, not throated"),
    ("worm_wheel_m2", "Worm wheel, module 2", "helical teeth, not throated"),
    ("worm_wheel_m3", "Worm wheel, module 3", "helical teeth, not throated"),
    ("cycloidal_disc_d40", "Cycloidal disc, pin circle Ø40", "lobes + 1 pins"),
    ("cycloidal_disc_d60", "Cycloidal disc, pin circle Ø60", "lobes + 1 pins"),
    ("cycloidal_disc_d80", "Cycloidal disc, pin circle Ø80", "lobes + 1 pins"),
    ("cycloidal_disc_d100", "Cycloidal disc, pin circle Ø100", "lobes + 1 pins"),
    ("pulley_gt2", "GT2 timing pulley", "2 mm pitch (approximate groove)"),
    ("pulley_htd3m", "HTD 3M timing pulley", "3 mm pitch (approximate groove)"),
    ("pulley_htd5m", "HTD 5M timing pulley", "5 mm pitch (approximate groove)"),
];

/// The kinds of gear that come in the modules of `MODULES`, each family `<prefix><module>`.
#[derive(Clone, Copy, Debug, PartialEq)]
enum GearKind {
    Spur,
    Helical,
    Herringbone,
    Ring,
    Worm,
    WormWheel,
}

const GEAR_KINDS: [(&str, GearKind); 6] = [
    ("spur_gear_", GearKind::Spur),
    ("helical_gear_", GearKind::Helical),
    ("herringbone_gear_", GearKind::Herringbone),
    ("ring_gear_", GearKind::Ring),
    ("worm_wheel_", GearKind::WormWheel),
    ("worm_", GearKind::Worm),
];

/// Modules (normal module for helical gears, axial for worms): (family suffix, module).
const MODULES: [(&str, f64); 5] = [("m0_5", 0.5), ("m1", 1.0), ("m1_5", 1.5), ("m2", 2.0), ("m3", 3.0)];

/// A gear family's kind and module: `helical_gear_m1_5` → (Helical, 1.5).
fn gear_family(family: &str) -> Option<(GearKind, f64)> {
    GEAR_KINDS.iter().find_map(|(prefix, kind)| {
        let m = family.strip_prefix(prefix)?;
        MODULES.iter().find(|x| x.0 == m).map(|x| (*kind, x.1))
    })
}

/// Tooth counts the library lists (any 6…200 can be asked for).
const GEAR_TEETH: [u32; 27] = [10, 12, 13, 14, 15, 16, 17, 18, 20, 22, 24, 25, 28, 30, 32, 36, 40, 45, 48, 50, 56, 60, 64, 72, 80, 90, 100];
const RING_TEETH: [u32; 14] = [30, 32, 36, 40, 45, 48, 50, 56, 60, 64, 72, 80, 90, 100];
const WHEEL_TEETH: [u32; 13] = [20, 24, 25, 28, 30, 32, 36, 40, 45, 50, 60, 80, 100];
const PULLEY_TEETH: [u32; 17] = [12, 14, 16, 18, 20, 24, 25, 28, 30, 32, 36, 40, 48, 50, 60, 72, 80];

/// Gear face widths (mm).
const GEAR_WIDTHS: [f64; 11] = [3.0, 4.0, 5.0, 6.0, 8.0, 10.0, 12.0, 15.0, 20.0, 25.0, 30.0];

/// A helical gear's helix angle when none is given, and the range one may have (degrees).
const HELIX_DEFAULT: f64 = 20.0;
const HELIX_RANGE: (f64, f64) = (5.0, 45.0);

/// Worms: pitch diameter by module (mm), and their lengths.
const WORM_D1: [(f64, f64); 5] = [(0.5, 8.0), (1.0, 12.0), (1.5, 15.0), (2.0, 20.0), (3.0, 30.0)];
const WORM_LENGTHS: [f64; 13] = [6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 20.0, 25.0, 30.0, 40.0, 50.0, 60.0, 80.0];
/// The most turns a worm's thread makes over its length, and per sweep of it.
const WORM_MAX_TURNS: f64 = 16.0;
const TURNS_PER_SWEEP: f64 = 4.0;

fn worm_d1(m: f64) -> f64 {
    WORM_D1.iter().find(|w| w.0 == m).map_or(10.0 * m, |w| w.1)
}

/// A worm's lead angle (at its pitch diameter) for `starts` starts.
fn lead_angle(m: f64, starts: u32) -> f64 {
    (f64::from(starts) * m / worm_d1(m)).atan()
}

/// Cycloidal discs: (family, pin circle diameter); the lobes the library lists, and disc widths.
const CYCLOIDS: [(&str, f64); 4] =
    [("cycloidal_disc_d40", 40.0), ("cycloidal_disc_d60", 60.0), ("cycloidal_disc_d80", 80.0), ("cycloidal_disc_d100", 100.0)];
const CYCLOID_LOBES: [u32; 8] = [8, 10, 12, 15, 20, 25, 30, 40];
const DISC_WIDTHS: [f64; 8] = [3.0, 4.0, 5.0, 6.0, 8.0, 10.0, 12.0, 15.0];

/// A cycloidal disc's eccentricity for pin circle radius `r` and `n` lobes: half the most it may
/// have before the pins' path gets cusps (e·(n + 1) < r).
fn cycloid_e(r: f64, n: u32) -> f64 {
    0.5 * r / f64::from(n + 1)
}

/// The pins' path seen from a cycloidal disc and its first two derivatives at `u`:
/// q(u) = r·(cos u, sin u) − e·(cos ku, sin ku), k = n + 1.
fn trochoid(r: f64, n: u32, e: f64, u: f64) -> ([f64; 2], [f64; 2], [f64; 2]) {
    let k = f64::from(n + 1);
    let (s, c, sk, ck) = (u.sin(), u.cos(), (k * u).sin(), (k * u).cos());
    ([r * c - e * ck, r * s - e * sk], [-r * s + e * k * sk, r * c - e * k * ck], [-r * c + e * k * k * ck, -r * s + e * k * k * sk])
}

/// The smallest radius of curvature where the pins' path bends towards the disc's centre: a pin
/// at least this big would cut the lobes' tips (undercut).
fn cycloid_rho_min(r: f64, n: u32, e: f64) -> f64 {
    const SAMPLES: u32 = 4096;
    (0..SAMPLES)
        .map(|i| {
            let (_, d, dd) = trochoid(r, n, e, 2.0 * std::f64::consts::PI * f64::from(i) / f64::from(SAMPLES));
            let cross = d[0] * dd[1] - d[1] * dd[0];
            if cross > 0.0 { (d[0] * d[0] + d[1] * d[1]).powf(1.5) / cross } else { f64::INFINITY }
        })
        .fold(f64::INFINITY, f64::min)
}

/// A cycloidal disc's pin diameter: the largest ISO 8734 dowel that takes at most half of the
/// space between neighbouring pins and stays clear of undercut.
fn cycloid_pin(r: f64, n: u32) -> Option<f64> {
    let half_gap = r * (std::f64::consts::PI / f64::from(n + 1)).sin();
    let limit = (0.8 * cycloid_rho_min(r, n, cycloid_e(r, n))).min(0.5 * half_gap);
    DOWEL.iter().copied().rev().find(|d| d / 2.0 <= limit)
}

/// Timing pulleys: (id, pitch p, pitch line differential, groove depth, groove radius, flange
/// thickness, belt widths). The pulley's outside diameter is z·p/π − 2·PLD (GT2 20T: 12.22).
/// The groove is one circular arc of the given depth and radius, an approximation of the
/// proprietary profiles: print one and try it on the belt before relying on it.
const PULLEYS: [(&str, f64, f64, f64, f64, f64, &[f64]); 3] = [
    ("pulley_gt2", 2.0, 0.254, 0.75, 0.555, 1.0, &[6.0, 9.0, 10.0, 15.0]),
    ("pulley_htd3m", 3.0, 0.381, 1.14, 0.86, 1.0, &[6.0, 9.0, 15.0]),
    ("pulley_htd5m", 5.0, 0.5715, 2.06, 1.49, 1.5, &[9.0, 15.0, 25.0]),
];

/// How much wider a pulley's toothed part is than its belt (mm), so the belt runs free between
/// the flanges (6 mm belt: 7 mm of teeth).
const BELT_CLEARANCE: f64 = 1.0;

/// Bores the generated gears and pulleys get: the largest that leaves half the root diameter.
const BORES: [f64; 12] = [2.0, 3.0, 4.0, 5.0, 6.0, 8.0, 10.0, 12.0, 15.0, 20.0, 25.0, 30.0];

fn bore_for(root_radius: f64) -> f64 {
    BORES.iter().copied().rev().find(|b| *b <= root_radius).unwrap_or(2.0)
}

/// The library's categories, as the dialogs show them: (id, name, group, and a part that
/// pictures the category: family, size, length).
const CATEGORIES: [(&str, &str, &str, &str, &str, f64); 13] = [
    ("screws", "Screws", "fasteners", "socket_head_cap_screw", "M6", 16.0),
    ("set_screws", "Set screws", "fasteners", "set_screw_dog", "M8", 10.0),
    ("nuts", "Nuts", "fasteners", "hex_nut", "M8", 0.0),
    ("washers", "Washers", "fasteners", "washer", "M8", 0.0),
    ("pins", "Pins", "fasteners", "dowel_pin", "6", 20.0),
    ("bearings", "Bearings", "fasteners", "bearing", "6204", 0.0),
    ("spur_gears", "Spur", "gears_pulleys", "spur_gear_m1", "16", 6.0),
    ("helical_gears", "Helical", "gears_pulleys", "helical_gear_m1", "14", 6.0),
    ("herringbone_gears", "Herringbone", "gears_pulleys", "herringbone_gear_m1", "14", 8.0),
    ("ring_gears", "Internal", "gears_pulleys", "ring_gear_m1", "40", 6.0),
    ("worm_drives", "Worm", "gears_pulleys", "worm_m1", "1", 10.0),
    ("cycloidal_discs", "Cycloidal", "gears_pulleys", "cycloidal_disc_d40", "10", 5.0),
    ("timing_pulleys", "Pulleys", "gears_pulleys", "pulley_gt2", "20", 6.0),
];

/// The category a family is listed under.
fn category(family: &str) -> &'static str {
    match family {
        f if f.starts_with("set_screw") => "set_screws",
        f if f == "hex_bolt" || !screw_drives(f).is_empty() => "screws",
        "hex_nut" => "nuts",
        "washer" => "washers",
        "dowel_pin" => "pins",
        "bearing" => "bearings",
        f if f.starts_with("pulley_") => "timing_pulleys",
        f if f.starts_with("cycloidal_disc_") => "cycloidal_discs",
        f => match gear_family(f).map(|g| g.0) {
            Some(GearKind::Spur) => "spur_gears",
            Some(GearKind::Helical) => "helical_gears",
            Some(GearKind::Herringbone) => "herringbone_gears",
            Some(GearKind::Ring) => "ring_gears",
            Some(GearKind::Worm | GearKind::WormWheel) => "worm_drives",
            None => "screws",
        },
    }
}

/// Gears, discs and pulleys (the Insert Gear / Pulley dialog), as against fasteners.
fn is_drive(family: &str) -> bool {
    gear_family(family).is_some() || family.starts_with("pulley_") || family.starts_with("cycloidal_disc_")
}

/// The options a family takes besides size and length (the dialog shows a field for each).
fn options(family: &str) -> Vec<&'static str> {
    match gear_family(family).map(|g| g.0) {
        Some(GearKind::Ring) => vec![],
        Some(GearKind::Helical | GearKind::Herringbone) => vec!["bore", "helix_angle"],
        Some(GearKind::WormWheel) => vec!["bore", "starts"],
        Some(_) => vec!["bore"],
        None if is_drive(family) => vec!["bore"],
        None => {
            let mut o = Vec::new();
            if screw_drives(family).len() > 1 {
                o.push("drive");
            }
            if threaded(family) {
                o.push("thread");
            }
            o
        }
    }
}

/// A part with the bore asked for: more than 0.5 mm, and a wall of at least 1 mm left to the
/// tooth roots.
fn with_bore(sh: Shape, bore: Option<f64>) -> std::result::Result<Shape, String> {
    let Some(b) = bore else { return Ok(sh) };
    let root = match &sh {
        Shape::Gear { tooth, z, .. } => tooth.root_radius(*z),
        Shape::Worm { m, d1, .. } => d1 / 2.0 - 1.25 * m,
        Shape::Cycloid { r, e, rr, .. } => r - e - rr,
        Shape::Pulley { p, pld, depth, z, .. } => f64::from(*z) * p / std::f64::consts::PI / 2.0 - pld - depth,
        Shape::RingGear { .. } => return Err("a ring gear has no bore".into()),
        _ => return Err("only gears, worms, cycloidal discs and pulleys take a bore".into()),
    };
    if !(b.is_finite() && b > 0.5 && b / 2.0 <= root - 1.0) {
        return Err(format!("bore {b} mm does not fit: at most {:.1} mm for this size", 2.0 * (root - 1.0)));
    }
    Ok(match sh {
        Shape::Gear { tooth, z, w, helix, herringbone, .. } => Shape::Gear { tooth, z, w, bore: b, helix, herringbone },
        Shape::Worm { m, d1, starts, len, .. } => Shape::Worm { m, d1, starts, len, bore: b },
        Shape::Cycloid { r, n, e, rr, w, .. } => Shape::Cycloid { r, n, e, rr, w, bore: b },
        Shape::Pulley { p, pld, depth, rg, z, w, flange, .. } => Shape::Pulley { p, pld, depth, rg, z, w, flange, bore: b },
        other => other,
    })
}

fn bore_param(p: &Value, cmd: &str) -> Result<Option<f64>> {
    match p.get("bore") {
        None | Some(Value::Null) => Ok(None),
        Some(_) => num(p, "bore").filter(|b| b.is_finite() && *b > 0.0).map(Some).ok_or_else(|| bad(cmd, "`bore` must be a diameter in mm")),
    }
}

/// What a gear is made with besides its size and length: a helical gear's helix angle (radians,
/// negative: left hand) and the starts of the worm a worm wheel meshes with.
#[derive(Clone, Debug, Default, PartialEq)]
struct Extra {
    helix: Option<f64>,
    starts: Option<u32>,
    /// A screw's drive (`DRIVES`); none: its family's first.
    drive: Option<String>,
    /// The thread modelled (cut into the part) rather than cosmetic.
    modeled: bool,
}

/// `helix_angle` (a number in degrees or an angle expression) and `starts` (1…4).
fn extra_params(s: &Session, p: &Value, cmd: &str) -> Result<Extra> {
    let helix = match p.get("helix_angle") {
        None | Some(Value::Null) => None,
        Some(_) => {
            let e = crate::params::expr(p, "helix_angle").ok_or_else(|| bad(cmd, "`helix_angle` must be an angle"))?;
            let v = s.doc.eval(&e, solidcraft_doc::expr::Kind::Angle).map_err(|e| bad(cmd, format!("helix_angle: {e}")))?;
            Some(v)
        }
    };
    let starts = match p.get("starts") {
        None | Some(Value::Null) => None,
        Some(v) => {
            Some(v.as_u64().and_then(|n| u32::try_from(n).ok()).filter(|n| (1..=4).contains(n)).ok_or_else(|| bad(cmd, "`starts` must be 1…4"))?)
        }
    };
    let drive = match p.get("drive") {
        None | Some(Value::Null) => None,
        Some(v) => Some(
            v.as_str()
                .filter(|d| DRIVES.iter().any(|x| x.0 == *d))
                .map(str::to_string)
                .ok_or_else(|| bad(cmd, "`drive` must be hex, hexalobular, cross or slot"))?,
        ),
    };
    let modeled = match p.get("thread") {
        None | Some(Value::Null) => false,
        Some(v) => match v.as_str() {
            Some("modeled") => true,
            Some("cosmetic") => false,
            _ => return Err(bad(cmd, "`thread` must be cosmetic or modeled")),
        },
    };
    Ok(Extra { helix, starts, drive, modeled })
}

/// "20", "z20", "20T" -> 20 (6…200 teeth).
fn teeth(size: &str) -> Option<u32> {
    let t = size.trim().trim_start_matches(['z', 'Z']).trim_end_matches(['t', 'T']).trim();
    t.parse::<u32>().ok().filter(|z| (6..=200).contains(z))
}

/// ISO metric coarse: (size, d, pitch).
const METRIC: [(&str, f64, f64); 14] = [
    ("M2", 2.0, 0.4),
    ("M2.5", 2.5, 0.45),
    ("M3", 3.0, 0.5),
    ("M4", 4.0, 0.7),
    ("M5", 5.0, 0.8),
    ("M6", 6.0, 1.0),
    ("M8", 8.0, 1.25),
    ("M10", 10.0, 1.5),
    ("M12", 12.0, 1.75),
    ("M14", 14.0, 2.0),
    ("M16", 16.0, 2.0),
    ("M20", 20.0, 2.5),
    ("M24", 24.0, 3.0),
    ("M30", 30.0, 3.5),
];

/// ISO 4762: (size, head diameter dk, head height k, socket s, socket depth t).
const SHCS: [(&str, f64, f64, f64, f64); 14] = [
    ("M2", 3.8, 2.0, 1.5, 1.0),
    ("M2.5", 4.5, 2.5, 2.0, 1.1),
    ("M3", 5.5, 3.0, 2.5, 1.3),
    ("M4", 7.0, 4.0, 3.0, 2.0),
    ("M5", 8.5, 5.0, 4.0, 2.5),
    ("M6", 10.0, 6.0, 5.0, 3.0),
    ("M8", 13.0, 8.0, 6.0, 4.0),
    ("M10", 16.0, 10.0, 8.0, 5.0),
    ("M12", 18.0, 12.0, 10.0, 6.0),
    ("M14", 21.0, 14.0, 12.0, 7.0),
    ("M16", 24.0, 16.0, 14.0, 8.0),
    ("M20", 30.0, 20.0, 17.0, 10.0),
    ("M24", 36.0, 24.0, 19.0, 12.0),
    ("M30", 45.0, 30.0, 22.0, 15.5),
];

/// ISO 4014/4017 hex head: (size, across flats s, head height k).
const HEX_BOLT: [(&str, f64, f64); 14] = [
    ("M2", 4.0, 1.4),
    ("M2.5", 5.0, 1.7),
    ("M3", 5.5, 2.0),
    ("M4", 7.0, 2.8),
    ("M5", 8.0, 3.5),
    ("M6", 10.0, 4.0),
    ("M8", 13.0, 5.3),
    ("M10", 16.0, 6.4),
    ("M12", 18.0, 7.5),
    ("M14", 21.0, 8.8),
    ("M16", 24.0, 10.0),
    ("M20", 30.0, 12.5),
    ("M24", 36.0, 15.0),
    ("M30", 46.0, 18.7),
];

/// ISO 4032: (size, across flats s, height m).
const HEX_NUT: [(&str, f64, f64); 14] = [
    ("M2", 4.0, 1.6),
    ("M2.5", 5.0, 2.0),
    ("M3", 5.5, 2.4),
    ("M4", 7.0, 3.2),
    ("M5", 8.0, 4.7),
    ("M6", 10.0, 5.2),
    ("M8", 13.0, 6.8),
    ("M10", 16.0, 8.4),
    ("M12", 18.0, 10.8),
    ("M14", 21.0, 12.8),
    ("M16", 24.0, 14.8),
    ("M20", 30.0, 18.0),
    ("M24", 36.0, 21.5),
    ("M30", 46.0, 25.6),
];

/// ISO 7089: (size, inner d1, outer d2, thickness h).
const WASHER: [(&str, f64, f64, f64); 14] = [
    ("M2", 2.2, 5.0, 0.3),
    ("M2.5", 2.7, 6.0, 0.5),
    ("M3", 3.2, 7.0, 0.5),
    ("M4", 4.3, 9.0, 0.8),
    ("M5", 5.3, 10.0, 1.0),
    ("M6", 6.4, 12.0, 1.6),
    ("M8", 8.4, 16.0, 1.6),
    ("M10", 10.5, 20.0, 2.0),
    ("M12", 13.0, 24.0, 2.5),
    ("M14", 15.0, 28.0, 2.5),
    ("M16", 17.0, 30.0, 3.0),
    ("M20", 21.0, 37.0, 3.0),
    ("M24", 25.0, 44.0, 4.0),
    ("M30", 31.0, 56.0, 4.0),
];

/// ISO 4026: (size, socket s, socket depth t).
const SET_SCREW: [(&str, f64, f64); 7] =
    [("M3", 1.5, 1.2), ("M4", 2.0, 1.5), ("M5", 2.5, 2.0), ("M6", 3.0, 2.0), ("M8", 4.0, 3.0), ("M10", 5.0, 4.0), ("M12", 6.0, 4.8)];

/// ISO 7380-1 button head, hex socket: (size, head diameter dk, height k, socket s, depth t).
const BUTTON: [(&str, f64, f64, f64, f64); 8] = [
    ("M3", 5.7, 1.65, 2.0, 1.04),
    ("M4", 7.6, 2.2, 2.5, 1.3),
    ("M5", 9.5, 2.75, 3.0, 1.56),
    ("M6", 10.5, 3.3, 4.0, 2.08),
    ("M8", 14.0, 4.4, 5.0, 2.6),
    ("M10", 17.5, 5.5, 6.0, 3.12),
    ("M12", 21.0, 6.6, 8.0, 4.16),
    ("M16", 28.0, 8.8, 10.0, 5.2),
];

/// ISO 7380-2 button head with collar: (size, dk, k, collar dc, collar thickness c, s, t).
const COLLAR: [(&str, f64, f64, f64, f64, f64, f64); 8] = [
    ("M3", 5.2, 1.65, 6.9, 0.7, 2.0, 1.04),
    ("M4", 7.2, 2.2, 9.4, 0.8, 2.5, 1.3),
    ("M5", 8.8, 2.75, 11.8, 1.0, 3.0, 1.56),
    ("M6", 10.0, 3.3, 13.6, 1.2, 4.0, 2.08),
    ("M8", 13.2, 4.4, 17.8, 1.5, 5.0, 2.6),
    ("M10", 16.5, 5.5, 21.9, 2.0, 6.0, 3.12),
    ("M12", 19.4, 6.6, 26.0, 2.4, 8.0, 4.16),
    ("M16", 26.0, 8.8, 34.0, 2.8, 10.0, 5.2),
];

/// ISO 10642 countersunk hex socket, 90°: (size, theoretical dk, k, s, t).
const CSK_SOCKET: [(&str, f64, f64, f64, f64); 9] = [
    ("M3", 6.72, 1.86, 2.0, 1.1),
    ("M4", 8.96, 2.48, 2.5, 1.5),
    ("M5", 11.2, 3.1, 3.0, 1.9),
    ("M6", 13.44, 3.72, 4.0, 2.2),
    ("M8", 17.92, 4.96, 5.0, 3.0),
    ("M10", 22.4, 6.2, 6.0, 3.6),
    ("M12", 26.88, 7.44, 8.0, 4.3),
    ("M16", 33.6, 8.8, 10.0, 4.8),
    ("M20", 40.32, 10.16, 12.0, 5.6),
];

/// DIN 7984 low head socket cap: (size, dk, k, s, t).
const LOW_HEAD: [(&str, f64, f64, f64, f64); 9] = [
    ("M3", 5.5, 2.0, 2.0, 1.5),
    ("M4", 7.0, 2.8, 2.5, 2.3),
    ("M5", 8.5, 3.5, 3.0, 2.7),
    ("M6", 10.0, 4.0, 4.0, 3.0),
    ("M8", 13.0, 5.0, 5.0, 3.8),
    ("M10", 16.0, 6.0, 7.0, 4.5),
    ("M12", 18.0, 7.0, 8.0, 5.0),
    ("M16", 24.0, 9.0, 12.0, 5.5),
    ("M20", 30.0, 11.0, 14.0, 7.5),
];

/// Pan head, ISO 7045 (cross recess H) and ISO 14583 (hexalobular), one head: (size, dk, k,
/// cross recess size PH, its diameter m, its depth, hexalobular size).
const PAN: [(&str, f64, f64, u8, f64, f64, &str); 6] = [
    ("M3", 5.6, 2.4, 1, 3.0, 1.6, "T10"),
    ("M4", 8.0, 3.1, 2, 4.4, 2.15, "T20"),
    ("M5", 9.5, 3.7, 2, 4.9, 2.65, "T25"),
    ("M6", 12.0, 4.6, 3, 6.9, 3.35, "T30"),
    ("M8", 16.0, 6.0, 4, 9.0, 4.3, "T45"),
    ("M10", 20.0, 7.5, 4, 10.1, 5.5, "T50"),
];

/// ISO 1580 pan head, slotted: (size, dk, k, slot width n, slot depth t).
const PAN_SLOTTED: [(&str, f64, f64, f64, f64); 6] = [
    ("M3", 5.6, 1.8, 0.8, 0.9),
    ("M4", 8.0, 2.4, 1.2, 1.2),
    ("M5", 9.5, 3.0, 1.2, 1.4),
    ("M6", 12.0, 3.6, 1.6, 1.6),
    ("M8", 16.0, 4.8, 2.0, 2.1),
    ("M10", 20.0, 6.0, 2.5, 2.6),
];

/// Cheese head: (size, dk, k slotted (ISO 1207), k hexalobular (ISO 14580), slot n, slot t,
/// hexalobular size).
const CHEESE: [(&str, f64, f64, f64, f64, f64, &str); 6] = [
    ("M3", 5.5, 2.0, 2.4, 0.8, 1.0, "T10"),
    ("M4", 7.0, 2.6, 3.1, 1.2, 1.3, "T20"),
    ("M5", 8.5, 3.3, 3.65, 1.2, 1.5, "T25"),
    ("M6", 10.0, 3.9, 4.4, 1.6, 1.8, "T30"),
    ("M8", 13.0, 5.0, 5.8, 2.0, 2.2, "T45"),
    ("M10", 16.0, 6.0, 6.9, 2.5, 2.6, "T50"),
];

/// Countersunk, 90°, one head for ISO 2009 (slotted), ISO 7046-1 (cross recess H) and ISO 14581
/// (hexalobular): (size, theoretical dk, k, slot n, slot t, PH, recess m, recess depth,
/// hexalobular size).
const CSK: [(&str, f64, f64, f64, f64, u8, f64, f64, &str); 6] = [
    ("M3", 6.3, 1.65, 0.8, 0.72, 1, 3.2, 1.9, "T10"),
    ("M4", 9.4, 2.7, 1.2, 1.15, 2, 4.6, 2.35, "T20"),
    ("M5", 10.4, 2.7, 1.2, 1.25, 2, 5.2, 2.95, "T25"),
    ("M6", 12.6, 3.3, 1.6, 1.4, 3, 6.8, 3.25, "T30"),
    ("M8", 17.3, 4.65, 2.0, 2.05, 4, 8.9, 4.3, "T45"),
    ("M10", 20.0, 5.0, 2.5, 2.3, 4, 10.0, 5.4, "T50"),
];

/// ISO 4762 / ISO 14579 socket cap: the hexalobular size by thread.
const SHCS_LOBE: [(&str, &str); 8] =
    [("M3", "T10"), ("M4", "T20"), ("M5", "T25"), ("M6", "T30"), ("M8", "T45"), ("M10", "T50"), ("M12", "T55"), ("M16", "T70")];

/// ISO 10664 hexalobular recesses: (size, across the lobes' points A, across their roots B).
const LOBES: [(&str, f64, f64); 13] = [
    ("T6", 1.75, 1.27),
    ("T8", 2.4, 1.75),
    ("T10", 2.8, 2.05),
    ("T15", 3.35, 2.4),
    ("T20", 3.95, 2.85),
    ("T25", 4.5, 3.25),
    ("T30", 5.6, 4.05),
    ("T40", 6.75, 4.85),
    ("T45", 7.93, 5.64),
    ("T50", 8.95, 6.45),
    ("T55", 11.35, 8.05),
    ("T60", 13.45, 9.6),
    ("T70", 15.7, 11.2),
];

/// Cross recess type H (ISO 4757): the width of its arms by size (PH1…PH4).
const CROSS_WIDTH: [f64; 4] = [0.97, 1.47, 2.41, 3.48];

/// Set screw points: (size, cone tip dt (ISO 4027), dog dp and length z (ISO 4028, short), cup
/// dz (ISO 4029)).
const SET_POINTS: [(&str, f64, f64, f64, f64); 7] = [
    ("M3", 0.75, 2.0, 1.0, 1.4),
    ("M4", 1.0, 2.5, 1.25, 2.0),
    ("M5", 1.25, 3.5, 1.5, 2.5),
    ("M6", 1.5, 4.0, 1.75, 3.0),
    ("M8", 2.0, 5.5, 2.25, 5.0),
    ("M10", 2.5, 7.0, 2.75, 6.0),
    ("M12", 3.0, 8.5, 3.25, 8.0),
];

/// Screw drives: (id, name).
const DRIVES: [(&str, &str); 4] = [("hex", "Hex socket"), ("hexalobular", "Hexalobular (TX)"), ("cross", "Cross recess (PH)"), ("slot", "Slotted")];

/// The drives a screw family comes with, the first its default.
fn screw_drives(family: &str) -> &'static [&'static str] {
    match family {
        "socket_head_cap_screw" => &["hex", "hexalobular"],
        "low_head_cap_screw" | "button_head_screw" | "flanged_button_head_screw" | "countersunk_socket_screw" => &["hex"],
        "countersunk_screw" => &["cross", "slot", "hexalobular"],
        "pan_head_screw" => &["cross", "hexalobular"],
        "pan_head_slotted_screw" => &["slot"],
        "cheese_head_screw" => &["slot", "hexalobular"],
        _ => &[],
    }
}

/// Parts with a thread that can be modelled (cut into the part) as well as cosmetic.
fn threaded(family: &str) -> bool {
    !screw_drives(family).is_empty() || matches!(family, "hex_bolt" | "hex_nut") || family.starts_with("set_screw")
}

/// A hexalobular recess's outline sizes.
fn lobe(size: &str) -> Option<(f64, f64)> {
    LOBES.iter().find(|l| l.0 == size).map(|l| (l.1, l.2))
}

/// ISO 8734 diameters.
const DOWEL: [f64; 9] = [2.0, 3.0, 4.0, 5.0, 6.0, 8.0, 10.0, 12.0, 16.0];

/// Deep groove ball bearings, open (ISO 15 boundary dimensions): (designation, bore, outside
/// diameter, width), by series: 68 thin, 69, 60, 62 and 63 heavy, the miniature sizes first.
/// The series is the designation's first two digits.
const BEARINGS: [(&str, f64, f64, f64); 91] = [
    ("681", 1.0, 3.0, 1.0),
    ("682", 2.0, 5.0, 1.5),
    ("683", 3.0, 7.0, 2.0),
    ("684", 4.0, 9.0, 2.5),
    ("685", 5.0, 11.0, 3.0),
    ("686", 6.0, 13.0, 3.5),
    ("687", 7.0, 14.0, 3.5),
    ("688", 8.0, 16.0, 4.0),
    ("689", 9.0, 17.0, 4.0),
    ("6800", 10.0, 19.0, 5.0),
    ("6801", 12.0, 21.0, 5.0),
    ("6802", 15.0, 24.0, 5.0),
    ("6803", 17.0, 26.0, 5.0),
    ("6804", 20.0, 32.0, 7.0),
    ("6805", 25.0, 37.0, 7.0),
    ("6806", 30.0, 42.0, 7.0),
    ("6807", 35.0, 47.0, 7.0),
    ("6808", 40.0, 52.0, 7.0),
    ("6809", 45.0, 58.0, 7.0),
    ("6810", 50.0, 65.0, 7.0),
    ("692", 2.0, 6.0, 2.3),
    ("693", 3.0, 8.0, 3.0),
    ("694", 4.0, 11.0, 4.0),
    ("695", 5.0, 13.0, 4.0),
    ("696", 6.0, 15.0, 5.0),
    ("697", 7.0, 17.0, 5.0),
    ("698", 8.0, 19.0, 6.0),
    ("699", 9.0, 20.0, 6.0),
    ("6900", 10.0, 22.0, 6.0),
    ("6901", 12.0, 24.0, 6.0),
    ("6902", 15.0, 28.0, 7.0),
    ("6903", 17.0, 30.0, 7.0),
    ("6904", 20.0, 37.0, 9.0),
    ("6905", 25.0, 42.0, 9.0),
    ("6906", 30.0, 47.0, 9.0),
    ("6907", 35.0, 55.0, 10.0),
    ("6908", 40.0, 62.0, 12.0),
    ("6909", 45.0, 68.0, 12.0),
    ("6910", 50.0, 72.0, 12.0),
    ("604", 4.0, 12.0, 4.0),
    ("605", 5.0, 14.0, 5.0),
    ("606", 6.0, 17.0, 6.0),
    ("607", 7.0, 19.0, 6.0),
    ("608", 8.0, 22.0, 7.0),
    ("609", 9.0, 24.0, 7.0),
    ("6000", 10.0, 26.0, 8.0),
    ("6001", 12.0, 28.0, 8.0),
    ("6002", 15.0, 32.0, 9.0),
    ("6003", 17.0, 35.0, 10.0),
    ("6004", 20.0, 42.0, 12.0),
    ("6005", 25.0, 47.0, 12.0),
    ("6006", 30.0, 55.0, 13.0),
    ("6007", 35.0, 62.0, 14.0),
    ("6008", 40.0, 68.0, 15.0),
    ("6009", 45.0, 75.0, 16.0),
    ("6010", 50.0, 80.0, 16.0),
    ("623", 3.0, 10.0, 4.0),
    ("624", 4.0, 13.0, 5.0),
    ("625", 5.0, 16.0, 5.0),
    ("626", 6.0, 19.0, 6.0),
    ("627", 7.0, 22.0, 7.0),
    ("628", 8.0, 24.0, 8.0),
    ("629", 9.0, 26.0, 8.0),
    ("6200", 10.0, 30.0, 9.0),
    ("6201", 12.0, 32.0, 10.0),
    ("6202", 15.0, 35.0, 11.0),
    ("6203", 17.0, 40.0, 12.0),
    ("6204", 20.0, 47.0, 14.0),
    ("6205", 25.0, 52.0, 15.0),
    ("6206", 30.0, 62.0, 16.0),
    ("6207", 35.0, 72.0, 17.0),
    ("6208", 40.0, 80.0, 18.0),
    ("6209", 45.0, 85.0, 19.0),
    ("6210", 50.0, 90.0, 20.0),
    ("634", 4.0, 16.0, 5.0),
    ("635", 5.0, 19.0, 6.0),
    ("636", 6.0, 22.0, 7.0),
    ("637", 7.0, 26.0, 9.0),
    ("638", 8.0, 28.0, 9.0),
    ("639", 9.0, 30.0, 10.0),
    ("6300", 10.0, 35.0, 11.0),
    ("6301", 12.0, 37.0, 12.0),
    ("6302", 15.0, 42.0, 13.0),
    ("6303", 17.0, 47.0, 14.0),
    ("6304", 20.0, 52.0, 15.0),
    ("6305", 25.0, 62.0, 17.0),
    ("6306", 30.0, 72.0, 19.0),
    ("6307", 35.0, 80.0, 21.0),
    ("6308", 40.0, 90.0, 23.0),
    ("6309", 45.0, 100.0, 25.0),
    ("6310", 50.0, 110.0, 27.0),
];

/// Standard lengths for screws, bolts and pins (mm).
const LENGTHS: [f64; 22] =
    [4.0, 5.0, 6.0, 8.0, 10.0, 12.0, 16.0, 20.0, 25.0, 30.0, 35.0, 40.0, 45.0, 50.0, 55.0, 60.0, 65.0, 70.0, 80.0, 90.0, 100.0, 120.0];

fn metric(size: &str) -> Option<(f64, f64)> {
    METRIC.iter().find(|(s, _, _)| s.eq_ignore_ascii_case(size)).map(|(_, d, p)| (*d, *p))
}

fn default_length(d: f64) -> f64 {
    LENGTHS.iter().copied().find(|l| *l >= 2.5 * d).unwrap_or(100.0)
}

/// One part's shape, in its own frame.
#[derive(Clone, Debug, PartialEq)]
enum Shape {
    /// A head on z = 0 (a countersunk one below it, its top flush with z = 0), a shank down −z
    /// to −len, an optional drive in the head's top; the thread modelled or cosmetic.
    Screw { d: f64, pitch: f64, len: f64, head: Head, drive: Option<Drive>, modeled: bool },
    /// A hex prism with a tapping-drill hole and its thread (nut), z 0…m.
    Nut { d: f64, pitch: f64, s: f64, m: f64, modeled: bool },
    /// A ring (washer), z 0…h.
    Ring { d_in: f64, d_out: f64, h: f64 },
    /// A ball bearing's outline, z 0…b: inner and outer ring, the seals between them set in.
    Bearing { d: f64, od: f64, b: f64 },
    /// A cylinder down −z with a socket in its top and a point at its end (set screw), or plain
    /// (pin).
    Rod { d: f64, pitch: Option<f64>, len: f64, socket: Option<(f64, f64)>, point: Point, modeled: bool },
    /// An involute gear, `z` teeth of the transverse tooth system `tooth`, z 0…w, with a bore;
    /// `helix` its helix angle at the pitch circle (0: spur, positive: right hand), and a
    /// herringbone turns that way up to half its width and back the other way above.
    Gear { tooth: Involute, z: u32, w: f64, bore: f64, helix: f64, herringbone: bool },
    /// An internal ring gear: `z` teeth pointing in, cut from a cylinder of outside diameter
    /// `od` by the outline of their spaces (`space`: addendum and dedendum swapped), z 0…w.
    RingGear { space: Involute, z: u32, w: f64, od: f64 },
    /// A right-hand Archimedean worm: axial module `m`, pitch diameter `d1`, `starts` thread
    /// starts with straight 20° flanks in its axial section, z 0…len, with a bore.
    Worm { m: f64, d1: f64, starts: u32, len: f64, bore: f64 },
    /// A cycloidal reducer disc: `n` lobes rolling on n + 1 pins of radius `rr` on a circle of
    /// radius `r` with eccentricity `e`, z 0…w, with a bore for its eccentric bearing.
    Cycloid { r: f64, n: u32, e: f64, rr: f64, w: f64, bore: f64 },
    /// A timing pulley: `z` grooves (one arc of radius `rg`, `depth` deep) on the outside
    /// diameter, for a belt `w` wide: teeth `BELT_CLEARANCE` wider than the belt between two
    /// flanges `flange` thick, z 0…w + BELT_CLEARANCE + 2·flange.
    Pulley { p: f64, pld: f64, depth: f64, rg: f64, z: u32, w: f64, flange: f64, bore: f64 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Head {
    /// A cylinder (socket cap, cheese, low head).
    Round {
        dk: f64,
        k: f64,
    },
    Hex {
        s: f64,
        k: f64,
    },
    /// A dome on a short cylindrical rim `edge` high (button, pan).
    Dome {
        dk: f64,
        k: f64,
        edge: f64,
    },
    /// A dome on a collar `c` thick of diameter `dc` (flanged button).
    Collar {
        dk: f64,
        k: f64,
        dc: f64,
        c: f64,
    },
    /// 90° countersunk, below z = 0.
    Countersunk {
        dk: f64,
        k: f64,
    },
}

/// A drive cut into a screw's head from its top.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Drive {
    Hex {
        s: f64,
        t: f64,
    },
    /// Hexalobular, `a` across the lobes' points and `b` across their roots.
    Lobe {
        a: f64,
        b: f64,
        t: f64,
    },
    /// Cross recess: arms `w` wide reaching `m` across.
    Cross {
        m: f64,
        w: f64,
        t: f64,
    },
    /// A slot `n` wide across the whole head (`across`).
    Slot {
        n: f64,
        t: f64,
        across: f64,
    },
}

/// A set screw's end.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Point {
    Flat,
    Cone { dt: f64 },
    Dog { dp: f64, z: f64 },
    Cup { dz: f64 },
}

/// An involute tooth system in a gear's transverse plane: module `mt`, pressure angle `alpha`,
/// addendum `ha` and dedendum `hf` (mm).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Involute {
    mt: f64,
    alpha: f64,
    ha: f64,
    hf: f64,
}

impl Involute {
    /// The ISO 53 basic rack (20°, addendum m, dedendum 1.25 m) of module `m` in the normal
    /// plane of teeth at helix angle `helix` (0: spur).
    fn iso53(m: f64, helix: f64) -> Self {
        let c = helix.cos();
        Involute { mt: m / c, alpha: (20f64.to_radians().tan() / c).atan(), ha: m, hf: 1.25 * m }
    }

    fn pitch_radius(&self, z: u32) -> f64 {
        self.mt * f64::from(z) / 2.0
    }

    fn tip_radius(&self, z: u32) -> f64 {
        self.pitch_radius(z) + self.ha
    }

    fn root_radius(&self, z: u32) -> f64 {
        self.pitch_radius(z) - self.hf
    }
}

/// The default width of a gear of module `m` (about six modules).
fn gear_width(m: f64) -> f64 {
    GEAR_WIDTHS.iter().copied().find(|w| *w >= 6.0 * m).unwrap_or(30.0)
}

/// A gear family's shape.
fn gear_shape(kind: GearKind, m: f64, size: &str, length: Option<f64>, extra: Extra) -> std::result::Result<Shape, String> {
    let helical = matches!(kind, GearKind::Helical | GearKind::Herringbone);
    if extra.helix.is_some() && !helical {
        return Err("only helical and herringbone gears take a helix angle".into());
    }
    if extra.starts.is_some() && kind != GearKind::WormWheel {
        return Err("only worm wheels take `starts` (a worm's starts are its size)".into());
    }
    if kind == GearKind::Worm {
        let starts = size.trim().trim_end_matches("-start").trim().parse::<u32>().ok().filter(|n| (1..=4).contains(n));
        let starts = starts.ok_or_else(|| format!("no worm with `{size}` starts (1…4)"))?;
        let d1 = worm_d1(m);
        // By default as long as one sweep makes it (TURNS_PER_SWEEP leads).
        let one_sweep = TURNS_PER_SWEEP * std::f64::consts::PI * m * f64::from(starts);
        let len = length.unwrap_or_else(|| WORM_LENGTHS.iter().copied().rev().find(|l| *l <= one_sweep).unwrap_or(WORM_LENGTHS[0]));
        let turns = len / (std::f64::consts::PI * m * f64::from(starts));
        if turns > WORM_MAX_TURNS {
            return Err(format!(
                "a worm this long makes {turns:.1} turns of thread; at most {:.0} mm for this module and starts",
                (WORM_MAX_TURNS * std::f64::consts::PI * m * f64::from(starts)).floor()
            ));
        }
        return Ok(Shape::Worm { m, d1, starts, len, bore: bore_for(d1 / 2.0 - 1.25 * m) });
    }
    let z = teeth(size).ok_or_else(|| format!("no tooth count `{size}` (6…200)"))?;
    let w = length.unwrap_or_else(|| gear_width(m));
    Ok(match kind {
        GearKind::Ring => {
            if z < 20 {
                return Err("a ring gear needs at least 20 teeth".into());
            }
            let space = Involute { ha: 1.25 * m, hf: m, ..Involute::iso53(m, 0.0) };
            // A rim of three modules (at least 3 mm) outside the tooth spaces, to the next mm.
            let od = (2.0 * (space.tip_radius(z) + (3.0 * m).max(3.0))).ceil();
            Shape::RingGear { space, z, w, od }
        }
        GearKind::WormWheel => {
            // Its transverse section is the worm's axial section: module m, 20°, helical at the
            // worm's lead angle, the same hand.
            let starts = extra.starts.unwrap_or(1);
            let tooth = Involute::iso53(m, 0.0);
            let w = length.unwrap_or_else(|| GEAR_WIDTHS.iter().copied().find(|w| *w >= 0.75 * worm_d1(m)).unwrap_or(30.0));
            Shape::Gear { tooth, z, w, bore: bore_for(tooth.root_radius(z)), helix: lead_angle(m, starts), herringbone: false }
        }
        _ => {
            let helix = if helical { extra.helix.unwrap_or(HELIX_DEFAULT.to_radians()) } else { 0.0 };
            let deg = helix.abs().to_degrees();
            if helical && !(deg >= HELIX_RANGE.0 - 1e-9 && deg <= HELIX_RANGE.1 + 1e-9) {
                return Err(format!("the helix angle must be {}…{} deg (negative: left hand)", HELIX_RANGE.0, HELIX_RANGE.1));
            }
            let tooth = Involute::iso53(m, helix);
            Shape::Gear { tooth, z, w, bore: bore_for(tooth.root_radius(z)), helix, herringbone: kind == GearKind::Herringbone }
        }
    })
}

/// A cycloidal disc family's shape: `size` the number of lobes.
fn cycloid_shape(d: f64, size: &str, length: Option<f64>) -> std::result::Result<Shape, String> {
    let n = size.trim().parse::<u32>().ok().filter(|n| (6..=60).contains(n)).ok_or_else(|| format!("no lobe count `{size}` (6…60)"))?;
    let r = d / 2.0;
    let rr = cycloid_pin(r, n).ok_or_else(|| format!("{n} lobes leave no room for pins on a Ø{d} pin circle"))? / 2.0;
    let e = cycloid_e(r, n);
    Ok(Shape::Cycloid { r, n, e, rr, w: length.unwrap_or(DISC_WIDTHS[3]), bore: bore_for(r - e - rr) })
}

/// A screw family's shape: its head from its standard's table, its drive (`extra.drive`, else
/// the family's first).
fn screw_shape(family: &str, size: &str, length: Option<f64>, extra: &Extra) -> std::result::Result<Shape, String> {
    let need = |what: &str| format!("no {what} size `{size}` (parts.library lists them)");
    let (d, pitch) = metric(size).ok_or_else(|| need("metric"))?;
    let drives = screw_drives(family);
    let drive = extra.drive.as_deref().or(drives.first().copied()).unwrap_or("hex");
    if !drives.contains(&drive) {
        return Err(format!("this screw comes with {}", drives.join(", ")));
    }
    fn find(t: &[(&'static str, f64, f64, f64, f64)], size: &str) -> Option<(&'static str, f64, f64, f64, f64)> {
        t.iter().find(|r| r.0.eq_ignore_ascii_case(size)).copied()
    }
    let row = |t: &[(&'static str, f64, f64, f64, f64)], std: &str| find(t, size).ok_or_else(|| need(std));
    let lobed = |t: &str, depth: f64| lobe(t).map(|(a, b)| Drive::Lobe { a, b, t: depth }).ok_or_else(|| need("hexalobular"));
    let cross = |ph: u8, m: f64, t: f64| Drive::Cross { m, w: CROSS_WIDTH.get(usize::from(ph.max(1)) - 1).copied().unwrap_or(1.0), t };
    let (head, drv) = match family {
        "socket_head_cap_screw" => {
            let (_, dk, k, s, t) = row(&SHCS, "ISO 4762")?;
            let drv = if drive == "hexalobular" {
                let tn = SHCS_LOBE.iter().find(|r| r.0.eq_ignore_ascii_case(size)).map(|r| r.1).ok_or_else(|| need("ISO 14579"))?;
                lobed(tn, t)?
            } else {
                Drive::Hex { s, t }
            };
            (Head::Round { dk, k }, drv)
        }
        "low_head_cap_screw" => {
            let (_, dk, k, s, t) = row(&LOW_HEAD, "DIN 7984")?;
            (Head::Round { dk, k }, Drive::Hex { s, t })
        }
        "button_head_screw" => {
            let (_, dk, k, s, t) = row(&BUTTON, "ISO 7380-1")?;
            (Head::Dome { dk, k, edge: 0.12 * k }, Drive::Hex { s, t })
        }
        "flanged_button_head_screw" => {
            let (_, dk, k, dc, c, s, t) = COLLAR.iter().find(|r| r.0.eq_ignore_ascii_case(size)).copied().ok_or_else(|| need("ISO 7380-2"))?;
            (Head::Collar { dk, k, dc, c }, Drive::Hex { s, t })
        }
        "countersunk_socket_screw" => {
            let (_, dk, k, s, t) = row(&CSK_SOCKET, "ISO 10642")?;
            (Head::Countersunk { dk, k }, Drive::Hex { s, t })
        }
        "countersunk_screw" => {
            let (_, dk, k, n, t, ph, m, depth, tn) = CSK.iter().find(|r| r.0.eq_ignore_ascii_case(size)).copied().ok_or_else(|| need("ISO 2009"))?;
            let drv = match drive {
                "slot" => Drive::Slot { n, t, across: dk + 1.0 },
                "hexalobular" => lobed(tn, depth)?,
                _ => cross(ph, m, depth),
            };
            (Head::Countersunk { dk, k }, drv)
        }
        "pan_head_screw" => {
            let (_, dk, k, ph, m, depth, tn) = PAN.iter().find(|r| r.0.eq_ignore_ascii_case(size)).copied().ok_or_else(|| need("ISO 7045"))?;
            let drv = if drive == "hexalobular" { lobed(tn, depth)? } else { cross(ph, m, depth) };
            (Head::Dome { dk, k, edge: 0.45 * k }, drv)
        }
        "pan_head_slotted_screw" => {
            let (_, dk, k, n, t) = row(&PAN_SLOTTED, "ISO 1580")?;
            (Head::Dome { dk, k, edge: 0.6 * k }, Drive::Slot { n, t, across: dk + 1.0 })
        }
        "cheese_head_screw" => {
            let (_, dk, k_slot, k_lobe, n, t, tn) =
                CHEESE.iter().find(|r| r.0.eq_ignore_ascii_case(size)).copied().ok_or_else(|| need("ISO 1207"))?;
            if drive == "hexalobular" {
                (Head::Round { dk, k: k_lobe }, lobed(tn, 0.55 * k_lobe)?)
            } else {
                (Head::Round { dk, k: k_slot }, Drive::Slot { n, t, across: dk + 1.0 })
            }
        }
        f => return Err(format!("unknown part family `{f}` (parts.library lists them)")),
    };
    let len = length.unwrap_or_else(|| default_length(d));
    if let Head::Countersunk { k, .. } = head
        && len <= k + pitch
    {
        return Err(format!("a countersunk {size} screw is longer than its head: more than {:.1} mm", k + pitch));
    }
    Ok(Shape::Screw { d, pitch, len, head, drive: Some(drv), modeled: extra.modeled })
}

fn shape(family: &str, size: &str, length: Option<f64>, extra: Extra) -> std::result::Result<Shape, String> {
    let need = |what: &str| format!("no {what} size `{size}` (parts.library lists them)");
    let len = |d: f64| length.unwrap_or_else(|| default_length(d));
    if let Some((kind, m)) = gear_family(family) {
        return gear_shape(kind, m, size, length, extra);
    }
    if extra.helix.is_some() || extra.starts.is_some() {
        return Err("only gears take a helix angle or starts".into());
    }
    if extra.drive.is_some() && screw_drives(family).is_empty() {
        return Err("this part has no drive".into());
    }
    if extra.modeled && !threaded(family) {
        return Err("this part has no thread to model".into());
    }
    if !screw_drives(family).is_empty() {
        return screw_shape(family, size, length, &extra);
    }
    if let Some(&(_, d)) = CYCLOIDS.iter().find(|c| c.0 == family) {
        return cycloid_shape(d, size, length);
    }
    Ok(match family {
        "hex_bolt" => {
            let (d, pitch) = metric(size).ok_or_else(|| need("metric"))?;
            let (_, s, k) = HEX_BOLT.iter().find(|r| r.0.eq_ignore_ascii_case(size)).copied().ok_or_else(|| need("ISO 4014"))?;
            Shape::Screw { d, pitch, len: len(d), head: Head::Hex { s, k }, drive: None, modeled: extra.modeled }
        }
        "hex_nut" => {
            let (d, pitch) = metric(size).ok_or_else(|| need("metric"))?;
            let (_, s, m) = HEX_NUT.iter().find(|r| r.0.eq_ignore_ascii_case(size)).copied().ok_or_else(|| need("ISO 4032"))?;
            Shape::Nut { d, pitch, s, m, modeled: extra.modeled }
        }
        "washer" => {
            let (_, d1, d2, h) = WASHER.iter().find(|r| r.0.eq_ignore_ascii_case(size)).copied().ok_or_else(|| need("ISO 7089"))?;
            Shape::Ring { d_in: d1, d_out: d2, h }
        }
        f if f.starts_with("set_screw") => {
            let (d, pitch) = metric(size).ok_or_else(|| need("metric"))?;
            let (_, s, t) = SET_SCREW.iter().find(|r| r.0.eq_ignore_ascii_case(size)).copied().ok_or_else(|| need("ISO 4026"))?;
            let pts = SET_POINTS.iter().find(|r| r.0.eq_ignore_ascii_case(size)).copied();
            let point = match (f, pts) {
                ("set_screw_cone", Some((_, dt, ..))) => Point::Cone { dt },
                ("set_screw_dog", Some((_, _, dp, z, _))) => Point::Dog { dp, z },
                ("set_screw_cup", Some((.., dz))) => Point::Cup { dz },
                ("set_screw", _) => Point::Flat,
                _ => return Err(need("set screw")),
            };
            let len = length.unwrap_or_else(|| LENGTHS.iter().copied().find(|l| *l >= d).unwrap_or(d));
            // Room for the socket and the point: at least the socket, the dog and a thread pitch.
            let point_len = match point {
                Point::Dog { z, .. } => z,
                Point::Cone { dt } => (d - dt) / 2.0,
                _ => 0.0,
            };
            if len < t + point_len + pitch {
                return Err(format!("an {size} set screw of this point is at least {:.1} mm long", t + point_len + pitch));
            }
            Shape::Rod { d, pitch: Some(pitch), len, socket: Some((s, t)), point, modeled: extra.modeled }
        }
        "dowel_pin" => {
            let d: f64 = size.trim_start_matches(['d', 'D', 'Ø']).trim().parse().map_err(|_| need("dowel pin"))?;
            if !DOWEL.contains(&d) {
                return Err(need("ISO 8734"));
            }
            Shape::Rod { d, pitch: None, len: len(d * 1.6), socket: None, point: Point::Flat, modeled: false }
        }
        "bearing" => {
            let (_, d, od, b) = BEARINGS.iter().find(|r| r.0 == size.trim()).copied().ok_or_else(|| need("bearing"))?;
            Shape::Bearing { d, od, b }
        }
        f if f.starts_with("pulley_") => {
            let (_, p, pld, depth, rg, flange, widths) =
                PULLEYS.iter().find(|r| r.0 == f).copied().ok_or_else(|| format!("unknown part family `{f}`"))?;
            let z = teeth(size).ok_or_else(|| need("tooth count (6…200)"))?;
            let w = length.unwrap_or(widths[0]);
            let ro = f64::from(z) * p / std::f64::consts::PI / 2.0 - pld;
            Shape::Pulley { p, pld, depth, rg, z, w, flange, bore: bore_for(ro - depth) }
        }
        o => return Err(format!("unknown part family `{o}` (parts.library lists them)")),
    })
}

fn run(s: &mut Session, id: &str, p: Value) -> Result<Value> {
    s.execute(id, &p)
}

/// A hex prism (across flats `af`) from z0 to z1, joined or cut.
fn hex(s: &mut Session, af: f64, z0: f64, z1: f64, op: &str) -> Result<()> {
    run(s, "sketch.create", json!({"plane": "XY", "offset": format!("{z0} mm")}))?;
    run(s, "sketch.polygon.circumscribed", json!({"center": [0, 0], "radius": af / 2.0, "sides": 6}))?;
    fix_all(s)?;
    run(s, "sketch.finish", json!({}))?;
    let (dist, dir) = if z1 >= z0 { (z1 - z0, "positive") } else { (z0 - z1, "negative") };
    run(s, "solid.extrude", json!({"distance": format!("{dist} mm"), "direction": dir, "operation": op}))?;
    Ok(())
}

fn cyl(s: &mut Session, d: f64, z0: f64, z1: f64, op: &str) -> Result<()> {
    run(
        s,
        "solid.cylinder",
        json!({"diameter": format!("{d} mm"), "height": format!("{} mm", z1 - z0), "base": [0, 0, z0], "axis": [0, 0, 1], "operation": op}),
    )?;
    Ok(())
}

/// A solid of revolution about z from a closed (r, z) outline, as one body or joined.
fn revolve(s: &mut Session, pts: &[(f64, f64)], op: &str) -> Result<()> {
    run(s, "sketch.create", json!({"plane": "XZ"}))?;
    let points: Vec<[f64; 2]> = pts.iter().map(|(r, z)| [*r, -*z]).collect();
    run(s, "sketch.line", json!({"points": points, "closed": true}))?;
    fix_all(s)?;
    run(s, "sketch.finish", json!({}))?;
    run(s, "solid.revolve", json!({"axis": "y", "operation": op}))?;
    Ok(())
}

/// Fix every point and curve of the sketch being drawn: a standard part's outline is its size, so
/// its sketches are fully defined and a check of the design finds nothing left free in them.
fn fix_all(s: &mut Session) -> Result<()> {
    let sketch = s.active_sketch.and_then(|id| s.doc.features.iter().find(|f| f.id == id)).and_then(|f| match &f.kind {
        solidcraft_doc::FeatureKind::Sketch { sketch, .. } => Some(sketch),
        _ => None,
    });
    let entities: Vec<String> =
        sketch.map(|sk| sk.points.iter().map(|p| p.id.clone()).chain(sk.curves.iter().map(|c| c.id.clone())).collect()).unwrap_or_default();
    if !entities.is_empty() {
        run(s, "sketch.constraint.fix", json!({"entities": entities, "fixed": true}))?;
    }
    Ok(())
}

/// A solid of revolution about z from a closed outline of lines and arcs in (r, z): drawn on
/// the XZ plane (whose sketch y runs down z, so an arc's turn reverses) and revolved.
fn revolve_outline(s: &mut Session, segs: &[Seg], op: &str) -> Result<()> {
    let flip = |p: &[f64; 2]| [p[0], -p[1]];
    let mapped: Vec<Seg> = segs
        .iter()
        .map(|g| match g {
            Seg::Line(a, b) => Seg::Line(flip(a), flip(b)),
            Seg::Spline(pts) => Seg::Spline(pts.iter().map(flip).collect()),
            Seg::Arc { c, a, b } => Seg::Arc { c: flip(c), a: flip(b), b: flip(a) },
        })
        .collect();
    outline_sketch_on(s, "XZ", &mapped, &[], 0.0)?;
    run(s, "solid.revolve", json!({"axis": "y", "operation": op}))?;
    Ok(())
}

/// A hexalobular (six-lobe) socket's outline, `a` across its lobes' points and `b` across the
/// lobes' roots (ISO 10664): six lobe arcs joined by six concave arcs, each tangent to both
/// neighbours.
fn hexalobular_outline(a: f64, b: f64) -> Vec<Seg> {
    let re = 0.1 * a;
    let ca = a / 2.0 - re;
    // The concave arcs' radius: their centres at b/2 + ri at 30° between lobes, touching both.
    let s3 = 3f64.sqrt();
    let ri = (re * re - ca * ca - b * b / 4.0 + s3 / 2.0 * ca * b) / (b - s3 * ca - 2.0 * re);
    let lobe = |k: f64| polar(ca, k * std::f64::consts::FRAC_PI_3);
    let hollow = |k: f64| polar(b / 2.0 + ri, (k + 0.5) * std::f64::consts::FRAC_PI_3);
    let touch = |c1: [f64; 2], r1: f64, c2: [f64; 2]| {
        let (dx, dy) = (c2[0] - c1[0], c2[1] - c1[1]);
        let l = dx.hypot(dy).max(1e-12);
        [c1[0] + r1 * dx / l, c1[1] + r1 * dy / l]
    };
    let mut out = Vec::new();
    for k in 0..6 {
        let k = f64::from(k);
        let (c, prev, next) = (lobe(k), hollow(k - 1.0), hollow(k));
        let tip = polar(ca + re, k * std::f64::consts::FRAC_PI_3);
        out.push(arc3(touch(c, re, prev), tip, touch(c, re, next)));
        let low = polar(b / 2.0, (k + 0.5) * std::f64::consts::FRAC_PI_3);
        out.push(arc3(touch(c, re, next), low, touch(lobe(k + 1.0), re, next)));
    }
    out
}

/// A cross recess's outline (simplified type H): a plus of arms `w` wide reaching `m` across.
fn cross_outline(m: f64, w: f64) -> Vec<Seg> {
    let (h, e) = (w / 2.0, m / 2.0);
    let pts = [[e, -h], [e, h], [h, h], [h, e], [-h, e], [-h, h], [-e, h], [-e, -h], [-h, -h], [-h, -e], [h, -e], [h, -h]];
    (0..pts.len()).map(|i| Seg::Line(pts[i], pts[(i + 1) % pts.len()])).collect()
}

/// An ISO metric thread on the cylindrical face at `at`: cut into the part when `modeled` (it
/// takes seconds), else cosmetic (listed, the model unchanged), which is also what a thread that
/// cannot be modelled falls back to.
fn thread(s: &mut Session, d: f64, pitch: f64, at: Vec3, modeled: bool) {
    let designation = format!("M{d}x{pitch}");
    if modeled && run(s, "solid.thread", json!({"face": at, "designation": designation, "modeled": true})).is_ok() {
        return;
    }
    let _ = run(s, "solid.thread", json!({"face": at, "designation": designation}));
}

/// A screw's outline in (r, z) with a domed head: the shank down to −len, an optional collar
/// (`dc` across, `c` thick), and a dome `dk` across, `k` high, on a rim `edge` high.
fn domed(d: f64, len: f64, dk: f64, k: f64, dc: f64, c: f64, edge: f64) -> Vec<Seg> {
    let (rk, edge) = (dk / 2.0, edge.min(0.9 * k));
    // The dome's sphere: centred on the axis, through the rim's top and the head's top.
    let zc = (k * k - edge * edge - rk * rk) / (2.0 * (k - edge));
    let (a0, a1) = ((edge - zc).atan2(rk), std::f64::consts::FRAC_PI_2);
    let rad = k - zc;
    let mid = [rad * ((a0 + a1) / 2.0).cos(), zc + rad * ((a0 + a1) / 2.0).sin()];
    let mut pts = vec![[0.0, -len], [d / 2.0, -len], [d / 2.0, 0.0]];
    if dc > 0.0 {
        pts.extend([[dc / 2.0, 0.0], [dc / 2.0, c], [rk, c]]);
    } else {
        pts.extend([[rk, 0.0], [rk, edge]]);
    }
    let mut segs: Vec<Seg> = pts.windows(2).map(|w| Seg::Line(w[0], w[1])).collect();
    let rim = pts.last().copied().unwrap_or([rk, edge]);
    segs.push(arc3(rim, mid, [0.0, k]));
    segs.push(Seg::Line([0.0, k], [0.0, -len]));
    segs
}

/// Cut a drive into a screw's head from its top at z = `top`.
fn cut_drive(s: &mut Session, drive: Drive, top: f64) -> Result<()> {
    let (segs, depth) = match drive {
        Drive::Hex { s: af, t } => return hex(s, af, top, top - t, "cut"),
        Drive::Lobe { a, b, t } => (hexalobular_outline(a, b), t),
        Drive::Cross { m, w, t } => (cross_outline(m, w), t),
        Drive::Slot { n, t, across } => {
            let (h, e) = (n / 2.0, across / 2.0);
            let pts = [[-e, -h], [e, -h], [e, h], [-e, h]];
            ((0..4).map(|i| Seg::Line(pts[i], pts[(i + 1) % 4])).collect(), t)
        }
    };
    outline_solid(s, &segs, &[], 0.0, top - depth, depth + 1.0, "cut")
}

/// One piece of a closed outline in the XY sketch: `Arc` runs counter-clockwise start -> end.
enum Seg {
    Line([f64; 2], [f64; 2]),
    Spline(Vec<[f64; 2]>),
    Arc { c: [f64; 2], a: [f64; 2], b: [f64; 2] },
}

fn polar(r: f64, a: f64) -> [f64; 2] {
    [r * a.cos(), r * a.sin()]
}

fn inv(a: f64) -> f64 {
    a.tan() - a
}

/// The outline of an involute gear in its transverse plane: per tooth a radial line from the
/// root to the base circle (when the root is inside it), the involute flank as a fit spline, the
/// tip arc, the other flank and the root arc to the next tooth. No undercut or root fillet is
/// modelled.
fn gear_outline(t: Involute, z: u32, flanks: Flanks) -> std::result::Result<Vec<Seg>, String> {
    let zf = f64::from(z);
    let (rp, rb) = (t.pitch_radius(z), t.pitch_radius(z) * t.alpha.cos());
    // Half the tooth's angle at radius r (r ≥ rb): π/2z at the pitch circle, along the involute.
    let psi = |r: f64| std::f64::consts::PI / (2.0 * zf) + inv(t.alpha) - inv((rb / r).min(1.0).acos());
    toothed_outline(z, rp - t.hf, rp + t.ha, rb, &psi, flanks).map_err(|e| format!("{z} teeth of module {:.3}: {e}", t.mt))
}

/// The transverse section of a right-hand Archimedean worm at z = 0: per start, flanks that are
/// the straight 20° flanks of its axial section turned along the thread (the tooth's angle at
/// radius r is 2π·t(r)/lead, t its axial thickness there).
fn worm_outline(m: f64, d1: f64, starts: u32) -> std::result::Result<Vec<Seg>, String> {
    let rp = d1 / 2.0;
    let lead = std::f64::consts::PI * m * f64::from(starts);
    let psi = |r: f64| std::f64::consts::PI * (std::f64::consts::PI * m / 2.0 - 2.0 * (r - rp) * 20f64.to_radians().tan()) / lead;
    toothed_outline(starts, rp - 1.25 * m, rp + m, 0.0, &psi, Flanks::Arcs).map_err(|e| format!("a {starts}-start worm of module {m}: {e}"))
}

/// How a tooth's flank is drawn: a fit spline through its points, or two circular arcs through
/// them (for a twisted sweep, which walls each piece of a spline span by span).
#[derive(Clone, Copy, Debug, PartialEq)]
enum Flanks {
    Spline,
    Arcs,
}

/// The arc from `a` through `m` to `b` (a line when they are in line).
fn arc3(a: [f64; 2], m: [f64; 2], b: [f64; 2]) -> Seg {
    let d = 2.0 * (a[0] * (m[1] - b[1]) + m[0] * (b[1] - a[1]) + b[0] * (a[1] - m[1]));
    if d.abs() < 1e-12 {
        return Seg::Line(a, b);
    }
    let sq = |p: [f64; 2]| p[0] * p[0] + p[1] * p[1];
    let c = [
        (sq(a) * (m[1] - b[1]) + sq(m) * (b[1] - a[1]) + sq(b) * (a[1] - m[1])) / d,
        (sq(a) * (b[0] - m[0]) + sq(m) * (a[0] - b[0]) + sq(b) * (m[0] - a[0])) / d,
    ];
    // Counter-clockwise from a to b when m lies to the right of a → b.
    if (m[0] - a[0]) * (b[1] - a[1]) - (m[1] - a[1]) * (b[0] - a[0]) > 0.0 { Seg::Arc { c, a, b } } else { Seg::Arc { c, a: b, b: a } }
}

/// A flank through `pts` (an odd number, at least 5) as `Flanks` says.
fn flank(pts: &[[f64; 2]], how: Flanks) -> Vec<Seg> {
    let n = pts.len() - 1;
    match (how, pts.first(), pts.get(n / 4), pts.get(n / 2), pts.get(3 * n / 4), pts.last()) {
        (Flanks::Arcs, Some(a), Some(q1), Some(m), Some(q3), Some(b)) => vec![arc3(*a, *q1, *m), arc3(*m, *q3, *b)],
        _ => vec![Seg::Spline(pts.to_vec())],
    }
}

/// A closed outline of `z` teeth from root radius `rf` to tip radius `ra`, tooth k centred at
/// angle 2πk/z with half-angle `psi(r)` at radius r from `max(r_flank, rf)` out (a radial line
/// below that).
fn toothed_outline(z: u32, rf: f64, ra: f64, r_flank: f64, psi: &dyn Fn(f64) -> f64, flanks: Flanks) -> std::result::Result<Vec<Seg>, String> {
    let zf = f64::from(z);
    if psi(ra) <= 0.0 || rf <= 0.0 || ra <= rf {
        return Err("pointed or rootless teeth".into());
    }
    let r0 = r_flank.max(rf);
    if 2.0 * psi(r0) >= 2.0 * std::f64::consts::PI / zf {
        return Err("the teeth meet at their roots".into());
    }
    const N: usize = 8;
    let rs: Vec<f64> = (0..=N).map(|i| r0 + (ra - r0) * i as f64 / N as f64).collect();
    let step = 2.0 * std::f64::consts::PI / zf;
    // Per tooth: the rising flank (root side first), the falling flank (tip first), and the
    // points where the outline leaves and reaches the root circle.
    struct Tooth {
        right: Vec<[f64; 2]>,
        left: Vec<[f64; 2]>,
        root_in: [f64; 2],
        root_out: [f64; 2],
    }
    let teeth: Vec<Tooth> = (0..z)
        .map(|k| {
            let t = step * f64::from(k);
            let right: Vec<[f64; 2]> = rs.iter().map(|r| polar(*r, t - psi(*r))).collect();
            let left: Vec<[f64; 2]> = rs.iter().rev().map(|r| polar(*r, t + psi(*r))).collect();
            let (root_in, root_out) = if rf < r0 { (polar(rf, t - psi(r0)), polar(rf, t + psi(r0))) } else { (right[0], left[N]) };
            Tooth { right, left, root_in, root_out }
        })
        .collect();
    let mut out = Vec::new();
    for (k, t) in teeth.iter().enumerate() {
        if rf < r0 {
            out.push(Seg::Line(t.root_in, t.right[0]));
        }
        out.extend(flank(&t.right, flanks));
        out.push(Seg::Arc { c: [0.0, 0.0], a: t.right[N], b: t.left[0] });
        out.extend(flank(&t.left, flanks));
        if rf < r0 {
            out.push(Seg::Line(t.left[N], t.root_out));
        }
        let next = &teeth[(k + 1) % teeth.len()];
        out.push(Seg::Arc { c: [0.0, 0.0], a: t.root_out, b: next.root_in });
    }
    Ok(out)
}

/// The outline of a timing pulley: the outside circle with `z` circular grooves.
fn pulley_outline(p: f64, pld: f64, depth: f64, rg: f64, z: u32) -> std::result::Result<Vec<Seg>, String> {
    let zf = f64::from(z);
    let ro = zf * p / std::f64::consts::PI / 2.0 - pld;
    let rc = ro - depth + rg;
    let cos_b = (ro * ro + rc * rc - rg * rg) / (2.0 * ro * rc);
    if !(cos_b > -1.0 && cos_b < 1.0) {
        return Err("the groove does not cut the outside diameter".into());
    }
    let beta = cos_b.acos();
    let step = 2.0 * std::f64::consts::PI / zf;
    if 2.0 * beta >= step {
        return Err(format!("{z} teeth are too few for this belt's grooves"));
    }
    let tau = 2.0 * std::f64::consts::PI;
    let edges: Vec<([f64; 2], [f64; 2])> = (0..z).map(|k| (polar(ro, step * f64::from(k) - beta), polar(ro, step * f64::from(k) + beta))).collect();
    let mut out = Vec::new();
    for k in 0..z as usize {
        let t = step * k as f64;
        let c = polar(rc, t);
        let (a, b) = edges[k];
        // The groove is the part of its circle inside the outside diameter: the way from a to b
        // that passes the side facing the pulley's centre.
        let ang = |q: [f64; 2]| (q[1] - c[1]).atan2(q[0] - c[0]);
        let (fa, fb, inner) = (ang(a), ang(b), t + std::f64::consts::PI);
        if (inner - fa).rem_euclid(tau) < (fb - fa).rem_euclid(tau) {
            out.push(Seg::Arc { c, a, b });
        } else {
            out.push(Seg::Arc { c, a: b, b: a });
        }
        out.push(Seg::Arc { c: [0.0, 0.0], a: b, b: edges[(k + 1) % edges.len()].0 });
    }
    Ok(out)
}

/// The edge of a cycloidal disc: the pins' path seen from the disc (`trochoid`) offset inwards
/// by the pin radius, one fit spline per lobe from valley to valley. Refused where the pins
/// would undercut the lobes or touch each other.
fn cycloid_outline(r: f64, n: u32, e: f64, rr: f64) -> std::result::Result<Vec<Seg>, String> {
    if !(e > 0.0 && e * f64::from(n + 1) < r && rr > 0.0) {
        return Err("the eccentricity makes cusps on the lobes".into());
    }
    if rr >= cycloid_rho_min(r, n, e) || rr >= r * (std::f64::consts::PI / f64::from(n + 1)).sin() {
        return Err(format!("pins of Ø{} undercut the lobes or touch each other", 2.0 * rr));
    }
    let edge = |u: f64| {
        let (q, d, _) = trochoid(r, n, e, u);
        let l = d[0].hypot(d[1]);
        // The left normal of the counter-clockwise path points into the disc.
        [q[0] - rr * d[1] / l, q[1] + rr * d[0] / l]
    };
    const PER_LOBE: u32 = 24;
    let lobe = 2.0 * std::f64::consts::PI / f64::from(n);
    Ok((0..n).map(|j| Seg::Spline((0..=PER_LOBE).map(|i| edge(lobe * (f64::from(j) + f64::from(i) / f64::from(PER_LOBE)))).collect())).collect())
}

/// The outline turned by `a` about the origin.
fn rotated(segs: &[Seg], a: f64) -> Vec<Seg> {
    let (s, c) = a.sin_cos();
    let t = |p: &[f64; 2]| [c * p[0] - s * p[1], s * p[0] + c * p[1]];
    segs.iter()
        .map(|g| match g {
            Seg::Line(a, b) => Seg::Line(t(a), t(b)),
            Seg::Spline(pts) => Seg::Spline(pts.iter().map(t).collect()),
            Seg::Arc { c, a, b } => Seg::Arc { c: t(c), a: t(a), b: t(b) },
        })
        .collect()
}

/// Draw a closed outline in a new XY sketch at height z0, with circles of the given diameters
/// about the origin (a circle inside the outline is a hole in it, a circle round it the outside
/// of a ring); the sketch's id. The curves go in as one edit of the sketch: drawn one command
/// at a time, each solves the whole sketch again, which grows with the square of the teeth.
fn outline_sketch(s: &mut Session, segs: &[Seg], circles: &[f64], z0: f64) -> Result<Value> {
    outline_sketch_on(s, "XY", segs, circles, z0)
}

/// `outline_sketch` on a principal plane (`XY`, `XZ`, `YZ`), offset along its normal.
fn outline_sketch_on(s: &mut Session, plane: &str, segs: &[Seg], circles: &[f64], offset: f64) -> Result<Value> {
    use solidcraft_geom::Vec2;
    use solidcraft_sketch::CurveKind;
    let sketch = run(s, "sketch.create", json!({"plane": plane, "offset": format!("{offset} mm")}))?["sketch"].clone();
    super::sketch::edit(s, &json!({}), "parts.insert", false, |sk, _| {
        // Curves that meet share their end point (the outline's pieces end exactly where the
        // next begins); centres are points of their own, or the outline would reach the circles
        // through them and the circles would stop being holes.
        let mut ends: std::collections::HashMap<(u64, u64), usize> = std::collections::HashMap::new();
        let mut end = |sk: &mut solidcraft_sketch::Sketch, p: [f64; 2]| -> Result<usize> {
            if let Some(i) = ends.get(&(p[0].to_bits(), p[1].to_bits())) {
                return Ok(*i);
            }
            let i = sk.add_point(Vec2::new(p[0], p[1]), None)?;
            ends.insert((p[0].to_bits(), p[1].to_bits()), i);
            Ok(i)
        };
        for g in segs {
            let kind = match g {
                Seg::Line(a, b) => CurveKind::Line { a: end(sk, *a)?, b: end(sk, *b)? },
                Seg::Arc { c, a, b } => CurveKind::Arc { c: sk.add_point(Vec2::new(c[0], c[1]), None)?, a: end(sk, *a)?, b: end(sk, *b)? },
                Seg::Spline(pts) => {
                    let mut idx = Vec::with_capacity(pts.len());
                    for (k, q) in pts.iter().enumerate() {
                        idx.push(if k == 0 || k + 1 == pts.len() { end(sk, *q)? } else { sk.add_point(Vec2::new(q[0], q[1]), None)? });
                    }
                    CurveKind::Spline { pts: idx, control: false, degree: 3, periodic: false }
                }
            };
            sk.add_curve(kind, None)?;
        }
        for d in circles {
            let c = sk.add_point(Vec2::new(0.0, 0.0), None)?;
            sk.add_curve(CurveKind::Circle { c, r: d / 2.0 }, None)?;
        }
        Ok(())
    })?;
    // Not run as a command: the model learns of the new curves (and profiles) here.
    s.refresh();
    fix_all(s)?;
    run(s, "sketch.finish", json!({}))?;
    Ok(sketch)
}

/// The profiles to make solid: with circles drawn, the one through the point at radius
/// `material` on the x axis (each circle's inside is a profile of its own as well as a hole);
/// without, every profile.
fn profiles(circles: &[f64], material: f64) -> Value {
    if circles.is_empty() { json!("all") } else { json!([{"point": [material, 0.0]}]) }
}

/// Draw a closed outline (and `circles`, as `outline_sketch`) in an XY sketch at height z0 and
/// extrude it up by `h` (`op`: new, join or cut); `material` as in `profiles`.
fn outline_solid(s: &mut Session, segs: &[Seg], circles: &[f64], material: f64, z0: f64, h: f64, op: &str) -> Result<()> {
    outline_sketch(s, segs, circles, z0)?;
    run(
        s,
        "solid.extrude",
        json!({"distance": format!("{h} mm"), "direction": "positive", "operation": op, "profiles": profiles(circles, material)}),
    )?;
    Ok(())
}

/// Draw a closed outline (and `circles`, as `outline_sketch`) at height z0 and sweep it straight
/// up by `h`, turning `twist` radians about z on the way (positive: counter-clockwise seen from
/// above, a right-hand helix); `material` as in `profiles`. The holes are drawn rather than cut
/// afterwards: a boolean against the twisted faces costs many times the sweep.
#[allow(clippy::too_many_arguments)]
fn twisted_solid(s: &mut Session, segs: &[Seg], circles: &[f64], material: f64, z0: f64, h: f64, twist: f64, op: &str) -> Result<()> {
    let sketch = outline_sketch(s, segs, circles, z0)?;
    let path = run(s, "sketch.create", json!({"plane": "XZ"}))?["sketch"].clone();
    // The XZ sketch's y runs down world z.
    run(s, "sketch.line", json!({"points": [[0.0, -z0], [0.0, -(z0 + h)]], "ids": ["axis"]}))?;
    fix_all(s)?;
    run(s, "sketch.finish", json!({}))?;
    run(
        s,
        "solid.sweep",
        json!({"sketch": sketch, "profiles": profiles(circles, material), "path_sketch": path, "path": ["axis"],
               "twist": format!("{} deg", twist.to_degrees()), "operation": op}),
    )?;
    Ok(())
}

/// Build a part in the active component.
fn build(s: &mut Session, sh: &Shape) -> Result<()> {
    let _frame = crate::frames::local_geometry();
    match *sh {
        Shape::Screw { d, pitch, len, head, drive, modeled } => {
            // The head's top (where the drive is cut) and the shank's top.
            let (top, shank_top) = match head {
                // One revolved outline: shank and round head.
                Head::Round { dk, k } => {
                    revolve(s, &[(0.0, -len), (d / 2.0, -len), (d / 2.0, 0.0), (dk / 2.0, 0.0), (dk / 2.0, k), (0.0, k)], "new")?;
                    (k, 0.0)
                }
                // A hex head joined over a shank that runs up into it.
                Head::Hex { s: af, k } => {
                    hex(s, af, 0.0, k, "new")?;
                    revolve(s, &[(0.0, -len), (d / 2.0, -len), (d / 2.0, k / 2.0), (0.0, k / 2.0)], "join")?;
                    (k, 0.0)
                }
                Head::Dome { dk, k, edge } => {
                    revolve_outline(s, &domed(d, len, dk, k, 0.0, 0.0, edge), "new")?;
                    (k, 0.0)
                }
                Head::Collar { dk, k, dc, c } => {
                    revolve_outline(s, &domed(d, len, dk, k, dc, c, c), "new")?;
                    (k, 0.0)
                }
                Head::Countersunk { dk, k } => {
                    revolve(s, &[(0.0, -len), (d / 2.0, -len), (d / 2.0, -k), (dk / 2.0, 0.0), (0.0, 0.0)], "new")?;
                    (0.0, -k)
                }
            };
            if let Some(drv) = drive {
                cut_drive(s, drv, top)?;
            }
            thread(s, d, pitch, Vec3::new(d / 2.0, 0.0, (shank_top - len) / 2.0), modeled);
        }
        Shape::Nut { d, pitch, s: af, m, modeled } => {
            hex(s, af, 0.0, m, "new")?;
            // The tapping drill (d − P); the thread is cut or marked in it.
            cyl(s, d - pitch, -1.0, m + 1.0, "cut")?;
            thread(s, d, pitch, Vec3::new((d - pitch) / 2.0, 0.0, m / 2.0), modeled);
        }
        Shape::Ring { d_in, d_out, h } => {
            revolve(s, &[(d_in / 2.0, 0.0), (d_out / 2.0, 0.0), (d_out / 2.0, h), (d_in / 2.0, h)], "new")?;
        }
        Shape::Bearing { d, od, b } => {
            // Each ring a third of the section, the seals between them a little in from the faces.
            let t = (od - d) / 2.0;
            let (ri, ro, s_in) = (d / 2.0 + t / 3.0, od / 2.0 - t / 3.0, (0.08 * b).min(0.4));
            let pts = [
                (d / 2.0, 0.0),
                (ri, 0.0),
                (ri, s_in),
                (ro, s_in),
                (ro, 0.0),
                (od / 2.0, 0.0),
                (od / 2.0, b),
                (ro, b),
                (ro, b - s_in),
                (ri, b - s_in),
                (ri, b),
                (d / 2.0, b),
            ];
            revolve(s, &pts, "new")?;
        }
        Shape::Rod { d, pitch, len, socket, point, modeled } => {
            let r = d / 2.0;
            // The end, from the axis out to the shank (where the thread begins).
            let (end, shank_end): (Vec<(f64, f64)>, f64) = match point {
                Point::Flat => (vec![(0.0, -len), (r, -len)], -len),
                Point::Cone { dt } => (vec![(0.0, -len), (dt / 2.0, -len), (r, -len + r - dt / 2.0)], -len + r - dt / 2.0),
                Point::Dog { dp, z } => (vec![(0.0, -len), (dp / 2.0, -len), (dp / 2.0, -len + z), (r, -len + z)], -len + z),
                // A conical cup a quarter of its diameter deep, its rim chamfered to the shank.
                Point::Cup { dz } => (vec![(0.0, -len + dz / 4.0), (dz / 2.0, -len), (r, -len + r - dz / 2.0)], -len + r - dz / 2.0),
            };
            let mut pts = end;
            pts.extend([(r, 0.0), (0.0, 0.0)]);
            revolve(s, &pts, "new")?;
            if let Some((af, t)) = socket {
                hex(s, af, 0.0, -t, "cut")?;
            }
            if let Some(p) = pitch {
                thread(s, d, p, Vec3::new(r, 0.0, shank_end / 2.0), modeled);
            }
        }
        Shape::Gear { tooth, z, w, bore, helix, herringbone } => {
            // Swept flanks as arcs (see `Flanks`); a spur gear keeps the fit splines.
            let segs = gear_outline(tooth, z, if helix == 0.0 { Flanks::Spline } else { Flanks::Arcs }).map_err(EngineError::Other)?;
            // Between the bore and the tooth roots.
            let mat = (bore / 2.0 + tooth.root_radius(z)) / 2.0;
            // The teeth turn tan(helix)/r_pitch radians per mm up the axis.
            let rate = helix.tan() / tooth.pitch_radius(z);
            if helix == 0.0 {
                outline_solid(s, &segs, &[bore], mat, 0.0, w, "new")?;
            } else if herringbone {
                // Two bodies meeting at mid-width: joining them costs far more than both sweeps.
                let half = w / 2.0;
                twisted_solid(s, &segs, &[bore], mat, 0.0, half, rate * half, "new")?;
                twisted_solid(s, &rotated(&segs, rate * half), &[bore], mat, half, half, -rate * half, "new")?;
            } else {
                twisted_solid(s, &segs, &[bore], mat, 0.0, w, rate * w, "new")?;
            }
        }
        Shape::RingGear { space, z, w, od } => {
            let segs = gear_outline(space, z, Flanks::Spline).map_err(EngineError::Other)?;
            // Between the tooth spaces' roots and the outside.
            outline_solid(s, &segs, &[od], (space.tip_radius(z) + od / 2.0) / 2.0, 0.0, w, "new")?;
        }
        Shape::Worm { m, d1, starts, len, bore } => {
            let segs = worm_outline(m, d1, starts).map_err(EngineError::Other)?;
            let mat = (bore / 2.0 + d1 / 2.0 - 1.25 * m) / 2.0;
            // The thread turns once per lead; a long worm is swept in pieces, one body each.
            let total = 2.0 * std::f64::consts::PI * len / (std::f64::consts::PI * m * f64::from(starts));
            let pieces = (total / (2.0 * std::f64::consts::PI * TURNS_PER_SWEEP) - 1e-9).ceil().max(1.0);
            let (h, turn) = (len / pieces, total / pieces);
            let mut i = 0.0;
            while i < pieces {
                twisted_solid(s, &rotated(&segs, turn * i), &[bore], mat, h * i, h, turn, "new")?;
                i += 1.0;
            }
        }
        Shape::Cycloid { r, n, e, rr, w, bore } => {
            let segs = cycloid_outline(r, n, e, rr).map_err(EngineError::Other)?;
            outline_solid(s, &segs, &[bore], (bore / 2.0 + r - e - rr) / 2.0, 0.0, w, "new")?;
        }
        Shape::Pulley { p, pld, depth, rg, z, w, flange, bore } => {
            let segs = pulley_outline(p, pld, depth, rg, z).map_err(EngineError::Other)?;
            // Teeth a little wider than the belt, flanges just outside them at the toothed part's ends.
            let t = w + BELT_CLEARANCE;
            outline_solid(s, &segs, &[], 0.0, flange, t, "new")?;
            let ro = f64::from(z) * p / std::f64::consts::PI / 2.0 - pld;
            let df = 2.0 * (ro + 1.5 * depth + 0.5);
            cyl(s, df, 0.0, flange, "join")?;
            cyl(s, df, flange + t, 2.0 * flange + t, "join")?;
            cyl(s, bore, -1.0, 2.0 * flange + t + 1.0, "cut")?;
        }
    }
    Ok(())
}

fn library(_s: &mut Session, p: &Value) -> Result<Value> {
    let fam = |id: &str| -> Value {
        let sizes: Vec<Value> = match id {
            "socket_head_cap_screw" => SHCS.iter().map(|r| json!({"size": r.0, "d": metric(r.0).map(|m| m.0), "pitch": metric(r.0).map(|m| m.1), "head_diameter": r.1, "head_height": r.2, "socket": r.3, "socket_depth": r.4})).collect(),
            "hex_bolt" => HEX_BOLT.iter().map(|r| json!({"size": r.0, "d": metric(r.0).map(|m| m.0), "pitch": metric(r.0).map(|m| m.1), "across_flats": r.1, "head_height": r.2})).collect(),
            "hex_nut" => HEX_NUT.iter().map(|r| json!({"size": r.0, "d": metric(r.0).map(|m| m.0), "pitch": metric(r.0).map(|m| m.1), "across_flats": r.1, "height": r.2})).collect(),
            "washer" => WASHER.iter().map(|r| json!({"size": r.0, "inner": r.1, "outer": r.2, "thickness": r.3})).collect(),
            f if f.starts_with("set_screw") => SET_SCREW
                .iter()
                .filter(|r| f == "set_screw" || SET_POINTS.iter().any(|q| q.0 == r.0))
                .map(|r| json!({"size": r.0, "d": metric(r.0).map(|m| m.0), "pitch": metric(r.0).map(|m| m.1), "socket": r.1, "socket_depth": r.2}))
                .collect(),
            f if !screw_drives(f).is_empty() => screw_sizes(f),
            "dowel_pin" => DOWEL.iter().map(|d| json!({"size": format!("{d}"), "d": d})).collect(),
            "bearing" => BEARINGS.iter().map(|r| json!({"size": r.0, "series": r.0.get(..2).unwrap_or(""), "bore": r.1, "outer": r.2, "width": r.3})).collect(),
            f if gear_family(f).is_some() || f.starts_with("cycloidal_disc_") => drive_sizes(f),
            f if f.starts_with("pulley_") => match PULLEYS.iter().find(|r| r.0 == f) {
                Some(&(_, p, pld, depth, ..)) => PULLEY_TEETH
                    .iter()
                    .map(|z| {
                        let pd = f64::from(*z) * p / std::f64::consts::PI;
                        json!({"size": z.to_string(), "teeth": z, "pitch": p, "pitch_diameter": pd,
                               "outside_diameter": pd - 2.0 * pld, "bore": bore_for(pd / 2.0 - pld - depth)})
                    })
                    .collect(),
                None => Vec::new(),
            },
            _ => Vec::new(),
        };
        let (_, name, standard) = FAMILIES.iter().find(|f| f.0 == id).copied().unwrap_or(("", "", ""));
        let lengths = if let Some((kind, _)) = gear_family(id) {
            Some(if kind == GearKind::Worm { WORM_LENGTHS.to_vec() } else { GEAR_WIDTHS.to_vec() })
        } else if id.starts_with("cycloidal_disc_") {
            Some(DISC_WIDTHS.to_vec())
        } else if let Some(r) = PULLEYS.iter().find(|r| r.0 == id) {
            Some(r.6.to_vec())
        } else {
            (threaded(id) && id != "hex_nut" || id == "dowel_pin").then_some(LENGTHS.to_vec())
        };
        let group = if is_drive(id) { "gears_pulleys" } else { "fasteners" };
        let drives: Vec<Value> =
            screw_drives(id).iter().filter_map(|d| DRIVES.iter().find(|x| x.0 == *d)).map(|(d, n)| json!({"drive": d, "name": n})).collect();
        json!({"family": id, "name": name, "standard": standard, "group": group, "category": category(id), "sizes": sizes, "lengths": lengths,
               "options": options(id), "drives": drives})
    };
    match str_(p, "family") {
        Some(f) if FAMILIES.iter().any(|x| x.0 == f) => Ok(fam(f)),
        Some(f) => Err(bad("parts.library", format!("unknown part family `{f}`"))),
        None => {
            let categories: Vec<Value> = CATEGORIES
                .iter()
                .map(|(id, name, group, family, size, length)| {
                    let mut sample = json!({"family": family, "size": size});
                    if *length > 0.0 {
                        sample["length"] = json!(length);
                    }
                    json!({"category": id, "name": name, "group": group, "sample": sample})
                })
                .collect();
            Ok(json!({"families": FAMILIES.iter().map(|f| fam(f.0)).collect::<Vec<_>>(), "categories": categories}))
        }
    }
}

/// Rounded to a thousandth of a millimetre (or degree), for the library's tables.
fn r3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

/// The library rows of a gear or cycloidal disc family, each size's dimensions (mm; angles in
/// degrees; helical gears at the default helix angle, worm wheels for a one-start worm).
fn drive_sizes(family: &str) -> Vec<Value> {
    if let Some(&(_, d)) = CYCLOIDS.iter().find(|c| c.0 == family) {
        let r = d / 2.0;
        return CYCLOID_LOBES
            .iter()
            .filter_map(|n| {
                let (pin, e) = (cycloid_pin(r, *n)?, cycloid_e(r, *n));
                Some(json!({"size": n.to_string(), "lobes": n, "pins": n + 1, "ratio": n, "pin_circle_diameter": d, "pin_diameter": pin,
                            "eccentricity": r3(e), "outside_diameter": r3(2.0 * (r + e - pin / 2.0)), "bore": bore_for(r - e - pin / 2.0)}))
            })
            .collect();
    }
    let Some((kind, m)) = gear_family(family) else { return Vec::new() };
    let teeth: &[u32] = match kind {
        GearKind::Ring => &RING_TEETH,
        GearKind::WormWheel => &WHEEL_TEETH,
        _ => &GEAR_TEETH,
    };
    match kind {
        GearKind::Worm => (1..=4u32)
            .map(|n| {
                let d1 = worm_d1(m);
                json!({"size": n.to_string(), "starts": n, "module": m, "pitch_diameter": d1, "outside_diameter": d1 + 2.0 * m,
                       "root_diameter": d1 - 2.5 * m, "lead": r3(std::f64::consts::PI * m * f64::from(n)),
                       "lead_angle": r3(lead_angle(m, n).to_degrees()), "bore": bore_for(d1 / 2.0 - 1.25 * m)})
            })
            .collect(),
        GearKind::Ring => teeth
            .iter()
            .filter_map(|z| match gear_shape(kind, m, &z.to_string(), None, Extra::default()) {
                Ok(Shape::RingGear { space, z, od, .. }) => Some(json!({"size": z.to_string(), "teeth": z, "module": m,
                    "pitch_diameter": r3(2.0 * space.pitch_radius(z)), "inside_diameter": r3(2.0 * space.root_radius(z)),
                    "root_diameter": r3(2.0 * space.tip_radius(z)), "outside_diameter": od})),
                _ => None,
            })
            .collect(),
        _ => teeth
            .iter()
            .filter_map(|z| match gear_shape(kind, m, &z.to_string(), None, Extra::default()) {
                Ok(Shape::Gear { tooth, z, bore, helix, .. }) => {
                    let mut row = json!({"size": z.to_string(), "teeth": z, "module": m, "pitch_diameter": r3(2.0 * tooth.pitch_radius(z)),
                        "outside_diameter": r3(2.0 * tooth.tip_radius(z)), "root_diameter": r3(2.0 * tooth.root_radius(z)), "bore": bore});
                    if kind == GearKind::WormWheel {
                        row["center_distance"] = json!(r3(tooth.pitch_radius(z) + worm_d1(m) / 2.0));
                        row["lead_angle"] = json!(r3(helix.to_degrees()));
                    } else if helix != 0.0 {
                        row["helix_angle"] = json!(r3(helix.to_degrees()));
                    }
                    Some(row)
                }
                _ => None,
            })
            .collect(),
    }
}

/// The library rows of a screw family: each size's head (with the default drive's recess).
fn screw_sizes(family: &str) -> Vec<Value> {
    METRIC
        .iter()
        .filter_map(|(size, d, pitch)| {
            let Ok(Shape::Screw { head, drive, .. }) = screw_shape(family, size, Some(20.0), &Extra::default()) else { return None };
            let (dk, k) = match head {
                Head::Round { dk, k } | Head::Dome { dk, k, .. } | Head::Countersunk { dk, k } => (dk, k),
                Head::Collar { dc, k, .. } => (dc, k),
                Head::Hex { s, k } => (s, k),
            };
            let mut row = json!({"size": size, "d": d, "pitch": pitch, "head_diameter": dk, "head_height": k});
            match drive {
                Some(Drive::Hex { s, t }) => {
                    row["socket"] = json!(s);
                    row["socket_depth"] = json!(t);
                }
                Some(Drive::Slot { n, t, .. }) => {
                    row["slot"] = json!(n);
                    row["slot_depth"] = json!(t);
                }
                Some(Drive::Cross { m, .. }) => row["recess"] = json!(m),
                Some(Drive::Lobe { a, .. }) => row["recess"] = json!(a),
                None => {}
            }
            Some(row)
        })
        .collect()
}

/// A screw's drive as its id, from its shape.
fn drive_id(sh: &Shape) -> Option<&'static str> {
    match sh {
        Shape::Screw { drive: Some(Drive::Hex { .. }), .. } => Some("hex"),
        Shape::Screw { drive: Some(Drive::Lobe { .. }), .. } => Some("hexalobular"),
        Shape::Screw { drive: Some(Drive::Cross { .. }), .. } => Some("cross"),
        Shape::Screw { drive: Some(Drive::Slot { .. }), .. } => Some("slot"),
        _ => None,
    }
}

fn part_name(family: &str, size: &str, sh: &Shape) -> String {
    let m = gear_family(family).map_or(0.0, |g| g.1);
    match sh {
        Shape::Gear { z, w, helix, herringbone, .. } if !herringbone && *helix != 0.0 && family.starts_with("helical_gear_") => {
            format!("{} m{m} z{z}x{w} {}{:.0}°", label(family), if *helix < 0.0 { "LH " } else { "" }, helix.abs().to_degrees())
        }
        Shape::Gear { z, w, .. } | Shape::RingGear { z, w, .. } => format!("{} m{m} z{z}x{w}", label(family)),
        Shape::Worm { starts, len, .. } => format!("{} m{m} {starts}-start x{len}", label(family)),
        Shape::Cycloid { r, n, w, .. } => format!("{} Ø{} N{n}x{w}", label(family), 2.0 * r),
        Shape::Pulley { z, w, .. } => format!("{} {z}T x{w}", label(family)),
        Shape::Screw { len, .. } if screw_drives(family).len() > 1 => {
            let suffix = match drive_id(sh) {
                Some("hexalobular") => " TX",
                Some("cross") => " PH",
                Some("slot") => " Slotted",
                _ => "",
            };
            format!("{} {size}x{len}{suffix}", label(family))
        }
        Shape::Screw { len, .. } | Shape::Rod { len, .. } => format!("{} {size}x{len}", label(family)),
        _ => format!("{} {size}", label(family)),
    }
}

/// A standard part's part number and description: its standard and size (`ISO 4762 M6x20`), or
/// a bearing's designation, and the family's name with the size.
fn identity(family: &str, size: &str, sh: &Shape) -> (String, String) {
    let m = gear_family(family).map_or(0.0, |g| g.1);
    let size = match sh {
        Shape::Screw { len, .. } | Shape::Rod { len, .. } => format!("{size}x{len}"),
        Shape::Gear { z, w, helix, .. } if *helix != 0.0 => format!("m{m} z{z} b{w} β{:.1}°", helix.to_degrees()),
        Shape::Gear { z, w, .. } | Shape::RingGear { z, w, .. } => format!("m{m} z{z} b{w}"),
        Shape::Worm { starts, len, .. } => format!("m{m} z{starts} L{len}"),
        Shape::Cycloid { r, n, w, rr, .. } => format!("D{} N{n} d{} b{w}", 2.0 * r, 2.0 * rr),
        Shape::Pulley { z, w, .. } => format!("{z}T {w}mm"),
        _ => size.to_string(),
    };
    let standard = match (family, drive_id(sh)) {
        ("socket_head_cap_screw", Some("hexalobular")) => "ISO 14579",
        ("socket_head_cap_screw", _) => "ISO 4762",
        ("low_head_cap_screw", _) => "DIN 7984",
        ("button_head_screw", _) => "ISO 7380-1",
        ("flanged_button_head_screw", _) => "ISO 7380-2",
        ("countersunk_socket_screw", _) => "ISO 10642",
        ("countersunk_screw", Some("slot")) => "ISO 2009",
        ("countersunk_screw", Some("hexalobular")) => "ISO 14581",
        ("countersunk_screw", _) => "ISO 7046-1",
        ("pan_head_screw", Some("hexalobular")) => "ISO 14583",
        ("pan_head_screw", _) => "ISO 7045",
        ("pan_head_slotted_screw", _) => "ISO 1580",
        ("cheese_head_screw", Some("hexalobular")) => "ISO 14580",
        ("cheese_head_screw", _) => "ISO 1207",
        ("set_screw_cone", _) => "ISO 4027",
        ("set_screw_dog", _) => "ISO 4028",
        ("set_screw_cup", _) => "ISO 4029",
        (family, _) => match family {
            "hex_bolt" => "ISO 4017",
            "hex_nut" => "ISO 4032",
            "washer" => "ISO 7089",
            "set_screw" => "ISO 4026",
            "dowel_pin" => "ISO 8734",
            f if gear_family(f).is_some_and(|g| g.0 != GearKind::Worm) => "ISO 53",
            "pulley_gt2" => "GT2",
            "pulley_htd3m" => "HTD 3M",
            "pulley_htd5m" => "HTD 5M",
            _ => "",
        },
    };
    let name = FAMILIES.iter().find(|f| f.0 == family).map_or("Part", |f| f.1);
    let part_number = if standard.is_empty() { size.clone() } else { format!("{standard} {size}") };
    (part_number, format!("{name} {size}"))
}

/// Give a standard part its part number and description; a field someone changed since the last
/// size (`was`) is theirs and stays.
fn set_identity(s: &mut Session, comp: u64, now: (String, String), was: Option<(String, String)>) {
    use solidcraft_doc::metadata::{DESCRIPTION, PART_NUMBER};
    let Some(c) = s.doc.components.iter().find(|c| c.id == comp) else { return };
    let mut m = c.metadata.clone();
    for (k, new, old) in [(PART_NUMBER, now.0, was.as_ref().map(|w| w.0.as_str())), (DESCRIPTION, now.1, was.as_ref().map(|w| w.1.as_str()))] {
        if m.text(k).is_none_or(|cur| Some(cur) == old) {
            let _ = m.set(k, Value::String(new));
        }
    }
    if m != c.metadata
        && let Some(c) = s.doc_mut().components.iter_mut().find(|c| c.id == comp)
    {
        c.metadata = m;
    }
}

fn label(family: &str) -> &'static str {
    match family {
        "socket_head_cap_screw" => "SHCS",
        "low_head_cap_screw" => "Low Head SHCS",
        "button_head_screw" => "Button Head",
        "flanged_button_head_screw" => "Flanged Button Head",
        "countersunk_socket_screw" => "Countersunk SHCS",
        "countersunk_screw" => "Countersunk Screw",
        "pan_head_screw" | "pan_head_slotted_screw" => "Pan Head",
        "cheese_head_screw" => "Cheese Head",
        "set_screw_cone" => "Set Screw Cone",
        "set_screw_dog" => "Set Screw Dog",
        "set_screw_cup" => "Set Screw Cup",
        "hex_bolt" => "Hex Bolt",
        "hex_nut" => "Hex Nut",
        "washer" => "Washer",
        "set_screw" => "Set Screw",
        "dowel_pin" => "Dowel Pin",
        f if f.starts_with("spur_gear_") => "Spur Gear",
        f if f.starts_with("helical_gear_") => "Helical Gear",
        f if f.starts_with("herringbone_gear_") => "Herringbone Gear",
        f if f.starts_with("ring_gear_") => "Ring Gear",
        f if f.starts_with("worm_wheel_") => "Worm Wheel",
        f if f.starts_with("worm_") => "Worm",
        f if f.starts_with("cycloidal_disc_") => "Cycloidal Disc",
        "pulley_gt2" => "GT2 Pulley",
        "pulley_htd3m" => "HTD3M Pulley",
        "pulley_htd5m" => "HTD5M Pulley",
        _ => "Bearing",
    }
}

fn length_param(p: &Value, cmd: &str) -> Result<Option<f64>> {
    match p.get("length") {
        None | Some(Value::Null) => Ok(None),
        Some(_) => {
            let l = num(p, "length").filter(|l| l.is_finite() && *l > 0.0 && *l <= 2000.0).ok_or_else(|| bad(cmd, "`length` must be 0…2000 mm"))?;
            Ok(Some(l))
        }
    }
}

/// What a saved part was made with besides size, length and bore.
fn saved_extra(part: &solidcraft_doc::StandardPart) -> Extra {
    Extra { helix: part.helix, starts: part.starts, drive: part.drive.clone(), modeled: part.modeled_thread }
}

fn insert(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "parts.insert (Insert Fastener)";
    let family = str_(p, "family").ok_or_else(|| bad(cmd, "`family` is required (parts.library)"))?.to_string();
    let size = str_(p, "size").ok_or_else(|| bad(cmd, "`size` is required"))?.to_string();
    let bore = bore_param(p, cmd)?;
    let extra = extra_params(s, p, cmd)?;
    let sh = shape(&family, &size, length_param(p, cmd)?, extra.clone()).and_then(|sh| with_bore(sh, bore)).map_err(|e| bad(cmd, e))?;
    // A pick on a hole's rim also touches its end face: step onto the wall just inside the rim.
    let at = p.get("at").and_then(vec3).map(|a| hole_wall(s, a).unwrap_or(a));
    // The occurrence the hole belongs to, found before the part exists.
    let host = match at {
        Some(a) => Some(super::joints::occurrence_at(s, a).ok_or_else(|| bad(cmd, "no body at `at` (pick a hole's rim or wall)"))?),
        None => None,
    };
    let name = str_(p, "name").map(str::to_string).unwrap_or_else(|| part_name(&family, &size, &sh));
    let active = s.active_component;
    // Hardware is fitted to other parts: it starts free even as the design's first part (#2804).
    let c = run(s, "component.create", json!({"name": name, "grounded": false}))?;
    let (comp, occ) = (c["component"].as_u64().unwrap_or(0), c["occurrence"].as_u64().unwrap_or(0));
    build(s, &sh)?;
    let part = solidcraft_doc::StandardPart {
        family: family.clone(),
        size: size.clone(),
        length: shape_len(&sh),
        bore,
        helix: extra.helix,
        starts: extra.starts,
        drive: extra.drive.clone(),
        modeled_thread: extra.modeled,
    };
    s.doc_mut().parts.insert(comp, part);
    set_identity(s, comp, identity(&family, &size, &sh), None);
    run(s, "component.activate", json!({"component": active}))?;
    let mut out = json!({"component": comp, "occurrence": occ, "name": name});
    if let (Some(a), Some(h)) = (at, host) {
        // Seat it: its axis on the hole's, its seating plane on the hole's face, the shank going
        // into the hole (the joint mates the part's −z with the face's outward z).
        let j = run(
            s,
            "joint.create",
            json!({"type": "rigid", "a": {"occurrence": h, "circle": a}, "b": {"occurrence": occ, "point": [0, 0, 0], "z": [0, 0, -1]}, "name": format!("{name} seat")}),
        )?;
        out["joint"] = j["joint"].clone();
    } else if let Some(pt) = p.get("point").and_then(vec3) {
        // Placed at a point, its +z along `direction`.
        let dir = p.get("direction").and_then(vec3).and_then(|d| d.normalized()).unwrap_or(Vec3::Z);
        let m = solidcraft_doc::rigid(
            Vec3::ZERO,
            Vec3::ZERO,
            Vec3::Z.cross(dir).normalized().unwrap_or(Vec3::X),
            Vec3::Z.dot(dir).clamp(-1.0, 1.0).acos(),
        );
        let mut m = m;
        m[3] = [pt.x, pt.y, pt.z, 1.0];
        if let Some(o) = s.doc_mut().occurrences.iter_mut().find(|o| o.id == occ) {
            o.transform = m;
        }
    }
    Ok(out)
}

/// A point on the cylindrical wall of the hole at `a`, a little in from the picked rim.
fn hole_wall(s: &Session, a: Vec3) -> Option<Vec3> {
    let st = s.model.state();
    let dirs = [Vec3::Z * -1.0, Vec3::Z, Vec3::X, Vec3::X * -1.0, Vec3::Y, Vec3::Y * -1.0];
    for step in [0.2, 0.5, 1.0, 2.0] {
        for b in &st.bodies {
            for d in dirs {
                let q = a + d * step;
                if let Some(c) = solidcraft_doc::kernel::cylinder_face_at(&b.body, q) {
                    let v = q - c.axis_point;
                    let along = v.dot(c.axis);
                    let on_wall = ((v - c.axis * along).len() - c.radius).abs() < 1e-3 && along > c.start && along < c.end;
                    if on_wall {
                        return Some(q);
                    }
                }
            }
        }
    }
    None
}

fn shape_len(sh: &Shape) -> Option<f64> {
    match sh {
        Shape::Screw { len, .. } | Shape::Rod { len, .. } => Some(*len),
        Shape::Gear { w, .. } | Shape::RingGear { w, .. } | Shape::Cycloid { w, .. } | Shape::Pulley { w, .. } => Some(*w),
        Shape::Worm { len, .. } => Some(*len),
        _ => None,
    }
}

/// Building the replacement runs several feature commands. Keep their intermediate history
/// and any failed build on a scratch session; the outer command records one complete undo.
fn resize_atomic(s: &mut Session, p: &Value) -> Result<Value> {
    let mut computed = s.scratch();
    let result = resize(&mut computed, p)?;
    computed.undo.clear();
    computed.active_component = s.active_component;
    computed.active_occurrence_path = s.active_occurrence_path.clone();
    computed.active_sketch = s.active_sketch.filter(|id| computed.doc.feature(*id).is_some());
    s.accept_computed(computed);
    Ok(result)
}

fn resize(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "parts.resize";
    let key = match p.get("component") {
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::String(x)) => x.clone(),
        _ => return Err(bad(cmd, "`component` must be an id or a name")),
    };
    let comp = s.doc.find_component(&key).ok_or_else(|| bad(cmd, format!("no component `{key}`")))?;
    let part = s.doc.parts.get(&comp).cloned().ok_or_else(|| bad(cmd, "that component is not a standard part"))?;
    let size = str_(p, "size").map(str::to_string).unwrap_or(part.size.clone());
    let length = length_param(p, cmd)?.or(if size == part.size { part.length } else { None });
    let bore = bore_param(p, cmd)?.or(part.bore);
    let given = extra_params(s, p, cmd)?;
    let extra = Extra {
        helix: given.helix.or(part.helix),
        starts: given.starts.or(part.starts),
        drive: given.drive.or(part.drive.clone()),
        modeled: if p.get("thread").is_some() { given.modeled } else { part.modeled_thread },
    };
    let sh = shape(&part.family, &size, length, extra.clone()).and_then(|sh| with_bore(sh, bore)).map_err(|e| bad(cmd, e))?;
    // Rebuild: the component's features go, the new shape is built in their place.
    let ids: Vec<u64> = s.doc.features.iter().filter(|f| f.component == comp).map(|f| f.id).collect();
    for id in ids.iter().rev() {
        let _ = s.doc_mut().delete_feature(*id);
    }
    let active = s.active_component;
    run(s, "component.activate", json!({"component": comp}))?;
    build(s, &sh)?;
    let name = part_name(&part.family, &size, &sh);
    let was = shape(&part.family, &part.size, part.length, saved_extra(&part)).ok().map(|old| identity(&part.family, &part.size, &old));
    set_identity(s, comp, identity(&part.family, &size, &sh), was);
    let saved = solidcraft_doc::StandardPart {
        size: size.clone(),
        length: shape_len(&sh),
        bore,
        helix: extra.helix,
        starts: extra.starts,
        drive: extra.drive.clone(),
        modeled_thread: extra.modeled,
        ..part
    };
    s.doc_mut().parts.insert(comp, saved);
    if let Some(c) = s.doc_mut().components.iter_mut().find(|c| c.id == comp) {
        c.name = name.clone();
    }
    run(s, "component.activate", json!({"component": active}))?;
    s.refresh();
    if let Some(e) = s.model.results.iter().find_map(|r| r.error.clone()) {
        return Err(EngineError::Other(e));
    }
    Ok(json!({"component": comp, "name": name, "size": size, "length": shape_len(&sh)}))
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use crate::Session;

    fn run(s: &mut Session, id: &str, p: Value) -> Value {
        match s.execute(id, &p) {
            Ok(v) => v,
            Err(e) => panic!("{id} {p}: {e}"),
        }
    }

    fn bbox(s: &Session, prefix: &str) -> (solidcraft_geom::Vec3, solidcraft_geom::Vec3) {
        let st = s.world_state();
        let comp = s.doc.components.iter().find(|c| c.name.starts_with(prefix)).map(|c| c.id).unwrap_or(0);
        let b = st
            .bodies
            .iter()
            .filter(|b| s.doc.model_body_component(b) == comp)
            .fold(solidcraft_geom::Aabb3::EMPTY, |a, b| a.union(&b.mesh().bounds()));
        (b.min, b.max)
    }

    #[test]
    fn dimensions_follow_the_standard_tables() {
        let l = run(&mut Session::default(), "parts.library", json!({"family": "socket_head_cap_screw"}));
        let m6 = l["sizes"].as_array().and_then(|a| a.iter().find(|x| x["size"] == "M6")).cloned().unwrap_or_default();
        assert_eq!((m6["head_diameter"].as_f64(), m6["head_height"].as_f64(), m6["socket"].as_f64()), (Some(10.0), Some(6.0), Some(5.0)));
        // An M6 x 20 cap screw: head Ø10 x 6, shank 20 long.
        let mut s = Session::default();
        run(&mut s, "parts.insert", json!({"family": "socket_head_cap_screw", "size": "M6", "length": 20}));
        let (lo, hi) = bbox(&s, "SHCS M6x20");
        assert!((hi.x - lo.x - 10.0).abs() < 1e-6 && (lo.z + 20.0).abs() < 1e-6 && (hi.z - 6.0).abs() < 1e-6, "{lo:?} {hi:?}");
        assert!(s.model.state().threads.iter().any(|t| t.designation.starts_with("M6")), "{:?}", s.model.state().threads);
        // An M8 nut: 13 across flats (14.43 across corners), 6.8 high.
        let mut n = Session::default();
        run(&mut n, "parts.insert", json!({"family": "hex_nut", "size": "M8"}));
        let (lo, hi) = bbox(&n, "Hex Nut M8");
        let (w, h) = ((hi.x - lo.x).max(hi.y - lo.y), hi.z - lo.z);
        assert!((w - 13.0 / (std::f64::consts::PI / 6.0).cos()).abs() < 1e-6 && (h - 6.8).abs() < 1e-6, "{w} {h}");
        // A 608 bearing: 8 x 22 x 7.
        let mut b = Session::default();
        run(&mut b, "parts.insert", json!({"family": "bearing", "size": "608"}));
        let (lo, hi) = bbox(&b, "Bearing 608");
        assert!((hi.x - lo.x - 22.0).abs() < 1e-6 && (hi.z - lo.z - 7.0).abs() < 1e-6);
        assert!(b.execute("parts.insert", &json!({"family": "hex_nut", "size": "M7"})).is_err());
    }

    /// #2091: the second and larger ISO sizes, M2, M2.5, M14, M24 and M30, are in the screw, bolt,
    /// nut and washer tables (they stopped at M3 and M20).
    #[test]
    fn small_and_large_iso_sizes_insert() {
        for family in ["socket_head_cap_screw", "hex_bolt", "hex_nut", "washer"] {
            let l = run(&mut Session::default(), "parts.library", json!({"family": family}));
            let sizes: Vec<&str> = l["sizes"].as_array().unwrap().iter().filter_map(|x| x["size"].as_str()).collect();
            for size in ["M2", "M2.5", "M14", "M24", "M30"] {
                assert!(sizes.contains(&size), "{family} has {size}: {sizes:?}");
            }
        }
        // An M2.5 x 8 cap screw: head Ø4.5 x 2.5, shank 8 long.
        let mut s = Session::default();
        run(&mut s, "parts.insert", json!({"family": "socket_head_cap_screw", "size": "M2.5", "length": 8}));
        let (lo, hi) = bbox(&s, "SHCS M2.5x8");
        assert!((hi.x - lo.x - 4.5).abs() < 1e-6 && (lo.z + 8.0).abs() < 1e-6 && (hi.z - 2.5).abs() < 1e-6, "{lo:?} {hi:?}");
        // An M24 nut: 36 across flats, 21.5 high.
        let mut n = Session::default();
        run(&mut n, "parts.insert", json!({"family": "hex_nut", "size": "M24"}));
        let (lo, hi) = bbox(&n, "Hex Nut M24");
        let (w, h) = ((hi.x - lo.x).max(hi.y - lo.y), hi.z - lo.z);
        assert!((w - 36.0 / (std::f64::consts::PI / 6.0).cos()).abs() < 1e-6 && (h - 21.5).abs() < 1e-6, "{w} {h}");
        // An M30 washer: Ø56 x 4.
        let mut w = Session::default();
        run(&mut w, "parts.insert", json!({"family": "washer", "size": "M30"}));
        let (lo, hi) = bbox(&w, "Washer M30");
        assert!((hi.x - lo.x - 56.0).abs() < 1e-6 && (hi.z - lo.z - 4.0).abs() < 1e-6, "{lo:?} {hi:?}");
    }

    #[test]
    fn gears_and_pulleys_have_their_diameters_and_widths() {
        // m1 z20: tip diameter 22, 10 wide, with a bore.
        let mut s = Session::default();
        let g = run(&mut s, "parts.insert", json!({"family": "spur_gear_m1", "size": "20", "length": 10}));
        assert!(s.model.results.iter().all(|r| r.error.is_none()), "{g}");
        let (lo, hi) = bbox(&s, "Spur Gear m1 z20");
        assert!((hi.x - lo.x - 22.0).abs() < 0.05 && (hi.z - lo.z - 10.0).abs() < 1e-6, "{lo:?} {hi:?}");
        // Its bore goes through: the largest standard one within the root radius 8.75.
        assert!((inner(&s, "Spur Gear m1 z20") - 4.0).abs() < 1e-3, "{}", inner(&s, "Spur Gear m1 z20"));
        // GT2 20T for a 6 mm belt: 7 mm of teeth between two 1 mm flanges, 9 mm overall.
        let mut p = Session::default();
        run(&mut p, "parts.insert", json!({"family": "pulley_gt2", "size": "20", "length": 6}));
        assert!(p.model.results.iter().all(|r| r.error.is_none()));
        let (lo, hi) = bbox(&p, "GT2 Pulley 20T");
        assert!((hi.z - lo.z - 9.0).abs() < 1e-6, "{lo:?} {hi:?}");
        // The library gives the GT2 20T outside diameter 12.22.
        let l = run(&mut Session::default(), "parts.library", json!({"family": "pulley_gt2"}));
        let t20 = l["sizes"].as_array().and_then(|a| a.iter().find(|x| x["size"] == "20")).cloned().unwrap_or_default();
        assert!((t20["outside_diameter"].as_f64().unwrap_or(0.0) - 12.224).abs() < 0.01, "{t20}");
        // Resize: 30 teeth, 15 wide.
        let comp = g["component"].as_u64().unwrap_or(0);
        run(&mut s, "parts.resize", json!({"component": comp, "size": "30", "length": 15}));
        let (lo, hi) = bbox(&s, "Spur Gear m1 z30");
        assert!((hi.x - lo.x - 32.0).abs() < 0.05 && (hi.z - lo.z - 15.0).abs() < 1e-6, "{lo:?} {hi:?}");
        assert!(Session::default().execute("parts.insert", &json!({"family": "spur_gear_m1", "size": "3"})).is_err());
        // A chosen bore is kept through a resize; one larger than the root allows is refused.
        let mut b = Session::default();
        let g = run(&mut b, "parts.gear", json!({"family": "spur_gear_m1", "size": "20", "length": 10, "bore": 6}));
        let comp = g["component"].as_u64().unwrap_or(0);
        assert_eq!(b.doc.parts.get(&comp).and_then(|p| p.bore), Some(6.0));
        run(&mut b, "parts.resize", json!({"component": comp, "size": "24"}));
        assert_eq!(b.doc.parts.get(&comp).and_then(|p| p.bore), Some(6.0));
        assert!(Session::default().execute("parts.insert", &json!({"family": "spur_gear_m1", "size": "20", "bore": 18})).is_err());
        let l = run(&mut Session::default(), "parts.library", json!({}));
        let groups: Vec<&str> = l["families"].as_array().map_or(Vec::new(), |a| a.iter().filter_map(|f| f["group"].as_str()).collect());
        assert!(groups.contains(&"fasteners") && groups.contains(&"gears_pulleys"));
    }

    /// Every screw family builds in every size and drive it comes in: its head as wide as its
    /// standard's table says (the collar for a flanged head), as high (a countersunk head flush
    /// with z = 0, its length overall), with the drive cut in and a thread listed.
    #[test]
    fn every_screw_builds_in_every_size_and_drive() {
        let l = run(&mut Session::default(), "parts.library", json!({}));
        let families: Vec<Value> = l["families"].as_array().cloned().unwrap_or_default();
        let mut built = 0;
        for f in families.iter().filter(|f| f["drives"].as_array().is_some_and(|d| !d.is_empty())) {
            let id = f["family"].as_str().unwrap_or("");
            for row in f["sizes"].as_array().cloned().unwrap_or_default() {
                for drive in f["drives"].as_array().cloned().unwrap_or_default() {
                    let mut s = Session::default();
                    let r = s.execute("parts.insert", &json!({"family": id, "size": row["size"], "length": 20, "drive": drive["drive"]}));
                    let r = match r {
                        Ok(r) => r,
                        // A drive a size does not come in (no hexalobular M20 socket cap) is refused, not built.
                        Err(e) => {
                            assert!(
                                id == "socket_head_cap_screw" && row["size"] == "M20" && drive["drive"] == "hexalobular",
                                "{id} {} {}: {e}",
                                row["size"],
                                drive["drive"]
                            );
                            continue;
                        }
                    };
                    let errors: Vec<String> = s.model.results.iter().filter_map(|r| r.error.clone()).collect();
                    assert!(errors.is_empty(), "{id} {} {}: {errors:?}", row["size"], drive["drive"]);
                    let name = r["name"].as_str().unwrap_or("").to_string();
                    let (lo, hi) = bbox(&s, &name);
                    let (dk, mut k) = (row["head_diameter"].as_f64().unwrap_or(0.0), row["head_height"].as_f64().unwrap_or(0.0));
                    // The library lists the default drive's head; ISO 14580's hexalobular cheese head is taller.
                    if id == "cheese_head_screw" && drive["drive"] == "hexalobular" {
                        k = super::CHEESE.iter().find(|c| row["size"] == c.0).map_or(k, |c| c.3);
                    }
                    // A slot runs across the head along x: the width across it is along y (the mesh
                    // stands within a few hundredths of the circle).
                    let width = (hi.x - lo.x).max(hi.y - lo.y);
                    assert!((width - dk).abs() < 0.05, "{name}: width {width} vs {dk}");
                    // The shank ends 20 below the seat; the head's top is at k (a countersunk one flush at
                    // 0), a domed one a little lower where its recess takes the dome's crown.
                    let top = if id.starts_with("countersunk") { 0.0 } else { k };
                    assert!((lo.z + 20.0).abs() < 1e-3 && hi.z <= top + 1e-3 && hi.z >= top - 0.15 * k, "{name}: z {} … {} vs top {top}", lo.z, hi.z);
                    assert!(s.model.state().threads.iter().any(|t| (t.pitch - row["pitch"].as_f64().unwrap_or(0.0)).abs() < 1e-9), "{name} thread");
                    built += 1;
                }
            }
        }
        assert!(built > 80, "{built}");
        // A drive the family does not come in, or a thread on a washer: refused.
        let mut s = Session::default();
        assert!(s.execute("parts.insert", &json!({"family": "button_head_screw", "size": "M4", "drive": "slot"})).is_err());
        assert!(s.execute("parts.insert", &json!({"family": "pan_head_screw", "size": "M4", "drive": "slot"})).is_err());
        assert!(s.execute("parts.insert", &json!({"family": "washer", "size": "M4", "thread": "modeled"})).is_err());
    }

    /// Set screws end in their ISO points, a nut is drilled for its thread (d − P), and a
    /// modelled thread is cut into the part: one body, less material than the plain shank.
    #[test]
    fn set_screw_points_nut_drills_and_modelled_threads() {
        for (family, size) in [("set_screw_cone", "M6"), ("set_screw_dog", "M6"), ("set_screw_cup", "M6"), ("set_screw", "M6")] {
            let mut s = Session::default();
            let r = run(&mut s, "parts.insert", json!({"family": family, "size": size, "length": 10}));
            let (lo, hi) = bbox(&s, r["name"].as_str().unwrap_or(""));
            assert!((hi.z - lo.z - 10.0).abs() < 1e-6 && (hi.x - lo.x - 6.0).abs() < 0.02, "{family}: {lo:?} {hi:?}");
        }
        assert!(Session::default().execute("parts.insert", &json!({"family": "set_screw_dog", "size": "M6", "length": 3})).is_err());
        let mut n = Session::default();
        run(&mut n, "parts.insert", json!({"family": "hex_nut", "size": "M8"}));
        assert!((inner(&n, "Hex Nut M8") - 6.75 / 2.0).abs() < 1e-3, "{}", inner(&n, "Hex Nut M8"));
        let mut c = Session::default();
        run(&mut c, "parts.insert", json!({"family": "button_head_screw", "size": "M3", "length": 6}));
        let plain = measure(&c, "Button Head M3x6");
        let mut m = Session::default();
        run(&mut m, "parts.insert", json!({"family": "button_head_screw", "size": "M3", "length": 6, "thread": "modeled"}));
        assert!(m.model.results.iter().all(|r| r.error.is_none()));
        let cut = measure(&m, "Button Head M3x6");
        assert!(cut < plain - 1.0 && cut > 0.8 * plain, "{cut} vs {plain}");
        let comp = m.doc.components.iter().find(|c| c.name.starts_with("Button Head")).map_or(0, |c| c.id);
        assert!(m.doc.parts.get(&comp).is_some_and(|p| p.modeled_thread));
    }

    /// Every bearing of the table builds to its ISO 15 envelope (bore, outside diameter, width),
    /// and the library gives each its series.
    #[test]
    fn every_bearing_builds_to_its_envelope() {
        for (name, d, od, b) in super::BEARINGS {
            let mut s = Session::default();
            run(&mut s, "parts.insert", json!({"family": "bearing", "size": name}));
            let (lo, hi) = bbox(&s, &format!("Bearing {name}"));
            assert!((hi.x - lo.x - od).abs() < 1e-3 && (hi.z - lo.z - b).abs() < 1e-6, "{name}: {lo:?} {hi:?}");
            assert!((inner(&s, &format!("Bearing {name}")) - d / 2.0).abs() < 1e-3, "{name} bore");
        }
        let l = run(&mut Session::default(), "parts.library", json!({"family": "bearing"}));
        let r6305 = l["sizes"].as_array().and_then(|a| a.iter().find(|x| x["size"] == "6305")).cloned().unwrap_or_default();
        assert_eq!(r6305, json!({"size": "6305", "series": "63", "bore": 25.0, "outer": 62.0, "width": 17.0}));
    }

    /// Every family is in a category of its own group, and every category's sample part builds.
    #[test]
    fn categories_cover_the_library_and_their_samples_build() {
        let l = run(&mut Session::default(), "parts.library", json!({}));
        let cats = l["categories"].as_array().cloned().unwrap_or_default();
        let group_of = |c: &str| cats.iter().find(|x| x["category"] == c).and_then(|x| x["group"].as_str().map(str::to_string));
        for f in l["families"].as_array().cloned().unwrap_or_default() {
            let c = f["category"].as_str().unwrap_or("");
            assert_eq!(group_of(c).as_deref(), f["group"].as_str(), "{}", f["family"]);
        }
        for c in &cats {
            let mut s = Session::default();
            let mut p = c["sample"].clone();
            p["point"] = json!([0, 0, 0]);
            run(&mut s, "parts.insert", p);
            let errors: Vec<String> = s.model.results.iter().filter_map(|r| r.error.clone()).collect();
            assert!(errors.is_empty() && !s.world_state().bodies.is_empty(), "{}: {errors:?}", c["category"]);
        }
    }

    /// The mesh points of a part's bodies.
    fn points(s: &Session, prefix: &str) -> Vec<solidcraft_geom::Vec3> {
        let st = s.world_state();
        let comp = s.doc.components.iter().find(|c| c.name.starts_with(prefix)).map(|c| c.id).unwrap_or(0);
        st.bodies.iter().filter(|b| s.doc.model_body_component(b) == comp).flat_map(|b| b.mesh().positions.clone()).collect()
    }

    /// How far the tooth at height z sits from angle `expected` (degrees): the middle of the
    /// mesh points there beyond radius `r`, their angles taken within half a tooth pitch (`step`
    /// degrees) of `expected`.
    fn tip_offset(pts: &[solidcraft_geom::Vec3], z: f64, r: f64, step: f64, expected: f64) -> f64 {
        let (lo, hi) = pts
            .iter()
            .filter(|p| (p.z - z).abs() < 1e-6 && p.x.hypot(p.y) > r)
            .map(|p| (p.y.atan2(p.x).to_degrees() - expected + step / 2.0).rem_euclid(step) - step / 2.0)
            .fold((f64::MAX, f64::MIN), |(a, b), x| (a.min(x), b.max(x)));
        assert!(lo <= hi, "no tips at z {z}");
        (lo + hi) / 2.0
    }

    /// The smallest distance of a part's mesh points from its axis (its bore's radius).
    fn inner(s: &Session, prefix: &str) -> f64 {
        points(s, prefix).iter().map(|p| p.x.hypot(p.y)).fold(f64::MAX, f64::min)
    }

    fn measure(s: &Session, prefix: &str) -> f64 {
        let st = s.world_state();
        let comp = s.doc.components.iter().find(|c| c.name.starts_with(prefix)).map(|c| c.id).unwrap_or(0);
        st.bodies
            .iter()
            .filter(|b| s.doc.model_body_component(b) == comp)
            .filter_map(|b| solidcraft_kernel::measure(&b.body).ok())
            .map(|m| m.volume)
            .sum()
    }

    fn inserted(family: &str, p: Value) -> Session {
        let mut s = Session::default();
        let mut q = json!({"family": family});
        if let (Some(q), Some(p)) = (q.as_object_mut(), p.as_object()) {
            q.extend(p.clone());
        }
        run(&mut s, "parts.gear", q);
        let errors: Vec<String> = s.model.results.iter().filter_map(|r| r.error.clone()).collect();
        assert!(errors.is_empty(), "{family}: {errors:?}");
        s
    }

    /// Twisted parts draw each flank as two arcs (a fit spline is many pieces to a twisted sweep):
    /// within 5 µm per mm of module of the involute over its whole height (4.4 µm at m1 z10, the
    /// most curved).
    #[test]
    fn swept_flanks_are_arcs_close_to_the_involute() {
        for (m, z) in [(1.0, 10u32), (1.0, 30), (2.0, 100)] {
            let tooth = super::Involute::iso53(m, 20f64.to_radians());
            let rb = tooth.pitch_radius(z) * tooth.alpha.cos();
            let (r0, ra) = (rb.max(tooth.root_radius(z)), tooth.tip_radius(z));
            let psi = |r: f64| std::f64::consts::PI / (2.0 * f64::from(z)) + super::inv(tooth.alpha) - super::inv((rb / r).min(1.0).acos());
            let segs = super::gear_outline(tooth, z, super::Flanks::Arcs).unwrap_or_default();
            let arcs: Vec<([f64; 2], f64)> = segs
                .iter()
                .filter_map(|g| if let super::Seg::Arc { c, a, .. } = g { Some((*c, (a[0] - c[0]).hypot(a[1] - c[1]))) } else { None })
                .collect();
            let worst = (0..=40)
                .map(|i| {
                    let r = r0 + (ra - r0) * f64::from(i) / 40.0;
                    let p = super::polar(r, -psi(r));
                    arcs.iter().map(|(c, rad)| ((p[0] - c[0]).hypot(p[1] - c[1]) - rad).abs()).fold(f64::MAX, f64::min)
                })
                .fold(0.0, f64::max);
            assert!(worst < 5e-3 * m, "m{m} z{z}: {worst}");
        }
    }

    /// Helical gears are the spur gear's teeth turned along a helix: a right-hand one turns
    /// counter-clockwise going up by tan β / r_pitch per mm, a left-hand one the other way; a
    /// herringbone turns there and back, so its top lines up with its bottom.
    #[test]
    fn helical_and_herringbone_gears_turn_along_their_helix() {
        // m1 z10 β20°: transverse module 1/cos 20°, tips at 5.321 + 1; 4 mm turn 15.7°.
        let rp = 10.0 / 20f64.to_radians().cos() / 2.0;
        let twist = (4.0 * 20f64.to_radians().tan() / rp).to_degrees();
        let s = inserted("helical_gear_m1", json!({"size": "10", "length": 4}));
        let (lo, hi) = bbox(&s, "Helical Gear m1 z10");
        assert!((hi.x - lo.x - 2.0 * (rp + 1.0)).abs() < 0.05 && (hi.z - lo.z - 4.0).abs() < 1e-6, "{lo:?} {hi:?}");
        let pts = points(&s, "Helical Gear m1 z10");
        for (z, expected) in [(0.0, 0.0), (4.0, twist)] {
            let off = tip_offset(&pts, z, rp, 36.0, expected);
            assert!(off.abs() < 1.0, "at z {z}: {off} from {expected}");
        }
        let volume = measure(&s, "Helical Gear m1 z10");
        // The bore runs through the twisted teeth (root radius 4.07: a 4 mm bore).
        assert!((inner(&s, "Helical Gear m1 z10") - 2.0).abs() < 1e-3, "{}", inner(&s, "Helical Gear m1 z10"));
        // Left hand: the other way.
        let l = inserted("helical_gear_m1", json!({"size": "10", "length": 4, "helix_angle": "-20 deg"}));
        let pts = points(&l, "Helical Gear m1 z10x4 LH");
        let off = tip_offset(&pts, 4.0, rp, 36.0, -twist);
        assert!(off.abs() < 1.0, "{off}");
        // Herringbone, 8 wide: turned 15.7° at mid-height, back at the top; twice the volume.
        let h = inserted("herringbone_gear_m1", json!({"size": "10", "length": 8}));
        let pts = points(&h, "Herringbone Gear m1 z10");
        for (z, expected) in [(4.0, twist), (8.0, 0.0)] {
            let off = tip_offset(&pts, z, rp, 36.0, expected);
            assert!(off.abs() < 1.0, "at z {z}: {off} from {expected}");
        }
        let hv = measure(&h, "Herringbone Gear m1 z10");
        assert!((hv - 2.0 * volume).abs() < 0.01 * hv, "{hv} vs 2 x {volume}");
        // The helix angle is saved with the part and kept through a resize.
        let mut r = inserted("helical_gear_m1", json!({"size": "20", "length": 6, "helix_angle": 30}));
        let comp = r.doc.components.iter().find(|c| c.name.starts_with("Helical Gear")).map_or(0, |c| c.id);
        let saved = r.doc.parts.get(&comp).and_then(|p| p.helix).unwrap_or(0.0);
        assert!((saved - 30f64.to_radians()).abs() < 1e-9, "{saved}");
        run(&mut r, "parts.resize", json!({"component": comp, "size": "24"}));
        let (lo, hi) = bbox(&r, "Helical Gear m1 z24");
        let tip = 24.0 / 30f64.to_radians().cos() + 2.0;
        assert!((hi.x - lo.x - tip).abs() < 0.05, "{lo:?} {hi:?} vs {tip}");
        // Out of range, or on a gear that has none: refused.
        for bad in [
            json!({"family": "helical_gear_m1", "size": "20", "helix_angle": 60}),
            json!({"family": "spur_gear_m1", "size": "20", "helix_angle": 20}),
            json!({"family": "helical_gear_m1", "size": "20", "starts": 2}),
        ] {
            assert!(Session::default().execute("parts.gear", &bad).is_err(), "{bad}");
        }
    }

    /// A ring gear's teeth point in: tips at r_pitch − m, roots at r_pitch + 1.25 m, inside a
    /// rim of three modules; it takes no bore.
    #[test]
    fn ring_gears_have_internal_teeth() {
        let s = inserted("ring_gear_m1", json!({"size": "60", "length": 10}));
        let (lo, hi) = bbox(&s, "Ring Gear m1 z60");
        assert!((hi.x - lo.x - 69.0).abs() < 0.05 && (hi.z - lo.z - 10.0).abs() < 1e-6, "{lo:?} {hi:?}");
        let inner = points(&s, "Ring Gear m1 z60").iter().map(|p| p.x.hypot(p.y)).fold(f64::MAX, f64::min);
        assert!((inner - 29.0).abs() < 0.05, "{inner}");
        assert!(Session::default().execute("parts.gear", &json!({"family": "ring_gear_m1", "size": "60", "bore": 10})).is_err());
        assert!(Session::default().execute("parts.gear", &json!({"family": "ring_gear_m1", "size": "12"})).is_err());
    }

    /// A worm is a right-hand thread of straight 20° flanks: its tips at d1/2 + m, one turn per
    /// lead (π m · starts); its wheel is a helical gear at the worm's lead angle, module m.
    #[test]
    fn worms_and_worm_wheels() {
        let s = inserted("worm_m1", json!({"size": "1"}));
        // By default 12 long, the most one sweep of four turns (4π) makes.
        let (lo, hi) = bbox(&s, "Worm m1 1-start");
        assert!((hi.x - lo.x - 14.0).abs() < 0.05 && (hi.z - lo.z - 12.0).abs() < 1e-6, "{lo:?} {hi:?}");
        // Right hand: the thread's middle at height z sits at angle 360°·z/lead.
        let pts = points(&s, "Worm m1 1-start");
        let lead = std::f64::consts::PI;
        for z in [0.0, 12.0] {
            let expected = z / lead * 360.0;
            let off = tip_offset(&pts, z, 6.5, 360.0, expected);
            assert!(off.abs() < 2.0, "at z {z}: {off} from {expected}");
        }
        // A worm too long for its thread is refused.
        assert!(Session::default().execute("parts.gear", &json!({"family": "worm_m0_5", "size": "1", "length": 80})).is_err());
        let w = inserted("worm_wheel_m1", json!({"size": "30", "starts": 2}));
        let (lo, hi) = bbox(&w, "Worm Wheel m1 z30");
        assert!((hi.x - lo.x - 32.0).abs() < 0.05 && (hi.z - lo.z - 10.0).abs() < 1e-6, "{lo:?} {hi:?}");
        let l = run(&mut Session::default(), "parts.library", json!({"family": "worm_wheel_m1"}));
        let z30 = l["sizes"].as_array().and_then(|a| a.iter().find(|x| x["size"] == "30")).cloned().unwrap_or_default();
        assert_eq!(z30["center_distance"].as_f64(), Some(21.0), "{z30}");
    }

    /// A cycloidal disc's edge is the path of n + 1 pins rolling round it, offset by their radius:
    /// at most r + e − r_pin from the centre, at least r − e − r_pin.
    #[test]
    fn cycloidal_discs_follow_their_pins() {
        let s = inserted("cycloidal_disc_d80", json!({"size": "20", "length": 6}));
        let l = run(&mut Session::default(), "parts.library", json!({"family": "cycloidal_disc_d80"}));
        let n20 = l["sizes"].as_array().and_then(|a| a.iter().find(|x| x["size"] == "20")).cloned().unwrap_or_default();
        assert_eq!((n20["pins"].as_u64(), n20["pin_diameter"].as_f64()), (Some(21), Some(5.0)), "{n20}");
        let e = 40.0 / 2.0 / 21.0;
        let radii: Vec<f64> = points(&s, "Cycloidal Disc").iter().map(|p| p.x.hypot(p.y)).filter(|r| *r > 20.0).collect();
        let (lo, hi) = radii.iter().fold((f64::MAX, f64::MIN), |(a, b), r| (a.min(*r), b.max(*r)));
        assert!((hi - (40.0 + e - 2.5)).abs() < 0.02 && (lo - (40.0 - e - 2.5)).abs() < 0.02, "{lo} {hi}");
        // A bore for its eccentric bearing: 30 within the lobes' 36.5.
        assert!((inner(&s, "Cycloidal Disc") - 15.0).abs() < 1e-3, "{}", inner(&s, "Cycloidal Disc"));
        // Pins that would not fit: refused.
        assert!(Session::default().execute("parts.gear", &json!({"family": "cycloidal_disc_d40", "size": "40"})).is_err());
    }

    #[test]
    fn a_screw_seats_in_a_hole_and_resizes() {
        let mut s = Session::default();
        run(&mut s, "solid.box", json!({"length": 40, "width": 30, "height": 10, "body_name": "Plate"}));
        run(&mut s, "solid.hole", json!({"position": [20, 15, 10], "diameter": 6.4}));
        // Pick the hole's rim at the top face.
        let r = run(&mut s, "parts.insert", json!({"family": "socket_head_cap_screw", "size": "M6", "length": 20, "at": [23.2, 15, 10]}));
        assert!(r["joint"].is_number(), "{r}");
        let (lo, hi) = bbox(&s, "SHCS M6x20");
        // The head sits flush on the top face, the shank down the hole's axis.
        assert!((hi.z - 16.0).abs() < 1e-6 && (lo.z + 10.0).abs() < 1e-6, "{lo:?} {hi:?}");
        assert!(((lo.x + hi.x) / 2.0 - 20.0).abs() < 1e-6 && ((lo.y + hi.y) / 2.0 - 15.0).abs() < 1e-6);
        // M8 x 25: rebuilt in place, still seated.
        let comp = r["component"].as_u64().unwrap_or(0);
        run(&mut s, "parts.resize", json!({"component": comp, "size": "M8", "length": 25}));
        let (lo, hi) = bbox(&s, "SHCS M8x25");
        assert!((hi.z - 18.0).abs() < 1e-6 && (lo.z + 15.0).abs() < 1e-6, "{lo:?} {hi:?}");
        assert!(s.model.state().threads.iter().any(|t| t.designation == "M8" && (t.pitch - 1.25).abs() < 1e-9));
    }

    #[test]
    fn every_sketch_of_a_standard_part_is_fully_defined() {
        for (family, size, resized) in [
            ("socket_head_cap_screw", "M6", "M8"),
            ("hex_bolt", "M6", "M8"),
            ("hex_nut", "M6", "M8"),
            ("washer", "M6", "M8"),
            ("set_screw", "M6", "M8"),
            ("dowel_pin", "6", "8"),
            ("bearing", "608", "6200"),
        ] {
            let mut s = Session::default();
            let part = run(&mut s, "parts.insert", json!({"family": family, "size": size}));
            for step in ["inserted", "resized"] {
                let st = s.model.state();
                let free: Vec<_> =
                    st.sketches.iter().filter(|k| k.report.dof != 0 || !k.report.fully_constrained()).map(|k| (&k.name, k.report.dof)).collect();
                assert!(!st.sketches.is_empty() && free.is_empty(), "{family} {step}: {free:?}");
                assert!(s.model.results.iter().all(|r| r.error.is_none()), "{family} {step}");
                run(&mut s, "parts.resize", json!({"component": part["component"], "size": resized}));
            }
        }
    }

    #[test]
    fn resizing_placed_parts_uses_local_geometry_and_millimetres_in_every_design_unit() {
        for (family, initial, resized, prefix) in [
            ("socket_head_cap_screw", "M6", "M8", "SHCS"),
            ("hex_bolt", "M6", "M8", "Hex Bolt"),
            ("hex_nut", "M6", "M8", "Hex Nut"),
            ("washer", "M6", "M8", "Washer"),
            ("set_screw", "M6", "M8", "Set Screw"),
            ("dowel_pin", "6", "8", "Dowel Pin"),
            ("bearing", "608", "6200", "Bearing"),
        ] {
            let mut bounds = Vec::new();
            for units in ["mm", "in"] {
                let mut s = Session::default();
                run(&mut s, "document.units", json!({"units":units}));
                let part = run(&mut s, "parts.insert", json!({"family":family,"size":initial,"length":20,"point":[40,50,60],"direction":[1,0,0]}));
                run(&mut s, "parts.resize", json!({"component":part["component"],"size":resized,"length":30}));
                assert!(s.model.results.iter().all(|r| r.error.is_none()), "{family} {units}");
                let b = bbox(&s, prefix);
                assert!(b.0.x < b.1.x && b.0.y < b.1.y && b.0.z < b.1.z, "{family} {units}: {b:?}");
                if matches!(family, "socket_head_cap_screw" | "hex_bolt" | "hex_nut" | "set_screw") {
                    assert!(s.model.state().threads.iter().any(|t| t.designation == "M8" && (t.pitch - 1.25).abs() < 1e-9), "{family} {units}");
                }
                if family == "dowel_pin" {
                    assert!(b.0.dist(solidcraft_geom::Vec3::new(10., 46., 56.)) < 1e-6, "{b:?}");
                    assert!(b.1.dist(solidcraft_geom::Vec3::new(40., 54., 64.)) < 1e-6, "{b:?}");
                }
                bounds.push(b);
            }
            assert!(bounds[0].0.dist(bounds[1].0) < 1e-6 && bounds[0].1.dist(bounds[1].1) < 1e-6, "{family}: {bounds:?}");
        }
    }

    #[test]
    fn resizing_a_part_is_one_undo_and_keeps_the_active_occurrence() {
        let mut s = Session::default();
        let part = run(&mut s, "parts.insert", json!({"family":"socket_head_cap_screw","size":"M6","length":20}));
        let id = part["component"].as_u64().unwrap_or(0);
        let copied = run(&mut s, "occurrence.copy", json!({"component":id,"translate":[40,0,0]}));
        let path = vec![copied["occurrence"].as_u64().unwrap_or(0)];
        run(&mut s, "component.activate", json!({"component":id,"occurrence":copied["occurrence"],"path":path}));
        let before = s.doc.clone();
        let undo = s.undo.len();
        run(&mut s, "parts.resize", json!({"component":id,"size":"M8","length":25}));
        assert_eq!(s.undo.len(), undo + 1);
        assert_eq!(s.active_component, id);
        assert_eq!(s.active_occurrence_path, Some(path.clone()));
        let after = s.doc.clone();
        run(&mut s, "edit.undo", json!({}));
        assert_eq!(s.doc, before);
        let redo = s.redo.len();
        assert!(s.execute("parts.resize", &json!({"component":id,"length":0})).is_err());
        assert_eq!(s.doc, before);
        assert_eq!(s.redo.len(), redo);
        assert_eq!(s.active_occurrence_path, Some(path));
        run(&mut s, "edit.redo", json!({}));
        assert_eq!(s.doc, after);
    }

    #[test]
    fn resizing_a_part_at_the_history_limit_drops_only_one_old_step() {
        let mut s = Session::default();
        let part = run(&mut s, "parts.insert", json!({"family":"washer","size":"M6"}));
        let Some(snapshot) = s.undo.last().cloned() else { panic!("insert did not record history") };
        s.undo = (0..crate::MAX_UNDO)
            .map(|index| {
                let mut old = snapshot.clone();
                old.label = format!("Earlier step {index}");
                old
            })
            .collect();
        run(&mut s, "parts.resize", json!({"component":part["component"],"size":"M8"}));
        assert_eq!(s.undo.len(), crate::MAX_UNDO);
        assert_eq!(s.undo.first().map(|step| step.label.as_str()), Some("Earlier step 1"));
        assert_eq!(s.undo.last().map(|step| step.label.as_str()), Some("Change Part Size"));
    }
}
