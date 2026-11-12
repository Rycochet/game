//! What each kind of item is: its name, weight, cells, and how it is carried or worn.

use super::item_def::{HELD, bag, plain, wear};
use super::{ItemDef, ItemKind, Shape, WearDef};
use crate::creature::{Layer, Location};

/// All but its top-right cell: an L.
const BOOMERANG: Shape = Shape::rectangle(2, 2).without(1 << 1);
/// A bag of holding's grid: as wide as the ground's, and up to this many rows.
const BAG_OF_HOLDING_WIDTH: u8 = 8;
const BAG_OF_HOLDING_ROWS: u8 = 60;

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
                // Armour sets: in `set_def.rs`.
                Self::ArmourCoat | Self::Corslet | Self::FullArmour | Self::HalfArmour => {
                    self.set_def()
                }
                // Shoes, boots, sandals, clogs and slippers: in `feet_def.rs`.
                Self::Boots | Self::Clogs | Self::Sandals | Self::Shoes | Self::Slippers => {
                    self.feet_def()
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
                Self::Axe => plain("Axe", 900, Shape::rectangle(1, 3)).worn(HELD),
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
                // A placeholder for trying out bags that grow: far more inside than out.
                Self::BagOfHolding => bag(
                    "Bag of Holding",
                    500,
                    Shape::rectangle(3, 3),
                    (BAG_OF_HOLDING_WIDTH, BAG_OF_HOLDING_ROWS),
                    Some(100_000),
                    None,
                )
                .growing(),
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
                // 100 of one make one of the next (`coin.rs`).
                Self::CopperCoin => plain("Copper Coin", 1, square).without_quality(),
                Self::SilverCoin => plain("Silver Coin", 3, square).without_quality(),
                Self::GoldCoin => plain("Gold Coin", 5, square).without_quality(),
                Self::Key => plain("Key", 30, square).without_quality(),
                // Laminated to the knee, wrapping the thighs: Waist and Legs (the owner's
                // decision). Its form says its cells and weight.
                Self::CuirassierTassets => plain(
                    "Cuirassier Tassets",
                    4000,
                    Shape::rectangle(2, 4),
                )
                .worn(wear(Outer, &[Waist, Legs], (0, 0), false)),
                Self::Dagger => plain("Dagger", 400, Shape::rectangle(1, 2)).worn(HELD),
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
                Self::Knife => plain("Knife", 200, Shape::rectangle(1, 2)).worn(HELD),
                Self::Lantern => plain("Lantern", 800, Shape::rectangle(1, 2)),
                // Both legs, or a single Classical greave; its form says its name, cells
                // and weight.
                Self::LegArmour => plain("Leg Armour", 3000, Shape::rectangle(2, 3)).worn(wear(
                    Outer,
                    &[Legs],
                    (0, 0),
                    false,
                )),
                Self::Longsword => plain("Longsword", 2000, Shape::rectangle(1, 6)).worn(HELD),
                Self::Necklace => {
                    plain("Necklace", 40, square).worn(wear(Skin, &[Neck], (5, 5), true))
                }
                // Quilted leggings under mail and plate, in place of trousers.
                Self::PaddedChausses => plain("Padded Chausses", 1200, Shape::rectangle(2, 3))
                    .worn(wear(Middle, &[Legs], (1, 0), false)),
                // Under gauntlets.
                Self::PaddedGloves => plain("Padded Gloves", 150, Shape::rectangle(2, 1))
                    .worn(wear(Middle, &[Hands, Fingers], (1, 0), false)),
                Self::Polearm => plain("Polearm", 3000, Shape::rectangle(1, 8)).worn(HELD),
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
                Self::ShortSword => plain("Short Sword", 900, Shape::rectangle(1, 4)).worn(HELD),
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
                Self::Spear => plain("Spear", 1500, Shape::rectangle(1, 8)).worn(HELD),
                Self::Stone => plain("Stone", 5000, Shape::rectangle(2, 2)).without_quality(),
                // Belted over armour, to the knee, which doesn't count as the legs.
                Self::Surcoat => plain("Surcoat", 800, Shape::rectangle(2, 3)).worn(wear(
                    Over,
                    &[Torso, Waist],
                    (1, 1),
                    true,
                )),
                Self::Sword => plain("Sword", 1400, Shape::rectangle(1, 5)).worn(HELD),
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
                // Underclothes, and the old T-shirt: in `under_def.rs`.
                _ => self.under_def(),
            };
        // First guesses at how much work each is beside the usual.
        def.taking(match self {
            Self::Apple | Self::Stone => 10,
            _ if self.coin_value().is_some() => 10,
            Self::Trousers | Self::Hat | Self::Tabard => 50,
            _ if self.is_under() => 50,
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
            Self::FullArmour | Self::HalfArmour | Self::Barding => 400,
            Self::ArmourCoat | Self::Corslet => 300,
            _ => 100,
        })
    }
}
