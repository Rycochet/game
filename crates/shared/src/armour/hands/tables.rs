//! How often each choice is rolled, by construction and style.

use super::{Cuff, Fingers, HandsPiece, Knuckles};
use crate::armour::{Construction, Style};
use crate::material::Material;

/// Which constructions a kind of piece is made in, and how often.
pub(super) fn constructions(piece: HandsPiece) -> &'static [(Construction, u64)] {
    use Construction as C;
    match piece {
        HandsPiece::Gauntlets => &[
            (C::Plate, 30),
            (C::Mail, 20),
            (C::Leather, 15),
            (C::PlatesInCloth, 10),
            (C::Scale, 8),
            (C::MailAndPlate, 7),
        ],
        HandsPiece::Demi => &[(C::Plate, 2), (C::MailAndPlate, 1)],
        HandsPiece::Padded => &[(C::Padded, 1)],
    }
}

/// A style narrows plate to what looks like it (the owner's decision): Kastenbrust is
/// always mittens with long cuffs, Gothic always pointed, Maximilian fingered with
/// fluted bells, Milanese mostly mittens, Greenwich bells, Dwarven never long.
pub(super) fn fingers_for(
    construction: Construction,
    piece: HandsPiece,
    style: Style,
) -> &'static [(Fingers, u64)] {
    use Construction as C;
    use Fingers::{Fingered, Mitten, Open};
    match (piece, construction) {
        (HandsPiece::Demi, _) => &[(Open, 1)],
        (HandsPiece::Padded, _) => &[(Fingered, 1)],
        (_, C::Plate) => match style {
            Style::Kastenbrust => &[(Mitten, 1)],
            Style::Maximilian => &[(Fingered, 1)],
            Style::Milanese => &[(Fingered, 25), (Mitten, 75)],
            _ => &[(Fingered, 60), (Mitten, 40)],
        },
        (_, C::PlatesInCloth) => &[(Fingered, 50), (Mitten, 50)],
        (_, C::Mail) => &[(Fingered, 40), (Mitten, 60)],
        (_, C::Scale) => &[(Fingered, 60), (Mitten, 40)],
        (_, C::Leather) => &[(Fingered, 85), (Mitten, 15)],
        _ => &[(Fingered, 1)],
    }
}

pub(super) fn cuffs_for(
    construction: Construction,
    fingers: Fingers,
    style: Style,
) -> &'static [(Cuff, u64)] {
    use Construction as C;
    use Cuff::{Bell, Hourglass, Long, Pointed, Wrist};
    match (construction, fingers) {
        (C::Plate, Fingers::Open) => &[(Bell, 1)],
        (C::Plate, _) => match style {
            Style::Gothic => &[(Pointed, 1)],
            Style::Kastenbrust => &[(Long, 1)],
            Style::Maximilian | Style::Greenwich => &[(Bell, 1)],
            Style::Dwarven => &[(Hourglass, 40), (Bell, 60)],
            _ => &[(Hourglass, 25), (Bell, 40), (Pointed, 20), (Long, 15)],
        },
        (C::Leather, _) => &[(Wrist, 20), (Bell, 50), (Long, 30)],
        (C::Scale, _) => &[(Wrist, 50), (Bell, 50)],
        (C::PlatesInCloth, _) => &[(Wrist, 50), (Hourglass, 50)],
        (C::MailAndPlate, Fingers::Fingered) => &[(Wrist, 60), (Bell, 40)],
        _ => &[(Wrist, 1)],
    }
}

pub(super) fn knuckles_for(construction: Construction) -> &'static [(Knuckles, u64)] {
    use Construction as C;
    use Knuckles::{Plain, Ridged, Rondel, Spiked};
    match construction {
        C::Plate => &[(Plain, 40), (Ridged, 35), (Spiked, 10), (Rondel, 15)],
        C::Leather => &[(Plain, 80), (Spiked, 20)],
        C::Scale | C::PlatesInCloth => &[(Plain, 85), (Spiked, 15)],
        C::MailAndPlate => &[(Plain, 60), (Ridged, 40)],
        _ => &[(Plain, 1)],
    }
}

pub(super) fn may_have_claws(construction: Construction, fingers: Fingers) -> bool {
    use Construction as C;
    fingers == Fingers::Fingered && matches!(construction, C::Plate | C::Scale | C::MailAndPlate)
}

pub(super) fn may_lock(construction: Construction, fingers: Fingers) -> bool {
    construction == Construction::Plate && fingers == Fingers::Mitten
}

pub(super) fn materials_for(construction: Construction) -> &'static [(Material, u64)] {
    use Construction as C;
    use Material as M;
    match construction {
        C::Plate => &[
            (M::Steel, 50),
            (M::Iron, 35),
            (M::Bronze, 5),
            (M::Chitin, 3),
            (M::Mithril, 2),
            (M::Adamantine, 1),
        ],
        C::Mail => &[
            (M::Steel, 50),
            (M::Iron, 40),
            (M::Bronze, 5),
            (M::Mithril, 3),
            (M::Adamantine, 1),
        ],
        C::Scale => &[
            (M::Steel, 30),
            (M::Iron, 30),
            (M::Horn, 15),
            (M::HardenedLeather, 15),
            (M::Bronze, 10),
            (M::Chitin, 3),
            (M::DragonScale, 2),
            (M::Mithril, 2),
        ],
        C::PlatesInCloth | C::MailAndPlate => &[(M::Steel, 55), (M::Iron, 45)],
        C::Leather => &[
            (M::BuffLeather, 40),
            (M::Leather, 30),
            (M::HardenedLeather, 30),
        ],
        _ => &[(M::Linen, 45), (M::Wool, 30), (M::Cotton, 25)],
    }
}

pub(super) fn needs_cloth(construction: Construction, fingers: Fingers) -> bool {
    construction == Construction::PlatesInCloth
        || (construction == Construction::MailAndPlate && fingers == Fingers::Open)
}

pub(super) fn cloths_for(construction: Construction) -> &'static [(Material, u64)] {
    use Material as M;
    match construction {
        Construction::MailAndPlate => &[(M::Silk, 50), (M::Linen, 50)],
        _ => &[(M::Linen, 40), (M::Wool, 35), (M::Silk, 10)],
    }
}

pub(super) fn may_have_trim(construction: Construction, material: Material) -> bool {
    construction == Construction::Plate
        && matches!(
            material,
            Material::Steel | Material::Iron | Material::Mithril
        )
}
