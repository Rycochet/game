//! Who is on a site level to begin with: worked out from the site's secret seed,
//! apart from the level's shape, so adding or changing mobs moves no walls and
//! changes no generator version. See `docs/context/shared/mob.md`.

use serde::{Deserialize, Serialize};

use crate::creature::CreatureKind;
use crate::dungeon::{Cell, Level};
use crate::id::Id;
use crate::item::{Drop, ItemKind, Quality, roll_drop};
use crate::poi::PoiKind;
use crate::protocol::CharacterId;
use crate::rng::Seed;
use crate::sheet::{Appearance, Feeling, Trait};

/// Marks these hashes as being for mobs, apart from anything else from the seed.
const PURPOSE: i64 = 0x006d_6f62;

/// One walkable cell in this many has a mob.
const CELLS_PER_MOB: usize = 60;
const MOST_MOBS: usize = 12;
const MOST_BEGGARS: usize = 3;
/// How far from any stairs a mob starts, in cells: nobody arrives beside one.
const FROM_STAIRS: i64 = 6;
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
    /// What it starts with, innermost clothes first: it wears what it can.
    pub gear: Vec<(ItemKind, Option<Quality>)>,
}

/// How near someone must be to give something, in cells.
pub const GIVE_REACH: i64 = 2;

/// How many things a mob remembers: the oldest go first.
const MOST_REMEMBERED: usize = 32;

/// Something that happened, as a mob remembers it. Things, not words, so it can
/// say what it means for how the mob feels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    Gave(ItemKind),
}

impl Event {
    /// How much warmer it makes the mob feel about whoever did it.
    fn warmth(self) -> i32 {
        match self {
            Self::Gave(_) => 30,
        }
    }
}

/// What a mob remembers, about which characters.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Memory(Vec<(CharacterId, Event)>);

impl Memory {
    pub fn remember(&mut self, about: CharacterId, event: Event) {
        if self.0.len() >= MOST_REMEMBERED {
            self.0.remove(0);
        }
        self.0.push((about, event));
    }

    /// How much warmer it feels about this character than it would have.
    pub fn warmth_for(&self, about: &CharacterId) -> i32 {
        let events = self.0.iter().filter(|(who, _)| who == about);
        events.map(|(_, event)| event.warmth()).sum()
    }
}

/// What can be found in a kind of place, with how many of each in 100.
fn who_lives_in(site: PoiKind) -> &'static [(CreatureKind, u64)] {
    match site {
        _ if site.is_town() => &[(CreatureKind::Beggar, 100)],
        PoiKind::Farm => &[(CreatureKind::Chicken, 80), (CreatureKind::Horse, 20)],
        _ => &[
            (CreatureKind::Kobold, 75),
            (CreatureKind::Chicken, 15),
            (CreatureKind::Giant, 10),
        ],
    }
}

/// Where the rolls for a mob's gear start, clear of those for its place and looks.
const GEAR_ROLLS: i64 = 1000;

/// What a kind of creature may start with: for each chance, in 100, of having
/// something there, the kinds it could be and how much each weighs. First guesses.
fn gear_of(kind: CreatureKind) -> &'static [(u64, &'static [(ItemKind, u64)])] {
    use ItemKind::{
        Apple, Belt, Boomerang, Boots, Cloak, Coin, FullArmour, Gloves, Hat, Pouch, Shoes, Socks,
        Stone, Sword, TShirt, Trousers,
    };
    match kind {
        CreatureKind::Beggar => &[
            (100, &[(TShirt, 1)]),
            (100, &[(Trousers, 1)]),
            (50, &[(Shoes, 2), (Socks, 1), (Hat, 1)]),
            (40, &[(Apple, 1), (Coin, 2), (Stone, 1)]),
        ],
        CreatureKind::Giant => &[
            (100, &[(Trousers, 1)]),
            (60, &[(Belt, 1), (Boots, 2), (Cloak, 1)]),
            (20, &[(FullArmour, 1)]),
            (40, &[(Sword, 1)]),
            (30, &[(Coin, 2), (Stone, 2), (Pouch, 1)]),
        ],
        CreatureKind::Kobold => &[
            (50, &[(TShirt, 1), (Trousers, 1)]),
            (30, &[(Gloves, 1), (Boots, 1), (Hat, 1)]),
            (70, &[(Sword, 1), (Boomerang, 2)]),
            (30, &[(Coin, 2), (Stone, 2), (Apple, 1), (Pouch, 1)]),
        ],
        _ => &[],
    }
}

/// Rolls a mob's gear, each place on its own so changing one moves no others.
fn roll_gear(roll: &impl Fn(i64) -> u64, kind: CreatureKind) -> Vec<(ItemKind, Option<Quality>)> {
    let places = gear_of(kind).iter().zip(GEAR_ROLLS..);
    places
        .filter_map(|(&(chance, kinds), place)| {
            let mut rng = Seed::new(roll(place)).rng();
            if rng.range_u64(0, 100) >= chance {
                return None;
            }
            let table: Vec<Drop> = Quality::ALL
                .iter()
                .flat_map(|&quality| {
                    kinds.iter().map(move |&(kind, weight)| Drop {
                        quality,
                        kind,
                        weight,
                    })
                })
                .collect();
            roll_drop(&mut rng, &table)
        })
        .collect()
}

/// The mobs of level `index` of a site. The same every time for the same seed.
/// A town has a few beggars for now.
pub fn spawns(
    seed: Seed,
    index: u32,
    site: PoiKind,
    level: &Level,
    stairs: &[(i64, i64)],
) -> Vec<Spawn> {
    let (min_x, min_y, width, height) = level.bounds();
    let is_home = |x: i64, y: i64| {
        let far_from_stairs = stairs
            .iter()
            .all(|s| (s.0 - x).abs().max((s.1 - y).abs()) >= FROM_STAIRS);
        level.is_walkable(x, y) && level.cell(x, y) != Cell::Ground && far_from_stairs
    };
    let cells = (min_y..min_y + height)
        .flat_map(|y| (min_x..min_x + width).map(move |x| (x, y)))
        .filter(|&(x, y)| is_home(x, y))
        .count();
    // Towns are huge, and only have a few people in the street so far.
    let most = if site.is_town() {
        MOST_BEGGARS
    } else {
        MOST_MOBS
    };
    let wanted = (cells / CELLS_PER_MOB).min(most);
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
            gear: roll_gear(&roll, kind),
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
    /// How it feels about the player being told.
    pub feeling: Feeling,
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
        let first = spawns(Seed::new(5), 0, PoiKind::Castle, &level, &[]);
        assert!(!first.is_empty());
        assert_eq!(first, spawns(Seed::new(5), 0, PoiKind::Castle, &level, &[]));
        assert_ne!(first, spawns(Seed::new(6), 0, PoiKind::Castle, &level, &[]));
        assert_ne!(first, spawns(Seed::new(5), 1, PoiKind::Castle, &level, &[]));
    }

    #[test]
    fn mobs_stand_apart_on_walkable_cells_with_looks_that_suit_them() {
        let level = room();
        for seed in 0..20 {
            let mobs = spawns(Seed::new(seed), 0, PoiKind::Crypt, &level, &[]);
            for (index, mob) in mobs.iter().enumerate() {
                assert!(level.is_walkable(mob.position.0, mob.position.1));
                assert!(mob.appearance.suits(mob.kind));
                assert!(mobs[index + 1..].iter().all(|o| o.position != mob.position));
            }
        }
    }

    #[test]
    fn nobody_starts_beside_the_stairs() {
        let level = room();
        let stairs = [(10, 10), (30, 12)];
        for seed in 0..20 {
            for mob in spawns(Seed::new(seed), 0, PoiKind::Crypt, &level, &stairs) {
                for s in stairs {
                    let gap = (s.0 - mob.position.0)
                        .abs()
                        .max((s.1 - mob.position.1).abs());
                    assert!(gap >= FROM_STAIRS);
                }
            }
        }
    }

    #[test]
    fn mobs_start_with_what_their_kind_can_have_and_animals_with_nothing() {
        let (mut kobolds_armed, mut qualities) = (0, std::collections::BTreeSet::new());
        for seed in 0..300 {
            let roll = |what: i64| Seed::new(seed).at(&[PURPOSE, what]);
            let gear = roll_gear(&roll, CreatureKind::Kobold);
            kobolds_armed += gear
                .iter()
                .any(|&(kind, _)| matches!(kind, ItemKind::Sword | ItemKind::Boomerang))
                as u32;
            for (kind, quality) in gear {
                assert_eq!(quality.is_some(), kind.def().has_quality, "{kind:?}");
                qualities.extend(quality);
            }
            assert!(roll_gear(&roll, CreatureKind::Chicken).is_empty());
            assert!(roll_gear(&roll, CreatureKind::Horse).is_empty());
            let beggar = roll_gear(&roll, CreatureKind::Beggar);
            assert_eq!(beggar[0].0, ItemKind::TShirt);
        }
        assert!((150..270).contains(&kobolds_armed), "{kobolds_armed}");
        assert!(qualities.contains(&Quality::Poor) && qualities.contains(&Quality::Rare));
    }

    #[test]
    fn towns_have_beggars_and_farms_have_animals() {
        let level = room();
        let town = spawns(Seed::new(1), 0, PoiKind::Town, &level, &[]);
        assert!(!town.is_empty() && town.len() <= MOST_BEGGARS);
        assert!(town.iter().all(|mob| mob.kind == CreatureKind::Beggar));
        let farm = spawns(Seed::new(1), 0, PoiKind::Farm, &level, &[]);
        assert!(!farm.is_empty());
        assert!(farm.iter().all(|mob| !mob.kind.def().playable));
    }

    #[test]
    fn memories_warm_a_mob_to_whoever_it_remembers() {
        let (a, b) = (
            CharacterId(crate::id::Id::for_test(1)),
            CharacterId(crate::id::Id::for_test(2)),
        );
        let mut memory = Memory::default();
        memory.remember(a, Event::Gave(ItemKind::Coin));
        assert_eq!(memory.warmth_for(&a), 30);
        assert_eq!(memory.warmth_for(&b), 0);
        for _ in 0..MOST_REMEMBERED {
            memory.remember(b, Event::Gave(ItemKind::Coin));
        }
        // Only so much is kept.
        assert_eq!(memory.warmth_for(&a), 0);
        assert_eq!(memory.warmth_for(&b), 30 * MOST_REMEMBERED as i32);
        // A beggar given one thing goes from neutral to friendly.
        let warmer = Feeling::Neutral.warmth() + 30;
        assert_eq!(Feeling::from_warmth(warmer), Feeling::Friendly);
        // A kobold stops being unfriendly after one, and takes three to be friendly.
        assert_eq!(Feeling::from_warmth(-50 + 30), Feeling::Neutral);
        assert_eq!(Feeling::from_warmth(-50 + 60), Feeling::Neutral);
        assert_eq!(Feeling::from_warmth(-50 + 90), Feeling::Friendly);
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
            feeling: Feeling::Unfriendly,
            x: -2,
            y: 3,
        };
        let bytes = postcard::to_allocvec(&look).unwrap();
        assert_eq!(postcard::from_bytes::<EntityLook>(&bytes).unwrap(), look);
    }
}
