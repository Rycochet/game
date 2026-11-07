//! Where each known skill goes on the page: a radial tree round the character,
//! one ring per level. The known top levels go clockwise from the top in the
//! tree's order (Combat first), sharing the circle by the square root of how many
//! known skills end beneath each, so a small branch still has room beside a big one;
//! below them each known node gets a share of its parent's slice by how many end
//! beneath it, never wider than its level allows (`MAX_SLICE`), so every branch
//! points outwards, even a lone one. Only known skills are placed.

use bevy::math::Vec2;
use shared::skill::{SkillId, Skills};

/// How far out each level's ring is, at zoom 1, in interface pixels: the
/// character, then the top levels and each level below.
pub(super) const RINGS: [f32; 5] = [0.0, 160.0, 300.0, 440.0, 580.0];

/// How wide, in degrees, the children of a node at each level may spread round its
/// line. A child stays further out along its parent's line while the angle between
/// them is under the arccos of the two rings' ratio (58°, 47° and 41° between the
/// levels), so these keep half of each well inside that.
const MAX_SLICE: [f32; 5] = [360.0, 90.0, 72.0, 60.0, 60.0];

/// One known skill's place.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Placed {
    pub id: SkillId,
    /// From the character, at zoom 1; y is down, as on the screen.
    pub at: Vec2,
    /// Its parent's place, or the character's (the middle) for a top level.
    pub from: Vec2,
}

/// Every known skill's place, parents before their children.
pub(super) fn layout(skills: &Skills) -> Vec<Placed> {
    let tops: Vec<_> = SkillId::top_levels()
        .iter()
        .copied()
        .filter(|&top| skills.knows(top))
        .collect();
    let weight = |top| (ends_under(skills, top) as f32).sqrt();
    let total: f32 = tops.iter().map(|&top| weight(top)).sum();
    let mut placed = Vec::new();
    let mut start = 0.0;
    for top in tops {
        let share = std::f32::consts::TAU * weight(top) / total;
        place(skills, top, (start, start + share), Vec2::ZERO, &mut placed);
        start += share;
    }
    placed
}

/// Places `id` in the middle of its slice of the circle (angles clockwise from
/// the top), then its known children in shares of the slice.
fn place(
    skills: &Skills,
    id: SkillId,
    (from_angle, to_angle): (f32, f32),
    parent: Vec2,
    placed: &mut Vec<Placed>,
) {
    let angle = (from_angle + to_angle) / 2.0;
    let radius = RINGS[usize::from(id.depth()).min(RINGS.len() - 1)];
    let at = Vec2::new(angle.sin(), -angle.cos()) * radius;
    placed.push(Placed {
        id,
        at,
        from: parent,
    });
    // Its children spread no wider than this level allows, round its own line, so
    // each stays further out along it (a lone top level has the whole circle).
    let half = (to_angle - from_angle).min(MAX_SLICE[usize::from(id.depth())].to_radians()) / 2.0;
    let (from_angle, to_angle) = (angle - half, angle + half);
    let known: Vec<_> = known_children(skills, id).collect();
    let total: u32 = known.iter().map(|&child| ends_under(skills, child)).sum();
    let mut start = from_angle;
    for child in known {
        let share = (to_angle - from_angle) * ends_under(skills, child) as f32 / total as f32;
        place(skills, child, (start, start + share), at, placed);
        start += share;
    }
}

fn known_children(skills: &Skills, id: SkillId) -> impl Iterator<Item = SkillId> + '_ {
    id.children()
        .iter()
        .copied()
        .filter(|&child| skills.knows(child))
}

/// How many known skills end under this one (itself, if nothing under it is known).
fn ends_under(skills: &Skills, id: SkillId) -> u32 {
    let below: u32 = known_children(skills, id)
        .map(|child| ends_under(skills, child))
        .sum();
    below.max(1)
}

#[cfg(test)]
mod tests;
