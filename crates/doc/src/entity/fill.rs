use super::common::Handle;
use super::{Color, Deserialize, PolyVertex, Serialize, Vec2, one};
use cadcraft_geom::Polyline;

/// A hatch boundary loop: closed polyline with bulges.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HatchLoop {
    pub vertices: Vec<PolyVertex>,
    #[serde(default)]
    pub outer: bool,
    /// The objects the loop was made from (DXF group 97 count, 330 handles): an associative
    /// hatch re-fills the loop from them when they change.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<Handle>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hatch {
    pub pattern: String,
    pub solid: bool,
    pub loops: Vec<HatchLoop>,
    #[serde(default = "one")]
    pub scale: f64,
    #[serde(default)]
    pub angle: f64,
    #[serde(default)]
    pub associative: bool,
    /// 0 normal (odd parity), 1 outer, 2 ignore.
    #[serde(default)]
    pub style: u8,
    #[serde(default)]
    pub elevation: f64,
    /// Gradient fill: name + second colour.
    #[serde(default)]
    pub gradient: Option<Gradient>,
    #[serde(default)]
    pub origin: Vec2,
    #[serde(default)]
    pub background: Option<Color>,
    /// The pattern's line families as a file defined them, for a pattern that isn't in the
    /// standard library (unit scale and angle, like the library's; empty otherwise).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pattern_lines: Vec<crate::library::PatternLine>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Gradient {
    pub name: String,
    pub color1: Color,
    pub color2: Color,
    #[serde(default)]
    pub angle: f64,
    #[serde(default)]
    pub centered: bool,
}

impl Hatch {
    /// The filled area and the length of every loop, as drawn with odd parity: a loop nested
    /// inside an odd number of other loops is an unfilled island, one nested inside an even number
    /// is filled again. (What AREA's Object option and the Properties palette report.)
    pub fn area_and_perimeter(&self) -> (f64, f64) {
        let geoms: Vec<Polyline> = self.loops.iter().map(|l| Polyline { vertices: l.vertices.clone(), closed: true }).collect();
        let polys: Vec<Vec<Vec2>> = geoms.iter().map(|g| g.tessellate(1e-3)).collect();
        let mut total = (0.0, 0.0);
        for (i, g) in geoms.iter().enumerate() {
            let probe = polys.get(i).and_then(|p| p.first().copied());
            let depth = probe.map_or(0, |pt| polys.iter().enumerate().filter(|(j, q)| *j != i && cadcraft_geom::point_in_polygon(q, pt)).count());
            total.0 += if depth % 2 == 0 { g.area().abs() } else { -g.area().abs() };
            total.1 += g.len();
        }
        total
    }

    /// The filled area (see [`Hatch::area_and_perimeter`]).
    pub fn area(&self) -> f64 {
        self.area_and_perimeter().0
    }
}
