//! Armour for the hands, fingers included: gauntlets of plate, mail, scale, leather and
//! plates in cloth, demi-gauntlets and tekkō on the back of the hand only, and padded
//! gloves worn under them. What a piece is like is rolled once and saved; how big a
//! hand is comes from the body frame, so the same piece is drawn alone and worn. See
//! `docs/context/shared/armour.md`.

use serde::{Deserialize, Serialize};

use super::{Construction, Style};
use crate::body::{Frame, Joint, Side};
use crate::creature::Size;
use crate::material::Material;
use crate::rng::Rng;

/// What kind of hand piece an item is, which says the layer and places it takes and so
/// what it may be rolled as.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HandsPiece {
    /// Outer, over the hands and fingers.
    Gauntlets,
    /// Outer, over the back of the hand and wrist only, so rings stay free.
    Demi,
    /// Middle, under gauntlets.
    Padded,
}

/// How the fingers are covered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Fingers {
    /// Each finger on its own.
    Fingered,
    /// All but the thumb together, under one set of plates or one bag of mail.
    Mitten,
    /// Not at all: the back of the hand and the wrist only.
    Open,
}

/// The part over the wrist and up the forearm, over the end of any vambrace.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Cuff {
    /// A band round the wrist.
    Wrist,
    /// One plate narrowing at the wrist and flaring out: 14th century.
    Hourglass,
    /// Flared like a bell.
    Bell,
    /// A bell drawn out to a point on the outside of the forearm.
    Pointed,
    /// To the middle of the forearm.
    Long,
}

impl Cuff {
    /// How far up the forearm it reaches from the wrist, in millimetres on this frame
    /// (agreed with the arms' pieces, which it goes over).
    pub fn reach(self, frame: &Frame) -> i32 {
        let human = match self {
            Self::Wrist => 50,
            Self::Hourglass => 100,
            Self::Bell => 110,
            Self::Pointed => 140,
            Self::Long => 200,
        };
        human * frame.wrist_length / 260
    }

    /// How much wider than the wrist its open end is, out of 100.
    pub fn flare(self) -> i32 {
        match self {
            Self::Wrist => 0,
            Self::Hourglass => 45,
            Self::Bell | Self::Pointed => 55,
            Self::Long => 30,
        }
    }
}

/// What is over the knuckles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Knuckles {
    Plain,
    /// A raised bar across them.
    Ridged,
    /// A spike on each one (gadlings), or studs on leather.
    Spiked,
    /// A disc on the back of the hand, as on a jousting manifer.
    Rondel,
}

/// A pair of hand pieces: everything about them that is rolled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HandsForm {
    pub construction: Construction,
    pub fingers: Fingers,
    pub cuff: Cuff,
    pub knuckles: Knuckles,
    /// Claws on the fingertips (fantasy, after the tekkō-kagi).
    pub claws: bool,
    /// The fingers close and lock on a grip, for tournaments: plate mittens only.
    pub locking: bool,
    /// What it is mostly made of: the plates, rings, scales, leather or padding.
    pub material: Material,
    /// Cloth over the plates (plates in cloth) or under them (a tekkō's base).
    pub cloth: Option<Material>,
    /// Edges of another metal.
    pub trim: Option<Material>,
    pub style: Style,
}

/// The material its main weight is worked out against, in tenths of a gram per cm³.
const STEEL_DENSITY: u32 = 78;
const LEATHER_DENSITY: u32 = 9;
const LINEN_DENSITY: u32 = 15;

impl HandsForm {
    /// Every hand piece with these choices: the grid a roll keeps to.
    pub fn allowed(&self, piece: HandsPiece) -> bool {
        let in_list =
            |list: &[(Construction, u64)]| list.iter().any(|&(c, _)| c == self.construction);
        in_list(constructions(piece))
            && fingers_for(self.construction, piece, self.style)
                .iter()
                .any(|&(f, _)| f == self.fingers)
            && cuffs_for(self.construction, self.fingers, self.style)
                .iter()
                .any(|&(c, _)| c == self.cuff)
            && knuckles_for(self.construction)
                .iter()
                .any(|&(k, _)| k == self.knuckles)
            && (!self.claws || may_have_claws(self.construction, self.fingers))
            && (!self.locking || may_lock(self.construction, self.fingers))
            && materials_for(self.construction)
                .iter()
                .any(|&(m, _)| m == self.material)
            && self.cloth.is_some() == needs_cloth(self.construction, self.fingers)
            && (self.trim.is_none() || may_have_trim(self.construction, self.material))
    }

    /// What anything like it is called.
    pub fn generic_name(&self) -> &'static str {
        match (self.construction, self.fingers) {
            (Construction::Padded, _) => "Padded Gloves",
            (_, Fingers::Fingered) => "Gauntlets",
            (_, Fingers::Mitten) => "Mitten Gauntlets",
            (_, Fingers::Open) => "Demi-gauntlets",
        }
    }

    /// Its own name, when it fits one.
    pub fn specific_name(&self) -> Option<&'static str> {
        use Construction as C;
        let plate = self.construction == C::Plate;
        let metal = self.material.category() == crate::material::Category::Metal;
        Some(match (self.construction, self.fingers) {
            _ if self.claws => "Clawed Gauntlets",
            (C::Leather, _) if self.knuckles == Knuckles::Spiked => "Studded Leather Gauntlets",
            _ if self.knuckles == Knuckles::Spiked && metal => "Spiked Gauntlets",
            (C::Plate, Fingers::Mitten) if self.locking => "Locking Gauntlets",
            (C::Plate, Fingers::Mitten)
                if self.style == Style::Kastenbrust && self.cuff == Cuff::Long =>
            {
                "Kastenbrust Gauntlets"
            }
            _ if plate && self.style == Style::Gothic && self.fingers != Fingers::Open => {
                "Gothic Gauntlets"
            }
            (C::Plate, Fingers::Fingered) if self.cuff == Cuff::Hourglass => "Hourglass Gauntlets",
            (C::PlatesInCloth, _) => "Brigandine Gauntlets",
            (C::Mail, Fingers::Mitten) => "Mail Mittens",
            (C::Mail, _) => "Mail Gloves",
            (C::Scale, Fingers::Mitten) => "Scale Mittens",
            (C::Scale, _) => "Scale Gloves",
            (C::MailAndPlate, Fingers::Open) => "Tekkō",
            (C::MailAndPlate, _) => "Mail-and-plate Gauntlets",
            (C::Leather, _) => "Leather Gauntlets",
            _ => return None,
        })
    }

    /// What it is called.
    pub fn name(&self) -> &'static str {
        self.specific_name().unwrap_or(self.generic_name())
    }

    /// What the pair weighs at an ordinary size, in grams: its main material's share
    /// by that material's density, then backing, straps and extras. Worked out from
    /// area × thickness × density (see the research in `docs/context/research-armour-hands.md`).
    pub fn grams(&self) -> u32 {
        use Construction as C;
        // Grams at steel's density, over the hand and over the cuff.
        let hand = match self.fingers {
            Fingers::Fingered => 320,
            Fingers::Mitten => 280,
            Fingers::Open => 150,
        };
        let cuff = match self.cuff {
            Cuff::Wrist => 120,
            Cuff::Hourglass => 360,
            Cuff::Bell => 380,
            Cuff::Pointed => 430,
            Cuff::Long => 650,
        };
        let cuff = if self.fingers == Fingers::Open {
            cuff * 3 / 5
        } else {
            cuff
        };
        // Main weight at its reference density, the reference, and everything else.
        let (main, reference, rest) = match self.construction {
            C::Plate => {
                let backing = if self.fingers == Fingers::Open {
                    70
                } else {
                    180
                };
                (hand + cuff, STEEL_DENSITY, backing)
            }
            C::PlatesInCloth => ((hand + cuff) * 9 / 10, STEEL_DENSITY, 260),
            C::Mail => (hand * 2 + cuff - 80, STEEL_DENSITY, 60),
            C::Scale => ((hand + cuff) * 9 / 10, STEEL_DENSITY, 200),
            C::MailAndPlate if self.fingers == Fingers::Open => (150, STEEL_DENSITY, 40),
            C::MailAndPlate => (hand * 3 / 5 + 400 + cuff, STEEL_DENSITY, 60),
            C::Leather => (200 + cuff * 3 / 7, LEATHER_DENSITY, 0),
            _ => (150, LINEN_DENSITY, 0),
        };
        let extras = match self.knuckles {
            Knuckles::Plain | Knuckles::Ridged => 0,
            Knuckles::Spiked => 40,
            Knuckles::Rondel => 60,
        } + if self.claws { 80 } else { 0 }
            + if self.locking { 100 } else { 0 };
        main * self.material.density() / reference + rest + extras
    }

    /// What the pair weighs made at this size: a bigger hand in every direction.
    pub fn grams_at(&self, size: Size) -> u32 {
        let quarters = size.quarters() as u64;
        (self.grams() as u64 * quarters * quarters * quarters / 64) as u32
    }
}

/// Which constructions a kind of piece is made in, and how often.
fn constructions(piece: HandsPiece) -> &'static [(Construction, u64)] {
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
fn fingers_for(
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

fn cuffs_for(construction: Construction, fingers: Fingers, style: Style) -> &'static [(Cuff, u64)] {
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

fn knuckles_for(construction: Construction) -> &'static [(Knuckles, u64)] {
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

fn may_have_claws(construction: Construction, fingers: Fingers) -> bool {
    use Construction as C;
    fingers == Fingers::Fingered && matches!(construction, C::Plate | C::Scale | C::MailAndPlate)
}

fn may_lock(construction: Construction, fingers: Fingers) -> bool {
    construction == Construction::Plate && fingers == Fingers::Mitten
}

fn materials_for(construction: Construction) -> &'static [(Material, u64)] {
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

fn needs_cloth(construction: Construction, fingers: Fingers) -> bool {
    construction == Construction::PlatesInCloth
        || (construction == Construction::MailAndPlate && fingers == Fingers::Open)
}

fn cloths_for(construction: Construction) -> &'static [(Material, u64)] {
    use Material as M;
    match construction {
        Construction::MailAndPlate => &[(M::Silk, 50), (M::Linen, 50)],
        _ => &[(M::Linen, 40), (M::Wool, 35), (M::Silk, 10)],
    }
}

fn may_have_trim(construction: Construction, material: Material) -> bool {
    construction == Construction::Plate
        && matches!(
            material,
            Material::Steel | Material::Iron | Material::Mithril
        )
}

/// Rolls a piece of this kind. A set passes its style; a piece on its own rolls one.
/// Each choice comes from what the ones before allow, so nothing is thrown away.
pub fn roll(rng: &mut Rng, piece: HandsPiece, style: Option<Style>) -> HandsForm {
    let style = style.unwrap_or_else(|| Style::roll(rng));
    let construction = rng.pick_weighted(constructions(piece));
    let fingers = rng.pick_weighted(fingers_for(construction, piece, style));
    let cuff = rng.pick_weighted(cuffs_for(construction, fingers, style));
    let knuckles = rng.pick_weighted(knuckles_for(construction));
    let claws = may_have_claws(construction, fingers) && rng.range_u64(0, 12) == 0;
    let locking = may_lock(construction, fingers) && rng.range_u64(0, 8) == 0;
    let material = rng.pick_weighted(materials_for(construction));
    let cloth =
        needs_cloth(construction, fingers).then(|| rng.pick_weighted(cloths_for(construction)));
    let trim = (may_have_trim(construction, material) && rng.range_u64(0, 6) == 0)
        .then_some(Material::Brass);
    HandsForm {
        construction,
        fingers,
        cuff,
        knuckles,
        claws,
        locking,
        material,
        cloth,
        trim,
        style,
    }
}

/// One hand on a body frame, in millimetres, worked out from the frame's hand length
/// as artists proportion it: what hand pieces are built on. It hangs from the wrist
/// with the palm towards the thigh, the back of the hand outwards and the thumb forwards.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hand {
    pub side: Side,
    /// The middle of the wrist.
    pub wrist: [i32; 3],
    /// The tip of the longest finger.
    pub tip: [i32; 3],
    /// From the wrist to the tip.
    pub length: i32,
    /// From the wrist to the knuckles.
    pub palm: i32,
    /// Across the knuckles, front to back.
    pub breadth: i32,
    /// From palm to back.
    pub thickness: i32,
    /// The thumb's length, and how far below the wrist it starts.
    pub thumb: i32,
    pub thumb_from: i32,
    /// Fingers other than the thumb.
    pub fingers: u8,
    /// The forearm's radius at the wrist, and how long the forearm is.
    pub wrist_radius: i32,
    pub forearm_length: i32,
}

impl Hand {
    /// This side's hand on this frame, with this many digits, thumb included (half the
    /// fingers its owner has; at least a thumb and one finger).
    pub fn of(frame: &Frame, side: Side, digits: u8) -> Self {
        let length = frame.hand_length;
        Self {
            side,
            wrist: frame.joint(Joint::Wrist(side)),
            tip: frame.joint(Joint::Fingertips(side)),
            length,
            palm: length * 55 / 100,
            breadth: length * 45 / 100,
            thickness: length * 16 / 100,
            thumb: length * 33 / 100,
            thumb_from: length * 15 / 100,
            fingers: digits.max(2) - 1,
            // As the bare body draws its wrist.
            wrist_radius: frame.forearm * 7 / 10,
            forearm_length: frame.wrist_length,
        }
    }

    /// The digits on each hand of a body with this many fingers in all.
    pub fn digits(fingers: Option<u8>) -> u8 {
        fingers.map_or(5, |fingers| fingers / 2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creature::CreatureKind;

    const PIECES: [HandsPiece; 3] = [HandsPiece::Gauntlets, HandsPiece::Demi, HandsPiece::Padded];

    fn rolled(piece: HandsPiece) -> impl Iterator<Item = HandsForm> {
        (0..2000u64).map(move |seed| {
            let mut rng = crate::rng::Seed::new(seed).derive("hands").rng();
            roll(&mut rng, piece, None)
        })
    }

    #[test]
    fn every_roll_keeps_to_its_grid() {
        for piece in PIECES {
            for form in rolled(piece) {
                assert!(form.allowed(piece), "{piece:?} {form:?}");
            }
        }
    }

    #[test]
    fn each_kind_rolls_only_what_it_covers() {
        assert!(
            rolled(HandsPiece::Gauntlets)
                .all(|form| form.fingers != Fingers::Open
                    && form.construction != Construction::Padded)
        );
        assert!(rolled(HandsPiece::Demi).all(|form| form.fingers == Fingers::Open));
        assert!(rolled(HandsPiece::Padded).all(|form| form.construction == Construction::Padded));
    }

    #[test]
    fn every_option_turns_up() {
        let all: Vec<_> = PIECES.into_iter().flat_map(rolled).collect();
        for cuff in [
            Cuff::Wrist,
            Cuff::Hourglass,
            Cuff::Bell,
            Cuff::Pointed,
            Cuff::Long,
        ] {
            assert!(all.iter().any(|form| form.cuff == cuff), "{cuff:?}");
        }
        for knuckles in [
            Knuckles::Plain,
            Knuckles::Ridged,
            Knuckles::Spiked,
            Knuckles::Rondel,
        ] {
            assert!(
                all.iter().any(|form| form.knuckles == knuckles),
                "{knuckles:?}"
            );
        }
        assert!(all.iter().any(|form| form.claws));
        assert!(all.iter().any(|form| form.locking));
        assert!(all.iter().any(|form| form.trim.is_some()));
        for name in [
            "Hourglass Gauntlets",
            "Tekkō",
            "Mail Mittens",
            "Padded Gloves",
        ] {
            assert!(all.iter().any(|form| form.name() == name), "{name}");
        }
    }

    #[test]
    fn a_style_narrows_plate() {
        let rolled = |style| {
            (0..400u64).map(move |seed| {
                let mut rng = crate::rng::Seed::new(seed).rng();
                roll(&mut rng, HandsPiece::Gauntlets, Some(style))
            })
        };
        let plate = |form: &HandsForm| form.construction == Construction::Plate;
        assert!(
            rolled(Style::Kastenbrust)
                .filter(plate)
                .all(|form| form.fingers == Fingers::Mitten
                    && form.cuff == Cuff::Long
                    && form.style == Style::Kastenbrust)
        );
        assert!(
            rolled(Style::Gothic)
                .filter(plate)
                .all(|form| form.cuff == Cuff::Pointed)
        );
        assert!(
            rolled(Style::Maximilian)
                .filter(plate)
                .all(|form| form.fingers == Fingers::Fingered)
        );
    }

    #[test]
    fn weights_match_the_worked_out_figures() {
        let plate = HandsForm {
            construction: Construction::Plate,
            fingers: Fingers::Fingered,
            cuff: Cuff::Hourglass,
            knuckles: Knuckles::Plain,
            claws: false,
            locking: false,
            material: Material::Steel,
            cloth: None,
            trim: None,
            style: Style::White,
        };
        assert_eq!(plate.grams(), 860);
        let long = HandsForm {
            cuff: Cuff::Long,
            ..plate
        };
        assert_eq!(long.grams(), 1150);
        let mithril = HandsForm {
            material: Material::Mithril,
            ..plate
        };
        assert!(mithril.grams() < plate.grams() * 2 / 3);
        let mail = HandsForm {
            construction: Construction::Mail,
            fingers: Fingers::Mitten,
            cuff: Cuff::Wrist,
            ..plate
        };
        assert!((600..=700).contains(&mail.grams()), "{}", mail.grams());
        for piece in PIECES {
            for form in rolled(piece) {
                assert!((100..=1500).contains(&form.grams()), "{form:?}");
            }
        }
        assert!(plate.grams_at(Size::Large) > plate.grams());
    }

    #[test]
    fn hands_hang_from_the_wrist_and_fit_their_frame() {
        for kind in CreatureKind::ALL {
            let Some(frame) = Frame::of(kind) else {
                continue;
            };
            for side in Side::BOTH {
                let hand = Hand::of(&frame, side, Hand::digits(kind.def().fingers()));
                assert_eq!(hand.wrist[1] - hand.tip[1], hand.length);
                assert!(hand.palm < hand.length && hand.breadth > hand.thickness);
                assert!(hand.fingers >= 1);
            }
        }
        let frame = Frame::of(CreatureKind::Kobold).unwrap();
        assert_eq!(
            Hand::of(&frame, Side::Left, Hand::digits(Some(8))).fingers,
            3
        );
    }
}
