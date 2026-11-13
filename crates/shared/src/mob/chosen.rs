//! Mobs of a kind chosen by a developer (a Dev Area), put anywhere open.

use super::{
    ALIGNMENT_ROLL, ATTRIBUTE_ROLL, FROM_STAIRS, GENDER_ROLL, PURPOSE, Spawn, TRIES, looks,
    roll_coins, roll_gear,
};
use crate::alignment::Alignment;
use crate::creature::CreatureKind;
use crate::dungeon::Level;
use crate::gender::Gender;
use crate::rng::Seed;
use crate::sheet::{Attributes, Build};

/// Marks these rolls as for chosen mobs, apart from a level's own.
const CHOSEN: i64 = 0x6368_6f73;
/// Places tried for each, many more than a level's own as the room is small.
const CHOSEN_TRIES: i64 = 4 * TRIES;
/// Where the rolls for places start, clear of those for looks, gear and build.
const PLACES: i64 = 10_000;

/// `count` mobs of `kind` on walkable cells of `level`, none within
/// `FROM_STAIRS` of any of `clear_of` nor on another. Rolled like a level's
/// own (looks, build and gear); fewer if room can't be found.
pub fn chosen_spawns(
    seed: Seed,
    kind: CreatureKind,
    count: u8,
    level: &Level,
    clear_of: &[(i64, i64)],
) -> Vec<Spawn> {
    let (min_x, min_y, width, height) = level.bounds();
    let is_home = |x: i64, y: i64| {
        let clear = clear_of
            .iter()
            .all(|s| (s.0 - x).abs().max((s.1 - y).abs()) >= FROM_STAIRS);
        level.is_walkable(x, y) && clear
    };
    let mut mobs: Vec<Spawn> = Vec::new();
    for number in 0..u32::from(count) {
        let roll = |what: i64| seed.at(&[PURPOSE, CHOSEN, number as i64, what]);
        let spot = (0..CHOSEN_TRIES).find_map(|attempt| {
            let x = min_x + (roll(PLACES + attempt * 2) % width as u64) as i64;
            let y = min_y + (roll(PLACES + attempt * 2 + 1) % height as u64) as i64;
            let free = mobs.iter().all(|mob| mob.position != (x, y));
            (is_home(x, y) && free).then_some((x, y))
        });
        let Some(position) = spot else {
            continue;
        };
        mobs.push(spawn_at(seed, kind, number, position));
    }
    mobs
}

/// Mob `number` of `kind` at `position`, rolled from `seed` as `chosen_spawns`
/// rolls each (looks, build and gear), for callers choosing places of their own.
pub fn spawn_at(seed: Seed, kind: CreatureKind, number: u32, position: (i64, i64)) -> Spawn {
    let roll = |what: i64| seed.at(&[PURPOSE, CHOSEN, number as i64, what]);
    let gender = Gender::rolled(roll(GENDER_ROLL));
    Spawn {
        number,
        kind,
        position,
        appearance: looks(&roll, kind).fitted(gender),
        gender,
        alignment: Alignment::rolled(kind, roll(ALIGNMENT_ROLL)),
        attributes: Attributes::for_kind(
            kind,
            gender,
            &Build::rolled(&mut Seed::new(roll(ATTRIBUTE_ROLL)).rng()),
        ),
        gear: roll_gear(&roll, kind),
        coins: roll_coins(&roll, kind),
    }
}
