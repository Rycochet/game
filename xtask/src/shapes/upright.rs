//! Measuring a body standing upright on two legs (MakeHuman's base person,
//! as the shared body's bipeds are): rings up the trunk from the crotch to
//! the neck's base, and how thick the arms and legs are along each bone.
//!
//! The trunk's rings are level cuts. Front and back are measured from a
//! line straight up through the middle of the hip joints, as the shared
//! body's spine runs (`shared::body::Spine`), so they show how the chest
//! and the seat stand out from it and the small of the back curves in.

use super::cut::{Outline, Plane, cut, dot, minus};
use super::mesh::{Mesh, Point};
use super::ring::{Ring, ring};

/// How many steps the search for the crotch takes, from the knees up to the
/// hip joints.
const CROTCH_STEPS: usize = 400;
/// How many cuts along each bone of a limb.
pub const LIMB_CUTS: usize = 5;

/// Everything measured, in the mesh's own units. Heights are from the
/// ground; front and back from the line up through the hip joints.
#[derive(Clone, Debug)]
pub struct Upright {
    /// From the ground to the top of the head: what our sizes go by.
    pub height: f64,
    /// The trunk's levels: the crotch, the hip joints, the shoulder joints
    /// and the neck's base (MakeHuman's `neck` joint).
    pub crotch: f64,
    pub hips: f64,
    pub shoulders: f64,
    pub neck: f64,
    /// Rings from the crotch to the neck's base, each with its height.
    /// `belly` is how far its front is in front of the line, `back` how far
    /// its back is behind it.
    pub trunk: Vec<(f64, Ring)>,
    /// The left leg's bones (hip to knee, knee to ankle) and the left arm's
    /// (shoulder to elbow, elbow to wrist): each bone's length and its half
    /// thickness across and front to back at `LIMB_CUTS` places along it.
    pub limbs: Vec<Bone>,
}

/// One bone of a limb, measured.
#[derive(Clone, Debug)]
pub struct Bone {
    pub name: &'static str,
    pub length: f64,
    /// Half thickness (across, front to back) at each cut, from the bone's
    /// start: the cuts are at 1/(LIMB_CUTS + 1) steps, leaving out the joints'
    /// very ends where one bone runs into the next or into the trunk.
    pub cuts: Vec<Option<(f64, f64)>>,
}

/// The bones measured: a name and the joints it runs between.
const BONES: [(&str, &str, &str); 4] = [
    ("thigh", "l-upper-leg", "l-knee"),
    ("shin", "l-knee", "l-ankle"),
    ("upper arm", "l-shoulder", "l-elbow"),
    ("forearm", "l-elbow", "l-hand"),
];

impl Upright {
    /// Measures a mesh standing upright and facing forwards, `rings` rings
    /// up its trunk.
    pub fn measure(mesh: &Mesh, rings: usize) -> Result<Self, String> {
        let joint = |name: &str| {
            mesh.joint(name)
                .ok_or_else(|| format!("the mesh has no `{name}` joint"))
        };
        let (hip, knee, shoulder, neck) = (
            joint("l-upper-leg")?,
            joint("l-knee")?,
            joint("l-shoulder")?,
            joint("neck")?,
        );
        let (low, high) = mesh.bounds();
        let ground = low[1];
        // The spine's line: up through the middle of the hip joints.
        let line = hip[2];
        let across = |y: f64| ring(mesh, &Plane::facing([0.0, y, line], [0.0, 1.0, 0.0]));
        // The crotch: the lowest level whose cut crosses the middle line.
        let crotch = (0..=CROTCH_STEPS)
            .map(|step| knee[1] + (hip[1] - knee[1]) * step as f64 / CROTCH_STEPS as f64)
            .find(|&y| across(y).is_some())
            .ok_or("no trunk between the knees and the hip joints")?;
        let trunk = (0..rings)
            .filter_map(|at| {
                let y = crotch + (neck[1] - crotch) * at as f64 / (rings - 1).max(1) as f64;
                // The plane measures towards the back, so the front is the
                // first crossing, negative in front of the line.
                let cut = across(y)?;
                Some((
                    y - ground,
                    Ring {
                        belly: -cut.belly,
                        ..cut
                    },
                ))
            })
            .collect();
        let limbs = BONES
            .iter()
            .map(|&(name, from, to)| Ok(bone(mesh, name, joint(from)?, joint(to)?)))
            .collect::<Result<_, String>>()?;
        Ok(Self {
            height: high[1] - ground,
            crotch: crotch - ground,
            hips: hip[1] - ground,
            shoulders: shoulder[1] - ground,
            neck: neck[1] - ground,
            trunk,
            limbs,
        })
    }
}

/// A bone measured by cuts square to it, from joint `from` to joint `to`.
fn bone(mesh: &Mesh, name: &'static str, from: Point, to: Point) -> Bone {
    let way = minus(to, from);
    let length = dot(way, way).sqrt();
    let along = way.map(|value| value / length);
    // Front to back as near the body's own as square to the bone allows.
    let forwards = [0.0, 0.0, 1.0];
    let back = unit(minus(
        along.map(|value| value * dot(forwards, along)),
        forwards,
    ));
    let across = cross(back, along);
    let cuts = (1..=LIMB_CUTS)
        .map(|at| {
            let share = at as f64 / (LIMB_CUTS + 1) as f64;
            let through = [0, 1, 2].map(|i| from[i] + way[i] * share);
            let plane = Plane {
                through,
                normal: along,
                across,
                back,
            };
            limb_outline(&cut(mesh, &plane)).map(|outline| {
                let (a, b) = (outline.span(0), outline.span(1));
                ((a.1 - a.0) / 2.0, (b.1 - b.0) / 2.0)
            })
        })
        .collect();
    Bone { name, length, cuts }
}

/// Of a cut's outlines, the limb's own: the smallest round the bone's line.
fn limb_outline(outlines: &[Outline]) -> Option<&Outline> {
    outlines
        .iter()
        .filter(|outline| {
            let (a, b) = (outline.span(0), outline.span(1));
            a.0 < 0.0 && a.1 > 0.0 && b.0 < 0.0 && b.1 > 0.0
        })
        .min_by(|x, y| size(x).total_cmp(&size(y)))
}

/// How big an outline's box is.
fn size(outline: &Outline) -> f64 {
    let (a, b) = (outline.span(0), outline.span(1));
    (a.1 - a.0) * (b.1 - b.0)
}

fn unit(v: Point) -> Point {
    let length = dot(v, v).sqrt();
    v.map(|value| value / length)
}

fn cross(a: Point, b: Point) -> Point {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
