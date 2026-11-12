//! Cutting the skin with a flat plane: where the plane meets each face is a
//! short line, and the lines joined end to end are the outlines of whatever
//! the plane passes through (the trunk, and a leg either side of it).

use std::collections::BTreeMap;

use super::mesh::{Mesh, Point};

/// A plane through `through`, facing along `normal` (of length 1), with the
/// two directions things on it are measured along: `across` and `back`.
#[derive(Clone, Copy, Debug)]
pub struct Plane {
    pub through: Point,
    pub normal: Point,
    pub across: Point,
    pub back: Point,
}

impl Plane {
    /// A plane facing along `normal`, which has no sideways part: things on it
    /// are measured across the body, and towards its back (up for a level
    /// body facing forwards, backwards for one standing upright).
    pub fn facing(through: Point, normal: Point) -> Self {
        Self {
            through,
            normal,
            across: [1.0, 0.0, 0.0],
            back: [0.0, normal[2], -normal[1]],
        }
    }
}

/// A place on a plane: how far across and how far back from its middle.
pub type Flat = [f64; 2];

/// One outline: the short lines that make it, in no order.
#[derive(Clone, Debug, Default)]
pub struct Outline {
    pub lines: Vec<[Flat; 2]>,
}

/// An edge of the mesh, by its two points (lower number first): every place
/// the plane crosses the skin is on one.
type Edge = (usize, usize);

pub fn dot(a: Point, b: Point) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn minus(a: Point, b: Point) -> Point {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Every outline the plane cuts from the skin.
pub fn cut(mesh: &Mesh, plane: &Plane) -> Vec<Outline> {
    // How far in front of the plane each point is. A point exactly on it
    // counts as just in front, so no line ends at a corner.
    let ahead = |at: usize| {
        let distance = dot(minus(mesh.points[at], plane.through), plane.normal);
        if distance == 0.0 {
            f64::MIN_POSITIVE
        } else {
            distance
        }
    };
    let mut lines: Vec<[(Edge, Flat); 2]> = Vec::new();
    for face in &mesh.skin {
        let mut crossings = Vec::new();
        for (corner, &from) in face.iter().enumerate() {
            let to = face[(corner + 1) % face.len()];
            let (a, b) = (ahead(from), ahead(to));
            if (a > 0.0) == (b > 0.0) {
                continue;
            }
            let share = a / (a - b);
            let (p, q) = (mesh.points[from], mesh.points[to]);
            let meet = [0, 1, 2].map(|way| p[way] + (q[way] - p[way]) * share);
            let offset = minus(meet, plane.through);
            let flat = [dot(offset, plane.across), dot(offset, plane.back)];
            crossings.push(((from.min(to), from.max(to)), flat));
        }
        // A flat face is crossed twice or not at all; a bent four-sided one
        // may be crossed four times, as two lines.
        for pair in crossings.as_chunks::<2>().0 {
            lines.push(*pair);
        }
    }
    join(&lines)
}

/// Sorts lines into outlines: two lines are in the same one when they end on
/// the same edge of the mesh.
fn join(lines: &[[(Edge, Flat); 2]]) -> Vec<Outline> {
    // Each edge's group is the first edge found in it ("union-find").
    let mut parent: BTreeMap<Edge, Edge> = BTreeMap::new();
    fn root(parent: &mut BTreeMap<Edge, Edge>, edge: Edge) -> Edge {
        let mut at = edge;
        while let Some(&up) = parent.get(&at) {
            if up == at {
                break;
            }
            at = up;
        }
        parent.insert(edge, at);
        at
    }
    for [(a, _), (b, _)] in lines {
        let (a, b) = (root(&mut parent, *a), root(&mut parent, *b));
        parent.insert(a, b);
    }
    let mut outlines: BTreeMap<Edge, Outline> = BTreeMap::new();
    for [(a, from), (_, to)] in lines {
        let group = root(&mut parent, *a);
        outlines.entry(group).or_default().lines.push([*from, *to]);
    }
    outlines.into_values().collect()
}

impl Outline {
    /// The least and most of its places, across (0) or back (1).
    pub fn span(&self, way: usize) -> (f64, f64) {
        let places = self.lines.iter().flatten().map(|place| place[way]);
        places.fold((f64::MAX, f64::MIN), |(low, high), value| {
            (low.min(value), high.max(value))
        })
    }

    /// How far back each place is where the outline crosses the line `across`
    /// = `at`, lowest first.
    pub fn crossings(&self, at: f64) -> Vec<f64> {
        let mut found = Vec::new();
        for [from, to] in &self.lines {
            let (a, b) = (from[0] - at, to[0] - at);
            if (a > 0.0) != (b > 0.0) {
                found.push(from[1] + (to[1] - from[1]) * a / (a - b));
            }
        }
        found.sort_by(f64::total_cmp);
        found
    }
}
