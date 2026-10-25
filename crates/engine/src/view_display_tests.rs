use super::*;
use serde_json::json;
use solidcraft_geom::Vec2;
use solidcraft_sketch::{LinkGeom, LinkKind, LinkSource};

fn line(sk: &mut Sketch, y: f64) -> usize {
    sk.add_line(Vec2::new(0., y), Vec2::new(2., y), None, None, None).unwrap()
}

#[test]
fn sampler_keeps_style_precedence_hidden_references_text_and_disjoint_parts() {
    let mut sk = Sketch::new();
    let ordinary = line(&mut sk, 0.);
    let fixed = line(&mut sk, 1.);
    sk.curves[fixed].fixed = true;
    sk.curves[fixed].construction = true;
    let centerline = line(&mut sk, 2.);
    sk.curves[centerline].centerline = true;
    let construction = line(&mut sk, 3.);
    sk.curves[construction].construction = true;
    let determined = line(&mut sk, 4.);
    let projected = sk.add_link(LinkKind::Project, LinkSource::Origin, &[LinkGeom::Line(Vec2::new(0., 5.), Vec2::new(2., 5.))]).unwrap();
    let lost = sk.add_link(LinkKind::Project, LinkSource::Origin, &[LinkGeom::Line(Vec2::new(0., 6.), Vec2::new(2., 6.))]).unwrap();
    sk.set_link_lost(&lost, true);
    let text = sk
        .add_link(
            LinkKind::Text,
            LinkSource::Text {
                text: "A".into(),
                at: Vec2::ZERO,
                height: 2.,
                angle: 0.,
                font: None,
                style: Default::default(),
                exprs: Default::default(),
            },
            &[LinkGeom::Line(Vec2::new(0., 7.), Vec2::new(2., 7.))],
        )
        .unwrap();
    let mut determined_flags = vec![false; sk.curves.len()];
    determined_flags[determined] = true;
    let saved = serde_json::to_value(&sk).unwrap();
    let mut called = Vec::new();
    let lines = sketch_lines_with_sampler(&sk, &Plane::XY, true, &determined_flags, |_, i, _| {
        called.push(i);
        vec![vec![Vec3::new(i as f64, 0., 0.), Vec3::new(i as f64, 1., 0.)], vec![Vec3::new(i as f64, 3., 0.), Vec3::new(i as f64, 4., 0.)]]
    });
    assert_eq!(called, (0..sk.curves.len()).collect::<Vec<_>>());
    for (i, expected) in [
        colors::SKETCH,
        colors::SKETCH_LOCKED,
        colors::SKETCH_CENTERLINE,
        colors::SKETCH_CONSTRUCTION,
        colors::SKETCH_FIXED,
        colors::SKETCH_PROJECTED,
        colors::SKETCH_LOST,
        colors::SKETCH,
    ]
    .into_iter()
    .enumerate()
    {
        for part in &lines[2 * i..2 * i + 2] {
            assert_eq!(part.1, expected);
            assert_eq!(part.0.len(), 2);
        }
        assert_eq!(lines[2 * i].2, [fixed, centerline, construction].contains(&i));
    }
    assert_eq!(serde_json::to_value(&sk).unwrap(), saved);
    sk.view.hide_projected = true;
    called.clear();
    let finished = sketch_lines_with_sampler(&sk, &Plane::XY, false, &determined_flags, |_, i, _| {
        called.push(i);
        vec![vec![Vec3::ZERO, Vec3::X]]
    });
    assert_eq!(called, vec![ordinary, fixed, centerline, construction, determined, 7]);
    assert_eq!(finished[0].1, colors::SKETCH_DONE);
    assert_eq!(finished[4].1, colors::SKETCH_DONE);
    assert_eq!(finished[5].1, colors::SKETCH_DONE);
    assert!(sk.link(&projected).is_some() && sk.link(&text).is_some());
}

/// Construction Geometries hides the `construction` line type and nothing else. Centerlines
/// are their own line type. A projected curve that carries the construction flag (the trace of
/// a plane or an axis) is drawn in the projected colour, so Projected Geometries owns it and
/// Construction Geometries leaves it alone. `sketch_curve_visible` is the one answer the GPU
/// viewport's picking uses as well, so nothing hidden here can still be clicked there.
#[test]
fn hiding_construction_geometry_keeps_ordinary_curves_centerlines_and_projected_traces() {
    let mut sk = Sketch::new();
    let ordinary = line(&mut sk, 0.);
    let construction = line(&mut sk, 1.);
    sk.curves[construction].construction = true;
    let centerline = line(&mut sk, 2.);
    sk.curves[centerline].centerline = true;
    let axis = sk.add_link(LinkKind::Project, LinkSource::Origin, &[LinkGeom::Line(Vec2::new(0., 3.), Vec2::new(2., 3.))]).unwrap();
    let trace = sk.link_curves(&axis)[0];
    sk.curves[trace].construction = true;
    let determined = vec![false; sk.curves.len()];
    let drawn = |sk: &Sketch| -> Vec<usize> {
        let mut seen = Vec::new();
        sketch_lines_with_sampler(sk, &Plane::XY, true, &determined, |_, i, _| {
            seen.push(i);
            vec![vec![Vec3::ZERO, Vec3::X]]
        });
        seen
    };
    assert_eq!(drawn(&sk), vec![ordinary, construction, centerline, trace]);

    sk.view.hide_construction = true;
    assert_eq!(drawn(&sk), vec![ordinary, centerline, trace], "only the construction curve goes");
    for (i, want) in [(ordinary, true), (construction, false), (centerline, true), (trace, true)] {
        assert_eq!(sketch_curve_visible(&sk, i), want, "curve {i}");
    }
    // The projected trace answers to its own toggle, in either order.
    sk.view.hide_projected = true;
    assert_eq!(drawn(&sk), vec![ordinary, centerline]);
    sk.view.hide_construction = false;
    assert_eq!(drawn(&sk), vec![ordinary, construction, centerline]);
    // A curve index that does not exist is not visible, and nothing panics asking.
    assert!(!sketch_curve_visible(&sk, sk.curves.len()));
}

#[test]
fn legacy_paths_match_sampler_wrapper_and_placed_wires_are_unchanged() {
    let mut sk = Sketch::new();
    sk.add_circle(Vec2::new(2., 3.), 1., None, None).unwrap();
    sk.add_drawn_wire(&[Vec3::new(8., 9., 10.), Vec3::new(11., 12., 13.)]).unwrap();
    let plane = Plane::new(Vec3::new(20., 30., 40.), Vec3::Y, Vec3::Z).unwrap();
    let old = sketch_lines(&sk, &plane, true, &[false]);
    let adapted = sketch_lines_with_sampler(&sk, &plane, true, &[false], world_curve_parts);
    assert_eq!(old, adapted);
    let clipped = sketch_lines_with_sampler(&sk, &plane, true, &[], |_, _, _| Vec::new());
    assert_eq!(clipped, vec![(sk.wires[0].pts.clone(), colors::SKETCH, false)]);
    let mut included = sk.wires[0].clone();
    included.id = "included".into();
    included.fit.clear();
    sk.wires.push(included);
    let wires = sketch_lines_with_sampler(&sk, &plane, false, &[], |_, _, _| Vec::new());
    assert_eq!(wires[0].1, colors::SKETCH_DONE);
    assert_eq!(wires[1], (sk.wires[1].pts.clone(), colors::SKETCH_PROJECTED, false));
    sk.view.hide_projected = true;
    assert_eq!(sketch_lines_with_sampler(&sk, &plane, false, &[], |_, _, _| Vec::new()), vec![wires[0].clone()]);
}

#[test]
fn scene_sampler_uses_exact_active_occurrence_plane_and_preserves_other_scene_content() {
    let mut s = Session::default();
    s.execute("solid.box", &json!({"length":4,"width":5,"height":6})).unwrap();
    let component = s.execute("component.create", &json!({"name":"Part"})).unwrap()["component"].as_u64().unwrap();
    let occurrence = s.execute("occurrence.copy", &json!({"component":component,"translate":[20,30,40]})).unwrap()["occurrence"].as_u64().unwrap();
    s.execute("occurrence.move", &json!({"occurrence":occurrence,"angle":"90 deg","axis":[1,0,0],"origin":[20,30,40],"capture":false})).unwrap();
    s.execute("component.activate", &json!({"component":component,"occurrence":occurrence,"path":[occurrence]})).unwrap();
    s.execute("sketch.create", &json!({"plane":"XY"})).unwrap();
    s.execute("sketch.circle.center", &json!({"center":[2,3],"radius":1})).unwrap();
    s.section = Some((Vec3::new(0., 0., 3.), Vec3::Z));
    let cam = Camera::default();
    let old = scene(&s, &cam);
    let default = scene_with_sketch_sampler(&s, &cam, world_curve_parts);
    assert_eq!(old.lines.len(), default.lines.len());
    for (a, b) in old.lines.iter().zip(&default.lines) {
        assert_eq!((&a.points, a.color, a.width, a.on_top), (&b.points, b.color, b.width, b.on_top));
    }
    let mut planes = Vec::new();
    let parts = vec![vec![Vec3::new(22., 33., 40.), Vec3::new(23., 33., 40.)], vec![Vec3::new(25., 33., 40.), Vec3::new(26., 33., 40.)]];
    let sampled = scene_with_sketch_sampler(&s, &cam, |_, _, plane| {
        planes.push(*plane);
        parts.clone()
    });
    assert_eq!(planes.len(), 1);
    assert!(planes[0].origin.dist(Vec3::new(20., 30., 40.)) < 1e-9);
    assert!(planes[0].x.dist(Vec3::X) < 1e-9);
    assert!(planes[0].y.dist(Vec3::Z) < 1e-9);
    assert_eq!(sampled.clip, old.clip);
    assert_eq!(sampled.background, old.background);
    assert_eq!(sampled.radius, old.radius);
    assert_eq!(sampled.meshes.len(), old.meshes.len());
    assert!(Arc::ptr_eq(&sampled.meshes[0].mesh, &old.meshes[0].mesh));
    assert_eq!(sampled.meshes[0].color, old.meshes[0].color);
    assert_eq!(sampled.meshes[0].face_colors, old.meshes[0].face_colors);
    let lines = &sampled.lines[sampled.lines.len() - 2..];
    for (line, part) in lines.iter().zip(parts) {
        assert_eq!(line.points, part);
        assert!(line.on_top);
        assert_eq!(line.width, 1.6);
    }
    let active = s.active_sketch.unwrap();
    s.execute("sketch.finish", &json!({})).unwrap();
    s.visibility.hidden_sketches.push(active);
    let hidden = scene_with_sketch_sampler(&s, &cam, |_, _, _| panic!("hidden sketch sampled"));
    assert!(hidden.lines.iter().all(|line| !line.on_top));
}

fn thread(internal: bool, major: f64, pitch: f64, start: f64, end: f64) -> solidcraft_doc::ThreadInfo {
    solidcraft_doc::ThreadInfo {
        feature: 1,
        body: "Body".into(),
        designation: format!("M{major}x{pitch}"),
        major_diameter: major,
        pitch,
        internal,
        axis_point: Vec3::new(1., 2., 3.),
        axis: solidcraft_geom::UnitVec3::Z,
        start,
        end,
    }
}

/// Distance from the thread's axis, and position along it.
fn radial(t: &solidcraft_doc::ThreadInfo, p: Vec3) -> (f64, f64) {
    let d = p - t.axis_point;
    let along = d.dot(*t.axis);
    ((d - *t.axis * along).len(), along)
}

#[test]
fn a_shaft_thread_is_a_helix_at_the_major_radius_one_turn_per_pitch_over_its_length() {
    let t = thread(false, 6., 1., -20., 0.);
    let lines = thread_lines(&t, Some(3.), 0.).unwrap();
    assert!((lines.turns - 20.).abs() < 1e-12);
    assert_eq!(lines.helix.len(), 20 * THREAD_TURN_SEGMENTS + 1);
    for p in &lines.helix {
        assert!((radial(&t, *p).0 - 3.).abs() < 1e-9, "{p:?}");
    }
    let alongs: Vec<f64> = lines.helix.iter().map(|p| radial(&t, *p).1).collect();
    assert!((alongs[0] + 20.).abs() < 1e-9 && alongs.last().unwrap().abs() < 1e-9, "{alongs:?}");
    assert!(alongs.windows(2).all(|w| w[1] > w[0]), "the helix climbs steadily");
    // One whole turn per pitch: every turn ends where it started, a pitch further along.
    let first = lines.helix[0];
    let turn = lines.helix[THREAD_TURN_SEGMENTS];
    assert!((turn - first - Vec3::Z).len() < 1e-9, "{first:?} {turn:?}");
    // A circle at the minor diameter at each end of the threaded length.
    let minor = (6. - 1.082_532) / 2.;
    assert!((lines.end_radius - minor).abs() < 1e-9);
    assert_eq!(lines.ends.len(), 2);
    for (end, along) in lines.ends.iter().zip([-20., 0.]) {
        assert!((end[0] - *end.last().unwrap()).len() < 1e-9, "closed");
        for p in end {
            let (r, a) = radial(&t, *p);
            assert!((r - minor).abs() < 1e-9 && (a - along).abs() < 1e-9, "{p:?}");
        }
    }
}

#[test]
fn a_hole_thread_is_drawn_inside_the_hole_clear_of_its_facets() {
    // M6 tapped at the 5 mm drill: the helix just inside the wall, the major circle at the ends.
    let t = thread(true, 6., 1., 0., 12.);
    let lines = thread_lines(&t, Some(2.5), 0.01).unwrap();
    let minor = (6. - 1.082_532) / 2.;
    assert!((lines.helix_radius - (minor - 0.015)).abs() < 1e-9, "{}", lines.helix_radius);
    assert!(lines.helix_radius < 2.5);
    assert!((lines.end_radius - 3.).abs() < 1e-12);
    // A hole drilled under the basic minor diameter: the helix follows its wall.
    let tight = thread_lines(&t, Some(2.3), 0.01).unwrap();
    assert!((tight.helix_radius - 2.285).abs() < 1e-9, "{}", tight.helix_radius);
    // A shaft turned over its thread's size: the helix stays on its face, not inside it.
    let fat = thread_lines(&thread(false, 6., 1., 0., 12.), Some(3.1), 0.).unwrap();
    assert!((fat.helix_radius - 3.1).abs() < 1e-9);
}

#[test]
fn long_fine_threads_are_drawn_with_a_capped_number_of_turns() {
    let t = thread(false, 20., 0.1, 0., 500.);
    let lines = thread_lines(&t, None, 0.).unwrap();
    assert!((lines.turns - MAX_THREAD_TURNS).abs() < 1e-12);
    assert_eq!(lines.helix.len(), MAX_THREAD_TURNS as usize * THREAD_TURN_SEGMENTS + 1);
    let (_, last) = radial(&t, *lines.helix.last().unwrap());
    assert!((last - 500.).abs() < 1e-9, "still the whole threaded length");
    // A partial last turn ends where the thread does.
    let short = thread_lines(&thread(false, 6., 1., 0., 2.5), None, 0.).unwrap();
    assert!((short.turns - 2.5).abs() < 1e-12);
    assert!((radial(&t, *short.helix.last().unwrap()).1 - 2.5).abs() < 1e-9);
}

#[test]
fn threads_with_hostile_numbers_draw_nothing() {
    for t in [
        thread(false, f64::NAN, 1., 0., 10.),
        thread(false, 6., 0., 0., 10.),
        thread(false, 6., -1., 0., 10.),
        thread(false, 6., f64::INFINITY, 0., 10.),
        thread(false, 6., 1., 5., 5.),
        thread(false, 6., 1., 0., f64::NAN),
        thread(false, -6., 1., 0., 10.),
    ] {
        assert!(thread_lines(&t, Some(f64::NAN), f64::NAN).is_none(), "{t:?}");
    }
    // A face radius or tolerance that is not a number is ignored, not drawn.
    let lines = thread_lines(&thread(false, 6., 1., 0., 10.), Some(f64::NAN), f64::INFINITY).unwrap();
    assert!(lines.helix.iter().chain(lines.ends.iter().flatten()).all(|p| p.is_finite()));
    assert!((lines.helix_radius - 3.).abs() < 1e-12);
}

/// The headless scene's lines in the cosmetic thread colour.
fn thread_scene_lines(s: &Session) -> Vec<Vec<Vec3>> {
    let color = thread_color(Palette::DEFAULT.edge, Palette::DEFAULT.body);
    scene(s, &home_camera(s)).lines.into_iter().filter(|l| l.color == color).map(|l| l.points).collect()
}

#[test]
fn an_inserted_cap_screw_and_a_tapped_hole_show_their_threads() {
    let mut s = Session::default();
    s.execute("parts.insert", &json!({"family": "socket_head_cap_screw", "size": "M6", "length": 20})).unwrap();
    let lines = thread_scene_lines(&s);
    assert_eq!(lines.len(), 3, "a helix and two end circles");
    let st = s.world_state();
    let t = st.threads.first().unwrap();
    let helix = lines.iter().max_by_key(|l| l.len()).unwrap();
    assert!(helix.len() > 10 * THREAD_TURN_SEGMENTS, "{} points", helix.len());
    for p in helix {
        let (r, along) = radial(t, *p);
        assert!((2.9..3.1).contains(&r), "on the shank: {r}");
        assert!(along >= t.start.min(t.end) - 1e-6 && along <= t.start.max(t.end) + 1e-6);
    }
    // Hidden with its body.
    s.execute("browser.visibility", &json!({"items": [t.body], "visible": false})).unwrap();
    assert!(thread_scene_lines(&s).is_empty());

    let mut s = Session::default();
    s.execute("solid.box", &json!({"length": 30, "width": 30, "height": 12, "body_name": "Plate"})).unwrap();
    s.execute("solid.hole", &json!({"position": [15, 15, 12], "thread": "M6"})).unwrap();
    let st = s.world_state();
    let t = st.threads.first().unwrap();
    let face = thread_face_radius(t, &st.body("Plate").unwrap().mesh()).unwrap();
    assert!((face - 2.5).abs() < 1e-6, "the 5 mm tap drill: {face}");
    let lines = thread_scene_lines(&s);
    assert_eq!(lines.len(), 3);
    let helix = lines.iter().max_by_key(|l| l.len()).unwrap();
    assert!((helix.len() - 1) as f64 >= 12. * THREAD_TURN_SEGMENTS as f64, "a turn per millimetre over the plate");
    assert!(helix.iter().all(|p| radial(t, *p).0 < 2.5), "inside the hole");
}
