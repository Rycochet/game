//! Working out a creature's attributes from its kind and its triangles.

use super::{Attributes, Build, Row, WEIGHT_TOTAL};
use crate::creature::CreatureKind;

/// How far a dot can move an attribute from its kind's median, as a percentage
/// of the median per third of the triangle: a corner gives that attribute
/// 100 + 2 × this and the other two 100 − this. *To balance.*
pub const SPREAD: i64 = 30;

/// How much a row's extra point raises its three attributes, as a percentage: a
/// row at the top with 2 points has them this much higher. *To balance.*
pub const POINT_WEIGHT: i64 = 25;

impl Attributes {
    /// What a creature of this kind starts with, from where its dots sit and how
    /// its triangles are turned, with its kind's bonus on top. `build` must be
    /// valid (`Build::is_valid`).
    pub fn for_kind(kind: CreatureKind, build: &Build) -> Self {
        let def = kind.def();
        let points = build.points();
        let mut attributes = Self::default();
        for row in Row::ALL {
            let weights = build.dot(row).0;
            let raised = 1000 + (points[row as usize] as i64 - 1000) * POINT_WEIGHT / 100;
            for (which, weight) in row.attributes().into_iter().zip(weights) {
                // From −1 (at the far side) through 0 (the centre) to 2 (its corner),
                // in thousandths.
                let lean = 3 * weight as i64 - WEIGHT_TOTAL as i64;
                let median = def.attributes.get(which) as i64;
                // Rounded once at the end, so a dot in the middle gives the median.
                const PARTS: i64 = 100_000 * 1000;
                let shaped = median * (100_000 + lean * SPREAD) * raised;
                let value = (shaped + PARTS / 2) / PARTS + def.bonus.get(which) as i64;
                attributes.set(which, value.clamp(1, u16::MAX as i64) as u16);
            }
        }
        attributes
    }
}

#[cfg(test)]
mod tests;
