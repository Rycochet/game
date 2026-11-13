//! A building's plan: its floors' sizes and shapes, worked out once from the seed.

use super::super::grid::{chance, roll};
use super::super::ruin;
use super::super::{Exit, SiteLevel, StairsDirection};
use super::{
    BORDER, BuildingOptions, CELLS_PER_EXTRA_STAIRS, EXTRA_STAIRS_PERCENT, MIN_FLOOR_SIZE,
    MIN_STAIRS_GAP, MIN_TAPERED_FLOOR, MOST_EXTRA_STAIRS, Outline, SAME_SIZE_PERCENT,
    SMALL_TOP_PERCENT, Shape, Walled, add_windows,
};
use super::{Castle, CastleSize};
use crate::rng::Seed;

/// A building's plan: its floors' outlines and the stairs between them.
#[derive(Clone, Debug)]
pub struct Plan {
    pub(super) seed: Seed,
    /// The level's size: the ground floor and the border round it.
    pub(super) width: i64,
    pub(super) height: i64,
    pub(super) floors: Vec<Shape>,
    /// `stairs[n]` are the staircases joining floor `n` to floor `n + 1`, at
    /// least one each.
    pub(super) stairs: Vec<Vec<(i64, i64)>>,
    /// Where the stairs down to the basement are, if it has one. The basement
    /// is the last level, and is never ruined: nothing above it comes down
    /// that far.
    pub(super) basement: Option<(i64, i64)>,
    /// Its outline, which reaches at least as far as the ground floor's.
    pub(super) basement_shape: Shape,
    pub(super) windows_percent: u32,
    pub(super) ground_windows: bool,
    pub(super) stilts: bool,
    /// A ruined building's floors, all of them, from the ground up; empty if
    /// it isn't ruined.
    pub(super) ruined: Vec<SiteLevel>,
    /// A wall round the ground floor, if it has one.
    pub(super) wall: Option<Walled>,
    /// The castle round it, if it is a keep.
    pub(super) castle: Option<Castle>,
}

impl Plan {
    pub fn new(options: &BuildingOptions, seed: Seed) -> Self {
        let mut rng = seed.derive("building").rng();
        let outline = options.outlines[rng.range_u64(0, options.outlines.len() as u64) as usize];
        // A keep is what fits inside its castle.
        let castle_size = (options.castle.as_ref()).map(|castle| CastleSize::roll(castle, seed));
        let width = match &castle_size {
            Some(size) => size.keep().0,
            None => roll(&mut rng, options.width.0, options.width.1),
        };
        let height = if options.square || outline == Outline::Circle {
            width
        } else {
            match &castle_size {
                Some(size) => size.keep().1,
                None => roll(&mut rng, options.height.0, options.height.1),
            }
        };
        let corner = roll(&mut rng, 2, (width.min(height) / 4).max(2));
        let ground = Shape {
            outline,
            x: BORDER,
            y: BORDER,
            width,
            height,
            corner,
        };

        // Bigger buildings may have more floors.
        let least = options.min_floors.max(1) as i64;
        let most = (width.min(height) / options.size_per_floor.max(1))
            .clamp(1, options.max_floors.max(1) as i64)
            .max(least);
        let wanted = roll(&mut rng, least, most);
        let mut floors = vec![ground];
        while options.taper.is_some() || (floors.len() as i64) < wanted {
            let below = floors[floors.len() - 1];
            let smallest = |floor: Shape| floor.width.min(floor.height);
            let above = if let Some((fewest, greatest)) = options.taper {
                let by = roll(&mut rng, fewest, greatest);
                if smallest(below.shrunk(by)) >= MIN_TAPERED_FLOOR {
                    below.shrunk(by)
                } else {
                    // The bigger step may not fit where the smaller one does.
                    below.shrunk(fewest)
                }
            } else {
                // Half the time the same size; otherwise a cell smaller on each
                // side for every 10 in the roll above 50.
                let roll = rng.range_u64(0, 100);
                let by = if options.same_size || roll < SAME_SIZE_PERCENT {
                    0
                } else {
                    ((roll - SAME_SIZE_PERCENT) / 10 + 1) as i64
                };
                // Leaving room for the floors it must have.
                let needed = least - floors.len() as i64;
                let by = if needed > 0 {
                    by.min((smallest(below) - MIN_FLOOR_SIZE) / (2 * needed))
                } else {
                    by
                };
                below.shrunk(by)
            };
            let too_small = if options.taper.is_some() {
                MIN_TAPERED_FLOOR
            } else {
                MIN_FLOOR_SIZE
            };
            if smallest(above) < too_small {
                break;
            }
            floors.push(above);
        }
        if options.same_size && floors.len() as i64 > least && chance(&mut rng, SMALL_TOP_PERCENT) {
            let top = floors.len() - 1;
            floors[top] = floors[top].shrunk(floors[top].width.min(floors[top].height) / 4);
        }

        // Stairs: on cells well inside both floors (not next to an outer
        // wall), never next to those on the floor above. From the top down,
        // as small floors have the fewest places for them.
        let mut stairs: Vec<Vec<(i64, i64)>> = Vec::new();
        for pair in (0..floors.len() - 1).rev() {
            let above = floors[pair + 1];
            let clear_of = |spots: &[(i64, i64)], gap: i64, x: i64, y: i64| {
                spots
                    .iter()
                    .all(|&(sx, sy)| (sx - x).abs() > gap || (sy - y).abs() > gap)
            };
            let upper = stairs.last().cloned().unwrap_or_default();
            let inside = |x: i64, y: i64| {
                (-1..=1).all(|dy| (-1..=1).all(|dx| above.is_interior(x + dx, y + dy)))
                    && clear_of(&upper, 2, x, y)
            };
            let cells = above.width * above.height;
            let random = (0..100).find_map(|_| {
                let x = roll(&mut rng, above.x, above.x + above.width - 1);
                let y = roll(&mut rng, above.y, above.y + above.height - 1);
                inside(x, y).then_some((x, y))
            });
            let found = random.or_else(|| {
                (above.y..above.y + above.height)
                    .flat_map(|y| (above.x..above.x + above.width).map(move |x| (x, y)))
                    .find(|&(x, y)| inside(x, y))
            });
            let Some(first) = found else {
                // No way up from here: what is above goes.
                floors.truncate(pair + 1);
                stairs.clear();
                continue;
            };
            let mut here = vec![first];
            for _ in 0..(cells / CELLS_PER_EXTRA_STAIRS).min(MOST_EXTRA_STAIRS) {
                if options.stilts || !chance(&mut rng, EXTRA_STAIRS_PERCENT) {
                    continue;
                }
                let extra = (0..100).find_map(|_| {
                    let x = roll(&mut rng, above.x, above.x + above.width - 1);
                    let y = roll(&mut rng, above.y, above.y + above.height - 1);
                    (inside(x, y) && clear_of(&here, MIN_STAIRS_GAP, x, y)).then_some((x, y))
                });
                here.extend(extra);
            }
            stairs.push(here);
        }
        stairs.reverse();

        // (No draw at all for buildings that can't have one.)
        let basement = if options.basement_percent > 0 && chance(&mut rng, options.basement_percent)
        {
            let shape = floors[0];
            let above = stairs.first().cloned().unwrap_or_default();
            (0..100).find_map(|_| {
                let x = roll(&mut rng, shape.x, shape.x + shape.width - 1);
                let y = roll(&mut rng, shape.y, shape.y + shape.height - 1);
                let clear = above
                    .iter()
                    .all(|&(sx, sy)| (sx - x).abs() > 2 || (sy - y).abs() > 2);
                ((-1..=1).all(|dy| (-1..=1).all(|dx| shape.is_interior(x + dx, y + dy))) && clear)
                    .then_some((x, y))
            })
        } else {
            None
        };

        let basement_shape = match options.basement_size.filter(|_| basement.is_some()) {
            Some((wide, tall)) => {
                let ground = floors[0];
                let width = roll(&mut rng, wide.0.max(ground.width), wide.1.max(ground.width));
                let height = roll(
                    &mut rng,
                    tall.0.max(ground.height),
                    tall.1.max(ground.height),
                );
                Shape {
                    outline: Outline::Rectangle,
                    x: ground.x - roll(&mut rng, 0, width - ground.width),
                    y: ground.y - roll(&mut rng, 0, height - ground.height),
                    width,
                    height,
                    corner: ground.corner,
                }
            }
            None => floors[0],
        };

        let wall = (options.wall.as_ref())
            .and_then(|wall| Walled::plan(wall, seed, &floors[0], options.ruin));
        let mut plan = Self {
            seed,
            width: width + 2 * BORDER,
            height: height + 2 * BORDER,
            floors,
            stairs,
            basement,
            basement_shape,
            windows_percent: options.windows_percent,
            ground_windows: options.ground_windows,
            stilts: options.stilts,
            ruined: Vec::new(),
            wall,
            castle: None,
        };
        if options.ruin > 0 {
            let storeys = plan.floors.len() as u32;
            let intact = (0..storeys).map(|n| plan.intact_level(n)).collect();
            plan.ruined = ruin::ruin_floors(
                seed,
                options.ruin,
                intact,
                &plan.floors,
                &plan.stairs,
                basement,
            );
            // Floors may have been lost: the basement is still the last level.
            let last = plan.ruined.len() as u32;
            for stairs in &mut plan.ruined[0].stairs {
                if stairs.direction == StairsDirection::Down {
                    stairs.to = Exit::Level(last);
                }
            }
            for (index, floor) in plan.ruined.iter_mut().enumerate() {
                if index > 0 || plan.ground_windows {
                    add_windows(
                        &mut floor.level,
                        &plan.floors[index],
                        seed,
                        plan.windows_percent,
                        index as i64,
                    );
                }
            }
        }
        if let (Some(castle), Some(size)) = (&options.castle, &castle_size) {
            let storeys = plan.storeys();
            let keep = plan.floors[0];
            plan.castle = Some(Castle::plan(
                castle,
                size,
                seed,
                &keep,
                (options.ruin, storeys),
            ));
        }
        plan
    }
}
