//! A quest's own dungeon (owner): made from the quest's seed with the cave
//! generator, one copy for each character, entered through a cellar under the
//! giver's building ("Enter Cellar"). Sized by the giver's town and the
//! quest's kind: a hamlet's cellar is one small level, a city's up to three.
//! The server keeps the copies; this is what each is made from.

use serde::{Deserialize, Serialize};

use super::plan::Plan;
use super::world::size;
use super::{Hook, Step, Target};
use crate::creature::CreatureKind;
use crate::dungeon::Level;
use crate::mob::{Spawn, spawn_at};
use crate::poi::{Poi, PoiKind};
use crate::rng::{Rng, Seed};
use crate::site::{CaveEntrance, CaveOptions, Generator};
use crate::story::Goal;

/// The poi kind a quest's dungeon is made as: a dug cellar of barrels, with
/// doors, entered down stairs (its mobs and looks are that kind's).
pub const DUNGEON_KIND: PoiKind = PoiKind::Barrels;

/// Who may live in a cellar, and how likely each is: small folk, nothing
/// too big for its passages. *To balance.*
const CELLAR_CREATURES: [(CreatureKind, u64); 4] = [
    (CreatureKind::Kobold, 45),
    (CreatureKind::Goblin, 30),
    (CreatureKind::Lizardfolk, 15),
    (CreatureKind::Orc, 10),
];

/// How a quest's dungeon is entered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DungeonWay {
    /// Down from the giver's own building, from their talk panel.
    Cellar,
    // New ways go at the end: messages carry one by its place.
}

/// What a quest's dungeon is made from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuestDungeon {
    pub way: DungeonWay,
    /// A made-up poi standing for it: `DUNGEON_KIND` at the giver's town, with
    /// the dungeon's own seed (never the town's).
    pub poi: Poi,
    pub levels: u32,
    /// Each level's width and height are picked from this range.
    pub size: (i64, i64),
    /// Who lives there (the quest's kills are of them), and how many on each
    /// level.
    pub creatures: CreatureKind,
    pub per_level: u8,
}

impl QuestDungeon {
    /// The cave generator's options for it.
    pub fn generator(&self) -> Generator {
        Generator::Cave(CaveOptions {
            size: self.size,
            levels: (self.levels, self.levels),
            entrance: CaveEntrance::Hole,
            doors: true,
            water_percent: 10,
            lava_percent: 0,
        })
    }

    /// Who is on level `index` (`stairs`: kept away from), the same every
    /// time: `per_level` of its creatures on walkable cells, as far from the
    /// stairs as there is room for, one to a cell.
    pub fn spawns(&self, index: u32, level: &Level, stairs: &[(i64, i64)]) -> Vec<Spawn> {
        let seed = self.poi.seed.derive("dungeon mobs");
        let seed = Seed::new(seed.at(&[i64::from(index)]));
        let (min_x, min_y, width, height) = level.bounds();
        let mut cells: Vec<(u8, u64, (i64, i64))> = Vec::new();
        for y in min_y..min_y + height {
            for x in min_x..min_x + width {
                let apart = (stairs.iter())
                    .map(|s| (s.0 - x).abs().max((s.1 - y).abs()))
                    .min()
                    .unwrap_or(i64::MAX);
                if level.is_walkable(x, y) && apart > 0 {
                    // Far cells first, then nearer, then beside the stairs.
                    let nearness = match apart {
                        6.. => 0,
                        3..6 => 1,
                        _ => 2,
                    };
                    cells.push((nearness, seed.at(&[x, y]), (x, y)));
                }
            }
        }
        cells.sort_unstable();
        (cells
            .into_iter()
            .take(usize::from(self.per_level))
            .enumerate())
        .map(|(number, (_, _, at))| spawn_at(seed, self.creatures, number as u32, at))
        .collect()
    }
}

/// Percent of normal quests, and of dailies, that are a cellar of the
/// giver's own (owner: commoner for dailies). *To balance.*
pub(super) const NORMAL_PERCENT: u64 = 10;
pub(super) const DAILY_PERCENT: u64 = 35;

/// The plan for a cellar quest of `town`'s: clear its creatures, and on more
/// than one level reach the bottom too, alongside.
pub(super) fn plan_cellar(town: &Poi, daily: bool, rng: &mut Rng) -> Plan {
    let bigger = size(town.kind).unwrap_or(0) as u64;
    let deepest = if daily { 1 } else { bigger.max(1) };
    let levels = rng.range_u64(1, deepest + 1) as u32;
    let size = (28 + 4 * bigger as i64, 36 + 6 * bigger as i64);
    let creatures = rng.pick_weighted(&CELLAR_CREATURES);
    let count = rng.range_u64(3 + bigger, 6 + 2 * bigger) as u32;
    // One to spare on each level.
    let per_level = (count.div_ceil(levels) + 1) as u8;
    let poi = Poi {
        kind: DUNGEON_KIND,
        seed: Seed::new(rng.next_u64()),
        ..*town
    };
    let dungeon = QuestDungeon {
        way: DungeonWay::Cellar,
        poi,
        levels,
        size,
        creatures,
        per_level,
    };
    let step = |goal, hook, count| Step {
        goal,
        target: Target::Dungeon {
            chunk: town.chunk,
            hook,
            way: DungeonWay::Cellar,
        },
        count,
        after: Vec::new(),
        alternatives: Vec::new(),
    };
    let mut steps = vec![step(Goal::Slay, Hook::Kills(creatures), Some(count))];
    if levels > 1 {
        steps.push(step(Goal::Explore, Hook::Explore, None));
    }
    Plan {
        steps,
        creatures: Some((creatures, count)),
        dungeon: Some(dungeon),
        ..Plan::default()
    }
}
