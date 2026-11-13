//! Where a ruin's wall has fallen: runs of a few cells, more the more ruined.

use super::WallLook;
use crate::rng::Seed;
use crate::site::grid::roll;

/// Cells of ring for each gap at ruin 1: a gap every this many, and one more
/// for each step of ruin.
const RING_PER_GAP: i64 = 40;

/// The longest a gap runs.
const LONGEST_GAP: i64 = 3;

/// The cells of `wall` fallen at `ruin` (0 to 5): none at 0. Never a gate's or
/// a tower's. From its own seed, so nothing else changes.
pub(in crate::site) fn ruin_gaps(wall: &WallLook, seed: Seed, ruin: u8) -> Vec<(i64, i64)> {
    if ruin == 0 {
        return Vec::new();
    }
    let ring = ring(wall);
    let length = ring.len() as i64;
    let mut rng = seed.derive("wall ruin").rng();
    let count = (i64::from(ruin) * length / RING_PER_GAP).max(1);
    let mut gaps = Vec::new();
    for _ in 0..count {
        let start = roll(&mut rng, 0, length - 1);
        let run = roll(&mut rng, 1, LONGEST_GAP.min(i64::from(ruin)));
        for step in 0..run {
            let (x, y) = ring[((start + step) % length) as usize];
            if wall.has(x, y) && !wall.is_gate(x, y) && !gaps.contains(&(x, y)) {
                gaps.push((x, y));
            }
        }
    }
    gaps
}

/// The ring's cells in order, clockwise from the north-west corner.
fn ring(wall: &WallLook) -> Vec<(i64, i64)> {
    let rect = wall.rect;
    let (x0, y0) = (rect.x, rect.y);
    let (x1, y1) = (x0 + rect.width - 1, y0 + rect.height - 1);
    let north = (x0..x1).map(|x| (x, y0));
    let east = (y0..y1).map(|y| (x1, y));
    let south = (x0 + 1..=x1).rev().map(|x| (x, y1));
    let west = (y0 + 1..=y1).rev().map(|y| (x0, y));
    north.chain(east).chain(south).chain(west).collect()
}
