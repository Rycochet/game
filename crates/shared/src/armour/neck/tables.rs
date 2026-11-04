//! What neck pieces may be made of and shaped like, and how often: the lists rolls
//! pick from and the grid checks against.

use super::{Edging, Front, Hem, NeckKind, Plates};
use crate::armour::{Construction, Style};

/// Constructions a kind of neck piece is made in, by style. Each keeps to
/// `Style::allows` (checked by a test).
pub(super) fn constructions(kind: NeckKind, style: Style) -> &'static [(Construction, u64)] {
    use Construction as C;
    let european = style.european();
    match kind {
        NeckKind::Collar => match style {
            // The guruwa: a collar lined with small plates.
            Style::Japanese => &[(C::PlatesInCloth, 50), (C::Padded, 30), (C::Mail, 20)],
            _ if european => &[(C::Mail, 55), (C::Padded, 35), (C::RingMail, 10)],
            Style::Elven | Style::Dwarven => &[(C::Mail, 60), (C::Padded, 40)],
            _ => &[(C::Mail, 50), (C::Padded, 50)],
        },
        NeckKind::Gorget => match style {
            _ if european => &[(C::Plate, 75), (C::Leather, 15), (C::Scale, 10)],
            Style::Elven => &[(C::Plate, 70), (C::Leather, 15), (C::Scale, 15)],
            Style::Dwarven => &[(C::Plate, 80), (C::Leather, 20)],
            // The nodowa.
            Style::Japanese => &[(C::Laminar, 60), (C::Lamellar, 40)],
            // The tall bronze collars of Dendra and the Near East.
            Style::Classical => &[(C::Plate, 80), (C::Scale, 20)],
            Style::Chinese => &[(C::Leather, 60), (C::Scale, 40)],
            Style::Steppe => &[(C::Leather, 40), (C::Lamellar, 35), (C::Scale, 25)],
            Style::Byzantine => &[(C::Lamellar, 50), (C::Leather, 30), (C::Scale, 20)],
            _ => &[(C::Plate, 60), (C::Scale, 40)],
        },
        NeckKind::Bevor => match style {
            _ if european => &[(C::Plate, 90), (C::Leather, 10)],
            Style::Elven | Style::Dwarven => &[(C::Plate, 85), (C::Leather, 15)],
            Style::Japanese => &[(C::Plate, 60), (C::Leather, 40)],
            Style::Classical => &[(C::Plate, 70), (C::Leather, 30)],
            Style::IndoPersian => &[(C::Plate, 1)],
            _ => &[(C::Leather, 1)],
        },
        NeckKind::Aventail => match style {
            _ if european => &[(C::Mail, 85), (C::RingMail, 10), (C::Padded, 5)],
            Style::Elven => &[(C::Mail, 80), (C::Scale, 20)],
            Style::Dwarven => &[(C::Mail, 70), (C::Scale, 30)],
            Style::Japanese => &[(C::Mail, 40), (C::Lamellar, 40), (C::Padded, 20)],
            // Leather neck guards on helmets (Chinese armour).
            Style::Chinese => &[
                (C::Leather, 40),
                (C::Lamellar, 30),
                (C::Scale, 20),
                (C::Mail, 10),
            ],
            Style::Steppe => &[
                (C::Lamellar, 40),
                (C::Scale, 25),
                (C::Padded, 20),
                (C::Mail, 15),
            ],
            Style::Classical => &[(C::Mail, 50), (C::Scale, 50)],
            Style::Byzantine => &[(C::Mail, 50), (C::Lamellar, 40), (C::Padded, 10)],
            _ => &[(C::Mail, 70), (C::MailAndPlate, 30)],
        },
        NeckKind::Mantle => match style {
            Style::Japanese => &[(C::Mail, 1)],
            _ if european => &[(C::Mail, 85), (C::Scale, 15)],
            _ => &[(C::Mail, 60), (C::Scale, 40)],
        },
    }
}

/// How much a gorget covers, by construction and style; the collar alone for the rest.
pub(super) fn plates(
    kind: NeckKind,
    construction: Construction,
    style: Style,
) -> &'static [(Plates, u64)] {
    use Construction as C;
    use Plates as P;
    if kind != NeckKind::Gorget {
        return &[(P::Collar, 1)];
    }
    match construction {
        C::Plate if style == Style::Classical => &[(P::Collar, 60), (P::Plates, 40)],
        C::Plate if style == Style::IndoPersian => &[(P::Collar, 30), (P::Plates, 70)],
        C::Plate => &[(P::Collar, 15), (P::Plates, 60), (P::Shoulders, 25)],
        C::Leather => &[(P::Collar, 30), (P::Bib, 20), (P::Plates, 50)],
        C::Laminar | C::Lamellar => &[(P::Bib, 1)],
        _ => &[(P::Collar, 40), (P::Plates, 60)],
    }
}

/// Whether the collar is tall (or a bevor reaches the mouth): plate gorgets' standing
/// collars and shoulder pieces always, plate gorgets with plates mostly, the nodowa
/// always, and most bevors.
pub(super) fn highs(
    kind: NeckKind,
    construction: Construction,
    plates: Plates,
) -> &'static [(bool, u64)] {
    use Construction as C;
    match (kind, construction) {
        (NeckKind::Gorget, C::Plate) => match plates {
            Plates::Plates => &[(true, 65), (false, 35)],
            _ => &[(true, 1)],
        },
        (NeckKind::Gorget, C::Laminar | C::Lamellar) => &[(true, 1)],
        (NeckKind::Bevor, _) => &[(true, 65), (false, 35)],
        _ => &[(false, 1)],
    }
}

/// How many lames: (least, most).
pub(super) fn lames(
    kind: NeckKind,
    construction: Construction,
    plates: Plates,
    high: bool,
) -> (u8, u8) {
    use Construction as C;
    match (kind, construction) {
        (NeckKind::Gorget, C::Plate) if high && plates == Plates::Collar => (1, 2),
        (NeckKind::Gorget, C::Plate) if high => (2, 4),
        (NeckKind::Gorget, C::Laminar) => (2, 4),
        (NeckKind::Gorget, C::Lamellar) => (2, 3),
        (NeckKind::Gorget, _) => (1, 1),
        (NeckKind::Bevor, C::Plate) => (0, 3),
        _ => (0, 0),
    }
}

/// How an aventail's front is made, by construction and style; open for the rest.
pub(super) fn fronts(
    kind: NeckKind,
    construction: Construction,
    style: Style,
) -> &'static [(Front, u64)] {
    use Construction as C;
    use Front as F;
    if kind != NeckKind::Aventail {
        return &[(F::Open, 1)];
    }
    match construction {
        // Kulah khud aventails often came down over the face.
        C::Mail if style == Style::IndoPersian => &[(F::Open, 35), (F::Chin, 30), (F::Closed, 35)],
        C::Mail => &[
            (F::Open, 20),
            (F::Chin, 45),
            (F::Closed, 15),
            (F::Ventail, 20),
        ],
        C::MailAndPlate => &[(F::Chin, 50), (F::Closed, 50)],
        _ => &[(F::Open, 60), (F::Chin, 40)],
    }
}

/// Reaching over the shoulders: may for aventails and mail collars, always for mantles.
pub(super) fn capes(kind: NeckKind, construction: Construction) -> &'static [(bool, u64)] {
    match kind {
        NeckKind::Mantle => &[(true, 1)],
        NeckKind::Aventail => &[(true, 40), (false, 60)],
        NeckKind::Collar if mail(construction) => &[(true, 40), (false, 60)],
        _ => &[(false, 1)],
    }
}

fn mail(construction: Construction) -> bool {
    matches!(construction, Construction::Mail | Construction::RingMail)
}

/// The lower edges a hanging piece may have: mail zig-zag, scales and lamellae scalloped.
pub(super) fn hems(
    kind: NeckKind,
    construction: Construction,
    style: Style,
) -> &'static [(Hem, u64)] {
    use Construction as C;
    let hangs = matches!(
        kind,
        NeckKind::Collar | NeckKind::Aventail | NeckKind::Mantle
    );
    match construction {
        C::Mail if hangs && style == Style::IndoPersian => {
            &[(Hem::Vandyked, 60), (Hem::Straight, 40)]
        }
        C::Mail if hangs => &[(Hem::Straight, 65), (Hem::Vandyked, 35)],
        C::Scale | C::Lamellar if hangs => &[(Hem::Scalloped, 50), (Hem::Straight, 50)],
        _ => &[(Hem::Straight, 1)],
    }
}

/// Rings of another metal: only in riveted mail, patterned only Indo-Persian.
pub(super) fn edgings(construction: Construction, style: Style) -> &'static [(Edging, u64)] {
    match (construction, style) {
        (Construction::Mail, Style::IndoPersian) => &[
            (Edging::None, 40),
            (Edging::Brass, 20),
            (Edging::Patterned, 40),
        ],
        (Construction::Mail, Style::Japanese) => &[(Edging::None, 1)],
        (Construction::Mail, _) => &[(Edging::None, 70), (Edging::Brass, 30)],
        _ => &[(Edging::None, 1)],
    }
}

/// A bevor drawn to a point: Gothic always, elven often, others now and then.
pub(super) fn points(
    kind: NeckKind,
    construction: Construction,
    style: Style,
) -> &'static [(bool, u64)] {
    if kind != NeckKind::Bevor || construction != Construction::Plate {
        return &[(false, 1)];
    }
    match style {
        Style::Gothic => &[(true, 1)],
        Style::Elven => &[(true, 50), (false, 50)],
        _ => &[(true, 15), (false, 85)],
    }
}

/// Spikes round a gorget's collar: plate or leather, not over the shoulders.
pub(super) fn may_spike(kind: NeckKind, construction: Construction, plates: Plates) -> bool {
    kind == NeckKind::Gorget
        && matches!(construction, Construction::Plate | Construction::Leather)
        && matches!(plates, Plates::Collar | Plates::Plates)
}

/// Studs: leather gorgets, and quilted or leather aventails.
pub(super) fn may_stud(kind: NeckKind, construction: Construction) -> bool {
    matches!(
        (kind, construction),
        (NeckKind::Gorget, Construction::Leather)
            | (
                NeckKind::Aventail,
                Construction::Padded | Construction::Leather
            )
    )
}

/// Cloth: a padded piece's cover, the cloth plates are set in, Japanese lacing.
pub(super) fn has_cloth(construction: Construction, style: Style) -> bool {
    matches!(
        construction,
        Construction::Padded | Construction::PlatesInCloth
    ) || style == Style::Japanese
}

/// Whether a list offers this, at a weight above nothing.
pub(super) fn has<T: PartialEq>(list: &[(T, u64)], wanted: T) -> bool {
    list.iter()
        .any(|(have, weight)| *have == wanted && *weight > 0)
}
