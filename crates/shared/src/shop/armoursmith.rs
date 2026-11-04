//! What an armoursmith sells: every piece of armour, sets, and horse armour.

use crate::item::ItemKind;

/// The armoursmith's stock, with how much each weighs.
pub(super) const ARMOURSMITH: &[(ItemKind, u64)] = {
    use ItemKind::{
        ArmArmour, ArmourCoat, Aventail, Barding, Bevor, BevoredHelm, Boots, Caparison, Chausses,
        Coif, Corslet, CuirassierTassets, DemiGauntlets, FootArmour, FullArmour, Gambeson,
        Gauntlets, Gorget, HalfArmour, Helmet, LegArmour, Mantle, PaddedGloves, ShoulderArmour,
        TorsoArmour, WaistArmour,
    };
    &[
        (TorsoArmour, 3),
        (Helmet, 3),
        (ShoulderArmour, 2),
        (ArmArmour, 2),
        (Gauntlets, 2),
        (LegArmour, 2),
        (WaistArmour, 2),
        (FootArmour, 2),
        (Gorget, 1),
        (Aventail, 1),
        (Bevor, 1),
        (Mantle, 1),
        (Coif, 1),
        (BevoredHelm, 1),
        (Chausses, 1),
        (CuirassierTassets, 1),
        (DemiGauntlets, 1),
        (PaddedGloves, 1),
        (Gambeson, 1),
        (ArmourCoat, 1),
        (Corslet, 1),
        (HalfArmour, 1),
        (FullArmour, 1),
        (Boots, 1),
        (Barding, 1),
        (Caparison, 1),
    ]
};
