//! Combat: fighting, a level deeper than the rest as attacks happen far more often.
//! Under melee, the third level is the type of weapon and the fourth the weapon itself
//! (`combat/melee/longsword/longsword`, `combat/melee/shortsword/khopesh`), and
//! fighting with no weapon is `combat/melee/unarmed/<kind of creature>` (owner,
//! 2026-11-06). Defending is under melee too (`defence.rs`). A first draft, for the
//! owner to change.

use crate::blade::Family;
use crate::sheet::Attribute::{Agility, Empathy, Perception, Strength, Toughness};

use super::super::draft::{Draft, Filled, branch, filled, leaf};
use super::defence::DEFENCE;

const MELEE: Draft = branch(
    "melee",
    "Melee",
    &[Strength, Agility],
    &[
        filled(
            "unarmed",
            "Unarmed",
            &[Strength, Agility, Toughness],
            Filled::Kinds,
        ),
        filled("knife", "Knife", &[Agility], Filled::Weapons(Family::Knife)),
        filled(
            "dagger",
            "Dagger",
            &[Agility],
            Filled::Weapons(Family::Dagger),
        ),
        filled(
            "shortsword",
            "Short Sword",
            &[Agility, Strength],
            Filled::Weapons(Family::ShortSword),
        ),
        filled("sword", "Sword", &[], Filled::Weapons(Family::Sword)),
        filled(
            "longsword",
            "Longsword",
            &[Strength, Agility],
            Filled::Weapons(Family::LongSword),
        ),
        filled("axe", "Axe", &[Strength], Filled::Weapons(Family::Axe)),
        filled(
            "spear",
            "Spear",
            &[Agility, Strength],
            Filled::Weapons(Family::Spear),
        ),
        filled(
            "polearm",
            "Polearm",
            &[Strength],
            Filled::Weapons(Family::Polearm),
        ),
        branch("club", "Club", &[Strength], &[leaf("club", "Club", &[])]),
        branch("mace", "Mace", &[Strength], &[leaf("mace", "Mace", &[])]),
        branch(
            "hammer",
            "Hammer",
            &[Strength],
            &[leaf("warhammer", "War Hammer", &[])],
        ),
        branch(
            "flail",
            "Flail",
            &[Strength, Agility],
            &[leaf("flail", "Flail", &[])],
        ),
        branch(
            "staff",
            "Staff",
            &[Agility],
            &[leaf("quarterstaff", "Quarterstaff", &[])],
        ),
        branch("whip", "Whip", &[Agility], &[leaf("whip", "Whip", &[])]),
        DEFENCE,
    ],
);

const RANGED: Draft = branch(
    "ranged",
    "Ranged",
    &[Agility, Perception],
    &[
        branch(
            "thrown",
            "Thrown",
            &[Agility, Strength],
            &[
                leaf("knife", "Throwing Knife", &[]),
                leaf("axe", "Throwing Axe", &[]),
                leaf("javelin", "Javelin", &[]),
                leaf("stone", "Stone", &[]),
            ],
        ),
        branch(
            "bow",
            "Bow",
            &[],
            &[
                leaf("shortbow", "Shortbow", &[]),
                leaf("longbow", "Longbow", &[Strength, Perception]),
                leaf("recurve", "Recurve Bow", &[]),
            ],
        ),
        branch(
            "crossbow",
            "Crossbow",
            &[Perception],
            &[
                leaf("light", "Light Crossbow", &[]),
                leaf("heavy", "Heavy Crossbow", &[]),
            ],
        ),
        branch(
            "sling",
            "Sling",
            &[],
            &[
                leaf("sling", "Sling", &[]),
                leaf("staff", "Staff Sling", &[]),
            ],
        ),
    ],
);

pub(in super::super) const COMBAT: Draft = branch(
    "combat",
    "Combat",
    &[Strength, Agility],
    &[
        MELEE,
        RANGED,
        leaf("anticipation", "Anticipation", &[Perception, Empathy]),
        branch(
            "armour",
            "Armour",
            &[Toughness],
            &[
                leaf("light", "Light Armour", &[Agility]),
                leaf("heavy", "Heavy Armour", &[Strength, Toughness]),
            ],
        ),
    ],
);
