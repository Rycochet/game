//! What arm pieces may be made of and how much of the arm they cover, and how often:
//! the lists rolls pick from and the grid checks against.

use super::Reach;
use crate::armour::{Construction, Style};
use crate::material::Material;

/// Constructions arm pieces are made in, by style. Each keeps to `Style::allows`
/// (checked by a test), narrowed to what was worn on the arms.
pub(super) fn constructions(style: Style) -> &'static [(Construction, u64)] {
    use Construction as C;
    match style {
        Style::Elven => &[
            (C::Plate, 40),
            (C::Leather, 30),
            (C::Scale, 15),
            (C::Mail, 15),
        ],
        Style::Dwarven => &[
            (C::Plate, 60),
            (C::Leather, 15),
            (C::Mail, 15),
            (C::Splint, 10),
        ],
        Style::Japanese => &[
            (C::MailAndPlate, 55),
            (C::Mail, 20),
            (C::PlatesInCloth, 15),
            (C::Padded, 10),
        ],
        Style::Chinese => &[
            (C::Lamellar, 40),
            (C::Scale, 25),
            (C::Leather, 15),
            (C::PlatesInCloth, 15),
            (C::Padded, 5),
        ],
        Style::Steppe => &[
            (C::Lamellar, 40),
            (C::Laminar, 25),
            (C::Mail, 20),
            (C::Leather, 15),
        ],
        Style::Classical => &[
            (C::Laminar, 45),
            (C::Plate, 25),
            (C::Scale, 15),
            (C::Leather, 15),
        ],
        Style::Byzantine => &[
            (C::Splint, 35),
            (C::Lamellar, 25),
            (C::Mail, 25),
            (C::Leather, 15),
        ],
        Style::IndoPersian => &[(C::Plate, 55), (C::Mail, 30), (C::MailAndPlate, 15)],
        // The European harnesses.
        _ => &[
            (C::Plate, 50),
            (C::Leather, 15),
            (C::Mail, 15),
            (C::Splint, 8),
            (C::PlatesInCloth, 5),
            (C::RingMail, 4),
            (C::Padded, 3),
        ],
    }
}

/// How much of the arm a construction covers, in a style.
pub(super) fn reaches(construction: Construction, style: Style) -> &'static [(Reach, u64)] {
    use Construction as C;
    use Reach as R;
    match (construction, style) {
        // Bazubands, with or without the point over the elbow.
        (C::Plate, Style::IndoPersian) => &[(R::Forearm, 40), (R::ForearmAndElbow, 60)],
        // Bronze forearm guards: Dendra, and the coiled ones.
        (C::Plate, Style::Classical) => &[(R::Forearm, 1)],
        (C::Plate, _) => &[
            (R::Whole, 35),
            (R::ForearmAndElbow, 20),
            (R::Forearm, 15),
            (R::UpperAndElbow, 10),
            (R::Elbow, 10),
            (R::Wrist, 10),
        ],
        (C::Leather, _) => &[
            (R::Wrist, 40),
            (R::Forearm, 30),
            (R::ForearmAndElbow, 10),
            (R::Whole, 10),
            (R::UpperAndElbow, 5),
            (R::Elbow, 5),
        ],
        (C::Splint, _) => &[
            (R::Forearm, 35),
            (R::ForearmAndElbow, 30),
            (R::Whole, 25),
            (R::UpperAndElbow, 10),
        ],
        // The manica, shoulder to wrist or forearm and elbow.
        (C::Laminar, Style::Classical) => &[(R::Whole, 70), (R::ForearmAndElbow, 30)],
        (C::Laminar, _) => &[(R::Forearm, 1)],
        (C::Mail | C::RingMail, _) => &[(R::Whole, 1)],
        (C::MailAndPlate, _) => &[(R::Whole, 75), (R::Forearm, 25)],
        (C::PlatesInCloth | C::Padded, _) => &[(R::Whole, 70), (R::Forearm, 30)],
        _ => &[(R::Whole, 65), (R::Forearm, 35)],
    }
}

/// What the main part may be made of.
pub(super) fn materials(
    construction: Construction,
    style: Style,
    reach: Reach,
) -> &'static [(Material, u64)] {
    use Construction as C;
    use Material as M;
    match (construction, style) {
        (C::Plate, Style::Classical) => &[(M::Bronze, 1)],
        (C::Plate, Style::Elven) => &[(M::Steel, 50), (M::Mithril, 35), (M::Bronze, 15)],
        (C::Plate, Style::Dwarven) => &[(M::Steel, 50), (M::Iron, 25), (M::Adamantine, 25)],
        (C::Plate, Style::IndoPersian) => &[(M::Steel, 60), (M::Iron, 40)],
        (C::Plate, _) => &[
            (M::Steel, 55),
            (M::Iron, 35),
            (M::Mithril, 4),
            (M::Adamantine, 3),
            (M::Chitin, 3),
        ],
        // Soft leather only for bracers.
        (C::Leather, _) if reach == Reach::Wrist => &[(M::Leather, 60), (M::HardenedLeather, 40)],
        (C::Leather, _) => &[(M::HardenedLeather, 1)],
        (C::Mail, Style::Elven) => &[(M::Mithril, 50), (M::Steel, 50)],
        (C::Mail, Style::Japanese) => &[(M::Iron, 1)],
        (C::Mail, _) => &[
            (M::Iron, 50),
            (M::Steel, 40),
            (M::Bronze, 5),
            (M::Mithril, 4),
            (M::Adamantine, 1),
        ],
        (C::RingMail, _) => &[(M::Iron, 50), (M::Steel, 30), (M::Brass, 20)],
        (C::Scale, Style::Classical) => &[(M::Bronze, 60), (M::Iron, 40)],
        (C::Scale, Style::Elven) => &[(M::Steel, 40), (M::Mithril, 30), (M::DragonScale, 30)],
        (C::Scale, Style::Chinese) => &[
            (M::Iron, 50),
            (M::Steel, 20),
            (M::HardenedLeather, 20),
            (M::Horn, 10),
        ],
        (C::Scale, _) => &[
            (M::Iron, 45),
            (M::Steel, 25),
            (M::HardenedLeather, 15),
            (M::Horn, 5),
            (M::DragonScale, 5),
            (M::Chitin, 5),
        ],
        (C::Lamellar, Style::Steppe) => &[(M::Iron, 50), (M::HardenedLeather, 40), (M::Horn, 10)],
        (C::Lamellar, _) => &[
            (M::Iron, 50),
            (M::Steel, 20),
            (M::HardenedLeather, 20),
            (M::Horn, 10),
        ],
        (C::Laminar, Style::Classical) => &[(M::Iron, 70), (M::Bronze, 30)],
        (C::Laminar, _) => &[(M::Iron, 60), (M::HardenedLeather, 40)],
        (C::PlatesInCloth | C::MailAndPlate, _) => &[(M::Iron, 60), (M::Steel, 40)],
        (C::Splint, _) => &[
            (M::Iron, 40),
            (M::Steel, 25),
            (M::HardenedLeather, 20),
            (M::Wood, 5),
            (M::Bone, 5),
            (M::Horn, 5),
        ],
        (C::Padded, Style::Japanese) => &[(M::Silk, 40), (M::Cotton, 30), (M::Linen, 30)],
        _ => &[(M::Linen, 60), (M::Cotton, 25), (M::Wool, 15)],
    }
}

/// Whether a list has this.
pub(super) fn has<T: PartialEq>(list: &[(T, u64)], wanted: T) -> bool {
    list.iter()
        .any(|(have, weight)| *have == wanted && *weight > 0)
}

/// Whether a construction is built on cloth that may be dyed: padding, brigandine,
/// and Japanese kote and mail sleeves.
pub(super) fn has_cover(construction: Construction, style: Style) -> bool {
    match construction {
        Construction::Padded | Construction::PlatesInCloth => true,
        Construction::Mail | Construction::MailAndPlate => style == Style::Japanese,
        _ => false,
    }
}

/// Whether a piece may be on one arm only: an archer's bracer, or a manica on the
/// sword arm (the owner's decision).
pub(super) fn may_be_one_arm(construction: Construction, style: Style, reach: Reach) -> bool {
    match construction {
        Construction::Leather => reach == Reach::Wrist,
        Construction::Laminar => style == Style::Classical,
        _ => false,
    }
}

/// Whether a construction has a cup on the elbow when it reaches it. Bazubands rise
/// to their own point instead.
pub(super) fn has_couter(construction: Construction, style: Style, reach: Reach) -> bool {
    reach.elbow()
        && matches!(
            construction,
            Construction::Plate | Construction::Leather | Construction::Splint
        )
        && style != Style::IndoPersian
}

/// Whether a piece is made of tubes that may be closed or open: plate and leather
/// past the wrist.
pub(super) fn has_closure(construction: Construction, reach: Reach) -> bool {
    matches!(construction, Construction::Plate | Construction::Leather) && reach != Reach::Wrist
}

/// Whether tubes are always closed in this style.
pub(super) fn always_closed(style: Style) -> bool {
    matches!(style, Style::Elven | Style::Dwarven | Style::Classical)
}
