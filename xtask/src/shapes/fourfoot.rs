//! Measuring a beast standing on four legs (a cow, a dog, a horse), as the
//! shared body describes one (`shared::body::Fourfoot`): the height at the
//! withers, the hip and shoulder joints along the back, rings along the trunk
//! and the legs at levels up from the ground.
//!
//! MakeHuman's animals are its person reshaped, so their joints keep a
//! person's names: the forelegs are the arms (`shoulder`, `hand`) and the
//! hind legs the legs (`upper-leg`).

use super::cut::Plane;
use super::leg::{LegCut, legs};
use super::level::stand_level;
use super::mesh::{Mesh, Point};
use super::ring::{Ring, ring};

/// How many steps the search for the rump takes, from the body's rear end
/// (which may be a hind leg's hock) forwards to the hip joints.
const RUMP_STEPS: usize = 400;
/// The legs are cut at levels this share of the withers' height apart.
const LEG_STEP: f64 = 0.05;

/// Everything measured, in the mesh's own units. "Along" is forwards from the
/// hip joints; heights are from the ground.
#[derive(Clone, Debug)]
pub struct Fourfoot {
    /// The top of the back over the forefeet: what our animals' sizes go by.
    pub withers: f64,
    /// The hip joints' height.
    pub hips: f64,
    /// How far along the shoulder joints are: the body's length.
    pub shoulders: f64,
    /// Rings from the rump to the neck's base, each with how far along it is.
    /// Belly and back are heights from the ground.
    pub trunk: Vec<(f64, Ring)>,
    /// The left foreleg and hind leg at each height they stand clear of the
    /// trunk, lowest first; a leg's middle is along from the hip joints.
    pub legs: Vec<(f64, Option<LegCut>, Option<LegCut>)>,
}

impl Fourfoot {
    /// Measures a mesh facing forwards, with `rings` rings along its trunk.
    pub fn measure(mut mesh: Mesh, rings: usize) -> Result<Self, String> {
        let (hip, shoulder) = (
            joint_of(&mesh, "l-upper-leg")?,
            joint_of(&mesh, "l-shoulder")?,
        );
        let halfway = (hip[2] + shoulder[2]) / 2.0;
        stand_level(&mut mesh, halfway);
        let mesh = &mesh;
        // Levelling slid the joints too.
        let (hip, shoulder, forefoot) = (
            joint_of(mesh, "l-upper-leg")?,
            joint_of(mesh, "l-shoulder")?,
            joint_of(mesh, "l-hand")?,
        );
        let (low, _) = mesh.bounds();
        let ground = low[1];
        // A ring across the body at a place along it, measured from the ground.
        let across = |z: f64| ring(mesh, &Plane::facing([0.0, ground, z], [0.0, 0.0, 1.0]));
        let withers = across(forefoot[2]).ok_or("no body over the forefeet")?.back;

        let rump = (0..RUMP_STEPS)
            .map(|step| low[2] + (hip[2] - low[2]) * step as f64 / RUMP_STEPS as f64)
            .find(|&z| across(z).is_some())
            .ok_or("no body behind the hip joints")?;
        // The neck's base as the shared body has it: a third of the chest's
        // half depth in front of the shoulder joints.
        let chest = across(shoulder[2]).ok_or("no body at the shoulder joints")?;
        let neck_base = shoulder[2] + (chest.back - chest.belly) / 6.0;
        let trunk = (0..rings)
            .filter_map(|at| {
                let z = rump + (neck_base - rump) * at as f64 / (rings - 1).max(1) as f64;
                Some((z - hip[2], across(z)?))
            })
            .collect();

        let from_hips = |leg: LegCut| LegCut {
            middle: (leg.middle.0, leg.middle.1 - hip[2]),
            ..leg
        };
        let legs = (1..)
            .map(|step| step as f64 * LEG_STEP * withers)
            .take_while(|&height| height < hip[1] - ground)
            .filter_map(|height| {
                let (fore, hind) = legs(mesh, ground + height, halfway);
                (fore.is_some() || hind.is_some())
                    .then(|| (height, fore.map(from_hips), hind.map(from_hips)))
            })
            .collect();

        Ok(Self {
            withers,
            hips: hip[1] - ground,
            shoulders: shoulder[2] - hip[2],
            trunk,
            legs,
        })
    }
}

/// Where a joint is, or why not.
fn joint_of(mesh: &Mesh, name: &str) -> Result<Point, String> {
    let missing = || format!("the mesh has no `{name}` joint");
    mesh.joint(name).ok_or_else(missing)
}
