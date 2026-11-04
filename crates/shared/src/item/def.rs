//! What each kind of item is: its name, weight, cells, and how it is carried or worn.

use super::item_def::{bag, plain, wear};
use super::{ItemDef, ItemKind, Shape, WearDef};
use crate::creature::{Layer, Location};

/// All but its top-right cell: an L.
const BOOMERANG: Shape = Shape::rectangle(2, 2).without(1 << 1);

impl ItemKind {
    pub const fn def(self) -> ItemDef {
        use Layer::{Middle, Outer, Over, Skin};
        use Location::{
            Arms, Back, Feet, Fingers, Forelegs, Hands, Head, Hindlegs, Legs, Neck, Shoulders,
            Torso, Waist,
        };
        let square = Shape::rectangle(1, 1);
        let def =
            match self {
                // Collars, gorgets, bevors, aventails and mantles: in `neck_def.rs`.
                Self::Aventail | Self::Bevor | Self::Collar | Self::Gorget | Self::Mantle => {
                    self.neck_def()
                }
                Self::Apple => plain("Apple", 150, square).without_quality(),
                // Both arms, or one; its form says its name, cells and weight, and whether
                // it is padded (kote).
                Self::ArmArmour => plain("Arm Armour", 2000, Shape::rectangle(2, 3)).worn(wear(
                    Outer,
                    &[Arms],
                    (0, 0),
                    false,
                )),
                // The rest of the head pieces' looks, cells and weights are their forms'.
                Self::ArmingCap => plain("Arming Cap", 250, Shape::rectangle(2, 1)).worn(wear(
                    Skin,
                    &[Head],
                    (1, 1),
                    true,
                )),
                // Under torso armour; lined armour takes its place (`Item::padded`).
                Self::ArmingVest => plain("Arming Vest", 1200, Shape::rectangle(2, 2)).worn(wear(
                    Middle,
                    &[Torso],
                    (1, 0),
                    false,
                )),
                // A blade's weight and shape are its form's: these are only for one without.
                Self::Axe => plain("Axe", 900, Shape::rectangle(1, 3)),
                Self::Backpack => bag(
                    "Backpack",
                    1200,
                    Shape::rectangle(3, 4),
                    (10, 12),
                    Some(25_000),
                    None,
                )
                .worn(WearDef {
                    one_place: true,
                    ..wear(Outer, &[Back], (1, 1), true)
                }),
                Self::Barding => plain("Barding", 15_000, Shape::rectangle(4, 4)).worn(wear(
                    Outer,
                    &[Neck, Torso, Forelegs, Hindlegs],
                    (0, 0),
                    false,
                )),
                Self::Belt => plain("Belt", 200, Shape::rectangle(1, 3)).worn(wear(
                    Middle,
                    &[Waist],
                    (2, 2),
                    true,
                )),
                // Its own plates guard the throat, as a gorget would.
                Self::BevoredHelm => plain("Bevored Helm", 4000, Shape::rectangle(2, 3))
                    .worn(wear(Outer, &[Head, Neck], (0, 0), false)),
                Self::Boomerang => plain("Boomerang", 400, BOOMERANG),
                // Middle, as shoes are, so foot armour goes over them.
                Self::Boots => plain("Boots", 1100, Shape::rectangle(2, 2)).worn(wear(
                    Middle,
                    &[Feet],
                    (1, 0),
                    false,
                )),
                // Cloth over barding, to below the knees, which doesn't count as the legs.
                Self::Caparison => plain("Caparison", 4000, Shape::rectangle(3, 3)).worn(wear(
                    Over,
                    &[Head, Neck, Torso],
                    (0, 1),
                    false,
                )),
                // Mail chausses with feet, over the shoes; its form says the rest.
                Self::Chausses => plain("Chausses", 3800, Shape::rectangle(2, 3)).worn(wear(
                    Outer,
                    &[Legs, Feet],
                    (1, 0),
                    false,
                )),
                Self::Cloak => plain("Cloak", 900, Shape::rectangle(2, 3)).worn(WearDef {
                    one_place: true,
                    ..wear(Outer, &[Back], (1, 1), true)
                }),
                Self::Coif => plain("Coif", 2000, Shape::rectangle(2, 2)).worn(wear(
                    Middle,
                    &[Head, Neck],
                    (1, 0),
                    false,
                )),
                Self::Coin => plain("Coin", 10, square).without_quality(),
                // Laminated to the knee, wrapping the thighs: Waist and Legs (the owner's
                // decision). Its form says its cells and weight.
                Self::CuirassierTassets => plain(
                    "Cuirassier Tassets",
                    4000,
                    Shape::rectangle(2, 4),
                )
                .worn(wear(Outer, &[Waist, Legs], (0, 0), false)),
                Self::Dagger => plain("Dagger", 400, Shape::rectangle(1, 2)),
                // Sabatons, mail shoes and foot guards, over shoes or boots; its form says
                // its name, cells and weight.
                Self::FootArmour => plain("Foot Armour", 800, Shape::rectangle(2, 2)).worn(wear(
                    Outer,
                    &[Feet],
                    (0, 0),
                    false,
                )),
                // Over the back of the hand and the wrist only, so rings stay free.
                Self::DemiGauntlets => plain("Demi-gauntlets", 450, Shape::rectangle(2, 1))
                    .worn(wear(Outer, &[Hands], (0, 0), false)),
                Self::FingerlessGloves => plain("Fingerless Gloves", 80, Shape::rectangle(1, 2))
                    .worn(wear(Middle, &[Hands], (1, 1), true)),
                // With its own padding, so no gambeson under it.
                Self::FullArmour => {
                    plain("Full Armour", 20_000, Shape::rectangle(3, 4)).worn(WearDef {
                        padded: &[Shoulders, Arms, Torso],
                        ..wear(Outer, &[Shoulders, Arms, Torso, Legs], (0, 0), false)
                    })
                }
                // Over the shoulders too (the owner's decision), under pauldrons.
                Self::Gambeson => plain("Gambeson", 2500, Shape::rectangle(2, 3)).worn(wear(
                    Middle,
                    &[Shoulders, Torso, Arms],
                    (1, 0),
                    false,
                )),
                Self::Gauntlets => plain("Gauntlets", 1200, Shape::rectangle(2, 1)).worn(wear(
                    Outer,
                    &[Hands, Fingers],
                    (0, 0),
                    false,
                )),
                Self::Gem => plain("Gem", 5, square),
                Self::Gloves => plain("Gloves", 100, Shape::rectangle(1, 2)).worn(wear(
                    Middle,
                    &[Hands, Fingers],
                    (1, 0),
                    false,
                )),
                Self::Hat => plain("Hat", 150, Shape::rectangle(2, 1)).worn(wear(
                    Outer,
                    &[Head],
                    (1, 0),
                    false,
                )),
                Self::Helmet => plain("Helmet", 2000, Shape::rectangle(2, 2)).worn(wear(
                    Outer,
                    &[Head],
                    (0, 0),
                    false,
                )),
                Self::Knife => plain("Knife", 200, Shape::rectangle(1, 2)),
                Self::Lantern => plain("Lantern", 800, Shape::rectangle(1, 2)),
                // Both legs, or a single Classical greave; its form says its name, cells
                // and weight.
                Self::LegArmour => plain("Leg Armour", 3000, Shape::rectangle(2, 3)).worn(wear(
                    Outer,
                    &[Legs],
                    (0, 0),
                    false,
                )),
                Self::Longsword => plain("Longsword", 2000, Shape::rectangle(1, 6)),
                Self::Necklace => {
                    plain("Necklace", 40, square).worn(wear(Skin, &[Neck], (5, 5), true))
                }
                // Quilted leggings under mail and plate, in place of trousers.
                Self::PaddedChausses => plain("Padded Chausses", 1200, Shape::rectangle(2, 3))
                    .worn(wear(Middle, &[Legs], (1, 0), false)),
                // Under gauntlets.
                Self::PaddedGloves => plain("Padded Gloves", 150, Shape::rectangle(2, 1))
                    .worn(wear(Middle, &[Hands, Fingers], (1, 0), false)),
                Self::Polearm => plain("Polearm", 3000, Shape::rectangle(1, 8)),
                Self::Pouch => bag(
                    "Pouch",
                    80,
                    Shape::rectangle(2, 2),
                    (4, 4),
                    Some(1500),
                    Some(500),
                ),
                Self::Ring => plain("Ring", 10, square).worn(WearDef {
                    one_place: true,
                    ..wear(Skin, &[Fingers], (1, 1), true)
                }),
                Self::Shoes => plain("Shoes", 500, Shape::rectangle(2, 1)).worn(wear(
                    Middle,
                    &[Feet],
                    (1, 0),
                    false,
                )),
                Self::ShortSword => plain("Short Sword", 900, Shape::rectangle(1, 4)),
                // A pair: its form says its kind, name, cells and weight.
                Self::ShoulderArmour => plain("Shoulder Armour", 2600, Shape::rectangle(3, 2))
                    .worn(wear(Outer, &[Shoulders], (0, 0), false)),
                Self::SmallBox => bag(
                    "Small Box",
                    100,
                    Shape::rectangle(2, 1),
                    (3, 2),
                    Some(300),
                    None,
                ),
                Self::Socks => plain("Socks", 50, square).worn(wear(Skin, &[Feet], (1, 1), true)),
                Self::Spear => plain("Spear", 1500, Shape::rectangle(1, 8)),
                Self::Stone => plain("Stone", 5000, Shape::rectangle(2, 2)).without_quality(),
                // Belted over armour, to the knee, which doesn't count as the legs.
                Self::Surcoat => plain("Surcoat", 800, Shape::rectangle(2, 3)).worn(wear(
                    Over,
                    &[Torso, Waist],
                    (1, 1),
                    true,
                )),
                Self::Sword => plain("Sword", 1400, Shape::rectangle(1, 5)),
                Self::TShirt => plain("T-Shirt", 200, Shape::rectangle(2, 2)).worn(wear(
                    Skin,
                    &[Torso, Arms],
                    (1, 0),
                    false,
                )),
                // Open at the sides, so it fits more than one size.
                Self::Tabard => plain("Tabard", 500, Shape::rectangle(2, 3)).worn(wear(
                    Over,
                    &[Shoulders, Torso],
                    (1, 1),
                    true,
                )),
                // Its form says its name, cells and weight, and whether it is lined.
                Self::TorsoArmour => plain("Torso Armour", 5000, Shape::rectangle(3, 3))
                    .worn(wear(Outer, &[Torso], (0, 0), false)),
                // Faulds, tassets, skirts and the rest; its form says its name, cells and
                // weight. Never padded: Waist's Middle is the belt's.
                Self::WaistArmour => plain("Waist Armour", 2500, Shape::rectangle(2, 2))
                    .worn(wear(Outer, &[Waist], (0, 0), false)),
                Self::Trousers => plain("Trousers", 500, Shape::rectangle(2, 3)).worn(wear(
                    Middle,
                    &[Legs],
                    (1, 0),
                    false,
                )),
            };
        // First guesses at how much work each is beside the usual.
        def.taking(match self {
            Self::Apple | Self::Coin | Self::Stone => 10,
            Self::Socks | Self::TShirt | Self::Trousers | Self::Hat | Self::Tabard => 50,
            Self::Ring | Self::Pouch | Self::Belt | Self::Boomerang => 75,
            Self::Gauntlets
            | Self::Helmet
            | Self::BevoredHelm
            | Self::Coif
            | Self::DemiGauntlets
            | Self::ShoulderArmour
            | Self::TorsoArmour
            | Self::ArmArmour
            | Self::LegArmour
            | Self::Chausses
            | Self::FootArmour
            | Self::Collar
            | Self::Gorget
            | Self::Bevor
            | Self::Aventail
            | Self::Mantle
            | Self::WaistArmour
            | Self::CuirassierTassets
            | Self::Backpack
            | Self::Necklace => 150,
            Self::Sword
            | Self::Axe
            | Self::Dagger
            | Self::Knife
            | Self::Longsword
            | Self::Polearm
            | Self::ShortSword
            | Self::Spear => 200,
            Self::FullArmour | Self::Barding => 400,
            _ => 100,
        })
    }
}
