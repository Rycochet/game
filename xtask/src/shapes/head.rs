//! Measuring an animal's head, as the shared body describes one
//! (`shared::body::HeadShape`): rings along the head's own line, from the
//! back of the skull to the nose, and how far that line leans nose down.
//!
//! The heads are a person's reshaped (a person with a wolf's head, or the
//! horse's whole body), so their joints keep a person's names: the `head`
//! joint is where the skull sits on the neck. The head's line runs from there
//! to the tip of the nose. Below the jaw the skin carries on down the neck,
//! so every ring stops at the jaw's underside, where our neck takes over.

use super::cut::{Plane, cut, dot, minus};
use super::mesh::{Mesh, Point};
use super::ring::Ring;

/// How many steps the search for the back of the skull takes, from the
/// `head` joint backwards.
const BACK_STEPS: usize = 400;
/// The last ring is this share of the way from the back of the skull to the
/// nose, as the nose's very tip has nothing to cut.
const LAST: f64 = 0.98;
/// A horse's head grows out of its neck with no back of the skull to find:
/// the rings start no further behind the `head` joint than this share of the
/// way from it to the nose.
const MOST_BEHIND: f64 = 1.0;
/// The ears stand on the top of the head, beside its middle line: the half
/// width is measured below this share of a ring's depth from its top.
const EARS: f64 = 0.25;

/// Everything measured, in the mesh's own units. "Along" is from the `head`
/// joint towards the nose; "up" square to that, towards the top of the head.
#[derive(Clone, Debug)]
pub struct Head {
    /// From the back of the skull to the nose.
    pub length: f64,
    /// How far the line from the `head` joint to the nose leans down from
    /// level, in degrees.
    pub pitch: f64,
    /// Rings from the back of the skull to the nose, each with how far along
    /// it is; belly and back are how far up from the head's line.
    pub rings: Vec<(f64, Ring)>,
}

impl Head {
    /// Measures the head of a mesh facing forwards, `rings` rings along it.
    pub fn measure(mesh: &Mesh, rings: usize) -> Result<Self, String> {
        let joint = |name: &str| {
            mesh.joint(name)
                .ok_or_else(|| format!("the mesh has no `{name}` joint"))
        };
        let (atlas, neck) = (joint("head")?, joint("neck")?);
        let nose = mesh
            .skin
            .iter()
            .flatten()
            .map(|&at| mesh.points[at])
            .filter(|point| point[1] > neck[1])
            .max_by(|a, b| a[2].total_cmp(&b[2]))
            .ok_or("no skin above the neck")?;
        let line = minus(nose, atlas);
        let reach = dot(line, line).sqrt();
        let along_way = line.map(|value| value / reach);
        let up_way = [0.0, along_way[2], -along_way[1]];
        let frame = Frame {
            atlas,
            along: along_way,
            up: up_way,
        };
        // The jaw's underside: the lowest skin in front of the `head` joint
        // and above the neck, as far up the head's line as the nose is.
        let floor = mesh
            .skin
            .iter()
            .flatten()
            .map(|&at| mesh.points[at])
            .filter(|point| point[1] > neck[1])
            .map(|point| frame.of(point))
            .filter(|&(along, _)| along > reach * 0.5 && along <= reach)
            .map(|(_, up)| up)
            .fold(f64::MAX, f64::min);
        let across = |along: f64| head_ring(mesh, &frame, along, floor);
        let back = (0..BACK_STEPS)
            .map(|step| -reach * MOST_BEHIND * (1.0 - step as f64 / BACK_STEPS as f64))
            .find(|&along| across(along).is_some())
            .ok_or("no head behind the `head` joint")?;
        let length = reach - back;
        let rings = (0..rings)
            .filter_map(|at| {
                let along = back + length * LAST * at as f64 / (rings - 1).max(1) as f64;
                Some((along, across(along)?))
            })
            .collect();
        Ok(Self {
            length,
            pitch: (-along_way[1]).atan2(along_way[2]).to_degrees(),
            rings,
        })
    }
}

/// The head's own directions, from the `head` joint.
struct Frame {
    atlas: Point,
    along: Point,
    up: Point,
}

impl Frame {
    /// How far along the head's line a point is, and how far up from it.
    fn of(&self, point: Point) -> (f64, f64) {
        let offset = minus(point, self.atlas);
        (dot(offset, self.along), dot(offset, self.up))
    }
}

/// The ring a plane square to the head's line cuts `along` it: everything
/// crossing the middle line above `floor` (the jaw's underside) taken
/// together, upper and lower jaw alike, its half width only between belly
/// and the top's last `EARS`, which would take in the ears.
fn head_ring(mesh: &Mesh, frame: &Frame, along: f64, floor: f64) -> Option<Ring> {
    let through = [0, 1, 2].map(|way| frame.atlas[way] + frame.along[way] * along);
    let plane = Plane::facing(through, frame.along);
    let outlines: Vec<_> = cut(mesh, &plane)
        .into_iter()
        .filter(|outline| {
            outline
                .crossings(0.0)
                .last()
                .is_some_and(|&back| back > floor)
        })
        .collect();
    let crossings: Vec<f64> = outlines
        .iter()
        .flat_map(|outline| outline.crossings(0.0))
        .collect();
    let back = crossings.iter().copied().fold(f64::MIN, f64::max);
    let belly = crossings
        .iter()
        .copied()
        .fold(f64::MAX, f64::min)
        .max(floor);
    if outlines.is_empty() {
        return None;
    }
    let (low, high) = (belly, back - (back - belly) * EARS);
    let half_width = outlines
        .iter()
        .flat_map(|outline| &outline.lines)
        .filter_map(|line| widest(*line, low, high))
        .fold(0.0, f64::max);
    Some(Ring {
        half_width,
        belly,
        back,
    })
}

/// How far across the part of a line between `low` and `high` (how far up)
/// reaches, if any of it is between them.
fn widest([from, to]: [[f64; 2]; 2], low: f64, high: f64) -> Option<f64> {
    let (a, b) = if from[1] <= to[1] {
        (from, to)
    } else {
        (to, from)
    };
    if b[1] < low || a[1] > high {
        return None;
    }
    let at = |up: f64| {
        let share = if b[1] > a[1] {
            (up - a[1]) / (b[1] - a[1])
        } else {
            0.0
        };
        (a[0] + (b[0] - a[0]) * share).abs()
    };
    Some(at(a[1].max(low)).max(at(b[1].min(high))))
}
