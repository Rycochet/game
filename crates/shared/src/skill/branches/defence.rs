//! Defence, under melee: parrying and blocking, by the type of weapon (one level less
//! deep than attacking: no single weapons), with no weapon, or with a shield. Under
//! melee so attack and defence each build the other through it, and shields are here
//! so all defending practises Defence, and one passive ability can draw on it alone
//! (owner, 2026-11-06). Shields can be specialised; the weapon types here can't. The
//! weapon types match melee's own (a test checks).

use crate::sheet::Attribute::{Agility, Perception, Strength, Toughness};

use super::super::draft::{Draft, branch, leaf};

pub(super) const DEFENCE: Draft = branch(
    "defence",
    "Defence",
    &[Agility, Perception],
    &[
        leaf("unarmed", "Unarmed", &[Agility, Toughness]),
        leaf("knife", "Knife", &[]),
        leaf("dagger", "Dagger", &[]),
        leaf("shortsword", "Short Sword", &[]),
        leaf("sword", "Sword", &[]),
        leaf("longsword", "Longsword", &[Agility, Strength]),
        leaf("axe", "Axe", &[Strength, Agility]),
        leaf("spear", "Spear", &[]),
        leaf("polearm", "Polearm", &[Strength, Agility]),
        leaf("club", "Club", &[]),
        leaf("mace", "Mace", &[]),
        leaf("hammer", "Hammer", &[]),
        leaf("flail", "Flail", &[]),
        leaf("staff", "Staff", &[]),
        leaf("whip", "Whip", &[]),
        leaf("buckler", "Buckler", &[Agility]),
        leaf("roundshield", "Round Shield", &[Strength, Toughness]),
        leaf("kiteshield", "Kite Shield", &[Strength, Toughness]),
        leaf("towershield", "Tower Shield", &[Strength, Toughness]),
    ],
);
