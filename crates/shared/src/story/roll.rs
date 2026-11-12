//! Rolling the people a story might need, from the text seed. Every role gets
//! a candidate; only those a chosen beat uses end up in the cast.

use super::cast::{CastPet, Member, Role};
use super::facts::{Errand, Passerby, Relation, State, TimeAgo, Trait};
use super::{Household, Pet, PlaceKind};
use crate::creature::CreatureKind;
use crate::rng::{Rng, Seed};

/// One of `options`, by `rng`.
pub(super) fn pick<T: Copy>(rng: &mut Rng, options: &[T]) -> T {
    options[rng.range_u64(0, options.len() as u64) as usize]
}

/// What anyone is like.
fn nature(seed: Seed) -> Trait {
    pick(&mut seed.derive("nature").rng(), Trait::ALL)
}

pub(super) fn giver(household: &Household, index: u32, seed: Seed) -> Member {
    Member {
        role: Role::Giver,
        person: household.member(index),
        relation: None,
        nature: nature(seed.derive("giver")),
        state: State::Alive,
    }
}

/// Kin from the giver's household three times in four, else a friend,
/// betrothed, master or partner from another; the relation suits their gender.
pub(super) fn relative(household: &Household, giver_index: u32, seed: Seed) -> Member {
    let seed = seed.derive("relative");
    let mut rng = seed.rng();
    let kin = rng.range_u64(0, 4) > 0;
    let person = if kin {
        // Far from the giver's own index, so never the giver.
        let index = giver_index.wrapping_add(1 + rng.range_u64(0, 1000) as u32);
        household.member(index)
    } else {
        Household::new(household.kind, seed.derive("household")).member(0)
    };
    let relations: Vec<Relation> = (Relation::ALL.iter().copied())
        .filter(|r| r.is_kin() == kin && r.suits(person.gender))
        .collect();
    Member {
        role: Role::Relative,
        person,
        relation: Some(pick(&mut rng, &relations)),
        nature: nature(seed),
        state: State::Alive,
    }
}

/// Someone of `kind` from a household of their own, in `role`.
pub(super) fn outsider(
    kind: CreatureKind,
    role: Role,
    relation: Option<Relation>,
    seed: Seed,
) -> Member {
    let seed = seed.derive(role.key());
    let nature = if role == Role::Rival {
        let natures = [
            Trait::Greedy,
            Trait::Proud,
            Trait::Stubborn,
            Trait::HotTempered,
        ];
        pick(&mut seed.derive("nature").rng(), &natures)
    } else {
        nature(seed)
    };
    Member {
        role,
        person: Household::new(kind, seed.derive("household")).member(0),
        relation,
        nature,
        state: State::Alive,
    }
}

pub(super) fn passerby(seed: Seed) -> Passerby {
    pick(&mut seed.derive("passerby").rng(), Passerby::ALL)
}

pub(super) fn pet(pet: Pet, seed: Seed) -> CastPet {
    let natures = [
        Trait::Curious,
        Trait::Fearful,
        Trait::Adventurous,
        Trait::Stubborn,
        Trait::Kind,
    ];
    CastPet {
        pet,
        nature: pick(&mut seed.derive("pet nature").rng(), &natures),
        state: State::Alive,
    }
}

/// Why someone went to a place of this kind.
pub(super) fn errand(place: PlaceKind, seed: Seed) -> Errand {
    let allowed: Vec<Errand> = (Errand::ALL.iter().copied())
        .filter(|e| *e != Errand::Eloping && e.allowed_at(place))
        .collect();
    pick(&mut seed.derive("errand").rng(), &allowed)
}

/// How long ago; elders' stories lean to years and long ago.
pub(super) fn time_ago(elder: bool, seed: Seed) -> TimeAgo {
    use TimeAgo::*;
    let weights: [(TimeAgo, u64); 5] = if elder {
        [
            (Days, 1),
            (Weeks, 1),
            (LastSeason, 2),
            (Years, 4),
            (LongAgo, 3),
        ]
    } else {
        [
            (Days, 3),
            (Weeks, 3),
            (LastSeason, 2),
            (Years, 1),
            (LongAgo, 0),
        ]
    };
    seed.derive("time ago").rng().pick_weighted(&weights)
}
