//! Cosmetic threads as the viewport draws them: the model keeps a plain cylinder, so a thin helix
//! on the threaded face and a circle at the thread's other diameter at each end of the threaded
//! length show where the thread is. Lines only, in world space like the thread; nothing is added
//! to the body.

use solidcraft_doc::ThreadInfo;
use solidcraft_geom::{Mesh, Vec3};
use solidcraft_render::Rgb;

/// The most turns a thread's helix is drawn with: a longer thread (or a finer pitch) is drawn
/// with this many turns spread over its length, so a long fine thread stays a few thousand points.
pub const MAX_THREAD_TURNS: f64 = 120.0;

/// Segments per drawn turn of the helix, and per end circle.
pub const THREAD_TURN_SEGMENTS: usize = 24;

/// The lines one cosmetic thread is drawn with.
#[derive(Clone, Debug, PartialEq)]
pub struct ThreadLines {
    /// The helix on the threaded face, from the start of the threaded length to its end.
    pub helix: Vec<Vec3>,
    /// A closed circle at each end of the threaded length, at the other diameter of the thread
    /// (the minor for a shaft, the major for a hole).
    pub ends: Vec<Vec<Vec3>>,
    /// How many turns the helix is drawn with (at most [`MAX_THREAD_TURNS`]).
    pub turns: f64,
    pub helix_radius: f64,
    pub end_radius: f64,
}

/// The basic minor diameter of a thread: the major less twice the depth of the basic profile
/// (5/8 of the 60° triangle; 2/3 of the 55° pipe thread's).
pub fn thread_minor_diameter(t: &ThreadInfo) -> f64 {
    let pipe = solidcraft_doc::parse_tapped_thread(&t.designation).is_some_and(|s| s.profile_degrees() < 57.5);
    t.major_diameter - t.pitch * if pipe { 1.280_654 } else { 1.082_532 }
}

/// The radius of the face a thread is on, from the vertices of its body's mesh along the threaded
/// length: the one nearest the thread's own surface (the major radius of a shaft, the minor of a
/// hole). `None` when no vertex there is near it: the thread's own diameter is drawn then.
pub fn thread_face_radius(t: &ThreadInfo, mesh: &Mesh) -> Option<f64> {
    let target = if t.internal { thread_minor_diameter(t) } else { t.major_diameter } * 0.5;
    let (lo, hi) = (t.start.min(t.end), t.start.max(t.end));
    let slack = (hi - lo) * 1e-3 + 1e-6;
    let axis = *t.axis;
    let best = mesh
        .positions
        .iter()
        .filter_map(|p| {
            let d = *p - t.axis_point;
            let along = d.dot(axis);
            (along >= lo - slack && along <= hi + slack).then(|| (d - axis * along).len())
        })
        .filter(|r| r.is_finite())
        .min_by(|a, b| (a - target).abs().total_cmp(&(b - target).abs()))?;
    ((best - target).abs() <= t.major_diameter * 0.25).then_some(best)
}

/// The lines of thread `t`. `face_radius` is the radius of the face it is on
/// ([`thread_face_radius`]); `tol` the chord tolerance its body is meshed with, so the helix
/// stays clear of the facets: on a shaft they lie inside the true cylinder, in a hole they reach
/// into it. `None` for a thread with no length or a size that is not a number.
pub fn thread_lines(t: &ThreadInfo, face_radius: Option<f64>, tol: f64) -> Option<ThreadLines> {
    let (major, pitch, length) = (t.major_diameter * 0.5, t.pitch, t.end - t.start);
    if !(major.is_finite() && major > 0.0 && pitch.is_finite() && pitch > 0.0 && length.is_finite() && length.abs() > 1e-9) {
        return None;
    }
    if !(t.axis_point.is_finite() && t.start.is_finite()) {
        return None;
    }
    let minor = (thread_minor_diameter(t) * 0.5).max(major * 0.5);
    let tol = if tol.is_finite() { tol.clamp(0.0, major * 0.05) } else { 0.0 };
    let face = face_radius.filter(|r| r.is_finite() && *r > 0.0);
    let (helix_radius, end_radius) = if t.internal {
        (face.map_or(minor, |r| r.min(minor)) - tol * 1.5, major)
    } else {
        (face.map_or(major, |r| r.max(major)) + tol * 0.25, minor)
    };
    let helix_radius = helix_radius.max(minor * 0.5);
    let axis = *t.axis;
    let u = axis.any_perp();
    let v = axis.cross(u);
    let at = |along: f64, angle: f64, r: f64| t.axis_point + axis * along + (u * angle.cos() + v * angle.sin()) * r;
    let turns = (length.abs() / pitch).min(MAX_THREAD_TURNS);
    let n = ((turns * THREAD_TURN_SEGMENTS as f64).ceil() as usize).max(2);
    let helix = (0..=n)
        .map(|i| {
            let f = i as f64 / n as f64;
            at(t.start + length * f, std::f64::consts::TAU * turns * f, helix_radius)
        })
        .collect();
    let circle = |along: f64| -> Vec<Vec3> {
        (0..=THREAD_TURN_SEGMENTS).map(|i| at(along, std::f64::consts::TAU * i as f64 / THREAD_TURN_SEGMENTS as f64, end_radius)).collect()
    };
    Some(ThreadLines { helix, ends: vec![circle(t.start), circle(t.end)], turns, helix_radius, end_radius })
}

/// The colour a cosmetic thread is drawn in: the edge colour toward the body's, so the helix reads
/// as a light pattern on the face rather than as edges.
pub fn thread_color(edge: Rgb, body: Rgb) -> Rgb {
    let mix = |a: u8, b: u8| (f64::from(a) * 0.55 + f64::from(b) * 0.45).round() as u8;
    Rgb(mix(edge.0, body.0), mix(edge.1, body.1), mix(edge.2, body.2))
}
