//! Generating one cave level, and opening skylights in it.

use super::super::grid::{self, Reach, chance, roll};
use super::super::{Exit, SiteLevel, Stairs, StairsDirection};
use super::{
    BORDER, CELLS_PER_SKYLIGHT, Plan, SECRET_DOOR_PERCENT, SKYLIGHT_CLEARANCE, add_doors, add_pool,
    dig_cave, dig_mouths, furthest_cave, join_caves, nearest_reached, place_caves, wind,
};
use crate::dungeon::{Cell, Level, RoomId};
use crate::rng::Seed;

impl Plan {
    /// The level before any skylights are opened in it.
    pub(in crate::site) fn without_skylights(&self, index: u32) -> SiteLevel {
        let mut rng = Seed::new(self.seed.derive("cave level").at(&[index as i64])).rng();
        let width = roll(&mut rng, self.options.size.0, self.options.size.1);
        let height = roll(&mut rng, self.options.size.0, self.options.size.1);
        let mut level = Level::new(0, 0, width, height);
        let opening = index == 0 && self.opening;
        // Keep caves off the edges (and clear of any mouth's ground).
        let margin = if opening { BORDER + 2 } else { 2 };

        let caves = place_caves(&mut rng, width, height, margin);
        for (i, cave) in caves.iter().enumerate() {
            dig_cave(&mut level, &mut rng, cave, RoomId(i as u16));
        }
        // Passages stay off the edges (and clear of any mouth's ground).
        let inner = if opening {
            (1 + BORDER, 1, width - 2 - BORDER, height - 2 - BORDER)
        } else {
            (1, 1, width - 2, height - 2)
        };
        join_caves(&mut level, &mut rng, &caves, inner);

        // The way in.
        let mut stairs = Vec::new();
        let mut entrance = None;
        let arrival = if opening {
            let mouth = dig_mouths(&mut level, &mut rng, &caves, inner);
            entrance = Some(mouth);
            mouth
        } else {
            let cave = caves[rng.range_u64(0, caves.len() as u64) as usize];
            let to = if index == 0 {
                Exit::WorldMap
            } else {
                Exit::Level(index - 1)
            };
            stairs.push(Stairs {
                x: cave.x,
                y: cave.y,
                direction: StairsDirection::Up,
                to,
            });
            if index == 0 {
                entrance = Some((cave.x, cave.y));
            }
            (cave.x, cave.y)
        };

        // Anything not reached yet gets a passage to what is.
        loop {
            let mut reach = Reach::new(&level);
            reach.spread(&level, &[arrival]);
            let Some(from) = reach.first_unreached(&level) else {
                break;
            };
            let to = nearest_reached(&level, &reach, from);
            wind(&mut level, &mut rng, from, to, inner);
            // If the passage couldn't get there, dig straight across.
            let mut reach = Reach::new(&level);
            reach.spread(&level, &[arrival]);
            if !reach.is(from.0, from.1) {
                let edge = (width, height);
                let dug = grid::dig_to_unreached(&mut level, &reach, |x, y| {
                    x > 0 && y > 0 && x < edge.0 - 1 && y < edge.1 - 1
                });
                if !dug {
                    break;
                }
            }
        }

        // Stairs down in the cave furthest from the way in.
        if index + 1 < self.levels {
            let (x, y) = furthest_cave(&level, &caves, arrival);
            stairs.push(Stairs {
                x,
                y,
                direction: StairsDirection::Down,
                to: Exit::Level(index + 1),
            });
        }

        let keep: Vec<(i64, i64)> = stairs.iter().map(|s| (s.x, s.y)).chain([arrival]).collect();
        if self.options.doors {
            add_doors(&mut level, &mut rng, &keep);
        }
        if chance(&mut rng, self.options.water_percent) {
            for _ in 0..roll(&mut rng, 1, 3) {
                add_pool(&mut level, &mut rng, &caves, Cell::Water, &keep, arrival);
            }
        }
        if chance(&mut rng, self.options.lava_percent) {
            add_pool(&mut level, &mut rng, &caves, Cell::Lava, &keep, arrival);
        }
        if self.options.doors {
            grid::hide_some_doors(
                &mut level,
                &mut rng,
                SECRET_DOOR_PERCENT,
                &[arrival],
                |_, _| false,
            );
        }

        SiteLevel {
            level,
            stairs,
            entrance,
            buildings: Vec::new(),
            fields: Vec::new(),
        }
    }
}

/// Very rarely, a floor cell is open to the sky: outside ground, away from
/// liquid and stairs and off the level's edge (so it is no way out). Chosen by
/// a hash of its position, so nothing else about the level changes.
pub(super) fn open_skylights(level: &mut Level, seed: Seed, stairs: &[Stairs]) {
    let (min_x, min_y, width, height) = level.bounds();
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            if level.cell(x, y) != Cell::Floor
                || !seed.at(&[x, y]).is_multiple_of(CELLS_PER_SKYLIGHT)
            {
                continue;
            }
            let reach = -SKYLIGHT_CLEARANCE..=SKYLIGHT_CLEARANCE;
            let clear = reach.clone().all(|dy| {
                reach.clone().all(|dx| {
                    let (nx, ny) = (x + dx, y + dy);
                    let cell = level.cell(nx, ny);
                    level.contains(nx, ny)
                        && !cell.is_liquid()
                        && !cell.is_outside()
                        && !stairs.iter().any(|s| (s.x, s.y) == (nx, ny))
                })
            });
            if clear {
                level.set(x, y, Cell::Ground);
                level.set_room(x, y, None);
            }
        }
    }
}
