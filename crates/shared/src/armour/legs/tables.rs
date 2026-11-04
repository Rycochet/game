//! What leg pieces may be made of and how much of the leg they cover, and how often:
//! the lists rolls pick from and the grid checks against.

use super::{Cover, LegsKind};
use crate::armour::{Construction, Style};

/// Constructions leg armour is made in, by style. Each keeps to `Style::allows`
/// (checked by a test), narrowed to what was worn on the legs.
pub(super) fn constructions(kind: LegsKind, style: Style) -> &'static [(Construction, u64)] {
    use Construction as C;
    match kind {
        LegsKind::Chausses => return &[(C::Mail, 1)],
        LegsKind::Padded => return &[(C::Padded, 1)],
        LegsKind::Armour => {}
    }
    match style {
        Style::Elven => &[
            (C::Plate, 45),
            (C::Leather, 25),
            (C::Scale, 15),
            (C::Mail, 15),
        ],
        Style::Dwarven => &[
            (C::Plate, 60),
            (C::Mail, 15),
            (C::Leather, 15),
            (C::Splint, 10),
        ],
        // Suneate and haidate.
        Style::Japanese => &[
            (C::Splint, 35),
            (C::MailAndPlate, 20),
            (C::PlatesInCloth, 20),
            (C::Mail, 10),
            (C::Lamellar, 10),
            (C::Plate, 5),
        ],
        Style::Chinese => &[
            (C::Lamellar, 50),
            (C::Scale, 25),
            (C::PlatesInCloth, 15),
            (C::Leather, 10),
        ],
        Style::Steppe => &[
            (C::Lamellar, 35),
            (C::Laminar, 25),
            (C::Scale, 20),
            (C::Mail, 20),
        ],
        // Greaves, ankle and thigh guards of bronze, the leg manica, scale leggings.
        Style::Classical => &[(C::Plate, 65), (C::Laminar, 20), (C::Scale, 15)],
        Style::Byzantine => &[
            (C::Splint, 40),
            (C::Lamellar, 25),
            (C::Mail, 25),
            (C::Leather, 10),
        ],
        // The dizçek.
        Style::IndoPersian => &[
            (C::MailAndPlate, 50),
            (C::Mail, 25),
            (C::Plate, 15),
            (C::Lamellar, 10),
        ],
        // The European harnesses.
        _ => &[
            (C::Plate, 50),
            (C::Mail, 18),
            (C::Leather, 10),
            (C::Splint, 8),
            (C::PlatesInCloth, 5),
            (C::Padded, 5),
            (C::RingMail, 4),
        ],
    }
}

/// How much of the leg a piece of this kind covers: chausses and padded chausses the
/// whole leg, other pieces as their construction does.
pub(super) fn covers_for(
    kind: LegsKind,
    construction: Construction,
    style: Style,
) -> &'static [(Cover, u64)] {
    match kind {
        LegsKind::Armour => covers(construction, style),
        _ => &[(Cover::Whole, 1)],
    }
}

/// How much of the leg a construction covers, in a style.
fn covers(construction: Construction, style: Style) -> &'static [(Cover, u64)] {
    use Construction as C;
    use Cover as V;
    match (construction, style) {
        // Knemides, ocreae and ring greaves, perimeridia, episphyria.
        (C::Plate, Style::Classical) => &[
            (V::KneeAndShin, 40),
            (V::Shin, 35),
            (V::Thigh, 15),
            (V::Ankle, 10),
        ],
        // Tsutsu-suneate.
        (C::Plate, Style::Japanese) => &[(V::KneeAndShin, 1)],
        // Knee cops and greaves worn with mail.
        (C::Plate, Style::IndoPersian) => &[(V::Knee, 40), (V::KneeAndShin, 60)],
        (C::Plate, _) => &[
            (V::Whole, 35),
            (V::KneeAndShin, 15),
            (V::Shin, 15),
            (V::ThighAndKnee, 15),
            (V::Knee, 12),
            (V::Thigh, 8),
        ],
        (C::Leather, _) => &[
            (V::Shin, 30),
            (V::Knee, 20),
            (V::KneeAndShin, 15),
            (V::ThighAndKnee, 15),
            (V::Thigh, 10),
            (V::Whole, 10),
        ],
        // Kusari suneate and haidate.
        (C::Mail, Style::Japanese) => &[(V::Shin, 60), (V::Thigh, 40)],
        (C::Mail, _) => &[(V::Whole, 1)],
        (C::RingMail, _) => &[(V::Whole, 70), (V::Thigh, 30)],
        // Scale demi-chausses, as found at Dura Europos.
        (C::Scale, Style::Classical) => &[(V::Whole, 1)],
        (C::Scale, _) => &[
            (V::Shin, 30),
            (V::Thigh, 30),
            (V::KneeAndShin, 20),
            (V::Whole, 20),
        ],
        // Haidate of laced plates.
        (C::Lamellar, Style::Japanese) => &[(V::Thigh, 60), (V::ThighAndKnee, 40)],
        (C::Lamellar, _) => &[
            (V::Thigh, 35),
            (V::Shin, 30),
            (V::KneeAndShin, 15),
            (V::ThighAndKnee, 10),
            (V::Whole, 10),
        ],
        // The leg manica, and Steppe hooped greaves.
        (C::Laminar, Style::Steppe) => &[(V::Whole, 60), (V::Shin, 40)],
        (C::Laminar, _) => &[(V::Whole, 1)],
        // Kikko and karuta haidate, kikko suneate.
        (C::PlatesInCloth, Style::Japanese) => {
            &[(V::Thigh, 60), (V::ThighAndKnee, 20), (V::Shin, 20)]
        }
        (C::PlatesInCloth, _) => &[(V::ThighAndKnee, 40), (V::Thigh, 30), (V::Whole, 30)],
        (C::MailAndPlate, Style::IndoPersian) => &[(V::ThighAndKnee, 70), (V::Whole, 30)],
        // Shino suneate of splints and mail, and haidate of small plates.
        (C::MailAndPlate, Style::Japanese) => &[(V::KneeAndShin, 50), (V::Thigh, 50)],
        (C::MailAndPlate, _) => &[(V::ThighAndKnee, 60), (V::Whole, 40)],
        (C::Splint, Style::Japanese) => &[(V::KneeAndShin, 60), (V::Shin, 40)],
        (C::Splint, _) => &[
            (V::Shin, 30),
            (V::KneeAndShin, 30),
            (V::Whole, 25),
            (V::ThighAndKnee, 15),
        ],
        // Gamboised cuisses; padded chausses are Whole.
        _ => &[(V::Thigh, 50), (V::ThighAndKnee, 50)],
    }
}

/// Whether a list has this.
pub(super) fn has<T: PartialEq>(list: &[(T, u64)], wanted: T) -> bool {
    list.iter()
        .any(|(have, weight)| *have == wanted && *weight > 0)
}

/// Whether a piece is built on cloth that may be dyed: padding, brigandine, everything
/// Japanese, and the quilted thighs over reinforced chausses.
pub(super) fn has_cloth(construction: Construction, style: Style) -> bool {
    matches!(
        construction,
        Construction::Padded | Construction::PlatesInCloth
    ) || style == Style::Japanese
}

/// How a piece may go round the leg.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Wrap {
    Closed,
    Front,
    Either,
}

/// Whether a piece goes round the leg, over its front only, or may be either.
pub(super) fn wrap(construction: Construction, style: Style, cover: Cover) -> Wrap {
    use Construction as C;
    match (construction, style, cover) {
        (_, _, Cover::Knee | Cover::Ankle) => Wrap::Closed,
        (C::Padded | C::Laminar | C::RingMail, _, _) => Wrap::Closed,
        // Tsutsu-suneate are hinged round; the rest of Japan's are tied on the front.
        (C::Plate, Style::Japanese, _) => Wrap::Closed,
        (_, Style::Japanese, _) => Wrap::Front,
        (C::Mail, _, Cover::Whole) => Wrap::Either,
        (C::Mail, _, _) => Wrap::Front,
        // Knemides sprung shut; ocreae over the front, or ring greaves round; perimeridia
        // and scale demi-chausses over the front.
        (C::Plate, Style::Classical, Cover::KneeAndShin) => Wrap::Closed,
        (C::Plate, Style::Classical, Cover::Shin) => Wrap::Either,
        (_, Style::Classical, _) => Wrap::Front,
        (C::MailAndPlate, Style::IndoPersian, Cover::ThighAndKnee) => Wrap::Front,
        (_, Style::Elven | Style::Dwarven, _) => Wrap::Closed,
        (C::Plate | C::Leather | C::Splint | C::Scale | C::Lamellar, _, _) => Wrap::Either,
        (C::PlatesInCloth | C::MailAndPlate, _, _) => Wrap::Either,
        _ => Wrap::Closed,
    }
}

/// What a construction has over the knee, when it reaches it.
pub(super) fn knees(
    kind: LegsKind,
    construction: Construction,
    style: Style,
    cover: Cover,
) -> &'static [(super::Knee, u64)] {
    use super::Knee as K;
    use Construction as C;
    // Padded chausses go under other things, so have nothing on the knee.
    if !cover.knee() || kind == LegsKind::Padded {
        return &[(K::None, 1)];
    }
    match (construction, style) {
        (C::Plate | C::Splint | C::MailAndPlate | C::PlatesInCloth, Style::Japanese)
            if matches!(cover, Cover::KneeAndShin | Cover::Whole) =>
        {
            &[(K::Tall, 1)]
        }
        (_, Style::Japanese) => &[(K::None, 1)],
        (C::Plate, Style::Gothic | Style::Milanese) => &[(K::Winged, 80), (K::Cop, 20)],
        (C::Plate, Style::Classical | Style::IndoPersian) => &[(K::Cop, 1)],
        (C::Plate, _) => &[(K::Winged, 50), (K::Cop, 50)],
        (C::Leather, _) => &[(K::Cop, 80), (K::Winged, 20)],
        (C::Splint | C::MailAndPlate | C::PlatesInCloth | C::Padded, _) => &[(K::Cop, 1)],
        _ => &[(K::None, 1)],
    }
}

/// Whether mail chausses may have plates strapped over them: European and fantasy.
pub(super) fn may_reinforce(construction: Construction, style: Style, cover: Cover) -> bool {
    construction == Construction::Mail
        && cover == Cover::Whole
        && (style.european() || matches!(style, Style::Elven | Style::Dwarven))
}

/// Whether a piece may be for one leg only: a Classical greave (the owner's decision).
pub(super) fn may_be_one_leg(construction: Construction, style: Style, cover: Cover) -> bool {
    construction == Construction::Plate
        && style == Style::Classical
        && matches!(cover, Cover::Shin | Cover::KneeAndShin)
}

/// Whether a piece may have fantasy spikes or fins: plate and leather outside the
/// regional styles.
pub(super) fn may_ornament(construction: Construction, style: Style) -> bool {
    matches!(construction, Construction::Plate | Construction::Leather)
        && (style.european() || matches!(style, Style::Elven | Style::Dwarven))
}
