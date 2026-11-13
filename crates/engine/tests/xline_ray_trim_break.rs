//! TRIM and BREAK on xlines and rays (issue #1235): both refused them ("Cannot trim that object.",
//! "Cannot break that object."). As AutoCAD does, a finite piece left over becomes a line and an
//! unbounded one a ray: an xline trimmed or broken in its middle leaves two rays, a ray a line from
//! its base and a ray.

use cadcraft_engine::Session;
use cadcraft_engine::doc::EntityKind;
use serde_json::{Value, json};

/// Every ray as (base x, base y, dir x, dir y) and every line as its two ends, both sorted.
#[allow(clippy::type_complexity)]
fn pieces(s: &Session) -> (Vec<(f64, f64, f64, f64)>, Vec<((f64, f64), (f64, f64))>, usize) {
    let d = s.doc().unwrap();
    let mut rays: Vec<_> =
        d.model.iter().filter_map(|e| if let EntityKind::Ray(r) = &e.kind { Some((r.base.x, r.base.y, r.dir.x, r.dir.y)) } else { None }).collect();
    rays.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut lines: Vec<_> =
        d.model.iter().filter_map(|e| if let EntityKind::Line(l) = &e.kind { Some(((l.a.x, l.a.y), (l.b.x, l.b.y))) } else { None }).collect();
    lines.sort_by(|a, b| a.0.0.total_cmp(&b.0.0));
    let xlines = d.model.iter().filter(|e| matches!(e.kind, EntityKind::XLine(_))).count();
    (rays, lines, xlines)
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-12
}

/// A session with a horizontal xline (or ray from the origin) along y = 0 and two vertical
/// cutting lines at x = 10 and x = 20. Returns the session and the xline's or ray's handle.
fn crossed(command: &str) -> (Session, Value) {
    let mut s = Session::new();
    let out = s.execute(command, &json!({ "base": [0, 0], "through": [1, 0] })).unwrap();
    s.execute("line", &json!({ "points": [[10, -5], [10, 5]] })).unwrap();
    s.execute("line", &json!({ "points": [[20, -5], [20, 5]] })).unwrap();
    (s, out["handle"].clone())
}

#[test]
fn trimming_an_xline_between_two_edges_leaves_two_rays() {
    let (mut s, h) = crossed("xline");
    s.execute("trim", &json!({ "handle": h, "pick": [15, 0] })).unwrap();
    let (rays, _, xlines) = pieces(&s);
    assert_eq!(xlines, 0);
    assert_eq!(rays.len(), 2, "{rays:?}");
    assert!(near(rays[0].0, 10.0) && near(rays[0].2, -1.0), "a ray from x = 10 running back: {rays:?}");
    assert!(near(rays[1].0, 20.0) && near(rays[1].2, 1.0), "a ray from x = 20 running on: {rays:?}");
}

#[test]
fn trimming_an_xline_outside_the_edges_leaves_one_ray() {
    let (mut s, h) = crossed("xline");
    s.execute("trim", &json!({ "handle": h, "pick": [-50, 0] })).unwrap();
    let (rays, _, xlines) = pieces(&s);
    assert_eq!(xlines, 0);
    assert_eq!(rays.len(), 1, "{rays:?}");
    assert!(near(rays[0].0, 10.0) && near(rays[0].2, 1.0), "{rays:?}");
}

#[test]
fn trimming_a_ray_leaves_a_line_from_its_base_and_a_ray() {
    let (mut s, h) = crossed("ray");
    s.execute("trim", &json!({ "handle": h, "pick": [15, 0] })).unwrap();
    let (rays, lines, _) = pieces(&s);
    assert_eq!(rays.len(), 1, "{rays:?}");
    assert!(near(rays[0].0, 20.0) && near(rays[0].2, 1.0), "{rays:?}");
    assert!(lines.iter().any(|l| near(l.0.0, 0.0) && near(l.1.0, 10.0) && near(l.0.1, 0.0)), "a line from the base to x = 10: {lines:?}");
    // The far end of what is left: only the line remains.
    let h = s.doc().unwrap().model.iter().find(|e| matches!(e.kind, EntityKind::Ray(_))).unwrap().handle.hex();
    s.execute("trim", &json!({ "handle": h, "pick": [100, 0] })).unwrap_err();
    s.execute("trim", &json!({ "handle": h, "pick": [100, 0], "mode": "quick" })).unwrap();
    assert!(pieces(&s).0.is_empty(), "a quick trim of a ray no edge crosses erases it");
}

#[test]
fn trimming_an_xline_no_edge_crosses_is_refused() {
    let mut s = Session::new();
    let out = s.execute("xline", &json!({ "base": [0, 0], "through": [1, 0] })).unwrap();
    let e = s.execute("trim", &json!({ "handle": out["handle"], "pick": [5, 0] })).unwrap_err();
    assert!(e.to_string().contains("intersect"), "{e}");
}

#[test]
fn breaking_an_xline_leaves_two_rays() {
    let (mut s, h) = crossed("xline");
    s.execute("break", &json!({ "handle": h, "p1": [3, 0], "p2": [7, 0] })).unwrap();
    let (rays, _, xlines) = pieces(&s);
    assert_eq!(xlines, 0);
    assert_eq!(rays.len(), 2, "{rays:?}");
    assert!(near(rays[0].0, 3.0) && near(rays[0].2, -1.0), "{rays:?}");
    assert!(near(rays[1].0, 7.0) && near(rays[1].2, 1.0), "{rays:?}");
}

#[test]
fn breaking_an_xline_at_a_point_leaves_two_rays_from_it() {
    let (mut s, h) = crossed("xline");
    s.execute("breakatpoint", &json!({ "handle": h, "at": [4, 0] })).unwrap();
    let (rays, _, _) = pieces(&s);
    assert_eq!(rays.len(), 2, "{rays:?}");
    assert!(rays.iter().all(|r| near(r.0, 4.0)), "{rays:?}");
}

#[test]
fn breaking_a_ray_leaves_a_line_and_a_ray() {
    let (mut s, h) = crossed("ray");
    s.execute("break", &json!({ "handle": h, "p1": [3, 0], "p2": [7, 0] })).unwrap();
    let (rays, lines, _) = pieces(&s);
    assert_eq!(rays.len(), 1, "{rays:?}");
    assert!(near(rays[0].0, 7.0) && near(rays[0].2, 1.0), "{rays:?}");
    assert!(lines.iter().any(|l| near(l.0.0, 0.0) && near(l.1.0, 3.0)), "{lines:?}");
}

/// At survey coordinates (half a million metres from the origin) the cut lands on the edge.
#[test]
fn trimming_an_xline_far_from_the_origin_cuts_exactly_at_the_edge() {
    let mut s = Session::new();
    let out =
        s.execute("xline", &json!({ "base": [446269.0159378149, -35672.82525121742], "through": [446270.0159378149, -35672.82525121742] })).unwrap();
    s.execute("line", &json!({ "points": [[446300.25, -35700.0], [446300.25, -35600.0]] })).unwrap();
    s.execute("trim", &json!({ "handle": out["handle"], "pick": [446400.0, -35672.82525121742] })).unwrap();
    let (rays, _, _) = pieces(&s);
    assert_eq!(rays.len(), 1, "{rays:?}");
    assert!((rays[0].0 - 446300.25).abs() < 1e-9, "{rays:?}");
}
