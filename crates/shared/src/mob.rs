//! Who is on a site level to begin with: worked out from the site's secret seed,
//! apart from the level's shape, so adding or changing mobs moves no walls and
//! changes no generator version. See `docs/context/shared/mob.md`.

use serde::{Deserialize, Serialize};

use crate::creature::CreatureKind;
use crate::dungeon::{Cell, Level};
use crate::id::Id;
use crate::poi::PoiKind;
use crate::rng::Seed;
use crate::sheet::{Appearance, Trait};

/// Marks these hashes as being for mobs, apart from anything else from the seed.
const PURPOSE: i64 = 0x006d_6f62;

/// One walkable cell in this many has a mob.
const CELLS_PER_MOB: usize = 60;
const MOST_MOBS: usize = 12;
/// How many places are tried for each.
const TRIES: i64 = 16;

/// One of a level's mobs, as generated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spawn {
    /// Which of the level's mobs it is: what its id is saved against.
    pub number: u32,
    pub kind: CreatureKind,
    pub position: (i64, i64),
    pub appearance: Appearance,
}

/// What can be found in a kind of place, with how many of each in 100.
fn who_lives_in(site: PoiKind) -> &'static [(CreatureKind, u64)] {
    match site {
        PoiKind::Farm => &[(CreatureKind::Chicken, 80), (CreatureKind::Horse, 20)],
        _ => &[
            (CreatureKind::Kobold, 75),
            (CreatureKind::Chicken, 15),
            (CreatureKind::Giant, 10),
        ],
    }
}

/// The mobs of level `index` of a site. The same every time for the same seed.
/// None in towns, which will have people of their own.
pub fn spawns(seed: Seed, index: u32, site: PoiKind, level: &Level) -> Vec<Spawn> {
    if site.is_town() {
        return Vec::new();
    }
    let (min_x, min_y, width, height) = level.bounds();
    let is_home = |x: i64, y: i64| level.is_walkable(x, y) && level.cell(x, y) != Cell::Ground;
    let cells = (min_y..min_y + height)
        .flat_map(|y| (min_x..min_x + width).map(move |x| (x, y)))
        .filter(|&(x, y)| is_home(x, y))
        .count();
    let wanted = (cells / CELLS_PER_MOB).min(MOST_MOBS);
    let lives_here = who_lives_in(site);
    let mut mobs: Vec<Spawn> = Vec::new();
    for number in 0..wanted as u32 {
        let roll = |what: i64| seed.at(&[PURPOSE, index as i64, number as i64, what]);
        let spot = (0..TRIES).find_map(|attempt| {
            let x = min_x + (roll(attempt * 2) % width as u64) as i64;
            let y = min_y + (roll(attempt * 2 + 1) % height as u64) as i64;
            let free = mobs.iter().all(|mob| mob.position != (x, y));
            (is_home(x, y) && free).then_some((x, y))
        });
        // A number stays with its mob, gaps or not, so none is shuffled along.
        let Some(position) = spot else {
            continue;
        };
        let mut left = roll(TRIES * 2) % 100;
        let kind = lives_here
            .iter()
            .find(|&&(_, share)| {
                let here = left < share;
                left = left.saturating_sub(share);
                here
            })
            .map_or(lives_here[0].0, |&(kind, _)| kind);
        mobs.push(Spawn {
            number,
            kind,
            position,
            appearance: looks(&roll, kind),
        });
    }
    mobs
}

/// A look for a creature of this kind, by the rolls given.
fn looks(roll: &impl Fn(i64) -> u64, kind: CreatureKind) -> Appearance {
    let chosen = |(place, &which): (usize, &Trait)| {
        let rolled = roll(TRIES * 2 + 1 + place as i64);
        (which, (rolled % which.choices().len() as u64) as u8)
    };
    Appearance(kind.def().looks.iter().enumerate().map(chosen).collect())
}

/// What a player is told about something on the map they can see: never stats.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityLook {
    pub id: Id,
    pub kind: CreatureKind,
    pub appearance: Appearance,
    pub x: i64,
    pub y: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A room big enough for some mobs.
    fn room() -> Level {
        let mut level = Level::new(0, 0, 42, 22);
        for y in 1..21 {
            for x in 1..41 {
                level.set(x, y, Cell::Floor);
            }
        }
        level
    }

    #[test]
    fn the_same_seed_gives_the_same_mobs() {
        let level = room();
        let first = spawns(Seed::new(5), 0, PoiKind::Castle, &level);
        assert!(!first.is_empty());
        assert_eq!(first, spawns(Seed::new(5), 0, PoiKind::Castle, &level));
        assert_ne!(first, spawns(Seed::new(6), 0, PoiKind::Castle, &level));
        assert_ne!(first, spawns(Seed::new(5), 1, PoiKind::Castle, &level));
    }

    #[test]
    fn mobs_stand_apart_on_walkable_cells_with_looks_that_suit_them() {
        let level = room();
        for seed in 0..20 {
            let mobs = spawns(Seed::new(seed), 0, PoiKind::Crypt, &level);
            for (index, mob) in mobs.iter().enumerate() {
                assert!(level.is_walkable(mob.position.0, mob.position.1));
                assert!(mob.appearance.suits(mob.kind));
                assert!(mobs[index + 1..].iter().all(|o| o.position != mob.position));
            }
        }
    }

    #[test]
    fn towns_have_none_and_farms_have_animals() {
        let level = room();
        assert!(spawns(Seed::new(1), 0, PoiKind::Town, &level).is_empty());
        let farm = spawns(Seed::new(1), 0, PoiKind::Farm, &level);
        assert!(!farm.is_empty());
        assert!(farm.iter().all(|mob| !mob.kind.def().playable));
    }

    #[test]
    fn a_move_is_small_enough_to_send_often() {
        let moved = crate::protocol::ServerMessage::EntityMoved {
            id: Id::for_test(u128::MAX),
            x: 100,
            y: -100,
        };
        // Its kind, 17 for the id, and two short numbers.
        let bytes = postcard::to_allocvec(&moved).unwrap();
        assert!(bytes.len() <= 24, "{}", bytes.len());
    }

    #[test]
    fn a_look_reads_back_from_a_message() {
        let look = EntityLook {
            id: Id::for_test(9),
            kind: CreatureKind::Kobold,
            appearance: Appearance::default(),
            x: -2,
            y: 3,
        };
        let bytes = postcard::to_allocvec(&look).unwrap();
        assert_eq!(postcard::from_bytes::<EntityLook>(&bytes).unwrap(), look);
    }
}
