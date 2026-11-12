//! Who lives in the wilderness: a few creatures to each block, by its biome,
//! from the secret seed and the block's position alone, so a block let go and
//! made again has them back as they were.

use super::{Spawn, looks, roll_coins, roll_gear};
use crate::alignment::Alignment;
use crate::climate::Biome;
use crate::creature::CreatureKind;
use crate::dungeon::{Cell, Level};
use crate::gender::Gender;
use crate::id::Id;
use crate::rng::Seed;
use crate::sheet::{Attributes, Build};
use crate::wilderness::BlockPos;

#[cfg(test)]
#[path = "wild_tests.rs"]
mod tests;

/// Marks these hashes as being for wild mobs.
const PURPOSE: i64 = 0x0077_696c;

/// In this many blocks out of 100, nothing lives.
const EMPTY_PERCENT: u64 = 45;

/// The most of a pack or herd in one block.
const MOST_TOGETHER: u64 = 3;

/// How far a pack's members start from its den, in cells each way.
const SPREAD: i64 = 3;

/// How many places are tried for each.
const TRIES: i64 = 16;

/// Who lives in a biome, and how likely each is, in percent.
fn who_lives_in(biome: Biome) -> &'static [(CreatureKind, u64)] {
    match biome {
        Biome::Desert => &[],
        Biome::Tundra | Biome::BorealForest => &[(CreatureKind::Wolf, 100)],
        Biome::Grassland | Biome::Savanna => &[
            (CreatureKind::Horse, 45),
            (CreatureKind::Cattle, 45),
            (CreatureKind::Wolf, 10),
        ],
        Biome::Woodland => &[
            (CreatureKind::Wolf, 50),
            (CreatureKind::Cattle, 30),
            (CreatureKind::Horse, 20),
        ],
        Biome::SeasonalForest | Biome::TemperateRainforest | Biome::TropicalRainforest => {
            &[(CreatureKind::Wolf, 100)]
        }
    }
}

/// A block's creatures (`level`: its cells; `biome`: the land's at its
/// middle): none, or a pack or herd of one kind round a den on open ground.
/// Each one's number is its own across the wilderness (from a hash), and its
/// id `wild_mob_id`.
pub fn wild_spawns(seed: Seed, block: BlockPos, level: &Level, biome: Biome) -> Vec<Spawn> {
    let roll = |what: i64| seed.at(&[PURPOSE, block.x, block.y, what]);
    let lives_here = who_lives_in(biome);
    if lives_here.is_empty() || roll(0) % 100 < EMPTY_PERCENT {
        return Vec::new();
    }
    let mut left = roll(1) % 100;
    let kind = lives_here
        .iter()
        .find(|&&(_, share)| {
            let here = left < share;
            left = left.saturating_sub(share);
            here
        })
        .map_or(lives_here[0].0, |&(kind, _)| kind);
    let (min_x, min_y, width, height) = level.bounds();
    let open = |x: i64, y: i64| {
        level.is_walkable(x, y) && matches!(level.cell(x, y), Cell::Ground | Cell::Grass)
    };
    let den = (0..TRIES).find_map(|attempt| {
        let x = min_x + (roll(2 + attempt * 2) % width as u64) as i64;
        let y = min_y + (roll(3 + attempt * 2) % height as u64) as i64;
        open(x, y).then_some((x, y))
    });
    let Some(den) = den else {
        return Vec::new();
    };
    let together = 1 + roll(100) % MOST_TOGETHER;
    let mut mobs: Vec<Spawn> = Vec::new();
    for member in 0..together as i64 {
        let own = |what: i64| roll(1000 + member * 100 + what);
        let spot = (0..TRIES).find_map(|attempt| {
            let span = (2 * SPREAD + 1) as u64;
            let x = den.0 + (own(attempt * 2) % span) as i64 - SPREAD;
            let y = den.1 + (own(attempt * 2 + 1) % span) as i64 - SPREAD;
            let free = mobs.iter().all(|mob| mob.position != (x, y));
            (level.contains(x, y) && open(x, y) && free).then_some((x, y))
        });
        let Some(position) = spot else {
            continue;
        };
        let gender = Gender::rolled(own(61));
        mobs.push(Spawn {
            number: own(50) as u32,
            kind,
            position,
            appearance: looks(&own, kind),
            gender,
            alignment: Alignment::rolled(kind, own(62)),
            attributes: Attributes::for_kind(
                kind,
                gender,
                &Build::rolled(&mut Seed::new(own(60)).rng()),
            ),
            gear: roll_gear(&own, kind),
            coins: roll_coins(&own, kind),
        });
    }
    mobs
}

/// The id of the wild mob with this number: always the same, so nothing needs
/// saving for a mob nobody has done anything to.
pub fn wild_mob_id(seed: Seed, number: u32) -> Id {
    let hash = seed.at(&[PURPOSE, i64::from(number)]);
    let bytes = [hash.to_le_bytes(), u64::from(number).to_le_bytes()].concat();
    Id(uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, &bytes))
}
