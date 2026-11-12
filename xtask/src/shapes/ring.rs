//! One ring round a body: the outline a plane cuts from it, as three figures.

use super::cut::{Outline, Plane, cut};
use super::mesh::Mesh;

/// A ring's figures, in the mesh's own units, measured on the plane from the
/// point it was cut through.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ring {
    /// From the body's middle line out to its widest, between belly and back.
    pub half_width: f64,
    /// How far towards the back the front (a beast's underside) is, on the
    /// body's middle line.
    pub belly: f64,
    /// How far towards the back the back is, on the middle line.
    pub back: f64,
}

/// The ring a plane cuts from the body: of the outlines crossing the body's
/// middle line (so never a leg beside it), the one that is deepest there.
pub fn ring(mesh: &Mesh, plane: &Plane) -> Option<Ring> {
    let outlines = cut(mesh, plane);
    let (outline, belly, back) = outlines
        .iter()
        .filter_map(|outline| {
            let crossings = outline.crossings(0.0);
            Some((outline, *crossings.first()?, *crossings.last()?))
        })
        .max_by(|a, b| (a.2 - a.1).total_cmp(&(b.2 - b.1)))?;
    Some(Ring {
        half_width: half_width(outline, belly, back),
        belly,
        back,
    })
}

/// The furthest the outline reaches from the middle line, counting only what
/// lies between belly and back: where a leg joins the trunk, the leg below
/// the belly is left out.
fn half_width(outline: &Outline, belly: f64, back: f64) -> f64 {
    outline
        .lines
        .iter()
        .flatten()
        .filter(|place| (belly..=back).contains(&place[1]))
        .map(|place| place[0].abs())
        .fold(0.0, f64::max)
}
