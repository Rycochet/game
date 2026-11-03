//! Filling a floor with rooms, pillars and windows.

use super::super::grid::{self, STEPS};
use super::super::{SiteLevel, Stairs};
use super::{
    PILLAR_SPACING, Plan, ROOMS_PER_EXTRA_DOOR, Rect, SECRET_DOOR_PERCENT, Shape, Splitter, Spot,
    add_doors, add_entrances, add_extra_doors, join_corridors,
};
use crate::dungeon::{Cell, Level};
use crate::rng::{Rng, Seed};

impl Plan {
    /// Splits an outline into rooms round its stairs, with the ground outside
    /// and the doors out if it is on the ground.
    pub(super) fn fill_floor(
        &self,
        shape: Shape,
        mut rng: Rng,
        stairs: Vec<Stairs>,
        on_ground: bool,
    ) -> SiteLevel {
        // (A bigger basement can reach past the ground floor's level.)
        let (x0, y0) = (shape.x.min(0), shape.y.min(0));
        let x1 = (shape.x + shape.width).max(self.width);
        let y1 = (shape.y + shape.height).max(self.height);
        let mut level = Level::new(x0, y0, x1 - x0, y1 - y0);
        let required: Vec<(i64, i64)> = stairs.iter().map(|s| (s.x, s.y)).collect();

        if on_ground {
            for y in 0..self.height {
                for x in 0..self.width {
                    if !shape.contains(x, y) {
                        level.set(x, y, Cell::Ground);
                    }
                }
            }
        }

        let mut splitter = Splitter {
            level: &mut level,
            shape: &shape,
            required: &required,
            rng: &mut rng,
            rooms: 0,
            corridors: Vec::new(),
        };
        let all = Rect {
            x0: shape.x + 1,
            y0: shape.y + 1,
            x1: shape.x + shape.width - 1,
            y1: shape.y + shape.height - 1,
        };
        splitter.split(all, 0);
        let rooms = splitter.rooms;
        let corridors = std::mem::take(&mut splitter.corridors);
        join_corridors(&mut level, &shape, &corridors);

        // The way in: doors in the outer wall on the ground floor.
        let mut entrances = Vec::new();
        let mut entrance = None;
        if on_ground {
            entrances = add_entrances(&mut level, &shape, &mut rng);
            entrance = entrances.first().map(|&(_, outside)| outside);
        }
        let start = entrance
            .or_else(|| required.first().copied())
            .expect("a way in: an entrance or stairs");

        add_doors(&mut level, &shape, &mut rng, start);
        add_extra_doors(&mut level, &shape, &mut rng, rooms / ROOMS_PER_EXTRA_DOOR);
        let doors: Vec<(i64, i64)> = entrances.iter().map(|&(door, _)| door).collect();
        grid::hide_some_doors(
            &mut level,
            &mut rng,
            SECRET_DOOR_PERCENT,
            &[start],
            |x, y| doors.contains(&(x, y)),
        );

        SiteLevel {
            level,
            stairs,
            entrance,
            buildings: Vec::new(),
            fields: Vec::new(),
        }
    }
}

/// Where pillars stand round an outline's edge: going round it from the east,
/// each edge cell at least `PILLAR_SPACING` from those already chosen, so none
/// touch. In integers, like everything both sides work out.
pub(super) fn pillars(shape: &Shape) -> Vec<Spot> {
    // From the centre, in half cells so that it stays whole.
    let from_centre = |(x, y): Spot| {
        (
            2 * (x - shape.x) + 1 - shape.width,
            2 * (y - shape.y) + 1 - shape.height,
        )
    };
    let later_half = |(dx, dy): Spot| dy < 0 || (dy == 0 && dx < 0);
    let mut edge: Vec<Spot> = (shape.y..shape.y + shape.height)
        .flat_map(|y| (shape.x..shape.x + shape.width).map(move |x| (x, y)))
        .filter(|&(x, y)| shape.contains(x, y) && !shape.is_interior(x, y))
        .collect();
    edge.sort_by(|&a, &b| {
        let (a, b) = (from_centre(a), from_centre(b));
        later_half(a)
            .cmp(&later_half(b))
            .then((b.0 * a.1 - b.1 * a.0).cmp(&0))
    });
    let mut chosen: Vec<Spot> = Vec::new();
    for (x, y) in edge {
        let clear = chosen
            .iter()
            .all(|&(cx, cy)| (cx - x).abs().max((cy - y).abs()) >= PILLAR_SPACING);
        if clear {
            chosen.push((x, y));
        }
    }
    chosen
}

/// Puts windows in a floor's outside walls: a straight wall between the inside
/// and the outside of its outline, with wall on either side along it and none
/// at a diagonal, which would look cramped (the solid wall beyond an upper floor's
/// outline doesn't count). After any ruin, as what is left decides where they fit. By hashes, not the floor's
/// sequence: where they go changes nothing else.
pub(super) fn add_windows(level: &mut Level, shape: &Shape, seed: Seed, percent: u32, floor: i64) {
    if percent == 0 {
        return;
    }
    let hashes = seed.derive("building windows");
    let (min_x, min_y, width, height) = level.bounds();
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            let fits = STEPS.iter().any(|&(dx, dy)| {
                let beside = |sign: i64| {
                    let (bx, by) = (x + dy * sign, y + dx * sign);
                    level.cell(bx, by) == Cell::Wall && shape.contains(bx, by)
                };
                !shape.contains(x + dx, y + dy)
                    && level.is_walkable(x - dx, y - dy)
                    && beside(1)
                    && beside(-1)
            });
            let clear = [(1, 1), (1, -1), (-1, 1), (-1, -1)]
                .iter()
                .all(|&(dx, dy)| {
                    !(level.cell(x + dx, y + dy).is_wall() && shape.contains(x + dx, y + dy))
                });
            if level.cell(x, y) == Cell::Wall
                && shape.contains(x, y)
                && fits
                && clear
                && hashes.at(&[floor, x, y]) % 100 < u64::from(percent)
            {
                level.set(x, y, Cell::Window);
            }
        }
    }
}
