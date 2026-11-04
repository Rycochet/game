//! The armour sets' rows of the table of kinds (`def.rs`): how each is worn. Their
//! names and weights are their forms' (`armour::set`): every piece's together.

use super::item_def::{plain, wear};
use super::{ItemDef, ItemKind, Shape, WearDef};
use crate::creature::{Layer, Location};

impl ItemKind {
    pub(super) const fn set_def(self) -> ItemDef {
        use Layer::Outer;
        use Location::{Arms, Feet, Hands, Head, Legs, Neck, Shoulders, Torso, Waist};
        let big = Shape::rectangle(3, 4);
        match self {
            // Torso, sleeves and skirt of one construction: a mail hauberk goes over a
            // gambeson, so no padding of its own (a lined one says so: `Item::padded`).
            Self::ArmourCoat => plain("Armour Coat", 12_000, big).worn(wear(
                Outer,
                &[Arms, Torso, Waist],
                (0, 0),
                false,
            )),
            // Breast and back with their padding, a gorget and tassets.
            Self::Corslet => plain("Corslet", 9000, Shape::rectangle(3, 3)).worn(WearDef {
                padded: &[Torso],
                ..wear(Outer, &[Neck, Torso, Waist], (0, 0), false)
            }),
            // A set comes with its padding where it is worn whole (the owner's decision),
            // so no gambeson under it; never on the waist, which is the belt's.
            Self::FullArmour => plain("Full Armour", 25_000, big).worn(WearDef {
                padded: &[Shoulders, Arms, Torso],
                ..wear(
                    Outer,
                    &[Head, Neck, Shoulders, Arms, Torso, Waist, Hands, Legs, Feet],
                    (0, 0),
                    false,
                )
            }),
            Self::HalfArmour => plain("Half Armour", 18_000, big).worn(WearDef {
                padded: &[Shoulders, Arms, Torso],
                ..wear(
                    Outer,
                    &[Head, Neck, Shoulders, Arms, Torso, Waist, Hands],
                    (0, 0),
                    false,
                )
            }),
            _ => panic!("not an armour set"),
        }
    }
}
